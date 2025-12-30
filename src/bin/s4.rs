// src/bin/s4.rs
//
// ‥ … ≤ ≥ ≠ ≟ ×

//! Driver program _s4_ drives the [_s4lib_].
//!
//! Processes user-passed command-line arguments.
//! Then processes paths passed; directories are enumerated for parseable files,
//! archive files (`.tar`) are enumerated for file entries, other
//! paths tested for suitability (readable? is it a file? etc.).
//!
//! For each parseable file found, a file processing thread is created.
//! Each file processing thread advances through the stages of processing
//! using a [`SyslogProcessor`] instance, a [`FixedStructReader`] instance,
//! a [`EvtxReader`] instance, or a [`JournalReader`] instance.
//!
//! For a `SyslogProcessor`, during the main processing stage,
//! [`Stage3StreamSyslines`], each thread
//! sends the last processed [`Sysline`] to the main processing thread.
//! The main processing thread compares the last [`DateTimeL`] received
//! from all processing threads.
//! The `Sysline` with the earliest `DateTimeL` is printed.
//! That file processing thread then processes another `Sysline`.
//! This continues until each file processing thread sends a message to the
//! main processing thread that is has completed processing,
//! or in case of errors, abruptly closes it's [sending channel].
//!
//! A `FixedStructReader` follow the same threaded message-passing pattern but
//! has much simpler processing stages.
//!
//! A `EvtxReader` follows the same threaded message-passing pattern but
//! uses underlying [`EvtxParser`].
//!
//! A `JournalReader` follows the same threaded message-passing pattern but
//! uses underlying [`JournalApiPtr`].
//!
//! If passed CLI option `--summary`, the main processing thread
//! prints a [`Summary`] about each file processed, and one final
//! [`SummaryPrinted`].
//!
//! `s4.rs` should be the main thread and the only thread that prints to STDOUT.
//!
//! [_s4lib_]: s4lib
//! [`Stage3StreamSyslines`]: s4lib::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
//! [`DateTimeL`]: s4lib::data::datetime::DateTimeL
//! [`Sysline`]: s4lib::data::sysline::Sysline
//! [sending channel]: self::ChanSendDatum
//! [`SyslogProcessor`]: s4lib::readers::syslogprocessor::SyslogProcessor
//! [`FixedStructReader`]: s4lib::readers::fixedstructreader::FixedStructReader
//! [`EvtxReader`]: s4lib::readers::evtxreader::EvtxReader
//! [`JournalReader`]: s4lib::readers::journalreader::JournalReader
//! [`Summary`]: s4lib::readers::summary::Summary
//! [`SummaryPrinted`]: self::SummaryPrinted
//! [`EvtxParser`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxParser.html
//! [`JournalApiPtr`]: s4lib::libload::systemd_dlopen2::JournalApiPtr

#![allow(non_camel_case_types)]

// first setup the custom global allocator
use ::s4lib::common::AllocatorChosen;

cfg_if::cfg_if! {
    if #[cfg(feature = "jemalloc")] {
        use ::tikv_jemallocator::Jemalloc;
        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
        const ALLOCATOR_CHOSEN: AllocatorChosen = AllocatorChosen::Jemalloc;
        const CLI_HELP_AFTER_ALLOCATOR: &str = "jemalloc";
    }
    else if #[cfg(feature = "mimalloc")] {
        use ::mimalloc::MiMalloc;
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;
        const ALLOCATOR_CHOSEN: AllocatorChosen = AllocatorChosen::Mimalloc;
        const CLI_HELP_AFTER_ALLOCATOR: &str = "mimalloc";
    }
    else if #[cfg(feature = "tcmalloc")] {
        use ::tcmalloc::TCMalloc;
        #[global_allocator]
        static GLOBAL: TCMalloc = TCMalloc;
        const ALLOCATOR_CHOSEN: AllocatorChosen = AllocatorChosen::TCMalloc;
        const CLI_HELP_AFTER_ALLOCATOR: &str = "tcmalloc";
    }
    else {
        const ALLOCATOR_CHOSEN: AllocatorChosen = AllocatorChosen::System;
        const CLI_HELP_AFTER_ALLOCATOR: &str = "system";
    }
}

use std::collections::{
    BTreeMap,
    HashMap,
};
use std::fmt;
use std::io::{
    BufRead, // for stdin::lock().lines()
    Error,
};
use std::process::ExitCode;
use std::sync::RwLock;
use std::time::Instant;
use std::{
    str,
    thread,
    thread_local,
};

use ::chrono::{
    DateTime,
    Datelike,
    Duration,
    FixedOffset,
    Local,
    LocalResult,
    TimeZone,
    Timelike,
};
use ::clap::{
    Parser,
    ValueEnum,
};
use ::const_format::concatcp;
use ::lazy_static::lazy_static;
use ::regex::Regex;

use ::s4lib::common::{
    Count,
    FPath,
    FPaths,
    FileOffset,
    FileProcessingResult,
    FileType,
    LogMessageType,
    NLu8a,
    Result3E,
    PathId,
    SetPathId,
    FILE_TOO_SMALL_SZ,
    SUBPATH_SEP,
    SUBPATH_SEP_DISPLAY_STR,
    summary_stats_enable,
};
use ::s4lib::data::common::LogMessage;
use ::s4lib::data::datetime::{
    datetime_parse_from_str,
    datetime_parse_from_str_w_tz,
    systemtime_to_datetime,
    DateTimeLOpt,
    DateTimePattern_str,
    MAP_TZZ_TO_TZz,
    Utc,
};
use ::s4lib::data::pydataevent::EtlParserUsed;
use ::s4lib::data::fixedstruct::ENTRY_SZ_MAX;
use ::s4lib::data::journal::datetimelopt_to_realtime_timestamp_opt;
use ::s4lib::data::sysline::SyslineP;
use ::s4lib::debug::printers::{
    de_err,
    de_wrn,
    e_err,
    e_wrn,
};
// `s4lib` is the local compiled `[lib]` of super_speedy_syslog_searcher
use ::s4lib::debug_panic;
use ::s4lib::libload::systemd_dlopen2::{
    load_library_systemd,
    LoadLibraryError,
    LIB_NAME_SYSTEMD,
};
use ::s4lib::printer::printers::{
    color_rand,
    color_default,
    fpath_to_prependname,
    fpath_to_prependpath,
    write_stdout,
    // termcolor imports
    Color,
    ColorChoice,
    //
    ColorTheme,
    ColorThemeGlobal,
    PrinterLogMessage,
    COLOR_THEME_DEFAULT,
};
use ::s4lib::printer::summary::{
    print_summary,
    summary_update,
    MapPathIdSummary,
    MapPathIdSummaryPrint,
    MapPathIdToColor,
    MapPathIdToFPath,
    MapPathIdToFileProcessingResultBlockZero,
    MapPathIdToFileType,
    MapPathIdToLogMessageType,
    MapPathIdToModifiedTime,
    MapPathIdToPrinterLogMessage,
    MapPathIdToProcessPathResult,
    MapPathIdToProcessPathResultOrdered,
    SummaryPrinted,
};
use ::s4lib::python::pyrunner::{
    PipeSz,
    PYTHON_ENV,
};
use ::s4lib::python::venv::{
    PYTHON_VENV_PATH_DEFAULT,
    create as venv_create,
};
use ::s4lib::readers::blockreader::{
    BlockSz,
    BLOCKSZ_DEF,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
};
use ::s4lib::readers::pyeventreader::{
    PyEventReader,
    PyEventType,
    ResultNextPyDataEvent,
};
use ::s4lib::readers::evtxreader::EvtxReader;
use ::s4lib::readers::filedecompressor::{
    count_temporary_files,
    remove_temporary_files,
};
use ::s4lib::readers::filepreprocessor::{
    process_path,
    ProcessPathResult,
    ProcessPathResults,
};
use ::s4lib::readers::fixedstructreader::{
    FixedStructReader,
    ResultFindFixedStruct,
    ResultFixedStructReaderNew,
};
use ::s4lib::readers::helpers::{
    basename,
    fpath_to_path,
};
use ::s4lib::readers::journalreader::{
    JournalOutput,
    JournalReader,
    ResultNext,
};
use ::s4lib::readers::summary::{
    Summary,
    SummaryOpt,
};
use ::s4lib::readers::syslinereader::ResultFindSysline;
use ::s4lib::readers::syslogprocessor::{
    FileProcessingResultBlockZero,
    SyslogProcessor,
};
use ::si_trace_print::stack::stack_offset_set;
use ::si_trace_print::{
    def1n,
    def1o,
    def1x,
    def1ñ,
    defn,
    defo,
    defx,
    defñ,
    deo,
};

use ::anyhow;
use ::crossbeam_channel;
use ::unicode_width;

// --------------------
// command-line parsing

/// user-passed signifier that file paths were passed on STDIN
const PATHS_ON_STDIN: &str = "-";

/// general error exit value
const EXIT_ERR: i32 = 1;

thread_local! {
    /// for user-passed strings of a duration that will be offset from the
    /// current datetime.
    static UTC_NOW: DateTime<Utc> = {
        defo!("thread_local! UTC_NOW::new()");

        Utc::now()
    };
    static LOCAL_NOW: DateTime<Local> = {
        defo!("thread_local! LOCAL_NOW::new()");

        UTC_NOW.with(|utc_now| {
            DateTime::from(utc_now.with_timezone(&Local))
        })
    };
    static LOCAL_NOW_OFFSET: FixedOffset = {
        defo!("thread_local! LOCAL_NOW_OFFSET::new()");

        LOCAL_NOW.with(|local_now| *local_now.offset())
    };
    static LOCAL_NOW_OFFSET_STR: String = {
        defo!("thread_local! LOCAL_NOW_OFFSET_STR::new()");

        LOCAL_NOW_OFFSET.with(|local_now_offset| local_now_offset.to_string())
    };
    pub static FIXEDOFFSET0: FixedOffset = {
        defo!("thread_local! FIXEDOFFSET0::new()");

        FixedOffset::east_opt(0).unwrap()
    }
}

/// CLI enum that maps to [`termcolor::ColorChoice`].
///
/// [`termcolor::ColorChoice`]: https://docs.rs/termcolor/1.1.3/termcolor/enum.ColorChoice.html
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ValueEnum, // from `clap`
)]
enum CLI_Color_Choice {
    always,
    auto,
    never,
}

/// Subset of [`DateTimeParseInstr`] for calls to
/// function [`datetime_parse_from_str`].
///
/// (DateTimePattern_str, has_year, has_timezone, has_Z, has_time, has_time_sec, has_date)
///
/// [`DateTimeParseInstr`]: s4lib::data::datetime::DateTimeParseInstr
/// [`datetime_parse_from_str`]: s4lib::data::datetime#fn.datetime_parse_from_str
#[allow(non_camel_case_types)]
type CLI_DT_Filter_Pattern<'b> = (&'b DateTimePattern_str, bool, bool, bool, bool, bool, bool);

const CLI_FILTER_PATTERNS_COUNT: usize = 78;

/// CLI acceptable datetime filter patterns for the user-passed `-a` or `-b`
// XXX: this is a an inelegant brute-force approach to matching potential
//      datetime patterns in the user-passed `-a` or `-b` arguments. But it
//      works and in ad-hoc experiments it didn't appear to add any significant
//      run-time.
const CLI_FILTER_PATTERNS: [CLI_DT_Filter_Pattern; CLI_FILTER_PATTERNS_COUNT] = [
    // XXX: use of `%Z` must be at the end of the `DateTimePattern_str` value
    //      as this is an assumption of the `process_dt` function.
    // YYYYmmddTHH:MM:SS*
    ("%Y%m%dT%H%M%S", true, false, false, true, true, true),
    ("%Y%m%dT%H%M%S.%3f", true, false, false, true, true, true),
    ("%Y%m%dT%H%M%S.%6f", true, false, false, true, true, true),
    // %z
    ("%Y%m%dT%H%M%S%z", true, true, false, true, true, true),
    ("%Y%m%dT%H%M%S.%3f%z", true, true, false, true, true, true),
    ("%Y%m%dT%H%M%S.%6f%z", true, true, false, true, true, true),
    // %:z
    ("%Y%m%dT%H%M%S%:z", true, true, false, true, true, true),
    ("%Y%m%dT%H%M%S.%3f%:z", true, true, false, true, true, true),
    ("%Y%m%dT%H%M%S.%6f%:z", true, true, false, true, true, true),
    // %#z
    ("%Y%m%dT%H%M%S%#z", true, true, false, true, true, true),
    ("%Y%m%dT%H%M%S.%3f%#z", true, true, false, true, true, true),
    ("%Y%m%dT%H%M%S.%6f%#z", true, true, false, true, true, true),
    // %Z
    ("%Y%m%dT%H%M%S%Z", true, true, true, true, true, true),
    ("%Y%m%dT%H%M%S.%3f%Z", true, true, true, true, true, true),
    ("%Y%m%dT%H%M%S.%6f%Z", true, true, true, true, true, true),
    // YYYY-mm-dd HH:MM:SS*
    ("%Y-%m-%d %H:%M:%S", true, false, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%3f", true, false, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%6f", true, false, false, true, true, true),
    // %z
    ("%Y-%m-%d %H:%M:%S %z", true, true, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%3f %z", true, true, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%6f %z", true, true, false, true, true, true),
    // %:z
    ("%Y-%m-%d %H:%M:%S %:z", true, true, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%3f %:z", true, true, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%6f %:z", true, true, false, true, true, true),
    // %#z
    ("%Y-%m-%d %H:%M:%S %#z", true, true, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%3f %#z", true, true, false, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%6f %#z", true, true, false, true, true, true),
    // %Z
    ("%Y-%m-%d %H:%M:%S %Z", true, true, true, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%3f %Z", true, true, true, true, true, true),
    ("%Y-%m-%d %H:%M:%S.%6f %Z", true, true, true, true, true, true),
    // YYYY-mm-ddTHH:MM:SS*
    ("%Y-%m-%dT%H:%M:%S", true, false, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f", true, false, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f", true, false, false, true, true, true),
    // %z
    ("%Y-%m-%dT%H:%M:%S%z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S %z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f %z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f %z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f%z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f%z", true, true, false, true, true, true),
    // %:z
    ("%Y-%m-%dT%H:%M:%S%:z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f%:z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f%:z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S %:z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f %:z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f %:z", true, true, false, true, true, true),
    // %#z
    ("%Y-%m-%dT%H:%M:%S%#z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f%#z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f%#z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S %#z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f %#z", true, true, false, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f %#z", true, true, false, true, true, true),
    // %Z
    ("%Y-%m-%dT%H:%M:%S%Z", true, true, true, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f%Z", true, true, true, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f%Z", true, true, true, true, true, true),
    ("%Y-%m-%dT%H:%M:%S %Z", true, true, true, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%3f %Z", true, true, true, true, true, true),
    ("%Y-%m-%dT%H:%M:%S.%6f %Z", true, true, true, true, true, true),
    // YYYY/mm/dd HH:MM:SS*
    ("%Y/%m/%d %H:%M:%S", true, false, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%3f", true, false, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%6f", true, false, false, true, true, true),
    // %z
    ("%Y/%m/%d %H:%M:%S %z", true, true, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%3f %z", true, true, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%6f %z", true, true, false, true, true, true),
    // %:z
    ("%Y/%m/%d %H:%M:%S %:z", true, true, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%3f %:z", true, true, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%6f %:z", true, true, false, true, true, true),
    // %#z
    ("%Y/%m/%d %H:%M:%S %#z", true, true, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%3f %#z", true, true, false, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%6f %#z", true, true, false, true, true, true),
    // %Z
    ("%Y/%m/%d %H:%M:%S %Z", true, true, true, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%3f %Z", true, true, true, true, true, true),
    ("%Y/%m/%d %H:%M:%S.%6f %Z", true, true, true, true, true, true),
    // YYYYmmdd
    ("%Y%m%d", true, false, false, false, false, true),
    // YYYY-mm-dd
    ("%Y-%m-%d", true, false, false, false, false, true),
    // YYYY/mm/dd
    ("%Y/%m/%d", true, false, false, false, false, true),
    // HH:mm:ss
    ("%H:%M:%S", true, false, false, true, true, false),
    // HH:mm
    ("%H:%M", true, false, false, true, false, false),
    // s
    ("+%s", false, false, false, true, false, true),
];

const CGN_DUR_OFFSET_TYPE: &str = "offset_type";
const CGN_DUR_OFFSET_ADDSUB: &str = "offset_addsub";
const CGN_DUR_OFFSET_SECONDS: &str = "seconds";
const CGN_DUR_OFFSET_MINUTES: &str = "minutes";
const CGN_DUR_OFFSET_HOURS: &str = "hours";
const CGN_DUR_OFFSET_DAYS: &str = "days";
const CGN_DUR_OFFSET_WEEKS: &str = "weeks";

const CGP_DUR_OFFSET_TYPE: &str = concatcp!("(?P<", CGN_DUR_OFFSET_TYPE, r">[@]?)");
const CGP_DUR_OFFSET_ADDSUB: &str = concatcp!("(?P<", CGN_DUR_OFFSET_ADDSUB, r">[+\-])");
const CGP_DUR_OFFSET_SECONDS: &str = concatcp!("(?P<", CGN_DUR_OFFSET_SECONDS, r">[\d]+s)");
const CGP_DUR_OFFSET_MINUTES: &str = concatcp!("(?P<", CGN_DUR_OFFSET_MINUTES, r">[\d]+m)");
const CGP_DUR_OFFSET_HOURS: &str = concatcp!("(?P<", CGN_DUR_OFFSET_HOURS, r">[\d]+h)");
const CGP_DUR_OFFSET_DAYS: &str = concatcp!("(?P<", CGN_DUR_OFFSET_DAYS, r">[\d]+d)");
const CGP_DUR_OFFSET_WEEKS: &str = concatcp!("(?P<", CGN_DUR_OFFSET_WEEKS, r">[\d]+w)");

thread_local! {
    /// user-passed strings of a duration that is a relative offset.
    static REGEX_DUR_OFFSET: Regex = {
        defñ!("thread_local! REGEX_DUR_OFFSET::new()");

        Regex::new(
            concatcp!(
                CGP_DUR_OFFSET_TYPE,
                CGP_DUR_OFFSET_ADDSUB, "(",
                CGP_DUR_OFFSET_SECONDS, "|",
                CGP_DUR_OFFSET_MINUTES, "|",
                CGP_DUR_OFFSET_HOURS, "|",
                CGP_DUR_OFFSET_DAYS, "|",
                CGP_DUR_OFFSET_WEEKS,
                ")+"
            )
        ).unwrap()
    };
}

/// Duration offset type; for CLI options `-a` and `-b` relative offset value.
/// Either relative offset from now (program run-time) or relative offset
/// from the other CLI option.
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
enum DUR_OFFSET_TYPE {
    Now,
    Other,
}

/// Duration offset is added or subtracted from a `DateTime`?
#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
enum DUR_OFFSET_ADDSUB {
    Add = 1,
    Sub = -1,
}

/// CLI time to append in `fn process_dt` when `has_time` is `false`.
const CLI_DT_FILTER_APPEND_TIME_VALUE: &str = " T000000";

/// CLI strftime format pattern to append in function `process_dt`
/// when `has_time` is `false`.
const CLI_DT_FILTER_APPEND_TIME_PATTERN: &str = " T%H%M%S";

/// CLI strftime format pattern to append in function `process_dt`
/// when `has_date` is `false`.
const CLI_DT_FILTER_APPEND_DATE_PATTERN: &str = "%Y%m%dT";

/// default separator for prepended strings
const CLI_PREPEND_SEP: &str = ":";

/// default CLI datetime format printed for CLI options `-u` or `-l`.
const CLI_OPT_PREPEND_FMT: &str = "%Y%m%dT%H%M%S%.3f%z";

#[cfg(debug_assertions)]
const CLI_HELP_AFTER_NOTE_DEBUG: &str = "\nDEBUG BUILD";
#[cfg(not(debug_assertions))]
const CLI_HELP_AFTER_NOTE_DEBUG: &str = "";

#[cfg(test)]
const CLI_HELP_AFTER_NOTE_TEST: &str = "\nTEST BUILD";
#[cfg(not(test))]
const CLI_HELP_AFTER_NOTE_TEST: &str = "";

/// `--help` _afterword_ message.
const CLI_HELP_AFTER: &str = concatcp!(
    "\
Given a file path, the file format will be processed based on a best guess of
the file name.
If the file format is not guessed then it will be treated as a UTF8 text file.
Given a directory path, found file names that have well-known non-log file name
extensions will be skipped.

DateTime Filters may be strftime specifier patterns:
    \"",
    CLI_FILTER_PATTERNS[0].0,
    "*\"
    \"",
    CLI_FILTER_PATTERNS[15].0,
    "*\"
    \"",
    CLI_FILTER_PATTERNS[30].0,
    "*\"
    \"",
    CLI_FILTER_PATTERNS[57].0,
    "*\"
    \"",
    CLI_FILTER_PATTERNS[72].0,
    "\"
    \"",
    CLI_FILTER_PATTERNS[73].0,
    "\"
    \"",
    CLI_FILTER_PATTERNS[74].0,
    "\"
    \"",
    CLI_FILTER_PATTERNS[75].0,
    "\"
    \"",
    CLI_FILTER_PATTERNS[76].0,
    "\"
    \"",
    CLI_FILTER_PATTERNS[77].0,
    "\"",
    r#"
Each trailing * is an optional trailing 3-digit fractional sub-seconds,
or 6-digit fractional sub-seconds, and/or timezone.

Pattern "+%s" is Unix epoch timestamp in seconds with a preceding "+".
For example, value "+946684800" is January 1, 2000 at 00:00, GMT.

DateTime Filters may be custom relative offset patterns:
    "+DwDdDhDmDs" or "-DwDdDhDmDs"
    "@+DwDdDhDmDs" or "@-DwDdDhDmDs"

Custom relative offset pattern "+DwDdDhDmDs" and "-DwDdDhDmDs" is the offset
from now (program start time) where "D" is a decimal number.
Each lowercase identifier is an offset duration:
"w" is weeks, "d" is days, "h" is hours, "m" is minutes, "s" is seconds.
For example, value "-1w22h" is one week and twenty-two hours in the past.
Value "+30s" is thirty seconds in the future.

Custom relative offset pattern "@+DwDdDhDmDs" and "@-DwDdDhDmDs" is relative
offset from the other datetime.
Arguments "-a 20220102 -b @+1d" are equivalent to "-a 20220102 -b 20220103".
Arguments "-a @-6h -b 20220101T120000" are equivalent to
"-a 20220101T060000 -b 20220101T120000".

DateTime Filters with a clock time but without a date are assumed to be today,
e.g. "-a 12:05" is today at time 12:05 in the local timezone.
DateTime Filters with a date but without a clock time are assumed to be that
date at time 00:00:00.000 in the local timezone, e.g. "-a 20220102" is
2022-01-02 at time 00:00:00.000 in the local timezone.

Without a timezone, the Datetime Filter is presumed to be the local
system timezone.

Command-line passed timezones may be numeric timezone offsets,
e.g. "+09:00", "+0900", or "+09", or named timezone offsets, e.g. "JST".
Ambiguous named timezones will be rejected, e.g. "SST".

--prepend-tz and --dt-offset function independently:
--dt-offset is used to interpret processed log message datetime stamps that
do not have a timezone offset.
--prepend-tz affects what is pre-printed before each printed log message line.

--separator accepts backslash escape sequences:
    ""#, unescape::BACKSLASH_ESCAPE_SEQUENCES0, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES1, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES2, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES3, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES4, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES5, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES6, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES7, "\", \
 \"", unescape::BACKSLASH_ESCAPE_SEQUENCES8, "\", \
  \"", unescape::BACKSLASH_ESCAPE_SEQUENCES9, r#""

Resolved values of "--dt-after" and "--dt-before" can be reviewed in
the "--summary" output.

s4 uses file naming to determine the file type.

s4 can process files compressed and named .bz2, .gz, .lz4, .xz, and files
archived within a .tar file.

Log messages from different files with the same datetime are printed in order
of the arguments from the command-line.

Datetimes printed for .journal file log messages may differ from datetimes
printed by program journalctl.
See Issue #101

DateTime strftime specifiers are described at
https://docs.rs/chrono/latest/chrono/format/strftime/

DateTimes supported are only of the Gregorian calendar.

DateTimes supported language is English.

The Python interpreter used during `--venv` requires Python 3.7 or higher.
This installs to "#, PYTHON_VENV_PATH_DEFAULT, r#"
These Python packages are installed:
  dissect-etl==3.14
  etl-parser==1.0.1
The Python interpreter used may be overridden by setting environment variable
"#, PYTHON_ENV, r#" to the path of the Python interpreter.

Is s4 failing to parse a log file? Report an Issue at
https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/new/choose

---

Version: "#, env!("CARGO_PKG_VERSION"), r#"
MSRV: "#, env!("CARGO_PKG_RUST_VERSION"), r#"
Allocator: "#, CLI_HELP_AFTER_ALLOCATOR, r#"
License: "#, env!("CARGO_PKG_LICENSE"), r#"
Repository: "#, env!("CARGO_PKG_REPOSITORY"), r#"
Author: "#, env!("CARGO_PKG_AUTHORS"), r#"
"#,
    CLI_HELP_AFTER_NOTE_DEBUG,
    CLI_HELP_AFTER_NOTE_TEST
);

static mut PREPEND_DT_FORMAT_PASSED: bool = false;

/// clap command-line arguments build-time definitions.
//
// Useful clap references:
// * inference types <https://github.com/clap-rs/clap/blob/v3.1.6/examples/derive_ref/README.md#arg-types>
// * other `clap::App` options <https://docs.rs/clap/latest/clap/struct.App.html>
//
// Note:
// * the `about` is taken from `Cargo.toml:[package]:description`.
#[derive(Parser, Debug)]
#[clap(
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS"),
    name = "s4",
    // write expanded information for the `--version` output
    version = concatcp!(
        "(Super Speedy Syslog Searcher)\n",
        "Version: ",
        env!("CARGO_PKG_VERSION_MAJOR"), ".",
        env!("CARGO_PKG_VERSION_MINOR"), ".",
        env!("CARGO_PKG_VERSION_PATCH"), "\n",
        "MSRV: ", env!("CARGO_PKG_RUST_VERSION"), "\n",
        "Allocator: ", CLI_HELP_AFTER_ALLOCATOR , "\n",
        "License: ", env!("CARGO_PKG_LICENSE"), "\n",
        "Repository: ", env!("CARGO_PKG_REPOSITORY"), "\n",
        "Author: ", env!("CARGO_PKG_AUTHORS"), "\n",
    ),
    after_help = CLI_HELP_AFTER,
    verbatim_doc_comment,
    // override usage to more clearly show `--venv` is an exclusive "mode".
    // clap does not support multiple usage statements with exclusive args
    // see https://github.com/clap-rs/clap/issues/4191
    override_usage = "\n  s4 [OPTIONS] <PATHS>...\n\n  s4 --venv",
)]
struct CLI_Args {
    /// Path(s) of log files or directories.
    /// Directories will be recursed. Symlinks will be followed.
    /// Paths may also be passed via STDIN, one per line. The user must
    /// supply argument "-" to signify PATHS are available from STDIN.
    #[clap(
        required = true,
        verbatim_doc_comment,
        groups = &[
            "command_mode",
        ],
    )]
    paths: Vec<String>,

    /// DateTime Filter After: print log messages with a datetime that is at
    /// or after this datetime. For example, "20200102T120000" or "-5d".
    #[clap(
        short = 'a',
        long,
        verbatim_doc_comment,
        // TODO: env="S4_DT_AFTER"
    )]
    dt_after: Option<String>,

    /// DateTime Filter Before: print log messages with a datetime that is at
    /// or before this datetime.
    /// For example, "2020-01-03T23:00:00.321-05:30" or "@+1d+11h"
    #[clap(
        short = 'b',
        long,
        verbatim_doc_comment,
        // TODO: env="S4_DT_BEFORE"
    )]
    dt_before: Option<String>,

    /// Default timezone offset for datetimes without a timezone.
    /// For example, log message "[20200102T120000] Starting service" has a
    /// datetime substring "20200102T120000".
    /// That datetime substring does not have a timezone offset
    /// so this TZ_OFFSET value would be used.
    /// Example values, "+12", "-0800", "+02:00", or "EDT".
    /// To pass a value with leading "-" use "=" notation, e.g. "-t=-0800".
    /// If not passed then the local system timezone offset is used.
    #[clap(
        short = 't',
        long,
        verbatim_doc_comment,
        value_parser = cli_process_tz_offset,
        default_value_t=LOCAL_NOW_OFFSET.with(|lno| *lno),
        // TODO: env="S4_TZ_OFFSET"
    )]
    tz_offset: FixedOffset,

    /// Prepend a DateTime in the timezone PREPEND_TZ for every line.
    /// Used in PREPEND_DT_FORMAT.
    #[clap(
        short = 'z',
        long = "prepend-tz",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt",
        ],
        value_parser = cli_process_tz_offset,
        // TODO: env="S4_PREPEND_TZ"
    )]
    prepend_tz: Option<FixedOffset>,

    /// Prepend a DateTime in the UTC timezone offset for every line.
    /// This is the same as "--prepend-tz Z".
    /// Used in PREPEND_DT_FORMAT.
    #[clap(
        short = 'u',
        long = "prepend-utc",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt",
        ],
        // TODO: env="S4_PREPEND_UTC"
    )]
    prepend_utc: bool,

    /// Prepend DateTime in the local system timezone offset for every line.
    /// This is the same as "--prepend-tz +XX" where +XX is the local system
    /// timezone offset.
    /// Used in PREPEND_DT_FORMAT.
    #[clap(
        short = 'l',
        long = "prepend-local",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt",
        ],
        // TODO: env="S4_PREPEND_LOCAL"
    )]
    prepend_local: bool,

    // Prepend a DateTime using strftime format string.
    #[clap(
        short = 'd',
        long = "prepend-dt-format",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt_format",
        ],
        hide_default_value = true,
        help = concatcp!(r#"Prepend a DateTime using the strftime format string.
If PREPEND_TZ is set then that value is used for any timezone offsets,
i.e. strftime "%z" "%:z" "%Z" values, otherwise the timezone offset value
is the local system timezone offset. [Default: "#, CLI_OPT_PREPEND_FMT, "]"),
        value_parser = cli_parser_prepend_dt_format,
        default_value = None,
        // TODO: env="S4_PREPEND_DT_FORMAT"
    )]
    prepend_dt_format: Option<String>,

    /// Prepend file basename to every line.
    #[clap(
        short = 'n',
        long = "prepend-filename",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_file",
        ]
        // TODO: env="S4_PREPEND_FILENAME"
    )]
    prepend_filename: bool,

    /// Prepend file full path to every line.
    #[clap(
        short = 'p',
        long = "prepend-filepath",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_file",
        ]
        // TODO: env="S4_PREPEND_FILEPATH"
    )]
    prepend_filepath: bool,

    /// Align column widths of prepended data.
    #[clap(
        short = 'w',
        long = "prepend-file-align",
        verbatim_doc_comment,
        requires = "group_prepend_file",
        // TODO: env="S4_PREPEND_FILE_ALIGN"
    )]
    prepend_file_align: bool,

    /// Separator string for prepended data.
    #[clap(
        long = "prepend-separator",
        verbatim_doc_comment,
        // TODO: how to require `any("prepend_file", "prepend_dt")`
        default_value_t = String::from(CLI_PREPEND_SEP)
        // TODO: env="S4_PREPEND_SEPARATOR"
    )]
    prepend_separator: String,

    // TODO: [2023/02/26] add option to prepend byte offset along with filename, helpful for development

    /// An extra separator string between printed log messages.
    /// Per log message not per line of text.
    /// Accepts a basic set of backslash escape sequences,
    /// e.g. "\0" for the null character, "\t" for tab, etc.
    #[clap(
        long = "separator",
        required = false,
        verbatim_doc_comment,
        default_value_t = String::from(""),
        hide_default_value = true,
        // TODO: env="S4_SEPARATOR"
    )]
    log_message_separator: String,

    /// The format for .journal file log messages.
    /// Matches journalctl --output options.
    /// :
    #[clap(
        long = "journal-output",
        required = false,
        verbatim_doc_comment,
        value_enum,
        default_value_t = JournalOutput::Short,
        // TODO: env="S4_JOURNAL_OUTPUT"
    )]
    journal_output: JournalOutput,

    /// For parsing Windows Event Tracing Log (.etl) files, use Python library
    /// etl-parser. By default, Python library dissect.etl is used.
    /// The etl-parser library may have more complete information but is slower
    /// than dissect.etl.
    /// Requires prior creation of a Python virtual environment with
    /// the --venv option. Or use environment variable S4_PYTHON set to
    /// a Python interpreter path with necessary packages installed.
    #[clap(
        long = "etl-parser",
        verbatim_doc_comment,
        default_value_t = false,
        // TODO: env = "S4_ETL_PARSER",
    )]
    etl_parser: bool,

    /// Choose to print using colors.
    #[clap(
        required = false,
        short = 'c',
        long = "color",
        verbatim_doc_comment,
        value_enum,
        default_value_t = CLI_Color_Choice::auto,
        // TODO: env="S4_COLOR"
    )]
    color_choice: CLI_Color_Choice,

    /// Print text using darker colors for a lighter terminal background.
    /// By default, a dark color theme is used (print text with lighter colors).
    /// Has no effect if --color is not "always" or "auto".
    #[clap(
        required = false,
        long = "light-theme",
        verbatim_doc_comment,
        default_value_t = false,
        // TODO: env="S4_COLOR_THEME_LIGHT"
    )]
    color_theme_light: bool,

    /// Create a Python virtual environment exclusively for s4.
    /// This is only necessary for parsing Windows Event Tracing Log
    /// (.etl) files. This only needs to be done once.
    /// When this option is used, no other options may be passed.
    /// The Python interpreter used may be set by environment variable
    /// S4_PYTHON.
    // XXX: S4_PYTHON must match PYTHON_ENV
    #[clap(
        long = "venv",
        verbatim_doc_comment,
        default_value_t = false,
        groups = &[
             "command_mode",
        ],
        conflicts_with = "paths",
        exclusive = true,
    )]
    python_venv: bool,

    /// Read blocks of this size in bytes.
    /// May pass value as any radix (hexadecimal, decimal, octal, binary).
    /// Using the default value is recommended.
    /// Most useful for developers.
    #[clap(
        required = false,
        long,
        verbatim_doc_comment,
        default_value_t = BLOCKSZ_DEF.to_string(),
        value_parser = cli_parse_blocksz,
        // TODO: env="S4_BLOCKSZ"
    )]
    blocksz: String,

    /// Print a summary of files processed to stderr.
    /// Most useful for developers.
    #[clap(
        short,
        long,
        verbatim_doc_comment,
        // TODO: env="S4_SUMMARY"
    )]
    summary: bool,
}

/// wrapper for checking `EARLY_EXIT`, returns if necessary
macro_rules! exit_early_check {
    () => {
        match EXIT_EARLY.read() {
            Ok(exit_early) => {
                if *exit_early {
                    defx!("EXIT_EARLY.read() true; return false");
                    return false;
                }
            }
            Err(err) => {
                e_err!("EXIT_EARLY.read() failed: {:?}", err);
                defx!("return false");
                return false;
            }
        }
    };
}

/// `clap` argument processor for `--blocksz`.
/// This implementation, as opposed to clap built-in number parsing, allows more
/// flexibility for how the user may pass a number
/// e.g. "0xF00", or "0b10100", etc.
// XXX: clap Enhancement Issue https://github.com/clap-rs/clap/issues/4564
fn cli_process_blocksz(blockszs: &String) -> std::result::Result<u64, String> {
    // TODO: there must be a more concise way to parse numbers with radix formatting
    let blocksz_: BlockSz;
    let errs = format!("Unable to parse a number for --blocksz {:?}", blockszs);

    if blockszs.starts_with("0x") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0x"), 16) {
            Ok(val) => val,
            Err(err) => return Err(format!("{} {}", errs, err)),
        };
    } else if blockszs.starts_with("0o") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0o"), 8) {
            Ok(val) => val,
            Err(err) => return Err(format!("{} {}", errs, err)),
        };
    } else if blockszs.starts_with("0b") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0b"), 2) {
            Ok(val) => val,
            Err(err) => return Err(format!("{} {}", errs, err)),
        };
    } else {
        blocksz_ = match blockszs.parse::<BlockSz>() {
            Ok(val) => val,
            Err(err) => return Err(format!("{} {}", errs, err)),
        };
    }

    let max_min = std::cmp::max(BLOCKSZ_MIN, SyslogProcessor::BLOCKSZ_MIN);
    if !(max_min <= blocksz_ && blocksz_ <= BLOCKSZ_MAX) {
        return Err(format!("--blocksz must be {} ≤ BLOCKSZ ≤ {}, it was {:?}", max_min, BLOCKSZ_MAX, blockszs));
    }

    Ok(blocksz_)
}

/// `clap` argument parser for `--blocksz`.
fn cli_parse_blocksz(blockszs: &str) -> std::result::Result<String, String> {
    match cli_process_blocksz(&String::from(blockszs)) {
        Ok(val) => Ok(val.to_string()),
        Err(err) => Err(err),
    }
}

/// CLI argument processing
fn cli_process_tz_offset(tzo: &str) -> std::result::Result<FixedOffset, String> {
    let tzo_ = match MAP_TZZ_TO_TZz.get(tzo) {
        Some(tz_offset) => {
            match tz_offset.is_empty() {
                // an empty value signifies an ambiguous named timezone
                true => {
                    return Err(format!(
                        "Given ambiguous timezone {:?} (this timezone abbreviation refers to several timezone offsets)",
                        tzo
                    ));
                }
                // unambiguous named timezone passed
                false => tz_offset,
            }
        }
        // no entry found, `tzo` is probably a numeric timezone offset,
        // e.g. `+01:00`
        None => tzo,
    };
    // transform the timezone string to a `FixedOffset` instance
    // using a dummy `DateTimeL`
    let mut data: String = String::from("2000-01-02 03:04:05 ");
    data.push_str(tzo_);
    for pattern in [
        "%Y-%m-%d %H:%M:%S %:z",
        "%Y-%m-%d %H:%M:%S %z",
        "%Y-%m-%d %H:%M:%S %#z",
    ] {
        let dt = datetime_parse_from_str_w_tz(data.as_str(), pattern);
        defo!("datetime_parse_from_str_w_tz({:?}, {:?}) returned {:?}", data, pattern, dt);
        if let Some(dt_) = dt {
            defx!("return {:?}", dt_.offset());
            return Ok(*dt_.offset());
        }
    }

    Err(format!("Unable to parse a timezone offset for --tz-offset {:?}", tzo))
}

/// `clap` argument validator for `--prepend-dt-format`.
///
/// Returning `Ok(None)` means that the user did not pass a value for
/// `--prepend-dt-format`.
fn cli_parser_prepend_dt_format(prepend_dt_format: &str) -> std::result::Result<String, String> {
    defñ!("cli_parser_prepend_dt_format({:?})", prepend_dt_format);
    unsafe {
        PREPEND_DT_FORMAT_PASSED = true;
    }
    if prepend_dt_format.is_empty() {
        return Ok(String::default());
    }
    let result = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0);
    let dt = match result {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(dt, _) => dt,
        LocalResult::None => {
            return Err(format!(
                "Unable to parse a datetime format for --prepend-dt-format {:?} (this datetime is invalid)",
                prepend_dt_format
            ))
        }
    };
    // try to format the datetime with the given format string in case something
    // panics then panic sooner rather than later
    _ = dt.format(prepend_dt_format);

    Ok(String::from(prepend_dt_format))
}

// maps named capture group matches of `CGP_DUR_OFFSET_TYPE` to
// `DUR_OFFSET_TYPE`
// helper to `string_wdhms_to_duration`
fn offset_match_to_offset_duration_type(offset_str: &str) -> DUR_OFFSET_TYPE {
    match offset_str.chars().next() {
        Some('@') => DUR_OFFSET_TYPE::Other,
        _ => DUR_OFFSET_TYPE::Now,
    }
}

// maps named capture group matches of `CGP_DUR_OFFSET_ADDSUB` to
// `DUR_OFFSET_ADDSUB`
// helper to `string_wdhms_to_duration`
fn offset_match_to_offset_addsub(offset_str: &str) -> DUR_OFFSET_ADDSUB {
    match offset_str.chars().next() {
        Some('+') => DUR_OFFSET_ADDSUB::Add,
        Some('-') => DUR_OFFSET_ADDSUB::Sub,
        _ => {
            panic!("Bad match offset_str {:?}, cannot determine DUR_OFFSET_ADDSUB", offset_str);
        }
    }
}

/// regular expression processing of a user-passed duration string like
/// `"-4m2s"` becomes duration of 4 minutes + 2 seconds
/// helper function to `process_dt`
fn string_wdhms_to_duration(val: &String) -> Option<(Duration, DUR_OFFSET_TYPE)> {
    defn!("({:?})", val);

    if val.is_empty() {
        // take the early exit to avoid building regex `REGEX_DUR_OFFSET` (expensive operation)
        defx!("is_empty; return None");
        return None;
    }

    let mut duration_offset_type: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Now;
    let mut duration_addsub: DUR_OFFSET_ADDSUB = DUR_OFFSET_ADDSUB::Add;
    let mut seconds: i64 = 0;
    let mut minutes: i64 = 0;
    let mut hours: i64 = 0;
    let mut days: i64 = 0;
    let mut weeks: i64 = 0;

    let captures: regex::Captures = match REGEX_DUR_OFFSET.with(|re| re.captures(val.as_str())) {
        Some(caps) => caps,
        None => {
            defx!("REGEX_DUR_OFFSET.captures(…) None");
            return None;
        }
    };

    if let Some(match_) = captures.name(CGN_DUR_OFFSET_TYPE) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_TYPE, match_.as_str());
        duration_offset_type = offset_match_to_offset_duration_type(match_.as_str());
    }

    if let Some(match_) = captures.name(CGN_DUR_OFFSET_ADDSUB) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_ADDSUB, match_.as_str());
        duration_addsub = offset_match_to_offset_addsub(match_.as_str());
    }

    let addsub: i64 = duration_addsub as i64;

    if let Some(match_) = captures.name(CGN_DUR_OFFSET_SECONDS) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_SECONDS, match_.as_str());
        let s_count = match_
            .as_str()
            .replace('s', "");
        match i64::from_str_radix(s_count.as_str(), 10) {
            Ok(val) => {
                seconds = val * addsub;
            }
            Err(err) => {
                e_err!("Unable to parse seconds from {:?} {}", match_.as_str(), err);
                std::process::exit(EXIT_ERR);
            }
        }
    }
    if let Some(match_) = captures.name(CGN_DUR_OFFSET_MINUTES) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_MINUTES, match_.as_str());
        let s_count = match_
            .as_str()
            .replace('m', "");
        match i64::from_str_radix(s_count.as_str(), 10) {
            Ok(val) => {
                minutes = val * addsub;
            }
            Err(err) => {
                e_err!("Unable to parse minutes from {:?} {}", match_.as_str(), err);
                std::process::exit(EXIT_ERR);
            }
        }
    }
    if let Some(match_) = captures.name(CGN_DUR_OFFSET_HOURS) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_HOURS, match_.as_str());
        let s_count = match_
            .as_str()
            .replace('h', "");
        match i64::from_str_radix(s_count.as_str(), 10) {
            Ok(val) => {
                hours = val * addsub;
            }
            Err(err) => {
                e_err!("Unable to parse hours from {:?} {}", match_.as_str(), err);
                std::process::exit(EXIT_ERR);
            }
        }
    }
    if let Some(match_) = captures.name(CGN_DUR_OFFSET_DAYS) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_DAYS, match_.as_str());
        let s_count = match_
            .as_str()
            .replace('d', "");
        match i64::from_str_radix(s_count.as_str(), 10) {
            Ok(val) => {
                days = val * addsub;
            }
            Err(err) => {
                e_err!("Unable to parse days from {:?} {}", match_.as_str(), err);
                std::process::exit(EXIT_ERR);
            }
        }
    }
    if let Some(match_) = captures.name(CGN_DUR_OFFSET_WEEKS) {
        defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_WEEKS, match_.as_str());
        let s_count = match_
            .as_str()
            .replace('w', "");
        match i64::from_str_radix(s_count.as_str(), 10) {
            Ok(val) => {
                weeks = val * addsub;
            }
            Err(err) => {
                e_err!("Unable to parse weeks from {:?} {}", match_.as_str(), err);
                std::process::exit(EXIT_ERR);
            }
        }
    }

    let duration = match (
        Duration::try_seconds(seconds),
        Duration::try_minutes(minutes),
        Duration::try_hours(hours),
        Duration::try_days(days),
        Duration::try_weeks(weeks),
    ) {
        (Some(s), Some(m), Some(h), Some(d), Some(w)) => s + m + h + d + w,
        _ => {
            e_err!("Unable to parse a duration from {:?}", val);
            return None;
        }
    };
    defx!("return {:?}, {:?}", duration, duration_offset_type);

    Some((duration, duration_offset_type))
}

/// Process duration string like `"-4m2s"` as relative offset of now,
/// or relative offset of other user-passed datetime argument (`dt_other`).
/// `val="-1d"` is one day ago.
/// `val="+1m"` is one day added to the `dt_other`.
/// helper function to function `process_dt`.
fn string_to_rel_offset_datetime(
    val: &String,
    tz_offset: &FixedOffset,
    dt_other_opt: &DateTimeLOpt,
    now_utc: &DateTime<Utc>,
) -> DateTimeLOpt {
    defn!("({:?}, {:?}, {:?}, {:?})", val, tz_offset, dt_other_opt, now_utc);
    let (duration, duration_offset_type) = match string_wdhms_to_duration(val) {
        Some((dur, dur_type)) => (dur, dur_type),
        None => {
            defx!();
            return None;
        }
    };
    match duration_offset_type {
        DUR_OFFSET_TYPE::Now => {
            // drop fractional seconds
            let now_utc_ = Utc
                .with_ymd_and_hms(
                    now_utc.year(),
                    now_utc.month(),
                    now_utc.day(),
                    now_utc.hour(),
                    now_utc.minute(),
                    now_utc.second(),
                )
                .unwrap();
            // convert `Utc` to `DateTimeL`
            let now = tz_offset.from_utc_datetime(&now_utc_.naive_utc());
            defo!("now     {:?}", now);
            let now_off = now.checked_add_signed(duration);
            defx!("now_off {:?}", now_off.unwrap());

            now_off
        }
        DUR_OFFSET_TYPE::Other => match dt_other_opt {
            Some(dt_other) => {
                defo!("other     {:?}", dt_other);
                let other_off = dt_other.checked_add_signed(duration);
                defx!("other_off {:?}", other_off.unwrap());

                other_off
            }
            None => {
                e_err!("passed relative offset to other datetime {:?}, but other datetime was not set", val);
                std::process::exit(EXIT_ERR);
            }
        },
    }
}

/// Transform a user-passed datetime `String` into a [`DateTimeL`].
///
/// Helper function to function `cli_process_args`.
///
/// [`DateTimeL`]: s4lib::data::datetime::DateTimeL
fn process_dt(
    dts_opt: &Option<String>,
    tz_offset: &FixedOffset,
    dt_other: &DateTimeLOpt,
    now_utc: &DateTime<Utc>,
) -> DateTimeLOpt {
    defn!("({:?}, {:?}, {:?}, {:?})", dts_opt, tz_offset, dt_other, now_utc);
    // parse datetime filters
    let dts = match dts_opt {
        Some(dts) => dts,
        None => {
            return None;
        }
    };
    let dto: DateTimeLOpt;
    // try to match user-passed string to chrono strftime format strings
    #[allow(non_snake_case)]
    for (
        pattern_,
        _has_year,
        has_tz,
        has_tzZ,
        has_time,
        _has_time_sec,
        has_date,
    ) in CLI_FILTER_PATTERNS.iter() {
        defo!(
            "(pattern {:?}, has_year {:?}, has_tz {:?}, has_tzZ {:?}, has_time {:?}, has_time_sec {:?}, has_date {:?})",
            pattern_,
            _has_year,
            has_tz,
            has_tzZ,
            has_time,
            _has_time_sec,
            has_date
        );
        let mut pattern: String = String::from(*pattern_);
        let mut dts_: String = dts.clone();
        // if !has_tzZ then modify trailing string timezone (e.g. "PDT") to
        // numeric timezone (e.g. "-0700")
        // modify pattern `%Z` to `%z`
        // XXX: presumes %Z and %Z value is at end of `pattern_`
        if *has_tzZ {
            defo!("has_tzZ {:?}", dts_);
            let mut val_Z: String = String::with_capacity(5);
            while dts_
                .chars()
                .next_back()
                .unwrap_or('\0')
                .is_alphabetic()
            {
                match dts_.pop() {
                    Some(c) => val_Z.insert(0, c),
                    None => continue,
                }
            }
            if MAP_TZZ_TO_TZz.contains_key(val_Z.as_str()) {
                dts_.push_str(
                    MAP_TZZ_TO_TZz
                        .get(val_Z.as_str())
                        .unwrap(),
                );
            } else {
                defo!("has_tzZ WARNING failed to find MAP_TZZ_TO_TZz({:?})", val_Z);
                continue;
            }
            defo!("has_tzZ replaced Z value {:?}", dts_);

            pattern = pattern_.replacen("%Z", "%z", 1);
            defo!(r#"has_tzZ replaced "%Z" with "%z" {:?}"#, pattern);
        }
        // if !has_time then modify the value and pattern
        // e.g. `"20220101"` becomes `"20220101 T000000"`
        //      `"%Y%d%m"` becomes `"%Y%d%m T%H%M%S"`
        if !has_time {
            dts_.push_str(CLI_DT_FILTER_APPEND_TIME_VALUE);
            pattern.push_str(CLI_DT_FILTER_APPEND_TIME_PATTERN);
            defo!("appended {:?}, {:?}", CLI_DT_FILTER_APPEND_TIME_VALUE, CLI_DT_FILTER_APPEND_TIME_PATTERN);
        }
        if !has_date {
            let mut ymd = String::with_capacity(11);
            LOCAL_NOW.with(|ln| {
                let y = ln.year();
                let m = ln.month();
                let d = ln.day();
                ymd = format!("{:04}{:02}{:02}T", y, m, d);
                dts_.insert_str(0, &ymd);
            });
            pattern.insert_str(0, CLI_DT_FILTER_APPEND_DATE_PATTERN);
            defo!("prepended {:?}, {:?}", ymd, CLI_DT_FILTER_APPEND_DATE_PATTERN);
        }
        defo!("datetime_parse_from_str({:?}, {:?}, {:?}, {:?})", dts_, pattern, has_tz, tz_offset);
        if let Some(val) = datetime_parse_from_str(dts_.as_str(), pattern.as_str(), *has_tz, tz_offset) {
            dto = Some(val);
            defx!("return {:?}", dto);
            return dto;
        };
    } // end for … in CLI_FILTER_PATTERNS
    // could not match specific datetime pattern
    // try relative offset pattern matching, e.g. `"-30m5s"`, `"+2d"`
    dto = string_to_rel_offset_datetime(dts, tz_offset, dt_other, now_utc);
    defx!("return {:?}", dto);

    dto
}

/// Transform a user-passed datetime `String` into a [`DateTimeL`].
///
/// Wrapper to `process_dt`. Exits if `process_dt` returns `None`.
///
/// [`DateTimeL`]: s4lib::data::datetime::DateTimeL
fn process_dt_exit(
    dts_opt: &Option<String>,
    tz_offset: &FixedOffset,
    dt_other: &DateTimeLOpt,
    now_utc: &DateTime<Utc>,
) -> DateTimeLOpt {
    if dts_opt.is_none() {
        return None;
    }

    match process_dt(dts_opt, tz_offset, dt_other, now_utc) {
        Some(dto) => Some(dto),
        None => {
            // user-passed string was not parseable
            e_err!("Unable to parse a datetime from {:?}", dts_opt.as_ref().unwrap_or(&String::from("")));
            std::process::exit(EXIT_ERR);
        }
    }
}

mod unescape {
    // this mod ripped from https://stackoverflow.com/a/58555097/471376

    #[derive(Debug, PartialEq)]
    pub(super) enum EscapeError {
        EscapeAtEndOfString,
        InvalidEscapedChar(char),
    }

    struct InterpretEscapedString<'a> {
        s: std::str::Chars<'a>,
    }

    impl Iterator for InterpretEscapedString<'_> {
        type Item = Result<char, EscapeError>;

        fn next(&mut self) -> Option<Self::Item> {
            self.s.next().map(
                |c| match c {
                    '\\' => match self.s.next() {
                        None => Err(EscapeError::EscapeAtEndOfString),
                        Some('0') => Ok('\0'), // null
                        Some('a') => Ok('\u{07}'), // alert
                        Some('b') => Ok('\u{08}'), // backspace
                        Some('e') => Ok('\u{1B}'), // escape
                        Some('f') => Ok('\u{0C}'), // form feed
                        Some('n') => Ok('\n'), // newline
                        Some('r') => Ok('\r'), // carriage return
                        Some('\\') => Ok('\\'), // backslash
                        Some('t') => Ok('\t'), // horizontal tab
                        Some('v') => Ok('\u{0B}'), // vertical tab
                        Some(c) => Err(EscapeError::InvalidEscapedChar(c)),
                    },
                    c => Ok(c),
                }
            )
        }
    }

    // XXX: these must agree with match statement in prior
    //      `Iterator for InterpretEscapedString`
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES0: &str = r"\0";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES1: &str = r"\a";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES2: &str = r"\b";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES3: &str = r"\e";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES4: &str = r"\f";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES5: &str = r"\n";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES6: &str = r"\r";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES7: &str = r"\\";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES8: &str = r"\t";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES9: &str = r"\v";

    pub(super) fn unescape_str(s: &str) -> Result<String, EscapeError> {
        (InterpretEscapedString { s: s.chars() }).collect()
    }
}

/// Process user-passed CLI argument strings into expected types.
///
/// This function will [`std::process::exit`] if there is an [`Err`].
fn cli_process_args() -> (
    FPaths,
    BlockSz,
    DateTimeLOpt,
    DateTimeLOpt,
    FixedOffset,
    ColorChoice,
    FixedOffset,
    Option<String>,
    bool,
    bool,
    bool,
    String,
    String,
    EtlParserUsed,
    bool,
    JournalOutput,
    bool,
) {
    let args = CLI_Args::parse();

    defo!("args {:?}", args);

    //
    // process string arguments into specific types
    //

    let blockszs: String = args.blocksz;
    let blocksz: BlockSz = match cli_process_blocksz(&blockszs) {
        Ok(val) => val,
        Err(err) => {
            e_err!("{}", err);
            std::process::exit(EXIT_ERR);
        }
    };
    defo!("blocksz {:?}", blocksz);

    let mut paths: Vec<FPath> = Vec::<FPath>::with_capacity(args.paths.len() + 1);
    let mut stdin_check = false;
    for path in args.paths.iter() {
        match path.as_str() {
            PATHS_ON_STDIN => {
                if stdin_check {
                    e_wrn!("passed special PATHS argument {:?} more than once", PATHS_ON_STDIN);
                    continue;
                }
                stdin_check = true;
                // stdin input is file paths, one per line
                for result in std::io::stdin()
                    .lock()
                    .lines()
                {
                    match result {
                        Ok(line) => {
                            paths.push(line);
                        }
                        Err(err) => {
                            // don't continue if there was an error
                            eprintln!("ERROR reading stdin; {}", err);
                            break;
                        }
                    }
                }
            }
            _ => paths.push(path.clone()),
        }
    }

    let tz_offset: FixedOffset = args.tz_offset;
    defo!("tz_offset {:?}", tz_offset);

    let filter_dt_after: DateTimeLOpt;
    let filter_dt_before: DateTimeLOpt;
    let empty_str: String = String::from("");
    let args_dt_after_s: &String = args
        .dt_after
        .as_ref()
        .unwrap_or(&empty_str);
    let args_dt_before_s: &String = args
        .dt_before
        .as_ref()
        .unwrap_or(&empty_str);

    // peek at `-a` and `-b` values:
    // if both are relative to the other then print error message and exit
    // if `-a` is relative to `-b` then process `-b` first
    // else process `-a` then `-b`
    let utc_now = UTC_NOW.with(|utc_now| *utc_now);
    match (string_wdhms_to_duration(args_dt_after_s), string_wdhms_to_duration(args_dt_before_s)) {
        (Some((_, DUR_OFFSET_TYPE::Other)), Some((_, DUR_OFFSET_TYPE::Other))) => {
            e_err!("cannot pass both --dt-after and --dt-before as relative to the other");
            std::process::exit(EXIT_ERR);
        }
        (Some((_, DUR_OFFSET_TYPE::Other)), _) => {
            // special-case: process `-b` value then process `-a` value
            // e.g. `-a "@+1d" -b "20010203"`
            filter_dt_before = process_dt_exit(&args.dt_before, &tz_offset, &None, &utc_now);
            defo!("filter_dt_before {:?}", filter_dt_before);
            filter_dt_after = process_dt_exit(&args.dt_after, &tz_offset, &filter_dt_before, &utc_now);
            defo!("filter_dt_after {:?}", filter_dt_after);
        }
        _ => {
            // normal case: process `-a` value then process `-b` value
            filter_dt_after = process_dt_exit(&args.dt_after, &tz_offset, &None, &utc_now);
            defo!("filter_dt_after {:?}", filter_dt_after);
            filter_dt_before = process_dt_exit(&args.dt_before, &tz_offset, &filter_dt_after, &utc_now);
            defo!("filter_dt_before {:?}", filter_dt_before);
        }
    }

    #[allow(clippy::single_match)]
    match (filter_dt_after, filter_dt_before) {
        (Some(dta), Some(dtb)) => {
            if dta > dtb {
                e_err!("Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
                std::process::exit(EXIT_ERR);
            }
        }
        _ => {}
    }

    // map `CLI_Color_Choice` to `ColorChoice`
    let color_choice: ColorChoice = match args.color_choice {
        CLI_Color_Choice::always => ColorChoice::Always,
        CLI_Color_Choice::auto => ColorChoice::Auto,
        CLI_Color_Choice::never => ColorChoice::Never,
    };
    defo!("color_choice {:?}", color_choice);

    let color_theme: ColorTheme = if args.color_theme_light {
        ColorTheme::Light
    } else {
        ColorTheme::Dark
    };
    defo!("color_theme {:?}", color_theme);
    if color_theme != COLOR_THEME_DEFAULT {
        defo!("ColorThemeGlobal.write({:?})", color_theme);
        match ColorThemeGlobal.write() {
            Ok(mut ctg) => {
                *ctg = color_theme;
            }
            Err(err) => {
                e_err!("ColorThemeGlobal.write() failed: {:?}", err);
                std::process::exit(EXIT_ERR);
            }
        }
    }

    let log_message_separator: String = match unescape::unescape_str(
        args.log_message_separator
            .as_str(),
    ) {
        Ok(val) => val,
        Err(err) => {
            e_err!("{:?}", err);
            std::process::exit(EXIT_ERR);
        }
    };

    let etl_parser_used: EtlParserUsed = if args.etl_parser {
        EtlParserUsed::EtlParser
    } else {
        EtlParserUsed::DissectEtl
    };

    defo!("args.prepend_dt_format {:?}", args.prepend_dt_format);
    let mut prepend_dt_format: Option<String> = None;
    unsafe {
        if PREPEND_DT_FORMAT_PASSED {
            prepend_dt_format = Some(String::from(""));
        }
    }
    if args.prepend_dt_format.is_some()
       && !args.prepend_dt_format.as_ref().unwrap().is_empty() {
        prepend_dt_format = Some(args.prepend_dt_format.unwrap());
    }

    defo!("args.prepend_tz {:?}", args.prepend_tz);
    let cli_opt_prepend_offset: FixedOffset;
    if args.prepend_tz.is_some() {
        match args.prepend_tz {
            Some(fixedoffset) => {
                cli_opt_prepend_offset = fixedoffset;
                if prepend_dt_format.is_none() {
                    prepend_dt_format = Some(String::from(CLI_OPT_PREPEND_FMT));
                }
            }
            None => {
                e_err!("Unable to parse --prepend-tz argument");
                std::process::exit(EXIT_ERR);
            }
        }
    } else if args.prepend_utc {
        cli_opt_prepend_offset = FIXEDOFFSET0.with(|fo0| *fo0);
        if prepend_dt_format.is_none() {
            prepend_dt_format = Some(String::from(CLI_OPT_PREPEND_FMT));
        }
    } else if args.prepend_local {
        cli_opt_prepend_offset = LOCAL_NOW_OFFSET.with(|lno| *lno);
        if prepend_dt_format.is_none() {
            prepend_dt_format = Some(String::from(CLI_OPT_PREPEND_FMT));
        }
    } else {
        cli_opt_prepend_offset = LOCAL_NOW_OFFSET.with(|lno| *lno);
    }

    defo!("prepend_dt_format {:?}", prepend_dt_format);

    defo!("prepend_utc {:?}", args.prepend_utc);
    defo!("prepend_local {:?}", args.prepend_local);
    defo!("cli_opt_prepend_offset {:?}", cli_opt_prepend_offset);

    defo!("prepend_filename {:?}", args.prepend_filename);
    defo!("prepend_filepath {:?}", args.prepend_filepath);
    defo!("prepend_file_align {:?}", args.prepend_file_align);
    defo!("prepend_separator {:?}", args.prepend_separator);
    defo!("log_message_separator {:?}", log_message_separator);
    defo!("etl_parser_used {:?}", etl_parser_used);
    defo!("python_venv {:?}", args.python_venv);
    defo!("journal_output {:?}", args.journal_output);
    defo!("summary {:?}", args.summary);

    (
        paths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_offset,
        prepend_dt_format,
        args.prepend_filename,
        args.prepend_filepath,
        args.prepend_file_align,
        args.prepend_separator,
        log_message_separator,
        etl_parser_used,
        args.python_venv,
        args.journal_output,
        args.summary,
    )
}

// --------------------
// command-line parsing

/// Process the user-passed command-line arguments.
/// Start function `processing_loop`.
/// Determine a process return code.
pub fn main() -> ExitCode {
    let start_time = Instant::now();
    if cfg!(debug_assertions) {
        stack_offset_set(Some(0));
    }
    defn!();

    let (
        paths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_offset,
        cli_prepend_dt_format,
        cli_opt_prepend_filename,
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_prepend_separator,
        log_message_separator,
        etl_parser_used,
        python_venv,
        journal_output,
        cli_opt_summary,
    ) = cli_process_args();

    if cli_opt_summary {
        summary_stats_enable();
    }

    if python_venv {
        let exitcode: ExitCode;
        match venv_create() {
            Result3E::Ok(_) => exitcode = ExitCode::SUCCESS,
            Result3E::Err(err) => {
                e_err!("{}", err);
                exitcode = ExitCode::FAILURE;
            }
            Result3E::ErrNoReprint(_err) => {
                exitcode = ExitCode::FAILURE;
            }
        }
        defx!("exitcode {:?}", exitcode);

        return exitcode;
    }

    // TODO: 2025/11/27 given .etl files but bad Python venv then need to print single error for user

    let mut processed_paths: ProcessPathResults = ProcessPathResults::with_capacity(paths.len() * 4);
    for path in paths.iter() {
        defo!("path {:?}", path);
        let ppaths: ProcessPathResults = process_path(path, true);
        for ppresult in ppaths.into_iter() {
            processed_paths.push(ppresult);
        }
    }

    let ret: bool = processing_loop(
        processed_paths,
        blocksz,
        &filter_dt_after,
        &filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_offset,
        cli_prepend_dt_format,
        cli_opt_prepend_filename,
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_prepend_separator,
        log_message_separator,
        etl_parser_used,
        journal_output,
        cli_opt_summary,
        start_time,
    );

    let exitcode = if ret { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    defx!("exitcode {:?}", exitcode);

    exitcode
}

lazy_static! {
    /// mapping of PathId to received data. Most important collection for function
    /// `processing_loop`.
    /// Must be lazy static (global) so that is may be called from the
    /// `ctrlc::set_handler` signal handler.
    //
    // XXX: there's a few imperfect uses that read then read-again with presumption
    //      `MAP_PATHID_CHANRECVDATUM` was not written to in-between. In the rare
    //      case that occurs, it should not be a problem.
    static ref MAP_PATHID_CHANRECVDATUM: RwLock<MapPathIdChanRecvDatum> = {
        defñ!("lazy_static! map_pathid_chanrecvdatum");

        RwLock::new(MapPathIdChanRecvDatum::new())
    };
    /// flag to signal to main thread should return ASAP.
    /// Polled by function `processing_loop`.
    static ref EXIT_EARLY: RwLock<bool> = {
        defñ!("lazy_static! exit_early");

        RwLock::new(false)
    };
}

/// set a process signal handler
pub fn set_signal_handler() -> anyhow::Result<(), ctrlc::Error> {
    defn!();

    // define the signal handler
    ctrlc::set_handler(move || {
        defn!();
        // XXX: could also use libc::pthread_kill ?

        // drop all receiver channels causing the receiver to close
        // the corresponding sender side will get an error on it's next `send`
        let mut map_pathid_chanrecvdatum = MAP_PATHID_CHANRECVDATUM.write().unwrap();
        defo!("map_pathid_chanrecvdatum.clear() {} entries", (*map_pathid_chanrecvdatum).len());
        (*map_pathid_chanrecvdatum).clear();

        // remove the named temporary files
        // (the entire reason this complex signal handler had to be implemented)
        _ = remove_temporary_files();

        // signal the `processing_loop` to return early
        match EXIT_EARLY.write() {
            Ok(mut exit_early) => {
                *exit_early = true;
            }
            Err(_err) => {
                de_err!("EXIT_EARLY.write().unwrap() failed {}", _err);
            }
        }

        defx!();
    })?;

    defx!();

    Ok(())
}

// -------------------------------------------------------------------------------------------------
// processing threads
// -------------------------------------------------------------------------------------------------

// short-hand helpers
const FILEERRSTUB: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrStub;
const FILEOK: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileOk;

// TODO: leave a long code comment explaining  why I chose this threading
//       pub-sub approach
//       see old archived code to see previous attempts

/// enum to pass filetype-specific data to a file processing thread
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileTypeExecData {
    /// all other log message types (nothing is needed)
    None,
    /// Journal processing thread needs to know the journal output format
    Journal(JournalOutput),
    /// Windows Event Trace Log processing thread needs to know the
    /// python library to use
    Etl(EtlParserUsed),
}

/// Data to initialize a file processing thread.
///
/// * file path `FPath`
/// * unique path ID `PathId`
/// * `FileType`
/// * Block Size `BlockSz`
/// * optional `DateTimeL` as the _after_ datetime filter
/// * optional `DateTimeL` as the _before_ datetime filter
/// * fallback timezone `FixedOffset` for datetime formats without a timezone
///
// TODO: change to a typed `struct ThreadInitData(...)`
type ThreadInitData = (
    FPath,
    PathId,
    FileType,
    FileTypeExecData,
    BlockSz,
    DateTimeLOpt,
    DateTimeLOpt,
    // TODO: rename all `tz_offset` to `fixed_offset`
    FixedOffset,
);

/// Is this the last [`Sysline`] of the log file?
///
/// [`Sysline`]: s4lib::data::sysline::Sysline
type IsLastLogMessage = bool;

/// A single data sent from file processing thread to the main printing thread.
///
/// * optional [`LogMessage`] (`Sysline`/`FixedStruct`)
/// * optional [`Summary`]
/// * is this the last LogMessage (`Sysline`/`FixedStruct`)?
/// * `FileProcessingResult`
///
/// There should never be a `LogMessage` and a `FileSummary` received
/// simultaneously.
///
/// [`LogMessage`]: self::LogMessage
/// [`Summary`]: s4lib::readers::summary::Summary
// TODO: [2025/12] this should support carrying an Error in a new variant
//       like `ProcessingErrContinue(String)` so that file processing
//       threads can report non-fatal errors to the main printing thread.
#[derive(Debug)]
enum ChanDatum {
    /// first data sent from file processing thread to main printing thread.
    /// exactly one must be sent first during the entire thread.
    FileInfo(DateTimeLOpt, FileProcessingResultBlockZero),
    /// data sent from file processing thread to main printing thread
    /// a processed log message for printing.
    /// zero or more of these are sent during the entire thread
    NewMessage(LogMessage, IsLastLogMessage),
    /// last data sent from file processing thread to main printing thread.
    /// zero or one should be sent during the entire thread
    ///
    // XXX: Would be ideal to store `FileProcessingResultBlockZero` in the
    //      `Summary`. But the `FileProcessingResultBlockZero` has an
    //      explicit lifetime because it can carry a `Error`.
    //      This complicates everything.
    FileSummary(SummaryOpt, FileProcessingResultBlockZero),
}

impl fmt::Display for ChanDatum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChanDatum::FileInfo(dt, _) => write!(f, "FileInfo({:?})", dt),
            ChanDatum::NewMessage(lm, illm) => write!(f, "NewMessage({:?}, {:?})", lm, illm),
            ChanDatum::FileSummary(..) => write!(f, "FileSummary(..)"),
        }
    }
}

type MapPathIdDatum = BTreeMap<PathId, (LogMessage, IsLastLogMessage)>;

/// Sender channel (used by each file processing thread).
///
/// Tuple described in [`ChanDatum`].
///
/// [`ChanDatum`]: self::ChanDatum
type ChanSendDatum = crossbeam_channel::Sender<ChanDatum>;

/// Receiver channel (used by main printing loop).
///
/// Tuple described in [`ChanDatum`].
///
/// [`ChanDatum`]: self::ChanDatum
type ChanRecvDatum = crossbeam_channel::Receiver<ChanDatum>;

type MapPathIdChanRecvDatum = BTreeMap<PathId, ChanRecvDatum>;

/// Helper to send a [`ChanDatum::NewMessage`] to the main printing thread
/// and print an error if there was an error sending.
#[inline(always)]
fn chan_send(
    chan_send_dt: &ChanSendDatum,
    chan_datum: ChanDatum,
    _path: &FPath,
) {
    if cfg!(debug_assertions) {
        let _err_s: String = match &chan_datum {
            ChanDatum::FileInfo(_, fileprocessingresult) => {
                format!("ChanDatum::FileInfo({:?})", fileprocessingresult)
            },
            ChanDatum::NewMessage(..) => String::from("ChanDatum::NewMessage(..)"),
            ChanDatum::FileSummary(_, fileprocessingresult) => {
                format!("ChanDatum::FileSummary({:?})", fileprocessingresult)
            }
        };
        def1ñ!("chan_send(..., chan_datum={}, _path={:?})", _err_s, _path);
    }
    match chan_send_dt.send(chan_datum) {
        Ok(_) => {}
        Err(_err) => de_err!("chan_send_dt.send(…) failed {} for {:?}", _err, _path),
    }
}

/// This creates a [`SyslogProcessor`] and processes the file.<br/>
/// If it is a syslog file, then continues processing by sending each
/// processed [`Sysline`]  in a [`ChanDatum`] through a [channel] to the main
/// thread which will print it.
///
/// This function drives a `SyslogProcessor` instance through it's formal
/// stages (described by [`ProcessingStage`]). During each stage, this function
/// makes the expected `SyslogProcessor::find*` calls (and does some other
/// checks).
///
/// [`ProcessingStage`]: s4lib::readers::syslogprocessor::ProcessingStage
/// [`SyslogProcessor`]: s4lib::readers::syslogprocessor::SyslogProcessor
/// [`Sysline`]: s4lib::data::sysline::Sysline
/// [channel]: self::ChanSendDatum
fn exec_syslogprocessor(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
    _tname: &str,
    _tid: thread::ThreadId,
) {
    let (
        path,
        _pathid,
        filetype,
        _filetypeexecdata,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("({:?})", path);
    debug_assert!(matches!(_filetypeexecdata, FileTypeExecData::None));

    // TODO: add thread runtime tracking with `Instant::now()`
    //       create an `Instant::now()` in the `SyslogProcessor::new()`
    //       and then store another `Instant::now()` during creation of
    //       the `summary_complete()`.
    //       Store both in the `SyslogProcessor::Summary`.
    //       Add a line to print it in the appropriate printing function.
    //       I think during printing the "Processed:" section of the printed summary.
    //       Do this for all other Readers as well.
    let mut syslogproc: SyslogProcessor = match SyslogProcessor::new(
        path.clone(),
        filetype,
        blocksz,
        tz_offset,
        filter_dt_after_opt,
        filter_dt_before_opt,
    ) {
        Ok(val) => val,
        Err(err) => {
            let err_string = err.to_string();
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(DateTimeLOpt::None, FileProcessingResultBlockZero::FileErrIoPath(err)),
                &path,
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                LogMessageType::Sysline,
                blocksz,
                Some(err_string)
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(Some(summary), FILEERRSTUB),
                &path
            );
            defx!("({:?}) thread will return early due to error", path);
            return;
        }
    };
    deo!("{:?}({}): syslogproc {:?}", _tid, _tname, syslogproc);

    // send `ChanDatum::FileInfo`
    let result = syslogproc.process_stage0_valid_file_check();
    let mtime = syslogproc.mtime();
    let dt = systemtime_to_datetime(&tz_offset, &mtime);
    chan_send(
        &chan_send_dt,
        ChanDatum::FileInfo(DateTimeLOpt::Some(dt), result),
        &path,
    );

    deo!("{:?}({}): processing stage 1", _tid, _tname);

    let result = syslogproc.process_stage1_blockzero_analysis();
    match result {
        FileProcessingResultBlockZero::FileOk => {}
        _ => {
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(syslogproc.summary_complete()),
                    result,
                ),
                &path
            );
            defx!("({:?}) return during stage 1 due to error", path);
            return;
        }
    }

    deo!("{:?}({}): processing stage 2", _tid, _tname);

    // find first sysline acceptable to the passed filters
    match syslogproc.process_stage2_find_dt(&filter_dt_after_opt) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(syslogproc.summary_complete()),
                    result,
                ),
                &path
            );
            defx!("({:?}) return during stage 2, no datetime within filters", path);
            return;
        }
    }

    let mut file_err: Option<FileProcessingResultBlockZero> = None;
    let mut sent_is_last: bool = false; // sanity check sending of `is_last`
    let mut fo1: FileOffset = 0;
    let search_more: bool;
    let result: ResultFindSysline = syslogproc.find_sysline_between_datetime_filters(0);
    match result {
        ResultFindSysline::Found((fo, syslinep)) => {
            fo1 = fo;
            let is_last: IsLastLogMessage = syslogproc.is_sysline_last(&syslinep) as IsLastLogMessage;
            deo!("{:?}({}): Found, chan_send_dt.send({:p}, None, {});", _tid, _tname, syslinep, is_last);
            chan_send(
                &chan_send_dt,
                ChanDatum::NewMessage(
                    LogMessage::Sysline(syslinep),
                    is_last,
                ),
                &path
            );
            if is_last {
                // XXX: sanity check
                assert!(
                    !sent_is_last,
                    "is_last {}, yet sent_is_last was also {} (is_last was already sent!)",
                    is_last, sent_is_last
                );
                sent_is_last = true;
                search_more = false;
            } else {
                search_more = true;
            }
        }
        ResultFindSysline::Done => {
            search_more = false;
        }
        ResultFindSysline::Err(err) => {
            deo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", _tid, _tname, err);
            file_err = Some(FileProcessingResultBlockZero::FileErrIoPath(err));
            search_more = false;
        }
    }

    if !search_more {
        deo!("{:?}({}): last line so quit searching", _tid, _tname);
        let summary_opt: SummaryOpt = Some(syslogproc.process_stage4_summary());
        chan_send(
            &chan_send_dt,
            ChanDatum::FileSummary(
                summary_opt,
                file_err.unwrap_or(FILEOK),
            ),
            &path
        );
        defx!("({:?}) return early during stage 2, at last line so no more searching to do", path);

        return;
    }

    deo!("{:?}({}): processing stage 3", _tid, _tname);

    // start stage 3 - find all proceeding syslines acceptable to the passed filters
    syslogproc.process_stage3_stream_syslines();
    // the majority of sysline processing for this file occurs in this loop
    let mut syslinep_last_opt: Option<SyslineP> = None;
    loop {
        // TODO: [2022/06/20] see note about refactoring `find` functions so
        //                    they are more intuitive
        let result: ResultFindSysline = syslogproc.find_sysline_between_datetime_filters(fo1);
        match result {
            ResultFindSysline::Found((fo, syslinep)) => {
                let syslinep_tmp = syslinep.clone();
                let is_last: IsLastLogMessage = syslogproc.is_sysline_last(&syslinep);
                chan_send(
                    &chan_send_dt,
                    ChanDatum::NewMessage(
                        LogMessage::Sysline(syslinep),
                        is_last,
                    ),
                    &path
                );
                fo1 = fo;
                // XXX: sanity check
                if is_last {
                    assert!(
                        !sent_is_last,
                        "is_last {}, yet sent_is_last was also {} (is_last was already sent!)",
                        is_last, sent_is_last
                    );
                    break;
                }
                // try to drop the prior SyslineP (and associated data)
                if let Some(syslinep_last) = syslinep_last_opt {
                    syslogproc.drop_data_try(&syslinep_last);
                }
                syslinep_last_opt = Some(syslinep_tmp);
            }
            ResultFindSysline::Done => {
                break;
            }
            ResultFindSysline::Err(err) => {
                deo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", _tid, _tname, err);
                e_err!("syslogprocessor.find_sysline({}) {} for {:?}", fo1, err, path);
                file_err = Some(FileProcessingResult::FileErrIoPath(err));
                break;
            }
        }
    }

    deo!("{:?}({}): processing stage 4", _tid, _tname);

    syslogproc.process_stage4_summary();

    let summary = syslogproc.summary_complete();
    deo!("{:?}({}): last chan_send_dt.send((None, {:?}, {}));", _tid, _tname, summary, false);
    chan_send(&chan_send_dt, ChanDatum::FileSummary(Some(summary), file_err.unwrap_or(FILEOK)), &path);

    defx!("({:?})", path);
}

/// This function drives a [`FixedStructReader`] instance through it's
/// processing. It makes the expected "find" calls (and other checks) during
/// each stage. Similar to [`exec_syslogprocessor`].
fn exec_fixedstructprocessor(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
    _tname: &str,
    _tid: thread::ThreadId,
) {
    let (
        path,
        _pathid,
        filetype,
        _filetypeexecdata,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("{:?}({}): ({:?})", _tid, _tname, path);
    debug_assert!(matches!(filetype, FileType::FixedStruct { .. }));
    debug_assert!(matches!(_filetypeexecdata, FileTypeExecData::None));
    let logmessagetype: LogMessageType = match filetype {
        FileType::FixedStruct { .. } => LogMessageType::FixedStruct,
        _ => {
            e_err!(
                "exec_fixedstructprocessor called with wrong filetype {:?} for path {:?}",
                filetype, path
            );
            return;
        }
    };

    let mut fixedstructreader: FixedStructReader = match FixedStructReader::new(
        path.clone(),
        filetype,
        blocksz,
        tz_offset,
        filter_dt_after_opt,
        filter_dt_before_opt,
    ) {
        ResultFixedStructReaderNew::FileErrIo(err) => {
            let err_string = err.to_string();
            let _err = err_string.clone();
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrIoPath(err)
                ),
                &path
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                logmessagetype,
                blocksz,
                Some(err_string),
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(Some(summary), FILEERRSTUB),
                &path
            );
            defx!("FixedStructReader::new returned FileErrIo for {:?} ({})", path, _err);
            return;
        }
        ResultFixedStructReaderNew::FileErrEmpty => {
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrEmpty
                ),
                &path
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                logmessagetype,
                blocksz,
                None,
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(summary), FileProcessingResultBlockZero::FileErrEmpty
                ),
                &path
            );
            defx!("FixedStructReader::new returned FileErrEmpty for {:?}", path);
            return;
        }
        ResultFixedStructReaderNew::FileErrTooSmall(err_string) => {
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrTooSmallS(err_string.clone()),
                ),
                &path,
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                logmessagetype,
                blocksz,
                Some(err_string.clone()),
            );
            let _err_s = err_string.clone();
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(summary), FileProcessingResultBlockZero::FileErrTooSmallS(err_string)
                ),
                &path
            );
            defx!("FixedStructReader::new returned FileErrTooSmall({}) for {:?}", _err_s, path);
            return;
        }
        ResultFixedStructReaderNew::FileErrNoValidFixedStruct => {
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrNoValidFixedStruct,
                ),
                &path
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                logmessagetype,
                blocksz,
                None,
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(summary),
                    FileProcessingResultBlockZero::FileErrNoValidFixedStruct,
                ),
                &path
            );
            defx!("FixedStructReader::new returned FileErrNoValidFixedStruct for {:?}", path);
            return;
        }
        ResultFixedStructReaderNew::FileErrNoFixedStructWithinDtFilters => {
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrNoFixedStructInDtRange,
                ),
                &path
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                logmessagetype,
                blocksz,
                None,
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(summary),
                    FileProcessingResultBlockZero::FileErrNoFixedStructInDtRange,
                ),
                &path
            );
            defx!("FixedStructReader::new returned FileErrNoFixedStructInDtRange for {:?}", path);
            return;
        }
        ResultFixedStructReaderNew::FileOk(val) => val,
    };
    defo!("{:?}({}): fixedstructreader {:?}", _tid, _tname, fixedstructreader);

    // send `ChanDatum::FileInfo`
    let mtime = fixedstructreader.mtime();
    let dt = systemtime_to_datetime(&tz_offset, &mtime);
    chan_send(
        &chan_send_dt,
        ChanDatum::FileInfo(DateTimeLOpt::Some(dt), FILEOK),
        &path
    );

    let mut file_err: Option<FileProcessingResultBlockZero> = None;

    let mut fo: FileOffset = match fixedstructreader.fileoffset_first() {
        Some(fo) => fo,
        None => {
            de_wrn!("fileoffset_first returned None for {:?}", path);

            // send `ChanDatum::FileSummary`
            let summary = fixedstructreader.summary_complete();
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(summary),
                    file_err.unwrap_or(FILEOK),
                ),
                &path
            );

            defx!("({:?})", path);
            return;
        }
    };
    defo!("starting process_entry_at loop from fileoffset_first {:?}", fo);
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    loop {
        let fo_next = match fixedstructreader.process_entry_at(fo, &mut buffer) {
            ResultFindFixedStruct::Found((fo_, fixedstruct)) => {
                defo!("ResultFindFixedStruct::Found({}, …)", fo_);
                let is_last = fixedstructreader.is_last(&fixedstruct);
                chan_send(
                    &chan_send_dt,
                    ChanDatum::NewMessage(
                        LogMessage::FixedStruct(fixedstruct),
                        is_last,
                    ),
                    &path
                );

                fo_
            }
            ResultFindFixedStruct::Done => {
                defo!("ResultFindFixedStruct::Done");
                break;
            }
            ResultFindFixedStruct::Err((fo_opt, err)) => {
                defo!("ResultFindFixedStruct::Err({:?}, {:?})", fo_opt, err);
                de_err!("process_entry_at({}) failed; {} for {:?}", fo, err, path);
                file_err = Some(FileProcessingResultBlockZero::FileErrIoPath(err));
                match fo_opt {
                    // an offset within an error signifies a recoverable error
                    Some(fo_) => fo_,
                    // `None` signifies an unrecoverable error and no more processing
                    // should be attempted
                    None => break,
                }
            }
        };
        fo = fo_next;
    }

    // send `ChanDatum::FileSummary`
    let summary = fixedstructreader.summary_complete();
    chan_send(
        &chan_send_dt,
        ChanDatum::FileSummary(
            Some(summary),
            file_err.unwrap_or(FILEOK),
        ),
        &path
    );

    defx!("({:?})", path);
}

/// This function drives a [`EvtxReader`] instance through it's processing.
/// Similar to [`exec_syslogprocessor`].
fn exec_evtxprocessor(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
    _tname: &str,
    _tid: thread::ThreadId,
) {
    let (
        path,
        _pathid,
        filetype,
        _filetypeexecdata,
        _blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("{:?}({}): ({:?}, {:?}, {:?})", _tid, _tname, path, filetype, tz_offset);
    debug_assert!(filetype.is_evtx());
    debug_assert!(matches!(_filetypeexecdata, FileTypeExecData::None));

    let mut evtxreader: EvtxReader = match EvtxReader::new(
        path.clone(),
        filetype,
    ) {
        Ok(val) => val,
        Err(err) => {
            let err_string = err.to_string();
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrIo(err)
                ),
                &path
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                LogMessageType::Evtx,
                0,
                Some(err_string)
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(Some(summary), FILEERRSTUB),
                &path
            );
            defx!("({:?}) thread will return early due to error", path);
            return;
        }
    };
    defo!("{:?}({}): evtxreader {:?}", _tid, _tname, evtxreader);

    // send `ChanDatum::FileInfo`
    let mtime = evtxreader.mtime();
    let dt = systemtime_to_datetime(&tz_offset, &mtime);
    chan_send(
        &chan_send_dt,
        ChanDatum::FileInfo(DateTimeLOpt::Some(dt), FILEOK),
        &path
    );

    evtxreader.analyze(
        &filter_dt_after_opt,
        &filter_dt_before_opt,
    );

    while let Some(evtx) = evtxreader.next()
    {
        let is_last = false;
        chan_send(
            &chan_send_dt,
            ChanDatum::NewMessage(
                LogMessage::Evtx(evtx),
                is_last,
            ),
            &path
        );
    }

    let summary = evtxreader.summary_complete();
    chan_send(
        &chan_send_dt,
        ChanDatum::FileSummary(
            Some(summary),
            FILEOK,
        ),
        &path
    );

    defx!("({:?})", path);
}

/// This function drives a [`PyEventReader`] instance through it's processing.
/// Similar to [`exec_syslogprocessor`].
fn exec_pyeventprocessor(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
    _tname: &str,
    _tid: thread::ThreadId,
) {
    let (
        path,
        _pathid,
        filetype,
        filetypeexecdata,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("{:?}({}): ({:?}, {:?}, {:?})", _tid, _tname, path, filetype, tz_offset);
    debug_assert!(filetype.is_etl() || filetype.is_odl());
    
    let etl_parser_used: EtlParserUsed = match filetypeexecdata {
        FileTypeExecData::Etl(etl_parser_used) => etl_parser_used,
        FileTypeExecData::None => EtlParserUsed::EtlParser,
        FileTypeExecData::Journal { .. } => {
            debug_panic!(
                "exec_pyeventprocessor called with filetypeexecdata {:?} for path {:?}",
                filetypeexecdata, path
            );
            e_err!("filetypeexecdata is {:?} not Etl which is unexpected", filetypeexecdata);
            defx!("({:?}) return early due filetypeexecdata is not Etl, is {:?}", path, filetypeexecdata);
            return;
        }
    };

    let pyevent_type: PyEventType = match filetype {
        FileType::Etl { .. } => {
            PyEventType::Etl
        }
        FileType::Odl { .. } => {
            PyEventType::Odl
        }
        _ => {
            debug_panic!(
                "exec_pyeventprocessor called with wrong filetype {:?} for path {:?}",
                filetype, path
            );
            e_err!("filetype is {:?} not Etl/Odl which is unexpected", filetype);
            defx!("({:?}) return early due filetype is not Etl/Odl", path);
            return;
        }
    };

    let mut py_event_reader: PyEventReader = match PyEventReader::new(
        path.clone(),
        etl_parser_used,
        filetype,
        tz_offset,
        blocksz as PipeSz,
    ) {
        Ok(val) => val,
        Err(err) => {
            let err_string = err.to_string();
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(
                    DateTimeLOpt::None,
                    FileProcessingResultBlockZero::FileErrIo(err)
                ),
                &path
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                LogMessageType::PyEvent,
                0,
                Some(err_string)
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(Some(summary), FILEERRSTUB),
                &path
            );
            defx!("({:?}) thread will return early due to error", path);
            return;
        }
    };
    defo!("{:?}({}): py_event_reader {:?}", _tid, _tname, py_event_reader);

    // send `ChanDatum::FileInfo`
    let mtime = py_event_reader.mtime();
    let dt = systemtime_to_datetime(&tz_offset, &mtime);
    chan_send(
        &chan_send_dt,
        ChanDatum::FileInfo(DateTimeLOpt::Some(dt), FILEOK),
        &path
    );

    let mut result_err: Option<FileProcessingResult<Error>> = None;
    // call py_event_reader.next() until exhausted
    loop {
        match py_event_reader.next(
            &filter_dt_after_opt,
            &filter_dt_before_opt,
        ) {
            ResultNextPyDataEvent::Found(etl_event) => {
                def1o!("ResultNextPyDataEvent::Found({} bytes); chan_send()…", etl_event.len());
                chan_send(
                    &chan_send_dt,
                    ChanDatum::NewMessage(
                        LogMessage::PyEvent(etl_event, pyevent_type),
                        false,
                    ),
                    &path
                );
            }
            ResultNextPyDataEvent::Done => {
                def1o!("ResultNextPyDataEvent::Done");
                break;
            }
            ResultNextPyDataEvent::Err(err) => {
                def1o!("ResultNextPyDataEvent::Err({:?})", err);
                de_err!("etl_reader.next(…) returned {}", err);
                result_err = Some(FileProcessingResult::FileErrIo(err));
                break;
            }
            ResultNextPyDataEvent::ErrIgnore(_err) => {
                def1o!("ResultNextPyDataEvent::ErrIgnore({:?})", _err);
                de_err!("etl_reader.next(…) returned {} (Ignored)", _err);
            }
        }
    };

    let summary = py_event_reader.summary_complete();
    chan_send(
        &chan_send_dt,
        ChanDatum::FileSummary(
            Some(summary),
            result_err.unwrap_or(FILEOK),
        ),
        &path
    );

    defx!("({:?})", path);
}

/// This function drives a [`JournalReader`] instance through it's processing.
/// Similar to [`exec_syslogprocessor`].
fn exec_journalprocessor(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
    _tname: &str,
    _tid: thread::ThreadId,
) {
    let (
        path,
        _pathid,
        filetype,
        filetypeexecdata,
        _blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("{:?}({}): ({:?})", _tid, _tname, path);
    debug_assert!(filetype.is_journal());
    debug_assert!(matches!(filetypeexecdata, FileTypeExecData::Journal(_)));

    let journal_output: JournalOutput = match filetypeexecdata {
        FileTypeExecData::Journal(journal_output) => journal_output,
        _ => {
            e_err!("filetypeexecdata is not Journal which is unexpected");
            defx!("({:?}) return early due filetypeexecdata is not Journal", path);
            return;
        }
    };

    let mut journalreader: JournalReader = match JournalReader::new(
        path.clone(),
        journal_output,
        tz_offset,
        filetype,
    ) {
        Ok(val) => val,
        Err(err) => {
            let err_string = err.to_string();
            // send `ChanDatum::FileInfo`
            chan_send(
                &chan_send_dt,
                ChanDatum::FileInfo(DateTimeLOpt::None, FileProcessingResultBlockZero::FileErrIoPath(err)),
                &path,
            );
            // send `ChanDatum::FileSummary`
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                LogMessageType::Journal,
                0,
                Some(err_string)
            );
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(Some(summary), FILEERRSTUB),
                &path
            );
            defx!("({:?}) thread will return early due to error", path);
            return;
        }
    };
    defo!("{:?}({}): journalreader {:?}", _tid, _tname, journalreader);

    // send `ChanDatum::FileInfo`
    let mtime = journalreader.mtime();
    let dt = systemtime_to_datetime(&tz_offset, &mtime);
    chan_send(
        &chan_send_dt,
        ChanDatum::FileInfo(DateTimeLOpt::Some(dt), FILEOK),
        &path
    );

    defo!("filter_dt_after_opt {:?}", filter_dt_after_opt);
    let ts_filter_after = datetimelopt_to_realtime_timestamp_opt(&filter_dt_after_opt);
    defo!("ts_filter_after {:?}", ts_filter_after);

    match journalreader.analyze(&ts_filter_after) {
        Ok(_) => {}
        Err(err) => {
            let mut summary = journalreader.summary_complete();
            summary.error = Some(err.to_string());
            chan_send(
                &chan_send_dt,
                ChanDatum::FileSummary(
                    Some(summary),
                    FileProcessingResult::FileErrIoPath(err)
                ),
                &path
            );
            defx!("({:?}) ERROR: journalreader.analyze(…) failed", path);
            return;
        }
    }
    defo!("filter_dt_before_opt {:?}", filter_dt_before_opt);
    let ts_filter_before = datetimelopt_to_realtime_timestamp_opt(&filter_dt_before_opt);
    defo!("ts_filter_before {:?}", ts_filter_before);

    let mut result_err: Option<FileProcessingResult<Error>> = None;
    loop {
        let result = journalreader.next(&ts_filter_before);
        match result {
            ResultNext::Found(journalentry) => {
                let is_last: IsLastLogMessage = false;
                chan_send(
                    &chan_send_dt,
                    ChanDatum::NewMessage(
                        LogMessage::Journal(journalentry),
                        is_last,
                    ),
                    &path
                );
            }
            ResultNext::Done => {
                break;
            }
            ResultNext::Err(err) => {
                de_err!("journalreader.next(…) returned {}", err);
                result_err = Some(FileProcessingResult::FileErrIo(err));
                break;
            }
            ResultNext::ErrIgnore(_err) =>
                de_err!("journalreader.next(…) returned {} (Ignored)", _err)
        }
    }

    let summary = journalreader.summary_complete();
    chan_send(
        &chan_send_dt,
        ChanDatum::FileSummary(
            Some(summary),
            result_err.unwrap_or(FILEOK),
        ),
        &path
    );

    defx!("({:?})", path);
}

/// Thread entry point for processing one file. Calls the correct `exec_`
/// function.
fn exec_fileprocessor_thread(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
) {
    if cfg!(debug_assertions) {
        stack_offset_set(Some(2));
    }

    let thread_cur: thread::Thread = thread::current();
    let tid: thread::ThreadId = thread_cur.id();

    #[cfg(any(debug_assertions, test))]
    let tname: &str = <&str>::clone(
        &thread_cur
            .name()
            .unwrap_or(""),
    );
    #[cfg(not(any(debug_assertions, test)))]
    let tname: &str = "";

    match thread_init_data.2 {
        FileType::FixedStruct { .. } => exec_fixedstructprocessor(chan_send_dt, thread_init_data, tname, tid),
        FileType::Etl { .. } => exec_pyeventprocessor(chan_send_dt, thread_init_data, tname, tid),
        FileType::Evtx { .. } => exec_evtxprocessor(chan_send_dt, thread_init_data, tname, tid),
        FileType::Journal { .. } => exec_journalprocessor(chan_send_dt, thread_init_data, tname, tid),
        FileType::Odl { .. } => exec_pyeventprocessor(chan_send_dt, thread_init_data, tname, tid),
        FileType::Text { .. } => exec_syslogprocessor(chan_send_dt, thread_init_data, tname, tid),
        _ => {
            debug_panic!("exec_fileprocessor_thread called with unexpected filetype {:?}", thread_init_data.2);
            e_err!("exec_fileprocessor_thread called with unexpected filetype {:?}", thread_init_data.2);
        }
    }
}

/// Five seems like a good number for the channel capacity *shrug*
const CHANNEL_CAPACITY: usize = 5;

/// The main [`LogMessage`] processing and printing loop.
///
/// 1. creates threads to process each file
///
/// 2. waits on each thread to receive a processed `LogMessage`
///    _or_ a closed [channel]
///    a. prints received `LogMessage` in datetime order
///    b. repeat 2. until each thread sends a `IsLastLogMessage` value `true`
///
/// 3. print each [`Summary`] (if CLI option `--summary`)
///
/// This main thread should be the only thread that prints to stdout.<br/>
/// In `--release` builds, other file processing threads may rarely print
/// messages to stderr.<br/>
/// In debug builds, other file processing threads print verbosely.
///
/// [`Summary`]: s4lib::readers::summary::Summary
/// [channel]: self::ChanRecvDatum
// XXX: this is a very large function
#[allow(clippy::too_many_arguments)]
fn processing_loop(
    mut paths_results: ProcessPathResults,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeLOpt,
    filter_dt_before_opt: &DateTimeLOpt,
    tz_offset: FixedOffset,
    color_choice: ColorChoice,
    cli_opt_prepend_offset: FixedOffset,
    cli_prepend_dt_format: Option<String>,
    cli_opt_prepend_filename: bool,
    cli_opt_prepend_filepath: bool,
    cli_opt_prepend_file_align: bool,
    cli_prepend_separator: String,
    log_message_separator: String,
    etl_parser_used: EtlParserUsed,
    journal_output: JournalOutput,
    cli_opt_summary: bool,
    start_time: Instant,
) -> bool {
    defn!(
        "({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})",
        paths_results,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        color_choice,
        cli_opt_prepend_offset,
        cli_prepend_dt_format,
        journal_output,
        cli_opt_summary
    );

    if paths_results.is_empty() {
        defx!("paths_results.is_empty(); nothing to do");
        return true;
    }

    // precount the number of valid files that will be processed
    let file_count: usize = paths_results
        .iter()
        .filter(|x| matches!(x, ProcessPathResult::FileValid(..)))
        .count();

    // create various mappings of PathId -> Thingy:
    //
    // separate `ProcessPathResult`s into different collections, valid and invalid
    //
    // the valid `map_pathid_results` is used extensively
    let mut map_pathid_results = MapPathIdToProcessPathResult::with_capacity(file_count);
    // `invalid` is used to help summarize why some files were not processed
    let mut map_pathid_results_invalid = MapPathIdToProcessPathResultOrdered::new();
    // use `map_pathid_path` for iterating, it is a BTreeMap (which iterates in consistent key order)
    let mut map_pathid_path = MapPathIdToFPath::new();
    // map `PathId` to the last `FileProcessResult
    let mut map_pathid_file_processing_result = MapPathIdToFileProcessingResultBlockZero::with_capacity(file_count);
    // map `PathId` to file's file-system _Modified Time_ attribute
    let mut map_pathid_modified_time = MapPathIdToModifiedTime::with_capacity(file_count);
    // map `PathId` to  acknowledgement of receipt of a `ChanDatum::FileInfo` message.
    // Every `exec_*process` thread must first send a `ChanDatum::FileInfo` message
    // before sending any `ChanDatum::NewMessage`.
    // This tracks the threads for which either a `ChanDatum::FileInfo`
    // is expected and whether it's been received. When all expected `ChanDatum::FileInfo`
    // messages have been received, then any errors should be printed, and this map is cleared.
    // In other words, it holds 3 states in this sequence:
    // 1. maps to values that are not all `true` and not empty
    // 2. maps to values that are all `true` and not empty
    // 3. cleared (and empty)
    // This helps to coordinate printing error messages in a predictable manner
    // and order. See Issue #104
    let mut map_pathid_received_fileinfo = HashMap::<PathId, bool>::with_capacity(file_count);
    // map `PathId` to `FileType`
    let mut map_pathid_filetype = MapPathIdToFileType::with_capacity(file_count);
    // map `PathId` to `LogMessageType`
    let mut map_pathid_logmessagetype = MapPathIdToLogMessageType::with_capacity(file_count);
    let mut paths_total: usize = 0;

    for (pathid_counter, processpathresult) in paths_results
        .drain(..)
        .enumerate()
    {
        defo!("match {:?}", processpathresult);
        match processpathresult {
            // XXX: use `ref` to avoid "use of partially moved value" error
            ProcessPathResult::FileValid(ref path, ref filetype) => {
                if matches!(filetype, FileType::Unparsable) {
                    // known unparsable file
                    defo!("paths_invalid_results.push(FileErrUnparsable)");
                    map_pathid_results_invalid.insert(
                        pathid_counter,
                        ProcessPathResult::FileErrNotSupported(path.clone(), None),
                    );
                    paths_total += 1;
                    continue;
                }
                if filetype.is_journal() {
                    defo!("load_library_systemd()");
                    match load_library_systemd() {
                        LoadLibraryError::Ok => {}
                        LoadLibraryError::Err(err) => {
                            e_err!("load_library_systemd() failed, library {:?}; {}", LIB_NAME_SYSTEMD, err);
                            defo!("paths_invalid_results.push(FileErrLoadingLibrary) (LoadLibraryError::Err)");
                            map_pathid_results_invalid.insert(
                                pathid_counter,
                                ProcessPathResult::FileErrLoadingLibrary(path.clone(), LIB_NAME_SYSTEMD, *filetype)
                            );
                            paths_total += 1;
                            continue;
                        }
                        LoadLibraryError::PrevErr => {
                            defo!("paths_invalid_results.push(FileErrLoadingLibrary) (LoadLibraryError::PrevErr)");
                            map_pathid_results_invalid.insert(
                                pathid_counter,
                                ProcessPathResult::FileErrLoadingLibrary(path.clone(), LIB_NAME_SYSTEMD, *filetype)
                            );
                            paths_total += 1;
                            continue;
                        }
                    }
                }
                // XXX: do an early check for easy to dismiss files
                //      this is a performance optimization for processing large
                //      numbers of files, as it avoids spawning a thread for each
                //      file.
                //      Mild slow down as it does an additional filesystem lookup by
                //      requesting the path's metadata.
                //      One project design principle is to avoid filesystem reads where possible,
                //      so this is a small violation of that but worth it.
                //      Relates to Issue #270
                if ! filetype.is_archived() {
                    let path_std = fpath_to_path(path);
                    let metadata = match path_std.metadata() {
                        Ok(metadata) => metadata,
                        Err(err) => {
                            e_err!("path.metadata() failed; {}", err);
                            defo!("paths_invalid_results.push(FileErrIoPath)");
                            map_pathid_results_invalid.insert(
                                pathid_counter,
                                ProcessPathResult::FileErr(path.clone(), err.to_string()),
                            );
                            paths_total += 1;
                            continue;
                        }
                    };
                    let flen = metadata.len();
                    if flen == 0 {
                        defo!("paths_invalid_results.push(FileErrEmpty)");
                        map_pathid_results_invalid.insert(
                            pathid_counter,
                            ProcessPathResult::FileErrEmpty(path.clone(), *filetype),
                        );
                        paths_total += 1;
                        continue;
                    } else if flen <= FILE_TOO_SMALL_SZ {
                        defo!("paths_invalid_results.push(FileErrTooSmall)");
                        map_pathid_results_invalid.insert(
                            pathid_counter,
                            ProcessPathResult::FileErrTooSmall(path.clone(), *filetype, flen),
                        );
                        paths_total += 1;
                        continue;
                    }
                }
                defo!("map_pathid_results.push({:?})", path);
                map_pathid_path.insert(pathid_counter, path.clone());
                map_pathid_filetype.insert(pathid_counter, *filetype);
                map_pathid_received_fileinfo.insert(pathid_counter, false);
                let logmessagetype: LogMessageType = filetype.to_logmessagetype();
                map_pathid_logmessagetype.insert(pathid_counter, logmessagetype);
                map_pathid_results.insert(pathid_counter, processpathresult);
            }
            _ => {
                defo!("paths_invalid_results.push({:?})", processpathresult);
                map_pathid_results_invalid.insert(pathid_counter, processpathresult);
            }
        };
        paths_total += 1;
    }

    // no more changes to these mappings so shrink them
    map_pathid_filetype.shrink_to_fit();
    map_pathid_logmessagetype.shrink_to_fit();
    map_pathid_results.shrink_to_fit();

    // rebind to be immutable just to be extra cautious
    let map_pathid_filetype = map_pathid_filetype;
    let map_pathid_logmessagetype = map_pathid_logmessagetype;

    for (_pathid, result_invalid) in map_pathid_results_invalid.iter() {
        match result_invalid {
            ProcessPathResult::FileErrEmpty(_path, _filetype) => {
                // behave like Unix tools; do not print errors about empty files
            }
            ProcessPathResult::FileErrTooSmall(_path, _filetype, _len) => {
                // behave like Unix tools; do not print errors about small files
            }
            ProcessPathResult::FileErrNoPermissions(path) =>
                e_err!("not enough permissions {:?}", path),
            ProcessPathResult::FileErrNotSupported(_path, _message) => {
                // behave like Unix tools; do not print errors about unsupported files
                // TODO: this should distinguish between files passed directly and those
                //       found via directory walk
                //       Files passed directly that are not supported should be printed (though
                //       currently such files are treated as Text Utf8 files, this different
                //       part of code should not make such presumptions).
            }
            ProcessPathResult::FileErrNotAFile(path) =>
                e_err!("not a file {:?}", path),
            ProcessPathResult::FileErrNotExist(path) =>
                e_err!("path does not exist {:?}", path),
            ProcessPathResult::FileErrLoadingLibrary(path, libname, ft) =>
                e_err!("failed to load library {:?} for {:?} {:?}", libname, ft, path),
            ProcessPathResult::FileErr(path, message) =>
                e_err!("{} for {:?}", message, path),
            ProcessPathResult::FileValid(..) => {}
        }
    }

    if !map_pathid_path.is_empty() {
        // there may be threads to process (and possible NamedTemporaryFiles to create)
        // so set up signal handling
        match set_signal_handler() {
            Ok(_) => {}
            Err(err) => {
                e_err!("set_signal_handler() failed: {:?}", err);
                return false;
            }
        }
    }

    // preprint the prepended name or path (if user requested it)
    type MapPathIdToPrependName = HashMap<PathId, String>;

    // create zero-sized `pathid_to_prependname`
    // it will be iterated but may or may not need to store anything.
    let mut pathid_to_prependname: MapPathIdToPrependName = MapPathIdToPrependName::with_capacity(0);

    //
    // prepare per-thread data keyed by `FPath`
    // create necessary channels for each thread
    // create one thread per `PathId`, each thread named for the file basename
    //

    // pre-created mapping for calls to `select.recv` and `select.select`
    type MapIndexToPathId = HashMap<usize, PathId>;
    let color_default_: Color = color_default();
    // mapping PathId to colors for printing.
    let mut map_pathid_color = MapPathIdToColor::with_capacity(file_count);
    // mapping PathId to a `Summary` for `--summary`
    let mut map_pathid_summary = MapPathIdSummary::with_capacity(file_count);
    // Track if an error has been printed regarding a particular type of
    // logmessage printing problem.
    // Only want to print this particular error once, not hundreds of times.
    // These repetitive errors that occur from logmessage printing are usually
    // due to process pipe failures or terminal malfunctions/oddities.
    // e.g. `s4 /var/logs | head -n1` will causes an error message during printing.
    // This `has_print_err` prevents deluge of error messages.
    let mut has_print_err: bool = false;
    // "mapping" of PathId to select index, used in `recv_many_data`
    let mut index_select = MapIndexToPathId::with_capacity(file_count);

    // initialize processing channels/threads, one per `PathId`
    for pathid in map_pathid_path.keys() {
        map_pathid_color.insert(*pathid, color_rand());
    }
    // XXX: probably a no-op
    map_pathid_color.shrink_to_fit();
    let mut thread_count: usize = 0;
    let mut thread_err_count: usize = 0;

    // very unlikely to be `true` already but check anyway before starting threads
    exit_early_check!();

    for (pathid, path) in map_pathid_path.iter() {
        let (filetype, _) = match map_pathid_results.get(pathid) {
            Some(processpathresult) => match processpathresult {
                ProcessPathResult::FileValid(_path, filetype) => (filetype, _path),
                val => {
                    e_err!("unhandled ProcessPathResult {:?}", val);
                    continue;
                }
            },
            None => {
                panic!("bad pathid {} for path {:?}", pathid, path);
            }
        };
        let filetypeexecdata = match filetype {
            FileType::Etl { .. } => FileTypeExecData::Etl(etl_parser_used),
            FileType::Journal { .. } => FileTypeExecData::Journal(journal_output),
            _ => FileTypeExecData::None,
        };
        let thread_data: ThreadInitData = (
            path.clone().to_owned(),
            *pathid,
            *filetype,
            filetypeexecdata,
            blocksz,
            *filter_dt_after_opt,
            *filter_dt_before_opt,
            tz_offset,
        );
        let (chan_send_dt, chan_recv_dt): (ChanSendDatum, ChanRecvDatum) =
            crossbeam_channel::bounded(CHANNEL_CAPACITY);
        defo!("map_pathid_chanrecvdatum.insert({}, …);", pathid);
        MAP_PATHID_CHANRECVDATUM.write().unwrap().insert(*pathid, chan_recv_dt);
        let basename_: FPath = basename(path).replace(SUBPATH_SEP, SUBPATH_SEP_DISPLAY_STR);
        match thread::Builder::new()
            .name(basename_.clone())
            .spawn(move || exec_fileprocessor_thread(chan_send_dt, thread_data))
        {
            Ok(_joinhandle) => {
                thread_count += 1;
            }
            Err(err) => {
                thread_err_count += 1;
                e_err!("thread.name({:?}).spawn() pathid {} failed {:?}", basename_, pathid, err);
                MAP_PATHID_CHANRECVDATUM.write().unwrap().remove(pathid);
                map_pathid_color.remove(pathid);
                continue;
            }
        }
    }

    if MAP_PATHID_CHANRECVDATUM.read().unwrap().is_empty() {
        // No threads were created. This can happen if user passes only paths
        // that do not exist.
        if !cli_opt_summary {
            return false;
        }
        let named_temp_files_count: usize = count_temporary_files();
        debug_assert_eq!(
            named_temp_files_count,
            0,
            "no threads were created yet temporary files were created?"
        );
        print_summary(
            map_pathid_results,
            map_pathid_results_invalid,
            map_pathid_path,
            map_pathid_modified_time,
            map_pathid_file_processing_result,
            map_pathid_filetype,
            map_pathid_logmessagetype,
            map_pathid_color,
            map_pathid_summary,
            MapPathIdSummaryPrint::new(),
            color_choice,
            color_default_,
            paths_total,
            SetPathId::with_capacity(0),
            SummaryPrinted::default(),
            filter_dt_after_opt,
            filter_dt_before_opt,
            &LOCAL_NOW.with(|local_now| *local_now),
            &UTC_NOW.with(|utc_now| *utc_now),
            0,
            0,
            start_time,
            named_temp_files_count,
            thread_count,
            thread_err_count,
            ALLOCATOR_CHOSEN,
        );
        return false;
    }

    type RecvResult4 = std::result::Result<ChanDatum, crossbeam_channel::RecvError>;

    /// run `.recv` on many Receiver channels simultaneously using `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    #[inline(always)]
    fn recv_many_chan<'a>(
        pathid_chans: &'a MapPathIdChanRecvDatum,
        map_index_pathid: &'a mut MapIndexToPathId,
        filter_: &SetPathId,
    ) -> Option<(PathId, RecvResult4)> {
        def1n!("(pathid_chans {} entries, map_index_pathid {} entries, filter_ {} entries)",
               pathid_chans.len(), map_index_pathid.len(), filter_.len());
        // "mapping" of index to data; required for various `Select` and `SelectedOperation` procedures,
        // order should match index numeric value returned by `select`
        map_index_pathid.clear();
        // Build a list of operations
        let mut select: crossbeam_channel::Select = crossbeam_channel::Select::new();
        let mut index: usize = 0;
        for pathid_chan in pathid_chans.iter() {
            // if there is already a DateTime "on hand" for the given pathid then
            // skip receiving on the associated channel
            if filter_.contains(pathid_chan.0) {
                continue;
            }
            map_index_pathid.insert(index, *(pathid_chan.0));
            index += 1;
            def1o!("select.recv({:?});", pathid_chan.1);
            // load `select` with "operations" (receive channels)
            select.recv(pathid_chan.1);
        }
        if map_index_pathid.is_empty() {
            // no channels to receive from.
            // this can occur if a file processing thread exits improperly
            // or
            // the interrupt handler has been triggered and it has cleared
            // `MAP_PATHID_CHANRECVDATUM`
            de_wrn!(
                "Did not load any recv operations for select.select(). Overzealous filter? possible channels count {}, filter {:?}",
                pathid_chans.len(),
                filter_
            );
            return None;
        }
        def1o!("map_index_pathid: {:?}", map_index_pathid);
        // `select()` blocks until one of the loaded channel operations becomes ready
        let soper: crossbeam_channel::SelectedOperation = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index: usize = soper.index();
        def1o!("soper.index() returned {}", index);
        let pathid: &PathId = match map_index_pathid.get(&index) {
            Some(pathid_) => pathid_,
            None => {
                e_err!("BUG: failed to map_index_pathid.get({})", index);
                return None;
            }
        };
        def1o!("map_index_pathid.get({}) returned {}", index, pathid);
        let chan: &ChanRecvDatum = match pathid_chans.get(pathid) {
            Some(chan_) => chan_,
            None => {
                e_err!("BUG: failed to pathid_chans.get({})", pathid);
                return None;
            }
        };
        def1o!("soper.recv({:?})", chan);
        // Get the result of the `recv` done during `select`
        let result = soper.recv(chan);
        def1o!("soper.recv returned {:?}", result);
        def1x!("pathid {:?}", pathid);

        Some((*pathid, result))
    }

    //
    // preparation for the main coordination loop (e.g. the "game loop", or the "printing loop")
    //

    let mut first_print = true;
    let mut map_pathid_datum: MapPathIdDatum = MapPathIdDatum::new();
    // `set_pathid_datum` shadows `map_pathid_datum` for faster filters in `recv_many_chan`
    // precreated buffer
    let mut set_pathid = SetPathId::with_capacity(file_count);
    let mut map_pathid_sumpr = MapPathIdSummaryPrint::new();
    // crude debugging stats
    let mut chan_recv_ok: Count = 0;
    let mut chan_recv_err: Count = 0;
    // the `SummaryPrinted` tallying the entire process (tallies each received
    // LogMessage).
    let mut summaryprinted: SummaryPrinted = SummaryPrinted::default();

    // mapping PathId to colors for printing.
    let mut map_pathid_printer = MapPathIdToPrinterLogMessage::with_capacity(file_count);

    // count of not okay FileProcessing
    let mut _fileprocessing_not_okay: usize = 0;

    // track which paths had syslines
    let mut paths_printed_logmessages: SetPathId = SetPathId::with_capacity(file_count);

    //
    // the main processing loop (e.g. the "game loop")
    //
    // process the "receiving sysline" channels from the running file processing threads.
    // print the earliest available `Sysline`.
    //

    // channels that should be disconnected per "game loop" loop iteration
    let mut disconnect = Vec::<PathId>::with_capacity(file_count);
    // shortcut to the "sep"arator "b"ytes of the `log_message_separator`
    let sepb: &[u8] = log_message_separator.as_bytes();
    // shortcut to check if `sepb` should be printed
    let sepb_print: bool = !sepb.is_empty();
    // debug sanity check
    let mut _count_since_received_fileinfo: usize = 0;

    // buffer to assist printing FixedStruct; passed to `FixedStruct::as_bytes`
    let mut buffer_utmp: [u8; ENTRY_SZ_MAX * 2] = [0; ENTRY_SZ_MAX * 2];

    loop {
        disconnect.clear();

        exit_early_check!();

        #[cfg(debug_assertions)]
        {
            defo!("map_pathid_datum.len() {}", map_pathid_datum.len());
            for (pathid, _datum) in map_pathid_datum.iter() {
                let _path: &FPath = map_pathid_path
                    .get(pathid)
                    .unwrap();
                defo!("map_pathid_datum: thread {} {} has data", _path, pathid);
            }
            defo!("map_pathid_chanrecvdatum.len() {}", MAP_PATHID_CHANRECVDATUM.read().unwrap().len());
            for (pathid, _chanrdatum) in MAP_PATHID_CHANRECVDATUM.read().unwrap().iter() {
                let _path: &FPath = map_pathid_path
                    .get(pathid)
                    .unwrap();
                defo!(
                    "map_pathid_chanrecvdatum: thread {} {} channel messages {}",
                    _path,
                    pathid,
                    _chanrdatum.len()
                );
            }
        }

        if MAP_PATHID_CHANRECVDATUM.read().unwrap().len() != map_pathid_datum.len()
            || ! map_pathid_received_fileinfo.is_empty()
        {
            // IF…
            // `map_pathid_chanrecvdatum` does not have a `LogMessage` (and thus a `DatetimeL`)
            // for every channel (files being processed).
            // (every channel must return a `DatetimeL` to then compare *all* of them and see which
            // is earliest).
            // So call `recv_many_chan` to check if any channels have a new `ChanRecvDatum` to
            // provide.
            // …OR…
            // have not yet received a `ChanDatum::FileInfo` for every processing thread.
            // …SO…
            // call `recv_many_chan` and wait for any channels (file processing threads) to send
            // a `ChanDatum`.

            let pathid: PathId;
            let result: RecvResult4;
            // here is the wait on the channels (file processing threads)
            (pathid, result) = match recv_many_chan(
                &MAP_PATHID_CHANRECVDATUM.read().unwrap(),
                &mut index_select,
                &set_pathid,
            ) {
                Some(val) => val,
                None => {
                    // this occurs during an interrupt, after the interrupt handler has
                    // cleared `MAP_PATHID_CHANRECVDATUM`
                    de_wrn!("recv_many_chan returned None which is unexpected");
                    break;
                }
            };
            _count_since_received_fileinfo += 1;
            defo!("B_ recv_many_chan returned result for PathId {:?};", pathid);
            match result {
                Ok(chan_datum) => {
                    defo!("B0 crossbeam_channel::Found for PathId {:?}; map_pathid_file_processing_result has {:?}",
                          pathid, map_pathid_file_processing_result.len());
                    match chan_datum {
                        // only expect 1 FileInfo per processing thread
                        ChanDatum::FileInfo(dt_opt, file_processing_result) => {
                            defn!("B1 received ChanDatum::FileInfo for {:?}, modified_time, for {:?}", pathid, dt_opt);
                            defo!("B1 file_processing_result {:?}", file_processing_result);
                            defo!("B1 map_pathid_modified_time.insert({:?}, {:?})",
                                  pathid, dt_opt);
                            if map_pathid_modified_time.insert(pathid, dt_opt).is_some() {
                                debug_panic!("Already had stored DateTimeLOpt for PathID {:?}", pathid)
                            }
                            defo!("B1 received file_processing_result {:?} for {:?}", file_processing_result, pathid);
                            if !file_processing_result.is_ok() {
                                _fileprocessing_not_okay += 1;
                            }
                            defo!("map_pathid_file_processing_result.insert({:?}, {:?})",
                                  pathid, file_processing_result);
                            map_pathid_file_processing_result.insert(pathid, file_processing_result);
                            defo!("B1 map_pathid_received_fileinfo.insert({:?}, true)", pathid);
                            map_pathid_received_fileinfo.insert(pathid, true);
                            _count_since_received_fileinfo = 0;
                            defx!("B1");
                        }
                        // expect multiple NewMessage per processing thread
                        ChanDatum::NewMessage(log_message, is_last_message) => {
                            defn!("B2 received ChanDatum::NewMessage for PathID {:?}, is_last_message {:?}",
                                  pathid, is_last_message);
                            map_pathid_datum.insert(pathid, (log_message, is_last_message));
                            set_pathid.insert(pathid);
                            defx!("B2");
                        }
                        // only expect 1 FileSummary per processing thread
                        ChanDatum::FileSummary(summary_opt, file_processing_result) => {
                            defn!("B4 received ChanDatum::FileSummary for {:?}", pathid);
                            defo!("B4 file_processing_result {:?}", file_processing_result);
                            match summary_opt {
                                Some(summary) => summary_update(&pathid, summary, &mut map_pathid_summary),
                                None => debug_panic!("No summary received for FileSummary with PathID {}", pathid),
                            }
                            defo!("B4 will disconnect channel {:?}", pathid);
                            disconnect.push(pathid);
                            if !file_processing_result.is_ok() {
                                _fileprocessing_not_okay += 1;
                            }
                            // only save the `FileProcessingResult` if it is not `FileOk` or `FileStub`
                            // and if its an `FILEERR`
                            let r = map_pathid_file_processing_result.get(&pathid);
                            if ! file_processing_result.is_ok()
                               && ! file_processing_result.is_stub()
                               && (
                                r.unwrap_or(&FILEOK).is_ok()
                                || r.unwrap_or(&FILEOK).is_stub()
                               )
                            {
                                defo!("B4 map_pathid_file_processing_result.insert({:?}, {:?})",
                                      pathid, file_processing_result);
                                match map_pathid_file_processing_result.insert(pathid, file_processing_result) {
                                    Some(_old) => {
                                        defo!("B4 replaced old FileProcessingResult for {:?}; {:?}", pathid, _old);
                                    }
                                    None => {}
                                }
                            }
                            defx!("B4");
                        }
                    }
                    defo!("B5 done processing ChanDatum for PathId {:?}; map_pathid_file_processing_result has {:?}",
                          pathid, map_pathid_file_processing_result.len());
                    chan_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    defo!("B6 crossbeam_channel::RecvError, will disconnect channel for PathId {:?};", pathid);
                    // this channel was closed by the sender, it should be disconnected
                    disconnect.push(pathid);
                    chan_recv_err += 1;
                }
            }

            // debug sanity check for infinite loop
            #[cfg(any(debug_assertions, test))]
            {
                // how long has it been since a `ChanDatum::FileInfo` was received?
                // was it longer than maximum possible number of file processing threads?
                if !map_pathid_received_fileinfo.is_empty() && _count_since_received_fileinfo > file_count {
                    // very likely stuck in a loop, e.g. a file processing thread
                    // exited before sending a `ChanDatum::FileInfo`
                    panic!(
                        "count_since_recieved_fileinfo_or_summary {} > file_count {}",
                        _count_since_received_fileinfo, file_count
                    );
                }
            }

            defo!("C_ map_pathid_file_processing_result has {} entries",
                  map_pathid_file_processing_result.len());
            defo!("C_ map_pathid_received_fileinfo has {} entries",
                  map_pathid_received_fileinfo.len());
            defo!("C_ map_pathid_received_fileinfo.all()? {}",
                  map_pathid_received_fileinfo.iter().all(|(_, v)| v == &true));

            if !map_pathid_received_fileinfo.is_empty()
                && map_pathid_received_fileinfo.iter().all(|(_, v)| v == &true)
            {
                defo!("C1 map_pathid_received_fileinfo.all() are true");

                // debug sanity check
                #[cfg(any(debug_assertions, test))]
                {
                    for (k, _v) in map_pathid_received_fileinfo.iter()
                    {
                        assert!(map_pathid_path.contains_key(k),
                            "map_pathid_received_fileinfo PathID key {:?} not in map_pathid_path", k);
                    }
                }

                defo!("C1 map_pathid_file_processing_result has {:?} entries",
                      map_pathid_file_processing_result.len());
                for (pathid, file_processing_result) in map_pathid_file_processing_result.iter() {
                    defo!("C2 process file_processing_result {:?} for {:?}", file_processing_result, pathid);
                    match file_processing_result {
                        FileProcessingResultBlockZero::FileOk => {}
                        _ => {
                            let path = match map_pathid_path.get(pathid) {
                                Some(path_) => path_,
                                None => {
                                    debug_panic!("PathID {:?} not in map_pathid_path", pathid);
                                    continue;
                                }
                            };
                            // Here is the printing of the error messages prior to printing
                            // any processed log messages. These errors might also be printed
                            // later during the `--summary` printing.
                            match &file_processing_result {
                                FileProcessingResultBlockZero::FileErrTooSmall =>
                                    e_err!("file too small {:?}", path),
                                FileProcessingResultBlockZero::FileErrTooSmallS(s) =>
                                    e_err!("file too small {}", s),
                                FileProcessingResultBlockZero::FileErrNullBytes =>
                                    e_err!("file contains too many null bytes {:?}", path),
                                FileProcessingResultBlockZero::FileErrNoLinesFound =>
                                    e_err!("no lines found {:?}", path),
                                FileProcessingResultBlockZero::FileErrNoSyslinesFound =>
                                    e_err!("no syslines found {:?}", path),
                                FileProcessingResultBlockZero::FileErrNoValidFixedStruct =>
                                    e_err!("no valid fixed struct {:?}", path),
                                FileProcessingResultBlockZero::FileErrDecompress =>
                                    e_err!("could not decompress {:?}", path),
                                FileProcessingResultBlockZero::FileErrWrongType =>
                                    e_err!("bad path {:?}", path),
                                FileProcessingResultBlockZero::FileErrIo(err) =>
                                    e_err!("{} for {:?}", err, path),
                                FileProcessingResultBlockZero::FileErrIoPath(err) =>
                                    e_err!("{}", err),
                                FileProcessingResultBlockZero::FileErrChanSend =>
                                    panic!("Should not receive ChannelSend Error {}", path),
                                FileProcessingResultBlockZero::FileOk => {}
                                FileProcessingResultBlockZero::FileErrEmpty => {}
                                FileProcessingResultBlockZero::FileErrNoSyslinesInDtRange => {}
                                FileProcessingResultBlockZero::FileErrNoFixedStructInDtRange => {}
                                FileProcessingResultBlockZero::FileErrStub => {}
                            }
                        }
                    }
                }
                // all `ChanDatum::FileInfo` have been received so clear the tracking map
                defo!("C3 map_pathid_received_fileinfo.clear()");
                map_pathid_received_fileinfo.clear();
            }
        } else {
            // ELSE…
            // There is a DateTime available for *every* channel (one channel is one File Processing
            // thread). The datetimes can be compared among all remaining files. The sysline with
            // the earliest datetime is printed.
            // …AND…
            // Every file processing thread has sent a `ChanDatum::FileInfo`
            // …SO…
            // no need to call `recv_many_chan` to check for new `ChanDatum`s. Process the data
            // that has been collected from the processing threads (i.e. print a log message).

            #[cfg(debug_assertions)]
            {
                for (_i, (_k, _v)) in MAP_PATHID_CHANRECVDATUM
                    .read()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    deo!("{} A1 map_pathid_chanrecvdatum[{:?}] = {:?}", _i, _k, _v);
                }
                for (_i, (_k, _v)) in map_pathid_datum
                    .iter()
                    .enumerate()
                {
                    deo!("{} A1 map_pathid_datum[{:?}] = {:?}", _i, _k, _v);
                }
            }

            defo!("CX map_pathid_received_fileinfo.all() are false");

            if first_print {
                // One-time creation of `PrinterLogMessage` and optional prepended strings
                // (user options `--prepend-filename`, `--prepend-filepath`, `--prepend-width`).

                // First, get a set of all pathids with awaiting LogMessages, ignoring paths
                // for which no LogMessages were found.
                // No LogMessages will be printed for those paths that did not return a
                // LogMessage:
                // - do not include them in determining prepended width (CLI option `-w`).
                // - do not create a `PrinterLogMessage` for them.
                let mut pathid_with_logmessages: SetPathId = SetPathId::with_capacity(map_pathid_datum.len());
                for (pathid, _) in map_pathid_datum.iter() {
                    pathid_with_logmessages.insert(*pathid);
                }

                // Pre-create the prepended strings based on passed CLI options `-w` `-p` `-f`
                let mut prependname_width: usize = 0;
                if cli_opt_prepend_filename {
                    // pre-create prepended filename strings once (`-f`)
                    if cli_opt_prepend_file_align {
                        // determine prepended width (`-w`)
                        for pathid in pathid_with_logmessages.iter() {
                            let path = match map_pathid_path.get(pathid) {
                                Some(path_) => path_,
                                None => continue,
                            };
                            let bname: String = fpath_to_prependname(path);
                            prependname_width = std::cmp::max(
                                prependname_width,
                                unicode_width::UnicodeWidthStr::width(bname.as_str()),
                            );
                        }
                    }
                    pathid_to_prependname = MapPathIdToPrependName::with_capacity(pathid_with_logmessages.len());
                    for pathid in pathid_with_logmessages.iter() {
                        let path = match map_pathid_path.get(pathid) {
                            Some(path_) => path_,
                            None => continue,
                        };
                        let bname: String = fpath_to_prependname(path);
                        let prepend: String =
                            format!("{0:<1$}{2}", bname, prependname_width, cli_prepend_separator);
                        pathid_to_prependname.insert(*pathid, prepend);
                    }
                } else if cli_opt_prepend_filepath {
                    // pre-create prepended filepath strings once (`-p`)
                    if cli_opt_prepend_file_align {
                        // determine max width needed (`-w`)
                        for pathid in pathid_with_logmessages.iter() {
                            let path = match map_pathid_path.get(pathid) {
                                Some(path_) => fpath_to_prependpath(path_),
                                None => continue,
                            };
                            prependname_width = std::cmp::max(
                                prependname_width,
                                unicode_width::UnicodeWidthStr::width(path.as_str()),
                            );
                        }
                    }
                    pathid_to_prependname = MapPathIdToPrependName::with_capacity(pathid_with_logmessages.len());
                    for pathid in pathid_with_logmessages.iter() {
                        let path = match map_pathid_path.get(pathid) {
                            Some(path_) => fpath_to_prependpath(path_),
                            None => continue,
                        };
                        let prepend: String =
                            format!("{0:<1$}{2}", path, prependname_width, cli_prepend_separator);
                        pathid_to_prependname.insert(*pathid, prepend);
                    }
                } else {
                    pathid_to_prependname = MapPathIdToPrependName::with_capacity(0);
                }

                // Initialize the printers for logmessages, one per `PathId` that sent a Sysline.
                for pathid in pathid_with_logmessages.iter() {
                    let color_: &Color = map_pathid_color
                        .get(pathid)
                        .unwrap_or(&color_default_);
                    let prepend_file: Option<String> =
                        match cli_opt_prepend_filename || cli_opt_prepend_filepath {
                            true => Some(
                                pathid_to_prependname
                                    .get(pathid)
                                    .unwrap()
                                    .clone(),
                            ),
                            false => None,
                        };
                    let prepend_date_format: Option<String> = match cli_prepend_dt_format {
                        Some(ref s) => Some(s.to_owned() + cli_prepend_separator.as_str()),
                        None => None,
                    };
                    let printer: PrinterLogMessage = PrinterLogMessage::new(
                        color_choice,
                        *color_,
                        prepend_file,
                        prepend_date_format,
                        cli_opt_prepend_offset,
                    );
                    map_pathid_printer.insert(*pathid, printer);
                }

                first_print = false;
            } // if first_print

            // (path, channel data) for the log message with earliest datetime ("minimum" datetime)
            //
            // Here is part of the "sorting" of different log messages from different files
            // by datetime.
            // In case of tie datetime values, the tie-breaker will be order of `BTreeMap::iter_mut`
            // which iterates in order of key sort.
            // https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.iter_mut
            //
            // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
            //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
            //

            let pathid: &PathId;
            let log_message: &LogMessage;
            // Is last log message of the file?
            let is_last: IsLastLogMessage;
            // select the logmessage with earliest datetime
            (pathid, log_message, is_last) = match map_pathid_datum
                .iter_mut()
                .min_by(|x, y| x.1.0.dt().cmp(y.1.0.dt()))
            {
                Some(val) => (val.0, &val.1.0, val.1.1),
                None => {
                    de_err!("map_pathid_datum.iter_mut().min_by() returned None");
                    // XXX: not sure what else to do here
                    continue;
                }
            };

            // the designated printer for this pathid
            let printer: &mut PrinterLogMessage = map_pathid_printer
                .get_mut(pathid)
                .unwrap();
            // XXX: regarding printing errors about printing errors,
            //      In release builds, this code does not print errors about prior printing errors.
            //      It does not distinguish errors either. So errors due to a pipe closing
            //      are treated no different than something really bad that I would normally
            //      like to inform the user about.
            //      However, having cross-platform support to distinguish the type of error
            //      and whether the user should be be informed is a lot of tedious work.
            //      Additionally, printing errors about errors is likely going to itself fail
            //      because printing is probably permanently in a bad state.

            // TODO: [2024/03/19] cost-savings: It would be easy to add `if summary` around
            //       all summary updates within this function. Only update Summary data
            //       when needed.
            //       (it's a little more work to do this in the various `*Readers`.)

            // this `match` statement is where the actual printing takes place in this
            // main thread or "printing thread"
            match log_message {
                LogMessage::PyEvent(pyevent, pyevent_type) => {
                    defo!("A3 PyEvent printing PyDataEvent PathId: {:?}", pathid);
                    let mut printed: Count = 0;
                    let mut flushed: Count = 0;
                    match printer.print_pyevent(pyevent) {
                        Ok((printed_, flushed_)) => {
                            printed = printed_ as Count;
                            flushed = flushed_ as Count;
                        }
                        Err(_err) => {
                            // Only print a printing error once and only for debug builds.
                            if !has_print_err {
                                has_print_err = true;
                                // BUG: Issue #3 colorization settings in the context of a pipe
                                de_err!("failed to print {}", _err);
                            }
                            defo!("print error, will disconnect channel {:?}", pathid);
                            disconnect.push(*pathid);
                        }
                    }
                    if sepb_print {
                        write_stdout(sepb);
                        if cli_opt_summary {
                            summaryprinted.bytes += sepb.len() as Count;
                            summaryprinted.flushed += 1;
                        }
                    }
                    if cli_opt_summary {
                        paths_printed_logmessages.insert(*pathid);
                        // update the per processing file `SummaryPrinted`
                        SummaryPrinted::summaryprint_map_update_pyevent(
                            pyevent,
                            pathid,
                            *pyevent_type,
                            &mut map_pathid_sumpr,
                            printed,
                            flushed,
                        );
                        // update the single total program `SummaryPrinted`
                        summaryprinted.summaryprint_update_pyevent(
                            pyevent, *pyevent_type, printed, flushed
                        );
                    }
                }
                LogMessage::Evtx(evtx) => {
                    defo!("A3 Evtx printing PathId: {:?}", pathid);
                    let mut printed: Count = 0;
                    let mut flushed: Count = 0;
                    match printer.print_evtx(evtx) {
                        Ok((printed_, flushed_)) => {
                            printed = printed_ as Count;
                            flushed = flushed_ as Count;
                        }
                        Err(_err) => {
                            // Only print a printing error once and only for debug builds.
                            if !has_print_err {
                                has_print_err = true;
                                // BUG: Issue #3 colorization settings in the context of a pipe
                                de_err!("failed to print {}", _err);
                            }
                            defo!("print error, will disconnect channel {:?}", pathid);
                            disconnect.push(*pathid);
                        }
                    }
                    if sepb_print {
                        write_stdout(sepb);
                        if cli_opt_summary {
                            summaryprinted.bytes += sepb.len() as Count;
                            summaryprinted.flushed += 1;
                        }
                    }
                    if cli_opt_summary {
                        paths_printed_logmessages.insert(*pathid);
                        // update the per processing file `SummaryPrinted`
                        SummaryPrinted::summaryprint_map_update_evtx(
                            evtx,
                            pathid,
                            &mut map_pathid_sumpr,
                            printed,
                            flushed,
                        );
                        // update the single total program `SummaryPrinted`
                        summaryprinted.summaryprint_update_evtx(evtx, printed, flushed);
                    }
                }
                LogMessage::FixedStruct(entry) => {
                    defo!("A3 FixedStruct printing PathId: {:?}", pathid);
                    let mut printed: Count = 0;
                    let mut flushed: Count = 0;
                    match printer.print_fixedstruct(entry, &mut buffer_utmp) {
                        Ok((printed_, flushed_)) => {
                            printed = printed_ as Count;
                            flushed = flushed_ as Count;
                        }
                        Err(_err) => {
                            // Only print a printing error once and only for debug builds.
                            if !has_print_err {
                                has_print_err = true;
                                // BUG: Issue #3 colorization settings in the context of a pipe
                                de_err!("failed to print {}", _err);
                            }
                            defo!("print error, will disconnect channel {:?}", pathid);
                            disconnect.push(*pathid);
                        }
                    }
                    if sepb_print {
                        write_stdout(sepb);
                        if cli_opt_summary {
                            summaryprinted.bytes += sepb.len() as Count;
                            summaryprinted.flushed += 1;
                        }
                    }
                    if cli_opt_summary {
                        paths_printed_logmessages.insert(*pathid);
                        // update the per processing file `SummaryPrinted`
                        SummaryPrinted::summaryprint_map_update_fixedstruct(
                            entry,
                            pathid,
                            &mut map_pathid_sumpr,
                            printed,
                            flushed,
                        );
                        // update the single total program `SummaryPrinted`
                        summaryprinted.summaryprint_update_fixedstruct(entry, printed, flushed);
                    }
                }
                LogMessage::Journal(journalentry) => {
                    defo!("A3 Journal printing JournalEntry PathId: {:?}", pathid);
                    let mut printed: Count = 0;
                    let mut flushed: Count = 0;
                    match printer.print_journalentry(journalentry) {
                        Ok((printed_, flushed_)) => {
                            printed = printed_ as Count;
                            flushed = flushed_ as Count;
                        }
                        Err(_err) => {
                            // Only print a printing error once and only for debug builds.
                            if !has_print_err {
                                has_print_err = true;
                                // BUG: Issue #3 colorization settings in the context of a pipe
                                de_err!("failed to print {}", _err);
                            }
                            defo!("print error, will disconnect channel {:?}", pathid);
                            disconnect.push(*pathid);
                        }
                    }
                    if sepb_print {
                        write_stdout(sepb);
                        if cli_opt_summary {
                            summaryprinted.bytes += sepb.len() as Count;
                            summaryprinted.flushed += 1;
                        }
                    }
                    if cli_opt_summary {
                        paths_printed_logmessages.insert(*pathid);
                        // update the per processing file `SummaryPrinted`
                        SummaryPrinted::summaryprint_map_update_journalentry(
                            journalentry,
                            pathid,
                            &mut map_pathid_sumpr,
                            printed,
                            flushed,
                        );
                        // update the single total program `SummaryPrinted`
                        summaryprinted.summaryprint_update_journalentry(journalentry, printed, flushed);
                    }
                }
                LogMessage::Sysline(syslinep) => {
                    defo!(
                        "A3 Sysline printing SyslineP @[{}, {}] PathId: {:?}",
                        syslinep.fileoffset_begin(),
                        syslinep.fileoffset_end(),
                        pathid,
                    );
                    let mut printed: Count = 0;
                    let mut flushed: Count = 0;
                    match printer.print_sysline(syslinep) {
                        Ok((printed_, flushed_)) => {
                            printed = printed_ as Count;
                            flushed = flushed_ as Count;
                        }
                        Err(_err) => {
                            // Only print a printing error once and only for debug builds.
                            if !has_print_err {
                                has_print_err = true;
                                // BUG: Issue #3 colorization settings in the context of a pipe
                                de_err!("failed to print {}", _err);
                            }
                            defo!("print error, will disconnect channel {:?}", pathid);
                            disconnect.push(*pathid);
                        }
                    }
                    if sepb_print {
                        write_stdout(sepb);
                        if cli_opt_summary {
                            summaryprinted.bytes += sepb.len() as Count;
                            summaryprinted.flushed += 1;
                        }
                    }
                    // If a file's last char is not a '\n' then the next printed sysline
                    // (from a different file) will print on the same terminal line.
                    // While this is accurate byte-wise, it's difficult to read and unexpected, and
                    // makes scripting line-oriented scripting more difficult. This is especially
                    // visually jarring when prepended data is present (`-l`, `-p`, etc.).
                    // So in case of no ending '\n', print an extra '\n' to improve human readability
                    // and scriptability.
                    if is_last && !(*syslinep).ends_with_newline() {
                        write_stdout(&NLu8a);
                        if cli_opt_summary {
                            summaryprinted.bytes += NLu8a.len() as Count;
                            summaryprinted.flushed += 1;
                        }
                    }
                    if cli_opt_summary {
                        paths_printed_logmessages.insert(*pathid);
                        // update the per processing file `SummaryPrinted`
                        SummaryPrinted::summaryprint_map_update_sysline(
                            syslinep,
                            pathid,
                            &mut map_pathid_sumpr,
                            printed,
                            flushed,
                        );
                        // update the single total program `SummaryPrinted`
                        summaryprinted.summaryprint_update_sysline(syslinep, printed, flushed);
                    }
                }
            }
            // XXX: create a copy of the borrowed key `pathid`, this avoids rustc error:
            //         cannot borrow `map_pathid_datum` as mutable more than once at a time
            let pathid_: PathId = *pathid;
            map_pathid_datum.remove(&pathid_);
            set_pathid.remove(&pathid_);
        } // else (datetime available)

        // remove channels (and keys) that are marked disconnected
        let mut map_pathid_chanrecvdatum = MAP_PATHID_CHANRECVDATUM.write().unwrap();
        for pathid in disconnect.iter() {
            defo!("D disconnect channel: map_pathid_chanrecvdatum.remove({:?});", pathid);
            map_pathid_chanrecvdatum.remove(pathid);
            defo!("D pathid_to_prependname.remove({:?});", pathid);
            pathid_to_prependname.remove(pathid);
            defo!("D map_pathid_printer.remove({:?});", pathid);
            map_pathid_printer.remove(pathid);
        }
        // are there any channels to receive from?
        if map_pathid_chanrecvdatum.is_empty() {
            defo!("E map_pathid_chanrecvdatum.is_empty(); no more channels to receive from!");
            // all channels are closed, break from main processing loop
            break;
        }
        defo!("F map_pathid_chanrecvdatum: {:?}", map_pathid_chanrecvdatum);
        defo!("F map_pathid_datum: {:?}", map_pathid_datum);
        defo!("F set_pathid: {:?}", set_pathid);
    } // end loop

    // Getting here means main program processing has completed.
    // Now to print the `--summary` (if it was requested).

    // quick count of `Summary` attached Errors
    let mut error_count: usize = 0;
    for (_pathid, summary) in map_pathid_summary.iter() {
        if summary.error.is_some() {
            error_count += 1;
        }
    }
    defo!("G summary error_count: {:?}", error_count);

    // check again before writing the final summary
    exit_early_check!();

    // temporary files should have been removed when the processing thread
    // dropped the `NamedTempFile` object.
    // But in case something went wrong, check them all again.
    if !remove_temporary_files() {
        de_err!("there was an error removing temporary files");
        error_count += 1;
    }

    if cli_opt_summary {
        defo!("H cli_opt_summary");
        // some errors may occur later in processing, e.g. File Permissions errors,
        // so update `map_pathid_results` and `map_pathid_results_invalid`
        for (pathid, summary) in map_pathid_summary.iter() {
            if summary.error.is_some() {
                if !summary.readerdata.is_dummy()
                    && summary.has_blockreader()
                    && summary.blockreader().unwrap().blockreader_blocks == 0
                    && !map_pathid_results_invalid.contains_key(pathid)
                    && map_pathid_results.contains_key(pathid)
                {
                    let result = map_pathid_results
                        .remove(pathid)
                        .unwrap();
                    map_pathid_results_invalid.insert(*pathid, result);
                }
            }
        }
        let named_temp_files_count: usize = count_temporary_files();
        // print the `--summary` of the entire process
        // consumes the various maps
        print_summary(
            map_pathid_results,
            map_pathid_results_invalid,
            map_pathid_path,
            map_pathid_modified_time,
            map_pathid_file_processing_result,
            map_pathid_filetype,
            map_pathid_logmessagetype,
            map_pathid_color,
            map_pathid_summary,
            map_pathid_sumpr,
            color_choice,
            color_default(),
            paths_total,
            paths_printed_logmessages,
            summaryprinted,
            filter_dt_after_opt,
            filter_dt_before_opt,
            &LOCAL_NOW.with(|local_now| *local_now),
            &UTC_NOW.with(|utc_now| *utc_now),
            chan_recv_ok,
            chan_recv_err,
            start_time,
            named_temp_files_count,
            thread_count,
            thread_err_count,
            ALLOCATOR_CHOSEN,
        );
    }
    defo!("I chan_recv_ok {:?} _count_recv_di {:?}", chan_recv_ok, chan_recv_err);

    // TODO: Issue #5 return code confusion
    //       the rationale for returning `false` (and then the process return code 1)
    //       is clunky, and could use a little refactoring.
    let mut ret: bool = true;
    if chan_recv_err > 0 {
        defo!("K chan_recv_err {}; return false", chan_recv_err);
        ret = false;
    }
    if error_count > 0 {
        defo!("L error_count {}; return false", error_count);
        ret = false;
    }
    defx!("return {:?}", ret);

    ret
}

// TODO: move this section to a s4_tests.rs file
#[cfg(test)]
mod tests {
    use s4lib::data::datetime::{
        ymdhmsl,
        ymdhmsm,
        DateTimeL,
        DateTimePattern_string,
    };

    use super::*;

    mod s4 {
        use ::test_case::test_case;
        use super::*;

    const FIXEDOFFSET0: FixedOffset = FixedOffset::east_opt(0).unwrap();

    #[test_case("500", true)]
    #[test_case("0x2", true)]
    #[test_case("0x4", true)]
    #[test_case("0xFFFFFF", true)]
    #[test_case("BAD_BLOCKSZ_VALUE", false)]
    #[test_case("", false)]
    fn test_cli_parse_blocksz(
        blocksz_str: &str,
        is_ok: bool,
    ) {
        match is_ok {
            true => assert!(cli_parse_blocksz(blocksz_str).is_ok()),
            false => assert!(!cli_parse_blocksz(blocksz_str).is_ok()),
        }
    }

    #[test_case(
        "0b10101010101",
        Some(0b10101010101)
    )]
    #[test_case("0o44", Some(0o44))]
    #[test_case("00500", Some(500))]
    #[test_case("500", Some(500))]
    #[test_case("0x4", Some(0x4))]
    #[test_case("0xFFFFFF", Some(0xFFFFFF))]
    #[test_case("BAD_BLOCKSZ_VALUE", None)]
    #[test_case("", None)]
    fn test_cli_process_blocksz(
        blocksz_str: &str,
        expect_: Option<BlockSz>,
    ) {
        match expect_ {
            Some(val_exp) => {
                let val_ret = cli_process_blocksz(&String::from(blocksz_str)).unwrap();
                assert_eq!(val_ret, val_exp);
            }
            None => {
                let ret = cli_process_blocksz(&String::from(blocksz_str));
                assert!(
                    ret.is_err(),
                    "Expected an Error for cli_process_blocksz({:?}), instead got {:?}",
                    blocksz_str,
                    ret
                );
            }
        }
    }

    #[test_case("+00", FIXEDOFFSET0; "+00 east(0)")]
    #[test_case("+0000", FIXEDOFFSET0; "+0000 east(0)")]
    #[test_case("+00:00", FIXEDOFFSET0; "+00:00 east(0)")]
    #[test_case("+00:01", FixedOffset::east_opt(60).unwrap(); "+00:01 east(60)")]
    #[test_case("+01:00", FixedOffset::east_opt(3600).unwrap(); "+01:00 east(3600) A")]
    #[test_case("-01:00", FixedOffset::east_opt(-3600).unwrap(); "-01:00 east(-3600) B")]
    #[test_case("+02:00", FixedOffset::east_opt(7200).unwrap(); "+02:00 east(7200)")]
    #[test_case("+02:30", FixedOffset::east_opt(9000).unwrap(); "+02:30 east(9000)")]
    #[test_case("+02:35", FixedOffset::east_opt(9300).unwrap(); "+02:30 east(9300)")]
    #[test_case("+23:00", FixedOffset::east_opt(82800).unwrap(); "+23:00 east(82800)")]
    #[test_case("gmt", FIXEDOFFSET0; "GMT (0)")]
    #[test_case("UTC", FIXEDOFFSET0; "UTC east(0)")]
    #[test_case("Z", FIXEDOFFSET0; "Z (0)")]
    #[test_case("vlat", FixedOffset::east_opt(36000).unwrap(); "vlat east(36000)")]
    #[test_case("IDLW", FixedOffset::east_opt(-43200).unwrap(); "IDLW east(-43200)")]
    fn test_cli_process_tz_offset(
        in_: &str,
        out_fo: FixedOffset,
    ) {
        let input: String = String::from(in_);
        let result = cli_process_tz_offset(&input);
        match result {
            Ok(fo) => {
                assert_eq!(out_fo, fo, "cli_process_tz_offset returned FixedOffset {:?}, expected {:?}", fo, out_fo);
            }
            Err(err) => {
                panic!("Error {}", err);
            }
        }
    }

    #[test_case("")]
    #[test_case("abc")]
    #[test_case(CLI_OPT_PREPEND_FMT)]
    #[test_case("%Y%Y%Y%Y%Y%Y%Y%%%%")]
    fn test_cli_parser_prepend_dt_format(input: &str) {
        assert!(cli_parser_prepend_dt_format(input).is_ok());
    }

    #[test_case(
        Some(String::from("2000-01-02T03:04:05")), FIXEDOFFSET0,
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "2000-01-02T03:04:05"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678")), FIXEDOFFSET0,
        Some(ymdhmsl(&FIXEDOFFSET0, 2000, 1, 2, 3, 4, 5, 678));
        "2000-01-02T03:04:05.678"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678901")), FIXEDOFFSET0,
        Some(ymdhmsm(&FIXEDOFFSET0, 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02T03:04:05.678901"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678901-01")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-3600).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02T03:04:05.678901-01"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678901-0100")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-3600).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02T03:04:05.678901-0100"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678901-01:00")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-3600).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02T03:04:05.678901-01:00"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678901 -01:00")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-3600).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02T03:04:05.678901 -01:00_"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05.678901 AZOT")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-3600).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02T03:04:05.678901 AZOT"
    )]
    #[test_case(
        Some(String::from("+946782245")), FIXEDOFFSET0,
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "+946782245"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05 -0100")), FIXEDOFFSET0,
        Some(FixedOffset::east_opt(-3600).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "2000-01-02T03:04:05 -0100"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05PDT")), FIXEDOFFSET0,
        Some(FixedOffset::east_opt(-25200).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "2000-01-02T03:04:05PDT"
    )]
    #[test_case(
        // bad timezone
        Some(String::from("2000-01-02T03:04:05FOOO")), FIXEDOFFSET0, None;
        "2000-01-02T03:04:05FOOO"
    )]
    #[test_case(
        Some(String::from("2000/01/02 03:04:05")), FIXEDOFFSET0,
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "2000-01-02T03:04:05 (no TZ)"
    )]
    #[test_case(
        Some(String::from("2000/01/02 03:04:05.678")), FIXEDOFFSET0,
        Some(ymdhmsl(&FIXEDOFFSET0, 2000, 1, 2, 3, 4, 5, 678));
        "2000-01-02 03:04:05.678"
    )]
    #[test_case(
        Some(String::from("2000/01/02 03:04:05.678901")), FIXEDOFFSET0,
        Some(ymdhmsm(&FIXEDOFFSET0, 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02 03:04:05.678901"
    )]
    #[test_case(
        Some(String::from("2000/01/02 03:04:05.678901 -01")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-3600).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02 03:04:05.678901 -01"
    )]
    #[test_case(
        Some(String::from("2000/01/02 03:04:05.678901 -01:30")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-5400).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02 03:04:05.678901 -01:30"
    )]
    #[test_case(
        Some(String::from("2000/01/02 03:04:05.678901 -0130")), FIXEDOFFSET0,
        Some(ymdhmsm(&FixedOffset::east_opt(-5400).unwrap(), 2000, 1, 2, 3, 4, 5, 678901));
        "2000-01-02 03:04:05.678901 -0130"
    )]
    fn test_process_dt(
        dts: Option<String>,
        tz_offset: FixedOffset,
        expect: DateTimeLOpt,
    ) {
        eprintln!(
            "test_process_dt: process_dt({:?}, {:?}, &None, UTC_NOW: {:?})",
            dts, tz_offset, UTC_NOW.with(|utc_now| *utc_now),
        );
        let dt = process_dt(&dts, &tz_offset, &None, &UTC_NOW.with(|utc_now| *utc_now));
        eprintln!("test_process_dt: process_dt returned {:?}", dt);
        assert_eq!(dt, expect);
    }

    #[test_case(
        Some(String::from("@+1s")),
        FIXEDOFFSET0,
        FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 6).unwrap());
        "2000-01-02T03:04:05 add 1s"
    )]
    #[test_case(
        Some(String::from("@-1s")),
        FIXEDOFFSET0,
        FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 4).unwrap());
        "2000-01-02T03:04:04 add 1s"
    )]
    #[test_case(
        Some(String::from("@+4h1d")),
        FIXEDOFFSET0,
        FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 3, 7, 4, 5).unwrap());
        "2000-01-02T03:04:05 sub 4h1d"
    )]
    #[test_case(
        Some(String::from("@+4h1d")),
        FixedOffset::east_opt(-3630).unwrap(),
        FixedOffset::east_opt(-3630).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
        Some(FixedOffset::east_opt(-3630).unwrap().with_ymd_and_hms(2000, 1, 3, 7, 4, 5).unwrap());
        "2000-01-02T03:04:05 sub 4h1d offset -3600"
    )]
    fn test_process_dt_other(
        dts: Option<String>,
        tz_offset: FixedOffset,
        dt_other: DateTimeL,
        expect: DateTimeLOpt,
    ) {
        let dt = process_dt(&dts, &tz_offset, &Some(dt_other), &UTC_NOW.with(|utc_now| *utc_now));
        assert_eq!(dt, expect);
    }

    #[test]
    fn test_cli_filter_patterns() {
        #[allow(non_snake_case)]
        for (
            pattern_,
            _has_year,
            has_tz,
            has_tzZ,
            has_time,
            has_time_sec,
            has_date,
        ) in CLI_FILTER_PATTERNS.iter() {
            let pattern: DateTimePattern_string = DateTimePattern_string::from(*pattern_);

            let has_tz_actual = pattern.contains("%z")
                || pattern.contains("%:z")
                || pattern.contains("%#z")
                || pattern.contains("%Z");
            assert!(
                has_tz == &has_tz_actual,
                "has_tz: {} != {} actual in pattern {:?}",
                has_tz, has_tz_actual, pattern
            );

            let has_tzZ_actual = pattern.contains("%Z");
            assert!(
                has_tzZ == &has_tzZ_actual,
                "has_tzZ: {} != {} actual in {:?}",
                has_tzZ, has_tzZ_actual, pattern
            );
            if has_tzZ_actual {
                assert!(
                    pattern.ends_with("%z")
                    || pattern.ends_with("%:z")
                    || pattern.ends_with("%#z")
                    || pattern.ends_with("%Z"),
                    "has_tz pattern must end with timezone strftime specifier in {:?}",
                    pattern
                );
            }

            assert!(!(*has_tzZ && !has_tz), "has_tzZ && !has_tz");

            let has_time_actual = pattern.contains("%H")
                && pattern.contains("%M")
                || pattern.contains("+%s");
            assert!(
                has_time == &has_time_actual,
                "has_time: {} != {} actual in {:?}",
                has_time,
                has_time_actual,
                pattern
            );

            let has_time_sec_actual = pattern.contains("%S");
            assert!(
                has_time_sec == &has_time_sec_actual,
                "has_time_sec: {} != {} actual in {:?}",
                has_time_sec,
                has_time_sec_actual,
                pattern
            );

            let has_date_actual = pattern.contains("%Y")
                && pattern.contains("%m")
                && pattern.contains("%d")
                || pattern.contains("+%s");
            assert!(
                has_date == &has_date_actual,
                "has_date: {} != {} actual in {:?}",
                has_date,
                has_date_actual,
                pattern
            );
        }
    }

    const NOW: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Now;
    const OTHER: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Other;

    #[test_case(String::from(""), None)]
    #[test_case(String::from("1s"), None; "1s")]
    #[test_case(String::from("@1s"), None; "at_1s")]
    #[test_case(String::from("-0s"), Some((Duration::try_seconds(0).unwrap(), NOW)))]
    #[test_case(String::from("@+0s"), Some((Duration::try_seconds(0).unwrap(), OTHER)))]
    #[test_case(String::from("-1s"), Some((Duration::try_seconds(-1).unwrap(), NOW)); "minus_1s")]
    #[test_case(String::from("+1s"), Some((Duration::try_seconds(1).unwrap(), NOW)); "plus_1s")]
    #[test_case(String::from("@-1s"), Some((Duration::try_seconds(-1).unwrap(), OTHER)); "at_minus_1s")]
    #[test_case(String::from("@+1s"), Some((Duration::try_seconds(1).unwrap(), OTHER)); "at_plus_1s")]
    #[test_case(String::from("@+9876s"), Some((Duration::try_seconds(9876).unwrap(), OTHER)); "other_plus_9876")]
    #[test_case(String::from("@-9876s"), Some((Duration::try_seconds(-9876).unwrap(), OTHER)); "other_minus_9876")]
    #[test_case(String::from("-9876s"), Some((Duration::try_seconds(-9876).unwrap(), NOW)); "now_minus_9876")]
    #[test_case(String::from("-3h"), Some((Duration::try_hours(-3).unwrap(), NOW)))]
    #[test_case(String::from("-4d"), Some((Duration::try_days(-4).unwrap(), NOW)))]
    #[test_case(String::from("-5w"), Some((Duration::try_weeks(-5).unwrap(), NOW)))]
    #[test_case(String::from("@+5w"), Some((Duration::try_weeks(5).unwrap(), OTHER)))]
    #[test_case(String::from("-2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap(), NOW)); "minus_2m1s")]
    #[test_case(String::from("-2d1h"), Some((Duration::try_hours(-1).unwrap() + Duration::try_days(-2).unwrap(), NOW)); "minus_2d1h")]
    #[test_case(String::from("+2d1h"), Some((Duration::try_hours(1).unwrap() + Duration::try_days(2).unwrap(), NOW)); "plus_2d1h")]
    #[test_case(String::from("@+2d1h"), Some((Duration::try_hours(1).unwrap() + Duration::try_days(2).unwrap(), OTHER)); "at_plus_2d1h")]
    // "reverse" order should not matter
    #[test_case(String::from("-1h2d"), Some((Duration::try_hours(-1).unwrap() + Duration::try_days(-2).unwrap(), NOW)); "minus_1h2d")]
    #[test_case(String::from("-4w3d2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
    // "mixed" order should not matter
    #[test_case(String::from("-3d4w1s2m"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
    // repeat values; only last is used
    #[test_case(String::from("-6w5w4w"), Some((Duration::try_weeks(-4).unwrap(), NOW)))]
    // repeat values; only last is used
    #[test_case(String::from("-5w4w3d2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
    // repeat values; only last is used
    #[test_case(String::from("-6w5w4w3d2m1s"), Some((Duration::try_seconds(-1).unwrap() + Duration::try_minutes(-2).unwrap() + Duration::try_days(-3).unwrap() + Duration::try_weeks(-4).unwrap(), NOW)))]
    fn test_string_wdhms_to_duration(
        input: String,
        expect: Option<(Duration, DUR_OFFSET_TYPE)>,
    ) {
        let actual = string_wdhms_to_duration(&input);
        assert_eq!(actual, expect);
    }

    #[test_case("", Some(""))]
    #[test_case("a", Some("a"))]
    #[test_case("abc", Some("abc"))]
    #[test_case(r"\t", Some("\t"))]
    #[test_case(r"\v", Some("\u{0B}"))]
    #[test_case(r"\e", Some("\u{1B}"))]
    #[test_case(r"\0", Some("\u{00}"))]
    #[test_case(r"-\0-", Some("-\u{00}-"); "dash null dash")]
    #[test_case(r":\t|", Some(":\t|"); "colon tab vertical pipe")]
    #[test_case(r":\t\\|", Some(":\t\\|"); "colon tab escape vertical pipe")]
    #[test_case(r"\\\t", Some("\\\t"); "escape tab")]
    #[test_case(r"\\t", Some("\\t"); "escape t")]
    #[test_case(r"\", None)]
    #[test_case(r"\X", None)]
    fn test_unescape_str(
        input: &str,
        expect: Option<&str>,
    ) {
        let result = unescape::unescape_str(input);
        match (result, expect) {
            (Ok(actual_s), Some(expect_s)) => {
                assert_eq!(actual_s, expect_s, "\nExpected {:?}\nActual   {:?}\n", expect_s, actual_s);
            }
            (Ok(actual_s), None) => {
                panic!("Expected Error, got {:?}", actual_s);
            }
            (Err(err), Some(_)) => {
                panic!("Got Error {:?}", err);
            }
            (Err(_), None) => {}
        }
    }

    }
}
