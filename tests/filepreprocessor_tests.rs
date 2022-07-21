// Readers/filepreprocessor_tests.rs
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate s4lib;

use s4lib::common::{
    FPath,
};

use s4lib::Readers::filepreprocessor::{
    mimeguess_to_filetype,
    path_to_filetype,
    parseable_filetype,
};

use s4lib::printer_debug::helpers::{
    create_temp_file_with_name_exact,
    NTF_Path,
};

use s4lib::printer_debug::stack::{
    sn,
    so,
    sx,
    stack_offset_set,
};

extern crate mime_sniffer;
use mime_sniffer::MimeTypeSniffer;  // adds extension method `sniff_mime_type` to `[u8]`

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_placeholder_until_these_other_tests_are_uncommented() {
    // placeholder test to ensure this file is processed.
    // in-place until these tests are uncommented
    let t = true;
    assert!(t, "");
}

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

/*

/// test `filepreprocessor::mimeguess_analysis`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_mimeguess_analysis(
    path: &FPath,
    expect_val: bool,
) {
    stack_offset_set(Some(2));
    eprintln!("{}_test_mimeguess_analysis({:?}, expect {:?})", sn(), path, expect_val);
    // TODO: 2022/06/06 fix this call
    let val = parseable_filetype();
    assert_eq!(
        expect_val, val,
        "blockzero_analysis expected {:?} result, got {:?} result for {:?}", expect_val, val, path,
    );
    eprintln!("{}_test_mimeguess_analysis()", sx());
}

#[test]
fn test_mimeguess_analysis_txt() {
    let ntf = create_temp_file_with_name_exact("", &String::from("foo.txt"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, true);
}

#[test]
fn test_mimeguess_analysis_log() {
    let ntf = create_temp_file_with_name_exact("", &String::from("foo.log"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, true);
}

#[test]
fn test_mimeguess_analysis_syslog() {
    let ntf = create_temp_file_with_name_exact("", &String::from("syslog"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, false);
}

#[test]
fn test_mimeguess_analysis_bin() {
    let ntf = create_temp_file_with_name_exact("", &String::from("foo.bin"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, false);
}

#[test]
fn test_mimeguess_analysis_dll() {
    let ntf = create_temp_file_with_name_exact("", &String::from("foo.dll"));
    let path = NTF_Path(&ntf);
    _test_mimeguess_analysis(&path, false);
}

*/
