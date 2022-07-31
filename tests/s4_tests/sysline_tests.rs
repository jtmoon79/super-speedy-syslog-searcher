// tests/s4_tests/sysline_tests.rs
//
// …

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate s4lib;

use s4lib::common::{
    FileOffset,
    FileType,
    FPath,
};

use s4lib::Data::datetime::{
    DateTimeL,
    Duration,
};

use s4lib::Data::line::{
    LinePart,
    Line,
    LineP,
    Lines,
};

use s4lib::Data::sysline::{
    Sysline,
    SyslineP,
};

use s4lib::Readers::blockreader::{
    Block,
    BlockIndex,
    BlockP,
    BlockOffset,
    BlockSz,
};

use s4lib::printer_debug::stack::{
    stack_offset_set,
};

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn block_new(data: &[u8]) -> BlockP {
    let mut block: Block = Block::from(data);

    BlockP::new(block)
}

fn blockp_new(sz: usize) -> BlockP {
    let mut block: Block = Block::with_capacity(0);
    block.clear();
    block.resize_with(sz, || 0);

    BlockP::new(block)
}

fn linepart_new(
    beg: BlockIndex, end: BlockIndex, fo: FileOffset, bo: BlockOffset, bsz: BlockSz
) -> LinePart {
    let blockp = blockp_new(bsz as usize);

    LinePart::new(
        blockp, beg, end, fo, bo, bsz
    )
}

const DT_STR0: &str = "2022-01-02T03:04:05+08:00";
//const DATA_STR0: &str = "2022-01-02T03:04:05 0800 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ 😀😁😂😃😄😅😆😇😈😉😊😋😌😍😎😏😐😑😒😓😔😕😖😗😘😙😚😛😜😝😞😟😠😡😢😣😤😥😦😧😨😩😪😫😬😭😮😯😰😱😲😳😴😵😶😷😸😹😺😻😼😽😾😿🙀🙁🙂🙃 🌚🌛🌜🌝";
const DATA_STR0: &str = "2022-01-02T03:04:05 0800 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZ";
const DT_BEG0: usize = 0;
const DT_END0: usize = 24;
const DT_STR1: &str = "2022-01-02T03:04:06+08:00";
const BLOCKSZ: BlockSz = 16;
const BLOCKOFFSET_INIT: BlockOffset = 2;
lazy_static!{
    static ref DT_0: DateTimeL = {
        DateTimeL::parse_from_rfc3339(DT_STR0).unwrap()
    };
    static ref DT_1: DateTimeL = {
        DateTimeL::parse_from_rfc3339(DT_STR1).unwrap()
    };
    static ref DIFF_0_1: Duration = {
        Duration::seconds(1)
    };
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
fn new_sysline1() -> Sysline {
    let at_stop: usize = DATA_STR0.as_bytes().len();
    let mut at_byte: usize = 0;
    let mut at_bi_beg: usize = 0;
    let mut bo_off: BlockOffset = BLOCKOFFSET_INIT;
    let mut fo_byte: FileOffset = (BLOCKOFFSET_INIT * (BLOCKSZ as BlockOffset)) as FileOffset;
    let mut line: Line = Line::new();
    let dt = DT_0.clone();
    eprintln!("new_sysline1: dt: {:?}\n", dt);
    while at_byte < at_stop {
        let mut block: Block = Block::with_capacity(BLOCKSZ as usize);
        eprintln!("new_sysline1: DATA_STR0.as_bytes().iter().skip({}).take({})", at_byte, BLOCKSZ);
        for byte_ in DATA_STR0.as_bytes().iter().skip(at_byte).take(BLOCKSZ as usize) {
            block.push(*byte_);
        }
        let blocksz = block.len();
        eprintln!("new_sysline1: block.resize({}, 0)", blocksz);
        block.resize(blocksz, 0);
        let blockp: BlockP = BlockP::new(block);
        eprintln!("new_sysline1: LinePart::(…, {}, {}, {}, {}, {})", 0, blocksz - 1, fo_byte, bo_off, blocksz);
        let linepart: LinePart = LinePart::new(
            blockp,
            0 as BlockIndex,
            blocksz as BlockIndex,
            fo_byte,
            bo_off,
            BLOCKSZ,
        );
        //eprintln!("new_sysline1: linepart: {:?}\n", linepart);
        eprintln!();
        line.append(linepart);
        bo_off += 1;
        fo_byte += blocksz as FileOffset;
        at_byte += blocksz;
    }
    let linep: LineP = LineP::new(line);
    let mut lines = Lines::with_capacity(1);
    lines.push(linep);
    let mut sysline: Sysline = Sysline::new_from_parts(
        lines,
        DT_BEG0,
        DT_END0,
        Some(dt),
    );

    sysline
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart testing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_sysline_new_sysline1() {
    new_sysline1();
}

#[test]
fn test_sysline_dt() {
    let sysline: Sysline = new_sysline1();
    sysline.dt();
}

#[test]
fn test_sysline_dt_difference() {
    let mut sysline1: Sysline = new_sysline1();
    let mut sysline2: Sysline = new_sysline1();
    let syslinep1 = SyslineP::new(sysline1);
    let duration: Duration = sysline2.dt_difference(&syslinep1);
    assert_eq!(duration.num_seconds(), 0);
}

#[test]
fn test_sysline_blockoffset() {
    let mut sysline: Sysline = new_sysline1();
    let fo_first = sysline.blockoffset_first();
    assert_eq!(fo_first, BLOCKOFFSET_INIT);
    let last_: BlockOffset = *BLOCKOFFSET_LAST;
    assert_eq!(sysline.blockoffset_last(), last_);
}

#[test]
fn test_sysline_len() {
    let mut sysline: Sysline = new_sysline1();
    assert_eq!(sysline.len(), DATA_STR0.as_bytes().len());
}

#[test]
fn test_sysline_count_lines() {
    let mut sysline1: Sysline = new_sysline1();
    assert_eq!(sysline1.count_lines(), 1);
}

#[test]
fn test_sysline_occupies_one_block() {
    let mut sysline1: Sysline = new_sysline1();
    assert!(!sysline1.occupies_one_block());
}

#[test]
fn test_sysline_get_slices() {
    let cap: usize = DATA_STR0.as_bytes().len() + 1;

    let mut sysline1: Sysline = new_sysline1();
    let slices = sysline1.get_slices();

    let mut buffer: Vec<u8> = Vec::<u8>::with_capacity(cap);
    for slice_ in slices.iter() {
        for byte_ in slice_.iter() {
            buffer.push(*byte_);
        }
    }
    let buf_str = String::from_utf8_lossy(&buffer);
    assert_eq!(DATA_STR0, buf_str);
}
