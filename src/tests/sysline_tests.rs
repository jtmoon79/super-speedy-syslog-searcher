// src/tests/sysline_tests.rs
// …

//! tests for `sysline.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::FileOffset;
use crate::data::datetime::{DateTimeL, Duration};
use crate::data::line::{Line, LineP, LinePart};
use crate::data::sysline::{Sysline, SyslineP};
use crate::readers::blockreader::{Block, BlockIndex, BlockOffset, BlockP, BlockSz};

use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::si_trace_print::printers::{defn, defo, defx};


const DT_STR0: &str = "2022-01-02T03:04:05+08:00";
//const DATA_STR0: &str = "2022-01-02T03:04:05 0800 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ 😀😁😂😃😄😅😆😇😈😉😊😋😌😍😎😏😐😑😒😓😔😕😖😗😘😙😚😛😜😝😞😟😠😡😢😣😤😥😦😧😨😩😪😫😬😭😮😯😰😱😲😳😴😵😶😷😸😹😺😻😼😽😾😿🙀🙁🙂🙃 🌚🌛🌜🌝";
const DATA_STR0: &str =
    "2022-01-02T03:04:05 0800 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZ";
const DATA_STR0_LAST_BYTE: u8 = b'Z';
const DT_BEG0: usize = 0;
const DT_END0: usize = 24;

const DT_STR1: &str = "2022-01-02T03:04:06+08:00";

#[allow(dead_code)]
const DT_STR2: &str = "2022-01-02T03:04:22+09:00";
const DATA_STR2: &str =
    "[DEBUG] 2022-01-02T03:04:22 0900 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZ\n";
const DATA_STR2_LAST_BYTE: u8 = b'\n';
const DT_BEG2: usize = 8;
const DT_END2: usize = 32;

const BLOCKSZ: BlockSz = 16;
const BLOCKOFFSET_INIT: BlockOffset = 2;
lazy_static! {
    static ref DT_0: DateTimeL = DateTimeL::parse_from_rfc3339(DT_STR0).unwrap();
    static ref DT_1: DateTimeL = DateTimeL::parse_from_rfc3339(DT_STR1).unwrap();
    static ref DIFF_0_1: Duration = Duration::try_seconds(1).unwrap();
    static ref BLOCKOFFSET_LAST: BlockOffset = {
        let n_: BlockOffset = (DATA_STR0.as_bytes().len() / (BLOCKSZ as usize)) as BlockOffset;
        let x_: BlockOffset = match DATA_STR0.as_bytes().len() % (BLOCKSZ as usize) {
            0 => 0 as BlockOffset,
            _ => 1 as BlockOffset,
        };

        (n_ + x_ + BLOCKOFFSET_INIT - 1) as BlockOffset
    };
}

/// create an interesting sysline
fn new_sysline(
    data: &str,
    _dt_beg: usize,
    _dt_end: usize,
) -> Sysline {
    let at_stop: usize = data.as_bytes().len();
    let mut at_byte: usize = 0;
    let mut bo_off: BlockOffset = BLOCKOFFSET_INIT;
    let mut fo_byte: FileOffset = (BLOCKOFFSET_INIT * (BLOCKSZ as BlockOffset)) as FileOffset;
    let mut line: Line = Line::new();
    let dt = *DT_0;
    defn!("dt: {:?}\n", dt);
    while at_byte < at_stop {
        let mut block: Block = Block::with_capacity(BLOCKSZ as usize);
        defo!("data.as_bytes().iter().skip({}).take({})", at_byte, BLOCKSZ);
        for byte_ in data
            .as_bytes()
            .iter()
            .skip(at_byte)
            .take(BLOCKSZ as usize)
        {
            block.push(*byte_);
        }
        let blocksz = block.len();
        defo!("block.resize({}, 0)", blocksz);
        block.resize(blocksz, 0);
        let blockp: BlockP = BlockP::new(block);
        defo!(
            "LinePart::(…, {}, {}, {}, {}, {})",
            0,
            blocksz - 1,
            fo_byte,
            bo_off,
            blocksz
        );
        let linepart: LinePart =
            LinePart::new(blockp, 0 as BlockIndex, blocksz as BlockIndex, fo_byte, bo_off, BLOCKSZ);
        eprintln!();
        line.append(linepart);
        bo_off += 1;
        fo_byte += blocksz as FileOffset;
        at_byte += blocksz;
    }
    let linep: LineP = LineP::new(line);
    let lines = vec![linep];
    let sysline: Sysline = Sysline::from_parts(lines, DT_BEG0, DT_END0, dt);

    sysline
}

fn new_sysline0() -> Sysline {
    new_sysline(DATA_STR0, DT_BEG0, DT_END0)
}

fn new_sysline2() -> Sysline {
    new_sysline(DATA_STR2, DT_BEG2, DT_END2)
}

// ---------------
// Sysline testing

#[test]
fn test_sysline_new_sysline1() {
    new_sysline0();
}

#[test]
fn test_sysline_dt() {
    let sysline: Sysline = new_sysline0();
    sysline.dt();
}

#[test]
fn test_sysline_dt_difference() {
    let sysline0: Sysline = new_sysline0();
    let sysline2: Sysline = new_sysline0();
    let syslinep1 = SyslineP::new(sysline0);
    let duration: Duration = sysline2.dt_difference(&syslinep1);
    assert_eq!(duration.num_seconds(), 0);
}

#[test]
fn test_sysline_blockoffset() {
    let sysline: Sysline = new_sysline0();
    let fo_first = sysline.blockoffset_first();
    assert_eq!(fo_first, BLOCKOFFSET_INIT);
    let last_: BlockOffset = *BLOCKOFFSET_LAST;
    assert_eq!(sysline.blockoffset_last(), last_);
}

#[test]
fn test_sysline_len() {
    let sysline: Sysline = new_sysline0();
    assert_eq!(sysline.len(), DATA_STR0.as_bytes().len());
}

#[test]
fn test_sysline_count_lines() {
    let sysline: Sysline = new_sysline0();
    assert_eq!(sysline.count_lines(), 1);
}

#[test]
fn test_sysline_occupies_one_block() {
    let sysline: Sysline = new_sysline0();
    assert!(!sysline.occupies_one_block());
}

#[test]
fn test_sysline_last_byte_sysline0() {
    let sysline: Sysline = new_sysline0();
    let last_byte: Option<u8> = sysline.last_byte();
    assert_eq!(
        Some(DATA_STR0_LAST_BYTE),
        last_byte,
        "expected {:?}, got {:?}",
        DATA_STR0_LAST_BYTE,
        last_byte
    )
}

#[test]
fn test_sysline_last_byte_sysline2() {
    let sysline: Sysline = new_sysline2();
    let last_byte: Option<u8> = sysline.last_byte();
    assert_eq!(
        Some(DATA_STR2_LAST_BYTE),
        last_byte,
        "expected {:?}, got {:?}",
        DATA_STR2_LAST_BYTE,
        last_byte
    )
}

#[test]
fn test_sysline_ends_with_newline_sysline0() {
    let sysline: Sysline = new_sysline0();
    let newline = sysline.ends_with_newline();
    assert!(!newline, "did not expect a newline!");
}

#[test]
fn test_sysline_ends_with_newline_sysline2() {
    let sysline: Sysline = new_sysline2();
    let newline = sysline.ends_with_newline();
    assert!(newline, "expected a newline!");
}

#[test]
fn test_sysline_get_slices() {
    let cap: usize = DATA_STR0.as_bytes().len() + 1;

    let sysline: Sysline = new_sysline0();
    let slices = sysline.get_slices();

    let mut buffer: Vec<u8> = Vec::<u8>::with_capacity(cap);
    for slice_ in slices.iter() {
        for byte_ in slice_.iter() {
            buffer.push(*byte_);
        }
    }
    let buf_str = String::from_utf8_lossy(&buffer);
    assert_eq!(DATA_STR0, buf_str);
}
