// src/tests/syslogprocessor_tests.rs

//! tests for `syslogprocessor.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{FPath, FileOffset};

use crate::debug::helpers::{create_temp_file, ntf_fpath, NamedTempFile};

use crate::readers::blockreader::BlockSz;

use crate::readers::filepreprocessor::fpath_to_filetype_mimeguess;

use crate::data::datetime::{
    datetime_parse_from_str, DateTimeL, DateTimePattern_str, FixedOffset, SystemTime,
};

use crate::readers::syslinereader::ResultS3SyslineFind;

use crate::readers::syslogprocessor::{FileProcessingResultBlockZero, SyslogProcessor};

extern crate const_format;
use const_format::concatcp;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06/01] these are repeated in several `_test.rs` files, declare them in
//       one common `common_tests.rs` file

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const SZ: BlockSz = SyslogProcessor::BLOCKSZ_MIN;

const NTF5_DATA_LINE0: &str = "Jan 1 01:00:00 5a\n";
const NTF5_DATA_LINE1: &str = "Feb 29 02:00:00 5b\n";
const NTF5_DATA_LINE2: &str = "Mar 3 03:00:00 5c\n";
const NTF5_DATA_LINE3: &str = "Apr 4 04:00:00 5d\n";
const NTF5_DATA_LINE4: &str = "May 5 05:00:00 5e\n";

const NTF5_DATA: &str =
    concatcp!(NTF5_DATA_LINE0, NTF5_DATA_LINE1, NTF5_DATA_LINE2, NTF5_DATA_LINE3, NTF5_DATA_LINE4,);

#[allow(dead_code)]
const NTF5_DATA_LINE0_OFFSET: usize = 0;
#[allow(dead_code)]
const NTF5_DATA_LINE1_OFFSET: usize = NTF5_DATA_LINE0
    .as_bytes()
    .len();
#[allow(dead_code)]
const NTF5_DATA_LINE2_OFFSET: usize = NTF5_DATA_LINE1_OFFSET
    + NTF5_DATA_LINE1
        .as_bytes()
        .len();
#[allow(dead_code)]
const NTF5_DATA_LINE3_OFFSET: usize = NTF5_DATA_LINE2_OFFSET
    + NTF5_DATA_LINE2
        .as_bytes()
        .len();
#[allow(dead_code)]
const NTF5_DATA_LINE4_OFFSET: usize = NTF5_DATA_LINE3_OFFSET
    + NTF5_DATA_LINE3
        .as_bytes()
        .len();

const NTF5_LINE2_DATETIME_STR: &str = "Mar 3 03:00:00 +0000";
const NTF5_LINE2_DATETIME_PATTERN: &DateTimePattern_str = "%b %e %H:%M:%S %z";

const NTF3_DATA_LINE0: &str = "Jan 1 01:00:00 2000 A3\n";
const NTF3_DATA_LINE1: &str = "Feb 2 02:00:00 2000 B3\n";
const NTF3_DATA_LINE2: &str = "Mar 3 03:00:00 2000 C3\n";

const NTF3_DATA: &str = concatcp!(NTF3_DATA_LINE0, NTF3_DATA_LINE1, NTF3_DATA_LINE2,);

const NTF3_LINE1_DATETIME_STR: &str = "Feb 2 02:00:00 2000 +0000";
const NTF3_LINE1_DATETIME_PATTERN: &DateTimePattern_str = "%b %e %H:%M:%S %Y %z";

lazy_static! {
    static ref TIMEZONE_0: FixedOffset = FixedOffset::west(0);

    // NTF5

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
        create_temp_file(NTF5_DATA)
    };

    static ref NTF5_PATH: FPath = {
        ntf_fpath(&NTF5)
    };

    // NTF3

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
    let tzo: FixedOffset = FixedOffset::east(0);
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
    slp.process_missing_year(SYSTEMTIME_1972_06_01.clone());
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

    match slp.process_stage2_find_dt() {
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

// -------------------------------------------------------------------------------------------------

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
