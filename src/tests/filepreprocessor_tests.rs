// src/tests/filepreprocessor_tests.rs

//! tests for `filepreprocessor.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str::FromStr;

#[allow(unused_imports)]
use crate::tests::common::{
    MIMEGUESS_EMPTY,
    MIMEGUESS_LOG,
    MIMEGUESS_TXT,
    MIMEGUESS_GZ,
    MIMEGUESS_XZ,
    MIMEGUESS_TAR,
    NTF_TAR_1BYTE,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FILEA_FILETYPE,
    NTF_TAR_1BYTE_FILEA_MIMEGUESS,
    NTF_LOG_EMPTY,
    NTF_LOG_EMPTY_FPATH,
    NTF_LOG_EMPTY_FILETYPE,
    NTF_LOG_EMPTY_MIMEGUESS,
    NTF_GZ_EMPTY,
    NTF_GZ_EMPTY_FPATH,
    NTF_GZ_EMPTY_FILETYPE,
    NTF_GZ_EMPTY_MIMEGUESS,
    FILE,
    FILE_GZ,
    FILE_TAR,
    FILE_XZ,
};

use crate::common::{
    FPath,
    FileType,
    Path,
};

#[allow(unused_imports)]
use crate::readers::filepreprocessor::{
    ProcessPathResult,
    fpath_to_filetype_mimeguess,
    path_to_filetype_mimeguess,
    MimeGuess,
    mimeguess_to_filetype,
    path_to_filetype,
    fpath_to_filetype,
    parseable_filetype,
    process_path_tar,
    process_path,
};

use crate::printer_debug::helpers::{
    NamedTempFile,
    ntf_fpath,
};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate si_trace_print;
use si_trace_print::{
    dpfn,
    dpfx,
};
use si_trace_print::stack::stack_offset_set;

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn test_mimeguess_to_filetype(mimeguess: &MimeGuess, check: FileType) {
    let filetype: FileType = mimeguess_to_filetype(mimeguess);
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_mimeguess_to_filetype_txt() {
    test_mimeguess_to_filetype(&MIMEGUESS_TXT, FileType::File);
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
    stack_offset_set(Some(2));
    eprintln!("test_fpath_to_filetype: name {:?}", &name);
    let fpath: FPath = FPath::from(name);
    let filetype = fpath_to_filetype(&fpath);
    eprintln!("test_fpath_to_filetype: filetype {:?}", filetype);
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_fpath_to_filetype_File_log() {
    test_fpath_to_filetype(&String::from("log"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_File_log_log() {
    test_fpath_to_filetype(&String::from("log.log"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_File_log_log1() {
    test_fpath_to_filetype(&String::from("log_media"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_File_log_log2() {
    test_fpath_to_filetype(&String::from("media_log"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_File_log_old() {
    test_fpath_to_filetype(&String::from("media.log.old"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_File_syslog() {
    test_fpath_to_filetype(&String::from("syslog"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_File_messages() {
    test_fpath_to_filetype(&String::from("messages"), FileType::File);
}

#[test]
fn test_fpath_to_filetype_FileGZ_gz_old() {
    test_fpath_to_filetype(&String::from("data.gz.old"), FileType::FileGz);
}

#[test]
fn test_fpath_to_filetype_FileGZ_gzip() {
    test_fpath_to_filetype(&String::from("data.gzip"), FileType::FileGz);
}

// -------------------------------------------------------------------------------------------------

fn test_process_file_path(ntf: &NamedTempFile, check: Vec<ProcessPathResult>) {
    stack_offset_set(Some(2));
    eprintln!("test_process_file_path: ntf {:?}", ntf);
    let fpath = ntf_fpath(&ntf);
    let results = process_path(&fpath);
    assert_eq!(check, results, "\nexpected {:?}\nactual  {:?}\n", check, results);
}

#[test]
fn test_process_file_path_log() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_LOG_EMPTY_FPATH.clone(), *NTF_LOG_EMPTY_MIMEGUESS, *NTF_LOG_EMPTY_FILETYPE,
        )
    ];
    test_process_file_path(&NTF_LOG_EMPTY, check);
}

#[test]
fn test_process_file_path_gz() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_GZ_EMPTY_FPATH.clone(), *NTF_GZ_EMPTY_MIMEGUESS, *NTF_GZ_EMPTY_FILETYPE,
        )
    ];
    test_process_file_path(&NTF_GZ_EMPTY, check);
}

#[test]
fn test_process_file_path_tar() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_1BYTE_FILEA_FPATH.clone(),
            *NTF_TAR_1BYTE_FILEA_MIMEGUESS,
            *NTF_TAR_1BYTE_FILEA_FILETYPE,
        )
    ];
    test_process_file_path(&NTF_TAR_1BYTE, check);
}

// -------------------------------------------------------------------------------------------------

lazy_static! {
    pub static ref MIMEGUESS_LOG_1: MimeGuess = {
        MimeGuess::from_path(Path::new("test.log"))
    };
}

/// test `fpath_to_filetype_mimeguess` (and `path_to_filetype_mimeguess`)
#[test_case("messages", FILE, &MIMEGUESS_EMPTY)]
#[test_case("syslog", FILE, &MIMEGUESS_EMPTY)]
#[test_case("syslog.3", FILE, &MIMEGUESS_EMPTY)]
#[test_case("output.txt", FILE, &MIMEGUESS_TXT)]
#[test_case("kern.log", FILE, &MIMEGUESS_LOG)]
#[test_case("kern.log.1", FILE, &MIMEGUESS_LOG)]
#[test_case("kern.log.2", FILE, &MIMEGUESS_LOG)]
#[test_case("systemsetup-server-info.log.208", FILE, &MIMEGUESS_LOG)]
#[test_case("syslog.gz", FILE_GZ, &MIMEGUESS_GZ)]
#[test_case("syslog.9.gz", FILE_GZ, &MIMEGUESS_GZ)]
#[test_case("unattended-upgrades-dpkg.log.3.gz", FILE_GZ, &MIMEGUESS_GZ)]
#[test_case("eipp.log.xz", FILE_XZ, &MIMEGUESS_XZ)]
#[test_case("logs.tar", FILE_TAR, &MIMEGUESS_TAR)]
#[test_case("log.1.tar", FILE_TAR, &MIMEGUESS_TAR)]
fn test_path_to_filetype_mimeguess(
    path_str: &str,
    filetype: FileType,
    mimeguess: &MimeGuess,
) {
    dpfn!("({:?})", path_str);
    let fpath: FPath = FPath::from_str(path_str).unwrap();
    let (filetype_, mimeguess_) = fpath_to_filetype_mimeguess(&fpath);
    assert_eq!(filetype, filetype_, "\nExpected {:?}\nActual   {:?}\n", filetype, filetype_);
    assert_eq!(mimeguess, &mimeguess_, "\nExpected {:?}\nActual   {:?}\n", mimeguess, mimeguess_);
    dpfx!();
}
