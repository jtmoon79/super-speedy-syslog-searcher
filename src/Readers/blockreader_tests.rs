// Readers/blockreader_tests.rs
//

use crate::common::{
    FileType,
};

use crate::Readers::filepreprocessor::{
    guess_filetype_from_fpath,
};

use super::blockreader::{
    FPath,
    FileOffset,
    BlockSz,
    BlockReader,
    ResultS3_ReadBlock,
    printblock,
    MimeGuess,
};

use crate::dbgpr::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_with_name_exact,
    create_temp_file_with_suffix,
    create_temp_file_bytes_with_suffix,
    NTF_Path,
};

use crate::dbgpr::stack::{
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper wrapper to create a new BlockReader
fn new_BlockReader(path: FPath, blocksz: BlockSz) -> BlockReader {
    stack_offset_set(Some(2));
    let filetype: FileType = guess_filetype_from_fpath(&path);
    match BlockReader::new(path.clone(), filetype, blocksz) {
        Ok(br) => {
            eprintln!("opened {:?}", path);
            eprintln!("new {:?}", &br);
            br
        },
        Err(err) => {
            panic!("ERROR: BlockReader.open({:?}, {}) {}", path, blocksz, err);
        },
    }
}

// -------------------------------------------------------------------------------------------------

/// quick and dirty test of basic test of BlockReader things
/// 
/// TODO: improve this: add proper checking with `assert`, allow other inputs
#[allow(non_snake_case)]
fn test_BlockReader(path: &FPath, blocksz: BlockSz) {
    eprintln!("test_BlockReader({:?}, {})", path, blocksz);
    let mut br1 = new_BlockReader(path.clone(), blocksz);
    let last_blk = BlockReader::block_offset_at_file_offset(br1.filesz(), blocksz);
    for offset in [0, 1, 5, 1, 99, 1, last_blk].iter() {
        {
            let rbp = br1.read_block(*offset);
            match rbp {
                ResultS3_ReadBlock::Found(val) => {
                    let boff: FileOffset = BlockReader::file_offset_at_block_offset(*offset, blocksz);
                    printblock(val.as_ref(), *offset, boff, blocksz, String::new());
                },
                ResultS3_ReadBlock::Done => {
                    continue;
                },
                ResultS3_ReadBlock::Err(err) => {
                    panic!("ERROR: blockreader.read({}) error {}", offset, err);
                }
            };
        }
    }
    eprintln!("after reads {:?}", &br1);
    // TODO: need to compare results to expected Block values
}

// -------------------------------------------------------------------------------------------------

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref NTF_empty0: NamedTempFile = create_temp_file("");
    static ref NTF_empty0_path: FPath = NTF_Path(&NTF_empty0);
    static ref NTF_nl_1: NamedTempFile = create_temp_file("\n");
    static ref NTF_nl_1_path: FPath = NTF_Path(&NTF_nl_1);
    static ref NTF_basic_basic_dt10: NamedTempFile = create_temp_file(
"2000-01-01 00:00:01 1
2000-01-01 00:00:02 1
2000-01-01 00:00:02 2
2000-01-01 00:00:03 1
2000-01-01 00:00:03 2
2000-01-01 00:00:03 3
2000-01-01 00:00:04 1
2000-01-01 00:00:04 2
2000-01-01 00:00:04 3
2000-01-01 00:00:04 4
2000-01-01 00:00:05 1
2000-01-01 00:00:05 2
2000-01-01 00:00:05 3
2000-01-01 00:00:05 4
2000-01-01 00:00:05 5
2000-01-01 00:00:06 1
2000-01-01 00:00:06 2
2000-01-01 00:00:06 3
2000-01-01 00:00:06 4
2000-01-01 00:00:06 5
2000-01-01 00:00:06 6
2000-01-01 00:00:07 1
2000-01-01 00:00:07 2
2000-01-01 00:00:07 3
2000-01-01 00:00:07 4
2000-01-01 00:00:07 5
2000-01-01 00:00:07 6
2000-01-01 00:00:07 7
2000-01-01 00:00:08 1
2000-01-01 00:00:08 2
2000-01-01 00:00:08 3
2000-01-01 00:00:08 4
2000-01-01 00:00:08 5
2000-01-01 00:00:08 6
2000-01-01 00:00:08 7
2000-01-01 00:00:08 8
2000-01-01 00:00:09 1
2000-01-01 00:00:09 2
2000-01-01 00:00:09 3
2000-01-01 00:00:09 4
2000-01-01 00:00:09 5
2000-01-01 00:00:09 6
2000-01-01 00:00:09 7
2000-01-01 00:00:09 8
2000-01-01 00:00:09 9
2000-01-01 00:00:10 1
2000-01-01 00:00:10 2
2000-01-01 00:00:10 3
2000-01-01 00:00:10 4
2000-01-01 00:00:10 5
2000-01-01 00:00:10 6
2000-01-01 00:00:10 7
2000-01-01 00:00:10 8
2000-01-01 00:00:10 9
2000-01-01 00:00:10 10"
    );
    static ref NTF_basic_basic_dt10_path: FPath = NTF_Path(&NTF_basic_basic_dt10);
}

// gzip of an empty file using `gzip`

const GZ_EMPTY_DATA: [u8; 26] = [
    0x1f, 0x8b, 0x08, 0x08, 0x47, 0xd7, 0xa6, 0x62,
    0x00, 0x03, 0x65, 0x6d, 0x70, 0x74, 0x79, 0x00,
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00,
];

lazy_static! {
    static ref NTF_EMPTY_GZ: NamedTempFile = create_temp_file_bytes_with_suffix(
        &GZ_EMPTY_DATA, &String::from("-empty.gz")
    );
    static ref NTF_GZ_EMPTY_PATH: FPath = NTF_Path(&NTF_EMPTY_GZ);
}

// gzip of a one-byte file using `gzip`

const GZ_ONEBYTE_DATA: [u8; 24] = [
    0x1f, 0x8b, 0x08, 0x08, 0xac, 0xdf, 0xa6, 0x62,
    0x00, 0x03, 0x41, 0x00, 0x4b, 0xe4, 0x02, 0x00,
    0x07, 0xa1, 0xea, 0xdd, 0x02, 0x00, 0x00, 0x00,
];

lazy_static! {
    static ref NTF_ONEBYTE_GZ: NamedTempFile = create_temp_file_bytes_with_suffix(
        &GZ_ONEBYTE_DATA, &String::from("-onebyte.gz")
    );
    static ref NTF_GZ_ONEBYTE_PATH: FPath = NTF_Path(&NTF_ONEBYTE_GZ);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_BlockReader1() {
    test_BlockReader(&NTF_basic_basic_dt10_path, 2);
}

// TODO: [2022/04] add more tests

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
fn test_mimeguess(suffix: &String, check: MimeGuess) {
    eprintln!("test_mimeguess: suffix {:?}", &suffix);
    let ntf = create_temp_file_with_suffix("", suffix);
    let path = NTF_Path(&ntf);
    eprintln!("test_mimeguess: tempfile {:?}", &path);
    let br1 = new_BlockReader(path, 2);
    eprintln!("test_mimeguess: blockreader.mimeguess {:?}", &br1.mimeguess());
    assert_eq!(check, br1.mimeguess(), "expected MimeGuess {:?}\nfound MimeGuess {:?}\n", check, br1.mimeguess());
}

#[test]
fn test_mimeguess_txt() {
    test_mimeguess(&String::from(".txt"), MimeGuess::from_ext("txt"));
}

#[test]
fn test_mimeguess_gz_onebyte() {
    //eprintln!("NTF_GZ_EMPTY_PATH: {:?}", NTF_GZ_EMPTY_PATH.to_string());
    //eprintln!("test_mimeguess_gz: sleep(33)…");
    //std::thread::sleep(std::time::Duration::from_secs(33));
// LAST WORKING HERE 2022/06/13 16:32:22
    let br1 = new_BlockReader(NTF_GZ_ONEBYTE_PATH.clone(), 2);
    eprintln!("test_mimeguess_gz: blockreader.mimeguess {:?}", &br1.mimeguess());
    let check = MimeGuess::from_ext("gz");
    assert_eq!(check, br1.mimeguess(), "expected MimeGuess {:?}\nfound MimeGuess {:?}\n", check, br1.mimeguess());
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
fn test_filetype(name: &String, check: FileType) {
    eprintln!("test_filetype: name {:?}", &name);
    let ntf = create_temp_file_with_name_exact("", name);
    let path = NTF_Path(&ntf);
    eprintln!("test_filetype: tempfile {:?}", &path);
    let br1 = new_BlockReader(path, 2);
    let filetype = br1.filetype();
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

// LAST WORKING HERE 2022/06/13 00:16:03
// test `test_filetype_FILE_messages` fails.
// `BlockReader::new` is doing simplistic mimeguess. Instead, have
// the caller decide the associated `FileType`.
// This will use functions in `filepreprocess.rs`. Something in the
// that same file should use `FPath` to determine `MimeGuess` then `FileType`.
// Something in `filepreprocssor.rs` could have a funciton to create new
// `SyslogProcessor` and hide the `MimeGuess` to `FileType` guessing.
// The new `SyslogProcessor` can pass the `FileType` all the way
// down to `BlockReader`. AFAICT, nothing needs to care about `MimeGuess` after
// `FileType` is chosen (so it can be dropped... or let SyslogProcessor hang on to it?).

#[test]
fn test_filetype_FILE_messages() {
    test_filetype(&String::from("messages"), FileType::FILE);
}

#[test]
fn test_filetype_FILEGZ_gz() {
    let br1 = new_BlockReader(NTF_GZ_ONEBYTE_PATH.clone(), 2);
    let filetype = br1.filetype();
    eprintln!("test_mimeguess_gz: blockreader.filetype {:?}", &filetype);
    let check = FileType::FILE_GZ;
    assert_eq!(check, filetype, "expected FileType {:?}\nfound FileType {:?}\n", check, filetype);
}

// -------------------------------------------------------------------------------------------------

/// quick self-test
#[test]
fn test_count_blocks() {
    eprintln!("test_count_blocks()");
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
    eprintln!("test_file_offset_at_block_offset()");
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
    eprintln!("test_block_offset_at_file_offset()");
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
    eprintln!("test_block_index_at_file_offset()");
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
    eprintln!("test_file_offset_at_block_offset_index()");
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
