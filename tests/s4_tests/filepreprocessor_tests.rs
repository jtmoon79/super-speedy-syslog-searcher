// Readers/filepreprocessor_tests.rs
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::s4_tests::common::{
    MIMEGUESS_TXT,
    MIMEGUESS_GZ,
    MIMEGUESS_XZ,
    MIMEGUESS_TAR,
    NTF_GZ_ONEBYTE_PATH,
    NTF_TAR_ONEBYTE_FPATH,
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

fn test_mimeguess_to_filetype(mimeguess: &MimeGuess, check: FileType) {
    let filetype: FileType = mimeguess_to_filetype(mimeguess);
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_mimeguess_to_filetype_txt() {
    test_mimeguess_to_filetype(&MIMEGUESS_TXT, FileType::FILE);
}

#[test]
fn test_mimeguess_to_filetype_gz() {
    test_mimeguess_to_filetype(&MIMEGUESS_GZ, FileType::FileGz);
}

#[test]
fn test_mimeguess_to_filetype_xz() {
    test_mimeguess_to_filetype(&MIMEGUESS_XZ, FileType::FileXz);
}

#[test]
fn test_mimeguess_to_filetype_tar() {
    test_mimeguess_to_filetype(&MIMEGUESS_TAR, FileType::FileTar);
}

// -------------------------------------------------------------------------------------------------

fn test_fpath_to_filetype(name: &String, check: FileType) {
    eprintln!("test_fpath_to_filetype: name {:?}", &name);
    let fpath: FPath = FPath::from(name);
    let filetype = fpath_to_filetype(&fpath);
    eprintln!("test_fpath_to_filetype: blockreader.filetype {:?}", filetype);
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_fpath_to_filetype_FILE_log() {
    test_fpath_to_filetype(&String::from("log"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILE_log_log() {
    test_fpath_to_filetype(&String::from("log.log"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILE_log_log1() {
    test_fpath_to_filetype(&String::from("log_media"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILE_log_log2() {
    test_fpath_to_filetype(&String::from("media_log"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILE_log_old() {
    test_fpath_to_filetype(&String::from("media.log.old"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILE_syslog() {
    test_fpath_to_filetype(&String::from("syslog"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILE_messages() {
    test_fpath_to_filetype(&String::from("messages"), FileType::FILE);
}

#[test]
fn test_fpath_to_filetype_FILEGZ_gz_old() {
    test_fpath_to_filetype(&String::from("data.gz.old"), FileType::FileGz);
}

#[test]
fn test_fpath_to_filetype_FILEGZ_gzip() {
    test_fpath_to_filetype(&String::from("data.gzip"), FileType::FileGz);
}
