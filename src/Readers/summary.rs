// Readers/summary.rs

use std::fmt;

use crate::common::{
    FileType,
};

use crate::Readers::blockreader::{
    BlockSz,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
};

use crate::Data::datetime::{
    DateTime_Parse_Datas_vec,
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
#[derive(Clone, Default)]
pub struct Summary {
    pub filetype: FileType,
    /// count of bytes stored by `BlockReader`
    pub BlockReader_bytes: u64,
    /// count of bytes in file
    pub BlockReader_bytes_total: u64,
    /// count of `Block`s read by `BlockReader`
    pub BlockReader_blocks: u64,
    /// count of `Block`s in file
    pub BlockReader_blocks_total: u64,
    /// `BlockSz` of `BlockReader`
    pub BlockReader_blocksz: BlockSz,
    /// `filesz()` of file, size of file on disk
    pub BlockReader_filesz: u64,
    /// `filesz()` of file, for compressed files this is the uncompressed filesz
    pub BlockReader_filesz_actual: u64,
    /// count of `Lines` processed by `LineReader`
    pub LineReader_lines: u64,
    /// count of `Syslines` processed by `SyslineReader`
    pub SyslineReader_syslines: u64,
    /// `SyslineReader::_syslines_by_range_hit`
    pub SyslineReader_syslines_by_range_hit: u64,
    /// `SyslineReader::_syslines_by_range_miss`
    pub SyslineReader_syslines_by_range_miss: u64,
    /// `SyslineReader::_syslines_by_range_insert`
    pub SyslineReader_syslines_by_range_insert: u64,
    /// datetime patterns used by `SyslineReader`
    pub SyslineReader_patterns: DateTime_Parse_Datas_vec,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_hit: u64,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_miss: u64,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_put: u64,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_hit: u64,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_miss: u64,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_put: u64,
    /// `LineReader::find_line`
    pub LineReader_find_line_lru_cache_hit: u64,
    /// `LineReader::find_line`
    pub LineReader_find_line_lru_cache_miss: u64,
    /// `LineReader::find_line`
    pub LineReader_find_line_lru_cache_put: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_hit: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_miss: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_put: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_hit: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_miss: u32,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_insert: u32,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        filetype: FileType,
        BlockReader_bytes: u64,
        BlockReader_bytes_total: u64,
        BlockReader_blocks: u64,
        BlockReader_blocks_total: u64,
        BlockReader_blocksz: BlockSz,
        BlockReader_filesz: u64,
        BlockReader_filesz_actual: u64,
        LineReader_lines: u64,
        SyslineReader_syslines: u64,
        SyslineReader_syslines_by_range_hit: u64,
        SyslineReader_syslines_by_range_miss: u64,
        SyslineReader_syslines_by_range_insert: u64,
        SyslineReader_patterns: DateTime_Parse_Datas_vec,
        SyslineReader_find_sysline_lru_cache_hit: u64,
        SyslineReader_find_sysline_lru_cache_miss: u64,
        SyslineReader_find_sysline_lru_cache_put: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_hit: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_miss: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_put: u64,
        LineReader_find_line_lru_cache_hit: u64,
        LineReader_find_line_lru_cache_miss: u64,
        LineReader_find_line_lru_cache_put: u64,
        BlockReader_read_block_lru_cache_hit: u32,
        BlockReader_read_block_lru_cache_miss: u32,
        BlockReader_read_block_lru_cache_put: u32,
        BlockReader_read_blocks_hit: u32,
        BlockReader_read_blocks_miss: u32,
        BlockReader_read_blocks_insert: u32,
    ) -> Summary {
        // some sanity checks
        assert_ge!(BlockReader_bytes, BlockReader_blocks, "There is less bytes than Blocks");
        assert_ge!(BlockReader_bytes, LineReader_lines, "There is less bytes than Lines");
        assert_ge!(BlockReader_bytes, SyslineReader_syslines, "There is less bytes than Syslines");
        assert_ge!(BlockReader_blocksz, BLOCKSZ_MIN, "blocksz too small");
        assert_le!(BlockReader_blocksz, BLOCKSZ_MAX, "blocksz too big");
        assert_ge!(LineReader_lines, SyslineReader_syslines, "There is less Lines than Syslines");
        Summary {
            filetype,
            BlockReader_bytes,
            BlockReader_bytes_total,
            BlockReader_blocks,
            BlockReader_blocks_total,
            BlockReader_blocksz,
            BlockReader_filesz,
            BlockReader_filesz_actual,
            LineReader_lines,
            SyslineReader_syslines,
            SyslineReader_syslines_by_range_hit,
            SyslineReader_syslines_by_range_miss,
            SyslineReader_syslines_by_range_insert,
            SyslineReader_patterns,
            SyslineReader_find_sysline_lru_cache_hit,
            SyslineReader_find_sysline_lru_cache_miss,
            SyslineReader_find_sysline_lru_cache_put,
            SyslineReader_parse_datetime_in_line_lru_cache_hit,
            SyslineReader_parse_datetime_in_line_lru_cache_miss,
            SyslineReader_parse_datetime_in_line_lru_cache_put,
            LineReader_find_line_lru_cache_hit,
            LineReader_find_line_lru_cache_miss,
            LineReader_find_line_lru_cache_put,
            BlockReader_read_block_lru_cache_hit,
            BlockReader_read_block_lru_cache_miss,
            BlockReader_read_block_lru_cache_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
        }
    }
}

impl fmt::Debug for Summary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.filetype {
            FileType::FILE => {
                f.debug_struct("")
                .field("bytes", &self.BlockReader_bytes)
                .field("bytes total", &self.BlockReader_bytes_total)
                .field("lines", &self.LineReader_lines)
                .field("syslines", &self.SyslineReader_syslines)
                .field("blocks", &self.BlockReader_blocks)
                .field("blocks total", &self.BlockReader_blocks_total)
                .field("blocksz", &format_args!("{0} (0x{0:X})", &self.BlockReader_blocksz))
                .field("filesz", &format_args!("{0} (0x{0:X})", &self.BlockReader_filesz))
                .finish()
            },
            FileType::FILE_GZ => {
                f.debug_struct("")
                .field("bytes", &self.BlockReader_bytes)
                .field("bytes total", &self.BlockReader_bytes_total)
                .field("lines", &self.LineReader_lines)
                .field("syslines", &self.SyslineReader_syslines)
                .field("blocks", &self.BlockReader_blocks)
                .field("blocks total", &self.BlockReader_blocks_total)
                .field("blocksz", &format_args!("{0} (0x{0:X})", &self.BlockReader_blocksz))
                .field("filesz uncompressed", &format_args!("{0} (0x{0:X})", &self.BlockReader_filesz_actual))
                .field("filesz compressed", &format_args!("{0} (0x{0:X})", &self.BlockReader_filesz))
                .finish()
            },
            _ => {
                unimplemented!("FileType {:?} not implemented", self.filetype);
            },
        }

    }
}

pub type Summary_Opt = Option<Summary>;
