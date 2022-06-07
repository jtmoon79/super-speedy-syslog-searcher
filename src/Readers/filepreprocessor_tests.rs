// Readers/filepreprocessor_tests.rs
//

use std::fs::File;

use crate::common::{
    FPath,
    Bytes,
};

use crate::Readers::blockreader::{
    BlockSz,
};

use crate::Readers::linereader::{
    FileOffset,
    ResultS4_LineFind,
    LineReader,
    LineIndex,
    enum_BoxPtrs,
};

use crate::Readers::helpers::{
    randomize,
    fill,
};

use crate::Readers::filepreprocessor::{
    //FilePreProcessor,
    ProcessPathResult,
    process_fpath,
};

use crate::dbgpr::helpers::{
    NamedTempFile,
    tempdir,
    create_temp_file,
    create_temp_file_with_name,
    create_temp_file_with_name_exact,
    NTF_Path,
    eprint_file,
};

use crate::dbgpr::printers::{
    //Color,
    //print_colored_stdout,
    byte_to_char_noraw,
    buffer_to_String_noraw,
};

use crate::dbgpr::stack::{
    sn,
    so,
    sx,
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_lt,
    assert_ge,
};


extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate mime_sniffer;
use mime_sniffer::MimeTypeSniffer;  // adds extension method `sniff_mime_type` to `[u8]`

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/*
/// helper to wrap the match and panic checks
#[cfg(test)]
fn new_FilePreprocessor(path: &FPath) -> FilePreProcessor {
    match FilePreProcessor::new(path.clone()) {
        Ok(val) => {
            eprintln!("{:?}", val);
            val
        },
        Err(err) => {
            panic!("ERROR: FilePreProcessor::new({:?}) failed {}", path, err);
        }
    }
}

#[test]
fn test_FilePreProcessor_new1() {
    let ntf = create_temp_file_with_name_exact("", String::from("foo.txt"));
    let path = NTF_Path(&ntf);
    new_FilePreprocessor(&path);
}
*/

// -------------------------------------------------------------------------------------------------

// TODO: test `filepreprocessor::mimesniff_analysis`

// -------------------------------------------------------------------------------------------------

/// test `filepreprocessor::mimeguess_analysis`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_mimeguess_analysis(
    path: &FPath,
    expect_val: bool,
) {
    stack_offset_set(Some(2));
    eprintln!("{}_test_mimeguess_analysis({:?}, expect {:?})", sn(), path, expect_val);
    /*
    // TODO: 2022/06/06 fix this call
    let val = mimeguess_analysis();
    assert_eq!(
        expect_val, val,
        "blockzero_analysis expected {:?} result, got {:?} result for {:?}", expect_val, val, path,
    );
    */
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
