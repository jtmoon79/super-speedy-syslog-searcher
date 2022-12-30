// src/tests/syslogprocessor_tests.rs

//! tests for `syslogprocessor.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{Count, FPath, FileOffset};

use crate::data::sysline::SyslineP;

use crate::debug::helpers::{create_temp_file, ntf_fpath, NamedTempFile};

use crate::readers::blockreader::BlockSz;

use crate::readers::filepreprocessor::fpath_to_filetype_mimeguess;

use crate::data::datetime::{
    datetime_parse_from_str, DateTimeL, DateTimeLOpt, DateTimePattern_str, FixedOffset, SystemTime,
};

use crate::readers::syslinereader::ResultS3SyslineFind;

use crate::readers::syslogprocessor::{FileProcessingResultBlockZero, SyslogProcessor};

use crate::tests::common::{
    TZO_0,
};

extern crate const_format;
use const_format::concatcp;

extern crate filetime;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_gt,
};

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06/01] these are repeated in several `_test.rs` files, declare them in
//       one common `common_tests.rs` file

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const SZ: BlockSz = SyslogProcessor::BLOCKSZ_MIN;

//
// NTF5
//

// the five lines of data that makes up file `NTF5`
const NTF5_DATA_LINE0: &str = "Jan 1 01:00:11 5a\n";
const NTF5_DATA_LINE1: &str = "Feb 29 02:00:22 5b\n";
const NTF5_DATA_LINE2: &str = "Mar 3 03:00:33 5c\n";
const NTF5_DATA_LINE3: &str = "Apr 4 04:00:44 5d\n";

const NTF5_DATA_LINE4: &str = "May 5 05:00:55 5e\n";
/// Unix epoch time for time `NTF5_DATA_LINE4` at UTC
const NTF5_MTIME_UNIXEPOCH: i64 = 957502855;

const NTF5_DATA: &str =
    concatcp!(NTF5_DATA_LINE0, NTF5_DATA_LINE1, NTF5_DATA_LINE2, NTF5_DATA_LINE3, NTF5_DATA_LINE4,);

const NTF5_LINE2_DATETIME_STR: &str = "Mar 3 03:00:00 +0000";
const NTF5_LINE2_DATETIME_PATTERN: &DateTimePattern_str = "%b %e %H:%M:%S %z";

//
// NTF3
//

const NTF3_DATA_LINE0: &str = "Jan 1 01:00:00 2000 A3\n";
const NTF3_DATA_LINE1: &str = "Feb 2 02:00:00 2000 B3\n";
const NTF3_DATA_LINE2: &str = "Mar 3 03:00:00 2000 C3\n";

const NTF3_DATA: &str = concatcp!(NTF3_DATA_LINE0, NTF3_DATA_LINE1, NTF3_DATA_LINE2,);

const NTF3_LINE1_DATETIME_STR: &str = "Feb 2 02:00:00 2000 +0000";
const NTF3_LINE1_DATETIME_PATTERN: &DateTimePattern_str = "%b %e %H:%M:%S %Y %z";

//
// NTF9
//

// the nine lines of data that makes up file `NTF9`
const NTF9_DATA_LINE0: &str = "Jan 11 01:31:21 2000 9à\n";
const NTF9_DATA_LINE1: &str = "Feb 29 02:32:22 2000 9bb\n";
const NTF9_DATA_LINE2: &str = "Mar 13 03:33:23 2000 9ccc\n";
const NTF9_DATA_LINE3: &str = "Apr 14 04:34:24 2000 9dddd\n";
const NTF9_DATA_LINE4: &str = "May 15 05:35:25 2000 9èèèèè\n";
const NTF9_DATA_LINE5: &str = "Jun 16 05:36:26 2000 9ffffff\n";
const NTF9_DATA_LINE6: &str = "Jul 17 05:37:27 2000 9ggggggg\n";
const NTF9_DATA_LINE7: &str = "Aug 18 05:38:28 2000 9hhhhhhhh\n";
const NTF9_DATA_LINE8: &str = "Sep 19 05:39:29 2000 9ììììììììì\n";

const NTF9_DATA: &str =
    concatcp!(
        NTF9_DATA_LINE0,
        NTF9_DATA_LINE1,
        NTF9_DATA_LINE2,
        NTF9_DATA_LINE3,
        NTF9_DATA_LINE4,
        NTF9_DATA_LINE5,
        NTF9_DATA_LINE6,
        NTF9_DATA_LINE7,
        NTF9_DATA_LINE8,
    );

const NTF9_DATA_LINE0_OFFSET: usize = 0;
const NTF9_DATA_LINE1_OFFSET: usize = NTF9_DATA_LINE0_OFFSET
    + NTF9_DATA_LINE0.as_bytes().len();
const NTF9_DATA_LINE2_OFFSET: usize = NTF9_DATA_LINE1_OFFSET
    + NTF9_DATA_LINE1.as_bytes().len();
const NTF9_BLOCKSZ_MIN: BlockSz = (NTF9_DATA_LINE2_OFFSET + 1) as BlockSz;

//
// NTF7_2
//

// seven syslines with two lines each that makes up file `NTF7_2`
const NTF7_2_DATA_LINE0: &str = "Jan 11 01:31:21 2000 6-3 à\n";
const NTF7_2_DATA_LINE1: &str = "ββββββββββββββββββββββββββ\n";
const NTF7_2_DATA_LINE2: &str = "Mar 13 03:33:23 2000 6-3 ccc\n";
const NTF7_2_DATA_LINE3: &str = "ΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔ\n";
const NTF7_2_DATA_LINE4: &str = "May 15 05:35:25 2000 6-3 èèèèè\n";
const NTF7_2_DATA_LINE5: &str = "ζζζζζζζζζζζζζζζζζζζζζζζζζζζζζζ\n";
const NTF7_2_DATA_LINE6: &str = "Jul 17 07:37:27 2000 6-3 ggggggg\n";
const NTF7_2_DATA_LINE7: &str = "ΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗ\n";
const NTF7_2_DATA_LINE8: &str = "Sep 19 09:39:29 2000 6-3 ììììììììì\n";
const NTF7_2_DATA_LINE9: &str = "ιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιι\n";
const NTF7_2_DATA_LINE10: &str = "Nov 21 11:41:41 2000 6-3 ììììììììì\n";
const NTF7_2_DATA_LINE11: &str = "ιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιι\n";
const NTF7_2_DATA_LINE12: &str = "Jan 31 01:02:03 2001 6-3 KKKKKKKKKK\n";
const NTF7_2_DATA_LINE13: &str = "ΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛ\n";

const NTF7_2_DATA: &str =
    concatcp!(
        NTF7_2_DATA_LINE0,
        NTF7_2_DATA_LINE1,
        NTF7_2_DATA_LINE2,
        NTF7_2_DATA_LINE3,
        NTF7_2_DATA_LINE4,
        NTF7_2_DATA_LINE5,
        NTF7_2_DATA_LINE6,
        NTF7_2_DATA_LINE7,
        NTF7_2_DATA_LINE8,
        NTF7_2_DATA_LINE9,
        NTF7_2_DATA_LINE10,
        NTF7_2_DATA_LINE11,
        NTF7_2_DATA_LINE12,
        NTF7_2_DATA_LINE13,
    );

const NTF7_2_DATA_LINE0_OFFSET: usize = 0;
const NTF7_2_DATA_LINE1_OFFSET: usize = NTF7_2_DATA_LINE0_OFFSET
    + NTF7_2_DATA_LINE0.as_bytes().len();
const NTF7_2_DATA_LINE2_OFFSET: usize = NTF7_2_DATA_LINE1_OFFSET
    + NTF7_2_DATA_LINE1.as_bytes().len();
const NTF7_2_DATA_LINE3_OFFSET: usize = NTF7_2_DATA_LINE2_OFFSET
    + NTF7_2_DATA_LINE2.as_bytes().len();
const NTF7_2_DATA_LINE4_OFFSET: usize = NTF7_2_DATA_LINE3_OFFSET
    + NTF7_2_DATA_LINE3.as_bytes().len();
const NTF7_2_BLOCKSZ_MIN: BlockSz = (NTF7_2_DATA_LINE4_OFFSET + NTF7_2_DATA_LINE4_OFFSET % 2 + 2) as BlockSz;

lazy_static! {
    static ref TIMEZONE_0: FixedOffset = FixedOffset::west_opt(0).unwrap();

    //
    // NTF5
    //

    // a `DateTimeL` instance a few hours before `NTF5_DATA_LINE2` and after
    // `NTF5_DATA_LINE1`
    static ref NTF5_DATA_LINE2_BEFORE: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2000-03-01T12:00:00-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    // a `DateTimeL` instance some hours after `NTF5_DATA_LINE4`
    static ref NTF5_DATA_LINE4_AFTER: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2000-05-05T23:00:00-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    static ref NTF5_LINE2_DATETIME: DateTimeL = {
        match datetime_parse_from_str(
            NTF5_LINE2_DATETIME_STR, NTF5_LINE2_DATETIME_PATTERN, true, &TIMEZONE_0
        ) {
            Some(dt) => dt,
            None => {
                panic!("bad parameters to datetime_parse_from_str for NTF5_LINE2_DATETIME_STR");
            }
        }
    };

    static ref NTF5: NamedTempFile = {
        let ntf = create_temp_file(NTF5_DATA);
        // set the file's modified time to `NTF5_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF5_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF5_PATH: FPath = {
        ntf_fpath(&NTF5)
    };

    //
    // NTF3
    //

    static ref NTF3_LINE1_DATETIME: DateTimeL = {
        match datetime_parse_from_str(
            NTF3_LINE1_DATETIME_STR, NTF3_LINE1_DATETIME_PATTERN, true, &TIMEZONE_0
        ) {
            Some(dt) => dt,
            None => {
                panic!("bad parameters to datetime_parse_from_str for NTF3_LINE1_DATETIME_STR");
            }
        }
    };

    static ref NTF3: NamedTempFile = {
        create_temp_file(NTF3_DATA)
    };

    static ref NTF3_PATH: FPath = {
        ntf_fpath(&NTF3)
    };

    //
    // NTF9
    //

    static ref NTF9: NamedTempFile = {
        create_temp_file(NTF9_DATA)
    };

    static ref NTF9_PATH: FPath = {
        ntf_fpath(&NTF9)
    };

    //
    // NTF7_2
    //

    static ref NTF7_2: NamedTempFile = {
        create_temp_file(NTF7_2_DATA)
    };

    static ref NTF7_2_PATH: FPath = {
        ntf_fpath(&NTF7_2)
    };

    //

    // 76208400
    // Thursday, June 1, 1972 1:00:00 AM GMT+00:00
    // Wednesday, May 31, 1972 6:00:00 PM GMT-07:00
    static ref SYSTEMTIME_1972_06_01: SystemTime = {
        let duration: std::time::Duration = std::time::Duration::from_secs(76208400);

        SystemTime::UNIX_EPOCH.checked_add(duration).unwrap()
    };

    // 107744400
    // Friday, June 1, 1973 1:00:00 AM GMT+00:00
    // Thursday, May 31, 1973 6:00:00 PM GMT-07:00
    static ref SYSTEMTIME_1973_06_01: SystemTime = {
        let duration: std::time::Duration = std::time::Duration::from_secs(107744400);

        SystemTime::UNIX_EPOCH.checked_add(duration).unwrap()
    };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper to wrap the match and panic checks
fn new_SyslogProcessor(
    path: &FPath,
    blocksz: BlockSz,
) -> SyslogProcessor {
    let tzo: FixedOffset = *TZO_0;
    let (filetype, _mimeguess) = fpath_to_filetype_mimeguess(path);
    match SyslogProcessor::new(path.clone(), filetype, blocksz, tzo, None, None) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslogProcessor::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// test `SyslogProcessor::new`
#[test]
fn test_SyslogProcessor_new1() {
    let ntf = create_temp_file("");
    let path = ntf_fpath(&ntf);
    let slp = new_SyslogProcessor(&path, SZ);
    eprintln!("{:?}", slp);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_process_missing_year_1972() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);
    slp.process_missing_year(*SYSTEMTIME_1972_06_01, &None);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_find_sysline() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);
    let mut fo: FileOffset = 0;
    loop {
        let result = slp.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("Error {:?}", err);
            }
        }
    }
}

#[test]
fn test_find_sysline_between_datetime_filters_Found() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);

    let result = slp.find_sysline_between_datetime_filters(0);
    match result {
        ResultS3SyslineFind::Found((_fo, _syslinep)) => {}
        ResultS3SyslineFind::Done => {
            panic!("Unexpected Done");
        }
        ResultS3SyslineFind::Err(err) => {
            panic!("Error {:?}", err);
        }
    }
}

#[test]
fn test_find_sysline_between_datetime_filters_Done() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);
    let fo: FileOffset = NTF5_DATA.len() as FileOffset;

    let result = slp.find_sysline_between_datetime_filters(fo);
    match result {
        ResultS3SyslineFind::Found((_fo, _syslinep)) => {
            panic!("Unexpected Found");
        }
        ResultS3SyslineFind::Done => {}
        ResultS3SyslineFind::Err(err) => {
            panic!("Error {:?}", err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[test_case(0x400)]
#[test_case(0x10 => panics)] // Issue #22
fn test_processing_stage_1_blockzero_analysis(blocksz: BlockSz) {
    let mut slp = new_SyslogProcessor(&NTF3_PATH, blocksz);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage0 {:?}", result);
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage1 {:?}", result);
        }
    }
}

#[test]
fn test_processing_stages_0_5() {
    let mut slp = new_SyslogProcessor(&NTF3_PATH, 0x400);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage0 {:?}", result);
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage1 {:?}", result);
        }
    }

    match slp.process_stage2_find_dt(&None) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage2 {:?}", result);
        }
    }

    match slp.process_stage3_stream_syslines() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage3 {:?}", result);
        }
    }

    let mut fo: FileOffset = 0;
    loop {
        let result = slp.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("Error {:?}", err);
            }
        }
    }

    let _summary = slp.process_stage4_summary();
}

// test files without a year and a `dt_filter_after_opt` do not process
// the entire file, only back to `dt_filter_after_opt`
#[test_case(&NTF5_PATH, &None, 5)]
#[test_case(&NTF5_PATH, &NTF5_DATA_LINE2_BEFORE, 4)]
#[test_case(&NTF5_PATH, &NTF5_DATA_LINE4_AFTER, 1)]
fn test_process_stage2_find_dt_and_missing_year(
    path: &FPath,
    filter_dt_after_opt: &DateTimeLOpt,
    count_syslines_expect: Count,
) {
    let mut slp = new_SyslogProcessor(path, 0xFFFF);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage0 {:?}", result);
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage1 {:?}", result);
        }
    }

    match slp.process_stage2_find_dt(filter_dt_after_opt) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage2 {:?}", result);
        }
    }

    assert_eq!(slp.count_syslines_stored(), count_syslines_expect);
}

// -------------------------------------------------------------------------------------------------

#[test_case(&NTF9_PATH, NTF9_BLOCKSZ_MIN)]
#[test_case(&NTF7_2_PATH, NTF7_2_BLOCKSZ_MIN)]
fn test_stage0to3_drop_data(
    path: &FPath,
    blocksz: BlockSz,
) {
    let mut slp = new_SyslogProcessor(path, blocksz);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage0 {:?}", result);
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage1 {:?}", result);
        }
    }

    match slp.process_stage2_find_dt(&None) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage2 {:?}", result);
        }
    }

    match slp.find_sysline_between_datetime_filters(0) {
        ResultS3SyslineFind::Found(_) => {}
        ResultS3SyslineFind::Done => {
            panic!("Unexpected Done");
        }
        ResultS3SyslineFind::Err(err) => {
            panic!(
                "ERROR: SyslogProcessor.find_sysline_between_datetime_filters(0) Path {:?} Error {}",
                path, err
            );
        }
    }

    match slp.process_stage3_stream_syslines() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage3 {:?}", result);
        }
    }

    let mut fo: FileOffset = 0;
    let mut syslinep_last_opt: Option<SyslineP> = None;
    loop {
        match slp.find_sysline(fo) {
            ResultS3SyslineFind::Found((fo_, syslinep)) => {
                fo = fo_;
                if let Some(syslinep_) = syslinep_last_opt {
                    slp.drop_data_try(&syslinep_);
                }
                syslinep_last_opt = Some(syslinep);
            }
            ResultS3SyslineFind::Done => break,
            ResultS3SyslineFind::Err(err) => {
                panic!(
                    "ERROR: SyslogProcessor.find_sysline({}) Path {:?} Error {}",
                    fo, path, err
                );
            }
        }
    }

    let dropped_syslines = slp.dropped_syslines();
    assert_gt!(dropped_syslines.len(), 0, "Expected *some* dropped Syslines but zero were dropped, blocksz {:?}, filesz {:?}", blocksz, slp.filesz());
    let dropped_lines = slp.dropped_lines();
    assert_gt!(dropped_lines.len(), 0, "Expected *some* dropped Lines but zero were dropped, blocksz {:?}, filesz {:?}", blocksz, slp.filesz());
    let dropped_blocks = slp.dropped_blocks();
    assert_gt!(dropped_blocks.len(), 0, "Expected *some* dropped Blocks but zero were dropped, blocksz {:?}, filesz {:?}", blocksz, slp.filesz());
}

/*

/// test `SyslogProcessor::blockzero_analysis`
#[allow(non_snake_case)]
fn test_blockzero_analysis(
    path: &FPath,
    blocksz: BlockSz,
    expect_result: FileProcessingResultBlockZero,
    expect_line_count: u64,
) {
    stack_offset_set(Some(2));
    eprintln!(
        "{}test_blockzero_analysis({:?}, blocksz {:?}, expect result {:?}, expect line count {:?})",
        sn(), path, blocksz, expect_result, expect_line_count
    );
    eprint_file(path);
    let mut sp1: SyslogProcessor = new_SyslogProcessor(path, blocksz);
    let result = sp1.process_stage0_valid_file_check();
    assert_eq!(result, FileProcessingResultBlockZero::FileOk, "stage0 failed");
    eprintln!("\n{}{:?}\n", so(), sp1);

    let result = sp1.blockzero_analysis();
    assert_eq!(result, expect_result, "blockzero_analysis() result {:?}, expected {:?}", result, expect_result);
    let line_count_ = sp1.count_lines();
    assert_eq!(
        expect_line_count, line_count_,
        "blockzero_analysis expected {:?} line count, got {:?} line count for {:?}", expect_line_count, line_count_, path,
    );

    eprintln!("{}test_blockzero_analysis()", sx());
}

#[test]
fn test_blockzero_analysis_empty0_FileErrEmpty() {
    test_blockzero_analysis(&NTF_EMPTY0_path, SZ, FileProcessingResultBlockZero::FileErrEmpty, 0);
}

#[test]
fn test_blockzero_analysis_nl1_FileOk() {
    test_blockzero_analysis(&NTF_NL_1_PATH, SZ, FileProcessingResultBlockZero::FileOk, 1);
}

#[test]
fn test_blockzero_analysis_nl2_FileOk() {
    test_blockzero_analysis(&NTF_NL_2_PATH, SZ, FileProcessingResultBlockZero::FileOk, 2);
}

#[test]
fn test_blockzero_analysis_nl20_FileOk() {
    let data = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
    let line_count: u64 = data.lines().count() as u64;
    let ntf = create_temp_log(data);
    let path = ntf_fpath(&ntf);
    let filesz: u64 = ntf.as_file().metadata().unwrap().len() as u64;
    let line_count_default: u64 = *BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP.get(&filesz).unwrap();
    let line_count_ = std::cmp::min(line_count, line_count_default);
    test_blockzero_analysis(&path, SZ, FileProcessingResultBlockZero::FileOk, line_count_);
}

#[test]
fn test_blockzero_analysis_nl0_bsz4_FileErrNoSyslinesFound() {
    let data = "                                                               ";
    let ntf = create_temp_log(data);
    let path = ntf_fpath(&ntf);
    test_blockzero_analysis(&path, 0x4, FileProcessingResultBlockZero::FileErrNoSyslinesFound, 0);
}

#[test]
fn test_blockzero_analysis_nl0_bszFF_FileErrNoSyslinesFound() {
    let data = "                                                               ";
    let ntf = create_temp_log(data);
    let path = ntf_fpath(&ntf);
    test_blockzero_analysis(&path, 0xFF, FileProcessingResultBlockZero::FileErrNoSyslinesFound, 1);
}

#[test]
fn test_blockzero_analysis_nl3_bszFF_FileErrNoLinesFound() {
    let data = "           \n  \n                                               ";
    let ntf = create_temp_log(data);
    let path = ntf_fpath(&ntf);
    test_blockzero_analysis(&path, 0xFF, FileProcessingResultBlockZero::FileErrNoLinesFound, 3);
}

*/

// TODO: [2022/06] need exhaustive test case set for `_test_blockzero_analysis`

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06] test `SyslogProcessor::blockzero_analysis_syslines`
