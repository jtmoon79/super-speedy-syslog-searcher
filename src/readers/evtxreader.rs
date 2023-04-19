// src/readers/evtxreader.rs

//! Implements a [`EvtxReader`],
//! the driver of deriving [`Etmpx`s] from a [Windows Event Log `.evtx` format]
//! file using [`EvtxParser`].
//!
//! Sibling of [`SyslogProcessor`]. But simpler in a number of ways due to
//! the predictable format of the evtx files.
//!
//! Implements [Issue #87] and [Issue #86].
//!
//! [`EvtxReader`]: self::EvtxReader
//! [`Etmpx`s]: crate::data::evtx::Evtx
//! [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
//! [Windows Event Log `.evtx` format]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86
//! [Issue #87]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/87

use crate::common::{
    File,
    FileOpenOptions,
};
use crate::common::{
    Count,
    FileSz,
    FPath,
    FileType,
    filetype_to_logmessagetype,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    Result_Filter_DateTime2,
};
use crate::data::evtx::Evtx;
use crate::readers::summary::Summary;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::io::{Error, ErrorKind, Result};

use ::chrono::{
    DateTime,
    Utc,
};
use ::evtx::{
    EvtxParser,
    ParserSettings,
};
use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::more_asserts::{assert_le, debug_assert_ge, debug_assert_le, debug_assert_lt};
#[allow(unused_imports)]
use ::si_trace_print::{
    de,
    defn,
    def1n,
    defo,
    def1o,
    defx,
    def1x,
    defñ,
    def1ñ,
    den,
    deo,
    dex,
    deñ,
    pfo,
    pfn,
    pfx,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// EvtxReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// The `DateTime` used by [`EvtxParser`], field [`EvtxRecord.timestamp`] which
/// is referred to as a "timestamp".
///
/// [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
/// [`EvtxRecord.timestamp`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxRecord.html#structfield.timestamp
pub type Timestamp = DateTime<Utc>;
/// Optional [`Timestamp`].
pub type TimestampOpt = Option<Timestamp>;

pub type EventsKey = (Timestamp, usize);
pub type Events = BTreeMap<EventsKey, Evtx>;

lazy_static! {
    static ref FO_0: FixedOffset = FixedOffset::east_opt(0).unwrap();
}

/// Convert a `evtx` "timestamp" (`DateTime<Utc>`)
/// to a `s4` "datetime" (`DateTimeL`).
pub fn timestamp_to_datetimel(
    timestamp: &Timestamp,
) -> DateTimeL {
    timestamp.with_timezone(
        &FixedOffset::east_opt(0).unwrap()
    )
}

/// Convert a `s4` "datetime" (`DateTimeL`)
/// to a `evtx` "timestamp" (`DateTime<Utc>`).
pub fn datetimel_to_timestamp(
    datetime: &DateTimeL,
) -> Timestamp {
    datetime.with_timezone(&Utc)
}

/// Convert a `s4` "datetime" (`DateTimeL`)
/// to a `evtx` "timestamp" (`DateTime<Utc>`).
pub fn datetimelopt_to_timestampopt(
    datetimeopt: &DateTimeLOpt,
) -> TimestampOpt {
    match datetimeopt {
        Some(dt) => {
            Some(datetimel_to_timestamp(dt))
        }
        None => None,
    }
}

/// A version of [`dt_pass_filters`] that takes a `Timestamp` instead of a
/// [`DateTimeL`].
///
/// [`dt_pass_filters`]: crate::data::datetime::dt_pass_filters
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
pub fn ts_pass_filters(
    ts: &Timestamp,
    ts_filter_after: &TimestampOpt,
    ts_filter_before: &TimestampOpt,
) -> Result_Filter_DateTime2 {
    defn!("({:?}, {:?}, {:?})", ts, ts_filter_after, ts_filter_before);
    match (ts_filter_after, ts_filter_before) {
        (None, None) => {
            defx!("return InRange; (no dt filters)");
            return Result_Filter_DateTime2::InRange;
        }
        (Some(da), Some(db)) => {
            debug_assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
            if ts < da {
                defx!("return BeforeRange");
                return Result_Filter_DateTime2::BeforeRange;
            }
            if db < ts {
                defx!("return AfterRange");
                return Result_Filter_DateTime2::AfterRange;
            }
            // assert da < dt && ts < db
            debug_assert_le!(da, ts, "Unexpected range values da ts");
            debug_assert_le!(ts, db, "Unexpected range values ts db");
            defx!("return InRange");
    
            Result_Filter_DateTime2::InRange
        }
        (Some(da), None) => {
            if ts < da {
                defx!("return BeforeRange");
                return Result_Filter_DateTime2::BeforeRange;
            }
            defx!("return InRange");
    
            Result_Filter_DateTime2::InRange
        }
        (None, Some(db)) => {
            if db < ts {
                defx!("return AfterRange");
                return Result_Filter_DateTime2::AfterRange;
            }
            defx!("return InRange");
    
            Result_Filter_DateTime2::InRange
        }
    }
}

/// A wrapper for using [`EvtxParser`] to read a [evtx format file].
///
/// .evtx files in the wild were found to store event in a non-chronological
/// order, e.g. the XML value at `Event.System.TimeCreated` are not
/// necessarily in ascending order.
/// About 2/3 of the files on a long-running Windows 11 system were found to be
/// in this "out of order" state.
/// More accurately, using `evtx_dump` to dump a .evtx file displayed events in
/// non-chronological order (so unlikely but possibly a problem with
/// `evtx_dump`). Either way, that is the underlying
/// library used to read the .evtx files so it's a problem for this program.
/// This `EvtxReader` wrapper sorts the events by timestamp and then by
/// order of enumeration.
/// Unfortunately, this means the entire file must be read into memory before
/// Events can be further processed (compared to other log messages) and then
/// printed.
/// Also see [Issue #86].
///
/// [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
/// [evtx format file]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
/// [Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86
pub struct EvtxReader {
    /// The internal [`EvtxParser`] that does the heavy lifting.
    ///
    /// [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
    evtxparser: EvtxParser<File>,
    /// The [`Evtx`]s read from the file, sorted by timestamp and then by
    /// enumeration order.
    events: Events,
    /// The [`FPath`] of the file being read.
    ///
    /// [`FPath`]: crate::common::FPath
    path: FPath,
    /// `Count` of [`Evtx`s] processed.
    ///
    /// [`Evtx`s]: crate::data::evtx::Evtx
    //pub(super) events_processed: Box<Count>,
    pub(super) events_processed: Count,
    /// `Count` of [`Evtx`s] accepted by the datetime filters.
    ///
    /// [`Evtx`s]: crate::data::evtx::Evtx
    pub(super) events_accepted: Count,
    /// First (soonest) processed [`Timestamp`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`Timestamp`]: Timestamp
    pub(super) ts_first_processed: TimestampOpt,
    /// Last (latest) processed [`Timestamp`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`Timestamp`]: Timestamp
    pub(super) ts_last_processed: TimestampOpt,
    /// First (soonest) accepted (printed) [`Timestamp`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`Timestamp`]: Timestamp
    pub(super) ts_first_accepted: TimestampOpt,
    /// Last (latest) accepted (printed) [`Timestamp`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`Timestamp`]: Timestamp
    pub(super) ts_last_accepted: TimestampOpt,
    /// File Size of the file being read in bytes.
    filesz: FileSz,
    /// Internal tracking of "`out_of_order`" state.
    //out_of_order_dt: Option<DateTime<Utc>>,
    /// Count of EVTX entries found to be out of order.
    out_of_order: Count,
    /// has `self.analyze()` been called?
    analyzed: bool,
    /// The last [`Error`], if any, as a `String`
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    error: Option<String>,
}

impl<'a> fmt::Debug for EvtxReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("EvtxReader")
            .field("Path", &self.path)
            .field("Events Processed", &self.events_processed)
            .field("Events Accepted", &self.events_accepted)
            .field("ts_first_accepted", &self.ts_first_accepted)
            .field("ts_last_accepted", &self.ts_last_accepted)
            .field("Error?", &self.error)
            .finish()
    }
}

// TODO: [2023/04] remove redundant variable prefix name `evtxreader_`
#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryEvtxReader {
    pub evtxreader_events_processed: Count,
    pub evtxreader_events_accepted: Count,
    /// datetime soonest processed
    pub evtxreader_datetime_first_processed: DateTimeLOpt,
    /// datetime latest processed
    pub evtxreader_datetime_last_processed: DateTimeLOpt,
    /// datetime soonest accepted (printed)
    pub evtxreader_datetime_first_accepted: DateTimeLOpt,
    /// datetime latest accepted (printed)
    pub evtxreader_datetime_last_accepted: DateTimeLOpt,
    pub evtxreader_filesz: FileSz,
    pub evtxreader_out_of_order: Count,
}

/// Implement the EvtxReader.
impl<'a> EvtxReader {
    /// Create a new `EvtxReader`.
    pub fn new(
        path: FPath,
    ) -> Result<EvtxReader> {
        def1n!("({:?})", path);

        // get the file size according to the file metadata
        let path_std: &Path = Path::new(&path);
        let mut open_options = FileOpenOptions::new();
        def1o!("open_options.read(true).open({:?})", path);
        let file: File = match open_options
            .read(true)
            .open(path_std)
        {
            Ok(val) => val,
            Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let filesz: FileSz = match file.metadata() {
            Ok(val) => val.len() as FileSz,
            Err(err) => {
                def1x!("return {:?}", err);
                eprintln!("ERROR: File::metadata() path {:?} {}", path_std, err);
                return Err(err);
            }
        };

        // create the EvtxParser
        let settings = ParserSettings::default().num_threads(0);
        let evtxparser: EvtxParser<File> = match EvtxParser::from_path(&path) {
            Ok(evtxparser) => evtxparser.with_configuration(settings),
            Err(err) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("EvtxParser::from_path({:?}): {}", path, err),
                ));
            }
        };
        def1x!();
        Ok(
            EvtxReader
            {
                evtxparser,
                events: Events::new(),
                path,
                events_processed: 0,
                events_accepted: 0,
                ts_first_processed: TimestampOpt::None,
                ts_last_processed: TimestampOpt::None,
                ts_first_accepted: TimestampOpt::None,
                ts_last_accepted: TimestampOpt::None,
                filesz,
                out_of_order: 0,
                analyzed: false,
                error: None,
            }
        )
    }

    /// Read the entire file and store in order.
    ///
    /// This should be called once before reading the via `next`.
    // TODO: [2023/03/26] add handling of files without "out of order" events.
    //       much more efficient and worth the divergent code paths.
    pub fn analyze(
        &mut self,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) {
        defn!("({:?}, {:?})", dt_filter_after, dt_filter_before);
        let ts_filter_after = datetimelopt_to_timestampopt(dt_filter_after);
        let ts_filter_before = datetimelopt_to_timestampopt(dt_filter_before);
        let mut timestamp_last: TimestampOpt = TimestampOpt::None;
        for (index, result) in self.evtxparser.records().enumerate() {
            match result {
                Ok(record) => {
                    // update fields *processed
                    self.events_processed += 1;
                    match self.ts_first_processed.as_ref() {
                        Some(ts_first_) => {
                            if ts_first_ > &record.timestamp {
                                self.ts_first_processed = Some(record.timestamp);
                            }
                        }
                        None => self.ts_first_processed = Some(record.timestamp),
                    }
                    match self.ts_last_processed.as_ref() {
                        Some(ts_last_) => {
                            if ts_last_ < &record.timestamp {
                                self.ts_last_processed = Some(record.timestamp);
                            }
                        }
                        None => self.ts_last_processed = Some(record.timestamp),
                    }
                    // update "out of order" counter
                    if let Some(ts_last_) = timestamp_last.as_ref() {
                        if ts_last_ > &record.timestamp {
                            self.out_of_order += 1;
                        }
                    }
                    timestamp_last = Some(record.timestamp);

                    // filter by date
                    match ts_pass_filters(
                        &record.timestamp,
                        &ts_filter_after,
                        &ts_filter_before,
                    ) {
                        Result_Filter_DateTime2::InRange => {
                            defo!("InRange");
                        }
                        Result_Filter_DateTime2::BeforeRange => {
                            defo!("BeforeRange");
                            continue;
                        }
                        Result_Filter_DateTime2::AfterRange => {
                            defo!("AfterRange");
                            continue;
                        }
                    }

                    let timestamp = record.timestamp;
                    let evtx =
                        Evtx::from_evtxrs(&record);
                    self.events.insert((timestamp, index), evtx);

                    // update fields *accepted
                    self.events_accepted += 1;
                    match self.ts_first_accepted.as_ref() {
                        Some(ts_first_) => {
                            if ts_first_ > &timestamp {
                                self.ts_first_accepted = Some(timestamp);
                            }
                        }
                        None => self.ts_first_accepted = Some(timestamp),
                    }
                    match self.ts_last_accepted.as_ref() {
                        Some(ts_last_) => {
                            if ts_last_ < &timestamp {
                                self.ts_last_accepted = Some(timestamp);
                            }
                        }
                        None => self.ts_last_accepted = Some(timestamp),
                    }
                }
                Err(err) => {
                    self.error = Some(err.to_string());
                }
            }
        }
        self.analyzed = true;

        defx!();
    }

    pub fn next(
        &mut self,
    ) -> Option<Evtx> {
        def1ñ!();
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next()`");

        self.events.pop_first().map(|(_key, evtx)| evtx)
    }

    /// `Count` of `Evtx`s processed by this `EvtxReader`
    /// (i.e. `self.events_processed`).
    #[inline(always)]
    pub fn count_events_processed(&self) -> Count {
        self.events_processed
    }

    #[inline(always)]
    pub fn count_events_accepted(&self) -> Count {
        self.events_accepted
    }

    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        &self.path
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        FileType::Evtx
    }

    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.filesz
    }

    /// return the `DateTimeL` of the first `Evtx` processed
    pub fn dt_first_processed(&self) -> DateTimeLOpt {
        match self.ts_first_processed {
            TimestampOpt::None => DateTimeLOpt::None,
            TimestampOpt::Some(ts) =>
                DateTimeLOpt::Some(timestamp_to_datetimel(&ts)),
        }
    }

    /// return the `DateTimeL` of the last `Evtx` processed
    pub fn dt_last_processed(&self) -> DateTimeLOpt {
        match self.ts_last_processed {
            TimestampOpt::None => DateTimeLOpt::None,
            TimestampOpt::Some(ts) =>
                DateTimeLOpt::Some(timestamp_to_datetimel(&ts)),
        }
    }

    /// return the `DateTimeL` of the first `Evtx` accepted by the datetime
    /// filters
    pub fn dt_first_accepted(&self) -> DateTimeLOpt {
        match self.ts_first_accepted {
            TimestampOpt::None => DateTimeLOpt::None,
            TimestampOpt::Some(ts) =>
                DateTimeLOpt::Some(timestamp_to_datetimel(&ts)),
        }
    }

    /// return the `DateTimeL` of the last `Evtx` accepted by the datetime
    /// filters
    pub fn dt_last_accepted(&self) -> DateTimeLOpt {
        match self.ts_last_accepted {
            TimestampOpt::None => DateTimeLOpt::None,
            TimestampOpt::Some(ts) =>
                DateTimeLOpt::Some(timestamp_to_datetimel(&ts)),
        }
    }

    /// Return an up-to-date `SummaryEvtxReader` instance for this
    /// `EvtxReader`.
    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryEvtxReader {
        let evtxreader_events_processed: Count = self.count_events_processed();
        let evtxreader_events_accepted: Count = self.count_events_accepted();
        let evtxreader_datetime_first_processed = self.dt_first_processed();
        let evtxreader_datetime_last_processed = self.dt_last_processed();
        let evtxreader_datetime_first_accepted = self.dt_first_accepted();
        let evtxreader_datetime_last_accepted = self.dt_last_accepted();
        let evtxreader_filesz = self.filesz();
        let evtxreader_out_of_order = self.out_of_order;

        SummaryEvtxReader {
            evtxreader_events_processed,
            evtxreader_events_accepted,
            evtxreader_datetime_first_processed,
            evtxreader_datetime_last_processed,
            evtxreader_datetime_first_accepted,
            evtxreader_datetime_last_accepted,
            evtxreader_filesz,
            evtxreader_out_of_order,
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `EvtxReader`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let filetype = self.filetype();
        let logmessagetype = filetype_to_logmessagetype(filetype);
        let summaryevtxreader = self.summary();
        let error: Option<String> = self.error.clone();

        Summary::new(
            path,
            filetype,
            logmessagetype,
            None,
            None,
            None,
            None,
            None,
            Some(summaryevtxreader),
            None,
            error,
        )
    }
}
