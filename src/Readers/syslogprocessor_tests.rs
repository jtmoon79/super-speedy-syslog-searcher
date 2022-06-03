// Readers/syslogprocessor_tests.rs
//

use crate::common::{
    FPath,
    FileOffset,
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
    create_temp_file_with_name_exact,
    NTF_Path,
    eprint_file,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::datetime::{
    FixedOffset,
    TimeZone,
};

pub use crate::Readers::syslogprocessor::{
    SyslogProcessor,
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
    match SyslogProcessor::new(path.clone(), blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslogProcessor::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06/01] these are repeated in several `_test.rs` files, declare them in
//       one common `common_tests.rs` file

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0: NamedTempFile = create_temp_file("");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0_path: FPath = NTF_Path(&NTF_empty0);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_1: NamedTempFile = create_temp_file("\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_1_path: FPath = NTF_Path(&NTF_nl_1);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_2: NamedTempFile = create_temp_file("\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_2_path: FPath = NTF_Path(&NTF_nl_2);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_3: NamedTempFile = create_temp_file("\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_3_path: FPath = NTF_Path(&NTF_nl_3);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_4: NamedTempFile = create_temp_file("\n\n\n\n");
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_4_path: FPath = NTF_Path(&NTF_nl_4);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_nl_5: NamedTempFile = create_temp_file("\n\n\n\n\n");
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

// TODO: test `SyslogProcessor::mimesniff_analysis`

// -------------------------------------------------------------------------------------------------

/// test `SyslogProcessor::mimeguess_analysis`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_mimeguess_analysis(
    path: &FPath,
    expect_val: bool,
) {
    stack_offset_set(Some(2));
    eprintln!("{}_test_mimeguess_analysis({:?}, expect {:?})", sn(), path, expect_val);
    let mut lr1: SyslogProcessor = new_SyslogProcessor(path, 0xFF);
    let val = lr1.mimeguess_analysis();
    assert_eq!(
        expect_val, val,
        "blockzero_analysis expected {:?} result, got {:?} result for {:?}", expect_val, val, path,
    );
    eprintln!("{}_test_mimeguess_analysis()", sx());
}

#[test]
fn test_mimeguess_analysis_txt() {
    let ntf = create_temp_file_with_name_exact("", String::from("foo.txt"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, true);
}

#[test]
fn test_mimeguess_analysis_log() {
    let ntf = create_temp_file_with_name_exact("", String::from("foo.log"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, true);
}

#[test]
fn test_mimeguess_analysis_syslog() {
    let ntf = create_temp_file_with_name_exact("", String::from("syslog"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, false);
}

#[test]
fn test_mimeguess_analysis_bin() {
    let ntf = create_temp_file_with_name_exact("", String::from("foo.bin"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, false);
}

#[test]
fn test_mimeguess_analysis_dll() {
    let ntf = create_temp_file_with_name_exact("", String::from("foo.dll"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, false);
}

// -------------------------------------------------------------------------------------------------

/// test `SyslogProcessor::blockzero_analysis_lines`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_blockzero_analysis_lines(
    path: &FPath,
    blocksz: BlockSz,
    expect_val: bool,
    expect_line_count: u64,
) {
    stack_offset_set(Some(2));
    eprintln!(
        "{}_test_blockzero_analysis({:?}, blocksz {:?}, expect result {:?}, expect line count {:?})",
        sn(), path, blocksz, expect_val, expect_line_count
    );
    eprint_file(path);
    let mut sp1: SyslogProcessor = new_SyslogProcessor(path, blocksz);
    eprintln!("\n{}{:?}\n", so(), sp1);

    let result = sp1.blockzero_analysis_lines();
    match result {
        Ok(val) => {
            assert_eq!(
                expect_val, val,
                "blockzero_analysis expected {:?} result, got {:?} result for {:?}", expect_val, val, path,
            );
        },
        Err(err) => {
            panic!("linereader.blockzero_analysis returned Error {:?}", err);
        },
    }
    let line_count_ = sp1.lines_count();
    assert_eq!(
        expect_line_count, line_count_,
        "blockzero_analysis expected {:?} line count, got {:?} line count for {:?}", expect_line_count, line_count_, path,
    );

    eprintln!("{}_test_blockzero_analysis()", sx());
}

#[test]
fn test_blockzero_analysis_lines_empty0_no() {
    _test_blockzero_analysis_lines(&NTF_empty0_path, 0xFF, false, 0);
}

#[test]
fn test_blockzero_analysis_lines_nl1_yes() {
    _test_blockzero_analysis_lines(&NTF_nl_1_path, 0xFF, true, 1);
}

#[test]
fn test_blockzero_analysis_lines_nl2_yes() {
    _test_blockzero_analysis_lines(&NTF_nl_2_path, 0xFF, true, 2);
}

#[test]
fn test_blockzero_analysis_lines_nl20_yes() {
    let data = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
    let line_count: u64 = data.lines().count() as u64;
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let line_count_ = std::cmp::min(line_count, SyslogProcessor::BLOCKZERO_ANALYSIS_LINE_COUNT);
    _test_blockzero_analysis_lines(&path, 0xFF, true, line_count_);
}

#[test]
fn test_blockzero_analysis_lines_nl0_bsz4_no() {
    let data = "                                                               ";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    _test_blockzero_analysis_lines(&path, 0x4, false, 0);
}

#[test]
fn test_blockzero_analysis_lines_nl0_bszFF_yes() {
    let data = "                                                               ";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    _test_blockzero_analysis_lines(&path, 0xFF, true, 1);
}

#[test]
fn test_blockzero_analysis_lines_nl3_bszFF_yes() {
    let data = "           \n  \n                                               ";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    _test_blockzero_analysis_lines(&path, 0xFF, true, 3);
}

// -------------------------------------------------------------------------------------------------

// TODO: [2022/06] test `SyslogProcessor::blockzero_analysis_syslines`

