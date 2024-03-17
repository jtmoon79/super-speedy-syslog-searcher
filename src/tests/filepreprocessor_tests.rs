// src/tests/filepreprocessor_tests.rs

//! tests for `filepreprocessor.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::tests::common::{
    MIMEGUESS_EMPTY,
    MIMEGUESS_GZ,
    MIMEGUESS_LOG,
    MIMEGUESS_TAR,
    MIMEGUESS_TARGZ,
    MIMEGUESS_TXT,
    MIMEGUESS_UTMP,
    MIMEGUESS_XZ,
    MIMEGUESS_EVTX,
    MIMEGUESS_ETL,
    MIMEGUESS_ZIP,
    NTF_GZ_EMPTY,
    NTF_GZ_EMPTY_FILETYPE,
    NTF_GZ_EMPTY_FPATH,
    NTF_GZ_EMPTY_MIMEGUESS,
    NTF_LOG_EMPTY,
    NTF_LOG_EMPTY_FILETYPE,
    NTF_LOG_EMPTY_FPATH,
    NTF_LOG_EMPTY_MIMEGUESS,
    NTF_TAR_1BYTE,
    NTF_TAR_1BYTE_FILEA_FILETYPE,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FILEA_MIMEGUESS,
    NTF_TAR_8BYTE_FILEA_FILETYPE,
    NTF_TAR_8BYTE_FILEA_FPATH,
    NTF_TAR_8BYTE_FILEA_MIMEGUESS,
    NTF_TAR_8BYTE_FPATH,
    NTF_TAR_AB_FILEA_FILETYPE,
    NTF_TAR_AB_FILEA_FPATH,
    NTF_TAR_AB_FILEA_MIMEGUESS,
    NTF_TAR_AB_FILEB_FILETYPE,
    NTF_TAR_AB_FILEB_FPATH,
    NTF_TAR_AB_FILEB_MIMEGUESS,
    NTF_TAR_AB_FPATH,
    NTF_TGZ_8BYTE,
    NTF_TGZ_8BYTE_FILETYPE,
    NTF_TGZ_8BYTE_FPATH,
    NTF_TGZ_8BYTE_MIMEGUESS,
};
use crate::common::{FPath, FileType, FixedStructFileType, Path};
use crate::readers::filepreprocessor::{
    copy_process_path_result_canonicalize_path,
    fpath_to_filetype,
    fpath_to_filetype_mimeguess,
    mimeguess_to_filetype,
    process_path,
    process_path_tar,
    MimeGuess,
    ProcessPathResult,
};
use crate::readers::helpers::{fpath_to_path, path_to_fpath};
use crate::debug::helpers::{create_files_and_tmpdir, ntf_fpath, NamedTempFile};

use std::str::FromStr;

#[allow(unused_imports)]
use ::filepath::FilePath; // provide `path` function on `File`
use ::lazy_static::lazy_static;
use ::si_trace_print::stack::stack_offset_set;
use ::si_trace_print::{defn, defo, defx};
use ::test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn test_mimeguess_to_filetype(
    mimeguess: &MimeGuess,
    check: FileType,
) {
    let filetype: FileType = mimeguess_to_filetype(mimeguess);
    assert_eq!(check, filetype, "\n  expected FileType::{:?}\n  found FileType::{:?}\n", check, filetype);
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

#[test]
fn test_mimeguess_to_filetype_targz() {
    test_mimeguess_to_filetype(&MIMEGUESS_TARGZ, FileType::Unparsable);
}

#[test]
fn test_mimeguess_to_filetype_utmp() {
    test_mimeguess_to_filetype(&MIMEGUESS_UTMP, FileType::Unknown);
}

#[test]
fn test_mimeguess_to_filetype_evtx() {
    test_mimeguess_to_filetype(&MIMEGUESS_EVTX, FileType::Unknown);
}

// -------------------------------------------------------------------------------------------------

#[test_case("log", FileType::File)]
#[test_case("LOG", FileType::File; "LOG ALLCAPS")]
#[test_case("log.log", FileType::File)]
#[test_case("log_media", FileType::File)]
#[test_case("media_log", FileType::File)]
#[test_case("MY_LOG", FileType::File)]
#[test_case("media.log.old", FileType::File)]
#[test_case("syslog", FileType::File)]
#[test_case("messages", FileType::File)]
#[test_case("rhsm.log-20230422", FileType::Unknown)]
#[test_case("2023.10.26.asl", FileType::Unknown)]
#[test_case("BB.2024.10.31.G80.asl", FileType::Unknown)]
#[test_case("a.log", FileType::Unknown)]
#[test_case("log.a", FileType::File)]
#[test_case("LOG.B", FileType::File)]
#[test_case("log.1", FileType::File)]
#[test_case("log.2", FileType::File)]
#[test_case("data.gz", FileType::Gz)]
#[test_case("DATA.GZ", FileType::Gz; "DATA.GZ ALLCAPS")]
#[test_case("data.gz.old", FileType::Gz)]
#[test_case("data.gzip", FileType::Gz)]
#[test_case("data.tgz", FileType::Unparsable)]
#[test_case("data.tar", FileType::Tar)]
#[test_case("DATA.TAR", FileType::Tar; "DATA.TAR ALLCAPS")]
#[test_case("data.tar.old", FileType::Tar)]
#[test_case("data.tgz.old", FileType::Unparsable)]
#[test_case("wtmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp}; "wtmp")]
#[test_case("WTMP", FileType::FixedStruct{ type_: FixedStructFileType::Utmp}; "WTMP ALLCAPS")]
#[test_case("btmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp}; "btmp")]
#[test_case("utmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp}; "utmp")]
#[test_case("UTMP", FileType::FixedStruct{ type_: FixedStructFileType::Utmp}; "UTMP ALLCAPS")]
#[test_case("UTMP.1", FileType::FixedStruct{ type_: FixedStructFileType::Utmp}; "UTMP.1 ALLCAPS")]
#[test_case("host.wtmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp})]
#[test_case("192.168.1.1.btmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp})]
#[test_case("file.utmp", FileType::FixedStruct{type_: FixedStructFileType::Utmp})]
#[test_case("btmpx", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx}; "btmpx")]
#[test_case("utmpx", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx}; "utmpx")]
#[test_case("utmpx.bak", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx}; "utmpx.bak")]
#[test_case("wtmpx", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx}; "wtmpx")]
#[test_case("wtmpx.1", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx}; "wtmpx.1")]
#[test_case("lastlog", FileType::FixedStruct{ type_: FixedStructFileType::Lastlog})]
#[test_case("lastlog.1", FileType::FixedStruct{ type_: FixedStructFileType::Lastlog})]
#[test_case("lastlogx", FileType::FixedStruct{ type_: FixedStructFileType::Lastlogx})]
#[test_case("acct", FileType::FixedStruct{ type_: FixedStructFileType::Acct})]
#[test_case("acct.2", FileType::FixedStruct{ type_: FixedStructFileType::Acct})]
#[test_case("acct-20220101", FileType::File)]
#[test_case("pacct", FileType::FixedStruct{ type_: FixedStructFileType::AcctV3})]
#[test_case("pacct.1", FileType::FixedStruct{ type_: FixedStructFileType::AcctV3})]
#[test_case("pacct.20220101", FileType::FixedStruct{ type_: FixedStructFileType::AcctV3})]
#[test_case("somefile", FileType::File)]
#[test_case("SOMEFILE", FileType::File; "SOMEFILE ALLCAPS")]
#[test_case("file.evtx", FileType::Evtx)]
#[test_case("FILE.EVTX", FileType::Evtx; "FILE.EVTX ALLCAPS")]
#[test_case("a.journal", FileType::Journal)]
#[test_case("A.JOURNAL", FileType::Journal; "A.JOURNAL ALLCAPS")]
#[test_case("user-1000.journal", FileType::Journal)]
#[test_case("system@a8b80590f2654a95aed5c11b3c9e3c48-0000000000000001-0005f6f737b6b0e0.journal", FileType::Journal)]
fn test_fpath_to_filetype(
    name: &str,
    check: FileType,
) {
    stack_offset_set(Some(2));
    let fpath: FPath = FPath::from(name);
    let fpath_full: FPath = FPath::from("/var/log/") + fpath.as_str();
    for path in [&fpath, &fpath_full].iter() {
        defo!("fpath_to_filetype({:?})", path);
        let filetype = fpath_to_filetype(path);
        defo!("fpath_to_filetype returned {:?}", filetype);
        assert_eq!(check, filetype, "\npath {:?}\nexpected FileType::{:?}\nactual FileType::{:?}\n", path, check, filetype);
    }
}

// -------------------------------------------------------------------------------------------------

fn test_process_path_fpath(
    path: &FPath,
    checks: Vec<ProcessPathResult>,
) {
    defn!("({:?}, …)", path);
    let results = process_path(path);
    for (i, result) in results.iter().enumerate() {
        defo!("result[{}] = {:?}", i, result);
    }
    // XXX: create a copy of `results`, canonicalize every `fpath` within each `ProcessPathResult`
    //      on some Windows systems, `process_path` will return an MS-DOS shortened form of a path
    //      e.g. `"C:\\Users\\RUNNER~1\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
    //           !=
    //           `"C:\\Users\\runneradmin\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
    //      So try harder to make sure the comparison succeeds.
    let mut results_can: Vec<ProcessPathResult> = vec![];
    for result in results.into_iter() {
        let result_can = copy_process_path_result_canonicalize_path(result);
        results_can.push(result_can);
    }
    for (i, result_can) in results_can.iter().enumerate() {
        defo!("result_can[{}] = {:?}", i, result_can);
    }
    // create copy of `checks` for the same reason as `results_can` above
    let mut checks_can: Vec<ProcessPathResult> = vec![];
    for check in checks.into_iter() {
        let check_can = copy_process_path_result_canonicalize_path(check);
        checks_can.push(check_can);
    }
    for (i, check_can) in checks_can.iter().enumerate() {
        defo!("check_can[{}] = {:?}", i, check_can);
    }
    // check that each `check` is in the `results`
    for (i, check) in checks_can.iter().enumerate() {
        defo!("check[{}] = {:?}", i, check);
        assert!(
            results_can.contains(check),
            "\nprocess_path({:?})\n  the check {:?}\n  is not contained in the results:\n       {:?}\n",
            path,
            check,
            results_can,
        );
    }
    // check that each `result` is in the `checks`
    for result in results_can.iter() {
        assert!(
            checks_can.contains(result),
            "\nprocess_path({:?})\n  the result {:?}\n  is not contained in the checks:\n       {:?}\n",
            path,
            result,
            checks_can,
        );
    }
    defx!();
}

fn test_process_path_ntf(
    ntf: &NamedTempFile,
    checks: Vec<ProcessPathResult>,
) {
    stack_offset_set(Some(2));
    let path = ntf_fpath(ntf);
    test_process_path_fpath(&path, checks);
}

// test individual files

#[test]
fn test_process_path_1_log() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_LOG_EMPTY_FPATH.clone(),
            *NTF_LOG_EMPTY_MIMEGUESS,
            NTF_LOG_EMPTY_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_LOG_EMPTY, checks);
}

#[test]
fn test_process_path_1_gz() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_GZ_EMPTY_FPATH.clone(),
            *NTF_GZ_EMPTY_MIMEGUESS,
            NTF_GZ_EMPTY_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_GZ_EMPTY, checks);
}

#[test]
fn test_process_path_1_tar() {
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_1BYTE_FILEA_FPATH.clone(),
            *NTF_TAR_1BYTE_FILEA_MIMEGUESS,
            NTF_TAR_1BYTE_FILEA_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_TAR_1BYTE, checks);
}

#[test]
fn test_process_path_1_tgz() {
    // XXX: TarGz is recognized but not supported Issue #14
    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TGZ_8BYTE_FPATH.clone(),
            *NTF_TGZ_8BYTE_MIMEGUESS,
            NTF_TGZ_8BYTE_FILETYPE,
        ),
    ];
    test_process_path_ntf(&NTF_TGZ_8BYTE, checks);
}

#[test]
fn test_process_path_1_not_exist() {
    let path: FPath = FPath::from("/THIS/FILE/DOES/NOT/EXIST!");
    let checks: Vec<ProcessPathResult> = vec![ProcessPathResult::FileErrNotExist(path.clone())];
    test_process_path_fpath(&path, checks);
}

#[test]
fn test_process_path_1_not_a_file() {
    let fpath: FPath = FPath::from("/dev/null");
    // do not test if path does not exist; avoids failures on unusual platforms
    if !fpath_to_path(&fpath).exists() {
        defo!("Path '{:?}' does not exist, pass test", fpath);
        return;
    }
    let checks: Vec<ProcessPathResult> = vec![ProcessPathResult::FileErrNotAFile(fpath.clone())];
    test_process_path_fpath(&fpath, checks);
}

// test directories of files

#[test]
fn test_process_path_dir1_file1() {
    let filenames = &[FPath::from("file1")];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> =
        vec![ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), *MIMEGUESS_EMPTY, FileType::File)];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

#[test]
fn test_process_path_dir2_file1_txt1() {
    let filenames = &[
        FPath::from("file1"),
        FPath::from("file2.txt"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), *MIMEGUESS_EMPTY, FileType::File),
        ProcessPathResult::FileValid(fpaths.get(1).unwrap().clone(), *MIMEGUESS_TXT, FileType::File),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

#[test]
fn test_process_path_dir3_gz1_tar1_txt1() {
    let filenames = &[
        FPath::from("file1.gz"),
        FPath::from("file2.tar"),
        FPath::from("file3.txt"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), *MIMEGUESS_GZ, FileType::Gz),
        // no .tar file in results
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), *MIMEGUESS_TXT, FileType::File),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

#[test]
fn test_process_path_dir4_dirA_file1() {
    let filenames = &[FPath::from(
        "dirA/fileA1.txt",
    )];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> =
        vec![ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), *MIMEGUESS_TXT, FileType::File)];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

#[test]
fn test_process_path_dir5_dirABC_files3() {
    let filenames = &[
        FPath::from("file1.txt"),
        FPath::from("dirA/fileA1.txt"),
        FPath::from("dirA/fileA2.gz"),
        FPath::from("dirB/"),
        FPath::from("dirC/"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(fpaths.get(0).unwrap().clone(), *MIMEGUESS_TXT, FileType::File),
        ProcessPathResult::FileValid(fpaths.get(1).unwrap().clone(), *MIMEGUESS_TXT, FileType::File),
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), *MIMEGUESS_GZ, FileType::Gz),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

#[test]
fn test_process_path_dir6_dirABC_files6() {
    let filenames = &[
        FPath::from("dirA1/dirA2/fileA12.tar"),
        FPath::from("dirB/fileB1.gz"),
        FPath::from("dirB/fileB2.xz"),
        FPath::from("dirB/fileB3.xz.tar"),
        FPath::from("dirB/fileB4.tar.xz"),
        FPath::from("dirC/fileC1.tgz"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        // fileA12.tar will not be in results
        ProcessPathResult::FileValid(fpaths.get(1).unwrap().clone(), *MIMEGUESS_GZ, FileType::Gz),
        ProcessPathResult::FileValid(fpaths.get(2).unwrap().clone(), *MIMEGUESS_XZ, FileType::Xz),
        // fileB3.xz.tar will not be in results
        ProcessPathResult::FileValid(fpaths.get(4).unwrap().clone(), *MIMEGUESS_XZ, FileType::Xz),
        ProcessPathResult::FileErrNotSupported(fpaths.get(5).unwrap().clone(), *MIMEGUESS_TARGZ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

#[test]
fn test_process_path_dir7_dirAB_files4() {
    let filenames = &[
        FPath::from("dirA1/system@f2e8a336aa58640aa39cac58b6ffc7e7-0000000000294e62-0d05dc1215b8e84c.journal"),
        FPath::from("dirB/picture.bmp"),
        FPath::from("dirB/picture.png"),
        FPath::from("dirB/this.crazy.file.name.has.many.extensions.chars.within.the.name"),
    ];
    let (dir, fpaths) = create_files_and_tmpdir(filenames);

    let checks: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            fpaths.get(0).unwrap().clone(), MimeGuess::from_ext("journal"), FileType::Journal
        ),
        ProcessPathResult::FileValid(
            fpaths.get(1).unwrap().clone(), MimeGuess::from_ext("bmp"), FileType::File
        ),
        ProcessPathResult::FileValid(
            fpaths.get(2).unwrap().clone(), MimeGuess::from_ext("png"), FileType::File
        ),
        ProcessPathResult::FileValid(
            fpaths.get(3).unwrap().clone(), *MIMEGUESS_EMPTY, FileType::Unknown
        ),
    ];

    test_process_path_fpath(&path_to_fpath(dir.path()), checks);
}

// -------------------------------------------------------------------------------------------------

fn test_process_path_tar(
    path: &FPath,
    checks: &Vec<ProcessPathResult>,
) {
    defn!("test_process_path_tar({:?}, …)", path);
    let results = process_path_tar(path);
    for check in checks.iter() {
        assert!(
            results.contains(check),
            "\nprocess_path({:?})\n  the check {:?}\n  is not contained in the results:\n       {:?}\n",
            path, check, results,
        );
    }
    for result in results.iter() {
        assert!(
            checks.contains(result),
            "\nprocess_path({:?})\n  the result {:?}\n  is not contained in the checks:\n       {:?}\n",
            path, result, checks,
        );
    }
    defx!();
}

#[test]
fn test_process_path_tar_tar1_file1() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_8BYTE_FILEA_FPATH.clone(),
            *NTF_TAR_8BYTE_FILEA_MIMEGUESS,
            NTF_TAR_8BYTE_FILEA_FILETYPE,
        ),
    ];

    test_process_path_tar(&NTF_TAR_8BYTE_FPATH, &check);
}

#[test]
fn test_process_path_tar_tar1_file2() {
    let check: Vec<ProcessPathResult> = vec![
        ProcessPathResult::FileValid(
            NTF_TAR_AB_FILEA_FPATH.clone(),
            *NTF_TAR_AB_FILEA_MIMEGUESS,
            NTF_TAR_AB_FILEA_FILETYPE,
        ),
        ProcessPathResult::FileValid(
            NTF_TAR_AB_FILEB_FPATH.clone(),
            *NTF_TAR_AB_FILEB_MIMEGUESS,
            NTF_TAR_AB_FILEB_FILETYPE,
        ),
    ];

    test_process_path_tar(&NTF_TAR_AB_FPATH, &check);
}

// -------------------------------------------------------------------------------------------------

lazy_static! {
    pub static ref MIMEGUESS_LOG_1: MimeGuess = MimeGuess::from_path(Path::new("test.log"));
}

/// test `fpath_to_filetype_mimeguess` (and `path_to_filetype_mimeguess`)
#[test_case("messages", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("MESSAGES", FileType::File, &MIMEGUESS_EMPTY; "MESSAGES ALLCAPS")]
#[test_case("pagefile.sys", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("syslog", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("syslog~", FileType::File, &MIMEGUESS_EMPTY; "syslog_tilde")]
#[test_case("syslog-", FileType::File, &MIMEGUESS_EMPTY; "syslog_dash")]
#[test_case("syslog.3", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("output.txt", FileType::File, &MIMEGUESS_TXT)]
#[test_case("cloud-init.log.out", FileType::File, &MIMEGUESS_LOG)]
#[test_case("cloud-init.out.log", FileType::File, &MIMEGUESS_LOG)]
#[test_case("cloud-init-output.log", FileType::File, &MIMEGUESS_LOG)]
#[test_case("droplet-agent.update.log", FileType::File, &MIMEGUESS_LOG)]
#[test_case("kern.log", FileType::File, &MIMEGUESS_LOG)]
#[test_case("KERN.LOG", FileType::File, &MIMEGUESS_LOG; "KERN.LOG ALLCAPS")]
#[test_case("kern.log.1", FileType::File, &MIMEGUESS_LOG)]
#[test_case("kern.log.2", FileType::File, &MIMEGUESS_LOG)]
#[test_case("kern.log.2~", FileType::File, &MIMEGUESS_LOG; "kern.log.2_tilde")]
#[test_case("rhsm.log-20230422", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("aptitude.4", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("aptitude.~", FileType::File, &MIMEGUESS_EMPTY; "aptitude_tilde")]
#[test_case("systemsetup-server-info.log.208", FileType::File, &MIMEGUESS_LOG)]
#[test_case("syslog.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("syslog.9.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("SYSLOG.9.GZ", FileType::Gz, &MIMEGUESS_GZ; "SYSLOG.9.GZ")]
#[test_case("logs.tgz", FileType::Unparsable, &MIMEGUESS_TARGZ)]
#[test_case("unattended-upgrades-dpkg.log.3.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("eipp.log.xz", FileType::Xz, &MIMEGUESS_XZ)]
#[test_case("logs.tar", FileType::Tar, &MIMEGUESS_TAR)]
#[test_case("LOGS.TAR", FileType::Tar, &MIMEGUESS_TAR; "LOGS.TAR")]
#[test_case("log.1.tar", FileType::Tar, &MIMEGUESS_TAR)]
#[test_case("LOG.1.TAR", FileType::Tar, &MIMEGUESS_TAR; "LOG.1.TAR ALLCAPS")]
#[test_case("data.tgz.old.1", FileType::Unparsable, &MIMEGUESS_TARGZ)]
#[test_case("data.tgz.old", FileType::Unparsable, &MIMEGUESS_TARGZ)]
#[test_case("HOSTNAME.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("log.HOSTNAME", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.nmbd", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("LOG.NMDB", FileType::File, &MIMEGUESS_EMPTY; "LOG.NMDB")]
#[test_case("log.nmbd.1", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.nmbd.old", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("log.gz.1", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.gz.2", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.gz.99", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.tgz.99", FileType::Unparsable, &MIMEGUESS_TARGZ)]
#[test_case("logs.tgz.99", FileType::Unparsable, &MIMEGUESS_TARGZ)]
#[test_case("LOGS.TGZ.99", FileType::Unparsable, &MIMEGUESS_TARGZ; "LOGS.TGZ.99")]
#[test_case("-.tgz.99", FileType::Unparsable, &MIMEGUESS_TARGZ)]
#[test_case("soap_agent", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("soap_agent.old", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("soap_agent.old.old", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("2023.10.26.asl", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("BB.2024.10.31.G80.asl", FileType::Unknown, &MIMEGUESS_EMPTY)]
#[test_case("telemetry", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("initial-status", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("smart_extend_log", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case(".disk_daily_info_send_udc_time", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("messages-DropletAgent", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("CC_AA_DD_EE_FF_00-ns", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("CC_AA_DD_EE_FF_00-ns.old", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("CC_AA_DD_EE_FF_00-ns.old.1", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("history", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("fe80::984c:ffff:eeee:eeee.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("[fe80::984c:ffff:eeee:eeef].log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("錄音.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("opname.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("บันทึก.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("innspilling.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("Запису.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("تسجيل.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("grabación.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("錄音.檔", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("錄音", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("บันทึก", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("innspilling", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("Запису", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("تسجيل", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("grabación", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("192.168.1.100.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("192.168.1.100.log.gz", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("192.168.1.100.log.gz.1", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("192.168.1.100.log.gz.old.1", FileType::Gz, &MIMEGUESS_GZ)]
#[test_case("log.192.168.1.100", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("setup.log.full", FileType::File, &MIMEGUESS_TXT)]
#[test_case("setup.log.full.1", FileType::File, &MIMEGUESS_TXT)]
#[test_case("setup.log.full.old", FileType::File, &MIMEGUESS_TXT)]
#[test_case("setup.log.full.old.1", FileType::File, &MIMEGUESS_TXT)]
#[test_case("setup.log.full.old.2", FileType::File, &MIMEGUESS_TXT)]
// on FreeBSD 13, there is a log file `utx.log` that is a variable-length utmpx-ish format file
#[test_case("utx.log", FileType::File, &MIMEGUESS_TXT)]
#[test_case("utx.log-", FileType::File, &MIMEGUESS_TXT; "utx.log dash")]
#[test_case("utx.active", FileType::File, &MIMEGUESS_EMPTY)]
// File `utx.lastlogin` exists on FreeBSD 13.   
#[test_case("utx.lastlogin", FileType::File, &MIMEGUESS_EMPTY)]
#[test_case("lastlog", FileType::FixedStruct{ type_: FixedStructFileType::Lastlog }, &MIMEGUESS_EMPTY)]
#[test_case("lastlog.1", FileType::FixedStruct{ type_: FixedStructFileType::Lastlog }, &MIMEGUESS_EMPTY)]
#[test_case("lastlog.bak", FileType::FixedStruct{ type_: FixedStructFileType::Lastlog }, &MIMEGUESS_EMPTY)]
#[test_case("lastlog.2.bak", FileType::FixedStruct{ type_: FixedStructFileType::Lastlog }, &MIMEGUESS_EMPTY)]
#[test_case("btmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY)]
#[test_case("utmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY)]
#[test_case("wtmp", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY)]
#[test_case("WTMP", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY; "WTMP ALLCAPS")]
#[test_case("btmpx", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx }, &MIMEGUESS_EMPTY)]
#[test_case("utmpx", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx }, &MIMEGUESS_EMPTY)]
#[test_case("wtmpx", FileType::FixedStruct{ type_: FixedStructFileType::Utmpx }, &MIMEGUESS_EMPTY)]
#[test_case("wtmpx~", FileType::File, &MIMEGUESS_EMPTY; "wtmpx_tilde")] // results in `FileType::File` because of the tilde
#[test_case("btmp.1", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY)]
#[test_case("utmp.2", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY)]
#[test_case("wtmp.1", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY)]
#[test_case("WTMP.1", FileType::FixedStruct{ type_: FixedStructFileType::Utmp }, &MIMEGUESS_EMPTY; "WTMP.1 ALLCAPS")]
#[test_case("file.evtx", FileType::Evtx, &MIMEGUESS_EVTX)]
#[test_case("FILE.EVTX", FileType::Evtx, &MIMEGUESS_EVTX; "FILE.EVTX ALLCAPS")]
#[test_case("file.evtx.1", FileType::Evtx, &MIMEGUESS_EVTX)]
#[test_case("a.journal", FileType::Journal, &MIMEGUESS_EMPTY)]
#[test_case("a.journal~", FileType::Journal, &MIMEGUESS_EMPTY; "a.journal tilde")]
#[test_case("a.journal~.1", FileType::Journal, &MIMEGUESS_EMPTY; "a.journal tilde 1")]
#[test_case("a.journal~.old", FileType::Journal, &MIMEGUESS_EMPTY; "a.journal tilde old")]
#[test_case("A.JOURNAL", FileType::Journal, &MIMEGUESS_EMPTY; "A.JOURNAL ALLCAPS")]
#[test_case("A.JOURNAL~", FileType::Journal, &MIMEGUESS_EMPTY; "A.JOURNAL ALLCAPS tilde")]
#[test_case("user-1000.journal", FileType::Journal, &MIMEGUESS_EMPTY)]
#[test_case("user-1000@2feff012228b405bb557ccd80a0ba755-000000005100032b-0006129e5481135e.journal", FileType::Journal, &MIMEGUESS_EMPTY)]
#[test_case("-", FileType::File, &MIMEGUESS_EMPTY; "dash")]
#[test_case("$", FileType::File, &MIMEGUESS_EMPTY; "dollar")]
#[test_case("SIH.20230422.034724.362.1.etl", FileType::Unparsable, &MIMEGUESS_ETL)]
#[test_case("logs.zip", FileType::Unparsable, &MIMEGUESS_ZIP)]
fn test_path_to_filetype_mimeguess(
    path_str: &str,
    filetype: FileType,
    mimeguess: &MimeGuess,
) {
    defn!("({:?})", path_str);
    // test the file name and full path
    let fpath: FPath = FPath::from_str(path_str).unwrap();
    let fpath_full: FPath = FPath::from_str("/var/log/").unwrap() + fpath.as_str();
    for fpath_ in [&fpath, &fpath_full].iter() {
        let (filetype_, mimeguess_) = fpath_to_filetype_mimeguess(fpath_);
        assert_eq!(filetype, filetype_, "\nfpath {:?}\nExpected {:?}\nActual   {:?}\n", fpath_, filetype, filetype_);
        assert_eq!(mimeguess, &mimeguess_, "\nfpath {:?}\nExpected {:?}\nActual   {:?}\n", fpath_, mimeguess, mimeguess_);
    }
    defx!();
}
