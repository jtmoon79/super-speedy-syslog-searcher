// Readers/summary.rs
//

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
    DateTimeL_Opt,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_ge,
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
    /// `SyslineReader::_syslines_hit`
    pub SyslineReader_syslines_hit: u64,
    /// `SyslineReader::_syslines_miss`
    pub SyslineReader_syslines_miss: u64,
    /// `SyslineReader::_syslines_by_range_hit`
    pub SyslineReader_syslines_by_range_hit: u64,
    /// `SyslineReader::_syslines_by_range_miss`
    pub SyslineReader_syslines_by_range_miss: u64,
    /// `SyslineReader::_syslines_by_range_insert`
    pub SyslineReader_syslines_by_range_insert: u64,
    /// datetime patterns used by `SyslineReader`
    pub SyslineReader_patterns: DateTime_Parse_Datas_vec,
    /// datetime soonest seen (not necessarily reflective of entire file)
    pub SyslineReader_pattern_first: DateTimeL_Opt,
    /// datetime latest seen (not necessarily reflective of entire file)
    pub SyslineReader_pattern_last: DateTimeL_Opt,
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
    /// `LineReader::find_line` `self.lines`
    pub LineReader_lines_hit: u64,
    /// `LineReader::find_line` `self.lines`
    pub LineReader_lines_miss: u64,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub LineReader_find_line_lru_cache_hit: u64,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub LineReader_find_line_lru_cache_miss: u64,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub LineReader_find_line_lru_cache_put: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_hit: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_miss: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_put: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_hit: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_miss: u64,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_insert: u64,
    /// `LineReader::drop_line_ok`
    pub LineReader_drop_line_ok: u64,
    /// `LineReader::drop_line_errors`
    pub LineReader_drop_line_errors: u64,
    /// `SyslineReader::drop_sysline_ok`
    pub SyslineReader_drop_sysline_ok: u64,
    /// `SyslineReader::drop_sysline_errors`
    pub SyslineReader_drop_sysline_errors: u64,
    /// the last IO error as a String, if any
    /// (`Error` does not implement `Clone`)
    pub Error_: Option<String>,
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
        SyslineReader_syslines_hit: u64,
        SyslineReader_syslines_miss: u64,
        SyslineReader_syslines_by_range_hit: u64,
        SyslineReader_syslines_by_range_miss: u64,
        SyslineReader_syslines_by_range_insert: u64,
        SyslineReader_patterns: DateTime_Parse_Datas_vec,
        SyslineReader_pattern_first: DateTimeL_Opt,
        SyslineReader_pattern_last: DateTimeL_Opt,
        SyslineReader_find_sysline_lru_cache_hit: u64,
        SyslineReader_find_sysline_lru_cache_miss: u64,
        SyslineReader_find_sysline_lru_cache_put: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_hit: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_miss: u64,
        SyslineReader_parse_datetime_in_line_lru_cache_put: u64,
        LineReader_lines_hit: u64,
        LineReader_lines_miss: u64,
        LineReader_find_line_lru_cache_hit: u64,
        LineReader_find_line_lru_cache_miss: u64,
        LineReader_find_line_lru_cache_put: u64,
        BlockReader_read_block_lru_cache_hit: u64,
        BlockReader_read_block_lru_cache_miss: u64,
        BlockReader_read_block_lru_cache_put: u64,
        BlockReader_read_blocks_hit: u64,
        BlockReader_read_blocks_miss: u64,
        BlockReader_read_blocks_insert: u64,
        LineReader_drop_line_ok: u64,
        LineReader_drop_line_errors: u64,
        SyslineReader_drop_sysline_ok: u64,
        SyslineReader_drop_sysline_errors: u64,
        Error_: Option<String>,
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
            SyslineReader_syslines_hit,
            SyslineReader_syslines_miss,
            SyslineReader_syslines_by_range_hit,
            SyslineReader_syslines_by_range_miss,
            SyslineReader_syslines_by_range_insert,
            SyslineReader_patterns,
            SyslineReader_pattern_first,
            SyslineReader_pattern_last,
            SyslineReader_find_sysline_lru_cache_hit,
            SyslineReader_find_sysline_lru_cache_miss,
            SyslineReader_find_sysline_lru_cache_put,
            SyslineReader_parse_datetime_in_line_lru_cache_hit,
            SyslineReader_parse_datetime_in_line_lru_cache_miss,
            SyslineReader_parse_datetime_in_line_lru_cache_put,
            LineReader_lines_hit,
            LineReader_lines_miss,
            LineReader_find_line_lru_cache_hit,
            LineReader_find_line_lru_cache_miss,
            LineReader_find_line_lru_cache_put,
            BlockReader_read_block_lru_cache_hit,
            BlockReader_read_block_lru_cache_miss,
            BlockReader_read_block_lru_cache_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
            LineReader_drop_line_ok,
            LineReader_drop_line_errors,
            SyslineReader_drop_sysline_ok,
            SyslineReader_drop_sysline_errors,
            Error_,
        }
    }

    /// return maximum value for hit/miss/insert number.
    /// helpful for format widths
    pub fn max_hit_miss(&self) -> u64 {
        *[
            self.SyslineReader_syslines_hit,
            self.SyslineReader_syslines_miss,
            self.SyslineReader_syslines_by_range_hit,
            self.SyslineReader_syslines_by_range_miss,
            self.SyslineReader_syslines_by_range_insert,
            self.SyslineReader_find_sysline_lru_cache_hit,
            self.SyslineReader_find_sysline_lru_cache_hit,
            self.SyslineReader_find_sysline_lru_cache_hit,
            self.SyslineReader_find_sysline_lru_cache_miss,
            self.SyslineReader_find_sysline_lru_cache_put,
            self.SyslineReader_parse_datetime_in_line_lru_cache_hit,
            self.SyslineReader_parse_datetime_in_line_lru_cache_miss,
            self.SyslineReader_parse_datetime_in_line_lru_cache_put,
            self.LineReader_lines_hit,
            self.LineReader_lines_miss,
            self.LineReader_find_line_lru_cache_hit,
            self.LineReader_find_line_lru_cache_miss,
            self.LineReader_find_line_lru_cache_put,
            self.BlockReader_read_block_lru_cache_hit,
            self.BlockReader_read_block_lru_cache_miss,
            self.BlockReader_read_block_lru_cache_put,
            self.BlockReader_read_blocks_hit,
            self.BlockReader_read_blocks_miss,
            self.BlockReader_read_blocks_insert,
        ].iter().max().unwrap()
    }

    /// return maximum value for drop number.
    /// helpful for format widths
    pub fn max_drop(&self) -> u64 {
        *[
            self.LineReader_drop_line_ok,
            self.LineReader_drop_line_errors,
            self.SyslineReader_drop_sysline_ok,
            self.SyslineReader_drop_sysline_errors,
        ].iter().max().unwrap()
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
            // Summary::default()
            FileType::FILE_UNSET_ => {
                f.debug_struct("")
                .finish()
            },
            _ => {
                unimplemented!("FileType {:?} not implemented", self.filetype);
            },
        }

    }
}

pub type Summary_Opt = Option<Summary>;
