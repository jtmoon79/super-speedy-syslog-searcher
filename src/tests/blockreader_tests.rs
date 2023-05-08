// src/tests/blockreader_tests.rs

//! tests for `BlockReader`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use crate::common::{
    Bytes,
    Count,
    FileSz,
    FileOffset,
    FileType,
    FPath,
    ResultS3,
};
use crate::debug::printers::{
    byte_to_char_noraw,
};
use crate::readers::blockreader::{
    BlockOffset,
    BlockReader,
    BlockSz,
    ResultS3ReadBlock,
    ResultReadData,
    ReadData,
    ReadDataParts,
    ResultReadDataToBuffer,
    SummaryBlockReader,
};
#[allow(unused_imports)]
use crate::debug::helpers::{
    create_temp_file,
    create_temp_file_bytes_with_suffix,
    create_temp_file_with_name_exact,
    create_temp_file_with_suffix,
    ntf_fpath,
    NamedTempFile,
};
#[allow(unused_imports)]
use crate::tests::common::{
    BYTES_A,
    BYTES_AB,
    BYTES_ABCD,
    BYTES_ABCDEFGH,
    BYTES_C,
    BYTES_CD,
    BYTES_EFGH,
    NTF_1BYTE_FPATH,
    NTF_3BYTE_FPATH,
    NTF_8BYTE_FPATH,
    NTF_SYSLINE_2_PATH,
    NTF_GZ_1BYTE_FPATH,
    NTF_GZ_8BYTE_FPATH,
    NTF_GZ_EMPTY_FPATH,
    NTF_LOG_EMPTY_FPATH,
    NTF_TAR_0BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FILEA_FPATH,
    NTF_TAR_1BYTE_FPATH,
    NTF_TAR_3BYTE_OLDGNU_FILEA_FPATH,
    NTF_TAR_3BYTE_PAX_FILEA_FPATH,
    NTF_TAR_3BYTE_USTAR_FILEA_FPATH,
    NTF_TAR_8BYTE_FILEA_FPATH,
    NTF_XZ_1BYTE_FPATH,
    NTF_XZ_8BYTE_FPATH,
    NTF_XZ_EMPTY_FPATH,
    NTF_UTMPX_2ENTRY_FPATH,
    NTF_UTMPX_2ENTRY_FILETYPE,
    UTMPX_2ENTRY_FILESZ,
};

use std::collections::BTreeMap;

use ::lazy_static::lazy_static;
use ::si_trace_print::stack::stack_offset_set;
use ::si_trace_print::{defn, defo, defx, defñ};
use ::test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper wrapper to create a new BlockReader
fn new_BlockReader(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
) -> BlockReader {
    stack_offset_set(Some(2));
    match BlockReader::new(path.clone(), filetype, blocksz) {
        Ok(br) => {
            defo!("opened {:?}", path);
            defo!("new {:?}", &br);
            br
        }
        Err(err) => {
            panic!("ERROR: BlockReader.open({:?}, {}) {}", path, blocksz, err);
        }
    }
}

#[test]
fn test_new_BlockReader_1() {
    new_BlockReader(
        &NTF_LOG_EMPTY_FPATH,
        FileType::File,
        1024
    );
}

#[test]
#[should_panic]
fn test_new_BlockReader_2_bad_path_panics() {
    new_BlockReader(
        &FPath::from("THIS/PATH_DOES/NOT///EXIST!!!"),
        FileType::File,
        1024
    );
}

// -------------------------------------------------------------------------------------------------

type ResultS3_Check = ResultS3<(), ()>;
type Checks = BTreeMap<BlockOffset, (Vec<u8>, ResultS3_Check)>;

const FOUND: ResultS3_Check = ResultS3_Check::Found(());
const DONE: ResultS3_Check = ResultS3_Check::Done;

/// test of basic test of BlockReader things
#[allow(non_snake_case)]
fn test_BlockReader(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
    offsets: &[BlockOffset],
    checks: &Checks,
) {
    defn!("({:?}, {})", path, blocksz);
    let mut br1 = new_BlockReader(path, filetype, blocksz);

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
        defo!("get_block({})", offset);
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

    defx!();
}

// TODO: [2022/08/05] tests for bad files, unparsable files, multiple streams, etc.

// -------------------------------------------------------------------------------------------------

// reading plain file tests

lazy_static! {
    static ref NTF_BASIC_10: NamedTempFile = {
        create_temp_file(
            "\
1901-01-01 00:01:01 1
1902-01-02 00:02:02 2
1903-01-03 00:03:03 3
1904-01-04 00:04:04 4
1905-01-05 00:05:05 5
1906-01-06 00:10:06 6
1907-01-07 00:11:07 7
1908-01-08 00:12:08 8
1909-01-09 00:13:09 9
1910-01-10 00:14:10 10",
        )
    };
    static ref NTF_BASIC_10_FPATH: FPath = ntf_fpath(&NTF_BASIC_10);
}

#[test]
fn test_new_read_block_basic10_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_basic10_0_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_basic10_0_1_2() {
    let offsets: Vec<BlockOffset> = vec![0, 1, 2];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    checks.insert(2, (vec![b'-', b'0'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_basic10_2_1_0() {
    let offsets: Vec<BlockOffset> = vec![2, 1, 0];
    let mut checks = Checks::new();
    checks.insert(2, (vec![b'-', b'0'], FOUND));
    checks.insert(1, (vec![b'0', b'1'], FOUND));
    checks.insert(0, (vec![b'1', b'9'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_basic10_0_1_2_3_1_1() {
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
fn test_new_read_block_basic10_33_34_32() {
    let offsets: Vec<BlockOffset> = vec![33, 34, 32];
    let mut checks = Checks::new();
    checks.insert(33, (vec![b'1', b'9'], FOUND));
    checks.insert(34, (vec![b'0', b'4'], FOUND));
    checks.insert(32, (vec![b'3', b'\n'], FOUND));
    let ft = FileType::File;
    test_BlockReader(&NTF_BASIC_10_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_basic10_34_Done_34() {
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
fn test_new_read_block_gz_0bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![], DONE));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_EMPTY_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_1bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'A'], FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_1BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_0_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_0_1_0() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_1_0() {
    let offsets: Vec<BlockOffset> = vec![1, 0];
    let mut checks = Checks::new();
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_0_1_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 4, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_0_1_Done_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    checks.insert(2, (vec![], DONE));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 4, &offsets, &checks);
}

#[test]
fn test_new_read_block_gz_8bytes_0_bsz16() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(1, (vec![], DONE));
    checks.insert(0, (BYTES_ABCDEFGH.clone(), FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::Gz;
    test_BlockReader(&NTF_GZ_8BYTE_FPATH, ft, 16, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

// reading xz file tests

#[test]
fn test_new_read_block_xz_0bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![], DONE));
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_EMPTY_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_1bytes() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'A'], FOUND));
    checks.insert(1, (vec![], DONE));
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_1BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_0_1() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_0_1_0() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_1_0() {
    let offsets: Vec<BlockOffset> = vec![1, 0];
    let mut checks = Checks::new();
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_xz_8bytes_0_1_bsz4() {
    let offsets: Vec<BlockOffset> = vec![0, 1];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_ABCD.clone(), FOUND));
    checks.insert(1, (BYTES_EFGH.clone(), FOUND));
    let ft = FileType::Xz;
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
    let ft = FileType::Xz;
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
    let ft = FileType::Xz;
    test_BlockReader(&NTF_XZ_8BYTE_FPATH, ft, 16, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

// reading tar file tests

#[test]
fn test_new_read_block_tar_0byte_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![], DONE));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_0BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_1byte_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (vec![b'A'], FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_1BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_3byte_oldgnu_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_3BYTE_OLDGNU_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_3byte_oldgnu_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_C.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_3BYTE_OLDGNU_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_3byte_pax_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_3BYTE_PAX_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_3byte_pax_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_C.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_3BYTE_PAX_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_3byte_ustar_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_3BYTE_USTAR_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_3byte_ustar_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_C.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_3BYTE_USTAR_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_8byte_0() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_8byte_0_1() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_8byte_0_3() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(0, (BYTES_AB.clone(), FOUND));
    checks.insert(3, (vec![b'G', b'H'], FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_8byte_1() {
    let offsets: Vec<BlockOffset> = vec![1];
    let mut checks = Checks::new();
    checks.insert(1, (BYTES_CD.clone(), FOUND));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

#[test]
fn test_new_read_block_tar_8byte_99() {
    let offsets: Vec<BlockOffset> = vec![0];
    let mut checks = Checks::new();
    checks.insert(99, (vec![], DONE));
    let ft = FileType::Tar;
    test_BlockReader(&NTF_TAR_8BYTE_FILEA_FPATH, ft, 2, &offsets, &checks);
}

// -------------------------------------------------------------------------------------------------

// `read_data` tests

// RD0

const RD0_SZ: usize = 0;
const RD0_BUFFER: [u8; RD0_SZ] = [];
const RD0_FILENAME: &str = "rd0.data";
const RD0_FT: FileType = FileType::File;

lazy_static! {
    static ref RD0_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD0_BUFFER.as_slice(), &String::from(RD0_FILENAME));
    static ref RD0_FP: FPath = ntf_fpath(&RD0_NTF);
}

// RD1

const RD1_SZ: usize = 1;
const RD1_BUFFER: [u8; RD1_SZ] = [0];
const RD1_FILENAME: &str = "rd1.data";
const RD1_FT: FileType = FileType::File;

lazy_static! {
    static ref RD1_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD1_BUFFER.as_slice(), &String::from(RD1_FILENAME));
    static ref RD1_FP: FPath = ntf_fpath(&RD1_NTF);
}

// RD2

const RD2_SZ: usize = 2;
const RD2_BUFFER: [u8; RD2_SZ] = [0, 1];
const RD2_FILENAME: &str = "rd2.data";
const RD2_FT: FileType = FileType::File;

lazy_static! {
    static ref RD2_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD2_BUFFER.as_slice(), &String::from(RD2_FILENAME));
    static ref RD2_FP: FPath = ntf_fpath(&RD2_NTF);
}

// RD3

const RD3_SZ: usize = 3;
const RD3_BUFFER: [u8; RD3_SZ] = [0, 1, 2];
const RD3_FILENAME: &str = "rd3.data";
const RD3_FT: FileType = FileType::File;

lazy_static! {
    static ref RD3_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD3_BUFFER.as_slice(), &String::from(RD3_FILENAME));
    static ref RD3_FP: FPath = ntf_fpath(&RD3_NTF);
}

// RD4

const RD4_SZ: usize = 4;
const RD4_BUFFER: [u8; RD4_SZ] = [0, 1, 2, 3];
const RD4_FILENAME: &str = "rd4.data";
const RD4_FT: FileType = FileType::File;

lazy_static! {
    static ref RD4_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD4_BUFFER.as_slice(), &String::from(RD4_FILENAME));
    static ref RD4_FP: FPath = ntf_fpath(&RD4_NTF);
}

// RD5

const RD5_SZ: usize = 5;
const RD5_BUFFER: [u8; RD5_SZ] = [0, 1, 2, 3, 4];
const RD5_FILENAME: &str = "rd5.data";
const RD5_FT: FileType = FileType::File;

lazy_static! {
    static ref RD5_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD5_BUFFER.as_slice(), &String::from(RD5_FILENAME));
    static ref RD5_FP: FPath = ntf_fpath(&RD5_NTF);
}

// RD6

const RD6_SZ: usize = 6;
const RD6_BUFFER: [u8; RD6_SZ] = [0, 1, 2, 3, 4, 5];
const RD6_FILENAME: &str = "rd6.data";
const RD6_FT: FileType = FileType::File;

lazy_static! {
    static ref RD6_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD6_BUFFER.as_slice(), &String::from(RD6_FILENAME));
    static ref RD6_FP: FPath = ntf_fpath(&RD6_NTF);
}

// RD7

const RD7_SZ: usize = 7;
const RD7_BUFFER: [u8; RD7_SZ] = [0, 1, 2, 3, 4, 5, 6];
const RD7_FILENAME: &str = "rd7.data";
const RD7_FT: FileType = FileType::File;

lazy_static! {
    static ref RD7_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD7_BUFFER.as_slice(), &String::from(RD7_FILENAME));
    static ref RD7_FP: FPath = ntf_fpath(&RD7_NTF);
}

// RD16

const RD16_SZ: usize = 16;
const RD16_BUFFER: [u8; RD16_SZ] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
    0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f
];
const RD16_FILENAME: &str = "rd16.data";
const RD16_FT: FileType = FileType::File;

lazy_static! {
    static ref RD16_NTF: NamedTempFile =
        create_temp_file_bytes_with_suffix(RD16_BUFFER.as_slice(), &String::from(RD16_FILENAME));
    static ref RD16_FP: FPath = ntf_fpath(&RD16_NTF);
}

#[derive(Debug, Eq, PartialEq)]
enum ResultReadData_Test {
    Found,
    Done,
    _Err,
}

const FOUND_: ResultReadData_Test = ResultReadData_Test::Found;
const DONE_: ResultReadData_Test = ResultReadData_Test::Done;

#[test_case(&RD0_FP, RD0_FT, 2, 0, 0, false, DONE_, &[]; "RD0_2_0_0")]
#[test_case(&RD0_FP, RD0_FT, 2, 0, 1, false, DONE_, &[]; "RD0_2_0_1")]
//
#[test_case(&RD1_FP, RD1_FT, 2, 0, 0, false, DONE_, &[]; "RD1_2_0_0")]
#[test_case(&RD1_FP, RD1_FT, 2, 0, 1, false, FOUND_, &[0]; "RD1_2_0_1")]
#[test_case(&RD1_FP, RD1_FT, 2, 0, 2, false, FOUND_, &[0]; "RD1_2_0_2")]
#[test_case(&RD1_FP, RD1_FT, 2, 0, 3, false, FOUND_, &[0]; "RD1_2_0_3")]
#[test_case(&RD1_FP, RD1_FT, 2, 0, 4, false, FOUND_, &[0]; "RD1_2_0_4")]
#[test_case(&RD1_FP, RD1_FT, 2, 1, 1, false, DONE_, &[]; "RD1_2_1_1")]
#[test_case(&RD1_FP, RD1_FT, 2, 1, 2, false, DONE_, &[]; "RD1_2_1_2")]
#[test_case(&RD1_FP, RD1_FT, 2, 1, 3, false, DONE_, &[]; "RD1_2_1_3")]
//
#[test_case(&RD2_FP, RD2_FT, 2, 0, 0, false, DONE_, &[]; "RD2_2_0_0")]
#[test_case(&RD2_FP, RD2_FT, 2, 0, 1, false, FOUND_, &[0]; "RD2_2_0_1")]
#[test_case(&RD2_FP, RD2_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD2_2_0_2")]
#[test_case(&RD2_FP, RD2_FT, 2, 1, 1, false, DONE_, &[]; "RD2_2_1_1")]
#[test_case(&RD2_FP, RD2_FT, 2, 1, 2, false, FOUND_, &[1]; "RD2_2_1_2")]
//
#[test_case(&RD3_FP, RD3_FT, 2, 0, 0, false, DONE_, &[]; "RD3_2_0_0")]
#[test_case(&RD3_FP, RD3_FT, 2, 0, 1, false, FOUND_, &[0]; "RD3_2_0_1")]
#[test_case(&RD3_FP, RD3_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD3_2_0_2")]
#[test_case(&RD3_FP, RD3_FT, 2, 0, 3, false, FOUND_, &[0, 1, 2]; "RD3_2_0_3")]
#[test_case(&RD3_FP, RD3_FT, 2, 1, 1, false, DONE_, &[]; "RD3_2_1_1")]
#[test_case(&RD3_FP, RD3_FT, 2, 1, 2, false, FOUND_, &[1]; "RD3_2_1_2")]
#[test_case(&RD3_FP, RD3_FT, 2, 1, 3, false, FOUND_, &[1, 2]; "RD3_2_1_3")]
#[test_case(&RD3_FP, RD3_FT, 2, 2, 2, false, DONE_, &[]; "RD3_2_2_2")]
#[test_case(&RD3_FP, RD3_FT, 2, 2, 3, false, FOUND_, &[2]; "RD3_2_2_3")]
//
#[test_case(&RD4_FP, RD4_FT, 2, 0, 0, false, DONE_, &[]; "RD4_2_0_0")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 1, false, FOUND_, &[0]; "RD4_2_0_1")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 1, true, FOUND_, &[0]; "RD4_2_0_1_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD4_2_0_2")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 2, true, FOUND_, &[0, 1]; "RD4_2_0_2_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 3, false, FOUND_, &[0, 1, 2]; "RD4_2_0_3")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD4_2_0_4")]
#[test_case(&RD4_FP, RD4_FT, 2, 0, 5, false, FOUND_, &[0, 1, 2, 3]; "RD4_2_0_5")]
#[test_case(&RD4_FP, RD4_FT, 2, 1, 1, false, DONE_, &[]; "RD4_2_1_1")]
#[test_case(&RD4_FP, RD4_FT, 2, 1, 2, false, FOUND_, &[1]; "RD4_2_1_2")]
#[test_case(&RD4_FP, RD4_FT, 2, 1, 3, false, FOUND_, &[1, 2]; "RD4_2_1_3")]
#[test_case(&RD4_FP, RD4_FT, 2, 1, 4, false, FOUND_, &[1, 2, 3]; "RD4_2_1_4")]
#[test_case(&RD4_FP, RD4_FT, 2, 2, 2, false, DONE_, &[]; "RD4_2_2_2")]
#[test_case(&RD4_FP, RD4_FT, 2, 2, 3, false, FOUND_, &[2]; "RD4_2_2_3")]
#[test_case(&RD4_FP, RD4_FT, 2, 2, 3, true, FOUND_, &[2]; "RD4_2_2_3_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 2, 4, false, FOUND_, &[2, 3]; "RD4_2_2_4")]
#[test_case(&RD4_FP, RD4_FT, 2, 2, 4, true, FOUND_, &[2, 3]; "RD4_2_2_4_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 2, 5, false, FOUND_, &[2, 3]; "RD4_2_2_5")]
//
#[test_case(&RD5_FP, RD5_FT, 2, 0, 0, false, DONE_, &[]; "RD5_2_0_0")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 1, false, FOUND_, &[0]; "RD5_2_0_1")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 1, true, FOUND_, &[0]; "RD5_2_0_1_oneblock")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD5_2_0_2")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 2, true, FOUND_, &[0, 1]; "RD5_2_0_2_oneblock")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 3, false, FOUND_, &[0, 1, 2]; "RD5_2_0_3")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD5_2_0_4")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 5, false, FOUND_, &[0, 1, 2, 3, 4]; "RD5_2_0_5")]
#[test_case(&RD5_FP, RD5_FT, 2, 0, 6, false, FOUND_, &[0, 1, 2, 3, 4]; "RD5_2_0_6")]
#[test_case(&RD5_FP, RD5_FT, 2, 1, 1, false, DONE_, &[]; "RD5_2_1_1")]
#[test_case(&RD5_FP, RD5_FT, 2, 1, 2, false, FOUND_, &[1]; "RD5_2_1_2")]
#[test_case(&RD5_FP, RD5_FT, 2, 1, 3, false, FOUND_, &[1, 2]; "RD5_2_1_3")]
#[test_case(&RD5_FP, RD5_FT, 2, 1, 4, false, FOUND_, &[1, 2, 3]; "RD5_2_1_4")]
#[test_case(&RD5_FP, RD5_FT, 2, 1, 5, false, FOUND_, &[1, 2, 3, 4]; "RD5_2_1_5")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 2, false, DONE_, &[]; "RD5_2_2_2")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 3, false, FOUND_, &[2]; "RD5_2_2_3")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 3, true, FOUND_, &[2]; "RD5_2_2_3_oneblock")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 4, false, FOUND_, &[2, 3]; "RD5_2_2_4")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 4, true, FOUND_, &[2, 3]; "RD5_2_2_4_oneblock")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 5, false, FOUND_, &[2, 3, 4]; "RD5_2_2_5")]
#[test_case(&RD5_FP, RD5_FT, 2, 2, 6, false, FOUND_, &[2, 3, 4]; "RD5_2_2_6")]
//
#[test_case(&RD6_FP, RD6_FT, 2, 0, 0, false, DONE_, &[]; "RD6_2_0_0")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 1, false, FOUND_, &[0]; "RD6_2_0_1")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 1, true, FOUND_, &[0]; "RD6_2_0_1_oneblock")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD6_2_0_2")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 2, true, FOUND_, &[0, 1]; "RD6_2_0_2_oneblock")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 3, false, FOUND_, &[0, 1, 2]; "RD6_2_0_3")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD6_2_0_4")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 5, false, FOUND_, &[0, 1, 2, 3, 4]; "RD6_2_0_5")]
#[test_case(&RD6_FP, RD6_FT, 2, 0, 6, false, FOUND_, &[0, 1, 2, 3, 4, 5]; "RD6_2_0_6")]
#[test_case(&RD6_FP, RD6_FT, 2, 1, 1, false, DONE_, &[]; "RD6_2_1_1")]
#[test_case(&RD6_FP, RD6_FT, 2, 1, 2, false, FOUND_, &[1]; "RD6_2_1_2")]
#[test_case(&RD6_FP, RD6_FT, 2, 1, 3, false, FOUND_, &[1, 2]; "RD6_2_1_3")]
#[test_case(&RD6_FP, RD6_FT, 2, 1, 4, false, FOUND_, &[1, 2, 3]; "RD6_2_1_4")]
#[test_case(&RD6_FP, RD6_FT, 2, 1, 5, false, FOUND_, &[1, 2, 3, 4]; "RD6_2_1_5")]
#[test_case(&RD6_FP, RD6_FT, 2, 1, 6, false, FOUND_, &[1, 2, 3, 4, 5]; "RD6_2_1_6")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 2, false, DONE_, &[]; "RD6_2_2_2")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 3, false, FOUND_, &[2]; "RD6_2_2_3")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 3, true, FOUND_, &[2]; "RD6_2_2_3_oneblock")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 4, false, FOUND_, &[2, 3]; "RD6_2_2_4")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 4, true, FOUND_, &[2, 3]; "RD6_2_2_4_oneblock")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 5, false, FOUND_, &[2, 3, 4]; "RD6_2_2_5")]
#[test_case(&RD6_FP, RD6_FT, 2, 2, 6, false, FOUND_, &[2, 3, 4, 5]; "RD6_2_2_6")]
#[test_case(&RD6_FP, RD6_FT, 2, 3, 3, false, DONE_, &[]; "RD6_2_3_3")]
#[test_case(&RD6_FP, RD6_FT, 2, 3, 4, false, FOUND_, &[3]; "RD6_2_3_4")]
#[test_case(&RD6_FP, RD6_FT, 2, 3, 5, false, FOUND_, &[3, 4]; "RD6_2_3_5")]
#[test_case(&RD6_FP, RD6_FT, 2, 3, 6, false, FOUND_, &[3, 4, 5]; "RD6_2_3_6")]
//
#[test_case(&RD7_FP, RD7_FT, 2, 0, 0, false, DONE_, &[]; "RD7_2_0_0")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 1, false, FOUND_, &[0]; "RD7_2_0_1")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD7_2_0_2")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 3, false, FOUND_, &[0, 1, 2]; "RD7_2_0_3")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD7_2_0_4")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 5, false, FOUND_, &[0, 1, 2, 3, 4]; "RD7_2_0_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 6, false, FOUND_, &[0, 1, 2, 3, 4, 5]; "RD7_2_0_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 7, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_0_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 7, true, DONE_, &[]; "RD7_2_0_7_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 8, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_0_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 9, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_0_9")]
#[test_case(&RD7_FP, RD7_FT, 2, 0, 15, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_0_15")]
#[test_case(&RD7_FP, RD7_FT, 2, 1, 7, false, FOUND_, &[1, 2, 3, 4, 5, 6]; "RD7_2_1_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 3, false, FOUND_, &[2]; "RD7_2_2_3")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 4, false, FOUND_, &[2, 3]; "RD7_2_2_4")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 4, true, FOUND_, &[2, 3]; "RD7_2_2_4_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 5, false, FOUND_, &[2, 3, 4]; "RD7_2_2_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 6, false, FOUND_, &[2, 3, 4, 5]; "RD7_2_2_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 7, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_2_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 8, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_2_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 9, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_2_9")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 10, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_2_10")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 11, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_2_11")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 11, true, DONE_, &[]; "RD7_2_2_11_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 2, 12, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_2_12")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 3, false, DONE_, &[]; "RD7_2_3_3")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 4, false, FOUND_, &[3]; "RD7_2_3_4")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 4, true, FOUND_, &[3]; "RD7_2_3_4_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 5, false, FOUND_, &[3, 4]; "RD7_2_3_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 6, false, FOUND_, &[3, 4, 5]; "RD7_2_3_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 7, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_3_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 8, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_3_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 9, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_3_9")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 10, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_3_10")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 11, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_3_11")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 11, true, DONE_, &[]; "RD7_2_3_11_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 3, 12, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_3_12")]
#[test_case(&RD7_FP, RD7_FT, 2, 5, 5, false, DONE_, &[]; "RD7_2_5_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 5, 6, false, FOUND_, &[5]; "RD7_2_5_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 5, 7, false, FOUND_, &[5, 6]; "RD7_2_5_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 5, 8, false, FOUND_, &[5, 6]; "RD7_2_5_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 6, 6, false, DONE_, &[]; "RD7_2_6_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 6, 7, false, FOUND_, &[6]; "RD7_2_6_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 6, 8, false, FOUND_, &[6]; "RD7_2_6_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 7, 7, false, DONE_, &[]; "RD7_2_7_7")]
//
#[test_case(&RD16_FP, RD16_FT, 50, 0, 0, true, DONE_, &[]; "RD16_50_0_0_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 50, 0, 0, false, DONE_, &[]; "RD16_50_0_0")]
#[test_case(&RD16_FP, RD16_FT, 50, 1, 1, false, DONE_, &[]; "RD16_50_1_1")]
#[test_case(&RD16_FP, RD16_FT, 50, 0, 1, true, FOUND_, &[0]; "RD16_50_0_1")]
#[test_case(&RD16_FP, RD16_FT, 50, 1, 2, true, FOUND_, &[1]; "RD16_50_1_2")]
#[test_case(&RD16_FP, RD16_FT, 50, 0, 2, true, FOUND_, &[0, 1]; "RD16_50_0_2")]
#[test_case(&RD16_FP, RD16_FT, 50, 0, 4, true, FOUND_, &[0, 1, 2, 3]; "RD16_50_0_4")]
#[test_case(&RD16_FP, RD16_FT, 50, 1, 4, true, FOUND_, &[1, 2, 3]; "RD16_50_1_4")]
#[test_case(&RD16_FP, RD16_FT, 50, 2, 4, true, FOUND_, &[2, 3]; "RD16_50_2_4")]
#[test_case(&RD16_FP, RD16_FT, 50, 3, 4, true, FOUND_, &[3]; "RD16_50_3_4")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 0, false, DONE_, &[]; "RD16_2_0_0")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 1, false, FOUND_, &[0]; "RD16_2_0_1")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 2, false, FOUND_, &[0, 1]; "RD16_2_0_2")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 3, false, FOUND_, &[0, 1, 2]; "RD16_2_0_3")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD16_2_0_4")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 5, false, FOUND_, &[0, 1, 2, 3, 4]; "RD16_2_0_5")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 6, false, FOUND_, &[0, 1, 2, 3, 4, 5]; "RD16_2_0_6")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 7, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD16_2_0_7")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 7, true, DONE_, &[]; "RD16_2_0_7_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 8, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7]; "RD16_2_0_8")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 15, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]; "RD16_2_0_15")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 16, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]; "RD16_2_0_16")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 17, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]; "RD16_2_0_17")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 18, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]; "RD16_2_0_18")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 19, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]; "RD16_2_0_19")]
#[test_case(&RD16_FP, RD16_FT, 2, 0, 20, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]; "RD16_2_0_20")]
#[test_case(&RD16_FP, RD16_FT, 2, 15, 16, false, FOUND_, &[15]; "RD16_2_15_16")]
#[test_case(&RD16_FP, RD16_FT, 2, 15, 17, false, FOUND_, &[15]; "RD16_2_15_17")]
#[test_case(&RD16_FP, RD16_FT, 2, 16, 16, false, DONE_, &[]; "RD16_2_16_16")]
#[test_case(&RD16_FP, RD16_FT, 2, 1, 11, false, FOUND_, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; "RD16_2_1_11")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 3, false, FOUND_, &[2]; "RD16_2_2_3")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 4, false, FOUND_, &[2, 3]; "RD16_2_2_4")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 4, true, FOUND_, &[2, 3]; "RD16_2_2_4_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 5, false, FOUND_, &[2, 3, 4]; "RD16_2_2_5")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 6, false, FOUND_, &[2, 3, 4, 5]; "RD16_2_2_6")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 7, false, FOUND_, &[2, 3, 4, 5, 6]; "RD16_2_2_7")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 8, false, FOUND_, &[2, 3, 4, 5, 6, 7]; "RD16_2_2_8")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 9, false, FOUND_, &[2, 3, 4, 5, 6, 7, 8]; "RD16_2_2_9")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 10, false, FOUND_, &[2, 3, 4, 5, 6, 7, 8, 9]; "RD16_2_2_10")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 11, false, FOUND_, &[2, 3, 4, 5, 6, 7, 8, 9, 10]; "RD16_2_2_11")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 11, true, DONE_, &[]; "RD16_2_2_11_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 2, 2, 12, false, FOUND_, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 11]; "RD16_2_2_12")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 3, false, DONE_, &[]; "RD16_2_3_3")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 4, false, FOUND_, &[3]; "RD16_2_3_4")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 4, true, FOUND_, &[3]; "RD16_2_3_4_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 5, false, FOUND_, &[3, 4]; "RD16_2_3_5")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 6, false, FOUND_, &[3, 4, 5]; "RD16_2_3_6")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 7, false, FOUND_, &[3, 4, 5, 6]; "RD16_2_3_7")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 8, false, FOUND_, &[3, 4, 5, 6, 7]; "RD16_2_3_8")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 9, false, FOUND_, &[3, 4, 5, 6, 7, 8]; "RD16_2_3_9")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 10, false, FOUND_, &[3, 4, 5, 6, 7, 8, 9]; "RD16_2_3_10")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 11, false, FOUND_, &[3, 4, 5, 6, 7, 8, 9, 10]; "RD16_2_3_11")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 11, true, DONE_, &[]; "RD16_2_3_11_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 2, 3, 12, false, FOUND_, &[3, 4, 5, 6, 7, 8, 9, 10, 11]; "RD16_2_3_12")]
#[test_case(&RD16_FP, RD16_FT, 2, 99999998, 99999999, true, DONE_, &[]; "99999998_99999999_oneblock")]
#[test_case(&RD16_FP, RD16_FT, 2, 99999998, 99999999, false, DONE_, &[]; "99999998_99999999")]
fn test_read_data(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
    beg: FileOffset,
    end: FileOffset,
    oneblock: bool,
    result: ResultReadData_Test,
    expect_data: &[u8],
) {
    defn!("({}, {}, {}, {}, {}, {}, {:?})", path, filetype, blocksz, beg, end, oneblock, result);
    if result == DONE_ {
        assert!(expect_data.is_empty(), "bad test parameters");
    };
    let mut br1 = new_BlockReader(path, filetype, blocksz);
    let data: ReadData = match br1.read_data(beg, end, oneblock){
        ResultReadData::Found(data) => {
            assert_eq!(result, FOUND_, "expected result to be {:?}", result);

            data
        }
        ResultReadData::Done => {
            assert_eq!(result, DONE_, "expected result to be {:?}", result);
            return;
        }
        ResultReadData::Err(err) => {
            panic!("unexpected error: {:?}", err);
        }
    };
    eprintln!("expect_data: {:?}", expect_data);

    let mut at: usize = 0;
    let a: usize = data.1;
    let b: usize = data.2;
    
    match data.0 {
        ReadDataParts::One(blockp) => {
            eprintln!("blockp[{}‥{}]: ", a, b);
            for c in (*blockp)[a..b].iter() {
                assert_eq!(c, &expect_data[at], "failed to match byte at index {}", at);
                at += 1;
            }
        }
        ReadDataParts::Two(blockp1, blockp2) => {
            eprintln!("blockp1[{}‥]: (len {})", a, (*blockp1)[a..].len());
            for (i, c) in (*blockp1)[a..].iter().enumerate() {
                assert_eq!(c, &expect_data[at], "failed to match byte at index {} in blockp1[{}]", at, i);
                at += 1;
            }
            eprintln!("blockp2[‥{}]: (len {})", b, (*blockp2)[..b].len());
            for (i, c) in (*blockp2)[..b].iter().enumerate() {
                assert_eq!(c, &expect_data[at], "failed to match byte at index {} in blockp2[{}]", at, i);
                at += 1;
            }
        }
        ReadDataParts::Many(blockps) => {
            for (i, blockp) in blockps.iter().enumerate() {
                for c in (*blockp).iter()
                {
                    let c_ = byte_to_char_noraw(*c);
                    eprintln!("blockps[{}] {:2} 0x{:02X} {:?}", i, at, c, c_);
                    at += 1;
                }
            }

            for (i, c) in expect_data.iter().enumerate()
            {
                let c_ = byte_to_char_noraw(*c);
                eprintln!("expect_data[{}] 0x{:02X} {:?}", i, c, c_);
            }

            at = 0;
            for (i, blockp) in blockps.iter().enumerate() {
                let a_ = if i == 0 { a } else { 0 };
                let b_ = if i == (*blockps).len() - 1 { b } else { (*blockp).len() };
                eprintln!("blockp[{}][{}‥{}]", i, a_, b_);
                for c in (*blockp)[a_..b_].iter()
                {
                    eprint!("  {:?} = expect_data[{}] ", c, at);
                    eprintln!("{:?}", c);
                    assert_eq!(c, &expect_data[at], "failed to match byte at index {} in blockp {}", at, i);
                    at += 1;
                }
            }
        }
    }
    assert_eq!(at, expect_data.len(), "failed to match all bytes, matched {}, expected {}", at, expect_data.len());
}

#[test_case(&RD0_FP, RD0_FT, 2, 2, 0, 1, false, DONE_, &[]; "RD0_2_2_0_0")]
//
#[test_case(&RD1_FP, RD1_FT, 2, 2, 0, 0, false, DONE_, &[]; "RD1_2_2_0_0")]
#[test_case(&RD1_FP, RD1_FT, 2, 2, 0, 1, false, FOUND_, &[0]; "RD1_2_2_0_1")]
#[test_case(&RD1_FP, RD1_FT, 2, 2, 0, 2, false, FOUND_, &[0]; "RD1_2_2_0_2")]
//
#[test_case(&RD2_FP, RD2_FT, 2, 2, 0, 0, false, DONE_, &[]; "RD2_2_2_0_0")]
#[test_case(&RD2_FP, RD2_FT, 2, 2, 0, 1, false, FOUND_, &[0]; "RD2_2_2_0_1")]
#[test_case(&RD2_FP, RD2_FT, 2, 2, 0, 2, false, FOUND_, &[0, 1]; "RD2_2_2_0_2")]
#[test_case(&RD2_FP, RD2_FT, 2, 2, 1, 1, false, DONE_, &[]; "RD2_2_2_1_1")]
#[test_case(&RD2_FP, RD2_FT, 2, 2, 1, 2, false, FOUND_, &[1]; "RD2_2_2_1_2")]
//
#[test_case(&RD3_FP, RD3_FT, 2, 4, 0, 0, false, DONE_, &[]; "RD3_2_4_0_0")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 0, 1, false, FOUND_, &[0]; "RD3_2_4_0_1")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 0, 2, false, FOUND_, &[0, 1]; "RD3_2_4_0_2")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 0, 3, false, FOUND_, &[0, 1, 2]; "RD3_2_4_0_3")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 1, 1, false, DONE_, &[]; "RD3_2_4_1_1")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 1, 2, false, FOUND_, &[1]; "RD3_2_4_1_2")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 1, 3, false, FOUND_, &[1, 2]; "RD3_2_4_1_3")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 2, 2, false, DONE_, &[]; "RD3_2_4_2_2")]
#[test_case(&RD3_FP, RD3_FT, 2, 4, 2, 3, false, FOUND_, &[2]; "RD3_2_4_2_3")]
//
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 0, false, DONE_, &[]; "RD4_2_4_0_0")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 1, false, FOUND_, &[0]; "RD4_2_4_0_1")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 1, true, FOUND_, &[0]; "RD4_2_4_0_1_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 2, false, FOUND_, &[0, 1]; "RD4_2_4_0_2")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 2, true, FOUND_, &[0, 1]; "RD4_2_4_0_2_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 3, false, FOUND_, &[0, 1, 2]; "RD4_2_4_0_3")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD4_2_4_0_4")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 0, 5, false, FOUND_, &[0, 1, 2, 3]; "RD4_2_4_0_5")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 1, 1, false, DONE_, &[]; "RD4_2_4_1_1")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 1, 2, false, FOUND_, &[1]; "RD4_2_4_1_2")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 1, 3, false, FOUND_, &[1, 2]; "RD4_2_4_1_3")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 1, 4, false, FOUND_, &[1, 2, 3]; "RD4_2_4_1_4")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 2, 2, false, DONE_, &[]; "RD4_2_4_2_2")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 2, 3, false, FOUND_, &[2]; "RD4_2_4_2_3")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 2, 3, true, FOUND_, &[2]; "RD4_2_4_2_3_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 2, 4, false, FOUND_, &[2, 3]; "RD4_2_4_2_4")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 2, 4, true, FOUND_, &[2, 3]; "RD4_2_4_2_4_oneblock")]
#[test_case(&RD4_FP, RD4_FT, 2, 4, 2, 5, false, FOUND_, &[2, 3]; "RD4_2_4_2_5")]
//
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 0, false, DONE_, &[]; "RD7_2_8_0_0")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 1, false, FOUND_, &[0]; "RD7_2_8_0_1")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 2, false, FOUND_, &[0, 1]; "RD7_2_8_0_2")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 3, false, FOUND_, &[0, 1, 2]; "RD7_2_8_0_3")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 4, false, FOUND_, &[0, 1, 2, 3]; "RD7_2_8_0_4")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 5, false, FOUND_, &[0, 1, 2, 3, 4]; "RD7_2_8_0_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 6, false, FOUND_, &[0, 1, 2, 3, 4, 5]; "RD7_2_8_0_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 7, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_8_0_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 7, true, DONE_, &[]; "RD7_2_8_0_7_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 8, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_8_0_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 9, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_8_0_9")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 0, 15, false, FOUND_, &[0, 1, 2, 3, 4, 5, 6]; "RD7_2_8_0_15")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 1, 7, false, FOUND_, &[1, 2, 3, 4, 5, 6]; "RD7_2_8_1_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 3, false, FOUND_, &[2]; "RD7_2_8_2_3")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 4, false, FOUND_, &[2, 3]; "RD7_2_8_2_4")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 4, true, FOUND_, &[2, 3]; "RD7_2_8_2_4_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 5, false, FOUND_, &[2, 3, 4]; "RD7_2_8_2_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 6, false, FOUND_, &[2, 3, 4, 5]; "RD7_2_8_2_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 7, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_8_2_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 8, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_8_2_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 9, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_8_2_9")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 10, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_8_2_10")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 11, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_8_2_11")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 11, true, DONE_, &[]; "RD7_2_8_2_11_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 2, 12, false, FOUND_, &[2, 3, 4, 5, 6]; "RD7_2_8_2_12")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 3, false, DONE_, &[]; "RD7_2_8_3_3")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 4, false, FOUND_, &[3]; "RD7_2_8_3_4")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 4, true, FOUND_, &[3]; "RD7_2_8_3_4_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 5, false, FOUND_, &[3, 4]; "RD7_2_8_3_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 6, false, FOUND_, &[3, 4, 5]; "RD7_2_8_3_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 7, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_8_3_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 8, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_8_3_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 9, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_8_3_9")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 10, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_8_3_10")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 11, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_8_3_11")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 11, true, DONE_, &[]; "RD7_2_8_3_11_oneblock")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 3, 12, false, FOUND_, &[3, 4, 5, 6]; "RD7_2_8_3_12")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 5, 5, false, DONE_, &[]; "RD7_2_8_5_5")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 5, 6, false, FOUND_, &[5]; "RD7_2_8_5_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 5, 7, false, FOUND_, &[5, 6]; "RD7_2_8_5_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 5, 8, false, FOUND_, &[5, 6]; "RD7_2_8_5_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 6, 6, false, DONE_, &[]; "RD7_2_8_6_6")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 6, 7, false, FOUND_, &[6]; "RD7_2_8_6_7")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 6, 8, false, FOUND_, &[6]; "RD7_2_8_6_8")]
#[test_case(&RD7_FP, RD7_FT, 2, 8, 7, 7, false, DONE_, &[]; "RD7_2_8_7_7")]
fn test_read_data_to_buffer(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
    capacity: usize,
    beg: FileOffset,
    end: FileOffset,
    oneblock: bool,
    result: ResultReadData_Test,
    expect_data: &[u8],
) {
    defn!("({}, {}, {}, {}, {},  {}, {}, {:?})", path, filetype, blocksz, capacity, beg, end, oneblock, result);
    if result == DONE_ {
        assert!(expect_data.is_empty(), "bad test parameters");
    };
    let mut br1 = new_BlockReader(path, filetype, blocksz);
    // TODO: how to make this vec creation more idiomatic?
    let mut buffer: Vec<u8> = Vec::with_capacity(capacity);
    for _ in 0..capacity {
        buffer.push(0)
    }
    let copyn = match br1.read_data_to_buffer(beg, end, oneblock, buffer.as_mut_slice()){
        ResultReadDataToBuffer::Found(copyn) => {
            assert_eq!(result, FOUND_, "expected result to be {:?}", result);

            copyn
        }
        ResultReadDataToBuffer::Done => {
            assert_eq!(result, DONE_, "expected result to be {:?}", result);
            return;
        }
        ResultReadDataToBuffer::Err(err) => {
            panic!("unexpected error: {:?}", err);
        }
    };
    let mut at: usize = 0;
    eprintln!("expect_data: {:?}", expect_data);
    eprintln!("buffer:      {:?}", &buffer.as_slice()[..copyn]);
    eprintln!("buffer.len() {}, copyn {}", buffer.len(), copyn);
    for (i, c) in buffer.iter().take(copyn).enumerate() {
        let c_ = &expect_data[i];
        assert_eq!(c, c_, "failed to match byte at index {}, value {}", i, c_);
        at += 1;
    }
    assert_eq!(at, expect_data.len(), "failed to match all bytes, matched {}, expected {}", at, expect_data.len());

}

#[test_case(&NTF_LOG_EMPTY_FPATH, FileType::File)]
#[test_case(&NTF_1BYTE_FPATH, FileType::File)]
#[test_case(&NTF_3BYTE_FPATH, FileType::File)]
#[test_case(&NTF_GZ_EMPTY_FPATH, FileType::Gz)]
#[test_case(&NTF_GZ_1BYTE_FPATH, FileType::Gz)]
#[test_case(&NTF_GZ_8BYTE_FPATH, FileType::Gz)]
#[test_case(&NTF_XZ_1BYTE_FPATH, FileType::Xz)]
#[test_case(&NTF_TAR_1BYTE_FILEA_FPATH, FileType::Tar)]
#[test_case(&NTF_TAR_8BYTE_FILEA_FPATH, FileType::Tar)]
fn test_mtime(
    path: &FPath,
    filetype: FileType,
) {
    let br1 = new_BlockReader(path, filetype, 0x100);
    // merely run the function
    _ = br1.mtime();
}

// -------------------------------------------------------------------------------------------------

#[test_case(&NTF_LOG_EMPTY_FPATH, FileType::File, 2, 0, 0; "LOG_0BYTE 2 0 0")]
#[test_case(&NTF_1BYTE_FPATH, FileType::File, 2, 0, 1; "LOG_1BYTE 2 0 1")]
#[test_case(&NTF_3BYTE_FPATH, FileType::File, 2, 0, 2; "LOG_3BYTE 2 0 2")]
#[test_case(&NTF_3BYTE_FPATH, FileType::File, 2, 1, 1; "LOG_3BYTE 2 1 1")]
#[test_case(&NTF_3BYTE_FPATH, FileType::File, 2, 2, 0 => panics; "LOG_3BYTE 2 2 0 panic")]
#[test_case(&NTF_GZ_EMPTY_FPATH, FileType::Gz, 2, 0, 0; "GZ_0BYTE 2 0 0")]
#[test_case(&NTF_GZ_EMPTY_FPATH, FileType::Gz, 2, 1, 0 => panics; "GZ_0BYTE 2 1 0 panic")]
#[test_case(&NTF_GZ_1BYTE_FPATH, FileType::Gz, 2, 0, 1; "GZ_1BYTE 2 0 1")]
#[test_case(&NTF_GZ_1BYTE_FPATH, FileType::Gz, 2, 1, 0 => panics; "GZ_1BYTE 2 1 0 panic")]
#[test_case(&NTF_GZ_8BYTE_FPATH, FileType::Gz, 2, 0, 2; "GZ_8BYTE 2 0 2")]
#[test_case(&NTF_GZ_8BYTE_FPATH, FileType::Gz, 2, 1, 2; "GZ_8BYTE 2 1 2")]
#[test_case(&NTF_GZ_8BYTE_FPATH, FileType::Gz, 2, 2, 2; "GZ_8BYTE 2 2 2")]
#[test_case(&NTF_GZ_8BYTE_FPATH, FileType::Gz, 2, 3, 2; "GZ_8BYTE 2 3 2")]
#[test_case(&NTF_GZ_8BYTE_FPATH, FileType::Gz, 2, 4, 0 => panics; "GZ_8BYTE 2 4 0 panic")]
#[test_case(&NTF_XZ_1BYTE_FPATH, FileType::Xz, 2, 0, 1; "XZ_1BYTE 2 0 1")]
#[test_case(&NTF_XZ_1BYTE_FPATH, FileType::Xz, 2, 1, 0 => panics; "XZ_1BYTE 2 1 0 panic")]
#[test_case(&NTF_TAR_1BYTE_FILEA_FPATH, FileType::Tar, 2, 0, 1; "TAR_1BYTE 2 0 1")]
#[test_case(&NTF_TAR_1BYTE_FILEA_FPATH, FileType::Tar, 2, 1, 0 => panics; "TAR_1BYTE 2 1 0 panic")]
#[test_case(&NTF_TAR_8BYTE_FILEA_FPATH, FileType::Tar, 2, 0, 2; "TAR_8BYTE 2 0 2")]
#[test_case(&NTF_TAR_8BYTE_FILEA_FPATH, FileType::Tar, 2, 1, 2; "TAR_8BYTE 2 1 2")]
#[test_case(&NTF_TAR_8BYTE_FILEA_FPATH, FileType::Tar, 2, 2, 2; "TAR_8BYTE 2 2 2")]
#[test_case(&NTF_TAR_8BYTE_FILEA_FPATH, FileType::Tar, 2, 3, 2; "TAR_8BYTE 2 3 2")]
#[test_case(&NTF_TAR_8BYTE_FILEA_FPATH, FileType::Tar, 2, 4, 0 => panics; "TAR_8BYTE 2 4 0 panic")]
fn test_blocksz_at_blockoffset(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
    blockoffset_input: BlockOffset,
    blocksz_expect: BlockSz,
) {
    let br1 = new_BlockReader(path, filetype, blocksz);
    let blocksz_actual: BlockSz = br1.blocksz_at_blockoffset(&blockoffset_input);
    assert_eq!(
        blocksz_expect, blocksz_actual,
        "BlockSz expect {}, BlockSz actual {} for blockoffset {}",
        blocksz_expect, blocksz_actual, blockoffset_input
    );
}

#[test]
fn test_count_blocks() {
    defñ!();
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
    defñ!();
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
    defñ!();
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
    defñ!();
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
    defñ!();
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

/// test `BlockReader::summary` before doing any processing
#[test_case(&NTF_LOG_EMPTY_FPATH, FileType::File, 2)]
#[test_case(&NTF_1BYTE_FPATH, FileType::File, 2)]
fn test_BlockReader_summary_empty(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
) {
    let blockreader = new_BlockReader(
        path, filetype, blocksz
    );
    _ = blockreader.summary();
}

#[test_case(
    &NTF_LOG_EMPTY_FPATH,
    FileType::File,
    0x2,
    0,
    0,
    0,
    0,
    2,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0
)]
#[test_case(
    &NTF_1BYTE_FPATH,
    FileType::File,
    0x2,
    0,
    1,
    0,
    1,
    2,
    1,
    1,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0
)]
#[test_case(
    &NTF_SYSLINE_2_PATH,
    FileType::File,
    0x2,
    88,
    90,
    44,
    45,
    2,
    90,
    90,
    0,
    44,
    44,
    0,
    44,
    44,
    4,
    41,
    0
)]
/// test `BlockReader.Summary()`
fn test_SummaryBlockReader(
    path: &FPath,
    filetype: FileType,
    blocksz: BlockSz,
    blockreader_bytes: Count,
    blockreader_bytes_total: FileSz,
    blockreader_blocks: Count,
    blockreader_blocks_total: Count,
    blockreader_blocksz: BlockSz,
    blockreader_filesz: FileSz,
    blockreader_filesz_actual: FileSz,
    blockreader_read_block_lru_cache_hit: Count,
    blockreader_read_block_lru_cache_miss: Count,
    blockreader_read_block_lru_cache_put: Count,
    blockreader_read_blocks_hit: Count,
    blockreader_read_blocks_miss: Count,
    blockreader_read_blocks_put: Count,
    blockreader_blocks_highest: usize,
    blockreader_blocks_dropped_ok: Count,
    blockreader_blocks_dropped_err: Count,
) {
    let mut blockreader = new_BlockReader(path, filetype, blocksz);
    for bo in 0..blockreader.blockoffset_last() {
        match blockreader.read_block(bo) {
            ResultS3ReadBlock::Found(_block) => {
                // do nothing
            }
            ResultS3ReadBlock::Done => {
                panic!("read_block({}) failed: Done was unexpected", bo);
            }
            ResultS3ReadBlock::Err(e) => {
                panic!("read_block({}) failed: {}", bo, e);
            }
        }
        if bo > 2 {
            blockreader.drop_block(bo - 2);
        }
    }

    let summary: SummaryBlockReader = blockreader.summary();
    assert_eq!(
        blockreader_bytes,
        summary.blockreader_bytes,
        "blockreader_bytes 1"
    );
    assert_eq!(
        blockreader_bytes_total,
        summary.blockreader_bytes_total,
        "blockreader_bytes_total 2"
    );
    assert_eq!(
        blockreader_blocks,
        summary.blockreader_blocks,
        "blockreader_blocks 3"
    );
    assert_eq!(
        blockreader_blocks_total,
        summary.blockreader_blocks_total,
        "blockreader_blocks_total 4"
    );
    assert_eq!(
        blockreader_blocksz,
        summary.blockreader_blocksz,
        "blockreader_blocksz 5"
    );
    assert_eq!(
        blockreader_filesz,
        summary.blockreader_filesz,
        "blockreader_filesz 6"
    );
    assert_eq!(
        blockreader_filesz_actual,
        summary.blockreader_filesz_actual,
        "blockreader_filesz_actual 7"
    );
    assert_eq!(
        blockreader_read_block_lru_cache_hit,
        summary.blockreader_read_block_lru_cache_hit,
        "blockreader_read_block_lru_cache_hit 8"
    );
    assert_eq!(
        blockreader_read_block_lru_cache_miss,
        summary.blockreader_read_block_lru_cache_miss,
        "blockreader_read_block_lru_cache_miss 9"
    );
    assert_eq!(
        blockreader_read_block_lru_cache_put,
        summary.blockreader_read_block_lru_cache_put,
        "blockreader_read_block_lru_cache_put 10"
    );
    assert_eq!(
        blockreader_read_blocks_hit,
        summary.blockreader_read_blocks_hit,
        "blockreader_read_blocks_hit 11"
    );
    assert_eq!(
        blockreader_read_blocks_miss,
        summary.blockreader_read_blocks_miss,
        "blockreader_read_blocks_miss 12"
    );
    assert_eq!(
        blockreader_read_blocks_put,
        summary.blockreader_read_blocks_put,
        "blockreader_read_blocks_put 13"
    );
    assert_eq!(
        blockreader_blocks_highest,
        summary.blockreader_blocks_highest,
        "blockreader_blocks_highest 14"
    );
    assert_eq!(
        blockreader_blocks_dropped_ok,
        summary.blockreader_blocks_dropped_ok,
        "blockreader_blocks_dropped_ok 15"
    );
    assert_eq!(
        blockreader_blocks_dropped_err,
        summary.blockreader_blocks_dropped_err,
        "blockreader_blocks_dropped_err 16"
    );
}
