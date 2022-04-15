// Readers/blockreader_tests.rs
//

use crate::Readers::blockreader::{
    FPath,
    FileOffset,
    BlockSz,
    BlockReader,
    EndOfFile,
    printblock,
};

/// basic test of BlockReader things
#[allow(non_snake_case)]
fn test_BlockReader(path_: &FPath, blocksz: BlockSz) {
    eprintln!("test_BlockReader()");

    // testing BlockReader basics

    let mut br1 = BlockReader::new(path_, blocksz);
    eprintln!("new {:?}", &br1);
    match br1.open() {
        Ok(_) => {
            eprintln!("opened {:?}", path_);
        }
        Err(err) => {
            panic!("ERROR: BlockReader.open('{:?}') {}", path_, err);
        }
    }
    eprintln!("opened {:?}", &br1);
    let last_blk = BlockReader::block_offset_at_file_offset(br1.filesz, blocksz);
    for offset in [0, 1, 5, 1, 99, 1, last_blk].iter() {
        {
            let rbp = br1.read_block(*offset);
            match rbp {
                Ok(val) => {
                    let boff: FileOffset = BlockReader::file_offset_at_block_offset(*offset, blocksz);
                    printblock(val.as_ref(), *offset, boff, blocksz, String::new());
                }
                Err(err) => {
                    if err.kind() == EndOfFile {
                        continue;
                    } else {
                        panic!("ERROR: blockreader.read({}) error {}", offset, err);
                    }
                }
            };
        }
    }
    eprintln!("after reads {:?}", &br1);
}

#[test]
fn test_BlockReader1() {
    test_BlockReader(&FPath::from("./logs/other/tests/basic-basic-dt10-repeats.log"), 2);
}

/// quick self-test
#[test]
fn test_file_blocks_count() {
    eprintln!("test_file_blocks_count()");
    assert_eq!(1, BlockReader::file_blocks_count(1, 1));
    assert_eq!(2, BlockReader::file_blocks_count(2, 1));
    assert_eq!(3, BlockReader::file_blocks_count(3, 1));
    assert_eq!(4, BlockReader::file_blocks_count(4, 1));
    assert_eq!(1, BlockReader::file_blocks_count(1, 2));
    assert_eq!(1, BlockReader::file_blocks_count(2, 2));
    assert_eq!(2, BlockReader::file_blocks_count(3, 2));
    assert_eq!(2, BlockReader::file_blocks_count(4, 2));
    assert_eq!(3, BlockReader::file_blocks_count(5, 2));
    assert_eq!(1, BlockReader::file_blocks_count(1, 3));
    assert_eq!(1, BlockReader::file_blocks_count(2, 3));
    assert_eq!(1, BlockReader::file_blocks_count(3, 3));
    assert_eq!(2, BlockReader::file_blocks_count(4, 3));
    assert_eq!(1, BlockReader::file_blocks_count(1, 4));
    assert_eq!(1, BlockReader::file_blocks_count(4, 4));
    assert_eq!(2, BlockReader::file_blocks_count(5, 4));
    assert_eq!(1, BlockReader::file_blocks_count(4, 5));
    assert_eq!(1, BlockReader::file_blocks_count(5, 5));
    assert_eq!(2, BlockReader::file_blocks_count(6, 5));
    assert_eq!(2, BlockReader::file_blocks_count(10, 5));
    assert_eq!(3, BlockReader::file_blocks_count(11, 5));
    assert_eq!(3, BlockReader::file_blocks_count(15, 5));
    assert_eq!(4, BlockReader::file_blocks_count(16, 5));
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
