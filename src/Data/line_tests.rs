// Data/line_tests.rs
//

use super::line::{
    Line,
    LinePart,
};

use crate::Readers::blockreader::{
    Block,
    BlockP,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_linepart_new1() {
    let data = vec![32 as u8, 32 as u8, 32 as u8, 32 as u8];
    let block: Block = Block::from(data);
    let len = block.len();
    let blockp: BlockP = BlockP::new(block);
    let _lp = LinePart::new(blockp, 0, 1, 0, 0, len as u64);
}

#[test]
fn test_line_new1() {
    let _line = Line::new();
}

// TODO: [2022/06/02] needs more tests of `Data/line.rs`
