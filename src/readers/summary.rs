// src/readers/summary.rs

//! Implements `Summary` statistics tracking struct.

#![allow(non_snake_case)]

use crate::common::{Count, FileSz, FileType};

use crate::data::datetime::{DateTimeLOpt, Year};

use crate::readers::blockreader::{BlockSz, BLOCKSZ_MAX, BLOCKSZ_MIN};

use crate::readers::syslinereader::DateTimePatternCounts;

#[allow(unused_imports)]
use crate::debug::printers::{dp_err, dp_wrn, p_err, p_wrn};

extern crate more_asserts;
use more_asserts::{debug_assert_ge, debug_assert_le};

extern crate si_trace_print;
#[allow(unused_imports)]
use si_trace_print::{dpfn, dpfo, dpfx, dpfñ, dpn, dpo, dpx, dpñ};

use std::fmt;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Summary
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Accumulated statistics to print about activity of a `SyslineReader`
/// and it's underlying `LineReader` and it's underlying `BlockReader`.
///
/// For CLI option `--summary`.
#[derive(Clone, Default)]
pub struct Summary {
    /// the `FileType`
    pub filetype: FileType,
    /// `Count` of bytes stored by `BlockReader`
    pub BlockReader_bytes: Count,
    /// count of bytes in file
    pub BlockReader_bytes_total: FileSz,
    /// `Count` of `Block`s read by `BlockReader`
    pub BlockReader_blocks: Count,
    /// `Count` of `Block`s in file
    pub BlockReader_blocks_total: Count,
    /// `BlockSz` of `BlockReader`
    pub BlockReader_blocksz: BlockSz,
    /// `filesz()` of file, size of file on disk
    pub BlockReader_filesz: FileSz,
    /// `filesz()` of file, for compressed files this is the uncompressed filesz
    pub BlockReader_filesz_actual: FileSz,
    /// `Count` of `Lines` processed by `LineReader`
    pub LineReader_lines: Count,
    /// "high watermark" of Lines stored in `LineReader.lines`
    pub LineReader_lines_stored_highest: usize,
    /// `Count` of `Syslines` processed by `SyslineReader`
    pub SyslineReader_syslines: Count,
    /// "high watermark"` of `Sysline`s stored by `SyslineReader.syslines`
    pub SyslineReader_syslines_stored_highest: usize,
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
    pub SyslineReader_patterns: DateTimePatternCounts,
    /// datetime soonest seen (not necessarily reflective of entire file)
    pub SyslineReader_pattern_first: DateTimeLOpt,
    /// datetime latest seen (not necessarily reflective of entire file)
    pub SyslineReader_pattern_last: DateTimeLOpt,
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
    /// `LineReader::lines_hits` for `self.lines`
    pub LineReader_lines_hits: Count,
    /// `LineReader::lines_miss` for `self.lines`
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
    /// `BlockReader::blocks_highest`
    pub BlockReader_blocks_highest: usize,
    /// `BlockReader::blocks_dropped_ok`
    pub BlockReader_blocks_dropped_ok: Count,
    /// `BlockReader::blocks_dropped_err`
    pub BlockReader_blocks_dropped_err: Count,
    /// `LineReader::drop_line_ok`
    pub LineReader_drop_line_ok: Count,
    /// `LineReader::drop_line_errors`
    pub LineReader_drop_line_errors: Count,
    /// `SyslineReader::drop_sysline_ok`
    pub SyslineReader_drop_sysline_ok: Count,
    /// `SyslineReader::drop_sysline_errors`
    pub SyslineReader_drop_sysline_errors: Count,
    /// `SyslogProcessor::missing_year`
    pub SyslogProcessor_missing_year: Option<Year>,
    /// The last IO error as a String, if any
    // XXX: `Error` does not implement `Clone`, see https://doc.rust-lang.org/std/io/struct.Error.html
    pub Error_: Option<String>,
}

impl Summary {

    /// Create a new `Summary`
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
        LineReader_lines_stored_highest: usize,
        SyslineReader_syslines: Count,
        SyslineReader_syslines_stored_highest: usize,
        SyslineReader_syslines_hit: Count,
        SyslineReader_syslines_miss: Count,
        SyslineReader_syslines_by_range_hit: Count,
        SyslineReader_syslines_by_range_miss: Count,
        SyslineReader_syslines_by_range_put: Count,
        SyslineReader_patterns: DateTimePatternCounts,
        SyslineReader_pattern_first: DateTimeLOpt,
        SyslineReader_pattern_last: DateTimeLOpt,
        SyslineReader_find_sysline_lru_cache_hit: Count,
        SyslineReader_find_sysline_lru_cache_miss: Count,
        SyslineReader_find_sysline_lru_cache_put: Count,
        SyslineReader_parse_datetime_in_line_lru_cache_hit: Count,
        SyslineReader_parse_datetime_in_line_lru_cache_miss: Count,
        SyslineReader_parse_datetime_in_line_lru_cache_put: Count,
        SyslineReader_get_boxptrs_singleptr: Count,
        SyslineReader_get_boxptrs_doubleptr: Count,
        SyslineReader_get_boxptrs_multiptr: Count,
        LineReader_lines_hits: Count,
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
        BlockReader_blocks_highest: usize,
        BlockReader_blocks_dropped_ok: Count,
        BlockReader_blocks_dropped_err: Count,
        LineReader_drop_line_ok: Count,
        LineReader_drop_line_errors: Count,
        SyslineReader_drop_sysline_ok: Count,
        SyslineReader_drop_sysline_errors: Count,
        SyslogProcessor_missing_year: Option<Year>,
        Error_: Option<String>,
    ) -> Summary {
        // some sanity checks
        debug_assert_ge!(BlockReader_bytes, BlockReader_blocks, "There is less bytes than Blocks");
        debug_assert_ge!(BlockReader_bytes, LineReader_lines, "There is less bytes than Lines");
        debug_assert_ge!(BlockReader_bytes, SyslineReader_syslines, "There is less bytes than Syslines");
        debug_assert_ge!(BlockReader_blocksz, BLOCKSZ_MIN, "blocksz too small");
        debug_assert_le!(BlockReader_blocksz, BLOCKSZ_MAX, "blocksz too big");
        // XXX: in case of a file without datetime stamp year, syslines may be reprocessed.
        //      the count of syslines processed may reflect reprocoessing the same line in the file,
        //      leading to a `SyslineReader_syslines` that is more than `LineReader_lines`.
        //      See `syslogprocessor.process_missing_year()`.
        //debug_assert_ge!(LineReader_lines, SyslineReader_syslines, "There is less Lines than Syslines");
        if LineReader_lines < SyslineReader_syslines {
            dp_wrn!("There is less Lines {} than Syslines {}", LineReader_lines, SyslineReader_syslines);
        }
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
            LineReader_lines_stored_highest,
            SyslineReader_syslines,
            SyslineReader_syslines_stored_highest,
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
            LineReader_lines_hits,
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
            BlockReader_blocks_highest,
            BlockReader_blocks_dropped_ok,
            BlockReader_blocks_dropped_err,
            LineReader_drop_line_ok,
            LineReader_drop_line_errors,
            SyslineReader_drop_sysline_ok,
            SyslineReader_drop_sysline_errors,
            SyslogProcessor_missing_year,
            Error_,
        }
    }

    /// Create a new `Summary` with limited known data.
    /// Useful for files that failed to process due to errors
    /// (e.g. PermissionDenied, etc.).
    #[allow(clippy::too_many_arguments)]
    pub fn new_failed(
        filetype: FileType,
        BlockReader_blocksz: BlockSz,
        Error_: Option<String>,
    ) -> Summary {
        // some sanity checks
        debug_assert_ge!(BlockReader_blocksz, BLOCKSZ_MIN, "blocksz too small");
        debug_assert_le!(BlockReader_blocksz, BLOCKSZ_MAX, "blocksz too big");
        let mut summary = Summary::default();
        summary.filetype = filetype;
        summary.BlockReader_blocksz = BlockReader_blocksz;
        summary.Error_ = Error_;

        summary
    }

    /// Return maximum value for hit/miss/insert number.
    ///
    /// Helpful to format teriminal column widths.
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
            self.LineReader_lines_hits,
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
        ]
        .iter()
        .max()
        .unwrap()
    }

    /// Return maximum value for drop number.
    ///
    /// Helpful to format teriminal column widths.
    pub fn max_drop(&self) -> Count {
        *[
            self.LineReader_drop_line_ok,
            self.LineReader_drop_line_errors,
            self.SyslineReader_drop_sysline_ok,
            self.SyslineReader_drop_sysline_errors,
        ]
        .iter()
        .max()
        .unwrap()
    }
}

impl fmt::Debug for Summary {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self.filetype {
            FileType::Tar | FileType::File => f
                .debug_struct("")
                .field("bytes", &self.BlockReader_bytes)
                .field("bytes total", &self.BlockReader_bytes_total)
                .field("lines", &self.LineReader_lines)
                .field("lines stored highest", &self.LineReader_lines_stored_highest)
                .field("syslines", &self.SyslineReader_syslines)
                .field("syslines stored highest", &self.SyslineReader_syslines_stored_highest)
                .field("blocks", &self.BlockReader_blocks)
                .field("blocks total", &self.BlockReader_blocks_total)
                .field("blocks stored highest", &self.BlockReader_blocks_highest)
                .field("blocksz", &format_args!("{0} (0x{0:X})", &self.BlockReader_blocksz))
                .field("filesz", &format_args!("{0} (0x{0:X})", &self.BlockReader_filesz))
                .finish(),
            FileType::Gz | FileType::Xz => f
                .debug_struct("")
                .field("bytes", &self.BlockReader_bytes)
                .field("bytes total", &self.BlockReader_bytes_total)
                .field("lines", &self.LineReader_lines)
                .field("lines stored highest", &self.LineReader_lines_stored_highest)
                .field("syslines", &self.SyslineReader_syslines)
                .field("syslines stored highest", &self.SyslineReader_syslines_stored_highest)
                .field("blocks", &self.BlockReader_blocks)
                .field("blocks total", &self.BlockReader_blocks_total)
                .field("blocks stored high", &self.BlockReader_blocks_highest)
                .field("blocksz", &format_args!("{0} (0x{0:X})", &self.BlockReader_blocksz))
                .field("filesz uncompressed", &format_args!("{0} (0x{0:X})", &self.BlockReader_filesz_actual))
                .field("filesz compressed", &format_args!("{0} (0x{0:X})", &self.BlockReader_filesz))
                .finish(),
            // Summary::default()
            FileType::Unset => f.debug_struct("").finish(),
            _ => {
                unimplemented!("FileType {:?} not implemented for Summary fmt::Debug", self.filetype);
            }
        }
    }
}

/// Optional `Summary`.
pub type SummaryOpt = Option<Summary>;
