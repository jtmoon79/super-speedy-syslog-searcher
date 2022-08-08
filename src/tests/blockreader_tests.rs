// src/tests/blockreader_tests.rs
//

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{
    Bytes,
    FileType,
    ResultS3,
};

use crate::readers::filepreprocessor::{
    fpath_to_filetype_mimeguess,
};

use crate::readers::blockreader::{
    FPath,
    FileOffset,
    BlockSz,
    BlockOffset,
    BlockReader,
    ResultS3ReadBlock,
};

#[allow(unused_imports)]
use crate::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_with_name_exact,
    create_temp_file_with_suffix,
    create_temp_file_bytes_with_suffix,
    ntf_fpath,
};

use crate::printer_debug::stack::{
    stack_offset_set,
};

use crate::printer_debug::printers::{
    dpnf,
    dpof,
    dpxf,
    dpnxf,
};

#[allow(unused_imports)]
use crate::tests::common::{
    BYTES_A,
    BYTES_AB,
    BYTES_CD,
    BYTES_C,
    BYTES_ABCD,
    BYTES_EFGH,
    BYTES_ABCDEFGH,
    NTF_LOG_EMPTY_FPATH,
    NTF_1BYTE_PATH,
    NTF_3BYTE_PATH,
    NTF_8BYTE_PATH,
    NTF_GZ_EMPTY_FPATH,
    NTF_GZ_1BYTE_FPATH,
    NTF_GZ_8BYTE_FPATH,
    NTF_XZ_EMPTY_FPATH,
    NTF_XZ_1BYTE_FPATH,
    NTF_XZ_8BYTE_FPATH,
    NTF_TAR_0BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FPATH,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_8BYTE_FILEA_FPATH,
    NTF_TAR_3BYTE_OLDGNU_FILEA_FPATH,
    NTF_TAR_3BYTE_PAX_FILEA_FPATH,
    NTF_TAR_3BYTE_USTAR_FILEA_FPATH,
    FILE,
    FILE_GZ,
    FILE_TAR,
    FILE_XZ,
};

use std::collections::BTreeMap;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate test_case;
use test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper wrapper to create a new BlockReader
#[allow(dead_code)]
fn new_BlockReader(path: FPath, blocksz: BlockSz) -> BlockReader {
    stack_offset_set(Some(2));
    let (filetype, _mimeguess) = fpath_to_filetype_mimeguess(&path);
    match BlockReader::new(path.clone(), filetype, blocksz) {
        Ok(br) => {
            eprintln!("opened {:?}", path);
            eprintln!("new {:?}", &br);
            br
        }
        Err(err) => {
            panic!("ERROR: BlockReader.open({:?}, {}) {}", path, blocksz, err);
        }
    }
}

/// helper wrapper to create a new BlockReader
fn new_BlockReader2(path: FPath, filetype: FileType, blocksz: BlockSz) -> BlockReader {
    stack_offset_set(Some(2));
    match BlockReader::new(path.clone(), filetype, blocksz) {
        Ok(br) => {
            eprintln!("opened {:?}", path);
            eprintln!("new {:?}", &br);
            br
        }
        Err(err) => {
            panic!("ERROR: BlockReader.open({:?}, {}) {}", path, blocksz, err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

type ResultS3_Check = ResultS3<(), ()>;
type Checks = BTreeMap::<BlockOffset, (Vec<u8>, ResultS3_Check)>;

const FOUND: ResultS3_Check = ResultS3_Check::Found(());
const DONE: ResultS3_Check = ResultS3_Check::Done;

/// test of basic test of BlockReader things
#[allow(non_snake_case)]
fn test_BlockReader(path: &FPath, filetype: FileType, blocksz: BlockSz, offsets: &Vec<BlockOffset>, checks: &Checks) {
    dpnf!("({:?}, {})", path, blocksz);
    let mut br1 = new_BlockReader2(path.clone(), filetype, blocksz);

    for offset in offsets.iter() {
        {
            let blockp = br1.read_block(*offset);
            match blockp {
                ResultS3ReadBlock::Found(_val) => {
                    let _boff: FileOffset = BlockReader::file_offset_at_block_offset(*offset, blocksz);
                }
                ResultS3ReadBlock::Done => {
                    continue;
                }
                ResultS3ReadBlock::Err(err) => {
                    panic!("ERROR: blockreader.read({}) error {}", offset, err);
                }
            };
        }
    }

    for (offset, (block_expect, results3)) in checks.iter() {
        // get the block data before calling `read_block`
        dpof!("get_block({})", offset);
        let block_actual_opt = br1.get_block(offset);
        match br1.read_block(*offset) {
            ResultS3ReadBlock::Found(_) => {
                assert!(results3.is_found(), "Got ResultS3::Found, Expected {:?}", results3);
            }
            ResultS3ReadBlock::Done => {
                assert!(results3.is_done(), "Got ResultS3::Done, Expected {:?}", results3);
                continue;
            }
            ResultS3ReadBlock::Err(err) => {
                eprintln!("ERROR: blockreader.read({}) error {}", offset, err);
                assert!(results3.is_err(), "Got ResultS3::Err, Expected {:?}", results3);
                continue;
            }
        }
        let block_actual: Bytes = block_actual_opt.unwrap();
        let block_expect_str = String::from_utf8_lossy(block_expect);
        let block_actual_str = String::from_utf8_lossy(&block_actual);
        assert_eq!(
            block_expect, &block_actual,
            "\nblocks at blockoffset {} do not match\nExpected {:?}\nActual   {:?}",
            offset, block_expect_str, block_actual_str,
        );
    }

    dpxf!();
}

// TODO: [2022/08/05] tests for bad files, unparseable files, multiple streams, etc.

// -------------------------------------------------------------------------------------------------

// reading plain file tests

lazy_static! {
    static ref NTF_BASIC_10: NamedTempFile = {
        create_temp_file("\
1901-01-01 00:01:01 1
1902-01-02 00:02:02 2
1903-01-03 00:03:03 3
1904-01-04 00:04:04 4
1905-01-05 00:05:05 5
1906-01-06 00:10:06 6
1907-01-07 00:11:07 7
1908-01-08 00:12:08 8
1909-01-09 00:13:09 9
1910-01-10 00:14:10 10"
        )
    };
    static ref NTF_BASIC_10_FPATH: FPath = ntf_fpath(&NTF_BASIC_10);
}

#[test]
fn  test_new_read_block_basic10_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_basic10_0_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_basic10_0_1_2() {
    let offsets: Vec<BlockOffset> = vec![0, 1, 2];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    checks.insert(2, (vec![b'-', b'0'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_basic10_2_1_0() {
    let offsets: Vec<BlockOffset> = vec![2, 1, 0];
    let mut checks = Checks::new();
    checks.insert(2, (vec![b'-', b'0'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_basic10_0_1_2_3_1_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1, 2, 3];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    checks.insert(2, (vec![b'-', b'0'], FOUND));
    checks.insert(3, (vec![b'1', b'-'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_basic10_33_34_32() {
    let offsets: Vec<BlockOffset> = vec![33, 34, 32];
    let mut checks = Checks::new();
    checks.insert(33, (vec![b'1', b'9'], FOUND));
    checks.insert(34, (vec![b'0', b'4'], FOUND));
    checks.insert(32, (vec![b'3', b'\n'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_basic10_34_Done_34() {
    let offsets: Vec<BlockOffset> = vec![34];
    let mut checks = Checks::new();
    checks.insert(34, (vec![b'0', b'4'], FOUND));
    checks.insert(99999, (vec![], DONE));
    checks.insert(34, (vec![b'0', b'4'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

// reading gz file tests

#[test]
fn  test_new_read_block_gz_0bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![], DONE));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_EMPTY_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_1bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'A'], FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_1BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_0_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_0_1_0() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_1_0() {
    let offsets: Vec<BlockOffset> = vec![1, 0];
    let mut checks = Checks::new();
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_0_1_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 4, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_0_1_Done_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    checks.insert(2, (vec![], DONE));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 4, &offsets, &checks);
}

#[test]
fn  test_new_read_block_gz_8bytes_0_bsz16() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(1, (vec![], DONE));
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::FileGz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 16, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

// reading xz file tests

#[test]
fn  test_new_read_block_xz_0bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![], DONE));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_EMPTY_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_xz_1bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'A'], FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_1BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_xz_8bytes_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_xz_8bytes_0_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_xz_8bytes_0_1_0() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_xz_8bytes_1_0() {
    let offsets: Vec<BlockOffset> = vec![1, 0];
    let mut checks = Checks::new();
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_xz_8bytes_0_1_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 4, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_0_1_Done_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    checks.insert(2, (vec![], DONE));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 4, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_0_bsz16() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(1, (vec![], DONE));
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::FileXz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 16, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

// reading tar file tests

#[test]
fn  test_new_read_block_tar_0byte_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![], DONE));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_0BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_1byte_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'A'], FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_1BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_3byte_oldgnu_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_3BYTE_OLDGNU_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_3byte_oldgnu_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_C.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_3BYTE_OLDGNU_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_3byte_pax_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_3BYTE_PAX_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_3byte_pax_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_C.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_3BYTE_PAX_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_3byte_ustar_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_3BYTE_USTAR_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_3byte_ustar_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_C.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_3BYTE_USTAR_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_8byte_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_8byte_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_8byte_0_3() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(3, (vec![b'G', b'H'], FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_8byte_1() {
    let offsets: Vec<BlockOffset> = vec![1];
    let mut checks = Checks::new();
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn  test_new_read_block_tar_8byte_99() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(99, (vec![], DONE));
    let ft = FileType::FileTar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

#[test_case(NTF_LOG_EMPTY_FPATH.clone(), FILE, 2, 0, 0; "LOG_0BYTE 2 0 0")]
#[test_case(NTF_1BYTE_PATH.clone(), FILE, 2, 0, 1; "LOG_1BYTE 2 0 1")]
#[test_case(NTF_3BYTE_PATH.clone(), FILE, 2, 0, 2; "LOG_3BYTE 2 0 2")]
#[test_case(NTF_3BYTE_PATH.clone(), FILE, 2, 1, 1; "LOG_3BYTE 2 1 1")]
#[test_case(NTF_3BYTE_PATH.clone(), FILE, 2, 2, 0 => panics; "LOG_3BYTE 2 2 0 panic")]
#[test_case(NTF_GZ_EMPTY_FPATH.clone(), FILE_GZ, 2, 0, 0; "GZ_0BYTE 2 0 0")]
#[test_case(NTF_GZ_EMPTY_FPATH.clone(), FILE_GZ, 2, 1, 0 => panics; "GZ_0BYTE 2 1 0 panic")]
#[test_case(NTF_GZ_1BYTE_FPATH.clone(), FILE_GZ, 2, 0, 1; "GZ_1BYTE 2 0 1")]
#[test_case(NTF_GZ_1BYTE_FPATH.clone(), FILE_GZ, 2, 1, 0 => panics; "GZ_1BYTE 2 1 0 panic")]
#[test_case(NTF_GZ_8BYTE_FPATH.clone(), FILE_GZ, 2, 0, 2; "GZ_8BYTE 2 0 2")]
#[test_case(NTF_GZ_8BYTE_FPATH.clone(), FILE_GZ, 2, 1, 2; "GZ_8BYTE 2 1 2")]
#[test_case(NTF_GZ_8BYTE_FPATH.clone(), FILE_GZ, 2, 2, 2; "GZ_8BYTE 2 2 2")]
#[test_case(NTF_GZ_8BYTE_FPATH.clone(), FILE_GZ, 2, 3, 2; "GZ_8BYTE 2 3 2")]
#[test_case(NTF_GZ_8BYTE_FPATH.clone(), FILE_GZ, 2, 4, 0 => panics; "GZ_8BYTE 2 4 0 panic")]
#[test_case(NTF_XZ_1BYTE_FPATH.clone(), FILE_XZ, 2, 0, 1; "XZ_1BYTE 2 0 1")]
#[test_case(NTF_XZ_1BYTE_FPATH.clone(), FILE_XZ, 2, 1, 0 => panics; "XZ_1BYTE 2 1 0 panic")]
#[test_case(NTF_TAR_1BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 0, 1; "TAR_1BYTE 2 0 1")]
#[test_case(NTF_TAR_1BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 1, 0 => panics; "TAR_1BYTE 2 1 0 panic")]
#[test_case(NTF_TAR_8BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 0, 2; "TAR_8BYTE 2 0 2")]
#[test_case(NTF_TAR_8BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 1, 2; "TAR_8BYTE 2 1 2")]
#[test_case(NTF_TAR_8BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 2, 2; "TAR_8BYTE 2 2 2")]
#[test_case(NTF_TAR_8BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 3, 2; "TAR_8BYTE 2 3 2")]
#[test_case(NTF_TAR_8BYTE_FILEA_FPATH.clone(), FILE_TAR, 2, 4, 0 => panics; "TAR_8BYTE 2 4 0 panic")]
fn test_blocksz_at_blockoffset(
    path: FPath,
    filetype: FileType,
    blocksz: BlockSz,
    blockoffset_input: BlockOffset,
    blocksz_expect: BlockSz,
) {
    let br1 = new_BlockReader2(path, filetype, blocksz);
    let blocksz_actual: BlockSz = br1.blocksz_at_blockoffset(&blockoffset_input);
    assert_eq!(blocksz_expect, blocksz_actual,
        "BlockSz expect {}, BlockSz actual {} for blockoffset {}", blocksz_expect, blocksz_actual, blockoffset_input
    );
}

#[test]
fn test_count_blocks() {
    dpnxf!();
    assert_eq!(1, BlockReader::count_blocks(1, 1));
    assert_eq!(2, BlockReader::count_blocks(2, 1));
    assert_eq!(3, BlockReader::count_blocks(3, 1));
    assert_eq!(4, BlockReader::count_blocks(4, 1));
    assert_eq!(1, BlockReader::count_blocks(1, 2));
    assert_eq!(1, BlockReader::count_blocks(2, 2));
    assert_eq!(2, BlockReader::count_blocks(3, 2));
    assert_eq!(2, BlockReader::count_blocks(4, 2));
    assert_eq!(3, BlockReader::count_blocks(5, 2));
    assert_eq!(1, BlockReader::count_blocks(1, 3));
    assert_eq!(1, BlockReader::count_blocks(2, 3));
    assert_eq!(1, BlockReader::count_blocks(3, 3));
    assert_eq!(2, BlockReader::count_blocks(4, 3));
    assert_eq!(1, BlockReader::count_blocks(1, 4));
    assert_eq!(1, BlockReader::count_blocks(4, 4));
    assert_eq!(2, BlockReader::count_blocks(5, 4));
    assert_eq!(1, BlockReader::count_blocks(4, 5));
    assert_eq!(1, BlockReader::count_blocks(5, 5));
    assert_eq!(2, BlockReader::count_blocks(6, 5));
    assert_eq!(2, BlockReader::count_blocks(10, 5));
    assert_eq!(3, BlockReader::count_blocks(11, 5));
    assert_eq!(3, BlockReader::count_blocks(15, 5));
    assert_eq!(4, BlockReader::count_blocks(16, 5));
}

/// quick self-test
#[test]
fn test_file_offset_at_block_offset() {
    dpnxf!();
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 1));
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 2));
    assert_eq!(0, BlockReader::file_offset_at_block_offset(0, 4));
    assert_eq!(1, BlockReader::file_offset_at_block_offset(1, 1));
    assert_eq!(2, BlockReader::file_offset_at_block_offset(1, 2));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(1, 4));
    assert_eq!(2, BlockReader::file_offset_at_block_offset(2, 1));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(2, 2));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(2, 4));
    assert_eq!(3, BlockReader::file_offset_at_block_offset(3, 1));
    assert_eq!(6, BlockReader::file_offset_at_block_offset(3, 2));
    assert_eq!(12, BlockReader::file_offset_at_block_offset(3, 4));
    assert_eq!(4, BlockReader::file_offset_at_block_offset(4, 1));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(4, 2));
    assert_eq!(16, BlockReader::file_offset_at_block_offset(4, 4));
    assert_eq!(5, BlockReader::file_offset_at_block_offset(5, 1));
    assert_eq!(10, BlockReader::file_offset_at_block_offset(5, 2));
    assert_eq!(20, BlockReader::file_offset_at_block_offset(5, 4));
    assert_eq!(8, BlockReader::file_offset_at_block_offset(8, 1));
    assert_eq!(16, BlockReader::file_offset_at_block_offset(8, 2));
    assert_eq!(32, BlockReader::file_offset_at_block_offset(8, 4));
}

/// quick self-test
#[test]
fn test_block_offset_at_file_offset() {
    dpnxf!();
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 1));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(1, 1));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(2, 1));
    assert_eq!(3, BlockReader::block_offset_at_file_offset(3, 1));
    assert_eq!(4, BlockReader::block_offset_at_file_offset(4, 1));
    assert_eq!(5, BlockReader::block_offset_at_file_offset(5, 1));
    assert_eq!(8, BlockReader::block_offset_at_file_offset(8, 1));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 2));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 2));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(2, 2));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(3, 2));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(4, 2));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(5, 2));
    assert_eq!(4, BlockReader::block_offset_at_file_offset(8, 2));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(2, 3));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(3, 3));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(4, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(6, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(7, 3));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(8, 3));
    assert_eq!(3, BlockReader::block_offset_at_file_offset(9, 3));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(0, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(1, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(2, 4));
    assert_eq!(0, BlockReader::block_offset_at_file_offset(3, 4));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(4, 4));
    assert_eq!(1, BlockReader::block_offset_at_file_offset(5, 4));
    assert_eq!(2, BlockReader::block_offset_at_file_offset(8, 4));
}

/// quick self-test
#[test]
fn test_block_index_at_file_offset() {
    dpnxf!();
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(1, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(2, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(3, 1));
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 2));
    assert_eq!(1, BlockReader::block_index_at_file_offset(1, 2));
    assert_eq!(0, BlockReader::block_index_at_file_offset(2, 2));
    assert_eq!(1, BlockReader::block_index_at_file_offset(3, 2));
    assert_eq!(0, BlockReader::block_index_at_file_offset(0, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(1, 3));
    assert_eq!(2, BlockReader::block_index_at_file_offset(2, 3));
    assert_eq!(0, BlockReader::block_index_at_file_offset(3, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(4, 3));
    assert_eq!(2, BlockReader::block_index_at_file_offset(5, 3));
    assert_eq!(0, BlockReader::block_index_at_file_offset(6, 3));
    assert_eq!(1, BlockReader::block_index_at_file_offset(7, 3));
}

/// quick self-test
#[test]
fn test_file_offset_at_block_offset_index() {
    dpnxf!();
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 1, 0));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(1, 1, 0));
    assert_eq!(2, BlockReader::file_offset_at_block_offset_index(2, 1, 0));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(3, 1, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(4, 1, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 2, 0));
    assert_eq!(2, BlockReader::file_offset_at_block_offset_index(1, 2, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(2, 2, 0));
    assert_eq!(6, BlockReader::file_offset_at_block_offset_index(3, 2, 0));
    assert_eq!(8, BlockReader::file_offset_at_block_offset_index(4, 2, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 3, 0));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(1, 3, 0));
    assert_eq!(6, BlockReader::file_offset_at_block_offset_index(2, 3, 0));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(3, 3, 0));
    assert_eq!(12, BlockReader::file_offset_at_block_offset_index(4, 3, 0));
    assert_eq!(0, BlockReader::file_offset_at_block_offset_index(0, 4, 0));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(1, 4, 0));
    assert_eq!(8, BlockReader::file_offset_at_block_offset_index(2, 4, 0));
    assert_eq!(12, BlockReader::file_offset_at_block_offset_index(3, 4, 0));
    assert_eq!(16, BlockReader::file_offset_at_block_offset_index(4, 4, 0));

    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 2, 1));
    assert_eq!(3, BlockReader::file_offset_at_block_offset_index(1, 2, 1));
    assert_eq!(5, BlockReader::file_offset_at_block_offset_index(2, 2, 1));
    assert_eq!(7, BlockReader::file_offset_at_block_offset_index(3, 2, 1));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(4, 2, 1));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 3, 1));
    assert_eq!(4, BlockReader::file_offset_at_block_offset_index(1, 3, 1));
    assert_eq!(7, BlockReader::file_offset_at_block_offset_index(2, 3, 1));
    assert_eq!(10, BlockReader::file_offset_at_block_offset_index(3, 3, 1));
    assert_eq!(13, BlockReader::file_offset_at_block_offset_index(4, 3, 1));
    assert_eq!(1, BlockReader::file_offset_at_block_offset_index(0, 4, 1));
    assert_eq!(5, BlockReader::file_offset_at_block_offset_index(1, 4, 1));
    assert_eq!(9, BlockReader::file_offset_at_block_offset_index(2, 4, 1));
    assert_eq!(13, BlockReader::file_offset_at_block_offset_index(3, 4, 1));
    assert_eq!(17, BlockReader::file_offset_at_block_offset_index(4, 4, 1));
}
