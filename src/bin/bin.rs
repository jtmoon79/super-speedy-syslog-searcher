// src/bin/bin.rs
//
// ‥ … ≤ ≥ ≠ ≟

//! Driver program _s4_.
//!
//! Processes user-passed command-line arguments.
//! Then processes paths passed; directories are enumerated for parseable files,
//! archive files (`.tar`) are enumerated for file entries, other
//! paths tested for suitability (readable? is it a file? etc.).
//!
//! For each parseable file found, a file processing thread is created.
//! Each file processing thread advances through the stages of processing
//! using a [`SyslogProcessor`] instance.
//!
//! During the main processing stage, [`Stage3StreamSyslines`], each thread
//! sends the last processed [`Sysline`] to the main processing thread.
//! The main processing thread compares the last [`DateTimeL`] received
//! from all processing threads.
//! The `Sysline` with the earliest `DateTimeL` is printed.
//! That file processing thread then processes another `Sysline`.
//! This continues until each file processing thread sends a message to the
//! main processing thread that is has completed processing,
//! or in case of errors, abruptly closes it's [sending channel].
//!
//! Then, if passed CLI option `--summary`, the main processing thread
//! prints a [`Summary`] about each file processed, and one [`SummaryPrinted`].
//!
//! [`Stage3StreamSyslines`]: s4lib::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
//! [`DateTimeL`]: s4lib::data::datetime::DateTimeL
//! [`Sysline`]: s4lib::data::sysline::Sysline
//! [sending channel]: self::ChanSendDatum
//! [`SyslogProcessor`]: s4lib::readers::syslogprocessor::SyslogProcessor
//! [`Summary`]: s4lib::readers::summary::Summary
//! [`SummaryPrinted`]: self::SummaryPrinted

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::process::ExitCode;
use std::str;
use std::thread;

extern crate chrono;
use chrono::{FixedOffset, Local, TimeZone};

extern crate clap;
use clap::{ArgEnum, Parser};

extern crate const_format;
use const_format::concatcp;

extern crate crossbeam_channel;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate si_trace_print;
use si_trace_print::{dpfn, dpfo, dpfx, dpfñ, dpn, dpo, stack::stack_offset_set};

extern crate unicode_width;

// `s4lib` is the local compiled `[lib]` of super_speedy_syslog_searcher
extern crate s4lib;

use s4lib::common::{Count, FPath, FPaths, FileOffset, FileType, NLu8a};

use s4lib::data::datetime::{
    datetime_parse_from_str, datetime_parse_from_str_w_tz, DateTimeLOpt, DateTimeParseInstr, DateTimePattern_str, DATETIME_PARSE_DATAS, MAP_TZZ_TO_TZz,
    Utc,
};

#[allow(unused_imports)]
use s4lib::debug::printers::{dp_err, dp_wrn, p_err, p_wrn};

use s4lib::printer::printers::{
    color_rand,
    print_colored_stderr,
    write_stdout,
    // termcolor imports
    Color,
    ColorChoice,
    PrinterSysline,
    //
    COLOR_DEFAULT,
    COLOR_ERROR,
};

use s4lib::data::sysline::{SyslineP, SyslineP_Opt};

use s4lib::readers::blockreader::{BlockSz, BLOCKSZ_DEF, BLOCKSZ_MAX, BLOCKSZ_MIN};

use s4lib::readers::filepreprocessor::{process_path, ProcessPathResult, ProcessPathResults};

use s4lib::readers::helpers::basename;

use s4lib::readers::summary::{Summary, SummaryOpt};

use s4lib::readers::syslinereader::ResultS3SyslineFind;

use s4lib::readers::syslogprocessor::{FileProcessingResultBlockZero, SyslogProcessor};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
    ArgEnum, // from `clap`
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

/// CLI time to append in `fn process_dt` when `has_time` is `false`.
const CLI_DT_FILTER_APPEND_TIME_VALUE: &str = " T000000";

/// CLI strftime format pattern to append in function `process_dt`
/// when `has_time` is `false`.
const CLI_DT_FILTER_APPEND_TIME_PATTERN: &str = " T%H%M%S";

/// default CLI datetime format printed for CLI options `-u` or `-l`.
const CLI_OPT_PREPEND_FMT: &str = "%Y%m%dT%H%M%S%.3f%z:";

/// `--help` _afterword_ message.
const CLI_HELP_AFTER: &str = concatcp!(
    "
DateTime Filter strftime specifier patterns may be:
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

Pattern \"+%s\" is Unix epoch timestamp in seconds with a preceding \"+\".
Without a timezone offset (\"%z\" or \"%Z\"), the Datetime Filter is presumed to be the local system
timezone.
Ambiguous user-passed named timezones will be rejected, e.g. \"SST\".

DateTime strftime specifier patterns are described at https://docs.rs/chrono/latest/chrono/format/strftime/

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
    //author,
    version,
    about,
    after_help = CLI_HELP_AFTER,
    //before_help = "",
    setting = clap::AppSettings::DeriveDisplayOrder,
)]
struct CLI_Args {
    /// Path(s) of syslog files or directories.
    /// Directories will be recursed, remaining on the same filesystem.
    /// Symlinks will be followed.
    #[clap(required = true)]
    paths: Vec<String>,

    /// DateTime Filter after.
    #[clap(
        short = 'a',
        long,
        help = "DateTime After filter - print syslog lines with a datetime that is at or after this datetime. For example, \"20200102T123000\""
    )]
    dt_after: Option<String>,

    /// DateTime Filter before.
    #[clap(
        short = 'b',
        long,
        help = "DateTime Before filter - print syslog lines with a datetime that is at or before this datetime. For example, \"20200102T123001\""
    )]
    dt_before: Option<String>,

    /// Default timezone offset for datetimes without a timezone.
    #[clap(
        short = 't',
        long,
        help = "DateTime Timezone offset - for syslines with a datetime that does not include a timezone, this will be used. For example, \"-0800\", \"+02:00\", or \"EDT\". Ambiguous named timezones parsed from logs will use this value, e.g. timezone \"IST\". (to pass a value with leading \"-\", use \", e.g. \"-t=-0800\"). Default is local system timezone offset.",
        validator = cli_validate_tz_offset,
        default_value_t=Local.timestamp(0, 0).offset().to_string(),
    )]
    tz_offset: String,

    /// Prepend DateTime in the UTC Timezone for every line.
    #[clap(
        short = 'u',
        long = "prepend-utc",
        group = "prepend_dt"
    )]
    prepend_utc: bool,

    /// Prepend DateTime in the Local Timezone for every line.
    #[clap(
        short = 'l',
        long = "prepend-local",
        group = "prepend_dt"
    )]
    prepend_local: bool,

    /// Prepend DateTime using strftime format string.
    #[clap(
        short = 'd',
        long = "prepend-dt-format",
        group = "prepend_dt_format",
        requires = "prepend_dt",
        validator = cli_validate_prepend_dt_format,
        default_value_t = String::from(CLI_OPT_PREPEND_FMT),
    )]
    prepend_dt_format: String,

    /// Prepend file basename to every line.
    #[clap(
        short = 'n',
        long = "prepend-filename",
        group = "prepend_file"
    )]
    prepend_filename: bool,

    /// Prepend file full path to every line.
    #[clap(
        short = 'p',
        long = "prepend-filepath",
        group = "prepend_file"
    )]
    prepend_filepath: bool,

    /// Align column widths of prepended data.
    #[clap(
        short = 'w',
        long = "prepend-file-align"
    )]
    prepend_file_align: bool,

    /// Choose to print to terminal using colors.
    #[clap(
        required = false,
        short = 'c',
        long = "--color",
        arg_enum,
        default_value_t=CLI_Color_Choice::auto,
    )]
    color_choice: CLI_Color_Choice,

    /// Read blocks of this size in bytes.
    /// May pass decimal or hexadecimal numbers.
    /// Using the default value is recommended.
    /// Most useful for developers.
    #[clap(
        required = false,
        short = 'z',
        long,
        default_value_t = BLOCKSZ_DEF.to_string(),
        validator = cli_validate_blocksz,
    )]
    blocksz: String,

    /// Print a summary of files processed to stderr.
    /// Most useful for developers.
    #[clap(short, long)]
    summary: bool,
}

/// CLI argument processing.
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

/// `clap` argument validator for `--blocksz`.
///
/// See <https://github.com/clap-rs/clap/blob/v3.1.6/examples/tutorial_derive/04_02_validate.rs>
fn cli_validate_blocksz(blockszs: &str) -> clap::Result<(), String> {
    match cli_process_blocksz(&String::from(blockszs)) {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    }
    Ok(())
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
        },
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
        let dt = datetime_parse_from_str_w_tz(
            data.as_str(), pattern,
        );
        dpfo!("datetime_parse_from_str_w_tz({:?}, {:?}) returned {:?}", data, pattern, dt);
        match dt {
            Some(dt_) => {
                dpfx!("return {:?}", dt_.offset());
                return Ok(*dt_.offset());
            }
            None => {}
        }
    };

    Err(
        format!("Unable to parse a timezone offset for --tz-offset {:?}", tzo)
    )
}

/// `clap` argument validator for `--tz-offset`.
fn cli_validate_tz_offset(tz_offset: &str) -> std::result::Result<(), String> {
    match cli_process_tz_offset(tz_offset) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

/// `clap` argument validator for `--prepend-dt-format`.
fn cli_validate_prepend_dt_format(prepend_dt_format: &str) -> std::result::Result<(), String> {
    let dt = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0);
    dt.format(prepend_dt_format);

    Ok(())
}

/// Transform a user-passed datetime `String` into a [`DateTimeL`].
///
/// Helper function to function `cli_process_args`.
///
/// [`DateTimeL`]: s4lib::data::datetime::DateTimeL
fn process_dt(
    dts: Option<String>,
    tz_offset: &FixedOffset,
) -> DateTimeLOpt {
    // parse datetime filters
    match dts {
        Some(dts) => {
            let mut dto: DateTimeLOpt = None;
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
                    dpfo!(
                        "appended {:?}, {:?}",
                        CLI_DT_FILTER_APPEND_TIME_VALUE,
                        CLI_DT_FILTER_APPEND_TIME_PATTERN
                    );
                }
                dpfo!("datetime_parse_from_str({:?}, {:?}, {:?}, {:?})", dts_, pattern, has_tz, tz_offset);
                if let Some(val) =
                    datetime_parse_from_str(dts_.as_str(), pattern.as_str(), *has_tz, tz_offset)
                {
                    dto = Some(val);
                    break;
                };
            }
            if dto.is_none() {
                eprintln!("ERROR: Unable to parse a datetime from {:?}", dts);
                std::process::exit(1);
            }

            dto
        }
        None => None,
    }
}

/// Process user-passed CLI argument strings into expected types.
///
/// This function will [`std::process::exit`] if there is an [`Err`].
fn cli_process_args(
) -> (FPaths, BlockSz, DateTimeLOpt, DateTimeLOpt, FixedOffset, ColorChoice, bool, bool, String, bool, bool, bool, bool)
{
    let args = CLI_Args::parse();

    dpfo!("args {:?}", args);

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
    dpfo!("blocksz {:?}", blocksz);

    let mut fpaths: Vec<FPath> = Vec::<FPath>::new();
    for path in args.paths.iter() {
        fpaths.push(path.clone());
    }

    let tz_offset: FixedOffset = match cli_process_tz_offset(&args.tz_offset) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    };
    dpfo!("tz_offset {:?}", tz_offset);

    let filter_dt_after: DateTimeLOpt = process_dt(args.dt_after, &tz_offset);
    dpfo!("filter_dt_after {:?}", filter_dt_after);
    let filter_dt_before: DateTimeLOpt = process_dt(args.dt_before, &tz_offset);
    dpfo!("filter_dt_before {:?}", filter_dt_before);

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

    dpfo!("color_choice {:?}", color_choice);
    dpfo!("prepend_utc {:?}", args.prepend_utc);
    dpfo!("prepend_local {:?}", args.prepend_local);
    dpfo!("prepend_dt_format {:?}", args.prepend_dt_format);
    dpfo!("prepend_filename {:?}", args.prepend_filename);
    dpfo!("prepend_filepath {:?}", args.prepend_filepath);
    dpfo!("prepend_file_align {:?}", args.prepend_file_align);
    dpfo!("summary {:?}", args.summary);

    (
        fpaths,
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
    dpfn!();
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
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_opt_summary,
    ) = cli_process_args();

    let mut processed_paths: ProcessPathResults = ProcessPathResults::with_capacity(paths.len() * 4);
    for fpath in paths.iter() {
        let ppaths: ProcessPathResults = process_path(fpath);
        for ppresult in ppaths.into_iter() {
            processed_paths.push(ppresult);
        }
        /*
        // TODO: [2022/06/06] carry forward invalid paths for printing with the `--summary`
        // XXX: can this be done in a one-liner?
        for processpathresult in ppaths.iter()
            .filter(|x| matches!(x,  ProcessPathResult::FileValid(_)))
        {
            let path: FPath = match filetype_path {
                ProcessPathResult::FileValid(val) => val.1,
                _ => { continue; },
            };
            processed_paths.push(path.clone());
        }
        */
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
        cli_opt_summary,
    );

    dpfx!();

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
type ThreadInitData = (FPath, PathId, FileType, BlockSz, DateTimeLOpt, DateTimeLOpt, FixedOffset);

/// Is this the last [`Sysline`] of the syslog file?
///
/// [`Sysline`]: s4lib::data::sysline::Sysline
type IsSyslineLast = bool;

/// The data sent from file processing thread to the main printing thread.
///
/// * optional `Sysline`
/// * optional `Summary`
/// * is this the last Sysline?
/// * `FileProcessingResult`
///
/// There should never be a `Sysline` and a `Summary` received simultaneously.
// TODO: Enforce that `Sysline` and `Summary` exclusivity with some kind of union.
type ChanDatum = (SyslineP_Opt, SummaryOpt, IsSyslineLast, FileProcessingResultBlockZero);

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

/// Thread entry point for processing one file.
///
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
fn exec_syslogprocessor_thread(
    chan_send_dt: ChanSendDatum,
    thread_init_data: ThreadInitData,
) {
    if cfg!(debug_assertions) {
        stack_offset_set(Some(2));
    }
    let (path, _pathid, filetype, blocksz, filter_dt_after_opt, filter_dt_before_opt, tz_offset) =
        thread_init_data;
    dpfn!("({:?})", path);

    #[cfg(any(debug_assertions,test))]
    let thread_cur: thread::Thread = thread::current();
    #[cfg(any(debug_assertions,test))]
    let tid: thread::ThreadId = thread_cur.id();
    #[cfg(any(debug_assertions,test))]
    let tname: &str = <&str>::clone(
        &thread_cur
            .name()
            .unwrap_or(""),
    );

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
            dp_err!("SyslogProcessor::new({:?}) failed {}", path.as_str(), err);
            let mut summary = Summary::default();
            // TODO: [2022/08] this design needs work: the Error instance should be passed
            //       back in the channel, not via the Summary. The `FileProcessResult`
            //       should travel inside or outside the `Summary`. Needs consideration.
            summary.Error_ = Some(err.to_string());
            match chan_send_dt.send((None, Some(summary), true, FILEERRSTUB)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            dpfx!("({:?})", path);
            return;
        }
    };
    dpfo!("syslogproc {:?}", syslogproc);

    syslogproc.process_stage0_valid_file_check();

    let result = syslogproc.process_stage1_blockzero_analysis();
    match &result {
        FileProcessingResultBlockZero::FileErrNoLinesFound => {
            eprintln!("WARNING: no lines found {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrNoSyslinesFound => {
            eprintln!("WARNING: no syslines found {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrDecompress => {
            eprintln!("WARNING: could not decompress {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrWrongType => {
            eprintln!("WARNING: bad path {:?}", path);
        }
        FileProcessingResultBlockZero::FileErrIo(err) => {
            eprintln!("ERROR: Error {} for {:?}", err, path);
        }
        FileProcessingResultBlockZero::FileOk => {}
        FileProcessingResultBlockZero::FileErrEmpty => {}
        FileProcessingResultBlockZero::FileErrNoSyslinesInDtRange => {}
        FileProcessingResultBlockZero::FileErrStub => {}
    }
    match result {
        FileProcessingResultBlockZero::FileOk => {}
        _ => {
            dpfo!("chan_send_dt.send((None, summary, true))");
            match chan_send_dt.send((None, Some(syslogproc.summary()), true, result)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: stage1 chan_send_dt.send(…) failed {}", err);
                }
            }
            dpfx!("({:?})", path);
            return;
        }
    }

    // find first sysline acceptable to the passed filters
    match syslogproc.process_stage2_find_dt() {
        FileProcessingResultBlockZero::FileOk => {}
        _result => {
            dpfx!("Result {:?} ({:?})", _result, path);
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
            let is_last: IsSyslineLast = syslogproc.is_sysline_last(&syslinep) as IsSyslineLast;
            dpo!("{:?}({}): Found, chan_send_dt.send({:p}, None, {});", tid, tname, syslinep, is_last);
            match chan_send_dt.send((Some(syslinep), None, is_last, FILEOK)) {
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
            dpo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", tid, tname, err);
            eprintln!(
                "ERROR: SyslogProcessor.find_sysline_between_datetime_filters(0) Path {:?} Error {}",
                path, err
            );
            search_more = false;
        }
    }

    if !search_more {
        dpo!("{:?}({}): quit searching…", tid, tname);
        let summary_opt: SummaryOpt = Some(syslogproc.process_stage4_summary());
        dpo!("{:?}({}): !search_more chan_send_dt.send((None, {:?}, {}));", tid, tname, summary_opt, false);
        match chan_send_dt.send((None, summary_opt, false, FILEOK)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: C chan_send_dt.send(…) failed {}", err);
            }
        }
        dpfx!("({:?})", path);

        return;
    }

    // find all proceeding syslines acceptable to the passed filters
    syslogproc.process_stage3_stream_syslines();

    loop {
        // TODO: [2022/06/20] see note about refactoring `find` functions so they are more intuitive
        let result: ResultS3SyslineFind = syslogproc.find_sysline_between_datetime_filters(fo1);
        match result {
            ResultS3SyslineFind::Found((fo, syslinep)) => {
                let is_last = syslogproc.is_sysline_last(&syslinep);
                dpo!("{:?}({}): chan_send_dt.send(({:p}, None, {}));", tid, tname, syslinep, is_last);
                match chan_send_dt.send((Some(syslinep), None, is_last, FILEOK)) {
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
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                dpo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", tid, tname, err);
                eprintln!("ERROR: syslogprocessor.find_sysline({}) {}", fo1, err);
                break;
            }
        }
    }

    syslogproc.process_stage4_summary();

    let summary = syslogproc.summary();
    dpo!("{:?}({}): last chan_send_dt.send((None, {:?}, {}));", tid, tname, summary, false);
    match chan_send_dt.send((None, Some(summary), false, FILEOK)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: E chan_send_dt.send(…) failed {}", err);
        }
    }

    dpfx!("({:?})", path);
}

/// Statistics about the main processing thread's printing activity.
/// Used with CLI option `--summary`.
#[derive(Copy, Clone, Default)]
pub struct SummaryPrinted {
    /// count of bytes printed
    pub bytes: Count,
    /// count of `Lines` printed
    pub lines: Count,
    /// count of `Syslines` printed
    pub syslines: Count,
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

// TODO: move `SummaryPrinted` into `printer/summary.rs`
impl SummaryPrinted {
    /// Print a `SummaryPrinted` with color on stderr.
    ///
    /// Mimics debug print but with colorized zero values.
    /// Only colorize if associated `SummaryOpt` has corresponding
    /// non-zero values.
    pub fn print_colored_stderr(
        &self,
        color_choice_opt: Option<ColorChoice>,
        summary_opt: &SummaryOpt,
    ) {
        let sumd = Summary::default();
        let sum_: &Summary = match summary_opt {
            Some(s) => s,
            None => &sumd,
        };
        eprint!("{{ bytes: ");
        if self.bytes == 0 && sum_.BlockReader_bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(
                COLOR_ERROR,
                color_choice_opt,
                self.bytes
                    .to_string()
                    .as_bytes(),
            ) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                }
                _ => {}
            }
        } else {
            eprint!("{}", self.bytes);
        }

        eprint!(", lines: ");
        if self.lines == 0 && sum_.BlockReader_bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(
                COLOR_ERROR,
                color_choice_opt,
                self.lines
                    .to_string()
                    .as_bytes(),
            ) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                }
                _ => {}
            }
        } else {
            eprint!("{}", self.lines);
        }

        eprint!(", syslines: ");
        if self.syslines == 0 && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(
                COLOR_ERROR,
                color_choice_opt,
                self.syslines
                    .to_string()
                    .as_bytes(),
            ) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                }
                _ => {}
            }
        } else {
            eprint!("{}", self.syslines);
        }

        eprint!(", dt_first: ");
        if self.dt_first.is_none() && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(COLOR_ERROR, color_choice_opt, "None".as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                }
                _ => {}
            }
        } else {
            eprint!("{:?}", self.dt_first);
        }

        eprint!(", dt_last: ");
        if self.dt_last.is_none() && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(COLOR_ERROR, color_choice_opt, "None".as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                }
                _ => {}
            }
        } else {
            eprint!("{:?}", self.dt_first);
        }
        eprint!(" }}");
    }

    /// Update a `SummaryPrinted` with information from a printed `Sysline`.
    //
    // TODO: 2022/06/21 any way to avoid a `DateTime` copy on every printed sysline?
    fn summaryprint_update(
        &mut self,
        syslinep: &SyslineP,
    ) {
        self.syslines += 1;
        self.lines += (*syslinep).count_lines();
        self.bytes += (*syslinep).count_bytes();
        if let Some(dt) = (*syslinep).dt() {
            match self.dt_first {
                Some(dt_first) => {
                    if dt < &dt_first {
                        self.dt_first = Some(*dt);
                    };
                }
                None => {
                    self.dt_first = Some(*dt);
                }
            };
            match self.dt_last {
                Some(dt_last) => {
                    if dt > &dt_last {
                        self.dt_last = Some(*dt);
                    };
                }
                None => {
                    self.dt_last = Some(*dt);
                }
            };
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted`.
    ///
    /// Helper function to function `processing_loop`.
    fn summaryprint_map_update(
        syslinep: &SyslineP,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
    ) {
        dpfñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update(syslinep);
            }
            None => {
                let mut sp = SummaryPrinted::default();
                sp.summaryprint_update(syslinep);
                map_.insert(*pathid, sp);
            }
        };
    }
}

type SummaryPrintedOpt = Option<SummaryPrinted>;

// -------------------------------------------------------------------------------------------------

/// Print the various caching statistics.
const OPT_SUMMARY_PRINT_CACHE_STATS: bool = true;

/// Print the various drop statistics.
const OPT_SUMMARY_PRINT_DROP_STATS: bool = true;

/// For printing `--summary` lines, indentation.
const OPT_SUMMARY_PRINT_INDENT: &str = "  ";

// -------------------------------------------------------------------------------------------------

type MapPathIdSummaryPrint = BTreeMap<PathId, SummaryPrinted>;
type MapPathIdSummary = HashMap<PathId, Summary>;

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

type MapPathIdToProcessPathResult = HashMap<PathId, ProcessPathResult>;
type MapPathIdToFPath = BTreeMap<PathId, FPath>;
type MapPathIdToColor = HashMap<PathId, Color>;
type MapPathIdToPrinterSysline = HashMap<PathId, PrinterSysline>;
type MapPathIdToFileType = HashMap<PathId, FileType>;
type MapPathIdToMimeGuess = HashMap<PathId, MimeGuess>;

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
    cli_opt_summary: bool,
) -> bool {
    dpfn!(
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
        dpfx!("paths_results.is_empty(); nothing to do");
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
    //       Would be best to first implment `FILE`, then `FILE_COMPRESS_GZ`, then `FILE_IN_ARCHIVE_TAR`

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
                dpfo!("map_pathid_results.push({:?})", path);
                map_pathid_path.insert(pathid_counter, path.clone());
                map_pathid_filetype.insert(pathid_counter, *filetype);
                map_pathid_mimeguess.insert(pathid_counter, *mimeguess);
                map_pathid_results.insert(pathid_counter, processpathresult);
            }
            _ => {
                dpfo!("paths_invalid_results.push({:?})", processpathresult);
                map_pathid_results_invalid.insert(pathid_counter, processpathresult);
            }
        };
        paths_total += 1;
    }

    for (_pathid, result_invalid) in map_pathid_results_invalid.iter() {
        match result_invalid {
            ProcessPathResult::FileErrNotParseable(path, _) => {
                eprintln!("WARNING: not a parseable type {:?}", path);
            }
            ProcessPathResult::FileErrNoPermissions(path, _) => {
                eprintln!("WARNING: not enough permissions {:?}", path);
            }
            ProcessPathResult::FileErrNotAFile(path, _) => {
                eprintln!("WARNING: not a file {:?}", path);
            }
            ProcessPathResult::FileErrNotSupported(path, _) => {
                eprintln!("WARNING: not a supported file {:?}", path);
            }
            _ => {}
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
    // track if an error has been printed regarding a particular sysling_print error
    // only want to print this particular error once, not hundreds of times
    let mut has_sysline_print_err: bool = false;
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
        let (chan_send_dt, chan_recv_dt): (ChanSendDatum, ChanRecvDatum) = crossbeam_channel::unbounded();
        dpfo!("map_pathid_chanrecvdatum.insert({}, …);", pathid);
        map_pathid_chanrecvdatum.insert(*pathid, chan_recv_dt);
        let basename_: FPath = basename(path);
        match thread::Builder::new()
            .name(basename_.clone())
            .spawn(move || exec_syslogprocessor_thread(chan_send_dt, thread_data))
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
        dp_err!("map_pathid_chanrecvdatum.is_empty(); nothing to do.");
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
        dpn!("processing_loop:recv_many_chan(…)");
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
            dpo!("processing_loop:recv_many_chan: select.recv({:?});", pathid_chan.1);
            // load `select` with "operations" (receive channels)
            select.recv(pathid_chan.1);
        }
        assert!(!map_index_pathid.is_empty(), "Did not load any recv operations for select.select(). Overzealous filter? possible channels count {}, filter {:?}", pathid_chans.len(), filter_);
        dpo!("processing_loop:recv_many_chan: map_index_pathid: {:?}", map_index_pathid);
        // `select()` blocks until one of the loaded channel operations becomes ready
        let soper: crossbeam_channel::SelectedOperation = select.select();
        // get the index of the chosen "winner" of the `select` operation
        let index: usize = soper.index();
        dpo!("processing_loop:recv_many_chan: soper.index() returned {}", index);
        let pathid: &PathId = match map_index_pathid.get(&index) {
            Some(pathid_) => pathid_,
            None => {
                eprintln!("ERROR: failed to map_index_pathid.get({})", index);
                return None;
            }
        };
        dpo!("processing_loop:recv_many_chan: map_index_pathid.get({}) returned {}", index, pathid);
        let chan: &ChanRecvDatum = match pathid_chans.get(pathid) {
            Some(chan_) => chan_,
            None => {
                eprintln!("ERROR: failed to pathid_chans.get({})", pathid);
                return None;
            }
        };
        dpo!("processing_loop:recv_many_chan: soper.recv({:?})", chan);
        // Get the result of the `recv` done during `select`
        let result = soper.recv(chan);
        dpo!("processing_loop:recv_many_chan: soper.recv returned {:?}", result);

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
    // the `SummaryPrinted` tallying the entire process (tallies each recieved `SyslineP`)
    let mut summaryprinted: SummaryPrinted = SummaryPrinted::default();
    let color_default = COLOR_DEFAULT;

    // mapping PathId to colors for printing.
    let mut map_pathid_printer = MapPathIdToPrinterSysline::with_capacity(file_count);

    // count of not okay FileProcessing
    let mut _fileprocessing_not_okay: usize = 0;

    // track which paths had syslines
    let mut paths_printed_syslines: SetPathId = SetPathId::with_capacity(file_count);

    //
    // the main processing loop (e.g the "game loop")
    //
    // process the "receiving sysline" channels from the running file processing threads.
    // print the earliest available `Sysline`.
    //

    // channels that should be disconnected per "game loop" loop iteration
    let mut disconnect = Vec::<PathId>::with_capacity(file_count);
    loop {
        disconnect.clear();

        if cfg!(debug_assertions) {
            dpfo!("map_pathid_datum.len() {}", map_pathid_datum.len());
            for (pathid, _datum) in map_pathid_datum.iter() {
                let _path: &FPath = map_pathid_path
                    .get(pathid)
                    .unwrap();
                dpfo!("map_pathid_datum: thread {} {} has data", _path, pathid);
            }
            dpfo!("map_pathid_chanrecvdatum.len() {}", map_pathid_chanrecvdatum.len());
            for (pathid, _chanrdatum) in map_pathid_chanrecvdatum.iter() {
                let _path: &FPath = map_pathid_path
                    .get(pathid)
                    .unwrap();
                dpfo!(
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
                // (SyslineP_Opt, SummaryOpt, IsSyslineLast, FileProcessingResultBlockZero)
                Ok(chan_datum) => {
                    dpfo!("B crossbeam_channel::Found for PathId {:?};", pathid);
                    match chan_datum.3 {
                        FileProcessingResultBlockZero::FileOk => {}
                        _ => {
                            _fileprocessing_not_okay += 1;
                        }
                    }
                    if let Some(summary) = chan_datum.1 {
                        assert!(chan_datum.0.is_none(), "ChanDatum Some(Summary) and Some(SyslineP); should only have one Some(). PathId {:?}", pathid);
                        summary_update(&pathid, summary, &mut map_pathid_summary);
                        dpfo!("B will disconnect channel {:?}", pathid);
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
                    dpfo!("B crossbeam_channel::RecvError, will disconnect channel for PathId {:?};", pathid);
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
                    dpo!("{} A1 map_pathid_chanrecvdatum[{:?}] = {:?}", _i, _k, _v);
                }
                for (_i, (_k, _v)) in map_pathid_datum
                    .iter()
                    .enumerate()
                {
                    dpo!("{} A1 map_pathid_datum[{:?}] = {:?}", _i, _k, _v);
                }
            }

            if first_print {
                // One-time creation of prepended datas and SyslinePrinters.

                // First, get a set of all pathids with awaiting Syslines, ignoring paths
                // for which no Syslines were found.
                // No Syslines will be printed for those paths that did not return a Sysline:
                // - do not include them in determining prepended width (CLI option `-w`).
                // - do not create a `SyslinePrinter` for them.
                let mut pathid_with_syslines: SetPathId = SetPathId::with_capacity(map_pathid_datum.len());
                for (pathid, _) in map_pathid_datum
                    .iter()
                    .filter(|(_k, v)| v.0.is_some())
                {
                    pathid_with_syslines.insert(*pathid);
                }

                // Pre-create the prepended strings based on passed CLI options `-w` `-p` `-f`
                let mut prependname_width: usize = 0;
                if cli_opt_prepend_filename {
                    // pre-create prepended filename strings once (`-f`)
                    if cli_opt_prepend_file_align {
                        // determine prepended width (`-w`)
                        for pathid in pathid_with_syslines.iter() {
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
                    pathid_to_prependname = MapPathIdToPrependName::with_capacity(pathid_with_syslines.len());
                    for pathid in pathid_with_syslines.iter() {
                        let path = match map_pathid_path.get(pathid) {
                            Some(path_) => path_,
                            None => continue,
                        };
                        let bname: String = basename(path);
                        let prepend: String = format!("{0:<1$}:", bname, prependname_width);
                        pathid_to_prependname.insert(*pathid, prepend);
                    }
                } else if cli_opt_prepend_filepath {
                    // pre-create prepended filepath strings once (`-p`)
                    if cli_opt_prepend_file_align {
                        // determine prepended width (`-w`)
                        for pathid in pathid_with_syslines.iter() {
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
                    pathid_to_prependname = MapPathIdToPrependName::with_capacity(pathid_with_syslines.len());
                    for pathid in pathid_with_syslines.iter() {
                        let path = match map_pathid_path.get(pathid) {
                            Some(path_) => path_,
                            None => continue,
                        };
                        let prepend: String = format!("{0:<1$}:", path, prependname_width);
                        pathid_to_prependname.insert(*pathid, prepend);
                    }
                } else {
                    pathid_to_prependname = MapPathIdToPrependName::with_capacity(0);
                }

                // Initialize the Sysline printers, one per `PathId` that sent a Sysline.
                for pathid in pathid_with_syslines.iter() {
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
                            true => Some(cli_prepend_dt_format.clone()),
                            false => None,
                        };
                    let prepend_date_offset: Option<FixedOffset> =
                        match (cli_opt_prepend_local, cli_opt_prepend_utc) {
                            (true, false) => Some(*Local::today().offset()),
                            (false, true) => Some(FixedOffset::east(0)),
                            (false, false) => None,
                            // XXX: this should not happen
                            _ => panic!("bad CLI options --local --utc"),
                        };
                    let printer: PrinterSysline = PrinterSysline::new(
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
                .min_by(|x, y| {
                    x.1 .0
                        .as_ref()
                        .unwrap()
                        .dt()
                        .cmp(y.1 .0.as_ref().unwrap().dt())
                }) {
                Some(val) => (val.0, val.1),
                None => {
                    eprintln!("ERROR map_pathid_datum.iter_mut().min_by() returned None");
                    // XXX: not sure what else to do here
                    continue;
                }
            };

            if let Some(summary) = chan_datum.1.clone() {
                // Receiving a Summary implies the last data was sent on the channel
                dpfo!("A2 chan_datum has Summary, PathId: {:?}", pathid);
                assert!(
                    chan_datum.0.is_none(),
                    "ChanDatum Some(Summary) and Some(SyslineP); should only have one Some(). PathId: {:?}",
                    pathid
                );
                if cli_opt_summary {
                    summary_update(pathid, summary, &mut map_pathid_summary);
                }
                dpfo!("A2 will disconnect channel {:?}", pathid);
                disconnect.push(*pathid);
            } else {
                // Is last sysline of the file?
                let is_last: bool = chan_datum.2;
                // Sysline of interest
                let syslinep: &SyslineP = chan_datum.0.as_ref().unwrap();
                dpfo!(
                    "A3 printing @[{}, {}] PathId: {:?}",
                    syslinep.fileoffset_begin(),
                    syslinep.fileoffset_end(),
                    pathid
                );
                // print the sysline!
                let printer: &mut PrinterSysline = map_pathid_printer
                    .get_mut(pathid)
                    .unwrap();
                match printer.print_sysline(syslinep) {
                    Ok(_) => {}
                    Err(err) => {
                        // Only print a printing error once.
                        // In case of piping to something like `head`, it looks bad to print
                        // the same error tens or hundreds of times for a common pipe operation.
                        if !has_sysline_print_err {
                            has_sysline_print_err = true;
                            // BUG: Issue #3 colorization settings in the context of a pipe
                            eprintln!("ERROR: failed to print {}", err);
                        }
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
                    summaryprinted.bytes += 1;
                }
                if cli_opt_summary {
                    paths_printed_syslines.insert(*pathid);
                    // update the per processing file `SummaryPrinted`
                    SummaryPrinted::summaryprint_map_update(syslinep, pathid, &mut map_pathid_sumpr);
                    // update the single total program `SummaryPrinted`
                    summaryprinted.summaryprint_update(syslinep);
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
            dpfo!("C map_pathid_chanrecvdatum.remove({:?});", pathid);
            map_pathid_chanrecvdatum.remove(pathid);
            dpfo!("C pathid_to_prependname.remove({:?});", pathid);
            pathid_to_prependname.remove(pathid);
        }
        // are there any channels to receive from?
        if map_pathid_chanrecvdatum.is_empty() {
            dpfo!("D map_pathid_chanrecvdatum.is_empty(); no more channels to receive from!");
            // all channels are closed, break from main processing loop
            break;
        }
        dpfo!("D map_pathid_chanrecvdatum: {:?}", map_pathid_chanrecvdatum);
        dpfo!("D map_pathid_datum: {:?}", map_pathid_datum);
        dpfo!("D set_pathid: {:?}", set_pathid);
    } // end loop

    // quick count of `Summary` attached Errors
    let mut error_count: usize = 0;
    for (_pathid, summary) in map_pathid_summary.iter() {
        if summary.Error_.is_some() {
            error_count += 1;
        }
    }

    // Getting here means main program processing has completed.
    // Now to print the `--summary` (if it was requested).

    if cli_opt_summary {
        eprintln!();
        eprintln!("Files:");
        // print details about all the valid files
        print_all_files_summaries(
            &map_pathid_path,
            &map_pathid_filetype,
            &map_pathid_mimeguess,
            &map_pathid_color,
            &mut map_pathid_summary,
            &mut map_pathid_sumpr,
            &color_choice,
            &color_default,
        );
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
            paths_printed_syslines.len(),
        );
        eprintln!("{:?}", summaryprinted);
        eprintln!("Datetime Filters: -a {:?} -b {:?}", filter_dt_after_opt, filter_dt_before_opt);
        eprintln!("Channel Receive ok {}, err {}", chan_recv_ok, chan_recv_err);
    }

    dpfo!("E chan_recv_ok {:?} _count_recv_di {:?}", chan_recv_ok, chan_recv_err);

    // TODO: Issue #5 return code confusion
    //       the rationale for returning `false` (and then the process return code 1)
    //       is clunky, and could use a little refactoring. Also needs a gituhub Issue
    let mut ret: bool = true;
    if chan_recv_err > 0 {
        dpfo!("F chan_recv_err {}; return false", chan_recv_err);
        ret = false;
    }
    //if _fileprocessing_not_okay > 0 {
    //    dpfo!("F fileprocessing_not_okay {}; return false", _fileprocessing_not_okay);
    //    ret = false;
    //}
    if error_count > 0 {
        dpfo!("F error_count {}; return false", error_count);
        ret = false;
    }
    dpfx!("return {:?}", ret);

    ret
}

// -------------------------------------------------------------------------------------------------

/// Print the filepath name (one line).
fn print_filepath(
    path: &FPath,
    filetype: &FileType,
    mimeguess: &MimeGuess,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprint!("File: ");
    match print_colored_stderr(*color, Some(*color_choice), path.as_bytes()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("ERROR: {:?}", err);
        }
    };
    eprint!(" ({}) {:?}", filetype, mimeguess);
    eprintln!();
}

/// Print the (optional) [`Summary`] (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_summary_opt_processed(summary_opt: &SummaryOpt) {
    const OPT_SUMMARY_PRINT_INDENT_UNDER: &str = "                   ";
    match summary_opt {
        Some(summary) => {
            eprintln!("{}Summary Processed:{:?}", OPT_SUMMARY_PRINT_INDENT, summary);
            // print datetime first and last
            match (summary.SyslineReader_pattern_first, summary.SyslineReader_pattern_last) {
                (Some(dt_first), Some(dt_last)) => {
                    eprintln!(
                        "{}{}datetime first {:?}",
                        OPT_SUMMARY_PRINT_INDENT, OPT_SUMMARY_PRINT_INDENT_UNDER, dt_first,
                    );
                    eprintln!(
                        "{}{}datetime last  {:?}",
                        OPT_SUMMARY_PRINT_INDENT, OPT_SUMMARY_PRINT_INDENT_UNDER, dt_last,
                    );
                }
                (None, Some(_)) | (Some(_), None) => {
                    eprintln!("ERROR: only one of dt_first or dt_last fulfilled; this is unexpected.");
                }
                _ => {}
            }
            // print datetime patterns
            for patt in summary
                .SyslineReader_patterns
                .iter()
            {
                let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*patt.0];
                eprintln!(
                    "{}{}   @{} {} {:?}",
                    OPT_SUMMARY_PRINT_INDENT, OPT_SUMMARY_PRINT_INDENT_UNDER, patt.0, patt.1, dtpd
                );
            }
            match summary.SyslogProcessor_missing_year {
                Some(year) => {
                    eprintln!(
                        "{}{}datetime format missing year; estimated year of last sysline {:?}",
                        OPT_SUMMARY_PRINT_INDENT, OPT_SUMMARY_PRINT_INDENT_UNDER, year
                    );
                }
                None => {}
            }
        }
        None => {
            // TODO: [2022/06/07] print filesz
            eprintln!("{}Summary Processed: None", OPT_SUMMARY_PRINT_INDENT);
        }
    }
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
            eprint!("{}Summary Printed  : ", OPT_SUMMARY_PRINT_INDENT);
            summary_print.print_colored_stderr(Some(*color_choice), summary_opt);
        }
        None => {
            eprint!("{}Summary Printed  : ", OPT_SUMMARY_PRINT_INDENT);
            SummaryPrinted::default().print_colored_stderr(Some(*color_choice), summary_opt);
        }
    }
    eprintln!();
}

/// Print the various (optional) [`Summary`] caching and storage statistics
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_cache_stats(summary_opt: &SummaryOpt) {
    if summary_opt.is_none() {
        return;
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

    let summary: &Summary = match summary_opt.as_ref() {
        Some(summary_) => summary_,
        None => {
            eprintln!("ERROR: unexpected None from match summary_opt");
            return;
        }
    };
    let wide: usize = summary
        .max_hit_miss()
        .to_string()
        .len();
    let mut ratio: f64;
    // SyslineReader
    // SyslineReader::get_boxptrs
    eprintln!(
        "{}copying: SyslineReader::get_boxptrs()                                  : sgl {:wide$}, dbl  {:wide$}, mult {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_get_boxptrs_singleptr,
        summary.SyslineReader_get_boxptrs_doubleptr,
        summary.SyslineReader_get_boxptrs_multiptr,
        wide = wide,
    );
    // SyslineReader::syslines
    ratio = ratio64(&summary.SyslineReader_syslines_hit, &summary.SyslineReader_syslines_miss);
    eprintln!(
        "{}storage: SyslineReader::find_sysline() syslines                        : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_syslines_hit,
        summary.SyslineReader_syslines_miss,
        ratio,
        wide = wide,
    );
    // SyslineReader::_syslines_by_range
    ratio =
        ratio64(&summary.SyslineReader_syslines_by_range_hit, &summary.SyslineReader_syslines_by_range_miss);
    eprintln!(
        "{}caching: SyslineReader::find_sysline() syslines_by_range_map           : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_syslines_by_range_hit,
        summary.SyslineReader_syslines_by_range_miss,
        ratio,
        summary.SyslineReader_syslines_by_range_put,
        wide = wide,
    );
    // SyslineReader::_find_sysline_lru_cache
    ratio = ratio64(
        &summary.SyslineReader_find_sysline_lru_cache_hit,
        &summary.SyslineReader_find_sysline_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::find_sysline() LRU cache                       : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_find_sysline_lru_cache_hit,
        summary.SyslineReader_find_sysline_lru_cache_miss,
        ratio,
        summary.SyslineReader_find_sysline_lru_cache_put,
        wide = wide,
    );
    // SyslineReader::_parse_datetime_in_line_lru_cache
    ratio = ratio64(
        &summary.SyslineReader_parse_datetime_in_line_lru_cache_hit,
        &summary.SyslineReader_parse_datetime_in_line_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::parse_datetime_in_line() LRU cache             : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_parse_datetime_in_line_lru_cache_hit,
        summary.SyslineReader_parse_datetime_in_line_lru_cache_miss,
        ratio,
        summary.SyslineReader_parse_datetime_in_line_lru_cache_put,
        wide = wide,
    );
    // LineReader::_lines
    ratio = ratio64(&summary.LineReader_lines_hits, &summary.LineReader_lines_miss);
    eprintln!(
        "{}storage: LineReader::find_line() lines                                 : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.LineReader_lines_hits,
        summary.LineReader_lines_miss,
        ratio,
        wide = wide,
    );
    // LineReader::_find_line_lru_cache
    ratio =
        ratio64(&summary.LineReader_find_line_lru_cache_hit, &summary.LineReader_find_line_lru_cache_miss);
    eprintln!(
        "{}caching: LineReader::find_line() LRU cache                             : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.LineReader_find_line_lru_cache_hit,
        summary.LineReader_find_line_lru_cache_miss,
        ratio,
        summary.LineReader_find_line_lru_cache_put,
        wide = wide,
    );
    // BlockReader::_read_blocks
    ratio = ratio64(&summary.BlockReader_read_blocks_hit, &summary.BlockReader_read_blocks_miss);
    eprintln!(
        "{}storage: BlockReader::read_block() blocks                              : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.BlockReader_read_blocks_hit,
        summary.BlockReader_read_blocks_miss,
        ratio,
        summary.BlockReader_read_blocks_put,
        wide = wide,
    );
    // BlockReader::_read_blocks_cache
    ratio = ratio64(
        &summary.BlockReader_read_block_lru_cache_hit,
        &summary.BlockReader_read_block_lru_cache_miss,
    );
    eprintln!(
        "{}caching: BlockReader::read_block() LRU cache                           : hit {:wide$}, miss {:wide$}, ratio {:1.2}, put {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.BlockReader_read_block_lru_cache_hit,
        summary.BlockReader_read_block_lru_cache_miss,
        ratio,
        summary.BlockReader_read_block_lru_cache_put,
        wide = wide,
    );
}

/// Print the (optional) various [`Summary`] drop error statistics
/// (multiple lines).
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_drop_stats(summary_opt: &SummaryOpt) {
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
    let wide: usize = summary
        .max_drop()
        .to_string()
        .len();
    eprintln!(
        "{}streaming: SyslineReader::drop_sysline(): Ok {:wide$} Err {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_drop_sysline_ok,
        summary.SyslineReader_drop_sysline_errors,
        wide = wide,
    );
    eprintln!(
        "{}streaming: LineReader::drop_line()      : Ok {:wide$} Err {:wide$}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.LineReader_drop_line_ok,
        summary.LineReader_drop_line_errors,
        wide = wide,
    );
}

/// Print the [`Summary.Error_`], if any (one line).
///
/// [`Summary.Error_`]: s4lib::readers::summary::Summary
fn print_error_summary(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_opt.as_ref() {
        Some(summary_) => match &summary_.Error_ {
            Some(err_string) => {
                eprint!("{}Error: ", OPT_SUMMARY_PRINT_INDENT);
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
fn print_file_summary(
    path: &FPath,
    filetype: &FileType,
    mimeguess: &MimeGuess,
    summary_opt: &SummaryOpt,
    summary_print_opt: &SummaryPrintedOpt,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprintln!();
    print_filepath(path, filetype, mimeguess, color, color_choice);
    print_summary_opt_processed(summary_opt);
    print_summary_opt_printed(summary_print_opt, summary_opt, color_choice);
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
            .unwrap_or(&FileType::FileUnknown);
        let mimeguess_default: MimeGuess = MimeGuess::from_ext("");
        let mimeguess: &MimeGuess = map_pathid_mimeguess
            .get(pathid)
            .unwrap_or(&mimeguess_default);
        let summary_opt: SummaryOpt = map_pathid_summary.remove(pathid);
        let summary_print_opt: SummaryPrintedOpt = map_pathid_sumpr.remove(pathid);
        print_file_summary(path, filetype, mimeguess, &summary_opt, &summary_print_opt, color, color_choice);
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
            ProcessPathResult::FileErrNotParseable(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not parseable)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotAFile(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not a file)".to_string(), color_choice, color_error);
            }
        }
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    extern crate test_case;
    use test_case::test_case;
    use super::{
        BlockSz, CLI_OPT_PREPEND_FMT, cli_process_tz_offset, cli_validate_blocksz, cli_validate_prepend_dt_format, cli_process_blocksz, DateTimeLOpt, FixedOffset, process_dt, TimeZone,
    };

    #[test_case("500", true)]
    #[test_case("0x2", true)]
    #[test_case("0x4", true)]
    #[test_case("0xFFFFFF", true)]
    #[test_case("BAD_BLOCKSZ_VALUE", false)]
    #[test_case("", false)]
    fn test_cli_validate_blocksz(blocksz_str: &str, is_ok: bool)
    {
        match is_ok {
            true => assert!(cli_validate_blocksz(blocksz_str).is_ok()),
            false => assert!(!cli_validate_blocksz(blocksz_str).is_ok()),
        }
    }

    #[test_case("0b10101010101", Some(0b10101010101))]
    #[test_case("0o44", Some(0o44))]
    #[test_case("00500", Some(500))]
    #[test_case("500", Some(500))]
    #[test_case("0x4", Some(0x4))]
    #[test_case("0xFFFFFF", Some(0xFFFFFF))]
    #[test_case("BAD_BLOCKSZ_VALUE", None)]
    #[test_case("", None)]
    fn test_cli_process_blocksz(blocksz_str: &str, expect_: Option<BlockSz>)
    {
        match expect_ {
            Some(val_exp) => {
                let val_ret = cli_process_blocksz(&String::from(blocksz_str)).unwrap();
                assert_eq!(val_ret, val_exp);
            }
            None => {
                let ret = cli_process_blocksz(&String::from(blocksz_str));
                assert!(ret.is_err(), "Expected an Error for cli_process_blocksz({:?}), instead got {:?}", blocksz_str, ret);
            }
        }
    }

    #[test_case("+00", FixedOffset::east(0); "+00 east(0)")]
    #[test_case("+0000", FixedOffset::east(0); "+0000 east(0)")]
    #[test_case("+00:00", FixedOffset::east(0); "+00:00 east(0)")]
    #[test_case("+00:01", FixedOffset::east(60); "+00:01 east(60)")]
    #[test_case("+01:00", FixedOffset::east(3600); "+01:00 east(3600) A")]
    #[test_case("-01:00", FixedOffset::east(-3600); "-01:00 east(-3600) B")]
    #[test_case("+02:00", FixedOffset::east(7200); "+02:00 east(7200)")]
    #[test_case("+02:30", FixedOffset::east(9000); "+02:30 east(9000)")]
    #[test_case("+02:35", FixedOffset::east(9300); "+02:30 east(9300)")]
    #[test_case("+23:00", FixedOffset::east(82800); "+23:00 east(82800)")]
    #[test_case("UTC", FixedOffset::east(0); "UTC east(0)")]
    #[test_case("vlat", FixedOffset::east(36000); "vlat east(36000)")]
    #[test_case("IDLW", FixedOffset::east(-43200); "IDLW east(-43200)")]
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
    fn test_cli_validate_prepend_dt_format(prepend_dt_format: &str)
    {
        assert!(cli_validate_prepend_dt_format(prepend_dt_format).is_ok());
    }

    #[test_case(
        Some(String::from("2000-01-02T03:04:05")), FixedOffset::east(0),
        Some(FixedOffset::east(0).ymd(2000, 1, 2).and_hms(3, 4, 5)); "2000-01-02T03:04:05"
    )]
    #[test_case(
        Some(String::from("2000-01-02T03:04:05 -0100")), FixedOffset::east(0),
        Some(FixedOffset::east(-3600).ymd(2000, 1, 2).and_hms(3, 4, 5)); "2000-01-02T03:04:05 -0100"
    )]
    fn test_process_dt(dts: Option<String>, tz_offset: FixedOffset, expect: DateTimeLOpt)
    {
        assert_eq!(process_dt(dts, &tz_offset), expect);
    }

}
