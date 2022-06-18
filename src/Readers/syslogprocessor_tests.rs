// Readers/syslogprocessor_tests.rs
//

use crate::common::{
    FPath,
    FileOffset,
    FileType,
};

use crate::dbgpr::stack::{
    stack_offset_set,
    sn,
    snx,
    so,
    sx,
};

use crate::dbgpr::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_bytes,
    create_temp_file_with_suffix,
    create_temp_file_with_name_exact,
    NTF_Path,
    eprint_file,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::filepreprocessor::{
    fpath_to_filetype,
    guess_filetype_from_fpath,
};

use crate::Data::datetime::{
    FixedOffset,
    TimeZone,
};

pub use crate::Readers::syslogprocessor::{
    SyslogProcessor,
    FileProcessingResult_BlockZero,
    BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP,
};

use std::io::{
    Error,
    Result,
    ErrorKind,
};

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
};

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper to wrap the match and panic checks
#[cfg(test)]
fn new_SyslogProcessor(path: &FPath, blocksz: BlockSz) -> SyslogProcessor {
    let tzo: FixedOffset = FixedOffset::east(0);
    let filetype: FileType = guess_filetype_from_fpath(path);
    match SyslogProcessor::new(path.clone(), filetype, blocksz, tzo, None, None) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslogProcessor::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

lazy_static! {
    static ref STRING_LOG: String = String::from(".log");
}

/// create a temp file filled with `data` ending in `.log`
fn create_temp_log(data: &str) -> NamedTempFile {
    create_temp_file_with_suffix(data, &STRING_LOG)
}

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06/01] these are repeated in several `_test.rs` files, declare them in
//       one common `common_tests.rs` file

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0: NamedTempFile = create_temp_log("");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0_path: FPath = NTF_Path(&NTF_empty0);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_1: NamedTempFile = create_temp_log("\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_1_path: FPath = NTF_Path(&NTF_nl_1);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_2: NamedTempFile = create_temp_log("\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_2_path: FPath = NTF_Path(&NTF_nl_2);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_3: NamedTempFile = create_temp_log("\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_3_path: FPath = NTF_Path(&NTF_nl_3);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_4: NamedTempFile = create_temp_log("\n\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_4_path: FPath = NTF_Path(&NTF_nl_4);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_5: NamedTempFile = create_temp_log("\n\n\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_5_path: FPath = NTF_Path(&NTF_nl_5);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// test `SyslogProcessor::new`
#[test]
fn test_SyslogProcessor_new1() {
    let ntf = create_temp_file("");
    let path = NTF_Path(&ntf);
    let slp = new_SyslogProcessor(&path, 0xF);
    debug_eprintln!("{:?}", slp);
}

// -------------------------------------------------------------------------------------------------

/// test `SyslogProcessor::blockzero_analysis_lines`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_blockzero_analysis_lines(
    path: &FPath,
    blocksz: BlockSz,
    expect_result: FileProcessingResult_BlockZero,
    expect_line_count: u64,
) {
    stack_offset_set(Some(2));
    eprintln!(
        "{}_test_blockzero_analysis({:?}, blocksz {:?}, expect result {:?}, expect line count {:?})",
        sn(), path, blocksz, expect_result, expect_line_count
    );
    eprint_file(path);
    let mut sp1: SyslogProcessor = new_SyslogProcessor(path, blocksz);
    eprintln!("\n{}{:?}\n", so(), sp1);

    let result = sp1.blockzero_analysis_lines();
    assert_eq!(result, expect_result, "blockzero_analysis_lines() result {:?}, expected {:?}", result, expect_result);
    let line_count_ = sp1.lines_count();
    assert_eq!(
        expect_line_count, line_count_,
        "blockzero_analysis expected {:?} line count, got {:?} line count for {:?}", expect_line_count, line_count_, path,
    );

    eprintln!("{}_test_blockzero_analysis()", sx());
}

#[test]
fn test_blockzero_analysis_lines_empty0_FILE_ERR_EMPTY() {
    _test_blockzero_analysis_lines(&NTF_empty0_path, 0xFF, FileProcessingResult_BlockZero::FILE_ERR_EMPTY, 0);
}

#[test]
fn test_blockzero_analysis_lines_nl1_FILE_OK() {
    _test_blockzero_analysis_lines(&NTF_nl_1_path, 0xFF, FileProcessingResult_BlockZero::FILE_OK, 1);
}

#[test]
fn test_blockzero_analysis_lines_nl2_FILE_OK() {
    _test_blockzero_analysis_lines(&NTF_nl_2_path, 0xFF, FileProcessingResult_BlockZero::FILE_OK, 2);
}

#[test]
fn test_blockzero_analysis_lines_nl20_FILE_OK() {
    let data = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
    let line_count: u64 = data.lines().count() as u64;
    let ntf = create_temp_log(data);
    let path = NTF_Path(&ntf);
    let filesz: u64 = ntf.as_file().metadata().unwrap().len() as u64;
    let line_count_default: u64 = *BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP.get(&filesz).unwrap();
    let line_count_ = std::cmp::min(line_count, line_count_default);
    _test_blockzero_analysis_lines(&path, 0xFF, FileProcessingResult_BlockZero::FILE_OK, line_count_);
}

#[test]
fn test_blockzero_analysis_lines_nl0_bsz4_FILE_ERR_NO_SYSLINES_FOUND() {
    let data = "                                                               ";
    let ntf = create_temp_log(data);
    let path = NTF_Path(&ntf);
    _test_blockzero_analysis_lines(&path, 0x4, FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_FOUND, 0);
}

#[test]
fn test_blockzero_analysis_lines_nl0_bszFF_FILE_ERR_NO_SYSLINES_FOUND() {
    let data = "                                                               ";
    let ntf = create_temp_log(data);
    let path = NTF_Path(&ntf);
    _test_blockzero_analysis_lines(&path, 0xFF, FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_FOUND, 1);
}

#[test]
fn test_blockzero_analysis_lines_nl3_bszFF_FILE_ERR_NO_LINES_FOUND() {
    let data = "           \n  \n                                               ";
    let ntf = create_temp_log(data);
    let path = NTF_Path(&ntf);
    _test_blockzero_analysis_lines(&path, 0xFF, FileProcessingResult_BlockZero::FILE_ERR_NO_LINES_FOUND, 3);
}

// TODO: [2022/06] need exhaustive test case set for `_test_blockzero_analysis_lines`

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06] test `SyslogProcessor::blockzero_analysis_syslines`

