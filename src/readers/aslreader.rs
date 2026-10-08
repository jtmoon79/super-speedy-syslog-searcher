//! Managed ASL input, chronological event ordering, and rendered-byte `Read`.

use std::collections::BTreeMap;
use std::io::{self, Error, ErrorKind, Read, Seek};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use tempfile::TempPath;

use crate::common::{Count, FPath, FileSz, FileType, LogMessageType, PathId, summary_stat};
use crate::data::asl::Asl;
use crate::data::datetime::{
    DateTimeL, DateTimeLOpt, FixedOffset, Result_Filter_DateTime2, SystemTime, dt_pass_filters,
};
use crate::readers::aslparser::AslParser;
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::filehandlemanager::{FILE_HANDLE_MANAGER, FileHandleManaged, FileHandleRole, OpenOptionsManaged};
use crate::readers::helpers::path_to_fpath;
use crate::readers::summary::Summary;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct SummaryAslReader {
    pub aslreader_events_processed: Count,
    pub aslreader_events_accepted: Count,
    pub aslreader_event_largest_processed: Count,
    pub aslreader_event_largest_accepted: Count,
    pub aslreader_datetime_first_processed: DateTimeLOpt,
    pub aslreader_datetime_last_processed: DateTimeLOpt,
    pub aslreader_datetime_first_accepted: DateTimeLOpt,
    pub aslreader_datetime_last_accepted: DateTimeLOpt,
    pub aslreader_filesz: FileSz,
    pub aslreader_out_of_order: Count,
}

pub struct AslSource {
    pub path_id: PathId,
    pub path: FPath,
    pub filetype: FileType,
    pub filesz: FileSz,
    pub mtime: SystemTime,
    pub fixed_offset: FixedOffset,
}

#[derive(PartialEq, Eq)]
enum Consumption {
    Unset,
    Events,
    Bytes,
}

pub struct AslReader<R: Read + Seek = FileHandleManaged> {
    source: AslSource,
    reader: Option<R>,
    events: BTreeMap<(DateTimeL, u64), Asl>,
    named_temp_file: Option<TempPath>,
    statistics: SummaryAslReader,
    analyzed: bool,
    error: Option<String>,
    consumption: Consumption,
    read_event: Option<Asl>,
    read_offset: usize,
}

impl AslReader<FileHandleManaged> {
    pub fn new(
        path_id: PathId,
        path: FPath,
        filetype: FileType,
        fixed_offset: FixedOffset,
    ) -> io::Result<Self> {
        if !filetype.is_asl() {
            return Err(Error::new(ErrorKind::InvalidInput, "AslReader requires FileType::Asl"));
        }
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
        let source = AslSource {
            path_id,
            path,
            filetype,
            filesz: metadata.len(),
            mtime: match original_mtime {
                Some(time) => time,
                None => metadata.modified()?,
            },
            fixed_offset,
        };
        let mut reader = Self::from_reader(file, source)?;
        reader.named_temp_file = named_temp_file;
        Ok(reader)
    }
}

impl<R: Read + Seek> AslReader<R> {
    pub fn from_reader(
        reader: R,
        source: AslSource,
    ) -> io::Result<Self> {
        if !source.filetype.is_asl() {
            return Err(Error::new(ErrorKind::InvalidInput, "AslReader requires FileType::Asl"));
        }
        let statistics = SummaryAslReader {
            aslreader_filesz: source.filesz,
            ..Default::default()
        };

        Ok(Self {
            source,
            reader: Some(reader),
            events: BTreeMap::new(),
            named_temp_file: None,
            statistics,
            analyzed: false,
            error: None,
            consumption: Consumption::Unset,
            read_event: None,
            read_offset: 0,
        })
    }

    pub fn mtime(&self) -> SystemTime {
        self.source.mtime
    }

    pub fn analyze(
        &mut self,
        after: &DateTimeLOpt,
        before: &DateTimeLOpt,
        cancel: &AtomicBool,
    ) -> io::Result<()> {
        if self.analyzed {
            return Err(Error::new(ErrorKind::InvalidInput, "ASL reader already analyzed"));
        }
        self.analyzed = true;
        let reader = self
            .reader
            .take()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "ASL reader has no input"))?;
        let mut parser = AslParser::new(reader).map_err(|error| self.record_error(error))?;
        let mut previous = None;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(self.record_error(Error::new(ErrorKind::Interrupted, "ASL analysis cancelled")));
            }
            let Some(record) = parser
                .next_record()
                .map_err(|error| self.record_error(error))?
            else {
                break;
            };
            let rendered = record
                .render(&self.source.fixed_offset)
                .map_err(|error| self.record_error(error))?;
            let dt = *rendered.dt();
            summary_stat!({
                let stats = &mut self.statistics;
                stats.aslreader_events_processed += 1;
                stats.aslreader_event_largest_processed = stats
                    .aslreader_event_largest_processed
                    .max(rendered.len() as Count);
                if stats
                    .aslreader_datetime_first_processed
                    .is_none_or(|time| time > dt)
                {
                    stats.aslreader_datetime_first_processed = Some(dt);
                }
                if stats
                    .aslreader_datetime_last_processed
                    .is_none_or(|time| time < dt)
                {
                    stats.aslreader_datetime_last_processed = Some(dt);
                }
                if previous.is_some_and(|time| time > dt) {
                    stats.aslreader_out_of_order += 1;
                }
                previous = Some(dt);
            });
            if !matches!(dt_pass_filters(&dt, after, before), Result_Filter_DateTime2::InRange) {
                continue;
            }
            summary_stat!({
                let stats = &mut self.statistics;
                stats.aslreader_events_accepted += 1;
                stats.aslreader_event_largest_accepted = stats
                    .aslreader_event_largest_accepted
                    .max(rendered.len() as Count);
                if stats
                    .aslreader_datetime_first_accepted
                    .is_none_or(|time| time > dt)
                {
                    stats.aslreader_datetime_first_accepted = Some(dt);
                }
                if stats
                    .aslreader_datetime_last_accepted
                    .is_none_or(|time| time < dt)
                {
                    stats.aslreader_datetime_last_accepted = Some(dt);
                }
            });
            self.events
                .insert((dt, record.ordinal), rendered);
        }

        Ok(())
    }

    fn record_error(
        &mut self,
        error: Error,
    ) -> Error {
        crate::e_err!("{}: {}", self.source.path, error);
        self.error = Some(error.to_string());

        error
    }

    pub fn next_event(&mut self) -> io::Result<Option<Asl>> {
        if !self.analyzed || self.consumption == Consumption::Bytes {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "analyze ASL input before consuming events; do not mix event and byte reads",
            ));
        }
        self.consumption = Consumption::Events;

        Ok(self
            .events
            .pop_first()
            .map(|(_, event)| event))
    }

    pub fn summary_complete(&self) -> Summary {
        Summary::new(
            self.source.path.clone(),
            self.named_temp_file
                .as_ref()
                .map(|path| path_to_fpath(path.as_ref())),
            self.source.filetype,
            LogMessageType::Asl,
            None,
            None,
            None,
            None,
            None,
            Some(self.statistics.clone()),
            None,
            None,
            None,
            None,
            self.error.clone(),
        )
    }
}

impl<R: Read + Seek> Read for AslReader<R> {
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.consumption == Consumption::Events {
            return Err(Error::new(ErrorKind::InvalidInput, "cannot read ASL bytes after consuming events"));
        }
        if !self.analyzed {
            self.analyze(&None, &None, &AtomicBool::new(false))?;
        }
        if let Some(error) = &self.error {
            return Err(Error::new(ErrorKind::InvalidData, error.clone()));
        }
        self.consumption = Consumption::Bytes;
        if self.read_event.is_none() {
            self.read_event = self
                .events
                .pop_first()
                .map(|(_, event)| event);
            self.read_offset = 0;
        }
        let Some(event) = &self.read_event else { return Ok(0) };
        let data = &event.as_bytes()[self.read_offset..];
        let count = data.len().min(buf.len());
        buf[..count].copy_from_slice(&data[..count]);
        self.read_offset += count;
        if self.read_offset == event.len() {
            self.read_event = None;
        }

        Ok(count)
    }
}
