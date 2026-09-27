// src/readers/etlreader.rs

//! Implements a [`EtlReader`], the driver of deriving [`Etl`s] from a
//! Windows [Event Trace Log] (`.etl`) file using an [`EtlParser`].
//!
//! Sibling of [`EvtxReader`]. Like `.evtx` files, events within a `.etl`
//! file are not stored in chronological order (buffers are flushed
//! per-processor), so the entire file is read and sorted by timestamp
//! before any event is returned.
//!
//! [`EtlReader`]: self::EtlReader
//! [`Etl`s]: crate::data::etl::Etl
//! [`EtlParser`]: crate::readers::etlparser::EtlParser
//! [Event Trace Log]: https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing
//! [`EvtxReader`]: crate::readers::evtxreader::EvtxReader

use std::collections::BTreeMap;
use std::fmt;
use std::io::Result;
use std::path::Path;

#[allow(unused_imports)]
use ::si_trace_print::{def1n, def1o, def1x, def1ñ, def2ñ, defn, defo, defx, defñ};
use ::tempfile::TempPath;

use crate::common::{debug_panic, summary_stat, Count, FPath, FileMetadata, FileSz, FileType, PathId};
use crate::data::datetime::{
    dt_pass_filters, DateTimeL, DateTimeLOpt, FixedOffset, Result_Filter_DateTime2, SystemTime,
};
use crate::data::etl::{Etl, EtlDecoder, EtlEvent};
use crate::de_err;
use crate::readers::etlparser::{EtlParser, EtlRecordError};
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::filehandlemanager::{FileHandleManaged, FileHandleRole, OpenOptionsManaged, FILE_HANDLE_MANAGER};
use crate::readers::helpers::path_to_fpath;
use crate::readers::summary::Summary;

// ---------
// EtlReader

pub type EventsKey = (DateTimeL, usize);
pub type Events = BTreeMap<EventsKey, Etl>;

/// Statistics of an `EtlReader` for `--summary`.
#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryEtlReader {
    pub etlreader_events_processed: Count,
    pub etlreader_events_accepted: Count,
    pub etlreader_event_largest_processed: Count,
    pub etlreader_event_largest_accepted: Count,
    /// datetime soonest processed
    pub etlreader_datetime_first_processed: DateTimeLOpt,
    /// datetime latest processed
    pub etlreader_datetime_last_processed: DateTimeLOpt,
    /// datetime soonest accepted (printed)
    pub etlreader_datetime_first_accepted: DateTimeLOpt,
    /// datetime latest accepted (printed)
    pub etlreader_datetime_last_accepted: DateTimeLOpt,
    pub etlreader_filesz: FileSz,
    pub etlreader_out_of_order: Count,
    pub etlreader_buffers_read: Count,
    pub etlreader_buffers_compressed: Count,
    /// events decoded from TraceLogging metadata
    pub etlreader_events_tracelogging: Count,
    /// events decoded by kernel decoders
    pub etlreader_events_kernel: Count,
    /// events with raw (undecoded) payloads
    pub etlreader_events_undecoded: Count,
    /// records skipped due to parse errors
    pub etlreader_records_skipped: Count,
    pub etlreader_session_name: String,
}

/// A wrapper for using [`EtlParser`] to read a `.etl` file.
///
/// The entire file is parsed by [`EtlReader::analyze`] into a `BTreeMap`
/// sorted by timestamp; [`EtlReader::next`] pops events in order.
///
/// [`EtlParser`]: crate::readers::etlparser::EtlParser
pub struct EtlReader {
    /// the open file; taken by `analyze()`
    file: Option<FileHandleManaged>,
    /// sorted events awaiting `next()`
    events: Events,
    path: FPath,
    path_id: PathId,
    fixed_offset: FixedOffset,
    /// If necessary, the extracted file as a temporary file.
    named_temp_file: Option<TempPath>,
    filetype: FileType,
    filesz: FileSz,
    mtime: SystemTime,
    pub(crate) events_processed: Count,
    pub(crate) events_accepted: Count,
    pub(crate) event_largest_processed: Count,
    pub(crate) event_largest_accepted: Count,
    pub(crate) dt_first_processed: DateTimeLOpt,
    pub(crate) dt_last_processed: DateTimeLOpt,
    pub(crate) dt_first_accepted: DateTimeLOpt,
    pub(crate) dt_last_accepted: DateTimeLOpt,
    pub(crate) out_of_order: Count,
    pub(crate) buffers_read: Count,
    pub(crate) buffers_compressed: Count,
    pub(crate) events_tracelogging: Count,
    pub(crate) events_kernel: Count,
    pub(crate) events_undecoded: Count,
    pub(crate) records_skipped: Count,
    session_name: String,
    analyzed: bool,
    /// The first [`Error`], if any, as a `String`
    ///
    /// [`Error`]: std::io::Error
    error: Option<String>,
}

impl fmt::Debug for EtlReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("EtlReader")
            .field("Path", &self.path)
            .field("Error?", &self.error)
            .finish()
    }
}

impl EtlReader {
    /// Create a new `EtlReader`.
    ///
    /// **NOTE:** should not attempt any file reads here, similar to other
    /// `*Readers::new()`
    pub fn new(
        path_id: PathId,
        path: FPath,
        filetype: FileType,
        fixed_offset: FixedOffset,
    ) -> Result<EtlReader> {
        def1n!("({}, {:?}, {:?})", path_id, path, filetype);

        let path_std: &Path = Path::new(&path);
        let (named_temp_file, mtime_opt): (Option<TempPath>, Option<SystemTime>) =
            match decompress_to_ntf(path_id, path_std, &filetype) {
                Ok(Some((ntf, mtime_opt, _filesz))) => (Some(ntf), mtime_opt),
                Ok(None) => (None, None),
                Err(err) => {
                    def1x!("decompress_to_ntf({:?}, {:?}) Error, return {:?}", path, filetype, err);
                    return Err(err);
                }
            };
        let path_actual: &Path = match named_temp_file {
            Some(ref ntf) => ntf.as_ref(),
            None => path_std,
        };
        def1o!("path_actual {:?}", path_actual);
        let file: FileHandleManaged = match FILE_HANDLE_MANAGER.request_open_managed(
            path_id,
            FileHandleRole::PrimaryRead,
            path_actual,
            OpenOptionsManaged::read_only(),
        ) {
            Ok(val) => val,
            Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let metadata: FileMetadata = match file.metadata() {
            Ok(val) => val,
            Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let mtime: SystemTime = match mtime_opt {
            Some(val) => val,
            None => match metadata.modified() {
                Ok(val) => val,
                Err(_err) => {
                    de_err!("metadata.modified() failed {}", _err);
                    SystemTime::UNIX_EPOCH
                }
            },
        };
        let filesz: FileSz = metadata.len() as FileSz;
        def1x!("return Ok(EtlReader); filesz {} mtime {:?}", filesz, mtime);

        Ok(EtlReader {
            file: Some(file),
            events: Events::new(),
            path,
            path_id,
            fixed_offset,
            named_temp_file,
            filetype,
            filesz,
            mtime,
            events_processed: 0,
            events_accepted: 0,
            event_largest_processed: 0,
            event_largest_accepted: 0,
            dt_first_processed: None,
            dt_last_processed: None,
            dt_first_accepted: None,
            dt_last_accepted: None,
            out_of_order: 0,
            buffers_read: 0,
            buffers_compressed: 0,
            events_tracelogging: 0,
            events_kernel: 0,
            events_undecoded: 0,
            records_skipped: 0,
            session_name: String::new(),
            analyzed: false,
            error: None,
        })
    }

    pub const fn mtime(&self) -> SystemTime {
        self.mtime
    }

    pub const fn path_id(&self) -> PathId {
        self.path_id
    }

    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        &self.path
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.filetype
    }

    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.filesz
    }

    fn set_error(
        &mut self,
        err: String,
    ) {
        if self.error.is_none() {
            self.error = Some(err);
        }
    }

    /// Read and sort the entire file; call once before `next()`.
    pub fn analyze(
        &mut self,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) {
        defn!("({:?}, {:?})", dt_filter_after, dt_filter_before);
        debug_assert!(!self.analyzed, "analyze() called twice");
        self.analyzed = true;
        let file: FileHandleManaged = match self.file.take() {
            Some(file) => file,
            None => {
                debug_panic!("EtlReader::analyze() file already taken");
                return;
            }
        };
        let mut parser: EtlParser<FileHandleManaged> = match EtlParser::new(file) {
            Ok(parser) => parser,
            Err(err) => {
                defx!("EtlParser::new() Error {:?}", err);
                self.set_error(err.to_string());
                return;
            }
        };
        self.session_name = parser
            .header()
            .session_name
            .clone();
        let mut dt_last: DateTimeLOpt = None;
        let mut index: usize = 0;
        while let Some(result) = parser.next_event() {
            let event: EtlEvent = match result {
                Ok(event) => event,
                Err(err @ EtlRecordError::Skipped { .. }) => {
                    de_err!("{}: {}", self.path, err);
                    self.set_error(err.to_string());
                    continue;
                }
                Err(EtlRecordError::Fatal(reason)) => {
                    de_err!("{}: {}", self.path, reason);
                    self.set_error(reason);
                    break;
                }
            };
            let (dt, data, dt_beg_end) = match event.render(&self.fixed_offset) {
                Some(val) => val,
                None => {
                    de_err!("{}: event has unrepresentable timestamp FILETIME {}", self.path, event.envelope.filetime);
                    continue;
                }
            };
            index += 1;
            let len: Count = data.len() as Count;
            summary_stat!(self.events_processed += 1);
            summary_stat!(
                self.event_largest_processed = self
                    .event_largest_processed
                    .max(len)
            );
            summary_stat!(match event.decoder {
                EtlDecoder::TraceLogging => self.events_tracelogging += 1,
                EtlDecoder::Kernel => self.events_kernel += 1,
                EtlDecoder::None => self.events_undecoded += 1,
            });
            summary_stat!(if self
                .dt_first_processed
                .is_none_or(|d| d > dt)
            {
                self.dt_first_processed = Some(dt);
            });
            summary_stat!(if self
                .dt_last_processed
                .is_none_or(|d| d < dt)
            {
                self.dt_last_processed = Some(dt);
            });
            summary_stat!(if let Some(last) = dt_last {
                if last > dt {
                    self.out_of_order += 1;
                }
            });
            dt_last = Some(dt);

            match dt_pass_filters(&dt, dt_filter_after, dt_filter_before) {
                Result_Filter_DateTime2::InRange => {}
                Result_Filter_DateTime2::BeforeRange | Result_Filter_DateTime2::AfterRange => continue,
            }
            summary_stat!(self.events_accepted += 1);
            summary_stat!(
                self.event_largest_accepted = self
                    .event_largest_accepted
                    .max(len)
            );
            summary_stat!(if self
                .dt_first_accepted
                .is_none_or(|d| d > dt)
            {
                self.dt_first_accepted = Some(dt);
            });
            summary_stat!(if self
                .dt_last_accepted
                .is_none_or(|d| d < dt)
            {
                self.dt_last_accepted = Some(dt);
            });
            let etl: Etl = Etl::new(dt, Some(dt_beg_end), data);
            if let Some(e) = self
                .events
                .insert((dt, index), etl)
            {
                debug_panic!("Duplicate key ({:?}, {}) in events BTreeMap: {:?}", dt, index, e);
            }
        }
        self.buffers_read = parser.buffers_read;
        self.buffers_compressed = parser.buffers_compressed;
        self.records_skipped = parser.records_skipped;
        defx!("events {}, buffers {}", self.events.len(), self.buffers_read);
    }

    /// Return the next event in chronological order.
    pub fn next(&mut self) -> Option<Etl> {
        def1ñ!();
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next()`");

        self.events
            .pop_first()
            .map(|(_key, etl)| etl)
    }

    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryEtlReader {
        SummaryEtlReader {
            etlreader_events_processed: self.events_processed,
            etlreader_events_accepted: self.events_accepted,
            etlreader_event_largest_processed: self.event_largest_processed,
            etlreader_event_largest_accepted: self.event_largest_accepted,
            etlreader_datetime_first_processed: self.dt_first_processed,
            etlreader_datetime_last_processed: self.dt_last_processed,
            etlreader_datetime_first_accepted: self.dt_first_accepted,
            etlreader_datetime_last_accepted: self.dt_last_accepted,
            etlreader_filesz: self.filesz,
            etlreader_out_of_order: self.out_of_order,
            etlreader_buffers_read: self.buffers_read,
            etlreader_buffers_compressed: self.buffers_compressed,
            etlreader_events_tracelogging: self.events_tracelogging,
            etlreader_events_kernel: self.events_kernel,
            etlreader_events_undecoded: self.events_undecoded,
            etlreader_records_skipped: self.records_skipped,
            etlreader_session_name: self.session_name.clone(),
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `EtlReader`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let path_ntf: Option<FPath> = self
            .named_temp_file
            .as_ref()
            .map(|ntf| path_to_fpath(ntf.as_ref()));
        let filetype = self.filetype();
        let logmessagetype = filetype.to_logmessagetype();
        let summaryetlreader = self.summary();
        let error: Option<String> = self.error.clone();

        Summary::new(
            path,
            path_ntf,
            filetype,
            logmessagetype,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(summaryetlreader),
            None,
            None,
            error,
        )
    }
}

impl Drop for EtlReader {
    fn drop(&mut self) {
        def2ñ!("EtlReader: PathID {} Path {:?}", self.path_id(), self.path());
    }
}
