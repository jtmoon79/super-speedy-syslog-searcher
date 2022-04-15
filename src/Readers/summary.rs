// Readers/summary.rs

use std::fmt;

use crate::Readers::blockreader::{
    BlockSz,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    //assert_lt,
    assert_ge,
    //assert_gt,
    //debug_assert_le,
    //debug_assert_lt,
    //debug_assert_ge,
    //debug_assert_gt,
};


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// statistics to print about `SyslineReader` activity
#[derive(Copy, Clone, Default)]
pub struct Summary {
    /// count of bytes stored by `BlockReader`
    pub bytes: u64,
    /// count of bytes in file
    pub bytes_total: u64,
    /// count of `Block`s read by `BlockReader`
    pub blocks: u64,
    /// count of `Block`s in file
    pub blocks_total: u64,
    /// `BlockSz` of `BlockReader`
    pub blocksz: BlockSz,
    /// count of `Lines` processed by `LineReader`
    pub lines: u64,
    /// count of `Syslines` processed by `SyslineReader`
    pub syslines: u64,
    /// `SyslineReader::_parse_datetime_in_line_lru_cache_hit`
    pub _parse_datetime_in_line_lru_cache_hit: u64,
    /// `SyslineReader::_parse_datetime_in_line_lru_cache_miss`
    pub _parse_datetime_in_line_lru_cache_miss: u64,
    /// `LineReader::_find_line_lru_cache_hit`
    pub _find_line_lru_cache_hit: u64,
    /// `LineReader::_find_line_lru_cache_miss`
    pub _find_line_lru_cache_miss: u64,
    /// `BlockReader`
    pub _read_block_cache_lru_hit: u32,
    pub _read_block_cache_lru_miss: u32,
    pub _read_blocks_hit: u32,
    pub _read_blocks_miss: u32,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bytes: u64,
        bytes_total: u64,
        blocks: u64,
        blocks_total: u64,
        blocksz: BlockSz,
        lines: u64,
        syslines: u64,
        _parse_datetime_in_line_lru_cache_hit: u64,
        _parse_datetime_in_line_lru_cache_miss: u64,
        _find_line_lru_cache_hit: u64,
        _find_line_lru_cache_miss: u64,
        _read_block_cache_lru_hit: u32,
        _read_block_cache_lru_miss: u32,
        _read_blocks_hit: u32,
        _read_blocks_miss: u32,
    ) -> Summary {
        // some sanity checks
        assert_ge!(bytes, blocks, "There is less bytes than Blocks");
        assert_ge!(bytes, lines, "There is less bytes than Lines");
        assert_ge!(bytes, lines, "There is less bytes than Syslines");
        assert_ge!(blocksz, BLOCKSZ_MIN, "blocksz too small");
        assert_le!(blocksz, BLOCKSZ_MAX, "blocksz too big");
        assert_ge!(lines, syslines, "There is less Lines than Syslines");
        Summary {
            bytes,
            bytes_total,
            blocks,
            blocks_total,
            blocksz,
            lines,
            syslines,
            _parse_datetime_in_line_lru_cache_hit,
            _parse_datetime_in_line_lru_cache_miss,
            _find_line_lru_cache_hit,
            _find_line_lru_cache_miss,
            _read_block_cache_lru_hit,
            _read_block_cache_lru_miss,
            _read_blocks_hit,
            _read_blocks_miss,
        }
    }
}

impl fmt::Debug for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("")
            .field("bytes", &self.bytes)
            .field("bytes total", &self.bytes_total)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field("blocks", &self.blocks)
            .field("blocks total", &self.blocks_total)
            .field("blocksz", &format_args!("{0} (0x{0:X})", &self.blocksz))
            .finish()
    }
}

pub type Summary_Opt = Option<Summary>;
