// src/tests/syslogprocessor_tests.rs

//! tests for `syslogprocessor.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{
    FPath,
};

use crate::printer_debug::helpers::{
    create_temp_file,
    ntf_fpath,
};

use crate::readers::blockreader::{
    BlockSz,
};

use crate::readers::filepreprocessor::{
    fpath_to_filetype_mimeguess,
};

use crate::data::datetime::{
    FixedOffset,
};

use crate::readers::syslogprocessor::{
    SyslogProcessor,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper to wrap the match and panic checks
fn new_SyslogProcessor(path: &FPath, blocksz: BlockSz) -> SyslogProcessor {
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

// TODO: [2022/06/01] these are repeated in several `_test.rs` files, declare them in
//       one common `common_tests.rs` file

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const SZ: BlockSz = SyslogProcessor::BLOCKSZ_MIN;

/// test `SyslogProcessor::new`
#[test]
fn test_SyslogProcessor_new1() {
    let ntf = create_temp_file("");
    let path = ntf_fpath(&ntf);
    let slp = new_SyslogProcessor(&path, SZ);
    eprintln!("{:?}", slp);
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
