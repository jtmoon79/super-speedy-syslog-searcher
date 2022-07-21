// Readers/summary.rs
//

#![allow(non_snake_case)]

use crate::common::{
    Count,
    FileType,
    FileSz,
};

use crate::Data::datetime::{
    DateTimeL_Opt,
};

use crate::Readers::blockreader::{
    BlockSz,
    BLOCKSZ_MAX,
    BLOCKSZ_MIN,
};

use crate::Readers::syslinereader::{
    DateTime_Pattern_Counts,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_ge,
};

use std::fmt;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Accumulated statistics to print about activity of a `SyslineReader` and it's underlying
/// `LineReader` and it's underlying `BlockReader`.
///
/// For CLI option `--summary`
#[derive(Clone, Default)]
pub struct Summary {
    pub filetype: FileType,
    /// count of bytes stored by `BlockReader`
    pub BlockReader_bytes: Count,
    /// count of bytes in file
    pub BlockReader_bytes_total: FileSz,
    /// count of `Block`s read by `BlockReader`
    pub BlockReader_blocks: Count,
    /// count of `Block`s in file
    pub BlockReader_blocks_total: Count,
    /// `BlockSz` of `BlockReader`
    pub BlockReader_blocksz: BlockSz,
    /// `filesz()` of file, size of file on disk
    pub BlockReader_filesz: FileSz,
    /// `filesz()` of file, for compressed files this is the uncompressed filesz
    pub BlockReader_filesz_actual: FileSz,
    /// count of `Lines` processed by `LineReader`
    pub LineReader_lines: Count,
    /// count of `Syslines` processed by `SyslineReader`
    pub SyslineReader_syslines: Count,
    /// `SyslineReader::_syslines_hit`
    pub SyslineReader_syslines_hit: Count,
    /// `SyslineReader::_syslines_miss`
    pub SyslineReader_syslines_miss: Count,
    /// `SyslineReader::_syslines_by_range_hit`
    pub SyslineReader_syslines_by_range_hit: Count,
    /// `SyslineReader::_syslines_by_range_miss`
    pub SyslineReader_syslines_by_range_miss: Count,
    /// `SyslineReader::_syslines_by_range_put`
    pub SyslineReader_syslines_by_range_put: Count,
    /// datetime patterns used by `SyslineReader`
    /// TODO: change name to whatever is decided on
    pub SyslineReader_patterns: DateTime_Pattern_Counts,
    /// datetime soonest seen (not necessarily reflective of entire file)
    pub SyslineReader_pattern_first: DateTimeL_Opt,
    /// datetime latest seen (not necessarily reflective of entire file)
    pub SyslineReader_pattern_last: DateTimeL_Opt,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_hit: Count,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_miss: Count,
    /// `SyslineReader::find_sysline`
    pub SyslineReader_find_sysline_lru_cache_put: Count,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_hit: Count,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_miss: Count,
    /// `SyslineReader::parse_datetime_in_line`
    pub SyslineReader_parse_datetime_in_line_lru_cache_put: Count,
    /// `SyslineReader::get_boxptrs_singleptr`
    pub SyslineReader_get_boxptrs_singleptr: Count,
    /// `SyslineReader::get_boxptrs_doubleptr`
    pub SyslineReader_get_boxptrs_doubleptr: Count,
    /// `SyslineReader::get_boxptrs_multiptr`
    pub SyslineReader_get_boxptrs_multiptr: Count,
    /// `LineReader::find_line` `self.lines`
    pub LineReader_lines_hit: Count,
    /// `LineReader::find_line` `self.lines`
    pub LineReader_lines_miss: Count,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub LineReader_find_line_lru_cache_hit: Count,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub LineReader_find_line_lru_cache_miss: Count,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub LineReader_find_line_lru_cache_put: Count,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_hit: Count,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_miss: Count,
    /// `BlockReader::read_block`
    pub BlockReader_read_block_lru_cache_put: Count,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_hit: Count,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_miss: Count,
    /// `BlockReader::read_block`
    pub BlockReader_read_blocks_put: Count,
    /// `LineReader::drop_line_ok`
    pub LineReader_drop_line_ok: Count,
    /// `LineReader::drop_line_errors`
    pub LineReader_drop_line_errors: Count,
    /// `SyslineReader::drop_sysline_ok`
    pub SyslineReader_drop_sysline_ok: Count,
    /// `SyslineReader::drop_sysline_errors`
    pub SyslineReader_drop_sysline_errors: Count,
    /// the last IO error as a String, if any
    /// (`Error` does not implement `Clone`)
    pub Error_: Option<String>,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        filetype: FileType,
        BlockReader_bytes: Count,
        BlockReader_bytes_total: FileSz,
        BlockReader_blocks: Count,
        BlockReader_blocks_total: Count,
        BlockReader_blocksz: BlockSz,
        BlockReader_filesz: FileSz,
        BlockReader_filesz_actual: FileSz,
        LineReader_lines: Count,
        SyslineReader_syslines: Count,
        SyslineReader_syslines_hit: Count,
        SyslineReader_syslines_miss: Count,
        SyslineReader_syslines_by_range_hit: Count,
        SyslineReader_syslines_by_range_miss: Count,
        SyslineReader_syslines_by_range_put: Count,
        // TODO: change name to `SyslineReader_pattern_counts` or whatever var name is decided on
        SyslineReader_patterns: DateTime_Pattern_Counts,
        SyslineReader_pattern_first: DateTimeL_Opt,
        SyslineReader_pattern_last: DateTimeL_Opt,
        SyslineReader_find_sysline_lru_cache_hit: Count,
        SyslineReader_find_sysline_lru_cache_miss: Count,
        SyslineReader_find_sysline_lru_cache_put: Count,
        SyslineReader_parse_datetime_in_line_lru_cache_hit: Count,
        SyslineReader_parse_datetime_in_line_lru_cache_miss: Count,
        SyslineReader_parse_datetime_in_line_lru_cache_put: Count,
        SyslineReader_get_boxptrs_singleptr: Count,
        SyslineReader_get_boxptrs_doubleptr: Count,
        SyslineReader_get_boxptrs_multiptr: Count,
        LineReader_lines_hit: Count,
        LineReader_lines_miss: Count,
        LineReader_find_line_lru_cache_hit: Count,
        LineReader_find_line_lru_cache_miss: Count,
        LineReader_find_line_lru_cache_put: Count,
        BlockReader_read_block_lru_cache_hit: Count,
        BlockReader_read_block_lru_cache_miss: Count,
        BlockReader_read_block_lru_cache_put: Count,
        BlockReader_read_blocks_hit: Count,
        BlockReader_read_blocks_miss: Count,
        BlockReader_read_blocks_put: Count,
        LineReader_drop_line_ok: Count,
        LineReader_drop_line_errors: Count,
        SyslineReader_drop_sysline_ok: Count,
        SyslineReader_drop_sysline_errors: Count,
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
            SyslineReader_syslines_by_range_put,
            SyslineReader_patterns,
            SyslineReader_pattern_first,
            SyslineReader_pattern_last,
            SyslineReader_find_sysline_lru_cache_hit,
            SyslineReader_find_sysline_lru_cache_miss,
            SyslineReader_find_sysline_lru_cache_put,
            SyslineReader_parse_datetime_in_line_lru_cache_hit,
            SyslineReader_parse_datetime_in_line_lru_cache_miss,
            SyslineReader_parse_datetime_in_line_lru_cache_put,
            SyslineReader_get_boxptrs_singleptr,
            SyslineReader_get_boxptrs_doubleptr,
            SyslineReader_get_boxptrs_multiptr,
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
            BlockReader_read_blocks_put,
            LineReader_drop_line_ok,
            LineReader_drop_line_errors,
            SyslineReader_drop_sysline_ok,
            SyslineReader_drop_sysline_errors,
            Error_,
        }
    }

    /// return maximum value for hit/miss/insert number.
    /// helpful for format widths
    pub fn max_hit_miss(&self) -> Count {
        *[
            self.SyslineReader_syslines_hit,
            self.SyslineReader_syslines_miss,
            self.SyslineReader_syslines_by_range_hit,
            self.SyslineReader_syslines_by_range_miss,
            self.SyslineReader_syslines_by_range_put,
            self.SyslineReader_find_sysline_lru_cache_hit,
            self.SyslineReader_find_sysline_lru_cache_hit,
            self.SyslineReader_find_sysline_lru_cache_hit,
            self.SyslineReader_find_sysline_lru_cache_miss,
            self.SyslineReader_find_sysline_lru_cache_put,
            self.SyslineReader_parse_datetime_in_line_lru_cache_hit,
            self.SyslineReader_parse_datetime_in_line_lru_cache_miss,
            self.SyslineReader_parse_datetime_in_line_lru_cache_put,
            self.SyslineReader_get_boxptrs_singleptr,
            self.SyslineReader_get_boxptrs_doubleptr,
            self.SyslineReader_get_boxptrs_multiptr,
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
            self.BlockReader_read_blocks_put,
        ].iter().max().unwrap()
    }

    /// return maximum value for drop number.
    /// helpful for format widths
    pub fn max_drop(&self) -> Count {
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
            FileType::FILE_TAR
            | FileType::FILE => {
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
            FileType::FILE_GZ 
            | FileType::FILE_XZ => {
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
                unimplemented!("FileType {:?} not implemented for Summary fmt::Debug", self.filetype);
            },
        }

    }
}

pub type Summary_Opt = Option<Summary>;
