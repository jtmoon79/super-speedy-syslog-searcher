//! Managed OneDrive Log input, optional companion discovery, and chronological
//! event ordering. Parsing is streaming; accepted rendered events are buffered
//! for sorting, just as in the native ETL reader.

use std::collections::BTreeMap;
use std::io::{
    self,
    Error,
    ErrorKind,
    Read,
};
use std::path::{
    Path,
    PathBuf,
};
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

use tempfile::TempPath;

use crate::common::{
    Count,
    FPath,
    FileSz,
    FileType,
    LogMessageType,
    PathId,
    SUBPATH_SEP,
    summary_stat,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    Result_Filter_DateTime2,
    SystemTime,
    dt_pass_filters,
};
use crate::data::odl::Odl;
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::filehandlemanager::{
    FILE_HANDLE_MANAGER,
    FileHandleManaged,
    FileHandleRole,
    OpenOptionsManaged,
};
use crate::readers::helpers::path_to_fpath;
use crate::readers::odlparser::{
    OdlDecodingContext,
    OdlParser,
    OdlRecordError,
};
use crate::readers::summary::Summary;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct SummaryOdlReader {
    pub odlreader_events_processed: Count,
    pub odlreader_events_accepted: Count,
    pub odlreader_event_largest_processed: Count,
    pub odlreader_event_largest_accepted: Count,
    pub odlreader_datetime_first_processed: DateTimeLOpt,
    pub odlreader_datetime_last_processed: DateTimeLOpt,
    pub odlreader_datetime_first_accepted: DateTimeLOpt,
    pub odlreader_datetime_last_accepted: DateTimeLOpt,
    pub odlreader_filesz: FileSz,
    pub odlreader_out_of_order: Count,
    pub odlreader_records_skipped: Count,
    pub odlreader_events_undecoded: Count,
    pub odlreader_decoding_failures: Count,
    pub odlreader_version: u32,
    pub odlreader_compressed: bool,
    pub odlreader_companions_available: bool,
    /// Successfully loaded supplementary files, including archive member paths.
    pub odlreader_supplementary_files_used: Vec<FPath>,
    /// Supplementary files searched for but missing or inaccessible.
    pub odlreader_supplementary_files_notfound: Vec<FPath>,
}

/// Metadata supplied explicitly for a stream that has no filesystem path.
pub struct OdlSource {
    pub path_id: PathId,
    pub path: FPath,
    pub filetype: FileType,
    pub filesz: FileSz,
    pub mtime: SystemTime,
    pub fixed_offset: FixedOffset,
}

pub struct OdlReader<R: Read = FileHandleManaged> {
    source: OdlSource,
    reader: Option<R>,
    decoding: Option<OdlDecodingContext>,
    events: BTreeMap<(DateTimeL, u64), Odl>,
    named_temp_file: Option<TempPath>,
    statistics: SummaryOdlReader,
    analyzed: bool,
    error: Option<String>,
}

impl OdlReader<FileHandleManaged> {
    pub fn new(
        path_id: PathId,
        path: FPath,
        filetype: FileType,
        fixed_offset: FixedOffset,
    ) -> io::Result<Self> {
        if !filetype.is_odl() {
            return Err(Error::new(ErrorKind::InvalidInput, "OdlReader requires FileType::Odl"));
        }
        // Load companions before acquiring the long-lived primary input handle.
        let companions = load_companions(path_id, &path, filetype)?;
        let decompressed = decompress_to_ntf(path_id, Path::new(&path), &filetype)?;
        let (named_temp_file, original_mtime) = match decompressed {
            Some((temporary, mtime, _)) => (Some(temporary), mtime),
            None => (None, None),
        };
        let actual = named_temp_file
            .as_ref()
            .map(|p| p.as_ref())
            .unwrap_or(Path::new(&path));
        let file = FILE_HANDLE_MANAGER.request_open_managed(
            path_id,
            FileHandleRole::PrimaryRead,
            actual,
            OpenOptionsManaged::read_only(),
        )?;
        let metadata = file.metadata()?;
        let source = OdlSource {
            path_id,
            path,
            filetype,
            fixed_offset,
            filesz: metadata.len(),
            mtime: match original_mtime {
                Some(time) => time,
                None => metadata.modified()?,
            },
        };
        let mut reader = Self::from_reader(file, source, companions.decoding)?;
        reader.named_temp_file = named_temp_file;
        summary_stat!({
            reader
                .statistics
                .odlreader_supplementary_files_used = companions.used;
            reader
                .statistics
                .odlreader_supplementary_files_notfound = companions.notfound;
        });
        for diagnostic in companions.diagnostics {
            reader.report_error(diagnostic);
        }
        Ok(reader)
    }
}

impl<R: Read> OdlReader<R> {
    pub fn from_reader(
        reader: R,
        source: OdlSource,
        decoding: OdlDecodingContext,
    ) -> io::Result<Self> {
        if !source.filetype.is_odl() {
            return Err(Error::new(ErrorKind::InvalidInput, "OdlReader requires FileType::Odl"));
        }
        let mut statistics = SummaryOdlReader::default();
        summary_stat!({
            statistics.odlreader_filesz = source.filesz;
            statistics.odlreader_companions_available = decoding.has_companions();
        });
        Ok(Self {
            source,
            reader: Some(reader),
            decoding: Some(decoding),
            events: BTreeMap::new(),
            named_temp_file: None,
            statistics,
            analyzed: false,
            error: None,
        })
    }

    pub fn mtime(&self) -> SystemTime {
        self.source.mtime
    }
    pub fn path(&self) -> &FPath {
        &self.source.path
    }
    pub fn path_id(&self) -> PathId {
        self.source.path_id
    }
    pub fn filesz(&self) -> FileSz {
        self.source.filesz
    }
    pub fn filetype(&self) -> FileType {
        self.source.filetype
    }
    pub fn summary(&self) -> SummaryOdlReader {
        self.statistics.clone()
    }

    fn report_error(
        &mut self,
        message: String,
    ) {
        if self.error.is_none() {
            crate::e_err!("{}: {}", self.source.path, message);
            self.error = Some(message);
        }
    }

    pub fn analyze(
        &mut self,
        after: &DateTimeLOpt,
        before: &DateTimeLOpt,
        cancel: &AtomicBool,
    ) -> io::Result<()> {
        if self.analyzed {
            return Err(Error::new(ErrorKind::InvalidInput, "ODL reader already analyzed"));
        }
        self.analyzed = true;
        let reader = self
            .reader
            .take()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "ODL reader has no input"))?;
        let decoding = self
            .decoding
            .take()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "ODL reader has no decoding context"))?;
        let mut parser = match OdlParser::new(reader, decoding) {
            Ok(parser) => parser,
            Err(error) => {
                self.report_error(error.to_string());
                return Err(error);
            }
        };
        summary_stat!({
            self.statistics
                .odlreader_version = parser.header().version;
            self.statistics
                .odlreader_compressed = parser.header().compressed;
        });
        let mut previous = None;
        let mut decoding_failures: Count = 0;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(Error::new(ErrorKind::Interrupted, "ODL analysis cancelled"));
            }
            let Some(result) = parser.next_event() else { break };
            let event = match result {
                Ok(event) => event,
                Err(error @ OdlRecordError::Skipped { .. }) => {
                    summary_stat!(
                        self.statistics
                            .odlreader_records_skipped += 1
                    );
                    self.report_error(error.to_string());
                    continue;
                }
                Err(error @ OdlRecordError::Fatal { .. }) => {
                    let message = error.to_string();
                    self.report_error(message.clone());
                    return Err(Error::new(ErrorKind::InvalidData, message));
                }
            };
            let rendered = match event.render(&self.source.fixed_offset) {
                Ok(rendered) => rendered,
                Err(error) => {
                    summary_stat!(
                        self.statistics
                            .odlreader_records_skipped += 1
                    );
                    self.report_error(format!("ODL record {}: {}", event.ordinal, error));
                    continue;
                }
            };
            let dt = *rendered.dt();
            decoding_failures += event.decoding_failures as Count;
            summary_stat!({
                let stats = &mut self.statistics;
                stats.odlreader_events_processed += 1;
                stats.odlreader_event_largest_processed = stats
                    .odlreader_event_largest_processed
                    .max(rendered.len() as Count);
                stats.odlreader_events_undecoded +=
                    Count::from(event.undecoded_bytes != 0 || event.decoding_failures != 0);
                stats.odlreader_decoding_failures = decoding_failures;
                if stats
                    .odlreader_datetime_first_processed
                    .is_none_or(|time| time > dt)
                {
                    stats.odlreader_datetime_first_processed = Some(dt);
                }
                if stats
                    .odlreader_datetime_last_processed
                    .is_none_or(|time| time < dt)
                {
                    stats.odlreader_datetime_last_processed = Some(dt);
                }
                if previous.is_some_and(|time| time > dt) {
                    stats.odlreader_out_of_order += 1;
                }
                previous = Some(dt);
            });
            if !matches!(dt_pass_filters(&dt, after, before), Result_Filter_DateTime2::InRange) {
                continue;
            }
            summary_stat!({
                let stats = &mut self.statistics;
                stats.odlreader_events_accepted += 1;
                stats.odlreader_event_largest_accepted = stats
                    .odlreader_event_largest_accepted
                    .max(rendered.len() as Count);
                if stats
                    .odlreader_datetime_first_accepted
                    .is_none_or(|time| time > dt)
                {
                    stats.odlreader_datetime_first_accepted = Some(dt);
                }
                if stats
                    .odlreader_datetime_last_accepted
                    .is_none_or(|time| time < dt)
                {
                    stats.odlreader_datetime_last_accepted = Some(dt);
                }
            });
            self.events
                .insert((dt, event.ordinal), rendered);
        }
        if decoding_failures != 0 {
            crate::e_wrn!(
                "{}: {} ODL protected tokens could not be decoded; original tokens retained",
                self.source.path,
                decoding_failures,
            );
        }
        if let Some(error) = &self.error {
            return Err(Error::new(ErrorKind::InvalidData, error.clone()));
        }
        Ok(())
    }

    pub fn next(&mut self) -> Option<Odl> {
        assert!(self.analyzed, "analyze ODL input before next()");
        self.events
            .pop_first()
            .map(|(_, event)| event)
    }

    pub fn summary_complete(&self) -> Summary {
        Summary::new(
            self.source.path.clone(),
            self.named_temp_file
                .as_ref()
                .map(|path| path_to_fpath(path.as_ref())),
            self.source.filetype,
            LogMessageType::Odl,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(self.summary()),
            self.error.clone(),
        )
    }
}

#[derive(Default)]
struct OdlCompanions {
    decoding: OdlDecodingContext,
    used: Vec<FPath>,
    notfound: Vec<FPath>,
    diagnostics: Vec<String>,
}

impl OdlCompanions {
    fn load(
        &mut self,
        index: usize,
        path: FPath,
        reader: &mut dyn Read,
    ) {
        let result = if index == 0 {
            self.decoding.load_map(reader)
        } else {
            self.decoding
                .load_keystore(reader)
        };
        match result {
            Ok(()) => {
                summary_stat!(if !self.used.contains(&path) {
                    self.used.push(path);
                });
            }
            Err(error) => self.report_error(path, error),
        }
    }

    fn report_error(
        &mut self,
        path: FPath,
        error: Error,
    ) {
        summary_stat!(if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::PermissionDenied)
            && !self.notfound.contains(&path)
        {
            self.notfound
                .push(path.clone());
        });
        self.diagnostics
            .push(format!("ODL companion {}: {}", path, error));
    }
}

fn load_companions(
    path_id: PathId,
    path: &str,
    filetype: FileType,
) -> io::Result<OdlCompanions> {
    let names = [
        PathBuf::from("ObfuscationStringMap.txt"),
        PathBuf::from("general.keystore"),
        PathBuf::from("EncryptionKeyStoreCopy").join("general.keystore"),
    ];
    let mut companions = OdlCompanions::default();
    if filetype.is_archived() {
        let (archive_path, member) = path
            .rsplit_once(SUBPATH_SEP)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "ODL tar input has no member path"))?;
        let parent = Path::new(member)
            .parent()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "ODL tar member has no parent"))?;
        let targets: Vec<PathBuf> = names
            .iter()
            .map(|name| parent.join(name))
            .collect();
        let file = FILE_HANDLE_MANAGER.request_open_managed(
            path_id,
            FileHandleRole::SecondaryRead,
            Path::new(archive_path),
            OpenOptionsManaged::read_only(),
        )?;
        let mut archive = tar::Archive::new(file);
        let mut found = [false; 3];
        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry.path()?.into_owned();
            if let Some(index) = targets
                .iter()
                .position(|path| path == &entry_path)
            {
                let companion_path = format!("{}{}{}", archive_path, SUBPATH_SEP, path_to_fpath(&entry_path));
                found[index] = true;
                companions.load(index, companion_path, &mut entry);
            }
        }
        summary_stat!({
            for (target, found) in targets.iter().zip(found) {
                if !found {
                    companions
                        .notfound
                        .push(format!("{}{}{}", archive_path, SUBPATH_SEP, path_to_fpath(target)));
                }
            }
        });
    } else {
        let parent = Path::new(path)
            .parent()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "ODL input has no parent directory"))?;
        for (index, name) in names.iter().enumerate() {
            let companion_path = parent.join(name);
            let file = FILE_HANDLE_MANAGER.request_open_managed(
                path_id,
                FileHandleRole::PrimaryRead,
                &companion_path,
                OpenOptionsManaged::read_only(),
            );
            match file {
                Ok(mut file) => companions.load(index, path_to_fpath(&companion_path), &mut file),
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    summary_stat!(
                        companions
                            .notfound
                            .push(path_to_fpath(&companion_path))
                    );
                }
                Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                    companions.report_error(path_to_fpath(&companion_path), error);
                }
                Err(error) => return Err(error),
            }
        }
    }
    Ok(companions)
}
