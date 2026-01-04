// src/readers/etlreader.rs

//! Create an ETL reader that uses a `PyRunner` with different Python scripts.

use std::collections::VecDeque;
use std::fmt;
use std::io::{
    Error,
    ErrorKind,
    Result,
};
use std::path::Path;
use std::time::Duration;

use chrono::{TimeZone, Timelike};
#[allow(unused_imports)]
use ::more_asserts::{
    assert_le,
    debug_assert_ge,
    debug_assert_gt,
    debug_assert_le,
    debug_assert_lt,
};
#[allow(unused_imports)]
use ::si_trace_print::{
    de,
    def1n,
    def1o,
    def1x,
    def1ñ,
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
    pfn,
    pfo,
    pfx,
    ef1n,
    ef1o,
    ef1x,
    ef1ñ,
    ef2n,
    ef2o,
    ef2x,
    ef2ñ,
};
use ::tempfile::NamedTempFile;

use crate::common::{
    Bytes,
    Count,
    FPath,
    File,
    FileMetadata,
    FileOpenOptions,
    FileSz,
    FileType,
    FileTypeArchive,
    ResultFind4,
    summary_stat,
    summary_stat_set,
    summary_stats_enabled,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    Result_Filter_DateTime2,
    SystemTime,
    dt_pass_filters,
};
use crate::data::pydataevent::{
    DtBegEndPairOpt,
    PyDataEvent,
    EtlParserUsed,
    EventBytes,
};
use crate::{
    debug_panic,
    de_err,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::buffer_to_String_noraw;
use crate::python::pyrunner::{
    ChunkDelimiter,
    ExitStatus,
    PipeSz,
    PythonToUse,
    PyRunner,
    RECV_TIMEOUT,
};
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::helpers::path_to_fpath;
use crate::readers::summary::Summary;

/// Return type for `next` methods.
pub type ResultNextPyDataEvent = ResultFind4<PyDataEvent, Error>;

/// Delimiter between Events (Null character)
/// Must match `DELIMITER_EVENTS` in `etl_reader.py`
const DELIMITER_EVENTS: ChunkDelimiter = b'\0';
/// Delimiter between Timestamp and Event (Record Separator character)
/// Must match `DELIMITER_TS_EVENT` in `etl_reader.py`
const DELIMITER_TS_EVENT: ChunkDelimiter = b'\x1E';
/// Input script terminator character (newline)
const SCRIPT_TERM: char = '\n';

/// per this many reads of the Python process, write to it
/// to signal readiness for more data
const WAIT_INPUT_PER_PRINTS: Count = 5;

/// This format must match the output of the underlying Python scripts.
const DATETIME_FORMAT: &str = r"%s";

type EntryBuffer = VecDeque<PyDataEvent>;

/// The PyEventReader supports these `FileType`s
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyEventType {
    Asl,
    Etl,
    Odl,
}

/// A wrapper for running a `PyRunner` instance that calls
/// local Python scripts to read `.etl` files.
/// The Python scripts send bytes back via stdout that are parsed
/// into `PyDataEvent`s.
pub struct PyEventReader {
    /// The type of file being read.
    #[allow(dead_code)]
    file_type: FileType,
    /// The specific type of PyEvent being read.
    #[allow(dead_code)]
    event_type: PyEventType,
    /// The buffer of `PyDataEvent`s that have been read but not yet sent
    /// to the main printing thread.
    fill_buffer: EntryBuffer,
    /// The `FPath` of the .etl file being read.
    path: FPath,
    /// If necessary, the extracted ETL file as a temporary file.
    named_temp_file: Option<NamedTempFile>,
    /// The `PyRunner` instance for running Python code.
    pyrunner: PyRunner,
    /// Has the Python process exited?
    exited: bool,
    /// Summary statistic.
    /// Arguments passed to the Python.
    python_arguments: Vec<String>,
    /// Conversion of Unix epoch timestamps to `DateTimeL` with this timezone offset.
    fixed_offset: FixedOffset,
    /// Summary statistic.
    /// Highest `Count` elements received in one read from the Python process.
    events_read_max: Count,
    /// Summary statistic.
    /// Highest `Count` elements in the `fill_buffer`.
    events_held_max: Count,
    /// Summary statistic.
    /// `Count` of [`PyDataEvent`] processed.
    events_processed: Count,
    /// Summary statistic.
    /// `Count` of [`PyDataEvent`] accepted by the datetime filters.
    events_accepted: Count,
    /// Not a summary statistic.
    write_read_calls: Count,
    /// Summary statistic.
    /// First (soonest) accepted (printed) `DateTimeL`.
    ///
    /// Intended for `--summary`.
    dt_first_accepted: DateTimeLOpt,
    /// Summary statistic.
    /// Last (latest) accepted (printed) `DateTimeL`.
    ///
    /// Intended for `--summary`.
    dt_last_accepted: DateTimeLOpt,
    /// Summary statistic.
    /// First (soonest) processed `DateTimeL`.
    ///
    /// Intended for `--summary`.
    dt_first_processed: DateTimeLOpt,
    /// Summary statistic.
    /// Last (latest) processed `DateTimeL`.
    ///
    /// Intended for `--summary`.
    dt_last_processed: DateTimeLOpt,
    /// File Size of the file being read in bytes.
    filesz: FileSz,
    /// file Last Modified time from file-system metadata
    mtime: SystemTime,
    /// Out of chronological order.
    out_of_order: Count,
    /// The last [`Error`], if any, as a `String`
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    // TRACKING: https://github.com/rust-lang/rust/issues/24135
    error: Option<String>,
}

impl fmt::Debug for PyEventReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("PyEventReader")
            .field("Path", &self.path)
            .field("Error?", &self.error)
            .finish()
    }
}

// TODO: [2023/05] instead of having 1:1 manual copying of `PyEventReader`
//       fields to `SummaryPyEventReader` fields, just store a
//       `SummaryPyEventReader` in `PyEventReader` and update directly.
#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryPyEventReader {
    pub pyeventreader_events_processed: Count,
    /// Acceptable to datetime filters and sent to main thread for printing.
    pub pyeventreader_events_accepted: Count,
    pub pyeventreader_python_count_proc_reads_stdout: Count,
    pub pyeventreader_python_count_proc_reads_stderr: Count,
    pub pyeventreader_python_count_pipe_recv_stdout: Count,
    pub pyeventreader_python_count_pipe_recv_stderr: Count,
    pub pyeventreader_python_count_proc_writes: Count,
    pub pyeventreader_python_count_proc_polls: Count,
    pub pyeventreader_python_arguments: Vec<String>,
    pub pyeventreader_python_exit_status: Option<ExitStatus>,
    /// datetime soonest accepted (printed)
    pub pyeventreader_datetime_first_accepted: DateTimeLOpt,
    /// datetime latest accepted (printed)
    pub pyeventreader_datetime_last_accepted: DateTimeLOpt,
    /// datetime soonest processed
    pub pyeventreader_datetime_first_processed: DateTimeLOpt,
    /// datetime latest processed
    pub pyeventreader_datetime_last_processed: DateTimeLOpt,
    pub pyeventreader_pipe_sz_stdout: PipeSz,
    pub pyeventreader_pipe_sz_stderr: PipeSz,
    pub pyeventreader_filesz: FileSz,
    pub pyeventreader_events_read_max: Count,
    pub pyeventreader_events_held_max: Count,
    pub pyeventreader_out_of_order: Count,
    pub pyeventreader_duration_proc_wait: Duration,
    pub pyeventreader_duration_proc_run: Duration,
}

/// Implement the PyEventReader.
impl PyEventReader {
    /// Entry buffer size.
    pub const ENTRY_BUFFER_SZ: usize = 5;

    /// Create a new `PyEventReader`.
    pub fn new(
        path: FPath,
        etl_parser_used: Option<EtlParserUsed>,
        file_type: FileType,
        fixed_offset: FixedOffset,
        pipe_sz: PipeSz,
    ) -> Result<PyEventReader> {
        def1n!("(path={:?}, etl_parser_used={:?}, {:?}, {:?}, pipe_sz={:?})",
               path, etl_parser_used, file_type, fixed_offset, pipe_sz);

        debug_assert_gt!(pipe_sz, 0,
            "pipe_sz must be greater than 0, got {:?}", pipe_sz);
        if pipe_sz == 0 {
            def1x!("pipe_sz is 0, return Error");
            return Err(
                Error::new(
                    ErrorKind::InvalidInput,
                    "pipe_sz must be greater than 0",
                )
            );
        }

        // get the file size according to the file metadata
        let path_std: &Path = Path::new(&path);
        let mut open_options = FileOpenOptions::new();
        let named_temp_file: Option<NamedTempFile>;
        let mtime_opt: Option<SystemTime>;

        (named_temp_file, mtime_opt) = match decompress_to_ntf(path_std, &file_type) {
            Ok(ntf_mtime) => match ntf_mtime {
                Some((ntf, mtime_opt, _filesz)) => (Some(ntf), mtime_opt),
                None => (None, None),
            },
            Err(err) => {
                def1x!("decompress_to_ntf({:?}, {:?}) Error, return {:?}", path, file_type, err,);
                return Err(err);
            }
        };
        def1o!("named_temp_file {:?}", named_temp_file);
        def1o!("mtime_opt {:?}", mtime_opt);

        let path_actual: &Path = match named_temp_file {
            Some(ref ntf) => ntf.path(),
            None => path_std,
        };
        def1o!("path_actual {:?}", path_actual);
        def1o!("open_options.read(true).open({:?})", path_actual);
        let file: File = match open_options
            .read(true)
            .open(path_actual)
        {
            Result::Ok(val) => val,
            Result::Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let metadata: FileMetadata = match file.metadata() {
            Result::Ok(val) => val,
            Result::Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let filesz: FileSz = metadata.len() as FileSz;
        def1o!("filesz {:?}", filesz);

        let mtime: SystemTime = match mtime_opt {
            Some(val) => val,
            None => match metadata.modified() {
                Result::Ok(val) => val,
                Result::Err(_err) => {
                    de_err!("metadata.modified() failed {}", _err);
                    SystemTime::UNIX_EPOCH
                }
            },
        };
        def1o!("mtime {:?}", mtime);

        // prepare the arguments to pass to the Python script

        let path_actual_s: &str = match path_actual.to_str() {
            Some(s) => s,
            None => {
                def1x!("could not convert path to str {:?}, return Unsupported", path_actual);
                return Err(
                    Error::new(
                        ErrorKind::Unsupported,
                        format!("could not convert path to str; {:?}", path_actual),
                    )
                );
            }
        };

        // TODO: how to make a wrong `FileType` a compile-time error? i.e. how to be more rustic?
        let event_type: PyEventType = match file_type {
            FileType::Asl { .. } => PyEventType::Asl,
            FileType::Etl { .. } => PyEventType::Etl,
            FileType::Odl { .. } => PyEventType::Odl,
            _ => panic!("PyEventReader only supports FileType::Asl, FileType::Etl, FileType::Odl"),
        };

        let s4_python_module: String;
        let mut extra_args: Vec<&str> = Vec::with_capacity(3);
        match event_type {
            PyEventType::Asl => {
                s4_python_module = String::from("s4_event_readers.ccl_asldb");
                extra_args.push("--quiet");
                extra_args.push("-t");
                extra_args.push("s4");
                if etl_parser_used.is_some() {
                    debug_panic!("etl_parser_used is Some for ASL file");
                }
            },
            PyEventType::Etl => {
                match etl_parser_used {
                    Some(EtlParserUsed::DissectEtl) => {
                        s4_python_module = String::from("s4_event_readers.etl_reader_dissect_etl");
                    }
                    Some(EtlParserUsed::EtlParser) => {
                        s4_python_module = String::from("s4_event_readers.etl_reader_etl_parser");
                    }
                    None => {
                        debug_panic!("etl_parser_used is None for ETL file");
                        def1x!("etl_parser_used is None for ETL file, return Error");
                        return Err(
                            Error::new(
                                ErrorKind::InvalidInput,
                                "etl_parser_used must be Some(EtlParserUsed) for ETL files",
                            )
                        );
                    }
                }
            },
            PyEventType::Odl => {
                s4_python_module = String::from("s4_event_readers.odl_reader");
                extra_args.push("--no-color");
                extra_args.push("--all_key_values");
                extra_args.push("--all_data");
                if etl_parser_used.is_some() {
                    debug_panic!("etl_parser_used is Some for ODL file");
                }
            }
        };
        let wait_input_per_prints: String = format!("--wait-input-per-prints={}", WAIT_INPUT_PER_PRINTS + 1);
        let mut args = vec![
                "-OO",
                "-m",
                s4_python_module.as_str(),
                path_actual_s,
                &wait_input_per_prints,
        ];
        args.extend(extra_args);
        let python_arguments: Vec<String> = summary_stat_set!(
            args.iter().map(|s| s.to_string()).collect(),
            Vec::<String>::with_capacity(0)
        );
        let pyrunner: PyRunner = match PyRunner::new(
            PythonToUse::EnvVenv,
            std::cmp::min(pipe_sz, 32768),
            RECV_TIMEOUT,
            Some(DELIMITER_EVENTS),
            None,
            None,
            args,
        ) {
            Result::Ok(val) => val,
            Result::Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };

        def1x!("return Ok(PyEventReader)");

        Result::Ok(PyEventReader {
            file_type,
            event_type,
            fill_buffer: EntryBuffer::with_capacity(WAIT_INPUT_PER_PRINTS as usize + 3),
            path,
            named_temp_file,
            pyrunner,
            exited: false,
            python_arguments,
            fixed_offset,
            events_read_max: 0,
            events_held_max: 0,
            events_processed: 0,
            events_accepted: 0,
            write_read_calls: 0,
            dt_first_accepted: DateTimeLOpt::None,
            dt_last_accepted: DateTimeLOpt::None,
            dt_first_processed: DateTimeLOpt::None,
            dt_last_processed: DateTimeLOpt::None,
            filesz,
            mtime,
            out_of_order: 0,
            error: None,
        })
    }

    pub const fn mtime(&self) -> SystemTime {
        self.mtime
    }

    pub const fn pipe_sz_stdout(&self) -> PipeSz {
        self.pyrunner.pipe_sz_stdout
    }

    pub const fn pipe_sz_stderr(&self) -> PipeSz {
        self.pyrunner.pipe_sz_stderr
    }

    fn process_bytes_to_pydataevent(
        &mut self,
        data: &[u8],
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> Option<PyDataEvent> {
        def1n!("data is {} bytes\n{}", data.len(), buffer_to_String_noraw(data));
        // the parsing that happens here must correspond the data sent in
        // the script `etl_reader.py`
        if data.is_empty() {
            def1x!("empty data, return None");
            return None;
        }

        self.events_processed += 1; // used in debug prints and summary

        // split the data into timestamp and event data
        let mut etdi = data.split(|x| *x == DELIMITER_TS_EVENT);
        // parse field with timestamp index start
        let ts_a: &[u8] = etdi.next().unwrap_or_default();
        let ts_a_s: String = String::from_utf8_lossy(ts_a).into_owned();
        let mut ts_a_d: usize = ts_a_s.parse().unwrap_or_default();
        def1o!("ts_a_s {:?}, ts_a_d {}", ts_a_s, ts_a_d);
        // parse field with timestamp index end
        let ts_b: &[u8] = etdi.next().unwrap_or_default();
        let ts_b_s: String = String::from_utf8_lossy(ts_b).into_owned();
        let mut ts_b_d: usize = ts_b_s.parse().unwrap_or_default();
        def1o!("ts_b_s {:?}, ts_b_d {}", ts_b_s, ts_b_d);
        // make sure the offsets make sense
        debug_assert_le!(ts_a_d, ts_b_d,
            "bad timestamp offsets {} {} from {:?} {:?}", ts_a_d, ts_b_d, ts_a, ts_b);
        if ts_a_d > ts_b_d {
            ts_a_d = 0;
            ts_b_d = 0;
        }
        // parse field with timestamp string
        let ts_data: &[u8] = etdi.next().unwrap_or_default();
        def1o!("ts_data is {} bytes '{}'", ts_data.len(),  buffer_to_String_noraw(ts_data));
        if ts_data.is_empty() {
            def1x!("empty ts_data, return None; events_processed {}", self.events_processed);
            return None;
        }
        // parse field with event data
        let event_data: &[u8] = etdi.next().unwrap_or_default();
        def1o!("event_data is {} bytes", event_data.len());
        if event_data.is_empty() {
            def1x!("empty event_data, return None; events_processed {}", self.events_processed);
            return None;
        }
        // sanity check the timestamp offsets
        debug_assert_le!(ts_a_d, event_data.len());
        debug_assert_le!(ts_b_d, event_data.len());
        if ts_a_d >= event_data.len() || ts_b_d >= event_data.len() {
            // if the timestamp offsets are out of bounds, reset them to 0
            ts_a_d = 0;
            ts_b_d = 0;
        }

        // convert the timestamp slices into a DateTimeL
        let dt: DateTimeL = match self.ts_data_to_datetime(ts_data) {
            Some(dt) => dt,
            None => {
                def1x!("invalid timestamp from {} bytes, return None; events_processed {}",
                    ts_data.len(), self.events_processed);
                return None;
            }
        };
        summary_stat!(self.dm_first_last_update_processed(&dt));

        // end of parsing

        match dt_pass_filters(&dt, dt_filter_after, dt_filter_before) {
            Result_Filter_DateTime2::InRange => {
                def1o!("InRange");
            }
            Result_Filter_DateTime2::BeforeRange => {
                def1x!("BeforeRange; return None");
                return None;
            }
            Result_Filter_DateTime2::AfterRange => {
                def1x!("AfterRange; return None");
                return None;
            }
        }
        summary_stat!(self.dm_first_last_update_accepted(&dt));
        self.events_accepted += 1; // used in debug prints and summary

        def1x!("return Some(PyDataEvent), events_processed {}, events_accepted {}",
            self.events_processed,
            self.events_accepted
        );

        Some(PyDataEvent::new(
            EventBytes::from(event_data),
            dt,
            DtBegEndPairOpt::Some((ts_a_d, ts_b_d)),
        ))
    }

    /// Convert `ts_data` bytes to `DateTimeL` that is UTC.
    /// Example `ts_data` data string
    /// ```text
    /// b"1590423555554"
    /// ``````
    /// which is *May 25, 2020 17:59:15.554 UTC*.
    ///
    /// Converts `bytes` to `String` to `u64` to `u64/1000` to `String` to
    /// `DateTimeLOpt`.
    pub(crate) fn ts_data_to_datetime(&self, ts_data: &[u8]) -> DateTimeLOpt {
        let ts_str = String::from_utf8_lossy(ts_data);
        let ts_ms_val: u64 = match ts_str.parse() {
            Ok(v) => v,
            Err(_err) => {
                def1ñ!("failed to convert {:?}: {}", ts_str, _err);
                return DateTimeLOpt::None;
            }
        };
        let ts_val: u64 = ts_ms_val / 1000;
        let ts_val_ms_remainder: u32 = u32::try_from(ts_ms_val % 1000).unwrap_or_default();
        let ts_val_s: String = ts_val.to_string();
        match chrono::NaiveDateTime::parse_from_str(
            &ts_val_s,
            DATETIME_FORMAT,
        ) {
            Ok(dt) => {
                match dt.with_nanosecond(ts_val_ms_remainder * 1_000_000) {
                    Some(dt_ns) => DateTimeLOpt::Some(
                        self.fixed_offset.from_utc_datetime(&dt_ns)
                    ),
                    None => {
                        def1ñ!("failed to set nanoseconds for {:?} remainder {}", dt, ts_val_ms_remainder);
                        DateTimeLOpt::None
                    }
                }
            }
            Err(_err) => {
                def1ñ!("failed to convert {:?}: {}", ts_str, _err);

                DateTimeLOpt::None
            },
        }
    }

    #[inline(always)]
    pub fn next(
        &mut self,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> ResultNextPyDataEvent {
        def1n!("({:?}, {:?})", dt_filter_after, dt_filter_before);

        while !self.exited {
            let stdout_data: Bytes;
            let mut _stderr_data: Bytes;

            'block: {
                self.write_read_calls += 1;

                // send a count + newline to the script
                let stdin_data: Option<Bytes> = match self.write_read_calls {
                    val if val % WAIT_INPUT_PER_PRINTS == 0 => {
                        let data = format!("{}{}", self.write_read_calls, SCRIPT_TERM);
                        def1o!("call pyrunner.write_read('{}')", buffer_to_String_noraw(&data.as_bytes()));

                        Some(Bytes::from(data.as_bytes()))
                    }
                    _ => {
                        def1o!("call pyrunner.write_read(None)");

                        None
                    }
                };

                let outputs = self.pyrunner.write_read(stdin_data.as_deref());
                (stdout_data, _stderr_data) = match outputs {
                    (exited, out, err) => {
                        if ! exited {
                            debug_assert!(! self.exited, "pyrunner.write_read returned exited=false but self.exited=true");
                        }
                        self.exited = exited;
                        def1o!(
                            "pyrunner.write_read returned ({}, {:?} stdout bytes, {:?} stderr bytes)",
                            exited, out.as_ref().map_or(0, |v| v.len()), err.as_ref().map_or(0, |v| v.len())
                        );

                        (
                            out.unwrap_or(Bytes::with_capacity(0)),
                            err.unwrap_or(Bytes::with_capacity(0))
                        )
                    }
                };

                def1o!("stdout: '{}'", buffer_to_String_noraw(&stdout_data));
                def1o!("stderr: '{}'", buffer_to_String_noraw(&_stderr_data));
                def1o!("self.exited {}, pyrunner.exit_okay() {}", self.exited, self.pyrunner.exit_okay());

                if self.exited {
                    def1o!("exited, break");
                    break 'block;
                }
                // if stdout_data.is_empty() {
                //     // ignore `stderr_data` for now.
                //     // later it is checked during process exit and retrieved from `pyrunner.stderr_all()`
                //     def1o!("empty stdout, continue…");
                //     continue;
                // }
                def1o!("not exited, has data, break");
            }

            def1o!("Python process is {}", if self.exited {"exited"} else {"running"});

            // count the number of DELIMITER_EVENTS in stdout_data
            // this could use `memchr` but it would require creating a new `Finder<'_>` each time
            // XXX: does that even matter for performance here? Probably not.
            let delim_events_count: usize = stdout_data.iter().filter(|x| **x == DELIMITER_EVENTS).count();
            def1o!("stdout_data is {} bytes, has {} DELIMITER_EVENTS",
                stdout_data.len(),
                delim_events_count
            );
            summary_stat!(
                self.events_read_max = std::cmp::max(
                    self.events_read_max,
                    delim_events_count as Count,
                )
            );
            def1o!("events_read_max is now {}", self.events_read_max);

            def1o!("stderr_data is {} bytes", _stderr_data.len());

            // Parse the events in the stdout_data, push them to `fill_buffer`,
            // and lastly pop the `fill_buffer` and return that event.
            for event_ts_data in stdout_data.split(|x| *x == DELIMITER_EVENTS) {
                def1o!("event_ts_data is {} bytes, has {} DELIMITER_TS_EVENT",
                    event_ts_data.len(),
                    event_ts_data.iter().filter(|x| **x == DELIMITER_TS_EVENT).count()
                );

                let etl_event = match self.process_bytes_to_pydataevent(
                    event_ts_data,
                    dt_filter_after,
                    dt_filter_before,
                ) {
                    Some(event) => event,
                    None => {
                        def1o!("process_bytes_to_pydataevent returned None; continue processing stdout_data…");
                        continue;
                    }
                };

                def1o!("fill_buffer.push_back({} bytes) (timestamp: {:?}) events_processed {}, events_accepted {}",
                    etl_event.len(), etl_event.dt(), self.events_processed, self.events_accepted);
                self.fill_buffer.push_back(etl_event);

                summary_stat!(
                    self.events_held_max = std::cmp::max(
                        self.events_held_max,
                        self.fill_buffer.len() as Count,
                    )
                )
            } // for event_ts_data

            def1o!("fill_buffer has {} events", self.fill_buffer.len());
            match self.fill_buffer.pop_front() {
                Some(event) => {
                    def1x!("return event with {} bytes, fill_buffer has {} events",
                        event.len(), self.fill_buffer.len());
                    return ResultNextPyDataEvent::Found(event);
                },
                None => {
                    // no events in the fill_buffer
                    if self.pyrunner.exited_exhausted() {
                        if ! self.pyrunner.exit_okay() {
                            // process exited badly
                            // attach the stderr output to the error
                            // later this will be included in the SummaryPyEventReader
                            self.error = Some(self.pyrunner.stderr_all()
                                .map_or(
                                    String::from("Unknown error"),
                                    |v| String::from_utf8_lossy(v).into_owned(),
                                )
                            );
                            def1x!("empty fill_buffer, process exited with {:?}, return Err",
                                self.pyrunner.exit_status());
                            return ResultNextPyDataEvent::Err(
                                Error::new(
                                    ErrorKind::Other,
                                    format!(
                                        "Python process {} for file {:?} exited with error: {}",
                                        self.pyrunner.pid(),
                                        self.path,
                                        self.error.as_ref().unwrap_or(&String::from("Unknown error")),
                                    ),
                                )
                            );
                        }
                        // process exited okay
                        def1x!("empty fill_buffer, process exited with {:?}, return Done",
                            self.pyrunner.exit_status());
                        return ResultNextPyDataEvent::Done;
                    }
                    def1o!("empty fill_buffer, process not exited or exhausted, continue…");
                    continue;
                }
            };
        } // while !self.exited

        // if the process has exited badly then return an Error
        if self.exited && ! self.pyrunner.exit_okay()
        {
            const DEFAULT_MESSAGE: &[u8] = b"Unknown error";
            // ignore recently returned _stderr_data, presume we want *all*
            // the stderr data (which includes the recently returned data)
            // since the process return code was non-zero, the actual printed
            // error may have been in a prior call to `next()` due to delays
            // in the reading stderr data and polling that the process exited
            let stderr_data_all = self.pyrunner.stderr_all().unwrap_or(DEFAULT_MESSAGE);
            def1x!(
                "process exited with non-zero return code, return Error with {} bytes stderr",
                stderr_data_all.len()
            );
            let s = String::from_utf8_lossy(stderr_data_all);
            return ResultNextPyDataEvent::Err(
                Error::new(
                    ErrorKind::Other,
                    s,
                ));
        }

        def1x!("return ResultNextPyDataEvent::Done");

        ResultNextPyDataEvent::Done
    }

    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        &self.path
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        FileType::Etl { archival_type: FileTypeArchive::Normal }
    }

    /// File size in bytes
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.filesz
    }

    /// Update the two statistic `DateTimeL` of
    /// `self.dt_first_processed` and `self.dt_last_processed`.
    fn dm_first_last_update_processed(
        &mut self,
        dt: &DateTimeL,
    ) {
        if ! summary_stats_enabled() {
            return;
        }
        defñ!("({})", dt);
        // the earliest processed datetime
        match self.dt_first_processed {
            None => self.dt_first_processed = Some(*dt),
            Some(dt_) => {
                if &dt_ > dt {
                    self.dt_first_processed = Some(*dt);
                    self.out_of_order += 1;
                    defo!("dm_first_last_update_processed: out of order {} > {}", dt_, dt);
                }
            }
        }

        // the latest processed datetime
        match self.dt_last_processed {
            None => self.dt_last_processed = Some(*dt),
            Some(dt_) => {
                if &dt_ < dt {
                    self.dt_last_processed = Some(*dt);
                }
            }
        }
    }

    fn dm_first_last_update_accepted(
        &mut self,
        dt: &DateTimeL,
    ) {
        if ! summary_stats_enabled() {
            return;
        }
        defñ!("({})", dt);
        // the earliest accepted datetime
        match self.dt_first_accepted {
            None => self.dt_first_accepted = Some(*dt),
            Some(dt_) => {
                if &dt_ > dt {
                    self.dt_first_accepted = Some(*dt);
                }
            }
        }

        // the latest accepted datetime
        match self.dt_last_accepted {
            None => self.dt_last_accepted = Some(*dt),
            Some(dt_) => {
                if &dt_ < dt {
                    self.dt_last_accepted = Some(*dt);
                }
            }
        }
    }

    /// Return an up-to-date `SummaryPyEventReader` instance for this `PyEventReader`.
    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryPyEventReader {
        let pyeventreader_events_processed: Count = self.events_processed;
        let pyeventreader_events_accepted: Count = self.events_accepted;
        let pyeventreader_python_count_proc_reads_stdout: Count = self.pyrunner.count_proc_reads_stdout;
        let pyeventreader_python_count_proc_reads_stderr: Count = self.pyrunner.count_proc_reads_stderr;
        let pyeventreader_python_count_pipe_recv_stdout: Count = self.pyrunner.count_pipe_recv_stdout;
        let pyeventreader_python_count_pipe_recv_stderr: Count = self.pyrunner.count_pipe_recv_stderr;
        let pyeventreader_python_count_proc_writes: Count = self.pyrunner.count_proc_writes;
        let pyeventreader_python_count_proc_polls: Count = self.pyrunner.count_proc_polls;
        let pyeventreader_python_arguments: Vec<String> = self.python_arguments.clone();
        let pyeventreader_datetime_first_accepted = self.dt_first_accepted;
        let pyeventreader_datetime_last_accepted = self.dt_last_accepted;
        let pyeventreader_datetime_first_processed = self.dt_first_processed;
        let pyeventreader_datetime_last_processed = self.dt_last_processed;
        let pyeventreader_pipe_sz_stdout: PipeSz = self.pipe_sz_stdout();
        let pyeventreader_pipe_sz_stderr: PipeSz = self.pipe_sz_stderr();
        let pyeventreader_filesz: FileSz = self.filesz();
        let pyeventreader_events_read_max: Count = self.events_read_max;
        let pyeventreader_events_held_max: Count = self.events_held_max;
        let pyeventreader_out_of_order: Count = self.out_of_order;
        let pyeventreader_python_exit_status: Option<ExitStatus> = self.pyrunner.exit_status();
        let pyeventreader_duration_proc_wait: Duration = self.pyrunner.duration_proc_wait;
        let pyeventreader_duration_proc_run: Duration = self.pyrunner.duration().unwrap_or_default();

        SummaryPyEventReader {
            pyeventreader_events_processed,
            pyeventreader_events_accepted,
            pyeventreader_python_count_proc_reads_stdout,
            pyeventreader_python_count_proc_reads_stderr,
            pyeventreader_python_count_pipe_recv_stdout,
            pyeventreader_python_count_pipe_recv_stderr,
            pyeventreader_python_count_proc_writes,
            pyeventreader_python_count_proc_polls,
            pyeventreader_python_arguments,
            pyeventreader_datetime_first_accepted,
            pyeventreader_datetime_last_accepted,
            pyeventreader_datetime_first_processed,
            pyeventreader_datetime_last_processed,
            pyeventreader_pipe_sz_stdout,
            pyeventreader_pipe_sz_stderr,
            pyeventreader_filesz,
            pyeventreader_events_read_max,
            pyeventreader_events_held_max,
            pyeventreader_out_of_order,
            pyeventreader_python_exit_status,
            pyeventreader_duration_proc_wait,
            pyeventreader_duration_proc_run,
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `PyEventReader`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let path_ntf: Option<FPath> = match &self.named_temp_file {
            Some(ntf) => Some(path_to_fpath(ntf.path())),
            None => None,
        };
        let filetype = self.filetype();
        let logmessagetype = filetype.to_logmessagetype();
        let summaryetlreader: SummaryPyEventReader = self.summary();
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
            Some(summaryetlreader),
            None,
            None,
            error,
        )
    }
}
