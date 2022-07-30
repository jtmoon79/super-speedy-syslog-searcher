// Readers/filepreprocessor_tests.rs
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::s4_tests::common::{
    NTF_GZ_ONEBYTE_PATH,
    NTF_TAR_ONEBYTE_FPATH_FILEA,
};

extern crate s4lib;

use s4lib::Readers::helpers::{
    path_to_fpath,
    fpath_to_path
};

use s4lib::common::{
    Path,
    FPath,
    FileType,
};

use s4lib::Readers::blockreader::{
    SUBPATH_SEP,
};

use s4lib::Readers::filepreprocessor::{
    fpath_to_filetype_mimeguess,
    path_to_filetype_mimeguess,
    MimeGuess,
    mimeguess_to_filetype,
    path_to_filetype,
    fpath_to_filetype,
    parseable_filetype,
};

use s4lib::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file_with_name_exact,
    create_temp_file_with_suffix,
    create_temp_file_bytes_with_suffix,
    NTF_Path,
};

use s4lib::printer_debug::stack::{
    sn,
    so,
    sx,
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

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

fn test_mimeguess(suffix: &String, check: MimeGuess) {
    eprintln!("test_mimeguess: suffix {:?}", &suffix);
    let ntf = create_temp_file_with_suffix("", suffix);
    let path: FPath = NTF_Path(&ntf);
    eprintln!("test_mimeguess: tempfile {:?}", &path);
    let (filetype, mimeguess) = fpath_to_filetype_mimeguess(&path);
    eprintln!("test_mimeguess: mimeguess {:?}", &mimeguess);
    assert_eq!(check, mimeguess, "expected MimeGuess {:?}\nfound MimeGuess {:?}\n", check, mimeguess);
}

#[test]
fn test_mimeguess_txt() {
    test_mimeguess(&String::from(".txt"), MimeGuess::from_ext("txt"));
}

#[test]
fn test_mimeguess_gz_onebyte() {
    let (filetype, mimeguess) = path_to_filetype_mimeguess(&NTF_GZ_ONEBYTE_PATH);
    eprintln!("test_mimeguess_gz: blockreader.mimeguess {:?}", &mimeguess);
    let check = MimeGuess::from_ext("gz");
    assert_eq!(check, mimeguess, "expected MimeGuess {:?}\nfound MimeGuess {:?}\n", check, mimeguess);
}

// -------------------------------------------------------------------------------------------------

fn test_filetype(name: &String, check: FileType) {
    eprintln!("test_filetype: name {:?}", &name);
    let ntf = create_temp_file_with_name_exact("", name);
    let path = NTF_Path(&ntf);
    eprintln!("test_filetype: tempfile {:?}", &path);
    let (filetype, mimeguess) = fpath_to_filetype_mimeguess(&path);
    eprintln!("test_filetype: blockreader.filetype {:?}", filetype);
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_filetype_FILE_txt() {
    test_filetype(&String::from("test_filetype_txt.txt"), FileType::FILE);
}

#[test]
fn test_filetype_FILE_log() {
    test_filetype(&String::from("test_filetype_log.log"), FileType::FILE);
}

#[test]
fn test_filetype_FILE_syslog() {
    test_filetype(&String::from("syslog"), FileType::FILE);
}

#[test]
fn test_filetype_FILE_messages() {
    test_filetype(&String::from("messages"), FileType::FILE);
}

#[test]
fn test_filetype_FILEGZ_gz() {
    let filetype = path_to_filetype(&NTF_GZ_ONEBYTE_PATH);
    eprintln!("test_mimeguess_gz: blockreader.filetype {:?}", &filetype);
    let check = FileType::FileGz;
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_filetype_FILETAR_tar() {
    let filetype = fpath_to_filetype(&NTF_TAR_ONEBYTE_FPATH_FILEA);
    eprintln!("test_mimeguess_gz: blockreader.filetype {:?}", &filetype);
    let check = FileType::FileTar;
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}
