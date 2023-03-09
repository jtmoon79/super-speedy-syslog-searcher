// src/bin/bin.rs
//
// ‥ … ≤ ≥ ≠ ≟ ×

//! Driver program _s4_.
//!
//! Processes user-passed command-line arguments.
//! Then processes paths passed; directories are enumerated for parseable files,
//! archive files (`.tar`) are enumerated for file entries, other
//! paths tested for suitability (readable? is it a file? etc.).
//!
//! For each parseable file found, a file processing thread is created.
//! Each file processing thread advances through the stages of processing
//! using a [`SyslogProcessor`] instance, or using a [`UtmpxReader`].
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
//! A `UtmpxReader` follow the same threaded message-passing pattern but
//! does not have processing stages.
//!
//! If passed CLI option `--summary`, the main processing thread
//! prints a [`Summary`] about each file processed, and one total
//! [`SummaryPrinted`].
//!
//! [`Stage3StreamSyslines`]: s4lib::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
//! [`DateTimeL`]: s4lib::data::datetime::DateTimeL
//! [`Sysline`]: s4lib::data::sysline::Sysline
//! [sending channel]: self::ChanSendDatum
//! [`SyslogProcessor`]: s4lib::readers::syslogprocessor::SyslogProcessor
//! [`UtmpxReader`]: s4lib::readers::utmpxreader::UtmpxReader
//! [`Summary`]: s4lib::readers::summary::Summary
//! [`SummaryPrinted`]: self::SummaryPrinted

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::io::BufRead; // for stdin::lock().lines()
use std::io::ErrorKind;
use std::process::ExitCode;
use std::str;
use std::thread;

extern crate chrono;
use chrono::{DateTime, Datelike, Duration, FixedOffset, Local, TimeZone, Timelike};

extern crate clap;
use clap::{Parser, ValueEnum};

extern crate const_format;
use const_format::concatcp;

// TODO: [2023/01] use std::sync::mpsc instead of crossbeam_channel when MSRV is >= 1.67.0
//       see https://github.com/rust-lang/rust/pull/93563/ and https://releases.rs/docs/1.67.0/
extern crate crossbeam_channel;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate regex;
use regex::Regex;

extern crate si_trace_print;
use si_trace_print::{def1n, def1o, def1x, defn, defo, defx, defñ, deo, stack::stack_offset_set};

extern crate unicode_width;

// `s4lib` is the local compiled `[lib]` of super_speedy_syslog_searcher
extern crate s4lib;

use s4lib::common::{
    Count,
    FPath,
    FPaths,
    FileOffset,
    FileType,
    LogMessageType,
    NLu8a,
    filetype_to_logmessagetype,
};

use s4lib::data::datetime::{
    datetime_parse_from_str, datetime_parse_from_str_w_tz, DateTimeLOpt, DateTimeParseInstr,
    DateTimePattern_str, MAP_TZZ_TO_TZz, Utc, DATETIME_PARSE_DATAS,
};

#[allow(unused_imports)]
use s4lib::debug::printers::{de_err, de_wrn, e_err, e_wrn};

use s4lib::printer::printers::{
    color_rand,
    print_colored_stderr,
    write_stderr,
    write_stdout,
    // termcolor imports
    Color,
    ColorChoice,
    PrinterLogMessage,
    //
    COLOR_DEFAULT,
    COLOR_ERROR,
};

use s4lib::data::utmpx::{UTMPX_SZ, Utmpx};

use s4lib::data::sysline::SyslineP;

use s4lib::readers::blockreader::{
    BlockSz,
    BLOCKSZ_DEF,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
    SummaryBlockReader,
};

use s4lib::readers::filepreprocessor::{process_path, ProcessPathResult, ProcessPathResults};

use s4lib::readers::helpers::basename;

use s4lib::readers::linereader::SummaryLineReader;

use s4lib::readers::summary::{
    Summary,
    SummaryOpt,
    SummaryReaderData,
};

use s4lib::readers::syslinereader::{ResultS3SyslineFind, SummarySyslineReader};

use s4lib::readers::syslogprocessor::{FileProcessingResultBlockZero, SyslogProcessor};

use s4lib::readers::utmpxreader::{ResultS3UtmpxFind, UtmpxReader, SummaryUtmpxReader};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// user-passed signifier that file paths were passed on STDIN
const PATHS_ON_STDIN: &str = "-";

lazy_static! {
    /// for user-passed strings of a duration that will be offset from the
    /// current datetime.
    static ref UTC_NOW: DateTime<Utc> = Utc::now();
    static ref LOCAL_NOW: DateTime<Local> = DateTime::from(UTC_NOW.clone());
    static ref FIXEDOFFSET0: FixedOffset = FixedOffset::east_opt(0).unwrap();
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
/// (DateTimePattern_str, has year, has timezone, has time)
///
/// [`DateTimeParseInstr`]: s4lib::data::datetime::DateTimeParseInstr
/// [`datetime_parse_from_str`]: s4lib::data::datetime#fn.datetime_parse_from_str
type CLI_DT_Filter_Pattern<'b> = (&'b DateTimePattern_str, bool, bool, bool);

// TODO: reject ambiguous timezone names.
//       best way to do this is to modify `DTPD!` defined in `datetime.rs` to
//       have a flag, "is it acceptable for CLI?". Then gather those at
//       run-time (or build-time), and iterate through them.
//       This allows re-using the facilities built in datetime.rs, and not having
//       divergent methods for transforming datetime string to `DateTimeL`.
const CLI_DT_FILTER_PATTERN1: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S", true, false, true);
const CLI_DT_FILTER_PATTERN2: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S%z", true, true, true);
const CLI_DT_FILTER_PATTERN3: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S%:z", true, true, true);
const CLI_DT_FILTER_PATTERN4: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S%#z", true, true, true);
const CLI_DT_FILTER_PATTERN5: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S%Z", true, true, true);
const CLI_DT_FILTER_PATTERN6: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S", true, false, true);
const CLI_DT_FILTER_PATTERN7: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S %z", true, true, true);
const CLI_DT_FILTER_PATTERN8: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S %:z", true, true, true);
const CLI_DT_FILTER_PATTERN9: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S %#z", true, true, true);
const CLI_DT_FILTER_PATTERN10: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S %Z", true, true, true);
const CLI_DT_FILTER_PATTERN11: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S", true, false, true);
const CLI_DT_FILTER_PATTERN12: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S %z", true, true, true);
const CLI_DT_FILTER_PATTERN13: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S %:z", true, true, true);
const CLI_DT_FILTER_PATTERN14: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S %#z", true, true, true);
const CLI_DT_FILTER_PATTERN15: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S %Z", true, true, true);
const CLI_DT_FILTER_PATTERN16: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S", true, false, true);
const CLI_DT_FILTER_PATTERN17: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S %z", true, true, true);
const CLI_DT_FILTER_PATTERN18: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S %:z", true, true, true);
const CLI_DT_FILTER_PATTERN19: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S %#z", true, true, true);
const CLI_DT_FILTER_PATTERN20: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S %Z", true, true, true);
const CLI_DT_FILTER_PATTERN21: CLI_DT_Filter_Pattern = ("%Y%m%d", true, false, false);
const CLI_DT_FILTER_PATTERN22: CLI_DT_Filter_Pattern = ("%Y-%m-%d", true, false, false);
const CLI_DT_FILTER_PATTERN23: CLI_DT_Filter_Pattern = ("%Y/%m/%d", true, false, false);
const CLI_DT_FILTER_PATTERN24: CLI_DT_Filter_Pattern = ("%Y%m%d %z", true, true, false);
const CLI_DT_FILTER_PATTERN25: CLI_DT_Filter_Pattern = ("%Y%m%d %:z", true, true, false);
const CLI_DT_FILTER_PATTERN26: CLI_DT_Filter_Pattern = ("%Y%m%d %#z", true, true, false);
const CLI_DT_FILTER_PATTERN27: CLI_DT_Filter_Pattern = ("%Y%m%d %Z", true, true, false);
const CLI_DT_FILTER_PATTERN28: CLI_DT_Filter_Pattern = ("+%s", false, false, true);

const CLI_FILTER_PATTERNS_COUNT: usize = 28;

/// CLI acceptable datetime filter patterns for the user-passed `-a` or `-b`
const CLI_FILTER_PATTERNS: [&CLI_DT_Filter_Pattern; CLI_FILTER_PATTERNS_COUNT] = [
    &CLI_DT_FILTER_PATTERN1,
    &CLI_DT_FILTER_PATTERN2,
    &CLI_DT_FILTER_PATTERN3,
    &CLI_DT_FILTER_PATTERN4,
    &CLI_DT_FILTER_PATTERN5,
    &CLI_DT_FILTER_PATTERN6,
    &CLI_DT_FILTER_PATTERN7,
    &CLI_DT_FILTER_PATTERN8,
    &CLI_DT_FILTER_PATTERN9,
    &CLI_DT_FILTER_PATTERN10,
    &CLI_DT_FILTER_PATTERN11,
    &CLI_DT_FILTER_PATTERN12,
    &CLI_DT_FILTER_PATTERN13,
    &CLI_DT_FILTER_PATTERN14,
    &CLI_DT_FILTER_PATTERN15,
    &CLI_DT_FILTER_PATTERN16,
    &CLI_DT_FILTER_PATTERN17,
    &CLI_DT_FILTER_PATTERN18,
    &CLI_DT_FILTER_PATTERN19,
    &CLI_DT_FILTER_PATTERN20,
    &CLI_DT_FILTER_PATTERN21,
    &CLI_DT_FILTER_PATTERN22,
    &CLI_DT_FILTER_PATTERN23,
    &CLI_DT_FILTER_PATTERN24,
    &CLI_DT_FILTER_PATTERN25,
    &CLI_DT_FILTER_PATTERN26,
    &CLI_DT_FILTER_PATTERN27,
    &CLI_DT_FILTER_PATTERN28,
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

lazy_static! {
    /// user-passed strings of a duration that is a relative offset.
    static ref REGEX_DUR_OFFSET: Regex = {
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

/// default separator for prepended strings
const CLI_PREPEND_SEP: &str = ":";

/// default CLI datetime format printed for CLI options `-u` or `-l`.
const CLI_OPT_PREPEND_FMT: &str = "%Y%m%dT%H%M%S%.3f%z";

/// `--help` _afterword_ message.
const CLI_HELP_AFTER: &str = concatcp!(
    "\
DateTime Filters may be strftime specifier patterns:
    \"",
    CLI_DT_FILTER_PATTERN1.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN2.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN3.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN4.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN5.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN6.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN7.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN8.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN9.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN10.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN11.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN12.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN13.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN14.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN15.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN16.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN17.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN18.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN19.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN20.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN21.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN22.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN23.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN24.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN25.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN26.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN27.0,
    "\"
    \"",
    CLI_DT_FILTER_PATTERN28.0,
    "\"

Or, DateTime Filter may be custom relative offset patterns:
    \"+DwDdDhDmDs\" or \"-DwDdDhDmDs\"
    \"@+DwDdDhDmDs\" or \"@-DwDdDhDmDs\"

Pattern \"+%s\" is Unix epoch timestamp in seconds with a preceding \"+\".
For example, value \"+946684800\" is be January 1, 2000 at 00:00, GMT.

Custom relative offset pattern \"+DwDdDhDmDs\" and \"-DwDdDhDmDs\" is the offset
from now (program start time) where \"D\" is a decimal number.
Each lowercase identifier is an offset duration:
\"w\" is weeks, \"d\" is days, \"h\" is hours, \"m\" is minutes, \"s\" is seconds.
For example, value \"-1w22h\" is one week and twenty-two hours in the past.
Value \"+30s\" is thirty seconds in the future.

Custom relative offset pattern \"@+DwDdDhDmDs\" and \"@-DwDdDhDmDs\" is relative
offset from the other datetime.
Arguments \"-a 20220102 -b @+1d\" are equivalent to \"-a 20220102 -b 20220103\".
Arguments \"-a @-6h -b 20220101T120000\" are equivalent to
\"-a 20220101T060000 -b 20220101T120000\".

Without a timezone offset (strftime specifier \"%z\" or \"%Z\"),
the Datetime Filter is presumed to be the local system timezone.

Ambiguous named timezones will be rejected, e.g. \"SST\".

Backslash escape sequences accepted by \"--sysline-separator\" are:
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES0, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES1, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES2, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES3, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES4, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES5, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES6, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES7, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES8, "\",
    \"", unescape::BACKSLASH_ESCAPE_SEQUENCES9, "\",

Resolved values of \"--dt-after\" and \"--dt-before\" can be reviewed in
the \"--summary\" output.

DateTime strftime specifiers are described at https://docs.rs/chrono/latest/chrono/format/strftime/

DateTimes supported are only of the Gregorian calendar.

DateTimes supported language is English."
);

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
    version,
    about,
    after_help = CLI_HELP_AFTER,
    verbatim_doc_comment,
)]
struct CLI_Args {
    /// Path(s) of log files or directories.
    /// Directories will be recursed. Symlinks will be followed.
    /// Paths may also be passed via STDIN, one per line. The user must
    /// supply argument "-" to signify PATHS are available from STDIN.
    #[clap(
        required = true,
        verbatim_doc_comment,
    )]
    paths: Vec<String>,

    /// DateTime Filter After: print syslog lines with a datetime that is at
    /// or after this datetime. For example, "20200102T120000" or "-5d".
    #[clap(
        short = 'a',
        long,
        verbatim_doc_comment,
    )]
    dt_after: Option<String>,

    /// DateTime Filter Before: print syslog lines with a datetime that is at
    /// or before this datetime.
    /// For example, "20200103T230000" or "@+1d+11h"
    #[clap(
        short = 'b',
        long,
        verbatim_doc_comment,
    )]
    dt_before: Option<String>,

    /// Default timezone offset for datetimes without a timezone.
    /// For example, "-0800", "+02:00", or "EDT". Ambiguous named timezones
    /// parsed from logs will use this value, e.g. timezone "IST" (to pass a
    /// value with leading "-", quote the argument, e.g. "-t=-0800").
    /// Default value is the local system timezone offset.
    #[clap(
        short = 't',
        long,
        verbatim_doc_comment,
        value_parser = cli_process_tz_offset,
        default_value_t=*Local.timestamp_opt(0, 0).unwrap().offset(),
    )]
    tz_offset: FixedOffset,

    /// Prepend DateTime in the UTC Timezone for every line.
    #[clap(
        short = 'u',
        long = "prepend-utc",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt",
        ],
    )]
    prepend_utc: bool,

    /// Prepend DateTime in the Local Timezone for every line.
    #[clap(
        short = 'l',
        long = "prepend-local",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt",
        ],
    )]
    prepend_local: bool,

    /// Prepend DateTime using strftime format string.
    #[clap(
        short = 'd',
        long = "prepend-dt-format",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_dt_format",
        ],
        requires = "group_prepend_dt",
        value_parser = cli_parser_prepend_dt_format,
        default_value_t = String::from(CLI_OPT_PREPEND_FMT),
    )]
    prepend_dt_format: String,

    /// Prepend file basename to every line.
    #[clap(
        short = 'n',
        long = "prepend-filename",
        verbatim_doc_comment,
        groups = &[
            "group_prepend_file",
        ]
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
    )]
    prepend_filepath: bool,

    /// Align column widths of prepended data.
    #[clap(
        short = 'w',
        long = "prepend-file-align",
        verbatim_doc_comment,
        requires = "group_prepend_file",
    )]
    prepend_file_align: bool,

    /// Separator string for prepended data.
    #[clap(
        long = "prepend-separator",
        verbatim_doc_comment,
        // TODO: how to require `any("prepend_file", "prepend_dt")`
        default_value_t = String::from(CLI_PREPEND_SEP)
    )]
    prepend_separator: String,

    /// An extra separator string between printed log lines.
    /// One "syslog line", or "sysline", may have multiple lines of text.
    /// Accepts a basic set of backslash escape sequences,
    /// e.g. "\0" for the null character.
    #[clap(
        long = "sysline-separator",
        required = false,
        verbatim_doc_comment,
        default_value_t = String::from(""),
        hide_default_value = true,
    )]
    sysline_separator: String,

    /// Choose to print to terminal using colors.
    #[clap(
        required = false,
        short = 'c',
        long = "color",
        verbatim_doc_comment,
        value_enum,
        default_value_t=CLI_Color_Choice::auto,
    )]
    color_choice: CLI_Color_Choice,

    /// Read blocks of this size in bytes.
    /// May pass value as any radix (hexadecimal, decimal, octal, binary).
    /// Using the default value is recommended.
    /// Most useful for developers.
    #[clap(
        required = false,
        short = 'z',
        long,
        verbatim_doc_comment,
        default_value_t = BLOCKSZ_DEF.to_string(),
        value_parser = cli_parse_blocksz,
    )]
    blocksz: String,

    /// Print a summary of files processed to stderr.
    /// Most useful for developers.
    #[clap(
        short,
        long,
        verbatim_doc_comment,
    )]
    summary: bool,
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
        return Err(format!(
            "--blocksz must be {} ≤ BLOCKSZ ≤ {}, it was {:?}",
            max_min, BLOCKSZ_MAX, blockszs
        ));
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
                    return Err(
                        format!("Given ambiguous timezone {:?} (this timezone abbreviation refers to several timezone offsets)", tzo)
                    );
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
        match dt {
            Some(dt_) => {
                defx!("return {:?}", dt_.offset());
                return Ok(*dt_.offset());
            }
            None => {}
        }
    }

    Err(format!("Unable to parse a timezone offset for --tz-offset {:?}", tzo))
}

/// `clap` argument validator for `--prepend-dt-format`.
fn cli_parser_prepend_dt_format(prepend_dt_format: &str) -> std::result::Result<String, String> {
    let dt = Utc
        .with_ymd_and_hms(2000, 1, 1, 0, 0, 0)
        .unwrap();
    dt.format(prepend_dt_format);

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

// regular expression processing of a user-passed duration string like `"-4m2s"`
// becomes duration of 4 minutes + 2 seconds
// helper function to `process_dt`
fn string_wdhms_to_duration(val: &String) -> Option<(Duration, DUR_OFFSET_TYPE)> {
    defn!("({:?})", val);

    let mut duration_offset_type: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Now;
    let mut duration_addsub: DUR_OFFSET_ADDSUB = DUR_OFFSET_ADDSUB::Add;
    let mut seconds: i64 = 0;
    let mut minutes: i64 = 0;
    let mut hours: i64 = 0;
    let mut days: i64 = 0;
    let mut weeks: i64 = 0;

    let captures = match REGEX_DUR_OFFSET.captures(val.as_str()) {
        Some(caps) => caps,
        None => {
            defx!("REGEX_DUR_OFFSET.captures(…) None");
            return None;
        }
    };

    match captures.name(CGN_DUR_OFFSET_TYPE) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_TYPE, match_.as_str());
            duration_offset_type = offset_match_to_offset_duration_type(match_.as_str());
        }
        None => {}
    }

    match captures.name(CGN_DUR_OFFSET_ADDSUB) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_ADDSUB, match_.as_str());
            duration_addsub = offset_match_to_offset_addsub(match_.as_str());
        }
        None => {}
    }

    let addsub: i64 = duration_addsub as i64;

    match captures.name(CGN_DUR_OFFSET_SECONDS) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_SECONDS, match_.as_str());
            let s_count = match_
                .as_str()
                .replace("s", "");
            match i64::from_str_radix(s_count.as_str(), 10) {
                Ok(val) => {
                    seconds = val * addsub;
                }
                Err(err) => {
                    eprintln!("ERROR: Unable to parse seconds from {:?} {}", match_.as_str(), err);
                    std::process::exit(1);
                }
            }
        }
        None => {}
    }
    match captures.name(CGN_DUR_OFFSET_MINUTES) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_MINUTES, match_.as_str());
            let s_count = match_
                .as_str()
                .replace("m", "");
            match i64::from_str_radix(s_count.as_str(), 10) {
                Ok(val) => {
                    minutes = val * addsub;
                }
                Err(err) => {
                    eprintln!("ERROR: Unable to parse minutes from {:?} {}", match_.as_str(), err);
                    std::process::exit(1);
                }
            }
        }
        None => {}
    }
    match captures.name(CGN_DUR_OFFSET_HOURS) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_HOURS, match_.as_str());
            let s_count = match_
                .as_str()
                .replace("h", "");
            match i64::from_str_radix(s_count.as_str(), 10) {
                Ok(val) => {
                    hours = val * addsub;
                }
                Err(err) => {
                    eprintln!("ERROR: Unable to parse hours from {:?} {}", match_.as_str(), err);
                    std::process::exit(1);
                }
            }
        }
        None => {}
    }
    match captures.name(CGN_DUR_OFFSET_DAYS) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_DAYS, match_.as_str());
            let s_count = match_
                .as_str()
                .replace("d", "");
            match i64::from_str_radix(s_count.as_str(), 10) {
                Ok(val) => {
                    days = val * addsub;
                }
                Err(err) => {
                    eprintln!("ERROR: Unable to parse days from {:?} {}", match_.as_str(), err);
                    std::process::exit(1);
                }
            }
        }
        None => {}
    }
    match captures.name(CGN_DUR_OFFSET_WEEKS) {
        Some(match_) => {
            defo!("matched named group {:?}, match {:?}", CGN_DUR_OFFSET_WEEKS, match_.as_str());
            let s_count = match_
                .as_str()
                .replace("w", "");
            match i64::from_str_radix(s_count.as_str(), 10) {
                Ok(val) => {
                    weeks = val * addsub;
                }
                Err(err) => {
                    eprintln!("ERROR: Unable to parse weeks from {:?} {}", match_.as_str(), err);
                    std::process::exit(1);
                }
            }
        }
        None => {}
    }

    let duration = Duration::seconds(seconds)
        + Duration::minutes(minutes)
        + Duration::hours(hours)
        + Duration::days(days)
        + Duration::weeks(weeks);
    defx!("return {:?}, {:?}", duration, duration_offset_type);

    Some((duration, duration_offset_type))
}

// Process duration string like `"-4m2s"` as relative offset of now,
// or relative offset of other user-passed datetime argument (`dt_other`).
// `val="-1d"` is one day ago.
// `val="+1m"` is one day added to the `dt_other`.
// helper function to function `process_dt`.
fn string_to_rel_offset_datetime(
    val: &String,
    tz_offset: &FixedOffset,
    dt_other_opt: &DateTimeLOpt,
    now_utc: &DateTime<Utc>,
) -> DateTimeLOpt {
    let (duration, duration_offset_type) = match string_wdhms_to_duration(val) {
        Some((dur, dur_type)) => (dur, dur_type),
        None => {
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
            defo!("now_sub {:?}", now_off.unwrap());

            now_off
        }
        DUR_OFFSET_TYPE::Other => {
            match dt_other_opt {
                Some(dt_other) => {
                    defo!("other     {:?}", dt_other);
                    let other_off = dt_other.checked_add_signed(duration);
                    defo!("other_off {:?}", other_off.unwrap());

                    other_off
                }
                None => {
                    eprintln!("ERROR: passed relative offset to other datetime {:?}, but other datetime was not set", val);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Transform a user-passed datetime `String` into a [`DateTimeL`].
///
/// Helper function to function `cli_process_args`.
///
/// [`DateTimeL`]: s4lib::data::datetime::DateTimeL
fn process_dt(
    dts_opt: Option<String>,
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
    for (pattern_, _has_year, has_tz, has_time) in CLI_FILTER_PATTERNS.iter() {
        let mut pattern: String = String::from(*pattern_);
        let mut dts_: String = dts.clone();
        // if !has_time then modify the value and pattern
        // e.g. `"20220101"` becomes `"20220101 T000000"`
        //      `"%Y%d%m"` becomes `"%Y%d%m T%H%M%S"`
        if !has_time {
            dts_.push_str(CLI_DT_FILTER_APPEND_TIME_VALUE);
            pattern.push_str(CLI_DT_FILTER_APPEND_TIME_PATTERN);
            defo!("appended {:?}, {:?}", CLI_DT_FILTER_APPEND_TIME_VALUE, CLI_DT_FILTER_APPEND_TIME_PATTERN);
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
    dto = match string_to_rel_offset_datetime(&dts, tz_offset, dt_other, now_utc) {
        Some(dto) => Some(dto),
        None => None,
    };
    // user-passed string was not parseable
    if dto.is_none() {
        eprintln!("ERROR: Unable to parse a datetime from {:?}", dts);
        std::process::exit(1);
    }
    defx!("return {:?}", dto);

    dto
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

    impl<'a> Iterator for InterpretEscapedString<'a> {
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
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES0: &'static str = r"\0";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES1: &'static str = r"\a";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES2: &'static str = r"\b";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES3: &'static str = r"\e";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES4: &'static str = r"\f";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES5: &'static str = r"\n";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES6: &'static str = r"\r";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES7: &'static str = r"\\";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES8: &'static str = r"\t";
    pub(super) const BACKSLASH_ESCAPE_SEQUENCES9: &'static str = r"\v";

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
    bool,
    bool,
    String,
    bool,
    bool,
    bool,
    String,
    String,
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
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
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
    match (string_wdhms_to_duration(args_dt_after_s), string_wdhms_to_duration(args_dt_before_s)) {
        (Some((_, DUR_OFFSET_TYPE::Other)), Some((_, DUR_OFFSET_TYPE::Other))) => {
            eprintln!("ERROR: cannot pass both --dt-after and --dt-before as relative to the other");
            std::process::exit(1);
        }
        (Some((_, DUR_OFFSET_TYPE::Other)), _) => {
            // special-case: process `-b` value then process `-a` value
            // e.g. `-a "@+1d" -b "20010203"`
            filter_dt_before = process_dt(args.dt_before, &tz_offset, &None, &UTC_NOW);
            defo!("filter_dt_before {:?}", filter_dt_before);
            filter_dt_after = process_dt(args.dt_after, &tz_offset, &filter_dt_before, &UTC_NOW);
            defo!("filter_dt_after {:?}", filter_dt_after);
        }
        _ => {
            // normal case: process `-a` value then process `-b` value
            filter_dt_after = process_dt(args.dt_after, &tz_offset, &None, &UTC_NOW);
            defo!("filter_dt_after {:?}", filter_dt_after);
            filter_dt_before = process_dt(args.dt_before, &tz_offset, &filter_dt_after, &UTC_NOW);
            defo!("filter_dt_before {:?}", filter_dt_before);
        }
    }

    #[allow(clippy::single_match)]
    match (filter_dt_after, filter_dt_before) {
        (Some(dta), Some(dtb)) => {
            if dta > dtb {
                eprintln!("ERROR: Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
                std::process::exit(1);
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

    let sysline_separator: String = match unescape::unescape_str(args.sysline_separator.as_str()) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: {:?}", err);
            std::process::exit(1);
        }
    };

    defo!("color_choice {:?}", color_choice);
    defo!("prepend_utc {:?}", args.prepend_utc);
    defo!("prepend_local {:?}", args.prepend_local);
    defo!("prepend_dt_format {:?}", args.prepend_dt_format);
    defo!("prepend_filename {:?}", args.prepend_filename);
    defo!("prepend_filepath {:?}", args.prepend_filepath);
    defo!("prepend_file_align {:?}", args.prepend_file_align);
    defo!("prepend_separator {:?}", args.prepend_separator);
    defo!("sysline_separator {:?}", sysline_separator);
    defo!("summary {:?}", args.summary);

    (
        paths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        args.prepend_utc,
        args.prepend_local,
        args.prepend_dt_format,
        args.prepend_filename,
        args.prepend_filepath,
        args.prepend_file_align,
        args.prepend_separator,
        sysline_separator,
        args.summary,
    )
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Process the user-passed command-line arguments.
/// Start function `processing_loop`.
/// Determine a process return code.
pub fn main() -> ExitCode {
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
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_prepend_dt_format,
        cli_opt_prepend_filename,
        // TODO: [2023/02/26] add option to prepend byte offset along with filename, helpful for development
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_prepend_separator,
        sysline_separator,
        cli_opt_summary,
    ) = cli_process_args();

    let mut processed_paths: ProcessPathResults = ProcessPathResults::with_capacity(paths.len() * 4);
    for path in paths.iter() {
        #[cfg(debug_assertions)]
        {
            defo!("path {:?}", path);
        }
        let ppaths: ProcessPathResults = process_path(path);
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
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_prepend_dt_format,
        cli_opt_prepend_filename,
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_prepend_separator,
        sysline_separator,
        cli_opt_summary,
    );

    defx!();

    if ret {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
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

/// File paths are needed as keys. Many such keys are passed around among
/// different threads.
/// Instead of passing clones of `FPath`, pass around a relatively light-weight
/// `usize` as a key.
/// The main processing thread uses the `PathId` key for various lookups,
/// including the file path.
type PathId = usize;

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
type ThreadInitData = (
    FPath,
    PathId,
    FileType,
    BlockSz,
    DateTimeLOpt,
    DateTimeLOpt,
    FixedOffset,
);

/// The type of log message sent from file processing thread to the main
/// printing thread enclosing the specific message.
#[derive(Debug)]
enum LogMessage {
    Sysline(SyslineP),
    Utmpx(Utmpx),
}
type LogMessageOpt = Option<LogMessage>;

/// Is this the last [`Sysline`] of the syslog file?
///
/// [`Sysline`]: s4lib::data::sysline::Sysline
type IsLastLogMessage = bool;

/// The data sent from file processing thread to the main printing thread.
///
/// * optional [`LogMessage`] (`Sysline`/`Utmpx`)
/// * optional [`Summary`]
/// * is this the last LogMessage (`Sysline`/`Utmpx`)?
/// * `FileProcessingResult`
///
/// There should never be a `Sysline` and a `Summary` received simultaneously.
///
/// [`LogMessage`]: self::LogMessage
/// [`Summary`]: s4lib::readers::summary::Summary
// TODO: Enforce that `Sysline` and `Summary` exclusivity with some kind of union.
type ChanDatum = (LogMessageOpt, SummaryOpt, IsLastLogMessage, FileProcessingResultBlockZero);

type MapPathIdDatum = BTreeMap<PathId, ChanDatum>;

type SetPathId = HashSet<PathId>;

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

/// This creates a [`SyslogProcessor`] and processes the file.<br/>
/// If it is a syslog file, then continues processing by sending each
/// processed [`Sysline`] through a [channel] to the main thread which
/// will print it.
///
/// Tuple `chan_send_dt` data described in [`ChanDatum`].
///
/// Tuple `thread_init_data` described in [`ThreadInitData`].
///
/// [`ThreadInitData`]: self::ThreadInitData
/// [`ChanDatum`]: self::ChanDatum
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
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("({:?})", path);

    let mut syslogproc = match SyslogProcessor::new(
        path.clone(),
        filetype,
        blocksz,
        tz_offset,
        filter_dt_after_opt,
        filter_dt_before_opt,
    ) {
        Ok(val) => val,
        Err(err) => {
            // TODO: [2022/08] this design needs work: the `Error` instance should be passed
            //       back in the channel but not via the Summary... I think....
            //       But more so, the current design has unnecessary
            //       duplication of `Error` information mapped to a `FileProcessingResultBlockZero`.
            //       e.g. remove `FileErrIo` and just have a more general `FileError((ErrorKind, String))`.
            //            Much easier to handle.
            //       The `FileProcessResult` should travel inside or outside the `Summary` and
            //       should not duplicate the information conveyed by the `Error` instance. Get rid
            //       of this tedious mapping. However, complicating that change this is the design
            //       of `Error`; they are difficult to pass around because they cannot be
            //       copied/cloned. Perhaps only save the `ErrorKind`?
            //       Additionally, this thread should not print error messages, only the main thread should do that.
            //       This needs more thought.
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                LogMessageType::Sysline,
                blocksz,
                Some(err.to_string())
            );
            let fileerr: FileProcessingResultBlockZero = match err.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!("ERROR: {} for {:?}", err, path);

                    FileProcessingResultBlockZero::FileErrIo(err)
                }
                _ => {
                    eprintln!("ERROR: {} for {:?}", err, path);

                    FILEERRSTUB
                }
            };
            match chan_send_dt.send((
                None,
                Some(summary),
                true,
                fileerr,
            )) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            defx!("({:?})", path);
            return;
        }
    };
    defo!("syslogproc {:?}", syslogproc);

    syslogproc.process_stage0_valid_file_check();

    let result = syslogproc.process_stage1_blockzero_analysis();
    match &result {
        FileProcessingResultBlockZero::FileErrNoLinesFound => {
            e_wrn!("no lines found {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrNoSyslinesFound => {
            e_wrn!("no syslines found {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrDecompress => {
            e_wrn!("could not decompress {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrWrongType => {
            e_wrn!("bad path {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrIo(err) => {
            e_err!("Error {} for {:?}", err, path);
        }
        FileProcessingResultBlockZero::FileOk => {}
        FileProcessingResultBlockZero::FileErrEmpty => {}
        FileProcessingResultBlockZero::FileErrNoSyslinesInDtRange => {}
        FileProcessingResultBlockZero::FileErrStub => {}
    }
    match result {
        FileProcessingResultBlockZero::FileOk => {}
        _ => {
            defo!("chan_send_dt.send((None, summary, true))");
            match chan_send_dt.send((
                None,
                Some(syslogproc.summary_complete()),
                true,
                result,
            )) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: stage1 chan_send_dt.send(…) failed {}", err);
                }
            }
            defx!("({:?})", path);
            return;
        }
    }

    // find first sysline acceptable to the passed filters
    match syslogproc.process_stage2_find_dt(&filter_dt_after_opt) {
        FileProcessingResultBlockZero::FileOk => {}
        _result => {
            defx!("Result {:?} ({:?})", _result, path);
            return;
        }
    }

    let mut sent_is_last: bool = false; // sanity check sending of `is_last`
    let mut fo1: FileOffset = 0;
    let search_more: bool;
    let result: ResultS3SyslineFind = syslogproc.find_sysline_between_datetime_filters(0);
    match result {
        ResultS3SyslineFind::Found((fo, syslinep)) => {
            fo1 = fo;
            let is_last: IsLastLogMessage = syslogproc.is_sysline_last(&syslinep) as IsLastLogMessage;
            deo!("{:?}({}): Found, chan_send_dt.send({:p}, None, {});", _tid, _tname, syslinep, is_last);
            match chan_send_dt.send((
                Some(LogMessage::Sysline(syslinep)),
                None,
                is_last,
                FILEOK,
            )) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            // XXX: sanity check
            if is_last {
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
        ResultS3SyslineFind::Done => {
            search_more = false;
        }
        ResultS3SyslineFind::Err(err) => {
            deo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", _tid, _tname, err);
            eprintln!(
                "ERROR: SyslogProcessor.find_sysline_between_datetime_filters(0) Path {:?} Error {}",
                path, err
            );
            search_more = false;
        }
    }

    if !search_more {
        deo!("{:?}({}): quit searching…", _tid, _tname);
        let summary_opt: SummaryOpt = Some(syslogproc.process_stage4_summary());
        deo!("{:?}({}): !search_more chan_send_dt.send((None, {:?}, {}));", _tid, _tname, summary_opt, false);
        match chan_send_dt.send((
            None,
            summary_opt,
            false,
            FILEOK,
        )) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: chan_send_dt.send(…) failed {}", err);
            }
        }
        defx!("({:?})", path);

        return;
    }

    // find all proceeding syslines acceptable to the passed filters
    syslogproc.process_stage3_stream_syslines();

    // the majority of sysline processing for this file occurs in this loop
    let mut syslinep_last_opt: Option<SyslineP> = None;
    loop {
        // TODO: [2022/06/20] see note about refactoring `find` functions so they are more intuitive
        let result: ResultS3SyslineFind = syslogproc.find_sysline_between_datetime_filters(fo1);
        match result {
            ResultS3SyslineFind::Found((fo, syslinep)) => {
                let syslinep_tmp = syslinep.clone();
                let is_last = syslogproc.is_sysline_last(&syslinep);
                deo!("{:?}({}): chan_send_dt.send(({:p}, None, {}));", _tid, _tname, syslinep, is_last);
                match chan_send_dt.send((
                    Some(LogMessage::Sysline(syslinep)),
                    None,
                    is_last,
                    FILEOK,
                )) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("ERROR: D chan_send_dt.send(…) failed {}", err);
                    }
                }
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
                match syslinep_last_opt {
                    Some(syslinep_last) => {
                        syslogproc.drop_data_try(&syslinep_last);
                    }
                    None => {}
                }
                syslinep_last_opt = Some(syslinep_tmp);
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                deo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", _tid, _tname, err);
                eprintln!("ERROR: syslogprocessor.find_sysline({}) {}", fo1, err);
                break;
            }
        }
    }

    syslogproc.process_stage4_summary();

    let summary = syslogproc.summary_complete();
    deo!("{:?}({}): last chan_send_dt.send((None, {:?}, {}));", _tid, _tname, summary, false);
    match chan_send_dt.send((
        None,
        Some(summary),
        false,
        FILEOK,
    )) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: E chan_send_dt.send(…) failed {}", err);
        }
    }

    defx!("({:?})", path);
}

fn exec_utmpprocessor(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
    _tname: &str,
    _tid: thread::ThreadId,
) {
    let (
        path,
        _pathid,
        filetype,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset,
    ) = thread_init_data;
    defn!("{:?}({}): ({:?})", _tid, _tname, path);
    matches!(filetype, FileType::Utmpx);

    let mut utmpreader = match UtmpxReader::new(
        path.clone(),
        blocksz,
        tz_offset,
    ) {
        Ok(val) => val,
        Err(err) => {
            let summary = Summary::new_failed(
                path.clone(),
                filetype,
                LogMessageType::Utmpx,
                blocksz,
                Some(err.to_string())
            );
            let fileerr: FileProcessingResultBlockZero = match err.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!("ERROR: {} for {:?}", err, path);

                    FileProcessingResultBlockZero::FileErrIo(err)
                }
                _ => {
                    eprintln!("ERROR: {} for {:?}", err, path);

                    FILEERRSTUB
                }
            };
            match chan_send_dt.send((
                None,
                Some(summary),
                true,
                fileerr,
            )) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            defx!("({:?})", path);
            return;
        }
    };
    defo!("{:?}({}): utmpreader {:?}", _tid, _tname, utmpreader);

    let mut fo: FileOffset = 0;
    //let mut first: bool = true;
    loop {
        let result: ResultS3UtmpxFind = utmpreader.find_entry_between_datetime_filters(
            fo,
            &filter_dt_after_opt,
            &filter_dt_before_opt,
        );
        let fo_last: FileOffset;
        match result {
            ResultS3UtmpxFind::Found((fo_, utmpentry)) => {
                defo!("ResultS3UtmpxFind::Found(({:?}, ...))", fo_);
                debug_assert_ne!(fo_, utmpentry.fileoffset_begin());
                fo_last = fo;
                fo = fo_;
                let is_last = utmpreader.is_last(&utmpentry);
                defo!("chan_send_dt.send((Some(LogMessage::Utmpx({}, …), None, {}, {:?}));", utmpentry.fileoffset_begin(), is_last, FILEOK);
                match chan_send_dt.send((
                    Some(LogMessage::Utmpx(utmpentry)),
                    None,
                    is_last,
                    FILEOK,
                )) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("ERROR: D chan_send_dt.send(…) failed {}", err);
                    }
                }
            }
            ResultS3UtmpxFind::Done => {
                defo!("ResultS3UtmpxFind::Done");
                break;
            }
            ResultS3UtmpxFind::Err(err) => {
                eprintln!("ERROR: find_entry({}) failed; {}", fo, err);
                defx!("ResultS3UtmpxFind::Err({})", err);
                // TODO: send error message through channel then return
                return;
            }
        }
        utmpreader.drop_entries(fo_last);
    }

    let summary = utmpreader.summary_complete();
    deo!("{:?}({}): last chan_send_dt.send((None, {:?}, {}));", _tid, _tname, summary, false);
    match chan_send_dt.send((
        None,
        Some(summary),
        false,
        FILEOK,
    )) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: chan_send_dt.send(…) failed {}", err);
        }
    }

    defx!("({:?})", path);
}

/// Thread entry point for processing one file.
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

    match &thread_init_data.2 {
        &FileType::Utmpx => exec_utmpprocessor(chan_send_dt, thread_init_data, tname, tid),
        _ => exec_syslogprocessor(chan_send_dt, thread_init_data, tname, tid),
    }
}

/// Statistics about the main processing thread's printing activity.
/// Used with CLI option `--summary`.
#[derive(Copy, Clone, Default)]
pub struct SummaryPrinted {
    /// count of bytes printed
    pub bytes: Count,
    /// underlying `LogMessageType`
    pub logmessagetype: LogMessageType,
    /// count of `Lines` printed
    pub lines: Count,
    /// count of `Syslines` printed
    pub syslines: Count,
    /// count of `Utmpx` printed
    pub utmpentries: Count,
    /// last datetime printed
    pub dt_first: DateTimeLOpt,
    pub dt_last: DateTimeLOpt,
}

impl fmt::Debug for SummaryPrinted {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Printed:")
            .field("bytes", &self.bytes)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field(
                "dt_first",
                &format_args!(
                    "{}",
                    match self.dt_first {
                        Some(dt) => {
                            dt.to_string()
                        }
                        None => {
                            String::from("None")
                        }
                    }
                ),
            )
            .field(
                "dt_last",
                &format_args!(
                    "{}",
                    match self.dt_last {
                        Some(dt) => {
                            dt.to_string()
                        }
                        None => {
                            String::from("None")
                        }
                    }
                ),
            )
            .finish()
    }
}

type MapPathIdSummaryPrint = BTreeMap<PathId, SummaryPrinted>;
type MapPathIdSummary = HashMap<PathId, Summary>;
type MapPathIdToProcessPathResult = HashMap<PathId, ProcessPathResult>;
type MapPathIdToFPath = BTreeMap<PathId, FPath>;
type MapPathIdToColor = HashMap<PathId, Color>;
type MapPathIdToPrinterLogMessage = HashMap<PathId, PrinterLogMessage>;
type MapPathIdToFileType = HashMap<PathId, FileType>;
type MapPathIdToLogMessageType = HashMap<PathId, LogMessageType>;
type MapPathIdToMimeGuess = HashMap<PathId, MimeGuess>;
type SummaryPrintedOpt = Option<SummaryPrinted>;

// TODO: move `SummaryPrinted` into `printer/summary.rs`
impl SummaryPrinted {
    pub fn new(logmessagetype: LogMessageType) -> SummaryPrinted {
        SummaryPrinted {
            bytes: 0,
            logmessagetype,
            lines: 0,
            syslines: 0,
            utmpentries: 0,
            dt_first: None,
            dt_last: None,
        }
    }

    /// Print a `SummaryPrinted` with color on stderr.
    ///
    /// Mimics debug print but with colorized zero values.
    /// Only colorize if associated `SummaryOpt` has corresponding
    /// non-zero values.
    pub fn print_colored_stderr(
        &self,
        color_choice_opt: Option<ColorChoice>,
        summary_opt: &SummaryOpt,
        indent1: &str,
        indent2: &str,
    ) {
        let summary: &Summary = match summary_opt {
            Some(s) => s,
            None => return,
        };
        let (
            summaryblockreader,
            summarylinereader_opt,
            _summarysyslinereader_opt,
            _summarysyslogprocessor_opt,
            summaryutmpreader_opt,
        ) = match &summary.readerdata {
            // `Dummy` may occur for files without adequate read permissions
            SummaryReaderData::Dummy => return,
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    summarylinereader,
                    summarysyslinereader,
                    summarysyslogprocessor,
                )
            ) => {
                (
                    summaryblockreader,
                    Some(summarylinereader),
                    Some(summarysyslinereader),
                    Some(summarysyslogprocessor),
                    None,
                )
            }
            SummaryReaderData::Utmpx(
                (
                    summaryblockreader,
                    summaryutmpreader,
                )
            ) => {
                (
                    summaryblockreader,
                    None,
                    None,
                    None,
                    Some(summaryutmpreader),
                )
            }
        };
        eprintln!("{}Summary Printed:", indent1);

        eprint!("{}bytes          ", indent2);
        if self.bytes == 0 && summaryblockreader.blockreader_bytes != 0 {
            match print_colored_stderr(
                COLOR_ERROR,
                color_choice_opt,
                self.bytes
                    .to_string()
                    .as_bytes(),
            ) {
                Err(err) => {
                    eprintln!("\nERROR: print_colored_stderr {:?}", err);
                    return;
                }
                Ok(_) => eprintln!(),
            }
        } else {
            eprintln!("{}", self.bytes);
        }

        if summarylinereader_opt.is_some() {
            eprint!("{}lines          ", indent2);
            if self.lines == 0 && summaryblockreader.blockreader_bytes != 0 {
                match print_colored_stderr(
                    COLOR_ERROR,
                    color_choice_opt,
                    self.lines
                        .to_string()
                        .as_bytes(),
                ) {
                    Err(err) => {
                        eprintln!("\nERROR: print_colored_stderr {:?}", err);
                        return;
                    }
                    Ok(_) => eprintln!(),
                }
            } else {
                eprintln!("{}", self.lines);
            }
        }

        match summaryutmpreader_opt {
            Some(summaryutmpreader) => {
                eprint!("{}utmpx          ", indent2);
                if self.utmpentries == 0 && summaryutmpreader.utmpxreader_utmp_entries != 0 {
                    match print_colored_stderr(
                        COLOR_ERROR,
                        color_choice_opt,
                        self.utmpentries
                            .to_string()
                            .as_bytes(),
                    ) {
                        Err(err) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", err);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    eprintln!("{}", self.utmpentries);
                }
            }
            None => {}
        }

        match summarylinereader_opt {
            Some(summarylinereader) => {
                eprint!("{}syslines       ", indent2);
                if self.syslines == 0 && summarylinereader.linereader_lines != 0 {
                    match print_colored_stderr(
                        COLOR_ERROR,
                        color_choice_opt,
                        self.syslines
                            .to_string()
                            .as_bytes(),
                    ) {
                        Err(err) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", err);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    eprintln!("{}", self.syslines);
                }
            }
            None => {}
        }

        match summarylinereader_opt {
            Some(summarylinereader) => {
                if self.dt_first.is_none() && summarylinereader.linereader_lines != 0 {
                    eprint!("{}datetime first ", indent2);
                    match print_colored_stderr(COLOR_ERROR, color_choice_opt, "None Found".as_bytes()) {
                        Err(err) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", err);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    match self.dt_first {
                        Some(dt) => eprintln!("{}datetime first {:?}", indent1, dt),
                        None => {}
                    }
                }
                if self.dt_last.is_none() && summarylinereader.linereader_lines != 0 {
                    eprint!("{}datetime last  ", indent2);
                    match print_colored_stderr(COLOR_ERROR, color_choice_opt, "None Found".as_bytes()) {
                        Err(err) => {
                            eprintln!("\nERROR: print_colored_stderr {:?}", err);
                            return;
                        }
                        Ok(_) => eprintln!(),
                    }
                } else {
                    match self.dt_last {
                        Some(dt) => eprintln!("{}datetime last  {:?}", indent1, dt),
                        None => {}
                    }
                }
        
            }
            None => {}
        }
    }

    // TODO: [2022/06/21] cost-savings: any way to avoid a `DateTime` copy on every printed sysline?
    fn summaryprint_update_dt(
        &mut self,
        dt: &DateTimeLOpt,
    ) {
        defñ!();
        if let Some(dt_) = dt {
            match self.dt_first {
                Some(dt_first) => {
                    if dt_ < &dt_first {
                        self.dt_first = Some(*dt_);
                    };
                }
                None => {
                    self.dt_first = Some(*dt_);
                }
            };
            match self.dt_last {
                Some(dt_last) => {
                    if dt_ > &dt_last {
                        self.dt_last = Some(*dt_);
                    };
                }
                None => {
                    self.dt_last = Some(*dt_);
                }
            };
        };
    }

    /// Update a `SummaryPrinted` with information from a printed `Sysline`.
    fn summaryprint_update_sysline(
        &mut self,
        syslinep: &SyslineP,
    ) {
        defñ!();
        debug_assert!(matches!(self.logmessagetype, LogMessageType::Sysline | LogMessageType::All), "Unexpected LogMessageType {:?}", self.logmessagetype);
        self.syslines += 1;
        self.lines += (*syslinep).count_lines();
        self.bytes += (*syslinep).count_bytes();
        self.summaryprint_update_dt(&(*syslinep).dt());
    }

    /// Update a `SummaryPrinted` with information from a printed `Utmpx`.
    fn summaryprint_update_utmpentry(
        &mut self,
        utmpentry: &Utmpx,
    ) {
        defñ!();
        debug_assert!(matches!(self.logmessagetype, LogMessageType::Utmpx | LogMessageType::All), "Unexpected LogMessageType {:?}", self.logmessagetype);
        self.utmpentries += 1;
        self.bytes += (*utmpentry).len() as Count;
        self.summaryprint_update_dt(&Some(*utmpentry.dt()));
    }

    /// Update a `SummaryPrinted` with information from a printed `LogMessage`.
    fn _summaryprint_update(
        &mut self,
        logmessage: &LogMessage,
    ) {
        defñ!();
        match logmessage {
            LogMessage::Sysline(syslinep) => {
                self.summaryprint_update_sysline(syslinep);
            }
            LogMessage::Utmpx(utmpentry) => {
                self.summaryprint_update_utmpentry(utmpentry);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `Sysline`.
    ///
    /// Helper function to function `processing_loop`.
    fn summaryprint_map_update_sysline(
        syslinep: &SyslineP,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_sysline(syslinep);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Sysline);
                sp.summaryprint_update_sysline(syslinep);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `Utmpx`.
    ///
    /// Helper function to function `processing_loop`.
    fn summaryprint_map_update_utmpentry(
        utmpentry: &Utmpx,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_utmpentry(utmpentry);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Utmpx);
                sp.summaryprint_update_utmpentry(utmpentry);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted`.
    ///
    /// Helper function to function `processing_loop`.
    fn _summaryprint_map_update(
        logmessage: &LogMessage,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
    ) {
        defñ!();
        match logmessage {
            LogMessage::Sysline(syslinep) => {
                Self::summaryprint_map_update_sysline(syslinep, pathid, map_)
            }
            LogMessage::Utmpx(utmpentry) => {
                Self::summaryprint_map_update_utmpentry(utmpentry, pathid, map_)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Print the various caching statistics.
const OPT_SUMMARY_PRINT_CACHE_STATS: bool = true;

/// Print the various drop statistics.
const OPT_SUMMARY_PRINT_DROP_STATS: bool = true;

/// For printing `--summary` lines, indentation.
const OPT_SUMMARY_PRINT_INDENT1: &str = "  ";
const OPT_SUMMARY_PRINT_INDENT2: &str = "      ";
const OPT_SUMMARY_PRINT_INDENT3: &str = "                   ";

// -------------------------------------------------------------------------------------------------

/// Helper function to function `processing_loop`.
#[inline(always)]
fn summary_update(
    pathid: &PathId,
    summary: Summary,
    map_: &mut MapPathIdSummary,
) {
    if let Some(val) = map_.insert(*pathid, summary) {
        eprintln!(
            "Error: processing_loop: map_pathid_summary already contains key {:?} with {:?}, overwritten",
            pathid, val
        );
    };
}

/// Five seems like a good number for the channel capacity *shrug*
const CHANNEL_CAPACITY: usize = 5;

/// The main [`Sysline`] processing and printing loop.
///
/// 1. creates threads to process each file
///
/// 2. waits on each thread to receive a processed `Sysline`
///    _or_ a closed [channel]
///    a. prints received `Sysline` in datetime order
///    b. repeat 2. until each thread sends a `IsLastSysline` value `true`
///
/// 3. print each [`Summary`] (if CLI option `--summary`)
///
/// This main thread should be the only thread that prints to stdout.<br/>
/// In `--release` builds, other file processing threads may rarely print
/// messages to stderr.<br/>
/// In debug builds, other file processing threads print verbosely.
///
/// [`Sysline`]: s4lib::data::sysline::Sysline
/// [`Summary`]: s4lib::readers::summary::Summary
/// [channel]: self::ChanRecvDatum
#[allow(clippy::too_many_arguments)]
fn processing_loop(
    mut paths_results: ProcessPathResults,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeLOpt,
    filter_dt_before_opt: &DateTimeLOpt,
    tz_offset: FixedOffset,
    color_choice: ColorChoice,
    cli_opt_prepend_utc: bool,
    cli_opt_prepend_local: bool,
    cli_prepend_dt_format: String,
    cli_opt_prepend_filename: bool,
    cli_opt_prepend_filepath: bool,
    cli_opt_prepend_file_align: bool,
    cli_prepend_separator: String,
    sysline_separator: String,
    cli_opt_summary: bool,
) -> bool {
    defn!(
        "({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})",
        paths_results,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        color_choice,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_prepend_dt_format,
        cli_opt_summary
    );

    // XXX: sanity check
    assert!(
        !(cli_opt_prepend_filename && cli_opt_prepend_filepath),
        "Cannot both cli_opt_prepend_filename && cli_opt_prepend_filepath"
    );
    // XXX: sanity check
    assert!(
        !(cli_opt_prepend_utc && cli_opt_prepend_local),
        "Cannot both cli_opt_prepend_utc && cli_opt_prepend_local"
    );

    if paths_results.is_empty() {
        defx!("paths_results.is_empty(); nothing to do");
        return true;
    }

    // TODO: [2022/06/02] this point needs a PathToPaths thingy that expands user-passed Paths to all possible paths_valid,
    //       e.g.
    //       given a directory path, returns paths_valid of possible syslog files found recursively.
    //       given a symlink, follows the symlink
    //       given a path to a tar file, returns paths_valid of possible syslog files within that .tar file.
    //       given a plain valid file path, just returns that path
    //       would return `Vec<(path: FPath, subpath: FPath, type_: FILE_TYPE, Option<result>: common::FileProcessingResult)>`
    //         where `path` is actual path,
    //         `subpath` is path within a .tar/.zip file
    //         `type_` is enum for `FILE` `FILE_IN_ARCHIVE_TAR`, `FILE_IN_ARCHIVE_TAR_COMPRESS_GZ`,
    //           `FILE_COMPRESS_GZ`, etc.
    //          `result` of `Some(FileProcessingResult)` if processing has completed or just `None`
    //       (this might be a better place for mimeguess and mimeanalysis?)
    //       Would be best to first implement `FILE`, then `FILE_COMPRESS_GZ`, then `FILE_IN_ARCHIVE_TAR`

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
    let mut map_pathid_results_invalid = MapPathIdToProcessPathResult::with_capacity(file_count);
    // use `map_pathid_path` for iterating, it is a BTreeMap (which iterates in consistent key order)
    let mut map_pathid_path = MapPathIdToFPath::new();
    // map `PathId` to `FileType`
    let mut map_pathid_filetype = MapPathIdToFileType::with_capacity(file_count);
    // map `PathId` to `LogMessageType`
    let mut map_pathid_logmessagetype = MapPathIdToLogMessageType::with_capacity(file_count);
    // map `PathId` to `MimeGuess`
    let mut map_pathid_mimeguess = MapPathIdToMimeGuess::with_capacity(file_count);
    let mut paths_total: usize = 0;

    for (pathid_counter, processpathresult) in paths_results
        .drain(..)
        .enumerate()
    {
        match processpathresult {
            // XXX: use `ref` to avoid "use of partially moved value" error
            ProcessPathResult::FileValid(ref path, ref mimeguess, ref filetype) => {
                defo!("map_pathid_results.push({:?})", path);
                map_pathid_path.insert(pathid_counter, path.clone());
                map_pathid_filetype.insert(pathid_counter, *filetype);
                let logmessagetype: LogMessageType = filetype_to_logmessagetype(*filetype);
                map_pathid_logmessagetype.insert(pathid_counter, logmessagetype);
                map_pathid_mimeguess.insert(pathid_counter, *mimeguess);
                map_pathid_results.insert(pathid_counter, processpathresult);
            }
            _ => {
                defo!("paths_invalid_results.push({:?})", processpathresult);
                map_pathid_results_invalid.insert(pathid_counter, processpathresult);
            }
        };
        paths_total += 1;
    }

    for (_pathid, result_invalid) in map_pathid_results_invalid.iter() {
        match result_invalid {
            ProcessPathResult::FileErrNoPermissions(path, _) => {
                e_wrn!("not enough permissions {:?}", path);
            }
            ProcessPathResult::FileErrNotSupported(path, _) => {
                e_wrn!("not a supported file {:?}", path);
            }
            ProcessPathResult::FileErrNotAFile(path) => {
                e_wrn!("not a file {:?}", path);
            }
            ProcessPathResult::FileErrNotExist(path) => {
                e_wrn!("path does not exist {:?}", path);
            }
            ProcessPathResult::FileValid(..) => {}
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
    // mapping of PathId to received data. Most important collection for the remainder
    // of this function.
    let mut map_pathid_chanrecvdatum = MapPathIdChanRecvDatum::new();
    // mapping PathId to colors for printing.
    let mut map_pathid_color = MapPathIdToColor::with_capacity(file_count);
    // mapping PathId to a `Summary` for `--summary`
    let mut map_pathid_summary = MapPathIdSummary::with_capacity(file_count);
    // Track if an error has been printed regarding a particular type of
    // logmessage printing problem.
    // only want to print this particular error once, not hundreds of times.
    // These repetitive errors that occur from logmessage printing are usually
    // due to process pipe failures or terminal malfunctions/oddities.
    let mut has_print_err: bool = false;
    // "mapping" of PathId to select index, used in `recv_many_data`
    let mut index_select = MapIndexToPathId::with_capacity(file_count);

    // initialize processing channels/threads, one per `PathId`
    for pathid in map_pathid_path.keys() {
        map_pathid_color.insert(*pathid, color_rand());
    }
    for (pathid, path) in map_pathid_path.iter() {
        let (filetype, _) = match map_pathid_results.get(pathid) {
            Some(processpathresult) => match processpathresult {
                ProcessPathResult::FileValid(path, _m, filetype) => (filetype, path),
                val => {
                    eprintln!("ERROR: unhandled ProcessPathResult {:?}", val);
                    continue;
                }
            },
            None => {
                panic!("bad pathid {}", pathid);
            }
        };
        let thread_data: ThreadInitData = (
            path.clone().to_owned(),
            *pathid,
            *filetype,
            blocksz,
            *filter_dt_after_opt,
            *filter_dt_before_opt,
            tz_offset,
        );
        let (chan_send_dt, chan_recv_dt): (ChanSendDatum, ChanRecvDatum) = crossbeam_channel::bounded(CHANNEL_CAPACITY);
        defo!("map_pathid_chanrecvdatum.insert({}, …);", pathid);
        map_pathid_chanrecvdatum.insert(*pathid, chan_recv_dt);
        let basename_: FPath = basename(path);
        match thread::Builder::new()
            .name(basename_.clone())
            .spawn(move || exec_fileprocessor_thread(chan_send_dt, thread_data))
        {
            Ok(_joinhandle) => {}
            Err(err) => {
                eprintln!("ERROR: thread.name({:?}).spawn() pathid {} failed {:?}", basename_, pathid, err);
                map_pathid_chanrecvdatum.remove(pathid);
                map_pathid_color.remove(pathid);
                continue;
            }
        }
    }

    if map_pathid_chanrecvdatum.is_empty() {
        de_err!("map_pathid_chanrecvdatum.is_empty(); nothing to do.");
        eprintln!("Summary:");
        print_summary_opt_printed(&None, &None, &color_choice);
        eprintln!(
            "Paths total {}, files not processed {}, files processed {}",
            paths_total,
            map_pathid_results_invalid.len(),
            map_pathid_results.len(),
        );
        return false;
    }

    type RecvResult4 = std::result::Result<ChanDatum, crossbeam_channel::RecvError>;

    /// run `.recv` on many Receiver channels simultaneously using `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    #[inline(always)]
    fn recv_many_chan<'a>(
        pathid_chans: &'a MapPathIdChanRecvDatum,
        map_index_pathid: &mut MapIndexToPathId,
        filter_: &SetPathId,
    ) -> Option<(PathId, RecvResult4)> {
        def1n!("(…)");
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
        assert!(!map_index_pathid.is_empty(),
            "Did not load any recv operations for select.select(). Overzealous filter? possible channels count {}, filter {:?}", pathid_chans.len(), filter_
        );
        def1o!("map_index_pathid: {:?}", map_index_pathid);
        // `select()` blocks until one of the loaded channel operations becomes ready
        let soper: crossbeam_channel::SelectedOperation = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index: usize = soper.index();
        def1o!("soper.index() returned {}", index);
        let pathid: &PathId = match map_index_pathid.get(&index) {
            Some(pathid_) => pathid_,
            None => {
                eprintln!("ERROR: failed to map_index_pathid.get({})", index);
                return None;
            }
        };
        def1o!("map_index_pathid.get({}) returned {}", index, pathid);
        let chan: &ChanRecvDatum = match pathid_chans.get(pathid) {
            Some(chan_) => chan_,
            None => {
                eprintln!("ERROR: failed to pathid_chans.get({})", pathid);
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
    // preparation for the main coordination loop (e.g. the "game loop")
    //

    let mut first_print = true;
    let mut map_pathid_datum = MapPathIdDatum::new();
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
    let color_default = COLOR_DEFAULT;

    // mapping PathId to colors for printing.
    let mut map_pathid_printer = MapPathIdToPrinterLogMessage::with_capacity(file_count);

    // count of not okay FileProcessing
    let mut _fileprocessing_not_okay: usize = 0;

    // track which paths had syslines
    let mut paths_printed_logmessages: SetPathId = SetPathId::with_capacity(file_count);

    //
    // the main processing loop (e.g the "game loop")
    //
    // process the "receiving sysline" channels from the running file processing threads.
    // print the earliest available `Sysline`.
    //

    // channels that should be disconnected per "game loop" loop iteration
    let mut disconnect = Vec::<PathId>::with_capacity(file_count);
    // shortcut to the bytes of the `sysline_separator`
    let sepb: &[u8] = sysline_separator.as_str().as_bytes();

    // buffer to assist printing Utmpx; passed to `Utmpx::as_bytes`
    let mut buffer_utmp: [u8; UTMPX_SZ * 2] = [0; UTMPX_SZ * 2];

    loop {
        disconnect.clear();

        if cfg!(debug_assertions) {
            defo!("map_pathid_datum.len() {}", map_pathid_datum.len());
            for (pathid, _datum) in map_pathid_datum.iter() {
                let _path: &FPath = map_pathid_path
                    .get(pathid)
                    .unwrap();
                defo!("map_pathid_datum: thread {} {} has data", _path, pathid);
            }
            defo!("map_pathid_chanrecvdatum.len() {}", map_pathid_chanrecvdatum.len());
            for (pathid, _chanrdatum) in map_pathid_chanrecvdatum.iter() {
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

        if map_pathid_chanrecvdatum.len() != map_pathid_datum.len() {
            // if…
            // `map_pathid_chanrecvdatum` does not have a `ChanRecvDatum` (and thus a `SyslineP` and
            // thus a `DatetimeL`) for every channel (file being processed).
            // (Every channel must return a `DatetimeL` to to then compare *all* of them, see which is earliest).
            // So call `recv_many_chan` to check if any channels have a new `ChanRecvDatum` to
            // provide.

            let pathid: PathId;
            let result: RecvResult4;
            (pathid, result) = match recv_many_chan(&map_pathid_chanrecvdatum, &mut index_select, &set_pathid)
            {
                Some(val) => val,
                None => {
                    eprintln!("ERROR: recv_many_chan returned None which is unexpected");
                    continue;
                }
            };
            match result {
                // (SyslineP_Opt, SummaryOpt, IsLastLogMessage, FileProcessingResultBlockZero)
                Ok(chan_datum) => {
                    defo!("B crossbeam_channel::Found for PathId {:?};", pathid);
                    match chan_datum.3 {
                        FileProcessingResultBlockZero::FileOk => {}
                        _ => {
                            _fileprocessing_not_okay += 1;
                        }
                    }
                    if let Some(summary) = chan_datum.1 {
                        assert!(chan_datum.0.is_none(), "ChanDatum Some(Summary) and Some(SyslineP); should only have one Some(). PathId {:?}", pathid);
                        summary_update(&pathid, summary, &mut map_pathid_summary);
                        defo!("B will disconnect channel {:?}", pathid);
                        // receiving a `Summary` means that was the last data sent on the channel
                        disconnect.push(pathid);
                    } else {
                        assert!(
                            chan_datum.0.is_some(),
                            "ChanDatum None(Summary) and None(SyslineP); should have one Some(). PathId {:?}",
                            pathid
                        );
                        map_pathid_datum.insert(pathid, chan_datum);
                        set_pathid.insert(pathid);
                    }
                    chan_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    defo!("B crossbeam_channel::RecvError, will disconnect channel for PathId {:?};", pathid);
                    // this channel was closed by the sender, it should be disconnected
                    disconnect.push(pathid);
                    chan_recv_err += 1;
                }
            }
        } else {
            // else…
            // There is a DateTime available for *every* channel (one channel is one File Processing
            // thread). The datetimes can be compared among all remaining files. The sysline with
            // the earliest datetime is printed.

            if cfg!(debug_assertions) {
                for (_i, (_k, _v)) in map_pathid_chanrecvdatum
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

            if first_print {
                // One-time creation of prepended datas and `PrinterLogMessage`.

                // First, get a set of all pathids with awaiting LogMessages, ignoring paths
                // for which no LogMessages were found.
                // No LogMessages will be printed for those paths that did not return a LogMessage:
                // - do not include them in determining prepended width (CLI option `-w`).
                // - do not create a `PrinterLogMessage` for them.
                let mut pathid_with_logmessages: SetPathId = SetPathId::with_capacity(map_pathid_datum.len());
                for (pathid, _) in map_pathid_datum
                    .iter()
                    .filter(|(_k, v)| v.0.is_some())
                {
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
                            let bname: String = basename(path);
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
                        let bname: String = basename(path);
                        let prepend: String =
                            format!("{0:<1$}{2}", bname, prependname_width, cli_prepend_separator);
                        pathid_to_prependname.insert(*pathid, prepend);
                    }
                } else if cli_opt_prepend_filepath {
                    // pre-create prepended filepath strings once (`-p`)
                    if cli_opt_prepend_file_align {
                        // determine prepended width (`-w`)
                        for pathid in pathid_with_logmessages.iter() {
                            let path = match map_pathid_path.get(pathid) {
                                Some(path_) => path_,
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
                            Some(path_) => path_,
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
                        .unwrap_or(&color_default);
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
                    let prepend_date_format: Option<String> =
                        match cli_opt_prepend_local || cli_opt_prepend_utc {
                            true => Some(cli_prepend_dt_format.clone() + cli_prepend_separator.as_str()),
                            false => None,
                        };
                    let prepend_date_offset: Option<FixedOffset> =
                        match (cli_opt_prepend_local, cli_opt_prepend_utc) {
                            (true, false) => Some(*Local::now().offset()),
                            (false, true) => Some(*FIXEDOFFSET0),
                            (false, false) => None,
                            // XXX: this should not happen
                            _ => panic!("bad CLI options --local --utc"),
                        };
                    let printer: PrinterLogMessage = PrinterLogMessage::new(
                        color_choice,
                        *color_,
                        prepend_file,
                        prepend_date_format,
                        prepend_date_offset,
                    );
                    map_pathid_printer.insert(*pathid, printer);
                }

                first_print = false;
            } // if first_print

            // (path, channel data) for the sysline with earliest datetime ("minimum" datetime)
            //
            // Here is part of the "sorting" of syslines process by datetime.
            // In case of tie datetime values, the tie-breaker will be order of `BTreeMap::iter_mut` which
            // iterates in order of key sort. https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.iter_mut
            //
            // XXX: assume `unwrap` will never fail
            //
            // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
            //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
            //

            let pathid: &PathId;
            let chan_datum: &mut ChanDatum;
            (pathid, chan_datum) = match map_pathid_datum
                .iter_mut()
                .min_by(|x, y|
                    {
                        match x.1.0.as_ref().unwrap() {
                            crate::LogMessage::Utmpx(utmpentry) => {
                                let y_dt = match y.1.0.as_ref().unwrap() {
                                    crate::LogMessage::Utmpx(utmpentry_y) => {
                                        utmpentry_y.dt()
                                    }
                                    crate::LogMessage::Sysline(syslinep_y) => {
                                        syslinep_y.dt().as_ref().unwrap()
                                    }
                                };

                                utmpentry.dt().cmp(y_dt)
                            }
                            crate::LogMessage::Sysline(syslinep) => {
                                let y_dt = match y.1.0.as_ref().unwrap() {
                                    crate::LogMessage::Utmpx(utmpentry_y) => {
                                        utmpentry_y.dt()
                                    }
                                    crate::LogMessage::Sysline(syslinep_y) => {
                                        syslinep_y.dt().as_ref().unwrap()
                                    }
                                };

                                syslinep.dt().as_ref().unwrap().cmp(y_dt)
                            }
                        }
                    }
                )
            {
                Some(val) => (val.0, val.1),
                None => {
                    eprintln!("ERROR map_pathid_datum.iter_mut().min_by() returned None");
                    // XXX: not sure what else to do here
                    continue;
                }
            };

            if let Some(summary) = chan_datum.1.clone() {
                // Receiving a Summary implies the last data was sent on the channel
                defo!("A2 chan_datum has Summary, PathId: {:?}", pathid);
                assert!(
                    chan_datum.0.is_none(),
                    "ChanDatum Some(Summary) and Some(SyslineP); should only have one Some(). PathId: {:?}",
                    pathid
                );
                summary_update(pathid, summary, &mut map_pathid_summary);
                defo!("A2 will disconnect channel {:?}", pathid);
                disconnect.push(*pathid);
            } else {
                // Is last log message of the file?
                let is_last: bool = chan_datum.2;
                match chan_datum.0.as_ref().unwrap() {
                    LogMessage::Sysline(syslinep) => {
                        //let syslinep: &SyslineP = chan_datum.0.as_ref().unwrap();
                        defo!(
                            "A3.1 printing SyslineP @[{}, {}] PathId: {:?}",
                            syslinep.fileoffset_begin(),
                            syslinep.fileoffset_end(),
                            pathid
                        );
                        // print the sysline!
                        // the most important part of this main thread loop
                        let printer: &mut PrinterLogMessage = map_pathid_printer
                            .get_mut(pathid)
                            .unwrap();
                        match printer.print_sysline(syslinep) {
                            Ok(_) => {}
                            Err(err) => {
                                // Only print a printing error once.
                                // In case of piping to something like `head`, it looks bad to print
                                // the same error tens or hundreds of times for a common pipe operation.
                                if !has_print_err {
                                    has_print_err = true;
                                    // BUG: Issue #3 colorization settings in the context of a pipe
                                    eprintln!("ERROR: failed to print {}", err);
                                }
                            }
                        }
                        if ! sepb.is_empty() {
                            write_stdout(sepb);
                            if cli_opt_summary {
                                summaryprinted.bytes += sepb.len() as Count;
                            }
                        }
                        // If a file's last char is not a '\n' then the next printed sysline
                        // (from a different file) will print on the same terminal line.
                        // While this is accurate byte-wise, it's difficult to read read and unexpected, and
                        // makes scripting line-oriented scripting more difficult. This is especially
                        // visually jarring when prepended data is present (`-l`, `-p`, etc.).
                        // So in case of no ending '\n', print an extra '\n' to improve human readability
                        // and scriptability.
                        if is_last && !(*syslinep).ends_with_newline() {
                            write_stdout(&NLu8a);
                            if cli_opt_summary {
                                summaryprinted.bytes += 1;
                            }
                        }
                        if cli_opt_summary {
                            paths_printed_logmessages.insert(*pathid);
                            // update the per processing file `SummaryPrinted`
                            SummaryPrinted::summaryprint_map_update_sysline(syslinep, pathid, &mut map_pathid_sumpr);
                            // update the single total program `SummaryPrinted`
                            summaryprinted.summaryprint_update_sysline(syslinep);
                        }
                    }
                    LogMessage::Utmpx(utmpentry) => {
                        defo!("A3.2 printing Utmpx PathId: {:?}", pathid);
                        // the most important part of this main thread loop
                        let printer: &mut PrinterLogMessage = map_pathid_printer
                            .get_mut(pathid)
                            .unwrap();
                        match printer.print_utmpentry(utmpentry, &mut buffer_utmp) {
                            Ok(_) => {}
                            Err(err) => {
                                // Only print a printing error once.
                                // In case of piping to something like `head`, it looks bad to print
                                // the same error tens or hundreds of times for a common pipe operation.
                                if !has_print_err {
                                    has_print_err = true;
                                    // BUG: Issue #3 colorization settings in the context of a pipe
                                    eprintln!("ERROR: failed to print {}", err);
                                }
                            }
                        }
                        if cli_opt_summary {
                            paths_printed_logmessages.insert(*pathid);
                            // update the per processing file `SummaryPrinted`
                            SummaryPrinted::summaryprint_map_update_utmpentry(utmpentry, pathid, &mut map_pathid_sumpr);
                            // update the single total program `SummaryPrinted`
                            summaryprinted.summaryprint_update_utmpentry(utmpentry);
                        }
                    }
                }
            }
            // create a copy of the borrowed key `pathid`, this avoids rustc error:
            //     cannot borrow `map_pathid_datum` as mutable more than once at a time
            let pathid_: PathId = *pathid;
            map_pathid_datum.remove(&pathid_);
            set_pathid.remove(&pathid_);
        }
        // remove channels (and keys) that are marked disconnected
        for pathid in disconnect.iter() {
            defo!("C map_pathid_chanrecvdatum.remove({:?});", pathid);
            map_pathid_chanrecvdatum.remove(pathid);
            defo!("C pathid_to_prependname.remove({:?});", pathid);
            pathid_to_prependname.remove(pathid);
        }
        // are there any channels to receive from?
        if map_pathid_chanrecvdatum.is_empty() {
            defo!("D map_pathid_chanrecvdatum.is_empty(); no more channels to receive from!");
            // all channels are closed, break from main processing loop
            break;
        }
        defo!("D map_pathid_chanrecvdatum: {:?}", map_pathid_chanrecvdatum);
        defo!("D map_pathid_datum: {:?}", map_pathid_datum);
        defo!("D set_pathid: {:?}", set_pathid);
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

    if cli_opt_summary {
        // some errors may occur later in processing, e.g. File Permissions errors,
        // so update `map_pathid_results` and `map_pathid_results_invalid`
        for (pathid, summary) in map_pathid_summary.iter() {
            match &summary.error {
                Some(_) => {
                    if ! summary.readerdata.is_dummy()
                        && summary.blockreader().blockreader_blocks == 0
                        && !map_pathid_results_invalid.contains_key(pathid)
                        && map_pathid_results.contains_key(pathid)
                    {
                        let result = map_pathid_results
                            .remove(pathid)
                            .unwrap();
                        map_pathid_results_invalid.insert(*pathid, result);
                    }
                }
                None => {}
            }
        }
        eprintln!();
        eprintln!("Files:");
        // print details about all the valid files
        print_all_files_summaries(
            &map_pathid_path,
            &map_pathid_filetype,
            &map_pathid_logmessagetype,
            &map_pathid_mimeguess,
            &map_pathid_color,
            &mut map_pathid_summary,
            &mut map_pathid_sumpr,
            &color_choice,
            &color_default,
        );
        if !map_pathid_path.is_empty(){
            eprintln!();
        }
        // print a short note about the invalid files
        print_files_processpathresult(
            &map_pathid_results_invalid,
            &color_choice,
            &color_default,
            &COLOR_ERROR,
        );
        eprintln!();
        eprintln!("Summary:");
        eprintln!(
            "Paths considered {}, paths not processed {}, files processed {}, files printed {}",
            paths_total,
            map_pathid_results_invalid.len(),
            map_pathid_results.len(),
            paths_printed_logmessages.len(),
        );
        // print the `summaryprinted` in a line-oriented manner and with
        // UTC comparisons
        eprintln!("Printed bytes   : {}", summaryprinted.bytes);
        eprintln!("Printed lines   : {}", summaryprinted.lines);
        eprintln!("Printed syslines: {}", summaryprinted.syslines);
        eprintln!("Printed utmpx   : {}", summaryprinted.utmpentries);

        let foffset0 = *FIXEDOFFSET0;
        eprint!("Datetime Filter -a    :");
        match filter_dt_after_opt {
            Some(dt) => {
                eprint!(" {:?}", dt);
                // convert `FixedOffset` to "UTC" (`FixedOffset` value `0`)
                let dt_utc = dt.with_timezone(&foffset0);
                eprintln!(" ({:?})", dt_utc);
            }
            None => eprintln!(),
        }
        eprint!("Datetime printed first:");
        match summaryprinted.dt_first {
            Some(dt) => {
                eprint!(" {:?}", dt);
                let utc_ = dt.with_timezone(&foffset0);
                eprintln!(" ({:?})", utc_);
            }
            None => eprintln!(),
        }
        eprint!("Datetime printed last :");
        match summaryprinted.dt_last {
            Some(dt) => {
                eprint!(" {:?}", dt);
                let utc_ = dt.with_timezone(&foffset0);
                eprintln!(" ({:?})", utc_);
            }
            None => eprintln!(),
        }
        eprint!("Datetime Filter -b    :");
        match filter_dt_before_opt {
            Some(dt) => {
                eprint!(" {:?}", dt);
                // convert `FixedOffset` to "UTC" (`FixedOffset` value `0`)
                let dt_utc = dt.with_timezone(&foffset0);
                eprintln!(" ({:?})", dt_utc);
            }
            None => eprintln!(),
        }
        // print the time now as this program sees it
        let local_now = Local
            .with_ymd_and_hms(
                LOCAL_NOW.year(),
                LOCAL_NOW.month(),
                LOCAL_NOW.day(),
                LOCAL_NOW.hour(),
                LOCAL_NOW.minute(),
                LOCAL_NOW.second(),
            )
            .unwrap();
        eprint!("Datetime Now          : {:?}", local_now);
        // print UTC now without fractional, and with numeric offset `-00:00`
        // instead of `Z`
        let utc_now = Utc
            .with_ymd_and_hms(
                UTC_NOW.year(),
                UTC_NOW.month(),
                UTC_NOW.day(),
                UTC_NOW.hour(),
                UTC_NOW.minute(),
                UTC_NOW.second(),
            )
            .unwrap()
            .with_timezone(&*FIXEDOFFSET0);
        eprintln!(" ({:?})", utc_now);
        // print crude stats
        eprintln!("Channel Receive ok {}, err {}", chan_recv_ok, chan_recv_err);
    } // cli_opt_summary

    defo!("E chan_recv_ok {:?} _count_recv_di {:?}", chan_recv_ok, chan_recv_err);

    // TODO: Issue #5 return code confusion
    //       the rationale for returning `false` (and then the process return code 1)
    //       is clunky, and could use a little refactoring. Also needs a gituhub Issue
    let mut ret: bool = true;
    if chan_recv_err > 0 {
        defo!("F chan_recv_err {}; return false", chan_recv_err);
        ret = false;
    }
    //if _fileprocessing_not_okay > 0 {
    //    defo!("F fileprocessing_not_okay {}; return false", _fileprocessing_not_okay);
    //    ret = false;
    //}
    if error_count > 0 {
        defo!("F error_count {}; return false", error_count);
        ret = false;
    }
    defx!("return {:?}", ret);

    ret
}

// -------------------------------------------------------------------------------------------------

/// Print the filepath name (one line).
fn print_filepath(
    path: &FPath,
    filetype: &FileType,
    logmessagetype: &LogMessageType,
    mimeguess: &MimeGuess,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprint!("File: ");
    // print path
    match print_colored_stderr(*color, Some(*color_choice), path.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: {:?}", err);
        }
    };
    // if symlink or relative path then print target
    // XXX: experimentation revealed std::fs::Metadata::is_symlink to be unreliable on WSL Ubuntu
    match std::fs::canonicalize(path) {
        Ok(pathb) => match pathb.to_str() {
            Some(s) => {
                if s != path.as_str() {
                    eprint!(" (");
                    write_stderr(s.as_bytes());
                    eprint!(")");
                }
            }
            None => {}
        },
        Err(_) => {}
    }
    // print other facts
    eprint!(" ({}) ({}) {:?}", filetype, logmessagetype, mimeguess);
    eprintln!();
}

/// Print the (optional) [`Summary`] (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_summary_opt_processed(summary_opt: &SummaryOpt) {
    let summary = match summary_opt {
        Some(summary) => {
            summary
        }
        None => {
            // TODO: [2022/06/07] print filesz
            eprintln!("{}Summary Processed: None", OPT_SUMMARY_PRINT_INDENT1);
            return;
        }
    };
    if summary.readerdata.is_dummy() {
        // `Dummy` may occur for files without adequate read permissions
        // there will be no interested information in gathered statistics
        return;
    }
    eprintln!("{}Summary Processed:", OPT_SUMMARY_PRINT_INDENT1);
    print_summary_opt_processed_summaryblockreader(
        summary,
        OPT_SUMMARY_PRINT_INDENT2,
    );
    match &summary.readerdata {
        // `Dummy` may occur for files without adequate read permissions
        SummaryReaderData::Dummy => return,
        SummaryReaderData::Syslog(
            (
                _summaryblockreader,
                summarylinereader,
                summarysyslinereader,
                _summarysyslogprocessor,
            ),
        ) => {
            eprintln!("{}lines          {}", OPT_SUMMARY_PRINT_INDENT2, summarylinereader.linereader_lines);
            eprintln!(
                "{}lines high     {}",
                OPT_SUMMARY_PRINT_INDENT2, summarylinereader.linereader_lines_stored_highest
            );
            eprintln!("{}syslines       {}", OPT_SUMMARY_PRINT_INDENT2, summarysyslinereader.syslinereader_syslines);
            eprintln!(
                "{}syslines high  {}",
                OPT_SUMMARY_PRINT_INDENT2, summarysyslinereader.syslinereader_syslines_stored_highest
            );
        }
        SummaryReaderData::Utmpx((
            _summaryblockreader,
            summaryutmpreader,
        )) => {
            eprintln!("{}utmpx          {}", OPT_SUMMARY_PRINT_INDENT2, summaryutmpreader.utmpxreader_utmp_entries);
            eprintln!(
                "{}utmpx high     {}",
                OPT_SUMMARY_PRINT_INDENT2, summaryutmpreader.utmpxreader_utmp_entries_max,
            );
        }
    }
    //eprintln!("{}lines          {}", OPT_SUMMARY_PRINT_INDENT2, summary.linereader_lines);
    //eprintln!(
    //    "{}lines high     {}",
    //    OPT_SUMMARY_PRINT_INDENT2, summary.linereader_lines_stored_highest
    //);
    // print datetime first and last
    match (summary.datetime_first(), summary.datetime_last()) {
        (Some(dt_first), Some(dt_last)) => {
            eprintln!("{}datetime first {:?}", OPT_SUMMARY_PRINT_INDENT2, dt_first,);
            eprintln!("{}datetime last  {:?}", OPT_SUMMARY_PRINT_INDENT2, dt_last,);
        }
        (None, Some(_)) | (Some(_), None) => {
            eprintln!("ERROR: only one of dt_first or dt_last fulfilled; this is unexpected.");
        }
        _ => {}
    }
    // print datetime patterns
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            _summaryblockreader,
            _summarylinereader,
            summarysyslinereader,
            summarysyslogprocessor,
        )) => {
            if !summarysyslinereader.syslinereader_patterns.is_empty()
            {
                eprintln!("{}Parsers:", OPT_SUMMARY_PRINT_INDENT1);
            }
            for patt in summarysyslinereader
                .syslinereader_patterns
                .iter()
            {
                let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*patt.0];
                eprintln!("{}@[{}] uses {} {:?}", OPT_SUMMARY_PRINT_INDENT2, patt.0, patt.1, dtpd);
            }
            match summarysyslogprocessor.SyslogProcessor_missing_year {
                Some(year) => {
                    eprintln!(
                        "{}datetime format missing year; estimated year of last sysline {:?}",
                        OPT_SUMMARY_PRINT_INDENT3, year
                    );
                }
                None => {}
            }
        }
        _ => {}
    }
}

/// helper to `print_summary_opt_processed`
fn print_summary_opt_processed_summaryblockreader(
    summary: &Summary,
    indent: &str,
) {
    if summary.readerdata.is_dummy() {
        return;
    }
    let summaryblockreader = summary.blockreader();
    match summary.filetype {
        FileType::File | FileType::Utmpx => {
            eprintln!(
                "{}file size      {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
        }
        FileType::Tar => {
            eprintln!(
                "{}file size archive    {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
            eprintln!(
                "{}file size unarchived {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz_actual
            );
        }
        FileType::Gz | FileType::Xz => {
            eprintln!(
                "{}file size compressed   {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
            eprintln!(
                "{}file size uncompressed {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz_actual
            );
        }
        ft => {
            eprintln!("{}unsupported filetype {:?}", OPT_SUMMARY_PRINT_INDENT2, ft);
            return;
        }
    }
    eprintln!("{}bytes          {1} (0x{1:X})", OPT_SUMMARY_PRINT_INDENT2, summaryblockreader.blockreader_bytes);
    eprintln!("{}bytes total    {1} (0x{1:X})", OPT_SUMMARY_PRINT_INDENT2, summaryblockreader.blockreader_bytes_total);
    eprintln!(
        "{}block size     {1} (0x{1:X})",
        OPT_SUMMARY_PRINT_INDENT2, summaryblockreader.blockreader_blocksz
    );
    eprintln!("{}blocks         {}", OPT_SUMMARY_PRINT_INDENT2, summaryblockreader.blockreader_blocks);
    eprintln!("{}blocks total   {}", OPT_SUMMARY_PRINT_INDENT2, summaryblockreader.blockreader_blocks_total);
    eprintln!("{}blocks high    {}", OPT_SUMMARY_PRINT_INDENT2, summaryblockreader.blockreader_blocks_highest);
}

/// Print the (optional) [`&SummaryPrinted`] (one line).
///
/// [`&SummaryPrinted`]: self::SummaryPrinted
fn print_summary_opt_printed(
    summary_print_opt: &SummaryPrintedOpt,
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_print_opt {
        Some(summary_print) => {
            defñ!("Some(summary_print)");
            
            summary_print.print_colored_stderr(
                Some(*color_choice),
                summary_opt,
                OPT_SUMMARY_PRINT_INDENT1,
                OPT_SUMMARY_PRINT_INDENT2,
            );
        }
        None => {
            defñ!("None");
            SummaryPrinted::default().print_colored_stderr(
                Some(*color_choice),
                summary_opt,
                OPT_SUMMARY_PRINT_INDENT1,
                OPT_SUMMARY_PRINT_INDENT2,
            );
        }
    }
}

fn ratio64(
    a: &u64,
    b: &u64,
) -> f64 {
    if b == &0 {
        return 0.0;
    }
    (*a as f64) / (*b as f64)
}

fn print_cache_stats_summaryblockreader(
    summaryblockreader: &SummaryBlockReader,
    indent: &str,
    wide: usize,
) {
    // BlockReader::_read_blocks
    let mut ratio = ratio64(&summaryblockreader.blockreader_read_blocks_hit, &summaryblockreader.blockreader_read_blocks_miss);
    eprintln!(
        "{}storage: BlockReader::read_block() blocks                    : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        indent,
        summaryblockreader.blockreader_read_blocks_hit,
        summaryblockreader.blockreader_read_blocks_miss,
        ratio,
        summaryblockreader.blockreader_read_blocks_put,
        wide = wide,
    );
    // BlockReader::_read_blocks_cache
    ratio = ratio64(
        &summaryblockreader.blockreader_read_block_lru_cache_hit,
        &summaryblockreader.blockreader_read_block_lru_cache_miss,
    );
    eprintln!(
        "{}caching: BlockReader::read_block() LRU cache                 : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        indent,
        summaryblockreader.blockreader_read_block_lru_cache_hit,
        summaryblockreader.blockreader_read_block_lru_cache_miss,
        ratio,
        summaryblockreader.blockreader_read_block_lru_cache_put,
        wide = wide,
    );
}

fn print_cache_stats_summarylinereader(
    summarylinereader: &SummaryLineReader,
    indent: &str,
    wide: usize,
) {
    // LineReader::_lines
    let mut ratio = ratio64(&summarylinereader.linereader_lines_hits, &summarylinereader.linereader_lines_miss);
    eprintln!(
        "{}storage: LineReader::find_line() lines                       : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        indent,
        summarylinereader.linereader_lines_hits,
        summarylinereader.linereader_lines_miss,
        ratio,
        wide = wide,
    );
    // LineReader::_find_line_lru_cache
    ratio =
        ratio64(&summarylinereader.linereader_find_line_lru_cache_hit, &summarylinereader.linereader_find_line_lru_cache_miss);
    eprintln!(
        "{}caching: LineReader::find_line() LRU cache                   : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        indent,
        summarylinereader.linereader_find_line_lru_cache_hit,
        summarylinereader.linereader_find_line_lru_cache_miss,
        ratio,
        summarylinereader.linereader_find_line_lru_cache_put,
        wide = wide,
    );
}

fn print_cache_stats_summarysyslinereader(
    summarysyslinereader: &SummarySyslineReader,
    indent: &str,
    wide: usize,
) {
    // SyslineReader
    // SyslineReader::get_boxptrs
    eprintln!(
        "{}copying: SyslineReader::get_boxptrs()                        : sgl {:wide$}, dbl  {:wide$}, mult {:wide$}",
        indent,
        summarysyslinereader.syslinereader_get_boxptrs_singleptr,
        summarysyslinereader.syslinereader_get_boxptrs_doubleptr,
        summarysyslinereader.syslinereader_get_boxptrs_multiptr,
        wide = wide,
    );
    // SyslineReader::syslines
    let mut ratio = ratio64(&summarysyslinereader.syslinereader_syslines_hit, &summarysyslinereader.syslinereader_syslines_miss);
    eprintln!(
        "{}storage: SyslineReader::find_sysline() syslines              : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        indent,
        summarysyslinereader.syslinereader_syslines_hit,
        summarysyslinereader.syslinereader_syslines_miss,
        ratio,
        wide = wide,
    );
    // SyslineReader::_syslines_by_range
    ratio =
        ratio64(&summarysyslinereader.syslinereader_syslines_by_range_hit, &summarysyslinereader.syslinereader_syslines_by_range_miss);
    eprintln!(
        "{}caching: SyslineReader::find_sysline() syslines_by_range_map : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_syslines_by_range_hit,
        summarysyslinereader.syslinereader_syslines_by_range_miss,
        ratio,
        summarysyslinereader.syslinereader_syslines_by_range_put,
        wide = wide,
    );
    // SyslineReader::_find_sysline_lru_cache
    ratio = ratio64(
        &summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
        &summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::find_sysline() LRU cache             : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
        ratio,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_put,
        wide = wide,
    );
    // SyslineReader::_parse_datetime_in_line_lru_cache
    ratio = ratio64(
        &summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
        &summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::parse_datetime_in_line() LRU cache   : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
        ratio,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_put,
        wide = wide,
    );
}

fn print_cache_stats_summaryutmpreader(
    summaryutmpreader: &SummaryUtmpxReader,
    indent: &str,
    wide: usize,
) {
    let ratio = ratio64(
        &summaryutmpreader.utmpxreader_utmp_entries_hit,
        &summaryutmpreader.utmpxreader_utmp_entries_miss,
    );
    eprintln!(
        "{}storage: UtmpxReader::find_entry()                           : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        indent,
        summaryutmpreader.utmpxreader_utmp_entries_hit,
        summaryutmpreader.utmpxreader_utmp_entries_miss,
        ratio,
        wide = wide,
    );
}

/// Print the various (optional) [`Summary`] caching and storage statistics
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_cache_stats(summary_opt: &SummaryOpt) {
    if summary_opt.is_none() {
        return;
    }
    let summary: &Summary = match summary_opt.as_ref() {
        Some(summary_) => summary_,
        None => {
            eprintln!("ERROR: unexpected None from match summary_opt");
            return;
        }
    };
    // `Dummy` may occur for files without adequate read permissions
    if summary.readerdata.is_dummy() {
        return;
    }
    eprintln!("{}Processing Stores:", OPT_SUMMARY_PRINT_INDENT1);
    let wide: usize = summary
        .max_hit_miss()
        .to_string()
        .len();
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            summaryblockreader,
            summarylinereader,
            summarysyslinereader,
            _summarysyslogprocessor,
        )) => {
            print_cache_stats_summarysyslinereader(
                summarysyslinereader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summarylinereader(
                summarylinereader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summaryblockreader(
                summaryblockreader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
        }
        SummaryReaderData::Utmpx((
            summaryblockreader,
            summaryutmpreader,
        )) => {
            print_cache_stats_summaryutmpreader(
                summaryutmpreader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summaryblockreader(
                summaryblockreader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
        }
        SummaryReaderData::Dummy => panic!("Unexpected SummaryReaderData::Dummy"),
    }
}

/// Print the (optional) various [`Summary`] drop error statistics
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_drop_stats(summary_opt: &SummaryOpt) {
    let summary: &Summary = match summary_opt {
        Some(ref summary) => summary,
        None => {
            return;
        }
    };
    if summary.readerdata.is_dummy() {
        return;
    }
    eprintln!("{}Processing Drops:", OPT_SUMMARY_PRINT_INDENT1);
    let wide: usize = summary
        .max_drop()
        .to_string()
        .len();
    let summaryblockreader = summary.blockreader();
    eprintln!(
        "{}streaming: BlockReader::drop_block()    : Ok {:wide$} Err {:wide$}",
        OPT_SUMMARY_PRINT_INDENT2,
        summaryblockreader.blockreader_blocks_dropped_ok,
        summaryblockreader.blockreader_blocks_dropped_err,
        wide = wide,
    );
    match &summary.readerdata {
        SummaryReaderData::Syslog(
            (
                _summaryblockreader,
                summarylinereader,
                summarysyslinereader,
                _summarysyslogreader,
            )
        ) => {
            eprintln!(
                "{}streaming: SyslineReader::drop_sysline(): Ok {:wide$} Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summarysyslinereader.syslinereader_drop_sysline_ok,
                summarysyslinereader.syslinereader_drop_sysline_errors,
                wide = wide,
            );
            eprintln!(
                "{}streaming: LineReader::drop_line()      : Ok {:wide$} Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summarylinereader.linereader_drop_line_ok,
                summarylinereader.linereader_drop_line_errors,
                wide = wide,
            );
        }
        SummaryReaderData::Utmpx(
            (
                _summaryblockreader,
                summaryutmpreader,
            )
        ) => {
            eprintln!(
                "{}streaming: UtmpxReader::drop_entry()    : Ok {:wide$} Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summaryutmpreader.utmpxreader_drop_entry_ok,
                summaryutmpreader.utmpxreader_drop_entry_errors,
                wide = wide,
            );
        }
        SummaryReaderData::Dummy => panic!("Unexpected SummaryReaderData::Dummy"),
    }
}

/// Print the [`Summary.error`], if any (one line).
///
/// [`Summary.error`]: s4lib::readers::summary::Summary
fn print_error_summary(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_opt.as_ref() {
        Some(summary_) => match &summary_.error {
            Some(err_string) => {
                eprint!("{}Error: ", OPT_SUMMARY_PRINT_INDENT1);
                #[allow(clippy::single_match)]
                match print_colored_stderr(COLOR_ERROR, Some(*color_choice), err_string.as_bytes()) {
                    Err(_err) => {}
                    _ => {}
                }
                eprintln!();
            }
            None => {}
        },
        None => {}
    }
}

/// For one file, print the [`Summary`] and [`SummaryPrinted`]
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
/// [`SummaryPrinted`]: self::SummaryPrinted
#[allow(clippy::too_many_arguments)]
fn print_file_summary(
    path: &FPath,
    filetype: &FileType,
    logmessagetype: &LogMessageType,
    mimeguess: &MimeGuess,
    summary_opt: &SummaryOpt,
    summary_print_opt: &SummaryPrintedOpt,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprintln!();
    print_filepath(path, filetype, logmessagetype, mimeguess, color, color_choice);
    print_summary_opt_printed(summary_print_opt, summary_opt, color_choice);
    print_summary_opt_processed(summary_opt);
    if OPT_SUMMARY_PRINT_CACHE_STATS {
        print_cache_stats(summary_opt);
    }
    if OPT_SUMMARY_PRINT_DROP_STATS {
        print_drop_stats(summary_opt);
    }
    print_error_summary(summary_opt, color_choice);
}

/// Printing for CLI option `--summary`. Print each files'
/// [`Summary`] and [`SummaryPrinted`].
///
/// Helper function to function `processing_loop`.
///
/// [`Summary`]: s4lib::readers::summary::Summary
/// [`SummaryPrinted`]: self::SummaryPrinted
#[allow(clippy::too_many_arguments)]
fn print_all_files_summaries(
    map_pathid_path: &MapPathIdToFPath,
    map_pathid_filetype: &MapPathIdToFileType,
    map_pathid_logmessagetype: &MapPathIdToLogMessageType,
    map_pathid_mimeguess: &MapPathIdToMimeGuess,
    map_pathid_color: &MapPathIdToColor,
    map_pathid_summary: &mut MapPathIdSummary,
    map_pathid_sumpr: &mut MapPathIdSummaryPrint,
    color_choice: &ColorChoice,
    color_default: &Color,
) {
    for (pathid, path) in map_pathid_path.iter() {
        let color: &Color = map_pathid_color
            .get(pathid)
            .unwrap_or(color_default);
        let filetype: &FileType = map_pathid_filetype
            .get(pathid)
            .unwrap_or(&FileType::Unknown);
        let logmessagetype: &LogMessageType = map_pathid_logmessagetype
            .get(pathid)
            .unwrap_or(&LogMessageType::Sysline);
        let mimeguess_default: MimeGuess = MimeGuess::from_ext("");
        let mimeguess: &MimeGuess = map_pathid_mimeguess
            .get(pathid)
            .unwrap_or(&mimeguess_default);
        let summary_opt: SummaryOpt = map_pathid_summary.remove(pathid);
        let summary_print_opt: SummaryPrintedOpt = map_pathid_sumpr.remove(pathid);
        print_file_summary(path, filetype, logmessagetype, mimeguess, &summary_opt, &summary_print_opt, color, color_choice);
    }
}

/// Printing for CLI option `--summary`; print an entry for invalid files.
///
/// Helper function to function `processing_loop`.
fn print_files_processpathresult(
    map_pathid_result: &MapPathIdToProcessPathResult,
    color_choice: &ColorChoice,
    color_default: &Color,
    color_error: &Color,
) {
    // local helper
    fn print_(
        buffer: String,
        color_choice: &ColorChoice,
        color: &Color,
    ) {
        match print_colored_stderr(*color, Some(*color_choice), buffer.as_bytes()) {
            Ok(()) => {}
            Err(err) => {
                eprintln!("ERROR: {:?}", err);
            }
        };
    }

    for (_pathid, result) in map_pathid_result.iter() {
        match result {
            ProcessPathResult::FileValid(path, mimeguess, _filetype) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
            }
            ProcessPathResult::FileErrNoPermissions(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(no permissions)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotSupported(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not supported)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotAFile(path) => {
                print_(format!("File: {} ", path), color_choice, color_default);
                print_("(not a file)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotExist(path) => {
                print_(format!("File: {} ", path), color_choice, color_default);
                print_("(does not exist)".to_string(), color_choice, color_error);
            }
        }
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    extern crate test_case;
    use s4lib::data::datetime::DateTime;
    use test_case::test_case;

    use super::{
        cli_parse_blocksz, cli_parser_prepend_dt_format, cli_process_blocksz, cli_process_tz_offset,
        process_dt, BlockSz, DateTimeLOpt, CLI_OPT_PREPEND_FMT, unescape,
    };
    use super::{
        string_wdhms_to_duration, Duration, FixedOffset, TimeZone, DUR_OFFSET_TYPE, FIXEDOFFSET0, UTC_NOW,
    };

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

    #[test_case("+00", *FIXEDOFFSET0; "+00 east(0)")]
    #[test_case("+0000", *FIXEDOFFSET0; "+0000 east(0)")]
    #[test_case("+00:00", *FIXEDOFFSET0; "+00:00 east(0)")]
    #[test_case("+00:01", FixedOffset::east_opt(60).unwrap(); "+00:01 east(60)")]
    #[test_case("+01:00", FixedOffset::east_opt(3600).unwrap(); "+01:00 east(3600) A")]
    #[test_case("-01:00", FixedOffset::east_opt(-3600).unwrap(); "-01:00 east(-3600) B")]
    #[test_case("+02:00", FixedOffset::east_opt(7200).unwrap(); "+02:00 east(7200)")]
    #[test_case("+02:30", FixedOffset::east_opt(9000).unwrap(); "+02:30 east(9000)")]
    #[test_case("+02:35", FixedOffset::east_opt(9300).unwrap(); "+02:30 east(9300)")]
    #[test_case("+23:00", FixedOffset::east_opt(82800).unwrap(); "+23:00 east(82800)")]
    #[test_case("UTC", *FIXEDOFFSET0; "UTC east(0)")]
    #[test_case("vlat", FixedOffset::east_opt(36000).unwrap(); "vlat east(36000)")]
    #[test_case("IDLW", FixedOffset::east_opt(-43200).unwrap(); "IDLW east(-43200)")]
    fn test_cli_process_tz_offset(in_: &str, out_fo: FixedOffset) {
        let input: String = String::from(in_);
        let result = cli_process_tz_offset(&input);
        match result {
            Ok(fo) => {
                assert_eq!(out_fo, fo, "cli_process_tz_offset returned FixedOffset {:?}, expected {:?}", fo, out_fo);
            }
            Err(err) => {
                panic!("Error {:?}", err);
            }
        }
    }

    #[test_case("")]
    #[test_case("abc")]
    #[test_case(CLI_OPT_PREPEND_FMT)]
    #[test_case("%Y%Y%Y%Y%Y%Y%Y%%%%")]
    fn test_cli_parser_prepend_dt_format(prepend_dt_format: &str) {
        assert!(cli_parser_prepend_dt_format(prepend_dt_format).is_ok());
    }

    #[test_case(
        Some(String::from("2000-01-02T03:04:05")), *FIXEDOFFSET0,
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "2000-01-02T03:04:05"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05 -0100")), *FIXEDOFFSET0,
        Some(FixedOffset::east_opt(-3600).unwrap().with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap());
        "2000-01-02T03:04:05 -0100"
    )]
    fn test_process_dt(
        dts: Option<String>,
        tz_offset: FixedOffset,
        expect: DateTimeLOpt,
    ) {
        assert_eq!(process_dt(dts, &tz_offset, &None, &UTC_NOW), expect);
    }

    #[test_case(
        Some(String::from("@+1s")),
        *FIXEDOFFSET0,
        FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 6).unwrap());
        "2000-01-02T03:04:05 add 1s"
    )]
    #[test_case(
        Some(String::from("@-1s")),
        *FIXEDOFFSET0,
        FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 5).unwrap(),
        Some(FIXEDOFFSET0.with_ymd_and_hms(2000, 1, 2, 3, 4, 4).unwrap());
        "2000-01-02T03:04:04 add 1s"
    )]
    #[test_case(
        Some(String::from("@+4h1d")),
        *FIXEDOFFSET0,
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
        dt_other: DateTime<FixedOffset>,
        expect: DateTimeLOpt,
    ) {
        assert_eq!(process_dt(dts, &tz_offset, &Some(dt_other), &UTC_NOW), expect);
    }

    const NOW: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Now;
    const OTHER: DUR_OFFSET_TYPE = DUR_OFFSET_TYPE::Other;

    #[test_case(String::from(""), None)]
    #[test_case(String::from("1s"), None; "1s")]
    #[test_case(String::from("@1s"), None; "at_1s")]
    #[test_case(String::from("-0s"), Some((Duration::seconds(0), NOW)))]
    #[test_case(String::from("@+0s"), Some((Duration::seconds(0), OTHER)))]
    #[test_case(String::from("-1s"), Some((Duration::seconds(-1), NOW)); "minus_1s")]
    #[test_case(String::from("+1s"), Some((Duration::seconds(1), NOW)); "plus_1s")]
    #[test_case(String::from("@-1s"), Some((Duration::seconds(-1), OTHER)); "at_minus_1s")]
    #[test_case(String::from("@+1s"), Some((Duration::seconds(1), OTHER)); "at_plus_1s")]
    #[test_case(String::from("@+9876s"), Some((Duration::seconds(9876), OTHER)); "other_plus_9876")]
    #[test_case(String::from("@-9876s"), Some((Duration::seconds(-9876), OTHER)); "other_minus_9876")]
    #[test_case(String::from("-9876s"), Some((Duration::seconds(-9876), NOW)); "now_minus_9876")]
    #[test_case(String::from("-3h"), Some((Duration::hours(-3), NOW)))]
    #[test_case(String::from("-4d"), Some((Duration::days(-4), NOW)))]
    #[test_case(String::from("-5w"), Some((Duration::weeks(-5), NOW)))]
    #[test_case(String::from("@+5w"), Some((Duration::weeks(5), OTHER)))]
    #[test_case(String::from("-2m1s"), Some((Duration::seconds(-1) + Duration::minutes(-2), NOW)); "minus_2m1s")]
    #[test_case(String::from("-2d1h"), Some((Duration::hours(-1) + Duration::days(-2), NOW)); "minus_2d1h")]
    #[test_case(String::from("+2d1h"), Some((Duration::hours(1) + Duration::days(2), NOW)); "plus_2d1h")]
    #[test_case(String::from("@+2d1h"), Some((Duration::hours(1) + Duration::days(2), OTHER)); "at_plus_2d1h")]
    // "reverse" order should not matter
    #[test_case(String::from("-1h2d"), Some((Duration::hours(-1) + Duration::days(-2), NOW)); "minus_1h2d")]
    #[test_case(String::from("-4w3d2m1s"), Some((Duration::seconds(-1) + Duration::minutes(-2) + Duration::days(-3) + Duration::weeks(-4), NOW)))]
    // "mixed" order should not matter
    #[test_case(String::from("-3d4w1s2m"), Some((Duration::seconds(-1) + Duration::minutes(-2) + Duration::days(-3) + Duration::weeks(-4), NOW)))]
    // repeat values; only last is used
    #[test_case(String::from("-6w5w4w"), Some((Duration::weeks(-4), NOW)))]
    // repeat values; only last is used
    #[test_case(String::from("-5w4w3d2m1s"), Some((Duration::seconds(-1) + Duration::minutes(-2) + Duration::days(-3) + Duration::weeks(-4), NOW)))]
    // repeat values; only last is used
    #[test_case(String::from("-6w5w4w3d2m1s"), Some((Duration::seconds(-1) + Duration::minutes(-2) + Duration::days(-3) + Duration::weeks(-4), NOW)))]
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
    fn test_unescape_str (input: &str, expect: Option<&str>) {
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
