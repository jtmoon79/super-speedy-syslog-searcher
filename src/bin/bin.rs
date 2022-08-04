// src/bin/bin.rs
//
// ‥ … ≤ ≥ ≠ ≟

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// uses
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::collections::{
    HashMap,
    HashSet,
    BTreeMap,
};
use std::fmt;
use std::str;
use std::thread;

extern crate backtrace;

extern crate chrono;
use chrono::{
    FixedOffset,
    Local,
    Offset,
    TimeZone,
};

extern crate clap;
use clap::{
    ArgEnum,
    Parser,
};

extern crate crossbeam_channel;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate unicode_width;

// `s4lib` is the local compiled `[lib]` of super_speedy_syslog_searcher
extern crate s4lib;

use s4lib::common::{
    Count,
    FPath,
    FPaths,
    FileOffset,
    FileType,
    NLu8a,
};

use s4lib::data::datetime::{
    DateTimeL_Opt,
    DateTimePattern_str,
    DateTime_Parse_Data,
    datetime_parse_from_str,
    DATETIME_PARSE_DATAS,
};

use s4lib::printer_debug::stack::{
    stack_offset_set,
};

#[allow(unused_imports)]
use s4lib::printer_debug::printers::{
    dpo,
    dpn,
    dpof,
    dpnf,
    dpxf,
    dpnxf,
    dp_err,
    dp_wrn,
    p_err,
    p_wrn,
};

use s4lib::printer::printers::{
    // termcolor imports
    Color,
    ColorChoice,
    //
    COLOR_ERROR,
    color_rand,
    PrinterSysline,
    print_colored_stderr,
    write_stdout,
};

use s4lib::data::sysline::{
    SyslineP,
    SyslineP_Opt,
};

use s4lib::readers::blockreader::{
    BlockSz,
    BLOCKSZ_MIN,
    BLOCKSZ_MAX,
    BLOCKSZ_DEF,
};

use s4lib::readers::filepreprocessor::{
    ProcessPathResult,
    ProcessPathResults,
    process_path,
};

use s4lib::readers::helpers::{
    basename,
};

use s4lib::readers::summary::{
    Summary,
    SummaryOpt,
};

use s4lib::readers::syslinereader::{
    ResultS4_SyslineFind,
};

use s4lib::readers::syslogprocessor::{
    SyslogProcessor,
    FileProcessingResultBlockZero,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// CLI enum that maps to `termcolor::ColorChoice`
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ArgEnum,  // from `clap`
)]
enum CLI_Color_Choice {
    always,
    auto,
    never,
}

/// subset of `DateTime_Parse_Data` for calls to `datetime_parse_from_str`
///
/// (DateTimePattern_str, has year, has timezone, has time)
type CLI_DT_Filter_Pattern<'b> = (
    &'b DateTimePattern_str,
    bool,
    bool,
    bool,
);

const CLI_DT_FILTER_PATTERN1: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S", true, false, true);
const CLI_DT_FILTER_PATTERN2: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S%z", true, true, true);
const CLI_DT_FILTER_PATTERN3: CLI_DT_Filter_Pattern = ("%Y%m%dT%H%M%S%Z", true, true, true);
const CLI_DT_FILTER_PATTERN4: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S", true, false, true);
const CLI_DT_FILTER_PATTERN5: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S %z", true, true, true);
const CLI_DT_FILTER_PATTERN6: CLI_DT_Filter_Pattern = ("%Y-%m-%d %H:%M:%S %Z", true, true, true);
const CLI_DT_FILTER_PATTERN7: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S", true, false, true);
const CLI_DT_FILTER_PATTERN8: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S %z", true, true, true);
const CLI_DT_FILTER_PATTERN9: CLI_DT_Filter_Pattern = ("%Y-%m-%dT%H:%M:%S %Z", true, true, true);
const CLI_DT_FILTER_PATTERN10: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S", true, false, true);
const CLI_DT_FILTER_PATTERN11: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S %z", true, true, true);
const CLI_DT_FILTER_PATTERN12: CLI_DT_Filter_Pattern = ("%Y/%m/%d %H:%M:%S %Z", true, true, true);
const CLI_DT_FILTER_PATTERN13: CLI_DT_Filter_Pattern = ("%Y%m%d", true, false, false);
const CLI_DT_FILTER_PATTERN14: CLI_DT_Filter_Pattern = ("%Y%m%d %z", true, true, false);
const CLI_DT_FILTER_PATTERN15: CLI_DT_Filter_Pattern = ("%Y%m%d %Z", true, true, false);
const CLI_DT_FILTER_PATTERN16: CLI_DT_Filter_Pattern = ("+%s", false, false, true);
// TODO: [2022/06/19] allow passing three-letter TZ abbreviation
//const CLI_DT_FILTER_PATTERN13: &CLI_DT_Filter_Pattern = &("%Y/%m/%d", true, false);
//const CLI_DT_FILTER_PATTERN14: &CLI_DT_Filter_Pattern = &("%Y-%m-%d", true, false);
//const CLI_DT_FILTER_PATTERN15: &CLI_DT_Filter_Pattern = &("%Y%m%d", true, false);
const CLI_FILTER_PATTERNS_COUNT: usize = 16;
/// acceptable datetime filter patterns for the user-passed `-a` or `-b`
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
];
/// time to append in `fn process_dt` when `has_time` is false
const CLI_DT_FILTER_APPEND_TIME_VALUE: &str = " T000000";
/// strftime format pattern to append in `fn process_dt` when `has_time` is false
const CLI_DT_FILTER_APPEND_TIME_PATTERN: &str = " T%H%M%S";
/// datetime format printed for CLI options `-u` or `-l`
const CLI_OPT_PREPEND_FMT: &str = "%Y%m%dT%H%M%S%.6f %z:";

const CLI_HELP_AFTER: &str = "\
DateTime Filter patterns may be:
    '%Y%m%dT%H%M%S'
    '%Y%m%dT%H%M%S%z'
    '%Y-%m-%d %H:%M:%S'
    '%Y-%m-%d %H:%M:%S %z'
    '%Y-%m-%d %H:%M:%S %Z'
    '%Y-%m-%dT%H:%M:%S'
    '%Y-%m-%dT%H:%M:%S %z'
    '%Y-%m-%dT%H:%M:%S %Z'
    '%Y/%m/%d %H:%M:%S'
    '%Y/%m/%d %H:%M:%S %z'
    '%Y/%m/%d %H:%M:%S %Z'
    '%Y%m%d'
    '%Y%m%d %z'
    '%Y%m%d %Z'
    '+%s'

Without a timezone offset (%z or %Z), the Datetime Filter is presumed to be the system timezone.
Pattern '+%s' is Unix epoch timestamp in seconds with a preceding '+'.

DateTime Filter formatting is described at
https://docs.rs/chrono/latest/chrono/format/strftime/

Prepended datetime, -u or -l, is printed in format '%Y%m%dT%H%M%S%.6f %z'.

DateTimes supported are only of the Gregorian calendar.
DateTimes languages is English.";

// clap references:
//   inference types https://github.com/clap-rs/clap/blob/v3.1.6/examples/derive_ref/README.md#arg-types
//   other `clap::App` options https://docs.rs/clap/latest/clap/struct.App.html
//   the `about` is taken from `Cargo.toml:[package]:description`
#[derive(Parser, Debug)]
#[clap(
    //author,
    version,
    about,
    after_help = CLI_HELP_AFTER,
    //before_help = "",
    setting = clap::AppSettings::DeriveDisplayOrder,
)]
/// this is the `CLI_Args` docstring, is it captured by clap?
struct CLI_Args {
    /// Path(s) of syslog files or directories.
    /// Directories will be recursed, remaining on the same filesystem.
    /// Symlinks will be followed.
    #[clap(required = true)]
    paths: Vec::<String>,
    //#[clap(parse(from_os_str))]
    //paths: Vec::<std::path::PathBuf>,

    /// DateTime Filter after.
    #[clap(
        short = 'a',
        long,
        help = "DateTime After filter - print syslog lines with a datetime that is at or after this datetime. For example, '20200102T123000'",
    )]
    dt_after: Option<String>,

    /// DateTime Filter before.
    #[clap(
        short = 'b',
        long,
        help = "DateTime Before filter - print syslog lines with a datetime that is at or before this datetime. For example, '20200102T123001'",
    )]
    dt_before: Option<String>,

    /// Default timezone offset for naive datetimes (without timezone offset)
    #[clap(
        short = 't',
        long,
        help = "DateTime Timezone offset - for syslines with a datetime that does not include a timezone, this will be used. For example, '-0800' '+02:00' (with or without ':'). If passing a value with leading '-', use the '=' to explicitly set the argument, e.g. '-t=-0800'. Otherwise the CLI argument parsing will fail. Default is local system timezone offset.",
        validator = cli_validate_tz_offset,
        default_value_t=Local.timestamp(0, 0).offset().to_string(),
    )]
    tz_offset: String,

    /// Prepend DateTime in the UTC Timezone for every sysline.
    #[clap(
        short = 'u',
        long = "prepend-utc",
        group = "prepend_dt",
    )]
    prepend_utc: bool,

    /// Prepend DateTime in the Local Timezone for every sysline.
    #[clap(
        short = 'l',
        long = "prepend-local",
        group = "prepend_dt",
    )]
    prepend_local: bool,

    /// Prepend file basename to every sysline.
    #[clap(
        short = 'n',
        long = "prepend-filename",
        group = "prepend_file",
    )]
    prepend_filename: bool,

    /// Prepend file full path to every sysline.
    #[clap(
        short = 'p',
        long = "prepend-filepath",
        group = "prepend_file",
    )]
    prepend_filepath: bool,

    /// Aligh column width of prepended file basename or file path.
    #[clap(
        short = 'w',
        long = "prepend-file-align",
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

    /// Read blocks of this size. May pass decimal or hexadecimal numbers.
    #[clap(
        required = false,
        short = 'z',
        long,
        default_value_t = BLOCKSZ_DEF.to_string(),
        validator = cli_validate_blocksz,
    )]
    blocksz: String,

    /// Print ending summary of files processed. Printed to stderr.
    #[clap(
        short,
        long,
    )]
    summary: bool,
}

/// CLI argument processing
fn cli_process_blocksz(blockszs: &String) -> std::result::Result<u64, String> {
    // TODO: there must be a more concise way to parse numbers with radix formatting
    let blocksz_: BlockSz;
    let errs = format!("Unable to parse a number for --blocksz {:?}", blockszs);

    if blockszs.starts_with("0x") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0x"), 16) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else if blockszs.starts_with("0o") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0o"), 8) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else if blockszs.starts_with("0b") {
        blocksz_ = match BlockSz::from_str_radix(blockszs.trim_start_matches("0b"), 2) {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    } else {
        blocksz_ = match blockszs.parse::<BlockSz>() {
            Ok(val) => val,
            Err(err) => { return Err(format!("{} {}", errs, err)) }
        };
    }

    let max_min = std::cmp::max(BLOCKSZ_MIN, SyslogProcessor::BLOCKSZ_MIN);
    if ! (max_min <= blocksz_ && blocksz_ <= BLOCKSZ_MAX) {
        return Err(format!("--blocksz must be {} ≤ BLOCKSZ ≤ {}, it was {:?}", max_min, BLOCKSZ_MAX, blockszs));
    }

    Ok(blocksz_)
}

/// argument validator for clap
/// see https://github.com/clap-rs/clap/blob/v3.1.6/examples/tutorial_derive/04_02_validate.rs
fn cli_validate_blocksz(blockszs: &str) -> clap::Result<(), String> {
    match cli_process_blocksz(&String::from(blockszs)) {
        Ok(_) => {},
        Err(err) => { return Err(err); }
    }
    Ok(())
}

/// CLI argument processing
// TODO: move some of this into small testable helper functions
fn cli_process_tz_offset(tzo: &String) -> std::result::Result<FixedOffset, String> {
    let mut tzo_: String;
    if tzo.is_empty() {
        // ripped from https://stackoverflow.com/a/59603899/471376
        let local_offs = Local.timestamp(0, 0).offset().fix().local_minus_utc();
        let hours = local_offs / 3600;
        let mins = local_offs % 3600;
        tzo_ = format!("{:+03}{:02}", hours, mins);
    } else {
        tzo_ = tzo.clone();
    }
    // default value is "+00:00", remove one ":"
    if let Some(index) = tzo_.find(':') {
        tzo_.remove(index);
    }
    #[allow(clippy::from_str_radix_10)]
    let fo_val = match i32::from_str_radix(tzo_.as_str(), 10) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    let hours: i32 = fo_val / 100;
    let mins: i32 = fo_val % 100;
    let east: i32 = (hours * 3600) + (mins * 60);
    let fo = match FixedOffset::east_opt(east) {
        Some(val) => val,
        None => {
            return Err(format!("Unable to parse a timezone FixedOffset for -t {:?} (value {:?})", tzo, east));
        }
    };

    Ok(fo)
}

/// argument validator for clap
fn cli_validate_tz_offset(tz_offset: &str) -> std::result::Result<(), String> {
    match cli_process_tz_offset(&String::from(tz_offset)) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

/// helper to `cli_process_args`
fn process_dt(dts: Option<String>, tz_offset: &FixedOffset) -> DateTimeL_Opt {
    // parse datetime filters
    match dts {
        Some(dts) => {
            let mut dto: DateTimeL_Opt = None;
            for (pattern_, _has_year, has_tz, has_time) in CLI_FILTER_PATTERNS.iter() {
                let mut pattern: String = String::from(*pattern_);
                let mut dts_: String = dts.clone();
                // if !has_time then modify the value and pattern
                // e.g. `"20220101"` becomes `"20220101 T000000"`
                //      `"%Y%d%m"` becomes `"%Y%d%m T%H%M%S"`
                if ! has_time {
                    dts_.push_str(CLI_DT_FILTER_APPEND_TIME_VALUE);
                    pattern.push_str(CLI_DT_FILTER_APPEND_TIME_PATTERN);
                    dpof!("appended {:?}, {:?}", CLI_DT_FILTER_APPEND_TIME_VALUE, CLI_DT_FILTER_APPEND_TIME_PATTERN);
                }
                dpof!("datetime_parse_from_str({:?}, {:?}, {:?}, {:?})", dts_, pattern, has_tz, tz_offset);
                if let Some(val) = datetime_parse_from_str(
                    dts_.as_str(), pattern.as_str(), *has_tz, tz_offset
                ) {
                    dto = Some(val);
                    break;
                };
            };
            if dto.is_none() {
                eprintln!("ERROR: Unable to parse a datetime from {:?}", dts);
                std::process::exit(1);
            }

            dto
        },
        None => None,
    }
}

/// process passed CLI arguments into types
/// this function will `std::process::exit` if there is an `Err`
fn cli_process_args() -> (
    FPaths,
    BlockSz,
    DateTimeL_Opt,
    DateTimeL_Opt,
    FixedOffset,
    ColorChoice,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
) {
    let args = CLI_Args::parse();

    dpof!("args {:?}", args);

    //
    // process string arguments into specific types
    //
    
    let blockszs: String = args.blocksz;
    let blocksz: BlockSz = match cli_process_blocksz(&blockszs) {
        Ok(val) => { val },
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    };
    dpof!("blocksz {:?}", blocksz);

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
    dpof!("tz_offset {:?}", tz_offset);

    let filter_dt_after: DateTimeL_Opt = process_dt(args.dt_after, &tz_offset);
    dpof!("filter_dt_after {:?}", filter_dt_after);
    let filter_dt_before: DateTimeL_Opt = process_dt(args.dt_before, &tz_offset);
    dpof!("filter_dt_before {:?}", filter_dt_before);

    #[allow(clippy::single_match)]
    match (filter_dt_after, filter_dt_before) {
        (Some(dta), Some(dtb)) => {
            if dta > dtb {
                eprintln!("ERROR: Datetime --dt-after ({}) is after Datetime --dt-before ({})", dta, dtb);
                std::process::exit(1);
            }
        },
        _ => {},
    }

    // map `CLI_Color_Choice` to `ColorChoice`
    let color_choice: ColorChoice = match args.color_choice {
        CLI_Color_Choice::always => ColorChoice::Always,
        CLI_Color_Choice::auto => ColorChoice::Auto,
        CLI_Color_Choice::never => ColorChoice::Never,
    };

    (
        fpaths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        args.prepend_utc,
        args.prepend_local,
        args.prepend_filename,
        args.prepend_filepath,
        args.prepend_file_align,
        args.summary
    )
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// command-line parsing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// process user-passed command-line arguments
pub fn main() -> std::result::Result<(), chrono::format::ParseError> {
    // set once, use `stackdepth_main` to access `_STACKDEPTH_MAIN`
    if cfg!(debug_assertions) {
        stack_offset_set(Some(0 ));
    }
    dpnf!();
    let (
        paths,
        blocksz,
        filter_dt_after,
        filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
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

    processing_loop(
        processed_paths,
        blocksz,
        &filter_dt_after,
        &filter_dt_before,
        tz_offset,
        color_choice,
        cli_opt_prepend_utc,
        cli_opt_prepend_local,
        cli_opt_prepend_filename,
        cli_opt_prepend_filepath,
        cli_opt_prepend_file_align,
        cli_opt_summary,
    );

    dpxf!();

    Ok(())
}

// -------------------------------------------------------------------------------------------------
// processing threads
// -------------------------------------------------------------------------------------------------

// TODO: leave a long code comment explaining  why I chose this threading pub-sub approach
//       see old archived code to see previous attempts

/// Paths are needed as keys. Many such keys are passed around among different threads.
/// This requires many `FPath::clone()`. Instead of clones, pass around a relatively light-weight
/// `usize` as a key.
/// The main processing thread can use a `PathId` key to lookup the `FPath` as-needed.
type PathId = usize;
/// data to initialize a file processing thread
type Thread_Init_Data4 = (
    FPath,
    PathId,
    FileType,
    BlockSz,
    DateTimeL_Opt,
    DateTimeL_Opt,
    FixedOffset,
);
type IsSyslineLast = bool;
/// the data sent from file processing thread to the main processing thread
type Chan_Datum = (SyslineP_Opt, SummaryOpt, IsSyslineLast);
type Map_PathId_Datum = BTreeMap<PathId, Chan_Datum>;
type Set_PathId = HashSet<PathId>;
type Chan_Send_Datum = crossbeam_channel::Sender<Chan_Datum>;
type Chan_Recv_Datum = crossbeam_channel::Receiver<Chan_Datum>;
type Map_PathId_ChanRecvDatum = BTreeMap<PathId, Chan_Recv_Datum>;

/// Thread entry point for processing a file
/// this creates `SyslogProcessor` and processes the syslog file `Syslines`.
/// Sends each processed sysline back across channel to main thread.
fn exec_4(chan_send_dt: Chan_Send_Datum, thread_init_data: Thread_Init_Data4) {
    stack_offset_set(Some(2));
    let (
        path,
        _pathid,
        filetype,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
        tz_offset
    ) = thread_init_data;
    dpnf!("({:?})", path);

    let thread_cur: thread::Thread = thread::current();
    let _tid: thread::ThreadId = thread_cur.id();
    let tname: &str = <&str>::clone(&thread_cur.name().unwrap_or(""));

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
            eprintln!("ERROR: SyslogProcessor::new({:?}) failed {}", path.as_str(), err);
            let mut summary = Summary::default();
            summary.Error_ = Some(err.to_string());
            // LAST WORKING HERE [2022/06/18]
            match chan_send_dt.send((None, Some(summary), true)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            return;
        }
    };
    dpof!("syslogproc {:?}", syslogproc);

    syslogproc.process_stage0_valid_file_check();

    let result = syslogproc.process_stage1_blockzero_analysis();
    match result {
        FileProcessingResultBlockZero::FileErrNoLinesFound => {
            eprintln!("WARNING: no lines found {:?}", path);
            return;
        }
        FileProcessingResultBlockZero::FileErrNoSyslinesFound => {
            eprintln!("WARNING: no syslines found {:?}", path);
            return;
        }
        FileProcessingResultBlockZero::FileErrDecompress => {
            eprintln!("WARNING: could not decompress {:?}", path);
            return;
        }
        FileProcessingResultBlockZero::FileErrWrongType => {
            eprintln!("WARNING: bad path {:?}", path);
            return;
        }
        FileProcessingResultBlockZero::FileErrIo(err) => {
            eprintln!("ERROR: Error {} for {:?}", err, path);
            return;
        }
        FileProcessingResultBlockZero::FileOk => {}
        FileProcessingResultBlockZero::FileErrEmpty => {}
        FileProcessingResultBlockZero::FileErrNoSyslinesInDtRange => {}
    }

    // find first sysline acceptable to the passed filters
    syslogproc.process_stage2_find_dt();

    // sanity check sending of `is_last`
    let mut sent_is_last: bool = false;
    let mut fo1: FileOffset = 0;
    let search_more: bool;
    let result: ResultS4_SyslineFind = syslogproc.find_sysline_between_datetime_filters(0);
    let eof: bool = result.is_eof();
    match result {
        ResultS4_SyslineFind::Found((fo, syslinep)) | ResultS4_SyslineFind::Found_EOF((fo, syslinep)) => {
            fo1 = fo;
            let is_last: IsSyslineLast = syslogproc.is_sysline_last(&syslinep) as IsSyslineLast;
            // XXX: yet another reason to get rid of `Found_EOF` (`Found` and `Done` are enough)
            assert_eq!(eof, is_last, "result.is_eof() {}, syslogproc.is_sysline_last(…) {}; they should match; Sysline @{:?}", eof, is_last, (*syslinep).fileoffset_begin());
            dpo!("{:?}({}): Found, chan_send_dt.send({:p}, None, {});", _tid, tname, syslinep, is_last);
            match chan_send_dt.send((Some(syslinep), None, is_last)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: A chan_send_dt.send(…) failed {}", err);
                }
            }
            // XXX: sanity check
            if is_last {
                assert!(!sent_is_last, "is_last {}, yet sent_is_last was also {} (is_last was already sent!)", is_last, sent_is_last);
                sent_is_last = true;
            }
            search_more = !eof;
        },
        ResultS4_SyslineFind::Done => {
            search_more = false;
        },
        ResultS4_SyslineFind::Err(err) => {
            dpo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", _tid, tname, err);
            eprintln!("ERROR: SyslogProcessor.find_sysline_between_datetime_filters(0) Path {:?} Error {}", path, err);
            search_more = false;
        },
    }

    if !search_more {
        dpo!("{:?}({}): quit searching…", _tid, tname);
        let summary_opt: SummaryOpt = Some(syslogproc.process_stage4_summary());
        dpo!("{:?}({}): !search_more chan_send_dt.send((None, {:?}, {}));", _tid, tname, summary_opt, false);
        match chan_send_dt.send((None, summary_opt, false)) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: C chan_send_dt.send(…) failed {}", err);
            }
        }
        dpxf!("({:?})", path);

        return;
    }

    // find all proceeding syslines acceptable to the passed filters
    syslogproc.process_stage3_stream_syslines();

    loop {
        // TODO: [2022/06/20] see note about refactoring `find` functions so they are more intuitive
        let result: ResultS4_SyslineFind = syslogproc.find_sysline_between_datetime_filters(fo1);
        let eof: bool = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, syslinep)) | ResultS4_SyslineFind::Found_EOF((fo, syslinep)) =>
            {
                let is_last = syslogproc.is_sysline_last(&syslinep);
                // XXX: yet another reason to get rid of `Found_EOF` (`Found` and `Done` are enough)
                assert_eq!(eof, is_last, "from find_sysline_between_datetime_filters({}), ResultS4_SyslineFind.is_eof is {:?} (EOF), yet the returned SyslineP.is_sysline_last is {:?}; they should always agree, for file {:?}", fo, eof, is_last, path);
                dpo!("{:?}({}): chan_send_dt.send(({:p}, None, {}));", _tid, tname, syslinep, is_last);
                match chan_send_dt.send((Some(syslinep), None, is_last)) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("ERROR: D chan_send_dt.send(…) failed {}", err);
                    }
                }
                fo1 = fo;
                // XXX: sanity check
                if is_last {
                    assert!(!sent_is_last, "is_last {}, yet sent_is_last was also {} (is_last was already sent!)", is_last, sent_is_last);
                    sent_is_last = true;
                }
                if eof {
                    break;
                }
            }
            ResultS4_SyslineFind::Done => {
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                dpo!("{:?}({}): find_sysline_at_datetime_filter returned Err({:?});", _tid, tname, err);
                eprintln!("ERROR: syslogprocessor.find_sysline({}) {}", fo1, err);
                break;
            }
        }
    }

    syslogproc.process_stage4_summary();

    let summary = syslogproc.summary();
    dpo!("{:?}({}): last chan_send_dt.send((None, {:?}, {}));", _tid, tname, summary, false);
    match chan_send_dt.send((None, Some(summary), false)) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: E chan_send_dt.send(…) failed {}", err);
        }
    }

    dpxf!("({:?})", path);
}

/// statistics to print about printing
#[derive(Copy, Clone, Default)]
pub struct SummaryPrinted {
    /// count of bytes printed
    pub bytes: Count,
    /// count of `Lines` printed
    pub lines: Count,
    /// count of `Syslines` printed
    pub syslines: Count,
    /// last datetime printed
    pub dt_first: DateTimeL_Opt,
    pub dt_last: DateTimeL_Opt,
}

impl fmt::Debug for SummaryPrinted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Printed:")
            .field("bytes", &self.bytes)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field("dt_first", &format_args!("{}",
                match self.dt_first {
                        Some(dt) => {
                            dt.to_string()
                        },
                        None => { String::from("None") },
                    }
                )
            )
            .field("dt_last", &format_args!("{}",
                match self.dt_last {
                        Some(dt) => {
                            dt.to_string()
                        },
                        None => { String::from("None") },
                    }
                )
            )
            .finish()
    }
}

// TODO: move `SummaryPrinted` into `printer/summary.rs`
impl SummaryPrinted {

    /// print a `SummaryPrinted` with color on stderr.
    ///
    /// mimics debug print but with colorized zero values
    /// only colorize if associated `SummaryOpt` has corresponding
    /// non-zero values
    pub fn print_colored_stderr(&self, color_choice_opt: Option<ColorChoice>, summary_opt: &SummaryOpt) {
        
        let sumd = Summary::default();
        let sum_: &Summary = match summary_opt {
            Some(s) => s,
            None => {
                &sumd
            }
        };
        eprint!("{{ bytes: ");
        if self.bytes == 0 && sum_.BlockReader_bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(COLOR_ERROR, color_choice_opt, self.bytes.to_string().as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{}", self.bytes);
        }

        eprint!(", lines: ");
        if self.lines == 0 && sum_.BlockReader_bytes != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(COLOR_ERROR, color_choice_opt, self.lines.to_string().as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
            }
        } else {
            eprint!("{}", self.lines);
        }

        eprint!(", syslines: ");
        if self.syslines == 0 && sum_.LineReader_lines != 0 {
            #[allow(clippy::single_match)]
            match print_colored_stderr(COLOR_ERROR, color_choice_opt, self.syslines.to_string().as_bytes()) {
                Err(err) => {
                    eprintln!("ERROR: print_colored_stderr {:?}", err);
                    return;
                },
                _ => {},
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
                },
                _ => {},
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
                },
                _ => {},
            }
        } else {
            eprint!("{:?}", self.dt_first);
        }
        eprint!(" }}");
    }

    /// update a `SummaryPrinted` with information from a printed `Sysline`
    //
    // TODO: 2022/06/21 any way to avoid a `DateTime` copy on every printed sysline?
    fn summaryprint_update(&mut self, syslinep: &SyslineP) {
        self.syslines += 1;
        self.lines += (*syslinep).count_lines();
        self.bytes += (*syslinep).count_bytes();
        if let Some(dt) = (*syslinep).dt() {
            match self.dt_first {
                Some(dt_first) => {
                    if dt < &dt_first {
                        self.dt_first = Some(*dt);
                    };
                },
                None => {
                    self.dt_first = Some(*dt);
                },
            };
            match self.dt_last {
                Some(dt_last) => {
                    if dt > &dt_last {
                        self.dt_last = Some(*dt);
                    };
                },
                None => {
                    self.dt_last = Some(*dt);
                },
            };
        };
    }
    
    /// update a mapping of `PathId` to `SummaryPrinted`.
    ///
    /// helper to `processing_loop`
    fn summaryprint_map_update(syslinep: &SyslineP, pathid: &PathId, map_: &mut Map_PathId_SummaryPrint) {
        dpnxf!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update(syslinep);
            },
            None => {
                let mut sp = SummaryPrinted::default();
                sp.summaryprint_update(syslinep);
                map_.insert(*pathid, sp);
            }
        };
    }
}

type SummaryPrinted_Opt = Option<SummaryPrinted>;

// -------------------------------------------------------------------------------------------------

/// print the various caching statistics
const OPT_SUMMARY_PRINT_CACHE_STATS: bool = true;
/// print the various drop statistics
const OPT_SUMMARY_PRINT_DROP_STATS: bool = true;
/// for printing `--summary` lines, indentation
const OPT_SUMMARY_PRINT_INDENT: &str = "  ";

// -------------------------------------------------------------------------------------------------

type Map_PathId_SummaryPrint = BTreeMap::<PathId, SummaryPrinted>;
type Map_PathId_Summary = HashMap::<PathId, Summary>;

/// small helper to `processing_loop`
#[inline(always)]
fn summary_update(pathid: &PathId, summary: Summary, map_: &mut Map_PathId_Summary) {
    if let Some(val) = map_.insert(*pathid, summary) {
        eprintln!("Error: processing_loop: map_pathid_summary already contains key {:?} with {:?}, overwritten", pathid, val);
    };
}

type Map_PathId_ProcessPathResult = HashMap::<PathId, ProcessPathResult>;
type Map_PathId_FPath = BTreeMap::<PathId, FPath>;
type Map_PathId_Color = HashMap::<PathId, Color>;
type Map_PathId_PrinterSysline = HashMap::<PathId, PrinterSysline>;
type Map_PathId_FileType = HashMap::<PathId, FileType>;
type Map_PathId_MimeGuess = HashMap::<PathId, MimeGuess>;

/// the main processing loop:
///
/// 1. creates threads to process each file
///
/// 2. waits on each thread to receive processed `Sysline` _or_ end
///    a. prints received `Sysline` in datetime order
///    b. repeat 2. until each thread sends a `Summary`
///
/// 3. print each `Summary` (if CLI option `--summary`)
///
/// This main thread should be the only thread that prints to stdout. In --release
/// builds, other file processing threads may rarely print messages to stderr.
///
#[allow(clippy::too_many_arguments)]
fn processing_loop(
    mut paths_results: ProcessPathResults,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeL_Opt,
    filter_dt_before_opt: &DateTimeL_Opt,
    tz_offset: FixedOffset,
    color_choice: ColorChoice,
    cli_opt_prepend_utc: bool,
    cli_opt_prepend_local: bool,
    cli_opt_prepend_filename: bool,
    cli_opt_prepend_filepath: bool,
    cli_opt_prepend_file_align: bool,
    cli_opt_summary: bool,
) {
    dpnf!("({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})", paths_results, blocksz, filter_dt_after_opt, filter_dt_before_opt, color_choice, cli_opt_prepend_local, cli_opt_prepend_utc, cli_opt_summary);

    // XXX: sanity check
    assert!(!(cli_opt_prepend_filename && cli_opt_prepend_filepath), "Cannot both cli_opt_prepend_filename && cli_opt_prepend_filepath");
    // XXX: sanity check
    assert!(!(cli_opt_prepend_utc && cli_opt_prepend_local), "Cannot both cli_opt_prepend_utc && cli_opt_prepend_local");

    if paths_results.is_empty() {
        dpxf!("paths_results.is_empty(); nothing to do");
        return;
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

    // precount the number of files that will be processed
    let file_count: usize = paths_results.iter()
        .filter(|x| matches!(x, ProcessPathResult::FileValid(..)))
        .count();

    // create various mappings of PathId -> Thingy:
    //
    // separate `ProcessPathResult`s into different collections, valid and invalid
    //
    // `valid` is used extensively
    let mut map_pathid_results = Map_PathId_ProcessPathResult::with_capacity(file_count);
    // `invalid` is used to help summarize why some files were not processed
    let mut map_pathid_results_invalid = Map_PathId_ProcessPathResult::with_capacity(file_count);
    // use `map_pathid_path` for iterating, it is a BTreeMap (which iterates in consistent key order)
    let mut map_pathid_path = Map_PathId_FPath::new();
    // map `PathId` to `FileType`
    let mut map_pathid_filetype = Map_PathId_FileType::with_capacity(file_count);
    let mut map_pathid_mimeguess = Map_PathId_MimeGuess::with_capacity(file_count);
    for (pathid_counter, processpathresult) in paths_results.drain(..).enumerate()
    {
        match processpathresult {
            // XXX: use `ref` to avoid "use of partially moved value" error
            ProcessPathResult::FileValid(ref path, ref mimeguess, ref filetype) =>
            {
                dpof!("map_pathid_results.push({:?})", path);
                map_pathid_path.insert(pathid_counter, path.clone());
                map_pathid_filetype.insert(pathid_counter, *filetype);
                map_pathid_mimeguess.insert(pathid_counter, *mimeguess);
                map_pathid_results.insert(pathid_counter, processpathresult);
            }
            _ =>
            {
                dpof!("paths_invalid_results.push({:?})", processpathresult);
                map_pathid_results_invalid.insert(pathid_counter, processpathresult);
            },
        };
    }

    // preprint the prepended name or path
    type PathId_PrependName = HashMap<PathId, String>;
    let mut pathid_to_prependname: PathId_PrependName;
    let mut prependname_width: usize = 0;
    if cli_opt_prepend_filename {
        if cli_opt_prepend_file_align {
            for path in map_pathid_path.values() {
                let bname: String = basename(path);
                prependname_width = std::cmp::max(
                    prependname_width, unicode_width::UnicodeWidthStr::width(bname.as_str())
                );
            }
        }
        pathid_to_prependname = PathId_PrependName::with_capacity(file_count);
        for (pathid, path) in map_pathid_path.iter() {
            let bname: String = basename(path);
            let prepend: String = format!("{0:<1$}:", bname, prependname_width);
            pathid_to_prependname.insert(*pathid, prepend);
        }
    } else if cli_opt_prepend_filepath {
        if cli_opt_prepend_file_align {
            for path in map_pathid_path.values() {
                prependname_width = std::cmp::max(
                    prependname_width, unicode_width::UnicodeWidthStr::width(path.as_str())
                );
            }
        }
        pathid_to_prependname = PathId_PrependName::with_capacity(file_count);
        for (pathid, path) in map_pathid_path.iter() {
            let prepend: String = format!("{0:<1$}:", path, prependname_width);
            pathid_to_prependname.insert(*pathid, prepend);
        }
    }
    else {
        pathid_to_prependname = PathId_PrependName::with_capacity(0);
    }

    // create one thread per file path, each thread named for the file basename

    //
    // prepare per-thread data keyed by `FPath`
    // create necessary channels for each thread
    // launch each thread
    //
    // pre-created mapping for calls to `select.recv` and `select.select`
    type Index_Select = HashMap<usize, PathId>;
    // mapping of PathId to received data. Most important collection for the remainder
    // of this function.
    let mut map_pathid_chanrecvdatum = Map_PathId_ChanRecvDatum::new();
    // mapping PathId to colors for printing.
    let mut map_pathid_color = Map_PathId_Color::with_capacity(file_count);
    // mapping PathId to a `Summary` for `--summary`
    let mut map_pathid_summary = Map_PathId_Summary::with_capacity(file_count);
    // "mapping" of PathId to select index, used in `recv_many_data`
    let mut index_select = Index_Select::with_capacity(file_count);

    // initialize processing channels/threads, one per file path
    for pathid in map_pathid_path.keys() {
        map_pathid_color.insert(*pathid, color_rand());
    }
    //let channel_message_queue_sz: usize = 10;
    for (pathid, path) in map_pathid_path.iter() {
        let (filetype, _) = match map_pathid_results.get(pathid) {
            Some(processpathresult) => {
                match processpathresult {
                    ProcessPathResult::FileValid(path, _m, filetype) => (filetype, path),
                    val => {
                        eprintln!("ERROR: unhandled ProcessPathResult {:?}", val);
                        continue;
                    },
                }
            }
            None => {
                panic!("bad pathid {}", pathid);
            }
        };
        let thread_data: Thread_Init_Data4 = (
            path.clone().to_owned(),
            *pathid,
            *filetype,
            blocksz,
            *filter_dt_after_opt,
            *filter_dt_before_opt,
            tz_offset,
        );
        let (chan_send_dt, chan_recv_dt): (Chan_Send_Datum, Chan_Recv_Datum) = crossbeam_channel::unbounded();
        dpof!("map_pathid_chanrecvdatum.insert({}, ...);", pathid);
        map_pathid_chanrecvdatum.insert(*pathid, chan_recv_dt);
        let basename_: FPath = basename(path);
        match thread::Builder::new().name(basename_.clone()).spawn(
                move || exec_4(chan_send_dt, thread_data)
            ) {
                    Ok(_joinhandle) => {},
                    Err(err) => {
                        eprintln!("ERROR: thread.name({:?}).spawn() pathid {} failed {:?}", basename_, pathid, err);
                        map_pathid_chanrecvdatum.remove(pathid);
                        map_pathid_color.remove(pathid);
                        continue;
                    }
                }
    }
    if map_pathid_chanrecvdatum.is_empty() {
        eprintln!("ERROR: map_pathid_chanrecvdatum.is_empty(); nothing to do.");
        return;
    }

    type Recv_Result4 = std::result::Result<Chan_Datum, crossbeam_channel::RecvError>;

    /// run `.recv` on many Receiver channels simultaneously using `crossbeam_channel::Select`
    /// https://docs.rs/crossbeam-channel/0.5.1/crossbeam_channel/struct.Select.html
    ///
    /// DONE: TODO: [2022/03/26] to avoid sending a new `FPath` on each channel send, instead have a single
    ///       Map<u32, FPath> that is referred to on "each side". The `u32` is the lightweight key sent
    ///       along the channel.
    ///       This mapping <u32, FPath> could be used for all other maps with keys `FPath`...
    ///       would a global static lookup map make this easier? No need to pass around instances of `Map<u32, FPath>`.
    ///
    #[inline(always)]
    fn recv_many_chan<'a>(
        pathid_chans: &'a Map_PathId_ChanRecvDatum,
        map_index_pathid: &mut Index_Select,
        filter_: &Set_PathId,
    ) -> Option<(PathId, Recv_Result4)> {
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
        let chan: &Chan_Recv_Datum = match pathid_chans.get(pathid) {
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
    // main coordination loop (e.g. "main game loop")
    // process the "receiving sysline" channels from the running threads
    // print the earliest available sysline
    //
    // TODO: [2022/03/24] change `map_pathid_datum` to `HashMap<FPath, (SyslineP, is_last)>` (`map_path_slp`);
    //       currently it's confusing that there is a special handler for `Summary` (`map_pathid_summary`),
    //       but not an equivalent `map_path_slp`.
    //       In other words, break apart the passed `Chan_Datum` to the more specific maps.
    //
    let mut map_pathid_datum = Map_PathId_Datum::new();
    // `set_pathid_datum` shadows `map_pathid_datum` for faster filters in `recv_many_chan`
    let mut set_pathid = Set_PathId::with_capacity(file_count);
    let mut map_pathid_sumpr = Map_PathId_SummaryPrint::new();
    // crude debugging stats
    let mut chan_recv_ok: Count = 0;
    let mut chan_recv_err: Count = 0;
    // the `SummaryPrinted` tallying the entire process (tallies each recieved `SyslineP`)
    let mut summaryprinted: SummaryPrinted = SummaryPrinted::default();
    let color_default = Color::White;

    // mapping PathId to colors for printing.
    let mut map_pathid_printer = Map_PathId_PrinterSysline::with_capacity(file_count);
    // initialize the printers, one per file
    for pathid in map_pathid_path.keys() {
        let color_: &Color = map_pathid_color.get(pathid).unwrap_or(&color_default);
        let prepend_file: Option<String> = match cli_opt_prepend_filename || cli_opt_prepend_filepath {
            true => {
                Some(pathid_to_prependname.get(pathid).unwrap().clone())
            },
            false => None,
        };
        let prepend_date_format: Option<String> = match cli_opt_prepend_local || cli_opt_prepend_utc {
            true => Some(CLI_OPT_PREPEND_FMT.to_string()),
            false => None,
        };
        let prepend_date_offset: Option<FixedOffset> = match (cli_opt_prepend_local, cli_opt_prepend_utc) {
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

    // main thread "game loop"
    //
    // here is the program coordination of file processing threads to print
    // Syslines ordered by datetime
    //
    // channels that should be disconnected per "game loop" loop iteration
    let mut disconnect = Vec::<PathId>::with_capacity(file_count);
    loop {
        disconnect.clear();

        if cfg!(debug_assertions) {
            dpo!("processing_loop: map_pathid_datum.len() {}", map_pathid_datum.len());
            for (pathid, _datum) in map_pathid_datum.iter() {
                let _path: &FPath = map_pathid_path.get(pathid).unwrap();
                dpo!("processing_loop: map_pathid_datum: thread {} {} has data", _path, pathid);
            }
            dpo!("processing_loop: map_pathid_chanrecvdatum.len() {}", map_pathid_chanrecvdatum.len());
            for (pathid, _chanrdatum) in map_pathid_chanrecvdatum.iter() {
                let _path: &FPath = map_pathid_path.get(pathid).unwrap();
                dpo!("processing_loop: map_pathid_chanrecvdatum: thread {} {} channel messages {}", _path, pathid, _chanrdatum.len());
            }
        }

        if map_pathid_chanrecvdatum.len() != map_pathid_datum.len() {
            // if...
            // `map_path_recv_dt` does not have a `Chan_Recv_Datum` (and thus a `SyslineP` and
            // thus a `DatetimeL`) for every channel (file being processed).
            // (Every channel must return a `DatetimeL` to to then compare *all* of them, see which is earliest).
            // So call `recv_many_chan` to check if any channels have a new `Chan_Recv_Datum` to
            // provide.

            let pathid: PathId;
            let result1: Recv_Result4;
            (pathid, result1) = match recv_many_chan(&map_pathid_chanrecvdatum, &mut index_select, &set_pathid) {
                Some(val) => val,
                None => {
                    eprintln!("ERROR: recv_many_chan returned None which is unexpected");
                    continue;
                }
            };
            match result1 {
                Ok(chan_datum) => {
                    dpo!("processing_loop: B crossbeam_channel::Found for PathId {:?};", pathid);
                    if let Some(summary) = chan_datum.1 {
                        assert!(chan_datum.0.is_none(), "Chan_Datum Some(Summary) and Some(SyslineP); should only have one Some(). PathId {:?}", pathid);
                        summary_update(&pathid, summary, &mut map_pathid_summary);
                        dpo!("processing_loop: B will disconnect channel {:?}", pathid);
                        // receiving a Summary must be the last data sent on the channel
                        disconnect.push(pathid);
                    } else {
                        assert!(chan_datum.0.is_some(), "Chan_Datum None(Summary) and None(SyslineP); should have one Some(). PathId {:?}", pathid);
                        map_pathid_datum.insert(pathid, chan_datum);
                        set_pathid.insert(pathid);
                    }
                    chan_recv_ok += 1;
                }
                Err(crossbeam_channel::RecvError) => {
                    dpo!("processing_loop: B crossbeam_channel::RecvError, will disconnect channel for PathId {:?};", pathid);
                    // this channel was closed by the sender
                    disconnect.push(pathid);
                    chan_recv_err += 1;
                }
            }
        } else {
            // else...
            // There is a DateTime available for *every* channel (one channel is one File Processing
            // thread). The datetimes can be compared among all remaining files. The sysline with
            // the earliest datetime is printed.

            if cfg!(debug_assertions) {
                for (_i, (_k, _v)) in map_pathid_chanrecvdatum.iter().enumerate() {
                    dpo!("{} A1 map_pathid_chanrecvdatum[{:?}] = {:?}", _i, _k, _v);
                }
                for (_i, (_k, _v)) in map_pathid_datum.iter().enumerate() {
                    dpo!("{} A1 map_pathid_datum[{:?}] = {:?}", _i, _k, _v);
                }
            }

            // (path, channel data) for the sysline with earliest datetime ("minimum" datetime)
            //
            // here is part of the "sorting" of syslines process by datetime.
            // In case of tie datetime values, the tie-breaker will be order of `BTreeMap::iter_mut` which
            // iterates in order of key sort. https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.iter_mut
            //
            // XXX: assume `unwrap` will never fail
            //
            // XXX: my small investigation into `min`, `max`, `min_by`, `max_by`
            //      https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=a6d307619a7797b97ef6cfc1635c3d33
            //
            let pathid: &PathId;
            let chan_datum: &mut Chan_Datum;
            (pathid, chan_datum) = match map_pathid_datum.iter_mut().min_by(
                |x, y|
                    x.1.0.as_ref().unwrap().dt().cmp(y.1.0.as_ref().unwrap().dt())
            ) {
                Some(val) => (
                    val.0, val.1
                ),
                None => {
                    eprintln!("ERROR map_pathid_datum.iter_mut().min_by() returned None");
                    // XXX: not sure what else to do here
                    continue;
                }
            };

            if let Some(summary) = chan_datum.1.clone() {
                dpof!("A2 chan_datum has Summary, PathId: {:?}", pathid);
                assert!(chan_datum.0.is_none(), "Chan_Datum Some(Summary) and Some(SyslineP); should only have one Some(). PathId: {:?}", pathid);
                if cli_opt_summary {
                    summary_update(pathid, summary, &mut map_pathid_summary);
                }
                dpof!("A2 will disconnect channel {:?}", pathid);
                // receiving a Summary implies the last data was sent on the channel
                disconnect.push(*pathid);
            } else {
                // is last sysline of the file?
                let is_last: bool = chan_datum.2;
                // Sysline of interest
                let syslinep: &SyslineP = chan_datum.0.as_ref().unwrap();
                dpof!("A3 printing @[{}, {}] PathId: {:?}", syslinep.fileoffset_begin(), syslinep.fileoffset_end(), pathid);
                // print the sysline!
                let printer: &mut PrinterSysline = map_pathid_printer.get_mut(pathid).unwrap();
                match printer.print_sysline(syslinep) {
                        Ok(_) => {},
                        Err(_err) => {
                            p_err!("failed to print; TODO abandon processing for PathId {:?}", pathid);
                            // TODO: 2022/04/09 if printing fails, then all future prints are very likely to fail
                            //       so shutdown program
                        }
                }
                // without printing this extra '\n' then for files with no terminating '\n' the
                // next printed sysline (from a different file) will be on the same line,
                // i.e. it'll look unreable and jenky
                if is_last {
                    write_stdout(&NLu8a);
                    summaryprinted.bytes += 1;
                }
                if cli_opt_summary {
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
        // remove channels (and keys) that have been disconnected
        for pathid in disconnect.iter() {
            dpof!("C map_pathid_chanrecvdatum.remove({:?});", pathid);
            map_pathid_chanrecvdatum.remove(pathid);
            dpof!("C pathid_to_prependname.remove({:?});", pathid);
            pathid_to_prependname.remove(pathid);
        }
        // are there any channels to receive from?
        if map_pathid_chanrecvdatum.is_empty() {
            dpof!("D map_pathid_chanrecvdatum.is_empty(); no more channels to receive from!");
            break;
        }
        dpof!("D map_pathid_chanrecvdatum: {:?}", map_pathid_chanrecvdatum);
        dpof!("D map_pathid_datum: {:?}", map_pathid_datum);
        dpof!("D set_pathid: {:?}", set_pathid);
    } // end loop

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
        eprintln!("{:?}", summaryprinted);
        eprintln!("Datetime Filters: -a {:?} -b {:?}", filter_dt_after_opt, filter_dt_before_opt);
        eprintln!("Channel Receive ok {}, err {}", chan_recv_ok, chan_recv_err);
    }

    dpof!("E chan_recv_ok {:?} _count_recv_di {:?}", chan_recv_ok, chan_recv_err);
    dpnf!();
}

// -------------------------------------------------------------------------------------------------

// TODO: move these functions to a neighboring file `print_summary.rs`

/// print the filepath name (one line)
fn print_filepath(
    path: &FPath,
    filetype: &FileType,
    mimeguess: &MimeGuess,
    color: &Color,
    color_choice: &ColorChoice
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

/// print the `&SummaryOpt` (multiple lines)
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
            for patt in summary.SyslineReader_patterns.iter() {
                let dtpd: &DateTime_Parse_Data = &DATETIME_PARSE_DATAS[*patt.0];
                eprintln!("{}{}   @{} {} {:?}", OPT_SUMMARY_PRINT_INDENT, OPT_SUMMARY_PRINT_INDENT_UNDER, patt.0, patt.1, dtpd);
            }
            match summary.SyslogProcessor_missing_year {
                Some(year) => {
                    eprintln!("{}{}datetime format missing year; estimated year of last sysline {:?}", OPT_SUMMARY_PRINT_INDENT, OPT_SUMMARY_PRINT_INDENT_UNDER, year);
                }
                None => {}
            }
        },
        None => {
            // TODO: [2022/06/07] print filesz
            eprintln!("{}Summary Processed: None", OPT_SUMMARY_PRINT_INDENT);
        }
    }
}

/// print the `&SummaryPrinted_Opt` (one line)
fn print_summary_opt_printed(
    summary_print_opt: &SummaryPrinted_Opt,
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_print_opt {
        Some(summary_print) => {
            eprint!("{}Summary Printed  : ", OPT_SUMMARY_PRINT_INDENT);
            summary_print.print_colored_stderr(Some(*color_choice), summary_opt);
        },
        None => {
            eprint!("{}Summary Printed  : ", OPT_SUMMARY_PRINT_INDENT);
            SummaryPrinted::default().print_colored_stderr(Some(*color_choice), summary_opt);
        }
    }
    eprintln!();
}

/// print the various `Summary` caching and storage statistics (multiple lines)
fn print_cache_stats(summary_opt: &SummaryOpt) {
    if summary_opt.is_none() {
        return;
    }

    fn ratio64(a: &u64, b: &u64) -> f64 {
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
    let wide: usize = summary.max_hit_miss().to_string().len();
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
    ratio = ratio64(
        &summary.SyslineReader_syslines_hit,
        &summary.SyslineReader_syslines_miss,
    );
    eprintln!(
        "{}storage: SyslineReader::find_sysline() syslines                        : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.SyslineReader_syslines_hit,
        summary.SyslineReader_syslines_miss,
        ratio,
        wide = wide,
    );
    // SyslineReader::_syslines_by_range
    ratio = ratio64(
        &summary.SyslineReader_syslines_by_range_hit,
        &summary.SyslineReader_syslines_by_range_miss,
    );
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
    ratio = ratio64(
        &summary.LineReader_lines_hit,
        &summary.LineReader_lines_miss,
    );
    eprintln!(
        "{}storage: LineReader::find_line() lines                                 : hit {:wide$}, miss {:wide$}, ratio {:1.2}",
        OPT_SUMMARY_PRINT_INDENT,
        summary.LineReader_lines_hit,
        summary.LineReader_lines_miss,
        ratio,
        wide = wide,
    );
    // LineReader::_find_line_lru_cache
    ratio = ratio64(
        &summary.LineReader_find_line_lru_cache_hit,
        &summary.LineReader_find_line_lru_cache_miss,
    );
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
    ratio = ratio64(
        &summary.BlockReader_read_blocks_hit,
        &summary.BlockReader_read_blocks_miss,
    );
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

/// print the various `Summary` drop error statistics (multiple lines)
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
    let wide: usize = summary.max_drop().to_string().len();
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

/// print the `Summary.Error_`, if any (one line)
fn print_error(summary_opt: &SummaryOpt, color_choice: &ColorChoice) {
    match summary_opt.as_ref() {
        Some(summary_) => {
            match &summary_.Error_ {
                Some(err_string) => {
                    eprint!("{}Error: ", OPT_SUMMARY_PRINT_INDENT);
                    #[allow(clippy::single_match)]
                    match print_colored_stderr(COLOR_ERROR, Some(*color_choice), err_string.as_bytes()) {
                        Err(_err) => {},
                        _ => {},
                    }
                    eprintln!();
                },
                None => {},
            }        
        }
        None => {}
    }
}

/// for one file, print the `Summary` and `SummaryPrinted` (multiple lines)
fn print_file_summary(
    path: &FPath,
    filetype: &FileType,
    mimeguess: &MimeGuess,
    summary_opt: &SummaryOpt,
    summary_print_opt: &SummaryPrinted_Opt,
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
    print_error(summary_opt, color_choice);
}

/// printing for CLI option `--summary`
/// print each files' `Summary` and `SummaryPrinted`
///
/// helper to `processing_loop`
#[allow(clippy::too_many_arguments)]
fn print_all_files_summaries(
    map_pathid_path: &Map_PathId_FPath,
    map_pathid_filetype: &Map_PathId_FileType,
    map_pathid_mimeguess: &Map_PathId_MimeGuess,
    map_pathid_color: &Map_PathId_Color,
    map_pathid_summary: &mut Map_PathId_Summary,
    map_pathid_sumpr: &mut Map_PathId_SummaryPrint,
    color_choice: &ColorChoice,
    color_default: &Color,
) {
    for (pathid, path) in map_pathid_path.iter() {
        let color: &Color = map_pathid_color.get(pathid).unwrap_or(color_default);
        let filetype: &FileType = map_pathid_filetype.get(pathid).unwrap_or(&FileType::FileUnknown);
        let mimeguess_default: MimeGuess = MimeGuess::from_ext("");
        let mimeguess: &MimeGuess = map_pathid_mimeguess.get(pathid).unwrap_or(&mimeguess_default);
        let summary_opt: SummaryOpt = map_pathid_summary.remove(pathid);
        let summary_print_opt: SummaryPrinted_Opt = map_pathid_sumpr.remove(pathid);
        print_file_summary(
            path,
            filetype,
            mimeguess,
            &summary_opt,
            &summary_print_opt,
            color,
            color_choice,
        );
    }
}

/// printing for CLI option `--summary`
/// print an entry for invalid files
///
/// helper to `processing_loop`
fn print_files_processpathresult(
    map_pathid_result: &Map_PathId_ProcessPathResult,
    color_choice: &ColorChoice,
    color_default: &Color,
    color_error: &Color,
) {
    // local helper
    fn print_(buffer: String, color_choice: &ColorChoice, color: &Color) {
        match print_colored_stderr(*color, Some(*color_choice), buffer.as_bytes()) {
            Ok(()) => {},
            Err(err) => {
                eprintln!("ERROR: {:?}", err);
            }
        };
    }

    for (_pathid, result) in map_pathid_result.iter() {
        match result {
            ProcessPathResult::FileValid(path, mimeguess, _filetype) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
            },
            ProcessPathResult::FileErrNoPermissions(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(no permissions)".to_string(), color_choice, color_error);
            },
            ProcessPathResult::FileErrNotSupported(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not supported)".to_string(), color_choice, color_error);
            },
            ProcessPathResult::FileErrNotParseable(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not parseable)".to_string(), color_choice, color_error);
            },
            ProcessPathResult::FileErrNotAFile(path, mimeguess) => {
                print_(format!("File: {} {:?} ", path, mimeguess), color_choice, color_default);
                print_("(not a file)".to_string(), color_choice, color_error);
            },
        }
        eprintln!();
    }
}
