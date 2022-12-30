// src/tests/filepreprocessor_tests.rs

//! tests for `filepreprocessor.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str::FromStr;

#[allow(unused_imports)]
use crate::tests::common::{
    MIMEGUESS_EMPTY, MIMEGUESS_GZ, MIMEGUESS_LOG, MIMEGUESS_TAR,
    MIMEGUESS_TXT, MIMEGUESS_XZ, NTF_GZ_EMPTY, NTF_GZ_EMPTY_FILETYPE, NTF_GZ_EMPTY_FPATH,
    NTF_GZ_EMPTY_MIMEGUESS, NTF_LOG_EMPTY, NTF_LOG_EMPTY_FILETYPE, NTF_LOG_EMPTY_FPATH,
    NTF_LOG_EMPTY_MIMEGUESS, NTF_TAR_1BYTE, NTF_TAR_1BYTE_FILEA_FILETYPE, NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FILEA_MIMEGUESS,
};

use crate::common::{FPath, FileType, Path};

#[allow(unused_imports)]
use crate::readers::filepreprocessor::{
    fpath_to_filetype, fpath_to_filetype_mimeguess, mimeguess_to_filetype, parseable_filetype,
    path_to_filetype, path_to_filetype_mimeguess, process_path, process_path_tar, MimeGuess,
    ProcessPathResult,
};

use crate::readers::helpers::fpath_to_path;

use crate::debug::helpers::{ntf_fpath, NamedTempFile};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate si_trace_print;
use si_trace_print::stack::stack_offset_set;
use si_trace_print::{dpfn, dpfx};

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn test_mimeguess_to_filetype(
    mimeguess: &MimeGuess,
    check: FileType,
) {
    let filetype: FileType = mimeguess_to_filetype(mimeguess);
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

#[test]
fn test_mimeguess_to_filetype_txt() {
    test_mimeguess_to_filetype(&MIMEGUESS_TXT, FileType::File);
}

#[test]
fn test_mimeguess_to_filetype_gz() {
    test_mimeguess_to_filetype(&MIMEGUESS_GZ, FileType::Gz);
}

#[test]
fn test_mimeguess_to_filetype_xz() {
    test_mimeguess_to_filetype(&MIMEGUESS_XZ, FileType::Xz);
}

#[test]
fn test_mimeguess_to_filetype_tar() {
    test_mimeguess_to_filetype(&MIMEGUESS_TAR, FileType::Tar);
}

// -------------------------------------------------------------------------------------------------

#[test_case("log", FileType::File)]
#[test_case("log.log", FileType::File)]
#[test_case("log_media", FileType::File)]
#[test_case("media_log", FileType::File)]
#[test_case("media.log.old", FileType::File)]
#[test_case("syslog", FileType::File)]
#[test_case("messages", FileType::File)]
#[test_case("a.log", FileType::Unknown)]
#[test_case("log.a", FileType::File)]
#[test_case("data.gz", FileType::Gz)]
#[test_case("data.gz.old", FileType::Gz)]
#[test_case("data.gzip", FileType::Gz)]
#[test_case("data.tar", FileType::Tar)]
#[test_case("data.tar.old", FileType::Tar)]
#[test_case("wtmp", FileType::Unparseable)]
#[test_case("btmp", FileType::Unparseable)]
#[test_case("utmp", FileType::Unparseable)]
#[test_case("SOMEFILE", FileType::File)]
fn test_fpath_to_filetype(
    name: &str,
    check: FileType,
) {
    stack_offset_set(Some(2));
    let fpath: FPath = FPath::from(name);
    let fpath_full: FPath = FPath::from("/var/log/") + fpath.as_str();
    for path in [&fpath, &fpath_full].iter() {
        eprintln!("test_fpath_to_filetype: fpath_to_filetype({:?})", path);
        let filetype = fpath_to_filetype(path);
        eprintln!("test_fpath_to_filetype: fpath_to_filetype returned {:?}", filetype);
        assert_eq!(check, filetype, "\npath {:?}\nexpected FileType {:?}\nactual FileType {:?}\n", path, check, filetype);
    }
}

// -------------------------------------------------------------------------------------------------

fn test_process_path_fpath(
    path: &FPath,
    check: Vec<ProcessPathResult>,
) {
    let results = process_path(path);
    assert_eq!(check, results, "\nprocess_path({:?})\nexpected {:?}\nactual  {:?}\n", path, check, results);
}

fn test_process_path_ntf(
    ntf: &NamedTempFile,
    check: Vec<ProcessPathResult>,
) {
    stack_offset_set(Some(2));
    let path = ntf_fpath(ntf);
    test_process_path_fpath(&path, check);
}

#[test]
fn test_process_path_log() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_LOG_EMPTY_FPATH.clone(),
            *NTF_LOG_EMPTY_MIMEGUESS,
            *NTF_LOG_EMPTY_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_LOG_EMPTY, check);
}

#[test]
fn test_process_path_gz() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_GZ_EMPTY_FPATH.clone(),
            *NTF_GZ_EMPTY_MIMEGUESS,
            *NTF_GZ_EMPTY_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_GZ_EMPTY, check);
}

#[test]
fn test_process_path_tar() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_1BYTE_FILEA_FPATH.clone(),
            *NTF_TAR_1BYTE_FILEA_MIMEGUESS,
            *NTF_TAR_1BYTE_FILEA_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_TAR_1BYTE, check);
}

#[test]
fn test_process_path_not_exist() {
    let path: FPath = FPath::from("/THIS/FILE/DOES/NOT/EXIST!");
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileErrNotExist(path.clone()),
    ];
    test_process_path_fpath(&path, check);
}

#[test]
fn test_process_path_not_a_file() {
    let fpath: FPath = FPath::from("/dev/null");
    // do not test if path does not exist. avoids failures on unusual platforms
    if ! fpath_to_path(&fpath).exists() {
        eprintln!("Path does not exist, pass test {:?}", fpath);
        return;
    }
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileErrNotAFile(fpath.clone()),
    ];
    test_process_path_fpath(&fpath, check);
}

// -------------------------------------------------------------------------------------------------

lazy_static! {
    pub static ref MIMEGUESS_LOG_1: MimeGuess = MimeGuess::from_path(Path::new("test.log"));
}

/// test `fpath_to_filetype_mimeguess` (and `path_to_filetype_mimeguess`)
#[test_case("messages", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("syslog", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("syslog.3", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("output.txt", FileType::File, &MIMEGUESS_TXT)]
#[test_case("kern.log", FileType::File, &MIMEGUESS_LOG)]
#[test_case("kern.log.1", FileType::File, &MIMEGUESS_LOG)]
#[test_case("kern.log.2", FileType::File, &MIMEGUESS_LOG)]
#[test_case("aptitude.4", FileType::Unknown, &MIMEGUESS_EMPTY)]
#[test_case("systemsetup-server-info.log.208", FileType::File, &MIMEGUESS_LOG)]
#[test_case("syslog.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("syslog.9.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("unattended-upgrades-dpkg.log.3.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("eipp.log.xz", FileType::Xz, &MIMEGUESS_XZ)]
#[test_case("logs.tar", FileType::Tar, &MIMEGUESS_TAR)]
#[test_case("log.1.tar", FileType::Tar, &MIMEGUESS_TAR)]
#[test_case("HOSTNAME.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("log.HOSTNAME", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.nmbd", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.nmbd.1", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.nmbd.old", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.gz.1", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.gz.2", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.gz.99", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("telemetry", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("initial-status", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("smart_extend_log", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case(".disk_daily_info_send_udc_time", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("messages-DropletAgent", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("CC_AA_DD_EE_FF_00-ns", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("CC_AA_DD_EE_FF_00-ns.old", FileType::Unknown, &MIMEGUESS_EMPTY)]
#[test_case("CC_AA_DD_EE_FF_00-ns.old.1", FileType::Unknown, &MIMEGUESS_EMPTY)]
#[test_case("history", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("192.168.1.100.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("192.168.1.100.log.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("192.168.1.100.log.gz.1", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("192.168.1.100.log.gz.old.1", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.192.168.1.100", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("setup.log.full", FileType::File, &MIMEGUESS_TXT)]
#[test_case("btmp", FileType::Unparseable, &MIMEGUESS_EMPTY)]
#[test_case("utmp", FileType::Unparseable, &MIMEGUESS_EMPTY)]
#[test_case("wtmp", FileType::Unparseable, &MIMEGUESS_EMPTY)]
fn test_path_to_filetype_mimeguess(
    path_str: &str,
    filetype: FileType,
    mimeguess: &MimeGuess,
) {
    dpfn!("({:?})", path_str);
    // test the file name and full path
    let fpath: FPath = FPath::from_str(path_str).unwrap();
    let fpath_full: FPath = FPath::from_str("/var/log/").unwrap() + fpath.as_str();
    for fpath_ in [&fpath, &fpath_full].iter() {
        let (filetype_, mimeguess_) = fpath_to_filetype_mimeguess(fpath_);
        assert_eq!(filetype, filetype_, "\nfpath {:?}\nExpected {:?}\nActual   {:?}\n", fpath_, filetype, filetype_);
        assert_eq!(mimeguess, &mimeguess_, "\nfpath {:?}\nExpected {:?}\nActual   {:?}\n", fpath_, mimeguess, mimeguess_);
    }
    dpfx!();
}
