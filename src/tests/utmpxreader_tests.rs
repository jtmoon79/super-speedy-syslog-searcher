// src/tests/utmpreader_tests.rs

//! tests for `utmpreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{Count, FPath, FileOffset};
use crate::data::datetime::FixedOffset;
use crate::data::utmpx::{
    UTMPX_SZ,
    UTMPX_SZ_FO,
};
use crate::readers::blockreader::BlockSz;
use crate::readers::summary::SummaryReaderData;
use crate::readers::utmpxreader::{
    ResultS3UtmpxFind,
    UtmpxReader,
};
use crate::tests::common::{
    FO_0,
    FO_P8,
    UTMPX_2ENTRY_FILESZ,
    NTF_LOG_EMPTY_FPATH,
    NTF_UTMPX_1ENTRY_FPATH,
    NTF_UTMPX_2ENTRY_FPATH,
    NTF_UTMPX_3ENTRY_FPATH,
    UTMPX_ENTRY_DT1,
    UTMPX_ENTRY_DT2,
};

use ::chrono::Duration;
#[allow(unused_imports)]
use ::more_asserts::{assert_gt, assert_ge};
use ::test_case::test_case;
#[allow(unused_imports)]
use ::si_trace_print::printers::{defn, defo, defx, defñ};
use ::si_trace_print::stack::stack_offset_set;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const UTMPX_SZ_BSZ: BlockSz = UTMPX_SZ as BlockSz;

/// helper to wrap the match and panic checks
fn new_UtmpxReader(
    path: &FPath,
    blocksz: BlockSz,
    tzo: FixedOffset,
) -> UtmpxReader {
    stack_offset_set(Some(2));
    match UtmpxReader::new(path.clone(), blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

#[test]
fn test_new_UtmpxReader_1() {
    new_UtmpxReader(&NTF_LOG_EMPTY_FPATH, 1024, *FO_P8);
}

#[test]
#[should_panic]
fn test_new_UtmpxReader_2_bad_path_panics() {
    new_UtmpxReader(&FPath::from("THIS/PATH_DOES/NOT///EXIST!!!"), 1024, *FO_P8);
}

#[test]
fn test_UtmpxReader_helpers0() {
    const BSZ: BlockSz = 64;
    let ur = new_UtmpxReader(
        &NTF_UTMPX_2ENTRY_FPATH,
        BSZ,
        *FO_P8
    );

    assert_eq!(ur.block_index_at_file_offset(0), 0);
    assert_eq!(ur.block_offset_at_file_offset(0), 0);
    assert_eq!(ur.blockoffset_last(), UTMPX_2ENTRY_FILESZ / BSZ - 1);
    assert_eq!(ur.blocksz(), BSZ);
    assert_eq!(ur.count_blocks(), UTMPX_2ENTRY_FILESZ / BSZ);
    assert_eq!(ur.count_entries_processed(), 0);
    assert_eq!(ur.file_offset_at_block_offset(0), 0);
    assert_eq!(ur.file_offset_at_block_offset_index(0, 1), 1);
    assert_eq!(ur.fileoffset_last() + 1, UTMPX_2ENTRY_FILESZ as FileOffset);
    assert_eq!(ur.fileoffset_to_utmpoffset(0), 0);
    assert_eq!(ur.filesz(), UTMPX_2ENTRY_FILESZ);
    assert_eq!(ur.utmpsize(), UTMPX_SZ);
    ur.get_fileoffsets();
}

const BSZ: BlockSz = 400;

#[test]
fn test_UtmpxReader_summary0() {
    const BSZ: BlockSz = 64;
    let ur = new_UtmpxReader(
        &NTF_UTMPX_2ENTRY_FPATH,
        BSZ,
        *FO_P8
    );
    let _summaryutmpxreader = ur.summary();
    let _summary = ur.summary_complete();
}

#[test]
fn test_UtmpxReader_read_find_entry_2() {
    let mut utmpreader = new_UtmpxReader(
        &NTF_UTMPX_2ENTRY_FPATH,
        BSZ,
        *FO_P8
    );

    let mut fo: FileOffset = 0;
    loop {
        let result: ResultS3UtmpxFind = utmpreader.find_entry(fo);
        match result {
            ResultS3UtmpxFind::Found((fo_, _utmpentry)) => {
                fo = fo_;
            }
            ResultS3UtmpxFind::Done => {
                break;
            }
            ResultS3UtmpxFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
    }
}

#[test]
fn test_UtmpxReader_read_find_entry_2_summary() {
    let mut utmpreader = new_UtmpxReader(
        &NTF_UTMPX_2ENTRY_FPATH,
        BSZ,
        *FO_P8
    );

    let mut fo: FileOffset = 0;
    loop {
        let result: ResultS3UtmpxFind = utmpreader.find_entry(fo);
        match result {
            ResultS3UtmpxFind::Found((fo_, _utmpentry)) => {
                fo = fo_;
            }
            ResultS3UtmpxFind::Done => {
                break;
            }
            ResultS3UtmpxFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
    }
    // do one extra redundant search to make a little more
    // interesting
    match utmpreader.find_entry(0) {
        ResultS3UtmpxFind::Found(_) => {},
        _ => panic!(),
    }

    let summaryutmpxreader = utmpreader.summary();
    assert_eq!(summaryutmpxreader.utmpxreader_utmp_entries, 2);
    assert_eq!(summaryutmpxreader.utmpxreader_utmp_entries_max, 2);
    assert_eq!(summaryutmpxreader.utmpxreader_utmp_entries_hit, 1);
    assert_eq!(summaryutmpxreader.utmpxreader_utmp_entries_miss, 2);
    assert_eq!(
        &summaryutmpxreader.utmpxreader_datetime_first.unwrap(),
        &*UTMPX_ENTRY_DT1,
    );
    assert_eq!(
        &summaryutmpxreader.utmpxreader_datetime_last.unwrap(),
        &*UTMPX_ENTRY_DT2,
    );

    let summary = utmpreader.summary_complete();
    match summary.readerdata {
        SummaryReaderData::Utmpx((
            _summaryblockreader,
            summaryutmpxreader_,
        )) => {
            assert_eq!(summaryutmpxreader_, summaryutmpxreader)
        }
        _ => panic!(),
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ResultS3UtmpxFind_Test {
    Found,
    Done,
}

const FOUND: ResultS3UtmpxFind_Test = ResultS3UtmpxFind_Test::Found;
const DONE: ResultS3UtmpxFind_Test = ResultS3UtmpxFind_Test::Done;

#[test_case(&*NTF_UTMPX_1ENTRY_FPATH, 0, 0, FOUND, 1; "a 1ENTRY")]
#[test_case(&*NTF_UTMPX_1ENTRY_FPATH, 0, 1, DONE, 0; "b 1ENTRY")]
#[test_case(&*NTF_UTMPX_1ENTRY_FPATH, UTMPX_SZ_FO, 1, DONE, 0; "c 1ENTRY")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 0, FOUND, 1; "a 2ENTRY")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 1, FOUND, 2; "b 2ENTRY")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, UTMPX_SZ_FO, 1, FOUND, 2; "c 2ENTRY")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, UTMPX_SZ_FO, 4, DONE, 0; "d 2ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 0, FOUND, 1; "a 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 1, FOUND, 2; "b 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 1, FOUND, 2; "c 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 0, FOUND, 3; "d 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 1, FOUND, 3; "e 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 2, FOUND, 3; "f 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 4, FOUND, 3; "g 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 5, DONE, 0; "h 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 3, 1, DONE, 0; "i 3ENTRY")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 5000, DONE, 0; "j 3ENTRY")]
fn test_UtmpxReader_read_find_entry_at_datetime_filter(
    path: &FPath,
    fo: FileOffset,
    seconds: i64,
    expect: ResultS3UtmpxFind_Test,
    expect_fo_index: FileOffset,
) {
    let mut utmpreader = new_UtmpxReader(
        path,
        BSZ,
        *FO_0,
    );
    let dt_filter = Some(
        *UTMPX_ENTRY_DT1 + Duration::seconds(seconds),
    );

    let result: ResultS3UtmpxFind =
        utmpreader.find_entry_at_datetime_filter(
            fo,
            &dt_filter,
        );
    match result {
        ResultS3UtmpxFind::Found((fo_, _utmpentry)) => {
            match expect {
                FOUND => {
                    let fo_i = expect_fo_index * UTMPX_SZ_FO;
                    assert_eq!(fo_ , fo_i, "expected fileoffset ({} * {}) = {}, got {}",
                        expect_fo_index, UTMPX_SZ_FO, fo_i, fo_);
                }
                DONE => {
                    panic!("expected DONE");
                }
            }
        }
        ResultS3UtmpxFind::Done => {
            match expect {
                FOUND => {
                    panic!("expected FOUND");
                }
                DONE => {}
            }
        }
        ResultS3UtmpxFind::Err(err) => {
            panic!("Error {}", err);
        }
    }
}

#[test_case(NTF_UTMPX_2ENTRY_FPATH.clone(), 2, 1, 0, 193, 0; "2ENTRY 2")]
#[test_case(NTF_UTMPX_2ENTRY_FPATH.clone(), 64, 1, 0, 7, 0; "2ENTRY 64")]
#[test_case(NTF_UTMPX_2ENTRY_FPATH.clone(), UTMPX_SZ_BSZ, 1, 0, 2, 0; "2ENTRY  UTMPX_SZ_BSZ")]
#[test_case(NTF_UTMPX_2ENTRY_FPATH.clone(), UTMPX_SZ_BSZ * 2, 1, 0, 1, 0; "2ENTRY  UTMPX_SZ_BSZ*2")]
#[test_case(NTF_UTMPX_2ENTRY_FPATH.clone(), UTMPX_SZ_BSZ * 3, 0, 0, 0, 0; "2ENTRY  UTMPX_SZ_BSZ*3")]
#[test_case(NTF_UTMPX_3ENTRY_FPATH.clone(), 2, 2, 0, 385, 0; "3ENTRY 2")]
#[test_case(NTF_UTMPX_3ENTRY_FPATH.clone(), 64, 2, 0, 6, 0; "3ENTRY 64")]
#[test_case(NTF_UTMPX_3ENTRY_FPATH.clone(), UTMPX_SZ_BSZ, 2, 0, 3, 0; "3ENTRY UTMPX_SZ_BSZ")]
#[test_case(NTF_UTMPX_3ENTRY_FPATH.clone(), UTMPX_SZ_BSZ * 2, 3, 0, 2, 0; "3ENTRY UTMPX_SZ_BSZ*2")]
#[test_case(NTF_UTMPX_3ENTRY_FPATH.clone(), UTMPX_SZ_BSZ * 3, 2, 0, 1, 0; "3ENTRY UTMPX_SZ_BSZ*3")]
fn test_UtmpxReader_drops(
    path: FPath,
    blocksz: BlockSz,
    expect_drop_entry_ok: Count,
    expect_drop_entry_err: Count,
    expect_drop_block_ok: Count,
    expect_drop_block_err: Count,
) {
    let mut utmpreader = new_UtmpxReader(&path, blocksz, *FO_P8);

    let mut fo: FileOffset = 0;
    let mut fo_last: FileOffset;

    // find all the entries
    loop {
        let result: ResultS3UtmpxFind = utmpreader.find_entry(fo);
        match result {
            ResultS3UtmpxFind::Found((fo_, _utmpentry)) => {
                fo_last = fo;
                fo = fo_;
            }
            ResultS3UtmpxFind::Done => {
                break;
            }
            ResultS3UtmpxFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
        utmpreader.drop_entries(fo_last);
    }

    // get the summary
    let summary = utmpreader.summary_complete();
    let (summaryutmpxreader, summaryblockreader) = match summary.readerdata {
        SummaryReaderData::Utmpx((
            summaryblockreader,
            summaryutmpxreader,
        )) => {
            (summaryutmpxreader, summaryblockreader)
        }
        _ => panic!(),
    };

    assert_eq!(
        summaryutmpxreader.utmpxreader_drop_entry_ok,
        expect_drop_entry_ok,
        "drop_entry_ok differs",
    );
    assert_eq!(
        summaryutmpxreader.utmpxreader_drop_entry_errors,
        expect_drop_entry_err,
        "drop_entry_err differs",
    );
    assert_ge!(
        summaryblockreader.blockreader_blocks_dropped_ok,
        expect_drop_block_ok,
        "drop_block_ok differs",
    );
    assert_eq!(
        summaryblockreader.blockreader_blocks_dropped_err,
        expect_drop_block_err,
        "drop_block_err differs",
    );
}

#[test_case(&*NTF_UTMPX_1ENTRY_FPATH, 0, 0, 2, FOUND, 0; "a 1ENTRY 0_0_2_FOUND_0")]
#[test_case(&*NTF_UTMPX_1ENTRY_FPATH, 0, 0, 2, FOUND, 0; "b 1ENTRY 0_0_2_DONE")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 1, 2, DONE, 0; "c 2ENTRY 0_1_2_DONE")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, -1, 3, FOUND, 0; "d 2ENTRY 0_-1_3_FOUND_0")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 1, 3, FOUND, 1; "e 2ENTRY 0_1_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 1, 2, DONE, 0; "f 2ENTRY 0_1_2_DONE")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 0, 2, FOUND, 0; "g 2ENTRY 0_0_2_FOUND_0")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, -1, 1, FOUND, 0; "h 2ENTRY 0_-1_1_FOUND_0")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, 0, 0, 3, FOUND, 0; "i 2ENTRY 0_0_3_FOUND_0")]
#[test_case(&*NTF_UTMPX_2ENTRY_FPATH, UTMPX_SZ_FO, 0, 3, FOUND, 1; "j 2ENTRY SZ_0_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 0, 3, FOUND, 0; "k 3ENTRY 0_0_3_FOUND_0")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 1, 3, FOUND, 1; "l 3ENTRY 0_1_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 2, 3, FOUND, 1; "m 3ENTRY 0_2_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 3, 3, DONE, 0; "n 3ENTRY 0_3_3_DONE")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 4, 6, FOUND, 2; "o 3ENTRY 0_4_6_FOUND_2")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, 0, 5, 6, DONE, 0; "p 3ENTRY 0_5_6_DONE")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 0, 3, FOUND, 1; "q 3ENTRY SZ_0_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 0, 3, FOUND, 1; "r 3ENTRY SZ_0_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 2, 4, FOUND, 1; "s 3ENTRY SZ_2_4_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 2, 3, FOUND, 1; "t 3ENTRY SZ_2_3_FOUND_1")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 2, 2, DONE, 0; "u 3ENTRY SZ_2_2_DONE")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO, 4, 5, FOUND, 2; "v 3ENTRY SZ_4_5_FOUND_2")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 4, 5, FOUND, 2; "w 3ENTRY SZ2_4_5_FOUND_2")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 2, 5, 5, DONE, 0; "x 3ENTRY SZ2_5_5_DONE")]
#[test_case(&*NTF_UTMPX_3ENTRY_FPATH, UTMPX_SZ_FO * 3, 4, 5, DONE, 0; "y 3ENTRY SZ3_4_5_DONE")]
fn test_UtmpxReader_find_entry_between_datetime_filters(
    path: &FPath,
    fo: FileOffset,
    diff_a: i64,
    diff_b: i64,
    expect: ResultS3UtmpxFind_Test,
    expect_index: FileOffset,
) {
    defn!("fo {}, diff_a {}, diff_b {}, expect {:?}, expect_index {}",
        fo, diff_a, diff_b, expect, expect_index,
    );
    let mut utmpreader = new_UtmpxReader(
        path,
        BSZ,
        *FO_0,
    );
    let dt_filter_a = Some(
        *UTMPX_ENTRY_DT1 + Duration::seconds(diff_a)
    );
    let dt_filter_b = Some(
        *UTMPX_ENTRY_DT1 + Duration::seconds(diff_b)
    );

    let result: ResultS3UtmpxFind =
        utmpreader.find_entry_between_datetime_filters(
            fo,
            &dt_filter_a,
            &dt_filter_b,
        );

    match result {
        ResultS3UtmpxFind::Found((_fo, utmpentry)) => {
            match expect {
                FOUND => {
                    let fo_exp: FileOffset = UTMPX_SZ_FO * expect_index;
                    assert_eq!(utmpentry.fileoffset, fo_exp,
                        "expected Utmpx with offset {}, got entry with offset {}",
                        fo_exp,
                        utmpentry.fileoffset,
                    );
                }
                DONE => {
                    panic!("expected DONE");
                }
            }
        }
        ResultS3UtmpxFind::Done => {
            match expect {
                FOUND => {
                    panic!("expected FOUND");
                }
                DONE => {}
            }
        }
        ResultS3UtmpxFind::Err(err) => {
            panic!("Error {}", err);
        }
    }
}
