// src/tests/fixedstructreader_tests.rs

//! tests for `fixedstructreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{
    Count,
    FileOffset,
    FileType,
    FixedStructFileType,
    FPath,
};
use crate::data::datetime::FixedOffset;
use crate::data::fixedstruct::{
    FixedStruct,
    FixedStructType,
    ENTRY_SZ_MAX,
    linux_x86,
};
use crate::readers::blockreader::{
    BlockOffset,
    BlockSz,
    SummaryBlockReader,
};
use crate::readers::summary::SummaryReaderData;
use crate::readers::fixedstructreader::{
    FixedStructReader,
    ResultFixedStructReaderNew,
    ResultFixedStructReaderNewError,
    ResultS3FixedStructFind,
    SummaryFixedStructReader,
};
use crate::tests::common::{
    FO_0,
    FO_P8,
    LINUX_X86_UTMPX_2ENTRY_FILESZ,
    LINUX_X86_LASTLOG_BUFFER1_DTO,
    NTF_LOG_EMPTY_FPATH,
    NTF_NL_1_PATH,
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_1ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH,
    NTF_LINUX_X86_UTMPX_00_ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_55_ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_AA_ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_FF_ENTRY_FPATH,
    LINUX_X86_UTMPX_BUFFER1_DT,
    LINUX_X86_UTMPX_BUFFER1_DTO,
    LINUX_X86_UTMPX_BUFFER2_DTO,
    LINUX_X86_UTMPX_BUFFER3_DTO,
};

use std::any::Any;
use std::collections::HashSet;

use ::chrono::Duration;
#[allow(unused_imports)]
use ::more_asserts::{assert_gt, assert_ge};
use ::test_case::test_case;
#[allow(unused_imports)]
use ::si_trace_print::printers::{defn, defo, defx, defñ};
use ::si_trace_print::stack::stack_offset_set;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// short alias
const U1SZ: FileOffset = linux_x86::UTMPX_SZ as FileOffset;
const L1SZ: FileOffset = linux_x86::LASTLOG_SZ as FileOffset;

// short alias
const UFS: FixedStructFileType = FixedStructFileType::Utmpx;
const LFS: FixedStructFileType = FixedStructFileType::Lastlog;

/// Helper to wrap the match and panic checks
fn new_FixedStructReader(
    path: &FPath,
    blocksz: BlockSz,
    tzo: FixedOffset,
) -> FixedStructReader {
    stack_offset_set(Some(2));
    match FixedStructReader::new(
        path.clone(),
        FileType::FixedStruct{ type_: FixedStructFileType::Utmpx },
        blocksz,
        tzo,
        None,
        None,
    ) {
        ResultFixedStructReaderNewError::FileOk(val) => val,
        result => {
            panic!(
                "ERROR: FixedStructReader::new({:?}, {:?}, {:?}) failed: {:?}",
                path, blocksz, tzo, result,
            );
        }
    }
}

#[test]
fn test_new_FixedStructReader_1_empty() {
    match FixedStructReader::new(
        NTF_LOG_EMPTY_FPATH.clone(),
        FileType::FixedStruct{ type_: FixedStructFileType::Utmpx },
        1024,
        *FO_P8,
        None,
        None,
    ) {
        //ResultFixedStructReaderNew::FileOk(_) => {},
        ResultFixedStructReaderNewError::FileErrEmpty => {},
        result => {
            panic!(
                "expected FileOk for empty file NTF_LOG_EMPTY_FPATH, got {:?}",
                result
            );
        }
    }
}

#[test]
fn test_new_FixedStructReader_2_bad_noerr() {
    match FixedStructReader::new(
        NTF_NL_1_PATH.clone(),
        FileType::FixedStruct{ type_: FixedStructFileType::Utmpx },
        1024,
        *FO_P8,
        None,
        None,
    ) {
        ResultFixedStructReaderNewError::FileErrTooSmall(_) => {},
        result => {
            panic!(
                "expected FileErrTooSmall for empty file NTF_NL_1_PATH, got {:?}",
                result
            );
        }
    }
}

#[test]
fn test_FixedStructReader_helpers() {
    const BSZ: BlockSz = 64;
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    let mut fsr = new_FixedStructReader(
        &NTF_LINUX_X86_UTMPX_2ENTRY_FPATH,
        BSZ,
        *FO_P8
    );

    assert_eq!(fsr.block_index_at_file_offset(0), 0);
    assert_eq!(fsr.block_offset_at_file_offset(0), 0);
    assert_eq!(fsr.blockoffset_last(), LINUX_X86_UTMPX_2ENTRY_FILESZ / BSZ - 1);
    assert_eq!(fsr.blocksz(), BSZ);
    assert_eq!(fsr.count_blocks(), LINUX_X86_UTMPX_2ENTRY_FILESZ / BSZ);
    assert_eq!(fsr.count_entries_processed(), 2);
    assert_eq!(fsr.file_offset_at_block_offset(0), 0);
    assert_eq!(fsr.file_offset_at_block_offset_index(0, 1), 1);
    assert_eq!(fsr.fileoffset_last() + 1, LINUX_X86_UTMPX_2ENTRY_FILESZ as FileOffset);
    assert_eq!(fsr.fileoffset_to_fixedstructoffset(0), 0);
    assert_eq!(fsr.filesz(), LINUX_X86_UTMPX_2ENTRY_FILESZ);
    assert_eq!(fsr.fixedstruct_size(), linux_x86::UTMPX_SZ);
    assert_eq!(fsr.fixedstruct_size_fo(), linux_x86::UTMPX_SZ_FO);
    let fo_last = fsr.fileoffset_last();
    eprintln!("fo_last: {}", fo_last);
    eprintln!("linux_x86::UTMPX_SZ_FO: {}", linux_x86::UTMPX_SZ_FO);
    assert!(!fsr.is_fileoffset_last(0));
    assert!(!fsr.is_fileoffset_last(linux_x86::UTMPX_SZ_FO));
    assert!(!fsr.is_fileoffset_last(linux_x86::UTMPX_SZ_FO * 2));
    assert!(fsr.is_fileoffset_last(linux_x86::UTMPX_SZ_FO * 2 - 1));
    // entry 1
    let fo_next: FileOffset = 0;
    let (fo_next, fs) = match fsr.process_entry_at(fo_next, &mut buffer) {
        ResultS3FixedStructFind::Found((fo, fs)) => (fo, fs),
        _ => panic!("process_entry_at({}) failed", fo_next),
    };
    assert!(!fsr.is_fileoffset_last(fo_next));
    assert!(!fsr.is_last(&fs));
    // entry 2
    let (fo_next, fs) = match fsr.process_entry_at(fo_next, &mut buffer) {
        ResultS3FixedStructFind::Found((fo, fs)) => (fo, fs),
        _ => panic!("process_entry_at({}) failed", fo_next),
    };
    assert!(!fsr.is_fileoffset_last(fo_next));
    assert!(fsr.is_last(&fs));
    // entry 3
    match fsr.process_entry_at(fo_next, &mut buffer) {
        ResultS3FixedStructFind::Done => {},
        _ => panic!("process_entry_at({}) unexpected return", fo_next),
    };

    fsr.type_id();

    _ = fsr.get_fileoffsets();
}

#[derive(Debug, Eq, PartialEq)]
enum ResultS3FixedStructFind_Test {
    Found,
    Done,
}

const FOUND: ResultS3FixedStructFind_Test = ResultS3FixedStructFind_Test::Found;
const DONE: ResultS3FixedStructFind_Test = ResultS3FixedStructFind_Test::Done;

const NEWNODT: Option<ResultFixedStructReaderNewError> =
    Some(ResultFixedStructReaderNewError::FileErrNoFixedStructWithinDtFilters);
const NEWERRNOVALID: Option<ResultFixedStructReaderNewError> =
    Some(ResultFixedStructReaderNewError::FileErrNoValidFixedStruct);

const BSZ: BlockSz = 400;

/// test `FixedStructReader::find_entry`
// UTMPX_1ENTRY
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, 2, 0, FOUND, U1SZ, None; "UTMPX_1ENTRY a")]
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, 16, 0, FOUND, U1SZ, None; "UTMPX_1ENTRY b")]
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, BSZ, 0, FOUND, U1SZ, None; "UTMPX_1ENTRY c")]
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, BSZ, U1SZ, DONE, 0, None; "UTMPX_1ENTRY d")]
// UTMPX_2ENTRY
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 2, 0, FOUND, U1SZ, None; "UTMPX_2ENTRY a")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 16, 0, FOUND, U1SZ, None; "UTMPX_2ENTRY b")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, BSZ, 0, FOUND, U1SZ, None; "UTMPX_2ENTRY c")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, BSZ, 0, FOUND, U1SZ, None; "UTMPX_2ENTRY d")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 2, U1SZ, FOUND, U1SZ * 2, None; "UTMPX_2ENTRY e")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 16, U1SZ, FOUND, U1SZ * 2, None; "UTMPX_2ENTRY f")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 1024, U1SZ, FOUND, U1SZ * 2, None; "UTMPX_2ENTRY g")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, BSZ, U1SZ * 2, DONE, 0, None; "UTMPX_2ENTRY h")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, BSZ, U1SZ * 2, DONE, 0, None; "UTMPX_2ENTRY i")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, BSZ, U1SZ * 50, DONE, 0, None; "UTMPX_2ENTRY j")]
// UTMPX_3ENTRY
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, BSZ, 0, FOUND, U1SZ, None; "UTMPX_3ENTRY a")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, BSZ, U1SZ, FOUND, U1SZ * 2, None; "UTMPX_3ENTRY b")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, BSZ, U1SZ * 2, FOUND, U1SZ * 3, None; "UTMPX_3ENTRY c")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, BSZ, U1SZ * 3, DONE, 0, None; "UTMPX_3ENTRY d")]
// LINUX_X86_UTMPX_3ENTRY_OOO
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, BSZ, 0, FOUND, U1SZ * 3, None; "LINUX_X86_UTMPX_3ENTRY_OOO a")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, BSZ, U1SZ, FOUND, 0, None; "LINUX_X86_UTMPX_3ENTRY_OOO b")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, BSZ, U1SZ * 2, FOUND, U1SZ, None; "LINUX_X86_UTMPX_3ENTRY_OOO c")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, BSZ, U1SZ * 4, DONE, 0, None; "LINUX_X86_UTMPX_3ENTRY_OOO d")]
// UTMPX_00
#[test_case(&NTF_LINUX_X86_UTMPX_00_ENTRY_FPATH, UFS, 2, 0, DONE, 0, NEWERRNOVALID; "UTMPX_00 a")]
#[test_case(&NTF_LINUX_X86_UTMPX_00_ENTRY_FPATH, UFS, BSZ, 0, DONE, 0, NEWERRNOVALID; "UTMPX_00 b")]
// UTMPX_55
#[test_case(&NTF_LINUX_X86_UTMPX_55_ENTRY_FPATH, UFS, 2, 0, FOUND, U1SZ, None; "UTMPX_55 a")]
#[test_case(&NTF_LINUX_X86_UTMPX_55_ENTRY_FPATH, UFS, BSZ, 0, FOUND, U1SZ, None; "UTMPX_55 b")]
// UTMPX_AA
#[test_case(&NTF_LINUX_X86_UTMPX_AA_ENTRY_FPATH, UFS, 2, 0, DONE, 0, NEWERRNOVALID; "UTMPX_AA a")]
#[test_case(&NTF_LINUX_X86_UTMPX_AA_ENTRY_FPATH, UFS, BSZ, 0, DONE, 0, NEWERRNOVALID; "UTMPX_AA b")]
// UTMPX_FF
#[test_case(&NTF_LINUX_X86_UTMPX_FF_ENTRY_FPATH, UFS, 2, 0, DONE, 0, NEWERRNOVALID; "UTMPX_FF a")]
#[test_case(&NTF_LINUX_X86_UTMPX_FF_ENTRY_FPATH, UFS, BSZ, 0, DONE, 0, NEWERRNOVALID; "UTMPX_FF b")]
// LASTLOG1
#[test_case(&NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH, LFS, 2, 0, FOUND, L1SZ, None; "LASTLOG1 a")]
#[test_case(&NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH, LFS, BSZ, 0, FOUND, L1SZ, None; "LASTLOG1 b")]
fn test_FixedStructReader_process_entry_at(
    path: &FPath,
    fixedstructfiletype: FixedStructFileType,
    blocksz: BlockSz,
    fo: FileOffset,
    expect_result: ResultS3FixedStructFind_Test,
    expect_fo_index: FileOffset,
    expect_new: Option<ResultFixedStructReaderNewError>,
) {
    let mut fixedstructreader = match FixedStructReader::new(
        path.clone(),
        FileType::FixedStruct{ type_: fixedstructfiletype },
        blocksz,
        *FO_0,
        None,
        None,
    ) {
        ResultFixedStructReaderNewError::FileOk(val) => {
            assert!(expect_new.is_none(), "expected {:?} yet got FileOk", expect_new);

            val
        }
        ResultFixedStructReaderNewError::FileErrEmpty => {
            assert!(
                matches!(
                    expect_new.unwrap(),
                    ResultFixedStructReaderNewError::FileErrEmpty,
                ));
            assert_eq!(expect_result, ResultS3FixedStructFind_Test::Done);
            assert_eq!(expect_fo_index, 0);
            return;
        }
        ResultFixedStructReaderNew::FileErrNoValidFixedStruct => {
            assert!(
                matches!(
                    expect_new.unwrap(),
                    ResultFixedStructReaderNewError::FileErrNoValidFixedStruct,
                ));
            assert_eq!(expect_result, ResultS3FixedStructFind_Test::Done);
            assert_eq!(expect_fo_index, 0);
            return;
        }
        ResultFixedStructReaderNew::FileErrNoFixedStructWithinDtFilters => {
            assert!(
                matches!(
                    expect_new.unwrap(),
                    ResultFixedStructReaderNewError::FileErrNoFixedStructWithinDtFilters,
                ));
            assert_eq!(expect_result, ResultS3FixedStructFind_Test::Done);
            assert_eq!(expect_fo_index, 0);
            return;
        }
        ret => {
            panic!("unexpected ResultFixedStructReaderNewError {:?}", ret);
        }
    };

    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    let fo_next = match fixedstructreader.process_entry_at(fo, &mut buffer) {
        ResultS3FixedStructFind::Found((fo_, _fixedstruct)) => {
            assert!(matches!(expect_result, ResultS3FixedStructFind_Test::Found));
            fo_
        }
        ResultS3FixedStructFind::Done => {
            assert!(matches!(expect_result, ResultS3FixedStructFind_Test::Done));
            assert_eq!(expect_fo_index, 0);
            return;
        }
        ResultS3FixedStructFind::Err((_fo_opt, err)) => {
            panic!("process_entry_at({:?}) failed; {} for {:?}", fo, err, path);
        }
    };

    assert_eq!(fo_next, expect_fo_index, "expected fileoffset {}, got {}", expect_fo_index, fo_next);

}

/// test `FixedStructReader::process_entry_at` and `FixedStructReader::summary`
/// and `FixedStructReader::summary_complete`
#[test]
fn test_FixedStructReader_process_entry_at_2_summary() {
    let mut fixedstructreader = new_FixedStructReader(
        &NTF_LINUX_X86_UTMPX_2ENTRY_FPATH,
        BSZ,
        *FO_P8
    );

    let mut fo: FileOffset = 0;
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    loop {
        let result: ResultS3FixedStructFind = fixedstructreader.process_entry_at(fo, &mut buffer);
        match result {
            ResultS3FixedStructFind::Found((fo_, _utmpx)) => {
                fo = fo_;
            }
            ResultS3FixedStructFind::Done => {
                break;
            }
            ResultS3FixedStructFind::Err(err) => {
                panic!("Error {:?}", err);
            }
        }
    }
    // do one extra redundant search to make it little more interesting
    match fixedstructreader.process_entry_at(0, &mut buffer) {
        ResultS3FixedStructFind::Found(_) => {},
        _ => panic!(),
    }

    let summaryfixedstructreader = fixedstructreader.summary();
    assert_eq!(summaryfixedstructreader.fixedstructreader_utmp_entries, 3);
    assert_eq!(summaryfixedstructreader.fixedstructreader_utmp_entries_max, 2);
    assert_eq!(summaryfixedstructreader.fixedstructreader_utmp_entries_hit, 2);
    assert_eq!(summaryfixedstructreader.fixedstructreader_utmp_entries_miss, 1);
    assert_eq!(
        &summaryfixedstructreader.fixedstructreader_datetime_first.unwrap(),
        &*LINUX_X86_UTMPX_BUFFER1_DT,
    );
    assert_eq!(
        &summaryfixedstructreader.fixedstructreader_datetime_last.unwrap(),
        &*LINUX_X86_UTMPX_BUFFER2_DTO,
    );

    let summary = fixedstructreader.summary_complete();
    match summary.readerdata {
        SummaryReaderData::FixedStruct((
            _summaryblockreader,
            summaryfixedstructreader_,
        )) => {
            assert_eq!(summaryfixedstructreader_, summaryfixedstructreader)
        }
        _ => panic!(),
    }
}

// UTMPX_1ENTRY
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, 0, 0, Some(FOUND), U1SZ, None; "a UTMPX_1ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, 0, 1, None, 0, NEWNODT; "b UTMPX_1ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, UFS, U1SZ, 1, None, 0, NEWNODT; "c UTMPX_1ENTRY")]
// UTMPX_2ENTRY
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 0, 0, Some(FOUND), U1SZ, None; "a UTMPX_2ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, 0, 1, Some(FOUND), U1SZ * 2, None; "b UTMPX_2ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, U1SZ, 1, Some(FOUND), U1SZ * 2, None; "c UTMPX_2ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, UFS, U1SZ, 4, None, 0, NEWNODT; "d UTMPX_2ENTRY")]
// UTMPX_3ENTRY
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 2, 0, Some(FOUND), U1SZ * 3, None; "a UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ, 1, Some(FOUND), U1SZ * 2, None; "b UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ, 1, Some(FOUND), U1SZ * 2, None; "c UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 2, 0, Some(FOUND), U1SZ * 3, None; "d UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 2, 1, Some(FOUND), U1SZ * 3, None; "e UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 2, 2, Some(FOUND), U1SZ * 3, None; "f UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 2, 4, Some(FOUND), U1SZ * 3, None; "g UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 2, 5, None, 0, NEWNODT; "h UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, U1SZ * 3, 1, Some(DONE), 0, None; "i UTMPX_3ENTRY")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, UFS, 0, 5000, None, 0, NEWNODT; "j UTMPX_3ENTRY")]
// LINUX_X86_UTMPX_3ENTRY_OOO
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, 0, Some(FOUND), U1SZ * 3, None; "a LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, U1SZ, 0, Some(FOUND), 0, None; "b LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, U1SZ * 2, 0, Some(FOUND), U1SZ, None; "c LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, 1, Some(FOUND), U1SZ * 3, None; "d LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, 2, Some(FOUND), U1SZ * 3, None; "e LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, 3, Some(FOUND), U1SZ * 3, None; "f LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, 4, Some(FOUND), U1SZ * 3, None; "g LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, 5, None, 0, NEWNODT; "h LINUX_X86_UTMPX_3ENTRY_OOO")]
#[test_case(&NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, UFS, 0, -99, Some(FOUND), U1SZ * 3, None; "i LINUX_X86_UTMPX_3ENTRY_OOO")]
// UTMPX_00
#[test_case(&NTF_LINUX_X86_UTMPX_00_ENTRY_FPATH, UFS, 0, 0, None, 0, NEWERRNOVALID; "a UTMPX_00")]
// UTMPX_FF
#[test_case(&NTF_LINUX_X86_UTMPX_FF_ENTRY_FPATH, UFS, 0, 0, None, 0, NEWERRNOVALID; "a UTMPX_FF")]
fn test_FixedStructReader_read_find_entry_at_datetime_filter(
    path: &FPath, // pass to `FixedStructReader::new`
    fixedstructfiletype: FixedStructFileType, // pass to `FixedStructReader::new`
    fo: FileOffset, // pass to `process_entry_at`
    seconds: i64, // create this dt_filter with this adjustment from LINUX_X86_UTMPX_BUFFER1_DT
    expect_opt: Option<ResultS3FixedStructFind_Test>, // expected result of `process_entry_at`
    expect_fo: FileOffset, // expected result of `process_entry_at`
    new_result_opt: Option<ResultFixedStructReaderNewError>, // expected result of `FixedStructReader::new`
) {
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];

    match expect_opt {
        Some(_) => {
            assert!(new_result_opt.is_none(), "bad test input");
        }
        None => {
            assert!(new_result_opt.is_some(), "bad test input");
        }
    }

    let dt_filter = Some(
        *LINUX_X86_UTMPX_BUFFER1_DT + Duration::seconds(seconds),
    );

    let blocksz: BlockSz = 0x100;
    let tzo = *FO_P8;
    let mut fixedstructreader = match FixedStructReader::new(
        path.clone(),
        FileType::FixedStruct{ type_: fixedstructfiletype },
        blocksz,
        tzo,
        dt_filter,
        None,
    ) {
        ResultFixedStructReaderNewError::FileOk(val) => {
            eprintln!("new_result_opt: {:?}", new_result_opt);
            assert!(matches!(new_result_opt, None));
            val
        },
        result => {
            match new_result_opt {
                Some(_expect_result) => {
                    assert!(matches!(result, _expect_result));
                    return;
                }
                None => {
                    panic!(
                        "ERROR: FixedStructReader::new({:?}, {:?}, {:?}, {:?}) failed: {:?}, expected FileOk",
                        path, blocksz, tzo, dt_filter, result,
                    );
                }
            }
        }
    };

    let expect = match expect_opt {
        Some(expect) => expect,
        // do not call `process_entry_at`
        None => return,
    };

    let result: ResultS3FixedStructFind =
        fixedstructreader.process_entry_at(
            fo,
            &mut buffer,
        );
    match result {
        ResultS3FixedStructFind::Found((fo_, _utmpx)) => {
            match expect {
                FOUND => {
                    assert_eq!(
                        fo_ , expect_fo,
                        "expected fileoffset {}, got {}",
                        expect_fo, fo_,
                    );
                }
                DONE => {
                    panic!("expected DONE");
                }
            }
        }
        ResultS3FixedStructFind::Done => {
            match expect {
                FOUND => {
                    panic!("expected FOUND");
                }
                DONE => {}
            }
        }
        ResultS3FixedStructFind::Err(err) => {
            panic!("Error {:?}", err);
        }
    }
}

// short alias
const TYU: FixedStructType = FixedStructType::Fs_Linux_x86_Utmpx;

// short alias
const FTU: FixedStructFileType = FixedStructFileType::Utmpx;

/// helper to `test_FixedStructReader_summary`
const fn SummaryBlockReader_new(dropped_blocks_ok: Count, dropped_blocks_err: Count)
    -> SummaryBlockReader {
    SummaryBlockReader {
        blockreader_bytes: 0,
        blockreader_bytes_total: 0,
        blockreader_blocks: 0,
        blockreader_blocks_total: 0,
        blockreader_blocksz: 0,
        blockreader_filesz: 0,
        blockreader_filesz_actual: 0,
        blockreader_read_block_lru_cache_hit: 0,
        blockreader_read_block_lru_cache_miss: 0,
        blockreader_read_block_lru_cache_put: 0,
        blockreader_read_blocks_hit: 0,
        blockreader_read_blocks_miss: 0,
        blockreader_read_blocks_put: 0,
        blockreader_read_blocks_reread_error: 0,
        blockreader_blocks_highest: 0,
        blockreader_blocks_dropped_ok: dropped_blocks_ok,
        blockreader_blocks_dropped_err: dropped_blocks_err,
    }
}

// UTMPX_2ENTRY
#[test_case(
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH.clone(), 2,
    SummaryBlockReader_new(384, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 2,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER2_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 2,
    };
    "2ENTRY 2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH.clone(), U1SZ - 2,
    SummaryBlockReader_new(3, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 2,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER2_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 2,
    };
    "2ENTRY Um1"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH.clone(), U1SZ,
    SummaryBlockReader_new(2, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 2,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER2_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 2,
    };
    "2ENTRY U*1"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH.clone(), U1SZ + 2,
    SummaryBlockReader_new(2, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 2,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER2_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 2,
    };
    "2ENTRY Up2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH.clone(), U1SZ * 2,
    SummaryBlockReader_new(1, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 2,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER2_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 2,
    };
    "2ENTRY U*2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH.clone(), U1SZ * 3,
    SummaryBlockReader_new(1, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 2,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER2_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 2,
    };
    "2ENTRY U*3"
)]
// UTMPX_3ENTRY
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH.clone(), 2,
    SummaryBlockReader_new(576, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY 2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH.clone(), U1SZ - 2,
    SummaryBlockReader_new(4, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY Um2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH.clone(), U1SZ,
    SummaryBlockReader_new(3, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY U*1"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH.clone(), U1SZ + 2,
    SummaryBlockReader_new(3, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY Up2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH.clone(), U1SZ * 2,
    SummaryBlockReader_new(2, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY U*2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_FPATH.clone(), U1SZ * 3,
    SummaryBlockReader_new(1, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 144,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY U*3"
)]
// LINUX_X86_UTMPX_3ENTRY_OOO
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH.clone(), 2,
    SummaryBlockReader_new(576, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 140,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 2,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY_OOO 2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH.clone(), U1SZ - 2,
    SummaryBlockReader_new(4, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 140,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 2,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY_OOO Um2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH.clone(), U1SZ,
    SummaryBlockReader_new(3, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 140,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 2,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY_OOO U*1"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH.clone(), U1SZ + 2,
    SummaryBlockReader_new(3, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 140,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 2,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 3,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY_OOO Up2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH.clone(), U1SZ * 2,
    SummaryBlockReader_new(2, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 140,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 2,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY_OOO U*2"
)]
#[test_case(
    NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH.clone(), U1SZ * 3,
    SummaryBlockReader_new(1, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(TYU),
        fixedstructreader_fixedstructfiletype_opt: Some(FTU),
        fixedstructreader_fixedstruct_size: U1SZ as usize,
        fixedstructreader_high_score: 140,
        fixedstructreader_utmp_entries: 3,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 2,
        fixedstructreader_utmp_entries_max: 2,
        fixedstructreader_utmp_entries_hit: 2,
        fixedstructreader_utmp_entries_miss: 1,
        fixedstructreader_drop_entry_ok: 2,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_UTMPX_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_UTMPX_BUFFER3_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 3,
    };
    "3ENTRY_OOO U*3"
)]
// LINUX_X86_LASTLOG_1ENTRY
#[test_case(
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH.clone(), 2,
    SummaryBlockReader_new(146, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(FixedStructType::Fs_Linux_x86_Lastlog),
        fixedstructreader_fixedstructfiletype_opt: Some(FixedStructFileType::Lastlog),
        fixedstructreader_fixedstruct_size: L1SZ as usize,
        fixedstructreader_high_score: 60,
        fixedstructreader_utmp_entries: 1,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 1,
        fixedstructreader_utmp_entries_hit: 1,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 1,
    };
    "LINUX_X86_LASTLOG_1ENTRY 2"
)]
#[test_case(
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH.clone(), L1SZ - 2,
    SummaryBlockReader_new(2, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(FixedStructType::Fs_Linux_x86_Lastlog),
        fixedstructreader_fixedstructfiletype_opt: Some(FixedStructFileType::Lastlog),
        fixedstructreader_fixedstruct_size: L1SZ as usize,
        fixedstructreader_high_score: 60,
        fixedstructreader_utmp_entries: 1,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 1,
        fixedstructreader_utmp_entries_hit: 1,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 1,
    };
    "LINUX_X86_LASTLOG_1ENTRY L1SZm2"
)]
#[test_case(
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH.clone(), L1SZ,
    SummaryBlockReader_new(1, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(FixedStructType::Fs_Linux_x86_Lastlog),
        fixedstructreader_fixedstructfiletype_opt: Some(FixedStructFileType::Lastlog),
        fixedstructreader_fixedstruct_size: L1SZ as usize,
        fixedstructreader_high_score: 60,
        fixedstructreader_utmp_entries: 1,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 1,
        fixedstructreader_utmp_entries_hit: 1,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 1,
    };
    "LINUX_X86_LASTLOG_1ENTRY L1SZ"
)]
#[test_case(
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH.clone(), L1SZ + 2,
    SummaryBlockReader_new(1, 0),
    SummaryFixedStructReader {
        fixedstructreader_fixedstructtype_opt: Some(FixedStructType::Fs_Linux_x86_Lastlog),
        fixedstructreader_fixedstructfiletype_opt: Some(FixedStructFileType::Lastlog),
        fixedstructreader_fixedstruct_size: L1SZ as usize,
        fixedstructreader_high_score: 60,
        fixedstructreader_utmp_entries: 1,
        fixedstructreader_first_entry_fileoffset: 0,
        fixedstructreader_entries_out_of_order: 0,
        fixedstructreader_utmp_entries_max: 1,
        fixedstructreader_utmp_entries_hit: 1,
        fixedstructreader_utmp_entries_miss: 0,
        fixedstructreader_drop_entry_ok: 1,
        fixedstructreader_drop_entry_errors: 0,
        fixedstructreader_datetime_first: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_datetime_last: Some(*LINUX_X86_LASTLOG_BUFFER1_DTO),
        fixedstructreader_map_tvpair_fo_max_len: 1,
    };
    "LINUX_X86_LASTLOG_1ENTRY L1SZp2"
)]
fn test_FixedStructReader_summary(
    path: FPath,
    blocksz: BlockSz,
    expect_summaryblockreader: SummaryBlockReader,
    expect_fixedstructreadersummary: SummaryFixedStructReader,
) {
    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
    let mut fixedstructreader = new_FixedStructReader(&path, blocksz, *FO_P8);

    let mut fo: FileOffset = match fixedstructreader.fileoffset_first() {
        Some(fo) => fo,
        None => {
            panic!("fileoffset_first failed");
        }
    };
    let mut _fo_last: FileOffset;

    // find all the entries
    loop {
        let result: ResultS3FixedStructFind = fixedstructreader.process_entry_at(
            fo, &mut buffer,
        );
        match result {
            ResultS3FixedStructFind::Found((fo_, _utmpx)) => {
                _fo_last = fo;
                fo = fo_;
            }
            ResultS3FixedStructFind::Done => {
                break;
            }
            ResultS3FixedStructFind::Err(err) => {
                panic!("Error {:?}", err);
            }
        }
    }

    // get the summaries
    let summary = fixedstructreader.summary_complete();
    let (summaryfixedstructreader, summaryblockreader) = match summary.readerdata {
        SummaryReaderData::FixedStruct((
            summaryblockreader,
            summaryfixedstructreader,
        )) => {
            (summaryfixedstructreader, summaryblockreader)
        }
        _ => panic!(),
    };
    eprintln!("\nsummaryblockreader:\n{:?}\n", summaryblockreader);
    eprintln!("\nsummaryfixedstructreader:\n{:?}\n", summaryfixedstructreader);

    // compare summaryblockreader
    assert_eq!(
        summaryblockreader.blockreader_blocks_dropped_ok,
        expect_summaryblockreader.blockreader_blocks_dropped_ok,
        "blockreader_blocks_dropped_ok differs",
    );
    assert_eq!(
        summaryblockreader.blockreader_blocks_dropped_err,
        expect_summaryblockreader.blockreader_blocks_dropped_err,
        "blockreader_blocks_dropped_err differs",
    );
    // compare summaryfixedstructreader
    let _ft = summaryfixedstructreader.fixedstructreader_fixedstructtype_opt;
    assert!(
        matches!(
            expect_fixedstructreadersummary.fixedstructreader_fixedstructtype_opt,
            _ft,
        ),
        "fixedstructreader_fixedstructtype_opt differs",
    );
    let _ft = summaryfixedstructreader.fixedstructreader_fixedstructfiletype_opt;
    assert!(
        matches!(
            expect_fixedstructreadersummary.fixedstructreader_fixedstructfiletype_opt,
            _ft,
        ),
        "fixedstructreader_fixedstructfiletype_opt differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_fixedstruct_size,
        expect_fixedstructreadersummary.fixedstructreader_fixedstruct_size,
        "fixedstructreader_fixedstruct_size differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_high_score,
        expect_fixedstructreadersummary.fixedstructreader_high_score,
        "fixedstructreader_high_score differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_utmp_entries,
        expect_fixedstructreadersummary.fixedstructreader_utmp_entries,
        "fixedstructreader_utmp_entries differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_first_entry_fileoffset,
        expect_fixedstructreadersummary.fixedstructreader_first_entry_fileoffset,
        "fixedstructreader_first_entry_fileoffset differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_entries_out_of_order,
        expect_fixedstructreadersummary.fixedstructreader_entries_out_of_order,
        "fixedstructreader_entries_out_of_order differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_utmp_entries_max,
        expect_fixedstructreadersummary.fixedstructreader_utmp_entries_max,
        "fixedstructreader_utmp_entries_max differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_utmp_entries_hit,
        expect_fixedstructreadersummary.fixedstructreader_utmp_entries_hit,
        "fixedstructreader_utmp_entries_hit differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_utmp_entries_miss,
        expect_fixedstructreadersummary.fixedstructreader_utmp_entries_miss,
        "fixedstructreader_utmp_entries_miss differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_drop_entry_ok,
        expect_fixedstructreadersummary.fixedstructreader_drop_entry_ok,
        "fixedstructreader_drop_entry_ok differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_drop_entry_errors,
        expect_fixedstructreadersummary.fixedstructreader_drop_entry_errors,
        "fixedstructreader_drop_entry_errors differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_datetime_first,
        expect_fixedstructreadersummary.fixedstructreader_datetime_first,
        "fixedstructreader_datetime_first differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_datetime_last,
        expect_fixedstructreadersummary.fixedstructreader_datetime_last,
        "fixedstructreader_datetime_last differs",
    );
    assert_eq!(
        summaryfixedstructreader.fixedstructreader_map_tvpair_fo_max_len,
        expect_fixedstructreadersummary.fixedstructreader_map_tvpair_fo_max_len,
        "fixedstructreader_map_tvpair_fo_max_len differs",
    );

    // test for no duplicate dropped blocks
    // duplicated dropped blocks means the dropping algorithm is too aggressive
    // and causing blocks to be retrieved more than once
    let mut set: HashSet<BlockOffset> = HashSet::new();
    for bo in fixedstructreader.dropped_blocks.iter() {
        defo!("dropped block at BlockOffset {:?}", bo);
        set.insert(*bo);
    }
    assert_eq!(
        set.len(), fixedstructreader.dropped_blocks.len(),
        "duplicate entries in fixedstructreader.dropped_blocks; a block was dropped, then retrieved, then dropped again"
    );
}

const FSF_U: FixedStructFileType = FixedStructFileType::Utmpx;

// NTF_LINUX_X86_UTMPX_1ENTRY_FPATH
#[test_case(&*NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, FSF_U, 0, -5, -4, None, 0, NEWNODT; "a UTMPX_1ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, FSF_U, 0, 0, 2, Some(FOUND), 0, None; "b UTMPX_1ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, FSF_U, U1SZ, 0, 2, Some(DONE), 0, None; "c UTMPX_1ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, FSF_U, 0, 8, 9, None, 0, NEWNODT; "d UTMPX_1ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_1ENTRY_FPATH, FSF_U, U1SZ, 8, 9, None, 0, NEWNODT; "e UTMPX_1ENTRY")]
// NTF_LINUX_X86_UTMPX_2ENTRY_FPATH
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, -5, -7, None, 0, NEWNODT; "a UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, 1, 2, None, 0, NEWNODT; "b UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, -1, 3, Some(FOUND), 0, None; "c UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, 1, 3, Some(FOUND), 0, None; "d UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, 1, 2, None, 0, NEWNODT; "e UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, 0, 2, Some(FOUND), 0, None; "f UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, -1, 1, Some(FOUND), 0, None; "g UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, 0, 0, 3, Some(FOUND), 0, None; "h UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, U1SZ, 0, 3, Some(FOUND), U1SZ, None; "i UTMPX_2ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, FSF_U, U1SZ, 8, 9, None, U1SZ, NEWNODT; "j UTMPX_2ENTRY")]
// NTF_LINUX_X86_UTMPX_3ENTRY_FPATH
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, -5, -4, None, 0, NEWNODT; "a UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, 0, 3, Some(FOUND), 0, None; "b UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, 1, 3, Some(FOUND), 0, None; "c UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, 2, 3, Some(FOUND), 0, None; "d UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, 3, 3, None, 0, NEWNODT; "e UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, 4, 6, Some(FOUND), 0, None; "f UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, 0, 5, 6, None, 0, NEWNODT; "g UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ, 0, 3, Some(FOUND), U1SZ, None; "h UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ, 0, 3, Some(FOUND), U1SZ, None; "i UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ, 2, 4, Some(FOUND), U1SZ, None; "j UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ, 2, 3, Some(FOUND), U1SZ, None; "k UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ, 2, 2, None, 0, NEWNODT; "l UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ, 4, 5, Some(FOUND), U1SZ, None; "m UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ * 2, 2, 3, Some(FOUND), U1SZ * 2, None; "n UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ * 2, 3, 4, None, 0, NEWNODT; "o UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ * 2, 4, 5, Some(FOUND), U1SZ * 2, None; "p UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ * 2, 5, 5, None, 0, NEWNODT; "q UTMPX_3ENTRY")]
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_FPATH, FSF_U, U1SZ * 3, 4, 5, Some(DONE), 0, None; "r UTMPX_3ENTRY")]
// NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH
#[test_case(&*NTF_LINUX_X86_UTMPX_3ENTRY_OOO_FPATH, FSF_U, U1SZ * 3, 4, 5, Some(DONE), 0, None; "a LINUX_X86_UTMPX_3ENTRY_OOO")]
fn test_FixedStructReader_process_entry_at_between_datetime_filters(
    path: &FPath, // pass to `FixedStructReader::new`
    fixedstructfiletype: FixedStructFileType, // pass to `FixedStructReader::new`
    fo: FileOffset, // call `process_entry_at` with this file offset
    diff_a: i64, // add this to `LINUX_X86_UTMPX_BUFFER1_DT` to get `dt_filter_a`
    diff_b: i64, // add this to `LINUX_X86_UTMPX_BUFFER1_DT` to get `dt_filter_b`
    expect_opt: Option<ResultS3FixedStructFind_Test>, // expected result of `process_entry_at`
    expect_fo: FileOffset, // expected file offset of `process_entry_at`
    new_result_opt: Option<ResultFixedStructReaderNewError>, // expected result of `FixedStructReader::new`
) {
    defn!(
        "fo {}, diff_a {}, diff_b {}, expect_opt {:?}, expect_fo {}, new_result_opt {:?}",
        fo, diff_a, diff_b, expect_opt, expect_fo, new_result_opt,
    );

    match &expect_opt {
        Some(res) => {
            assert!(new_result_opt.is_none(), "bad test inputs; given expected result for process_entry_at but also given new_result_opt != None");
            match *res {
                FOUND => {}
                DONE => {
                    assert_eq!(expect_fo, 0, "bad test inputs; given DONE but also given expect_fo != 0");
                }
            }
        }
        None => {
            assert!(new_result_opt.is_some(), "bad test inputs");
        }
    }

    let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];

    let dt_filter_a = Some(
        *LINUX_X86_UTMPX_BUFFER1_DT + Duration::seconds(diff_a)
    );
    let dt_filter_b = Some(
        *LINUX_X86_UTMPX_BUFFER1_DT + Duration::seconds(diff_b)
    );

    let mut fixedstructreader = 
        match FixedStructReader::new(
            path.clone(),
            FileType::FixedStruct{ type_: fixedstructfiletype },
            BSZ,
            *FO_0,
            dt_filter_a,
            dt_filter_b,
        ) {
            ResultFixedStructReaderNewError::FileOk(val) => val,
            result => {
                match new_result_opt {
                    Some(_expect_result) => {
                        defo!("FileStructReader::new() result was {:?}", result);
                        assert!(matches!(result, _expect_result));
                        defx!("FileStructReader::new() was expected value {:?}", _expect_result);
                        return;
                    }
                    None => {
                        panic!(
                            "ERROR: FixedStructReader::new({:?}, {:?}, {:?}, {:?}, {:?}) failed: {:?}, expected FileOk",
                            path, BSZ, *FO_0, dt_filter_a, dt_filter_b, result,
                        );
                    }
                };
            }
    };

    let result: ResultS3FixedStructFind =
        fixedstructreader.process_entry_at(
            fo,
            &mut buffer,
        );
    let expect = expect_opt.unwrap();

    let fs_: &FixedStruct;
    match &result {
        ResultS3FixedStructFind::Found((_fo, fixedstruct)) => {
            match expect {
                FOUND => {
                    assert_eq!(
                        fixedstruct.fileoffset, expect_fo,
                        "expected FixedStruct with offset {}, got FixedStruct with offset {}",
                        expect_fo, fixedstruct.fileoffset,
                    );
                }
                DONE => {
                    panic!("expected DONE, got ResultS3FixedStructFind::Found");
                }
            }
            fs_ = fixedstruct;
        }
        ResultS3FixedStructFind::Done => {
            match expect {
                FOUND => {
                    panic!("expected FOUND, got ResultS3FixedStructFind::Done");
                }
                DONE => {
                    defx!("process_entry_at({}, ...) returned Done", fo);
                    return;
                }
            }
        }
        ResultS3FixedStructFind::Err(err) => {
            panic!("Error {:?}", err);
        }
    }
    defx!(
        "process_entry_at({}, ...) returned {:?}\ndt_filter_a {:?}\nfixedstruct {:?}\ndt_filter_b {:?}",
        fo, result, dt_filter_a, Some(fs_.dt()), dt_filter_b,
    )
}
