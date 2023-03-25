// src/readers/evtxreader.rs

//! Implements a [`EvtxReader`],
//! the driver of deriving [`Etmpx`s] from a [Windows Event Log `.evtx` format]
//! file using [`EvtxParser`].
//!
//! Sibling of [`SyslogProcessor`]. But simpler in a number of ways due to
//! the predictable format of the evtx files.
//!
//! Evtx files may not store Events in chronological order. This means merging
//! evtx files may have errant behavior. See [Issue #86].
//!
//! [`EvtxReader`]: self::EvtxReader
//! [`Etmpx`s]: crate::data::etmpx::Etmpx
//! [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
//! [Windows Event Log `.evtx` format]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86

use crate::de_wrn;
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
    ResultS3,
};
use crate::data::datetime::{
    DateTimeLOpt,
    FixedOffset,
    Result_Filter_DateTime2,
    dt_pass_filters,
};
use crate::data::evtx::{Evtx, RecordId};
use crate::readers::summary::Summary;

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
#[allow(unused_imports)]
use ::si_trace_print::{de, defn, defo, defx, defñ, den, deo, dex, deñ, pfo, pfn, pfx};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// EvtxReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// [`EvtxReader.find`*] functions results.
///
/// [`EvtxReader.find`*]: self::EvtxReader#method.find_entry
pub type ResultS3EvtxFind = ResultS3<(RecordId, Evtx), Error>;

/// A wrapper for using [`EvtxParser`] to read a [evtx format file].
///
/// .evtx files in the wild were found to store entries in a non-chronological order, e.g. the
/// XML value at `Event.System.TimeCreated` are not necessarily in ascending order.
/// About 2/3 of the files on a long-running Windows 11 system were found to be in this
/// "out of order" state.
/// More accurately, using `evtx_dump` to dump a .evtx file displayed entries in non-chronological
/// order (so unlikely but possibly a problem with `evtx_dump`). Either way, that is the underlying
/// library used to read the .evtx files so it's a problem for this program.
/// This is not mitigated and may lead to unexpected behavior. See [Issue #86].
///
/// [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
/// [evtx format file]: https://github.com/libyal/libevtx/blob/main/documentation/Windows%20XML%20Event%20Log%20(EVTX).asciidoc
/// [Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86
pub struct EvtxReader {
    /// The internal [`EvtxParser`] that does the heavy lifting.
    ///
    /// [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
    evtxparser: EvtxParser<File>,
    /// The [`FPath`] of the file being read.
    ///
    /// [`FPath`]: crate::common::FPath
    path: FPath,
    /// `Count` of [`Evtx`s] processed.
    ///
    /// [`Evtx`s]: crate::data::evtx::Evtx
    pub(super) entries_processed: Box<Count>,
    /// `Count` of [`Evtx`s] accepted by the datetime filters.
    ///
    /// [`Evtx`s]: crate::data::evtx::Evtx
    pub(super) entries_accepted: Count,
    /// First (soonest) processed [`DateTimeL`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub(super) dt_first_processed: DateTimeLOpt,
    /// Last (latest) processed [`DateTimeL`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub(super) dt_last_processed: DateTimeLOpt,
    /// First (soonest) accepted (printed) [`DateTimeL`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub(super) dt_first_accepted: DateTimeLOpt,
    /// Last (latest) accepted (printed) [`DateTimeL`].
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub(super) dt_last_accepted: DateTimeLOpt,
    /// File Size of the file being read in bytes.
    filesz: FileSz,
    /// Internal tracking of "out of order" state.
    out_of_order_dt: Option<DateTime<Utc>>,
    /// Count of EVTX entries found to be out of order.
    out_of_order: Count,
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
            .field("Entries Processed", &*self.entries_processed)
            .field("Entries Accepted", &self.entries_accepted)
            .field("dt_first_accepted", &self.dt_first_accepted)
            .field("dt_last_accepted", &self.dt_last_accepted)
            .field("Error?", &self.error)
            .finish()
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryEvtxReader {
    pub evtxreader_entries_processed: Count,
    pub evtxreader_entries_accepted: Count,
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
        defn!("({:?})", path);

        // get the file size according to the file metadata
        let path_std: &Path = Path::new(&path);
        let mut open_options = FileOpenOptions::new();
        defo!("open_options.read(true).open({:?})", path);
        let file: File = match open_options
            .read(true)
            .open(path_std)
        {
            Ok(val) => val,
            Err(err) => {
                defx!("return {:?}", err);
                return Err(err);
            }
        };
        let filesz: FileSz = match file.metadata() {
            Ok(val) => val.len() as FileSz,
            Err(err) => {
                defx!("return {:?}", err);
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
        Ok(
            EvtxReader
            {
                evtxparser,
                path,
                entries_processed: Box::new(0),
                entries_accepted: 0,
                dt_first_processed: DateTimeLOpt::None,
                dt_last_processed: DateTimeLOpt::None,
                dt_first_accepted: DateTimeLOpt::None,
                dt_last_accepted: DateTimeLOpt::None,
                filesz,
                out_of_order_dt: None,
                out_of_order: 0,
                error: None,
            }
        )
    }

    pub fn next_between_datetime_filters(
        &mut self,
        dt_filter_after: DateTimeLOpt,
        dt_filter_before: DateTimeLOpt,
    ) -> impl Iterator<Item = ResultS3EvtxFind> + '_ {
        defñ!("({:?}, {:?})", dt_filter_after, dt_filter_before);
        // XXX: this sequence of linked iterator filters is somewhat messy
        //      due to closure lifetimes, and `move`s of the passed variables.
        //      The particular arrangement of accesses to local variables and
        //      `self.*` fields was found from trial and error.
        self.evtxparser.records()
            .inspect(
                // special inspection prior to handling the `result`
                // XXX: this is one closure that can acccess `self.*` fields
                //      cannot call `self.*` in some other linked iterator closures
                |result|
                {
                *self.entries_processed += 1;
                // monitor for "out of order" Events.
                // when two sequential Events have datetimes in the wrong
                // chronological order then increment `self.out_of_order`.
                // `dt_a` is earlier in the file so it should have earlier
                // datetime then current `timestamp`.
                // If not then increment `self.out_of_order`.
                // Issue #86
                let timestamp = match result {
                    Ok(serialrecord) => serialrecord.timestamp,
                    Err(_err) => {
                        return;
                    }
                };
                let dt_a = self.out_of_order_dt;
                self.out_of_order_dt = Some(timestamp);
                match (&dt_a, &self.out_of_order_dt) {
                    (&Some(dt_a), &Some(dt_b)) => {
                        if dt_a > dt_b {
                            self.out_of_order += 1;
                        }
                    }
                    _ => {}
                }
                // update `self.dt_first_processed` and `self.dt_last_processed`
                let timestamp = timestamp.with_timezone(
                    &FixedOffset::east_opt(0).unwrap()
                );
                match self.dt_first_processed.as_ref() {
                    Some(dt_first_) => {
                        if dt_first_ > &timestamp {
                            self.dt_first_processed = Some(
                                timestamp.with_timezone(
                                        &FixedOffset::east_opt(0).unwrap()
                                    )
                            );
                        }
                    }
                    None => {
                        self.dt_first_processed = Some(
                            timestamp.with_timezone(
                                &FixedOffset::east_opt(0).unwrap()
                            )
                        );
                    }
                }
                match self.dt_last_processed.as_ref() {
                    Some(dt_last_) => {
                        if dt_last_ < &timestamp {
                            self.dt_last_processed = Some(
                                timestamp.with_timezone(
                                    &FixedOffset::east_opt(0).unwrap()
                                )
                            );
                        }
                    }
                    None => {
                        self.dt_last_processed = Some(
                            timestamp.with_timezone(
                                &FixedOffset::east_opt(0).unwrap()
                            )
                        );
                    }
                }
            })
            // XXX: .evtx files may not have Events in chronological order.
            //      so every Event must be checked for acceptable datetime range.
            //      This is annoying because:
            //      1. the Entry will not be printed for the user in the correct
            //         chronological order.
            //      2. the entirety file of every .evtx file must be processed.
            //      If .evtx file was in chronological order then this could
            //      be `take_while` instead of `filter` which would be
            //      faster.
            //      Issue #86
            .filter(
                // take records while they are in the datetime range
                // cannot access any `self.*` in this closure
                move |result|
                {
                    defn!("take_while(result<SerializedEvtxRecord>)");
                    match result {
                        Ok(serialrecord) => {
                            match (&dt_filter_after, &dt_filter_before) {
                                (&None, &None) => {
                                    // if no filters then skip upcoming
                                    // timestamp conversion
                                    return true;
                                }
                                _ => {}
                            }
                            let datetime = serialrecord.timestamp.with_timezone(
                                &FixedOffset::east_opt(0).unwrap()
                            );
                            match dt_pass_filters(
                                &datetime,
                                &dt_filter_after,
                                &dt_filter_before,
                            ) {
                                Result_Filter_DateTime2::InRange => {
                                    defx!("take_while(result<SerializedEvtxRecord>): InRange");

                                    true
                                }
                                Result_Filter_DateTime2::BeforeRange => {
                                    defx!("take_while(result<SerializedEvtxRecord>): BeforeRange");

                                    false
                                }
                                Result_Filter_DateTime2::AfterRange => {
                                    defx!("take_while(result<SerializedEvtxRecord>): AfterRange");

                                    false
                                }
                            }
                        }
                        Err(_error) => {
                            defx!("take_while(result<SerializedEvtxRecord>): Error: {}", _error);
                            de_wrn!("take_while(result<SerializedEvtxRecord>): Error: {}", _error);
                            // XXX: cannot call `self.set_evtxerror(err)` here due to rustc error:
                            //      cannot move out of `self` because it is borrowed

                            false
                        }
                    }
                }
            )
            .inspect(
                // track various statistics about the returned `SerializedEvtxRecord`
                // XXX: this is one closure that can acccess `self.*` fields
                //      cannot call `self.*` in some other linked iterator closures
                |result| {
                    defn!("inspect(result<SerializedEvtxRecord>):");
                    match result {
                        Ok(serialrecord) => {
                            // XXX: would prefer to call `self.dt_first_last_update` here
                            //      but due to lifetime issues, it is not possible (or extremely tedious).
                            //      compiler complains:
                            //           evtxreader.rs(362, 9): borrow occurs here
                            //           evtxreader.rs(407, 29): second borrow occurs due to use of `*self` in closure
                            //           evtxreader.rs(357, 9): let's call the lifetime of this reference `'1`
                            //           evtxreader.rs(362, 9): returning this value requires that `self.evtxparser` is borrowed for `'1`
                            let timestamp = &serialrecord.timestamp;
                            match self.dt_first_accepted.as_ref() {
                                Some(dt_first_) => {
                                    if dt_first_ > timestamp {
                                        self.dt_first_accepted = Some(
                                            timestamp.with_timezone(
                                                    &FixedOffset::east_opt(0).unwrap()
                                                )
                                        );
                                    }
                                }
                                None => {
                                    self.dt_first_accepted = Some(
                                        timestamp.with_timezone(
                                            &FixedOffset::east_opt(0).unwrap()
                                        )
                                    );
                                }
                            }
                            match self.dt_last_accepted.as_ref() {
                                Some(dt_last_) => {
                                    if dt_last_ < timestamp {
                                        self.dt_last_accepted = Some(
                                            timestamp.with_timezone(
                                                &FixedOffset::east_opt(0).unwrap()
                                            )
                                        );
                                    }
                                }
                                None => {
                                    self.dt_last_accepted = Some(
                                        timestamp.with_timezone(
                                            &FixedOffset::east_opt(0).unwrap()
                                        )
                                    );
                                }
                            }
                            self.entries_accepted += 1;
                            defx!("inspect(serialrecord): Ok; entries_accepted: {}", self.entries_accepted);
                        }
                        Err(error) => {
                            defx!("inspect(serialrecord): Error: {}", error);
                            de_wrn!("inspect(serialrecord): Error: {}", error);
                            self.error = Some(error.to_string());
                        }
                    }
                }
            )
            .map(
                // map `Result<SerializedEvtxRecord>` to `ResultS3EvtxFind`
                |resultserializedrecord|
                {
                    defñ!(".map(resultserializedrecord)");
                    match Evtx::from_resultserializedrecord(&resultserializedrecord) {
                        Ok(evtx) => {
                            ResultS3EvtxFind::Found((0, evtx))
                        }
                        Err(error) => {
                            // XXX: cannot set `self.error` here due to:
                            //      two closures require unique access to `self.error` at the same time
                            //      second closure is constructed here
                            de_wrn!("Evtx::from_resultserializedrecord: {}", error);
                            ResultS3EvtxFind::Err(error)
                        }
                    }
                }
            )
    }

    /// `Count` of `Evtx`s processed by this `EvtxReader`
    /// (i.e. `self.entries_processed`).
    #[inline(always)]
    pub fn count_entries_processed(&self) -> Count {
        *self.entries_processed
    }

    #[inline(always)]
    pub fn count_entries_accepted(&self) -> Count {
        self.entries_accepted
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

    /// Return an up-to-date `SummaryEvtxReader` instance for this
    /// `EvtxReader`.
    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryEvtxReader {
        let evtxreader_entries_processed: Count = self.count_entries_processed();
        let evtxreader_entries_accepted: Count = self.count_entries_accepted();
        let evtxreader_datetime_first_processed = self.dt_first_processed;
        let evtxreader_datetime_last_processed = self.dt_last_processed;
        let evtxreader_datetime_first_accepted = self.dt_first_accepted;
        let evtxreader_datetime_last_accepted = self.dt_last_accepted;
        let evtxreader_filesz = self.filesz();
        let evtxreader_out_of_order = self.out_of_order;

        SummaryEvtxReader {
            evtxreader_entries_processed,
            evtxreader_entries_accepted,
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
            error,
        )
    }
}
