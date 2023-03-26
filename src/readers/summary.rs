// src/readers/summary.rs

//! Implements `Summary` statistics tracking struct.

#![allow(non_snake_case)]

use crate::common::{
    Count,
    FPath,
    FileType,
    LogMessageType,
};
use crate::common::debug_panic;
use crate::data::datetime::DateTimeLOpt;
#[allow(unused_imports)]
use crate::debug::printers::{de_err, de_wrn, e_err, e_wrn};
use crate::readers::blockreader::{
    BlockSz,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
    SummaryBlockReader,
};
use crate::readers::linereader::SummaryLineReader;
use crate::readers::syslinereader::SummarySyslineReader;
use crate::readers::syslogprocessor::SummarySyslogProcessor;
use crate::readers::utmpxreader::SummaryUtmpxReader;
use crate::readers::evtxreader::SummaryEvtxReader;

use std::fmt;

use ::min_max::max;
use ::more_asserts::{debug_assert_ge, debug_assert_le};
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[macro_export]
macro_rules! debug_assert_none {
    ($($arg:expr),+) => {
        $(
            if cfg!(debug_assertions) {
                match $arg {
                    None => {}
                    Some(..) => panic!("'{}' is not None", stringify!($arg))
                }
            }
        )+
    };
}

/// wrapper for various `Summary*` data types for different files corresponding
/// to [`LogMessage`] variants.
///
/// [`LogMessage`]: s4::LogMessage
#[derive(Clone, Default)]
pub enum SummaryReaderData {
    /// Unset. Useful for stand-in value where nothing actually occurred; e.g.
    /// files without adequate read permissions.
    #[default]
    Dummy,
    /// For a [`SyslogProcessor`] and underlying readers.
    ///
    /// [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
    Syslog((SummaryBlockReader, SummaryLineReader, SummarySyslineReader, SummarySyslogProcessor)),
    /// For a [`UtmpxReader`] and underlying readers.
    ///
    /// [`UtmpxReader`]: crate::readers::utmpxreader::UtmpxReader
    Utmpx((SummaryBlockReader, SummaryUtmpxReader)),
    Etvx(SummaryEvtxReader)
}

impl SummaryReaderData {
    pub fn is_dummy(&self) -> bool {
        match self {
            SummaryReaderData::Dummy => true,
            _ => false,
        }
    }
}

/// Accumulated statistics about processing and printing activity of a single
/// file processed by a `SyslineReader` and it's underlying `LineReader` and
/// it's underlying `BlockReader`, _or_ a `UtmpxReader` and it's underlying
/// `BlockReader`.
///
/// For CLI option `--summary`.
#[derive(Clone, Default)]
pub struct Summary {
    /// the `FPath` of the processed file
    pub path: FPath,
    /// the `FileType` of the processed file
    pub filetype: FileType,
    /// the `LogMessageType` of the processed file
    pub logmessagetype: LogMessageType,
    /// Data specific to the particular readers and processors.
    ///
    /// When `logmessagetype` is [`LogMessageType::Sysline`] then this must be
    /// [`SummaryReaderData::Syslog`].
    ///
    /// When `logmessagetype` is [`LogMessageType::Sysline`] then this must be
    /// [`SummaryReaderData::Utmpx`].
    pub readerdata: SummaryReaderData,
    /// The first encountered [`Error`], if any, as a `String`.
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    pub error: Option<String>,
}

impl Summary {
    /// Create a new `Summary`
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: FPath,
        filetype: FileType,
        logmessagetype: LogMessageType,
        summaryblockreader_opt: Option<SummaryBlockReader>,
        summarylinereader_opt: Option<SummaryLineReader>,
        summarysyslinereader_opt: Option<SummarySyslineReader>,
        summarysyslogprocessor_opt: Option<SummarySyslogProcessor>,
        summaryutmpreader_opt: Option<SummaryUtmpxReader>,
        summaryevtxreader_opt: Option<SummaryEvtxReader>,
        error: Option<String>,
    ) -> Summary {
        #[cfg(debug_assertions)]
        match summaryblockreader_opt.as_ref() {
            // XXX: random sanity checks
            Some(summaryblockreader) => {
                debug_assert_ge!(summaryblockreader.blockreader_bytes, summaryblockreader.blockreader_blocks, "There is less bytes than Blocks");
                debug_assert_ge!(summaryblockreader.blockreader_blocksz, BLOCKSZ_MIN, "blocksz too small");
                debug_assert_le!(summaryblockreader.blockreader_blocksz, BLOCKSZ_MAX, "blocksz too big");
            }
            None => {}
        }
        // XXX: in case of a file without datetime stamp year, syslines may be reprocessed.
        //      the count of syslines processed may reflect reprocessing the same line in the file,
        //      leading to a `syslinereader_syslines` that is more than `linereader_lines`.
        //      See `syslogprocessor.process_missing_year()`.
        //debug_assert_ge!(linereader_lines, syslinereader_syslines, "There is less Lines than Syslines");
        match logmessagetype {
            LogMessageType::Sysline => {
                debug_assert_none!(
                    summaryutmpreader_opt,
                    summaryevtxreader_opt
                );
                let summaryblockreader = summaryblockreader_opt.unwrap();
                let summarylinereader = summarylinereader_opt.unwrap();
                let summarysyslinereader = summarysyslinereader_opt.unwrap();
                let summarysyslogprocessor = summarysyslogprocessor_opt.unwrap();
                debug_assert_ge!(summaryblockreader.blockreader_bytes, summarylinereader.linereader_lines, "There is less bytes than Lines");
                debug_assert_ge!(summaryblockreader.blockreader_bytes, summarysyslinereader.syslinereader_syslines, "There is less bytes than Syslines");
                let readerdata: SummaryReaderData = SummaryReaderData::Syslog(
                    (
                        summaryblockreader,
                        summarylinereader,
                        summarysyslinereader,
                        summarysyslogprocessor,
                    ),
                );
                Summary {
                    path,
                    filetype,
                    logmessagetype,
                    readerdata,
                    error,
                }
            }
            LogMessageType::Utmpx => {
                debug_assert_none!(
                    summarylinereader_opt,
                    summarysyslinereader_opt,
                    summarysyslogprocessor_opt,
                    summaryevtxreader_opt
                );
                let summaryblockreader = summaryblockreader_opt.unwrap();
                let summaryutmpreader = summaryutmpreader_opt.unwrap();
                debug_assert_ge!(summaryblockreader.blockreader_bytes, summaryutmpreader.utmpxreader_utmp_entries, "There is less bytes than Utmpx Entries");
                let readerdata: SummaryReaderData = SummaryReaderData::Utmpx(
                    (
                        summaryblockreader,
                        summaryutmpreader,
                    ),
                );
                Summary {
                    path,
                    filetype,
                    logmessagetype,
                    readerdata,
                    error,
                }
            }
            LogMessageType::Evtx => {
                debug_assert_none!(
                    summaryblockreader_opt,
                    summarylinereader_opt,
                    summarysyslinereader_opt,
                    summarysyslogprocessor_opt,
                    summaryutmpreader_opt
                );
                let summaryevtxreader = summaryevtxreader_opt.unwrap();
                let readerdata: SummaryReaderData = SummaryReaderData::Etvx(
                    summaryevtxreader
                );
                Summary {
                    path,
                    filetype,
                    logmessagetype,
                    readerdata,
                    error,
                }
            }
            LogMessageType::All => {
                panic!("LogMessageType::All is not supported");
            }
        }
    }

    /// Create a new `Summary` with limited known data.
    /// Useful for files that failed to process due to errors
    /// (e.g. PermissionDenied, etc.).
    #[allow(clippy::too_many_arguments)]
    pub fn new_failed(
        path: FPath,
        filetype: FileType,
        logmessagetype: LogMessageType,
        blockreader_blocksz: BlockSz,
        error: Option<String>,
    ) -> Summary {
        // some sanity checks
        debug_assert_ge!(blockreader_blocksz, BLOCKSZ_MIN, "blocksz too small");
        debug_assert_le!(blockreader_blocksz, BLOCKSZ_MAX, "blocksz too big");

        Summary {
            path,
            filetype,
            logmessagetype,
            readerdata: SummaryReaderData::Dummy,
            error,
            ..Default::default()
        }
    }

    /// helper to get optional `SummaryBlockReader` reference
    pub fn blockreader(&self) -> Option<&SummaryBlockReader> {
        match &self.readerdata {
            SummaryReaderData::Dummy => {
                // `Dummy` can occur for files without adequate read permissions
                panic!("Summary::blockreader() called on readerdata type SummaryReaderData::Dummy");
            },
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    _summarylinereader,
                    _summarysyslinereader,
                    _summarysyslogprocessor,
                )
            ) => Some(summaryblockreader),
            SummaryReaderData::Utmpx(
                (
                    summaryblockreader,
                    _summaryutmpreader,
                )
            ) => Some(summaryblockreader),
            SummaryReaderData::Etvx(_) => None,
        }
    }

    pub fn has_blockreader(&self) -> bool {
        self.blockreader().is_some()
    }

    /// chronologically earliest printed datetime in file
    pub fn datetime_first(&self) -> &DateTimeLOpt {
        match &self.readerdata {
            SummaryReaderData::Dummy => panic!("Summary::datetime_first() called on Summary::Dummy"),
            SummaryReaderData::Syslog(
                (
                    _summaryblockreader,
                    _summarylinereader,
                    summarysyslinereader,
                    _summarysyslogprocessor,
                )
            ) => &summarysyslinereader.syslinereader_datetime_first,
            SummaryReaderData::Utmpx(
                (
                    _summaryblockreader,
                    summaryutmpreader,
                )
            ) => &summaryutmpreader.utmpxreader_datetime_first,
            SummaryReaderData::Etvx(summaryevtxreader)
                => &summaryevtxreader.evtxreader_datetime_first_accepted,
        }
    }

    /// chronologically latest printed datetime in file
    pub fn datetime_last(&self) -> &DateTimeLOpt {
        match &self.readerdata {
            SummaryReaderData::Dummy => panic!("Summary::datetime_last() called on Summary::Dummy"),
            SummaryReaderData::Syslog(
                (
                    _summaryblockreader,
                    _summarylinereader,
                    summarysyslinereader,
                    _summarysyslogprocessor,
                )
            ) => &summarysyslinereader.syslinereader_datetime_last,
            SummaryReaderData::Utmpx(
                (
                    _summaryblockreader,
                    summaryutmpreader,
                )
            ) => &summaryutmpreader.utmpxreader_datetime_last,
            SummaryReaderData::Etvx(summaryevtxreader)
                => &summaryevtxreader.evtxreader_datetime_last_accepted,
        }
    }

    /// Return maximum value for hit/miss/insert number.
    ///
    /// Helpful to format teriminal column widths.
    pub fn max_hit_miss(&self) -> Count {
        match &self.readerdata {
            SummaryReaderData::Dummy => 0,
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    summarylinereader,
                    summarysyslinereader,
                    _summarysyslogprocessor,
                )
            ) => {
                max!(
                    summaryblockreader.blockreader_read_block_lru_cache_hit,
                    summaryblockreader.blockreader_read_block_lru_cache_miss,
                    summaryblockreader.blockreader_read_block_lru_cache_put,
                    summaryblockreader.blockreader_read_blocks_hit,
                    summaryblockreader.blockreader_read_blocks_miss,
                    summaryblockreader.blockreader_read_blocks_put,
                    summarylinereader.linereader_lines_hits,
                    summarylinereader.linereader_lines_miss,
                    summarylinereader.linereader_find_line_lru_cache_hit,
                    summarylinereader.linereader_find_line_lru_cache_miss,
                    summarylinereader.linereader_find_line_lru_cache_put,
                    summarysyslinereader.syslinereader_syslines_hit,
                    summarysyslinereader.syslinereader_syslines_miss,
                    summarysyslinereader.syslinereader_syslines_by_range_hit,
                    summarysyslinereader.syslinereader_syslines_by_range_miss,
                    summarysyslinereader.syslinereader_syslines_by_range_put,
                    summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
                    summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
                    summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
                    summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
                    summarysyslinereader.syslinereader_find_sysline_lru_cache_put,
                    summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
                    summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
                    summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_put,
                    summarysyslinereader.syslinereader_get_boxptrs_singleptr,
                    summarysyslinereader.syslinereader_get_boxptrs_doubleptr,
                    summarysyslinereader.syslinereader_get_boxptrs_multiptr
                )
            }
            SummaryReaderData::Utmpx(
                (
                    summaryblockreader,
                    summaryutmpreader,
                )
            ) => {
                max!(
                    summaryblockreader.blockreader_read_block_lru_cache_hit,
                    summaryblockreader.blockreader_read_block_lru_cache_miss,
                    summaryblockreader.blockreader_read_block_lru_cache_put,
                    summaryblockreader.blockreader_read_blocks_hit,
                    summaryblockreader.blockreader_read_blocks_miss,
                    summaryblockreader.blockreader_read_blocks_put,
                    summaryutmpreader.utmpxreader_utmp_entries_max,
                    summaryutmpreader.utmpxreader_utmp_entries_miss
                )
            }
            SummaryReaderData::Etvx(summaryevtxreader) => {
                max!(
                    summaryevtxreader.evtxreader_events_accepted,
                    summaryevtxreader.evtxreader_events_processed
                )
            }
        }
    }

    /// Return maximum value for drop number.
    ///
    /// Helpful to format teriminal column widths.
    pub fn max_drop(&self) -> Count {
        match &self.readerdata {
            SummaryReaderData::Dummy => {
                debug_panic!("Should not be called on Dummy");

                0
            },
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    summarylinereader,
                    summarysyslinereader,
                    _summarysyslogprocessor
                )
            ) => {
                max!(
                    summaryblockreader.blockreader_blocks_dropped_ok,
                    summaryblockreader.blockreader_blocks_dropped_err,
                    summarylinereader.linereader_drop_line_ok,
                    summarylinereader.linereader_drop_line_errors,
                    summarysyslinereader.syslinereader_drop_sysline_ok,
                    summarysyslinereader.syslinereader_drop_sysline_errors
                )
            }
            SummaryReaderData::Utmpx(
                (
                    summaryblockreader,
                    _summaryutmpreader,
                )
            ) => {
                max!(
                    summaryblockreader.blockreader_blocks_dropped_ok,
                    summaryblockreader.blockreader_blocks_dropped_err
                )
            }
            SummaryReaderData::Etvx(_summaryevtxreader) => {
                0
            }
        }
    }
}

impl fmt::Debug for Summary {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match &self.readerdata {
            SummaryReaderData::Dummy => f.debug_struct("").finish(),
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    summarylinereader,
                    summarysyslinereader,
                    _summarysyslogprocessor
                )
            ) => {
                match self.filetype {
                    FileType::Tar | FileType::File => f
                        .debug_struct("")
                        .field("bytes", &summaryblockreader.blockreader_bytes)
                        .field("bytes total", &summaryblockreader.blockreader_bytes_total)
                        .field("lines", &summarylinereader.linereader_lines)
                        .field("lines stored highest", &summarylinereader.linereader_lines_stored_highest)
                        .field("syslines", &summarysyslinereader.syslinereader_syslines)
                        .field("syslines stored highest", &summarysyslinereader.syslinereader_syslines_stored_highest)
                        .field("blocks", &summaryblockreader.blockreader_blocks)
                        .field("blocks total", &summaryblockreader.blockreader_blocks_total)
                        .field("blocks stored highest", &summaryblockreader.blockreader_blocks_highest)
                        .field("blocksz", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_blocksz))
                        .field("filesz", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_filesz))
                        .finish(),
                    FileType::Gz | FileType::Xz => f
                        .debug_struct("")
                        .field("bytes", &summaryblockreader.blockreader_bytes)
                        .field("bytes total", &summaryblockreader.blockreader_bytes_total)
                        .field("lines", &summarylinereader.linereader_lines)
                        .field("lines stored highest", &summarylinereader.linereader_lines_stored_highest)
                        .field("syslines", &summarysyslinereader.syslinereader_syslines)
                        .field("syslines stored highest", &summarysyslinereader.syslinereader_syslines_stored_highest)
                        .field("blocks", &summaryblockreader.blockreader_blocks)
                        .field("blocks total", &summaryblockreader.blockreader_blocks_total)
                        .field("blocks stored high", &summaryblockreader.blockreader_blocks_highest)
                        .field("blocksz", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_blocksz))
                        .field("filesz uncompressed", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_filesz_actual))
                        .field("filesz compressed", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_filesz))
                        .finish(),
                    // Summary::default()
                    FileType::Unknown => f.debug_struct("").finish(),
                    FileType::Unparseable => f.debug_struct("").finish(),
                    FileType::Unset | _ =>
                        unimplemented!("FileType {:?} not implemented for Summary fmt::Debug", self.filetype),
                }
            }
            SummaryReaderData::Utmpx(
                (
                    summaryblockreader,
                    summaryutmpreader,
                )
            ) => {
                match self.filetype {
                    FileType::Utmpx => f
                        .debug_struct("")
                        .field("bytes", &summaryblockreader.blockreader_bytes)
                        .field("bytes total", &summaryblockreader.blockreader_bytes_total)
                        .field("utmp entries", &summaryutmpreader.utmpxreader_utmp_entries)
                        .field("blocks", &summaryblockreader.blockreader_blocks)
                        .field("blocks total", &summaryblockreader.blockreader_blocks_total)
                        .field("blocks stored highest", &summaryblockreader.blockreader_blocks_highest)
                        .field("blocksz", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_blocksz))
                        .field("filesz", &format_args!("{0} (0x{0:X})", &summaryblockreader.blockreader_filesz))
                        .finish(),
                    ft => panic!("Unpexected filetype {}", ft),
                }
            }
            SummaryReaderData::Etvx(summaryevtxreader) => {
                match self.filetype {
                    FileType::Evtx => f
                        .debug_struct("")
                        .field("evtx events processed", &summaryevtxreader.evtxreader_events_processed)
                        .field("evtx events accepted", &summaryevtxreader.evtxreader_events_accepted)
                        .finish(),
                    ft => panic!("Unpexected filetype {}", ft),
                }
            }
        }
    }
}

/// Optional `Summary`.
pub type SummaryOpt = Option<Summary>;
