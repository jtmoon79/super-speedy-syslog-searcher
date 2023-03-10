// src/tests/line_tests.rs

//! tests for `line.rs` functions

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::FileOffset;
use crate::data::line::{Line, LinePart, LinePartPtrs};
use crate::readers::blockreader::{
    Block,
    BlockIndex,
    BlockOffset,
    BlockP,
    BlockSz,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn block_new(data: &[u8]) -> BlockP {
    let block: Block = Block::from(data);

    BlockP::new(block)
}

fn blockp_new(sz: usize) -> BlockP {
    let mut block: Block = Block::with_capacity(0);
    block.clear();
    block.resize_with(sz, || 0);

    BlockP::new(block)
}

fn linepart_new(
    beg: BlockIndex,
    end: BlockIndex,
    fo: FileOffset,
    bo: BlockOffset,
    bsz: BlockSz,
) -> LinePart {
    let blockp = blockp_new(bsz as usize);

    LinePart::new(blockp, beg, end, fo, bo, bsz)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_linepart_new_0_2() {
    linepart_new(0, 2, 0, 0, 16);
}

#[test]
#[should_panic]
fn test_linepart_new_0_0() {
    linepart_new(0, 0, 0, 0, 16);
}

#[test]
#[should_panic]
fn test_linepart_new_beg999() {
    linepart_new(999, 1, 0, 0, 16);
}

#[test]
#[should_panic]
fn test_linepart_new_end999() {
    linepart_new(0, 999, 0, 0, 16);
}

#[test]
#[should_panic]
fn test_linepart_new_fo999() {
    linepart_new(0, 2, 999, 0, 16);
}

#[test]
#[should_panic]
fn test_linepart_new_bo999() {
    linepart_new(0, 2, 0, 999, 16);
}

#[test]
#[should_panic]
fn test_linepart_new_bsz999() {
    linepart_new(0, 1, 0, 1, 99999);
}

#[test]
fn test_linepart_len() {
    let linepart = linepart_new(0, 2, 0, 0, 16);
    assert_eq!(linepart.len(), 2);
}

#[test]
fn test_linepart_empty_0_2() {
    let linepart = linepart_new(0, 2, 0, 0, 16);
    assert!(!linepart.is_empty(), "expected linepart to be empty {:?}", linepart);
}

#[test]
fn test_linepart_fileoffset_begin() {
    let linepart = linepart_new(1, 2, 1 + 128, 1, 128);
    assert_eq!(linepart.fileoffset_begin(), 1 + 128);
}

#[test]
fn test_linepart_fileoffset_end() {
    let linepart = linepart_new(1, 2, 1 + 128, 1, 128);
    assert_eq!(linepart.fileoffset_end(), 1 + 2 + 128);
}

#[test]
fn test_linepart_blockoffset() {
    let linepart = linepart_new(5, 6, 5 + 128, 1, 128);
    assert_eq!(linepart.blockoffset(), 1);
}

#[test]
fn test_linepart_count_bytes() {
    let linepart = linepart_new(5, 7, 5 + 128, 1, 128);
    assert_eq!(linepart.count_bytes(), 2);
}

#[test]
fn test_linepart_contains() {
    let linepart = linepart_new(5, 7, 5 + 128, 1, 128);
    assert!(linepart.contains(&0));
}

#[test]
fn test_linepart_block_boxptr() {
    let linepart = linepart_new(5, 7, 5 + 128, 1, 128);
    let ptr = linepart.block_boxptr();
    assert_eq!(ptr.len(), 2);
}

#[test]
fn test_linepart_block_boxptr_a() {
    let linepart = linepart_new(5, 10, 5 + 128, 1, 128);
    let ptr = linepart.block_boxptr_a(&2);
    assert_eq!(ptr.len(), 3);
}

#[test]
#[should_panic]
fn test_linepart_block_boxptr_a_11_panic() {
    let linepart = linepart_new(5, 10, 5 + 128, 1, 128);
    linepart.block_boxptr_a(&11);
}

#[test]
fn test_linepart_block_boxptr_b() {
    let linepart = linepart_new(5, 10, 5 + 128, 1, 128);
    let ptr = linepart.block_boxptr_b(&2);
    assert_eq!(ptr.len(), 2);
}

#[test]
#[should_panic]
fn test_linepart_block_boxptr_b_11_panic() {
    let linepart = linepart_new(5, 10, 5 + 128, 1, 128);
    linepart.block_boxptr_b(&11);
}

#[test]
fn test_linepart_block_boxptr_ab() {
    let linepart = linepart_new(5, 10, 5 + 128, 1, 128);
    let ptr = linepart.block_boxptr_ab(&2, &4);
    assert_eq!(ptr.len(), 2);
}

#[test]
#[should_panic]
fn test_linepart_block_boxptr_ab_3_2_panic() {
    let linepart = linepart_new(5, 10, 5 + 128, 1, 128);
    linepart.block_boxptr_ab(&3, &2);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// create line `"bc"`
fn new_line_1_3() -> Line {
    let mut line: Line = Line::new();
    let blockp: BlockP = block_new(&[
        b'a', b'b', b'c', b'd',
    ]);
    let linepart: LinePart = LinePart::new(blockp, 1, 3, 1, 0, 4);
    line.append(linepart);

    line
}

/// create line `"cdef"`
fn new_line_2_6() -> Line {
    let mut line: Line = Line::new();
    // second half of first block
    let blockp: BlockP = block_new(&[
        b'a', b'b', b'c', b'd',
    ]);
    let linepart: LinePart = LinePart::new(blockp, 2, 4, 6, 1, 4);
    line.append(linepart);
    // first half of second block
    let blockp: BlockP = block_new(&[
        b'e', b'f', b'g', b'h',
    ]);
    let linepart: LinePart = LinePart::new(blockp, 0, 2, 8, 2, 4);
    line.append(linepart);

    line
}

#[test]
fn test_line_new_0() {
    new_line_2_6();
}

#[test]
fn test_line_fileoffset_begin() {
    let line = new_line_2_6();
    assert_eq!(line.fileoffset_begin(), 6);
}

#[test]
fn test_line_fileoffset_end() {
    let line = new_line_2_6();
    assert_eq!(line.fileoffset_end(), 9);
}

#[test]
fn test_line_blockoffset_first() {
    let line = new_line_2_6();
    assert_eq!(line.blockoffset_first(), 1);
}

#[test]
fn test_line_blockoffset_last() {
    let line = new_line_2_6();
    assert_eq!(line.blockoffset_last(), 2);
}

#[test]
fn test_line_occupies_one_block_2_4() {
    let line = new_line_1_3();

    assert!(line.occupies_one_block());
}

#[test]
fn test_line_occupies_one_block_2_6() {
    let line = new_line_2_6();
    assert!(!line.occupies_one_block());
}

#[test]
fn test_line_len() {
    let line = new_line_2_6();
    assert_eq!(line.len(), 4);
}

#[test]
fn test_line_count_lineparts1() {
    let line = new_line_1_3();
    assert_eq!(line.count_lineparts(), 1);
}

#[test]
fn test_line_count_lineparts2() {
    let line = new_line_2_6();
    assert_eq!(line.count_lineparts(), 2);
}

#[test]
fn test_line_stores_blockoffset_1() {
    let line = new_line_2_6();
    assert!(line.stores_blockoffset(2));
}

#[test]
fn test_line_stores_blockoffset_2() {
    let line = new_line_2_6();
    assert!(!line.stores_blockoffset(0));
}

#[test]
fn test_line_get_slices() {
    let line = new_line_2_6();
    let slices = line.get_slices();
    assert_eq!(slices.len(), 2);
}

#[test]
fn test_line_get_boxptrs_2_6_no_ptr() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(99, 999);
    assert!(boxptrs.is_no_ptr());
}

#[test]
fn test_line_get_boxptrs_2_6_single_zero_length() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(0, 0);
    assert!(boxptrs.is_single_ptr());
    let bptr: Box<&[u8]> = match boxptrs {
        LinePartPtrs::SinglePtr(ptr) => ptr,
        _ => {
            panic!("bad pointer type");
        }
    };
    assert_eq!((*bptr).len(), 0);
}

#[test]
fn test_line_get_boxptrs_2_6_single_0_2() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(0, 2);
    assert!(boxptrs.is_single_ptr());
    let bptr: Box<&[u8]> = match boxptrs {
        LinePartPtrs::SinglePtr(ptr) => ptr,
        _ => {
            panic!("bad pointer type");
        }
    };
    assert_eq!((*bptr).len(), 2);
}

#[test]
fn test_line_get_boxptrs_2_6_single_3_99() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(3, 99);
    assert!(boxptrs.is_single_ptr());
    let bptr: Box<&[u8]> = match boxptrs {
        LinePartPtrs::SinglePtr(ptr) => ptr,
        _ => {
            panic!("bad pointer type");
        }
    };
    assert_eq!((*bptr).len(), 1);
}

#[test]
fn test_line_get_boxptrs_2_6_single_0_1() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(0, 1);
    assert!(boxptrs.is_single_ptr());
    let bptr: Box<&[u8]> = match boxptrs {
        LinePartPtrs::SinglePtr(ptr) => ptr,
        _ => {
            panic!("bad pointer type");
        }
    };
    assert_eq!((*bptr).len(), 1);
}

#[test]
fn test_line_get_boxptrs_2_6_double_0_4() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(0, line.len());
    assert!(boxptrs.is_double_ptr());
    let (bptr1, bptr2): (Box<&[u8]>, Box<&[u8]>) = match boxptrs {
        LinePartPtrs::DoublePtr(ptr1, ptr2) => (ptr1, ptr2),
        _ => {
            panic!("bad pointer type");
        }
    };
    assert_eq!((*bptr1).len(), 2);
    assert_eq!((*bptr2).len(), 2);
}

#[test]
fn test_line_get_boxptrs_2_6_double_0_3() {
    let line = new_line_2_6();
    let boxptrs: LinePartPtrs = line.get_boxptrs(0, line.len() - 1);
    assert!(boxptrs.is_double_ptr());
    let (bptr1, bptr2): (Box<&[u8]>, Box<&[u8]>) = match boxptrs {
        LinePartPtrs::DoublePtr(ptr1, ptr2) => (ptr1, ptr2),
        _ => {
            panic!("bad pointer type");
        }
    };
    assert_eq!((*bptr1).len(), 2);
    assert_eq!((*bptr2).len(), 1);
}
