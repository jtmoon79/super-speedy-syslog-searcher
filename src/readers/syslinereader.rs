// src/readers/syslinereader.rs
// … ≤ ≥

//! Implements `SyslineReader`, the driver of deriving [`Sysline`]s using a
//! [`LineReader`].
//!
//! [`Sysline`]: crate::data::sysline::Sysline
//! [`LineReader`]: crate::readers::linereader::LineReader

use crate::common::{Bytes, CharSz, Count, FPath, FileOffset, FileSz, FileType, ResultS3};
use crate::data::line::{Line, LineIndex, LineP, LinePartPtrs};
use crate::data::sysline::{Sysline, SyslineP};
use crate::data::datetime::{
    bytes_to_regex_to_datetime,
    dt_after_or_before,
    dt_pass_filters,
    DateTimeL,
    DateTimeLOpt,
    DateTimeParseInstr,
    DateTimeParseInstrsIndex,
    FixedOffset,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    SystemTime,
    Year,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
    slice_contains_X_2,
    slice_contains_D2,
};
#[cfg(any(debug_assertions, test))]
use crate::data::datetime::{
    DateTimeRegex, DATETIME_PARSE_DATAS_REGEX_VEC
};
#[allow(unused_imports)]
use crate::debug::printers::{de_err, de_wrn, e_err, e_wrn};
use crate::readers::blockreader::{BlockIndex, BlockOffset, BlockSz};
use crate::readers::linereader::{LineReader, ResultS3LineFind};

#[cfg(test)]
use std::collections::HashSet;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use ::itertools::Itertools; // brings in `sorted_by`
use ::lru::LruCache;
use ::mime_guess::MimeGuess;
use ::more_asserts::{assert_le, debug_assert_le, debug_assert_lt, debug_assert_gt};
use ::rangemap::RangeMap;
#[allow(unused_imports)]
use ::si_trace_print::{def1n, def1o, def1x, def1ñ, defn, defo, defx, defñ, den, deo, dex, deñ};
use ::static_assertions::const_assert;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime typing, strings, and formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// `Count` of datetime format strings used.
///
/// Key is index into global [`DATETIME_PARSE_DATAS`]
/// (and [`DATETIME_PARSE_DATAS_REGEX_VEC`]).
///
/// Value is `Count` of use of those "pattern rules" to find datetimes in a
/// `Line`.
///
/// [`DATETIME_PARSE_DATAS`]: crate::data::datetime::DATETIME_PARSE_DATAS
/// [`DATETIME_PARSE_DATAS_REGEX_VEC`]: static@crate::data::datetime::DATETIME_PARSE_DATAS_REGEX_VEC
pub type DateTimePatternCounts = BTreeMap<DateTimeParseInstrsIndex, Count>;

/// Collection of `DateTimeParseInstrsIndex`.
pub type DateTimeParseDatasIndexes = BTreeSet<DateTimeParseInstrsIndex>;

/// Data returned by `SyslineReader::find_datetime_in_line` and
/// `SyslineReader::parse_datetime_in_line`.
///
/// - datetime substring index begin
/// - datetime substring index end
/// - the datetime found
/// - index into global `DATETIME_PARSE_DATAS_REGEX_VEC` and `DATETIME_PARSE_DATAS_REGEX_VEC` for the
///   "pattern rules" used to find the datetime.
pub type FindDateTimeData = (LineIndex, LineIndex, DateTimeL, DateTimeParseInstrsIndex);

/// Return type for `SyslineReader::find_datetime_in_line`.
pub type ResultFindDateTime = Result<FindDateTimeData>;

/// Return type for `SyslineReader::parse_datetime_in_line`.
pub type ResultParseDateTime = Result<FindDateTimeData>;

#[cfg(test)]
pub type SetDroppedSyslines = HashSet<FileOffset>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// [`Sysline`] Searching result.
///
/// [`Sysline`]: crate::data::sysline::Sysline
// TODO: rename `ResultS3SyslineFind` with `ResultSyslineFind`
pub type ResultS3SyslineFind = ResultS3<(FileOffset, SyslineP), Error>;

/// Storage for `Sysline`.
///
/// [`Sysline`]: crate::data::sysline::Sysline
pub type Syslines = BTreeMap<FileOffset, SyslineP>;

type SyslineRange = std::ops::Range<FileOffset>;

/// Range map where key is sysline begin to end
/// `\[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()\]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`).
///
/// Use the value to lookup associated [`Syslines`] map.
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;

/// [LRU cache] internally by `SyslineReader`.
///
/// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
type SyslinesLRUCache = LruCache<FileOffset, ResultS3SyslineFind>;

/// [LRU cache] internally by `SyslineReader`.
///
/// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
type LineParsedCache = LruCache<FileOffset, FindDateTimeData>;

/// A specialized reader that uses [`LineReader`] to find [`Sysline`s] in a file.
/// A `SyslineReader` has specialized knowledge of associating a parsed
/// datetime to a sequence of lines, and then creating a `Sysline`.
///
/// A `SyslineReader` does some `[u8]` to `char` interpretation.
///
/// A `SyslineReader` stores past lookups of data (calls to functions
/// `find_sysline*`).
///
/// _XXX: not a rust "Reader"; does not implement trait [`Read`]._
///
/// [`LineReader`]: crate::readers::linereader::LineReader
/// [`Sysline`s]: crate::data::sysline::Sysline
/// [`Read`]: std::io::Read
pub struct SyslineReader {
    /// Internal `LineReader` instance.
    pub(super) linereader: LineReader,
    /// Syslines keyed by `Sysline.fileoffset_begin`.
    pub(super) syslines: Syslines,
    /// `Count` of Syslines processed.
    syslines_count: Count,
    /// "high watermark" of Syslines stored in `self.syslines` at one time
    syslines_stored_highest: usize,
    /// Internal stats `Count` for `self.find_sysline()` use of `self.syslines`.
    pub(super) syslines_hit: Count,
    /// Internal stats `Count` for `self.find_sysline()` use of `self.syslines`.
    pub(super) syslines_miss: Count,
    /// Syslines fileoffset by sysline fileoffset range,
    /// i.e. `\[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`.
    /// The stored value can be used as a key for `self.syslines`.
    syslines_by_range: SyslinesRangeMap,
    /// `Count` of `self.syslines_by_range` lookup hit.
    pub(super) syslines_by_range_hit: Count,
    /// `Count` of `self.syslines_by_range` lookup miss.
    pub(super) syslines_by_range_miss: Count,
    /// `Count` of `self.syslines_by_range.insert`.
    pub(super) syslines_by_range_put: Count,
    /// First (soonest) processed [`DateTimeL`] (not necessarily printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    // TODO: [2022/07/27] cost-savings: save the ref
    // TODO: [2023/03/22] change behavior to be "first printed" instead of "first processed"
    pub(super) dt_first: DateTimeLOpt,
    pub(super) dt_first_prev: DateTimeLOpt,
    /// Last (latest) processed [`DateTimeL`] (not necessarily printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    // TODO: [2022/07/27] cost-savings: save the ref
    // TODO: [2023/03/22] change behavior to be "last printed" instead of "last processed"
    pub(super) dt_last: DateTimeLOpt,
    pub(super) dt_last_prev: DateTimeLOpt,
    /// `Count`s found patterns stored in `dt_patterns`.
    /// "mirrors" the global [`DATETIME_PARSE_DATAS`].
    /// Keys are indexes into `DATETIME_PARSE_DATAS`,
    /// values are counts of successful pattern match at that index.
    ///
    /// Initialized once in `fn SyslineReader::new`.
    ///
    /// Used to determine the best `DateTimeParseInstr` to use for sysline
    /// matching.
    /// Not used after `self.analyzed` becomes `true` though still updated.
    ///
    /// [`DATETIME_PARSE_DATAS`]: crate::data::datetime::DATETIME_PARSE_DATAS
    pub(super) dt_patterns_counts: DateTimePatternCounts,
    /// Keys of `self.dt_patterns_counts`. Expected to be sorted by value.
    /// Updated in function `dt_patterns_indexes_refresh`.
    ///
    /// Not updated after `self.analyzed` becomes `true`.
    dt_patterns_indexes: DateTimeParseDatasIndexes,
    /// Default [`FixedOffset`] for a found datetime without a timezone.
    ///
    /// Similar to the "fallback year". Different in that the
    /// "fallback timezone" offset will not change within one syslog file,
    /// whereas a "fallback year" may change per sysline.
    ///
    /// [`FixedOffset`]: https://docs.rs/chrono/0.4.22/chrono/offset/struct.FixedOffset.html
    tz_offset: FixedOffset,
    /// Enable or disable the internal LRU cache for `find_sysline()`.
    find_sysline_lru_cache_enabled: bool,
    /// Internal [LRU cache] for `find_sysline()`.
    /// Maintained in function `SyslineReader::find_sysline`.
    ///
    /// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
    // TODO: remove `pub(super)`
    pub(super) find_sysline_lru_cache: SyslinesLRUCache,
    /// `Count` of internal LRU cache lookup hits.
    pub(super) find_sysline_lru_cache_hit: Count,
    /// `Count` of internal LRU cache lookup misses.
    pub(super) find_sysline_lru_cache_miss: Count,
    /// `Count` of internal LRU cache lookup `.put`.
    pub(super) find_sysline_lru_cache_put: Count,
    /// Enable or disable `parse_datetime_in_line_lru_cache`.
    parse_datetime_in_line_lru_cache_enabled: bool,
    /// Internal [LRU cache] of calls to
    /// `SyslineReader::parse_datetime_in_line()`.
    /// Maintained in function `SyslineReader::find_sysline()`.
    ///
    /// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
    parse_datetime_in_line_lru_cache: LineParsedCache,
    /// `Count` of `self.parse_datetime_in_line_lru_cache` lookup hit.
    pub(super) parse_datetime_in_line_lru_cache_hit: Count,
    /// `Count` of `self.parse_datetime_in_line_lru_cache` lookup miss.
    pub(super) parse_datetime_in_line_lru_cache_miss: Count,
    /// `Count` of `self.parse_datetime_in_line_lru_cache.put`.
    pub(super) parse_datetime_in_line_lru_cache_put: Count,
    /// `Count` of `line.get_boxptrs` returning `SinglePtr`.
    pub(super) get_boxptrs_singleptr: Count,
    /// `Count` of `line.get_boxptrs` returning `DoublePtr`.
    pub(super) get_boxptrs_doubleptr: Count,
    /// `Count` of `line.get_boxptrs` returning `MultiPtr`.
    pub(super) get_boxptrs_multiptr: Count,
    /// Has `self.file_analysis` completed?
    ///
    /// Initially `false`. During function `parse_datetime_in_line` all patterns
    /// in [`DATETIME_PARSE_DATAS_REGEX_VEC`] may be used for a search.
    /// During this time, `dt_pattern_counts` is updated in
    /// function `dt_patterns_indexes_refresh`.
    ///
    /// Once `true` then only a subset of patterns that successfully matched
    /// syslines are used.
    /// This avoids likely fruitless or misleading searches for datetime in
    /// a `Line`. Those searches are resource expensive.
    ///
    /// [`DATETIME_PARSE_DATAS_REGEX_VEC`]: static@crate::data::datetime::DATETIME_PARSE_DATAS_REGEX_VEC
    analyzed: bool,
    /// `Count` of `Ok` to [`Arc::try_unwrap(syslinep)`], effectively a count of
    /// [`Sysline`] dropped.
    ///
    /// [`Arc::try_unwrap(syslinep)`]: std::sync::Arc#method.try_unwrap
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub(super) drop_sysline_ok: Count,
    /// `Count` of failures to [`Arc::try_unwrap(syslinep)`].
    ///
    /// A small count is typically okay.
    ///
    /// [`Arc::try_unwrap(syslinep)`]: std::sync::Arc#method.try_unwrap
    pub(super) drop_sysline_errors: Count,
    /// `Count` of EZCHECK12 attempts that "hit".
    ///
    /// EZCHECK12 is a simple hack to skipping regular expression matching for
    /// a `Line`. Regular expression matching is expensive.
    ///
    /// This hack is only applicable to ASCII/UTF-8 encoded text files.
    pub (super) ezcheck12_hit: Count,
    /// `Count` of EZCHECK12 attempts that missed.
    pub (super) ezcheck12_miss: Count,
    /// Highest EXCHECK12 `Line` byte length ignored.
    pub (super) ezcheck12_hit_max: LineIndex,
    /// `Count` of EZCHECKD2 attempts that "hit".
    ///
    /// EZCHECKD2 is a simple hack to skipping regular expression matching for
    /// a `Line`. Regular expression matching is expensive.
    ///
    /// This hack is only applicable to ASCII/UTF-8 encoded text files.
    pub (super) ezcheckd2_hit: Count,
    /// `Count` of EZCHECKD2 attempts that missed.
    pub (super) ezcheckd2_miss: Count,
    /// Highest EXCHECKD2 `Line` byte length ignored.
    pub (super) ezcheckd2_hit_max: LineIndex,
    /// testing-only tracker of successfully dropped `Sysline`
    #[cfg(test)]
    pub(crate) dropped_syslines: SetDroppedSyslines,
}

impl fmt::Debug for SyslineReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("SyslineReader")
            .field("linereader", &self.linereader)
            .field("syslines", &self.syslines)
            .finish()
    }
}

/// Debug helper function to print LRU cache.
#[doc(hidden)]
#[allow(non_snake_case, dead_code)]
#[cfg(debug_assertions)]
fn debug_eprint_LRU_cache<K, V>(cache: &LruCache<K, V>)
where
    K: std::fmt::Debug,
    K: std::hash::Hash,
    K: Eq,
    V: std::fmt::Debug,
{
    #[cfg(debug_assertions)]
    {
        eprintln!("[");
        for (key, val) in cache.iter() {
            eprintln!(" Key: {:?}, Value: {:?};", key, val);
        }
        eprintln!("]");
    }
}

#[derive(Clone, Default)]
pub struct SummarySyslineReader {
    /// `SyslineReader::drop_sysline_ok`
    pub syslinereader_drop_sysline_ok: Count,
    /// `SyslineReader::drop_sysline_errors`
    pub syslinereader_drop_sysline_errors: Count,
    /// `Count` of `Syslines` processed by `SyslineReader`
    pub syslinereader_syslines: Count,
    /// "high watermark"` of `Sysline`s stored by `SyslineReader.syslines`
    pub syslinereader_syslines_stored_highest: usize,
    /// `SyslineReader::_syslines_hit`
    pub syslinereader_syslines_hit: Count,
    /// `SyslineReader::_syslines_miss`
    pub syslinereader_syslines_miss: Count,
    /// `SyslineReader::_syslines_by_range_hit`
    pub syslinereader_syslines_by_range_hit: Count,
    /// `SyslineReader::_syslines_by_range_miss`
    pub syslinereader_syslines_by_range_miss: Count,
    /// `SyslineReader::_syslines_by_range_put`
    pub syslinereader_syslines_by_range_put: Count,
    /// datetime patterns used by `SyslineReader`
    pub syslinereader_patterns: DateTimePatternCounts,
    /// datetime soonest seen (not necessarily reflective of entire file)
    pub syslinereader_datetime_first: DateTimeLOpt,
    /// datetime latest seen (not necessarily reflective of entire file)
    pub syslinereader_datetime_last: DateTimeLOpt,
    /// `SyslineReader::find_sysline`
    pub syslinereader_find_sysline_lru_cache_hit: Count,
    /// `SyslineReader::find_sysline`
    pub syslinereader_find_sysline_lru_cache_miss: Count,
    /// `SyslineReader::find_sysline`
    pub syslinereader_find_sysline_lru_cache_put: Count,
    /// `SyslineReader::parse_datetime_in_line`
    pub syslinereader_parse_datetime_in_line_lru_cache_hit: Count,
    /// `SyslineReader::parse_datetime_in_line`
    pub syslinereader_parse_datetime_in_line_lru_cache_miss: Count,
    /// `SyslineReader::parse_datetime_in_line`
    pub syslinereader_parse_datetime_in_line_lru_cache_put: Count,
    /// `SyslineReader::get_boxptrs_singleptr`
    pub syslinereader_get_boxptrs_singleptr: Count,
    /// `SyslineReader::get_boxptrs_doubleptr`
    pub syslinereader_get_boxptrs_doubleptr: Count,
    /// `SyslineReader::get_boxptrs_multiptr`
    pub syslinereader_get_boxptrs_multiptr: Count,
    /// `SyslineReader::ezcheck12_hit`
    pub syslinereader_ezcheck12_hit: Count,
    /// `SyslineReader::ezcheck12_miss`
    pub syslinereader_ezcheck12_miss: Count,
    /// `SyslineReader::ezcheck12_hit_max`
    pub syslinereader_ezcheck12_hit_max: LineIndex,
    /// `SyslineReader::ezcheckd2_hit`
    pub syslinereader_ezcheckd2_hit: Count,
    /// `SyslineReader::ezcheckd2_miss`
    pub syslinereader_ezcheckd2_miss: Count,
    /// `SyslineReader::ezcheckd2_hit_max`
    pub syslinereader_ezcheckd2_hit_max: LineIndex,
}

/// Implement the `SyslineReader`
impl SyslineReader {
    /// Maximum number of datetime patterns for matching the remainder of a syslog file.
    pub(crate) const DT_PATTERN_MAX: usize = 1;

    /// Capacity of internal LRU cache `find_sysline_lru_cache`.
    const FIND_SYSLINE_LRU_CACHE_SZ: usize = 4;

    /// Capacity of internal LRU cache `parse_datetime_in_line_lru_cache`.
    const PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ: usize = 8;

    /// A `Line.len()` must at least this value to proceed with a datetime search, if `Line.len()`
    /// is less then it is presumed no datetime string could fit on the line.
    ///
    /// This allows skipping a few datetime searches that would fail.
    const DATETIME_STR_MIN: usize = 8;

    /// Default state of LRU caches.
    // TODO: add this to other LRU cache-having structs
    const CACHE_ENABLE_DEFAULT: bool = true;

    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz: BlockSz,
        tz_offset: FixedOffset,
    ) -> Result<SyslineReader> {
        deñ!("SyslineReader::new({:?}, {:?}, {:?}, {:?})", path, filetype, blocksz, tz_offset);
        let lr = match LineReader::new(path, filetype, blocksz) {
            Ok(val) => val,
            Err(err) => {
                //eprintln!("ERROR: LineReader::new({}, {}) failed {}", path, blocksz, err);
                return Err(err);
            }
        };
        let mut dt_patterns_counts = DateTimePatternCounts::new();
        let mut dt_patterns_indexes = DateTimeParseDatasIndexes::new();
        let mut index = 0;
        while index < DATETIME_PARSE_DATAS_LEN {
            dt_patterns_counts.insert(index as DateTimeParseInstrsIndex, 0);
            dt_patterns_indexes.insert(index as DateTimeParseInstrsIndex);
            index += 1;
        }
        Ok(SyslineReader {
            linereader: lr,
            syslines: Syslines::new(),
            syslines_count: 0,
            syslines_stored_highest: 0,
            syslines_by_range: SyslinesRangeMap::new(),
            syslines_hit: 0,
            syslines_miss: 0,
            syslines_by_range_hit: 0,
            syslines_by_range_miss: 0,
            syslines_by_range_put: 0,
            dt_first: None,
            dt_first_prev: None,
            dt_last: None,
            dt_last_prev: None,
            dt_patterns_counts,
            dt_patterns_indexes,
            tz_offset,
            find_sysline_lru_cache_enabled: SyslineReader::CACHE_ENABLE_DEFAULT,
            find_sysline_lru_cache: SyslinesLRUCache::new(
                std::num::NonZeroUsize::new(SyslineReader::FIND_SYSLINE_LRU_CACHE_SZ).unwrap(),
            ),
            find_sysline_lru_cache_hit: 0,
            find_sysline_lru_cache_miss: 0,
            find_sysline_lru_cache_put: 0,
            parse_datetime_in_line_lru_cache_enabled: SyslineReader::CACHE_ENABLE_DEFAULT,
            parse_datetime_in_line_lru_cache: LineParsedCache::new(
                std::num::NonZeroUsize::new(SyslineReader::PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ).unwrap(),
            ),
            parse_datetime_in_line_lru_cache_hit: 0,
            parse_datetime_in_line_lru_cache_miss: 0,
            parse_datetime_in_line_lru_cache_put: 0,
            get_boxptrs_singleptr: 0,
            get_boxptrs_doubleptr: 0,
            get_boxptrs_multiptr: 0,
            analyzed: false,
            drop_sysline_ok: 0,
            drop_sysline_errors: 0,
            ezcheck12_hit: 0,
            ezcheck12_miss: 0,
            ezcheck12_hit_max: 0,
            ezcheckd2_hit: 0,
            ezcheckd2_miss: 0,
            ezcheckd2_hit_max: 0,
            #[cfg(test)]
            dropped_syslines: SetDroppedSyslines::new(),
        })
    }

    /// See [`BlockReader::filetype`].
    ///
    /// [`BlockReader::filetype`]: crate::readers::blockreader::BlockReader#method.filetype
    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.linereader.filetype()
    }

    /// See [`BlockReader::blocksz`].
    ///
    /// [`BlockReader::blocksz`]: crate::readers::blockreader::BlockReader#method.blocksz
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    /// See [`BlockReader::filesz`].
    ///
    /// [`BlockReader::filesz`]: crate::readers::blockreader::BlockReader#method.filesz
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.linereader.filesz()
    }

    /// See [`BlockReader::mimeguess`].
    ///
    /// [`BlockReader::mimeguess`]: crate::readers::blockreader::BlockReader#method.mimeguess
    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.linereader.mimeguess()
    }

    /// See [`BlockReader::path`].
    ///
    /// [`BlockReader::path`]: crate::readers::blockreader::BlockReader#method.path
    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        self.linereader.path()
    }

    /// See [`BlockReader::block_offset_at_file_offset`].
    ///
    /// [`BlockReader::block_offset_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_offset_at_file_offset
    pub const fn block_offset_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockOffset {
        self.linereader
            .block_offset_at_file_offset(fileoffset)
    }

    /// See [`BlockReader::file_offset_at_block_offset`].
    ///
    /// [`BlockReader::file_offset_at_block_offset`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset
    pub const fn file_offset_at_block_offset(
        &self,
        blockoffset: BlockOffset,
    ) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset(blockoffset)
    }

    /// See [`BlockReader::file_offset_at_block_offset_index`].
    ///
    /// [`BlockReader::file_offset_at_block_offset_index`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset_index
    pub const fn file_offset_at_block_offset_index(
        &self,
        blockoffset: BlockOffset,
        blockindex: BlockIndex,
    ) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// See [`BlockReader::fileoffset_last`].
    ///
    /// [`BlockReader::fileoffset_last`]: crate::readers::blockreader::BlockReader#method.fileoffset_last
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.linereader
            .fileoffset_last()
    }

    /// See [`BlockReader::block_index_at_file_offset`].
    ///
    /// [`BlockReader::block_index_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_index_at_file_offset
    pub const fn block_index_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockIndex {
        self.linereader
            .block_index_at_file_offset(fileoffset)
    }

    /// See [`BlockReader::count_blocks`].
    ///
    /// [`BlockReader::count_blocks`]: crate::readers::blockreader::BlockReader#method.count_blocks
    pub const fn count_blocks(&self) -> Count {
        self.linereader.count_blocks()
    }

    /// See [`BlockReader::blockoffset_last`].
    ///
    /// [`BlockReader::blockoffset_last`]: crate::readers::blockreader::BlockReader#method.blockoffset_last
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.linereader
            .blockoffset_last()
    }

    /// See [`LineReader::charsz`].
    ///
    /// [`LineReader::charsz`]: crate::readers::linereader::LineReader#method.charsz
    pub const fn charsz(&self) -> usize {
        self.linereader.charsz()
    }

    /// See [`BlockReader::mtime`].
    ///
    /// [`BlockReader::mtime`]: crate::readers::blockreader::BlockReader#method.mtime
    pub fn mtime(&self) -> SystemTime {
        self.linereader.mtime()
    }

    /// `Count` of `Sysline`s processed so far, i.e. `self.syslines_count`.
    pub fn count_syslines_processed(&self) -> Count {
        self.syslines_count
    }

    /// `Count` of `Sysline`s currently stored, i.e. `self.syslines.len()`
    pub fn count_syslines_stored(&self) -> Count {
        self.syslines.len() as Count
    }

    /// "high watermark" of `Sysline`s stored in `self.syslines`
    pub fn syslines_stored_highest(&self) -> usize {
        self.syslines_stored_highest
    }

    /// See [`LineReader::count_lines_processed`].
    ///
    /// [`LineReader::count_lines_processed`]: crate::readers::linereader::LineReader#method.count_lines_processed
    #[inline(always)]
    pub fn count_lines_processed(&self) -> Count {
        self.linereader
            .count_lines_processed()
    }

    /// Does the `dt_pattern` have a year? e.g. specificer `%Y` or `%y`.
    pub fn dt_pattern_has_year(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            if !self.syslines.is_empty() {
                de_wrn!("called dt_pattern_has_year() without having processed some syslines");
            }
        }
        let dtpd: &DateTimeParseInstr = self.datetime_parse_data();
        defñ!("dtpd line {:?}", dtpd._line_num);

        dtpd.dtfs.has_year()
    }

    /// eEable internal LRU cache used by `find_sysline` and
    /// `parse_datetime_in_line`.
    ///
    /// Returns prior value of `find_sysline_lru_cache_enabled`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_enable(&mut self) -> bool {
        let ret = self.find_sysline_lru_cache_enabled;
        debug_assert_eq!(
            self.find_sysline_lru_cache_enabled, self.parse_datetime_in_line_lru_cache_enabled,
            "cache enables disagree"
        );
        if !self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_enabled = true;
            self.find_sysline_lru_cache
                .clear();
            self.find_sysline_lru_cache
                .resize(std::num::NonZeroUsize::new(SyslineReader::FIND_SYSLINE_LRU_CACHE_SZ).unwrap());
        }
        if !self.parse_datetime_in_line_lru_cache_enabled {
            self.parse_datetime_in_line_lru_cache_enabled = true;
            self.parse_datetime_in_line_lru_cache
                .clear();
            self.parse_datetime_in_line_lru_cache
                .resize(
                    std::num::NonZeroUsize::new(SyslineReader::PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ).unwrap(),
                );
        }

        defñ!("return {}", ret);

        ret
    }

    /// Disable internal LRU cache used by `find_sysline` and
    /// `parse_datetime_in_line`.
    ///
    /// Returns prior value of `find_sysline_lru_cache_enabled`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_disable(&mut self) -> bool {
        let ret = self.find_sysline_lru_cache_enabled;
        debug_assert_eq!(
            self.find_sysline_lru_cache_enabled, self.parse_datetime_in_line_lru_cache_enabled,
            "cache enables disagree"
        );
        self.find_sysline_lru_cache_enabled = false;
        self.find_sysline_lru_cache
            .clear();
        self.parse_datetime_in_line_lru_cache_enabled = false;
        self.parse_datetime_in_line_lru_cache
            .clear();

        defñ!("return {}", ret);

        ret
    }

    /// Print `Sysline` at `FileOffset`.
    ///
    /// Testing helper function only.
    #[doc(hidden)]
    #[cfg(test)]
    pub fn print(
        &self,
        fileoffset: FileOffset,
        raw: bool,
    ) {
        let syslinep: &SyslineP = match self.syslines.get(&fileoffset) {
            Some(val) => val,
            None => {
                eprintln!("ERROR: in print, self.syslines.get({}) returned None", fileoffset);
                return;
            }
        };
        for linep in &(*syslinep).lines {
            (*linep).print(raw);
        }
    }

    /// Return most used `DateTimeParseInstr`.
    ///
    /// Before `dt_patterns_analysis()` completes, this may return different
    /// values.
    ///
    /// After `dt_patterns_analysis()` completes, it will return the same value.
    pub(crate) fn datetime_parse_data(&self) -> &DateTimeParseInstr {
        &DATETIME_PARSE_DATAS[self.dt_pattern_index_max_count()]
    }

    /// Is this [`Sysline`] the last `Sysline` of the entire file?
    /// (not the same as last Sysline within the optional datetime filters).
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub fn is_sysline_last(
        &self,
        sysline: &Sysline,
    ) -> bool {
        let fo_end: FileOffset = sysline.fileoffset_end();
        if fo_end == self.fileoffset_last() {
            //defñ!("return true");
            return true;
        }
        debug_assert_lt!(
            fo_end,
            self.filesz(),
            "fileoffset_end {} is at or after filesz() {}",
            fo_end,
            self.filesz(),
        );
        //defñ!("return false");

        false
    }

    /// Wrapper for function [`is_sysline_last`].
    ///
    /// [`is_sysline_last`]: self::SyslineReader#method.is_sysline_last
    pub fn is_syslinep_last(
        &self,
        syslinep: &SyslineP,
    ) -> bool {
        self.is_sysline_last(syslinep.as_ref())
    }

    /// Clear all the syslines so far collected.
    /// Only intended to aid post-processing year updates.
    ///
    /// Users must know what they are doing with this.
    ///
    /// Leaves the statistics as-is which means they may be confusing. For example,
    /// given a file with 5 syslines, then all 5 syslines are parsed and stored, then
    /// `clear_syslines()` is called, and then the file is processed again, the resulting
    /// `self.syslines_count` will be value `10`.
    pub(crate) fn clear_syslines(&mut self) {
        defñ!();
        let cache_enable = self.LRU_cache_disable();
        self.syslines.clear();
        self.syslines_by_range = SyslinesRangeMap::new();
        self.dt_first = None;
        self.dt_first_prev = None;
        self.dt_last = None;
        self.dt_last_prev = None;
        if cache_enable {
            self.LRU_cache_enable();
        }
    }

    /// Remove the `Syline` at `FileOffset`.
    /// Only intended to aid post-processing year updates.
    ///
    /// Users must know what they are doing with this.
    pub(crate) fn remove_sysline(
        &mut self,
        fileoffset: FileOffset,
    ) -> bool {
        defn!("({:?})", fileoffset);
        let cache_enable = self.LRU_cache_disable();
        let syslinep_opt: Option<SyslineP> = self
            .syslines
            .remove(&fileoffset);
        let mut ret = true;
        match syslinep_opt {
            Some(syslinep) => {
                let fo_beg: FileOffset = (*syslinep).fileoffset_begin();
                let fo_end: FileOffset = (*syslinep).fileoffset_end();
                defo!(
                    "sysline at {} removed; {:?} {:?}",
                    fileoffset,
                    (*syslinep).dt(),
                    (*syslinep).to_String_noraw()
                );
                debug_assert_eq!(
                    fileoffset, fo_beg,
                    "mismatching fileoffset {}, fileoffset_begin {}",
                    fileoffset, fo_beg
                );
                let fo_end1: FileOffset = fo_end + (self.charsz() as FileOffset);
                let range: SyslineRange = SyslineRange {
                    start: fo_beg,
                    end: fo_end1,
                };
                defo!("syslines_by_range remove {:?}", range);
                self.syslines_by_range
                    .remove(range);
                self.dt_first = self.dt_first_prev;
                self.dt_last = self.dt_last_prev;
            }
            None => {
                defo!("syslines failed to remove {}", fileoffset);
                ret = false;
            }
        }
        if cache_enable {
            self.LRU_cache_enable();
        }
        defx!("({:?}) return {:?}", fileoffset, ret);

        ret
    }

    /// Store the passed `Sysline` in `self.syslines`.
    /// Update other fields.
    fn insert_sysline(
        &mut self,
        sysline: Sysline,
    ) -> SyslineP {
        let fo_beg: FileOffset = sysline.fileoffset_begin();
        let fo_end: FileOffset = sysline.fileoffset_end();
        let syslinep: SyslineP = SyslineP::new(sysline);
        defn!(
            "syslines.insert({}, Sysline @[{}, {}] datetime: {:?})",
            fo_beg,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).dt()
        );
        self.syslines
            .insert(fo_beg, syslinep.clone());
        self.syslines_count += 1;
        self.syslines_stored_highest = std::cmp::max(self.syslines.len(), self.syslines_stored_highest);
        // XXX: Issue #16 only handles UTF-8/ASCII encoding
        let fo_end1: FileOffset = fo_end + (self.charsz() as FileOffset);
        defx!("syslines_by_range.insert([{}‥{}), {})", fo_beg, fo_end1, fo_beg);
        self.syslines_by_range
            .insert(fo_beg..fo_end1, fo_beg);
        self.syslines_by_range_put += 1;

        syslinep
    }

    /// Forcefully `drop` data associated with the [`Block`] at [`BlockOffset`]
    /// *AND ALL PRIOR BLOCKS* (or at least, drop as much as possible).
    ///
    /// Caller must know what they are doing!
    ///
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`BlockOffset`]: crate::readers::blockreader::BlockOffset
    pub fn drop_data(
        &mut self,
        blockoffset: BlockOffset,
    ) -> bool {
        def1n!("({})", blockoffset);

        let mut ret = true;
        // vec of `fileoffset` must be ordered which is guaranteed by `syslines: BTreeMap`
        let mut drop_fo: Vec<FileOffset> = Vec::<FileOffset>::with_capacity(self.syslines.len());
        for (fo, _) in self
            .syslines
            .iter()
            .filter(|(_, s)| (*s).blockoffset_last() <= blockoffset)
        {
            drop_fo.push(*fo);
        }
        // XXX: it is not straightforward to get the collection of FileOffset keys to use
        //      This is because it
        // TODO: [2022/06/18] cost-savings: make this a "one time" creation that is reused
        //       this is challenging, as it runs into borrow errors during `.iter()`
        def1o!("collected keys {:?}", drop_fo);
        // XXX: using `self.syslines.value_mut()` would be cleaner.
        //      But `self.syslines.value_mut()` causes a clone of the `SyslineP`, which then
        //      increments the `Arc` "strong_count". That in turn prevents `Arc::get_mut(&SyslineP)`
        //      from returning the original `Sysline`.
        //      Instead of `syslines.values_mut()`, use `syslines.keys()` and then `syslines.get_mut`
        //      to get a `&SyslineP`. This does not increase the "strong_count".
        for fo in drop_fo.iter() {
            if !self.drop_sysline(fo) {
                ret = false;
            }
        }

        def1x!("({}) return {}", blockoffset, ret);

        ret
    }

    /// Forcefully `drop` data associated with the [`Sysline`] at
    /// [`FileOffset`] (or at least, drop as much as possible).
    ///
    /// Caller must know what they are doing!
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`FileOffset`]: crate::common::FileOffset
    pub fn drop_sysline(
        &mut self,
        fileoffset: &FileOffset,
    ) -> bool {
        defn!("({})", fileoffset);
        let mut ret = true;
        let syslinep: SyslineP = match self
            .syslines
            .remove(fileoffset)
        {
            Some(syslinep_) => syslinep_,
            None => {
                defx!();
                de_wrn!("syslines.remove({}) returned None which is unexpected", fileoffset);
                return false;
            }
        };
        defo!(
            "Processing SyslineP @[{}‥{}], Block @[{}‥{}] strong_count {}",
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).blockoffset_first(),
            (*syslinep).blockoffset_last(),
            Arc::strong_count(&syslinep),
        );
        self.find_sysline_lru_cache
            .pop(&(*syslinep).fileoffset_begin());
        match Arc::try_unwrap(syslinep) {
            Ok(sysline) => {
                defo!(
                    "Arc::try_unwrap(syslinep) dropped Sysline @[{}‥{}] Block @[{}‥{}]",
                    sysline.fileoffset_begin(),
                    sysline.fileoffset_end(),
                    sysline.blockoffset_first(),
                    sysline.blockoffset_last()
                );
                self.drop_sysline_ok += 1;
                #[cfg(test)]
                {
                    self.dropped_syslines.insert(sysline.fileoffset_begin());
                }
                if ! self.linereader.drop_lines(sysline.lines) {
                    ret = false;
                }
            }
            Err(_syslinep) => {
                defo!("Arc::try_unwrap(syslinep) failed to drop Sysline, strong_count {}", Arc::strong_count(&_syslinep));
                self.drop_sysline_errors += 1;
                ret = false;
            }
        }
        defx!("return {}", ret);

        ret
    }

    /// If datetime found in `Line` returns [`Ok`] around
    /// indexes into `Line` of found datetime string
    /// (start of string, end of string)`
    ///
    /// else returns [`Err`].
    ///
    /// [`Ok`]: self::ResultFindDateTime
    /// [`Err`]: self::ResultFindDateTime
    #[allow(clippy::too_many_arguments)]
    pub fn find_datetime_in_line(
        line: &Line,
        parse_data_indexes: &DateTimeParseDatasIndexes,
        charsz: &CharSz,
        year_opt: &Option<Year>,
        tz_offset: &FixedOffset,
        get_boxptrs_singleptr: &mut Count,
        get_boxptrs_doubleptr: &mut Count,
        get_boxptrs_multiptr: &mut Count,
        ezcheck12_hit: &mut Count,
        ezcheck12_miss: &mut Count,
        ezcheck12_hit_max: &mut LineIndex,
        ezcheckd2_hit: &mut Count,
        ezcheckd2_miss: &mut Count,
        ezcheckd2_hit_max: &mut LineIndex,
    ) -> ResultFindDateTime {
        defn!(
            "(…, …, {:?}, year_opt {:?}, {:?}) line {:?}",
            charsz,
            year_opt,
            tz_offset,
            line.to_String_noraw()
        );
        defo!("parse_data_indexes.len() {} {:?}", parse_data_indexes.len(), parse_data_indexes);

        // skip an easy case; no possible datetime
        if line.len() < SyslineReader::DATETIME_STR_MIN {
            defx!("return Err(ErrorKind::InvalidInput);");
            return ResultFindDateTime::Err(Error::new(
                ErrorKind::InvalidInput,
                "Line is too short to hold a datetime",
            ));
        }

        // EZCHECK12 is a fun hack to bypass regex checks.
        // Lines without `'1'` or `'2'` within some range from the start of the
        // line probably do not have a datetime.
        const EZCHECK12: &[u8; 2] = b"12";
        let mut ezcheck12_min: usize = 0;
        let mut ezcheckd2_min: usize = 0;

        #[cfg(any(debug_assertions, test))]
        let mut _attempts: usize = 0;

        // `sie` and `siea` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        for (_try, index) in parse_data_indexes
            .iter()
            .enumerate()
        {
            let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*index];
            defo!("pattern data try {} index {} dtpd.line_num {}", _try, index, dtpd._line_num);

            if line.len() <= dtpd.range_regex.start {
                defo!(
                    "line too short {} for  requested start {}; continue",
                    line.len(),
                    dtpd.range_regex.start
                );
                continue;
            }
            if line.len() <= ezcheck12_min {
                // the Line is shorter than established check for "12"
                *ezcheck12_hit += 1;
                defo!(
                    "line len {} ≤ {} ezcheck12_min; continue",
                    line.len(),
                    ezcheck12_min,
                );
                continue;
            }
            if line.len() <= ezcheckd2_min {
                // the Line is shorter than established check for two digits
                *ezcheckd2_hit += 1;
                defo!(
                    "line len {} ≤ {} ezcheckd2_min; continue",
                    line.len(),
                    ezcheckd2_min,
                );
                continue;
            }
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            let slice_end: usize = if line.len() > dtpd.range_regex.end {
                dtpd.range_regex.end
            } else {
                line.len()
            };
            if dtpd.range_regex.start >= slice_end {
                defo!("bad line slice indexes [{}, {}); continue", dtpd.range_regex.start, slice_end);
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // this is to force the parsing function `Local.datetime_from_str` to constrain where it
            // searches within the `Line`
            let mut hack_slice: Bytes;
            let slice_: &[u8];
            match line.get_boxptrs(dtpd.range_regex.start as LineIndex, slice_end as LineIndex) {
                LinePartPtrs::NoPtr => {
                    panic!(
                        "line.get_boxptrs({}, {}) returned NoPtr which means it was passed non-sense values",
                        dtpd.range_regex.start, slice_end
                    );
                    //continue;
                }
                LinePartPtrs::SinglePtr(box_slice) => {
                    slice_ = *box_slice;
                    *get_boxptrs_singleptr += 1;
                }
                LinePartPtrs::DoublePtr(box_slice1, box_slice2) => {
                    hack_slice = Bytes::with_capacity(box_slice1.len() + box_slice2.len());
                    hack_slice.extend_from_slice(*box_slice1);
                    hack_slice.extend_from_slice(*box_slice2);
                    slice_ = hack_slice.as_slice();
                    *get_boxptrs_doubleptr += 1;
                }
                LinePartPtrs::MultiPtr(vec_box_slice) => {
                    // inefficient case, hopefully very rarely occurs
                    let mut cap: usize = 0;
                    for box_ in vec_box_slice.iter() {
                        cap += box_.len();
                    }
                    hack_slice = Bytes::with_capacity(cap);
                    for box_ in vec_box_slice.into_iter() {
                        hack_slice.extend_from_slice(*box_);
                    }
                    slice_ = hack_slice.as_slice();
                    *get_boxptrs_multiptr += 1;
                }
            };
            defo!(
                "slice len {} [{}, {}) (requested [{}, {})) using DTPD from {}, data {:?}",
                slice_.len(),
                dtpd.range_regex.start,
                slice_end,
                dtpd.range_regex.start,
                dtpd.range_regex.end,
                dtpd._line_num,
                String::from_utf8_lossy(slice_),
            );

            // XXX: hack efficiency improvement, presumes all found years will
            //      have a '1' or a '2' in them.
            //      Regular expression matching is very expensive according to
            //      `tools/flamegraph.sh`. Skip where possible.
            //      Only applicable to ASCII/UTF-8 encoded text files.
            if charsz == &1 && dtpd.dtfs.has_year4() {
                if !slice_contains_X_2(slice_, EZCHECK12) {
                    defo!("skip slice, does not have '1' or '2' (EZCHECK12)");
                    if ezcheck12_min < slice_.len() {
                        // now proven that magic "12" is not in this `Line` up
                        // to this byte offset
                        ezcheck12_min = slice_.len() - charsz;
                        defo!("ezcheck12_min = {}", ezcheck12_min);
                    }
                    *ezcheck12_hit += 1;
                    *ezcheck12_hit_max = std::cmp::max(*ezcheck12_hit_max, ezcheck12_min);
                    continue;
                } else {
                    *ezcheck12_miss += 1;
                }
            }

            // XXX: hack efficiency improvement, presumes a datetime substring
            //      will have two consecutive digit chars in it.
            //      Regular expression matching is very expensive according to
            //      `tools/flamegraph.sh`. Skip where possible.
            //      Only applicable to ASCII/UTF-8 encoded text files.
            if charsz == &1 && dtpd.dtfs.has_d2() {
                if !slice_contains_D2(slice_) {
                    defo!("skip slice (EZCHECKD2)");
                    if ezcheckd2_min < slice_.len() {
                        // now proven that this `Line` does not have two
                        // consecutive digits up to this byte offset
                        ezcheckd2_min = slice_.len() - charsz;
                        defo!("ezcheckd2_min = {}", ezcheckd2_min);
                    }
                    *ezcheckd2_hit += 1;
                    *ezcheckd2_hit_max = std::cmp::max(*ezcheckd2_hit_max, ezcheckd2_min);
                    continue;
                } else {
                    *ezcheckd2_miss += 1;
                }
            }

            #[cfg(any(debug_assertions, test))]
            {
                _attempts += 1;
            }

            // find the datetime string using `Regex`, convert to a `DateTimeL`
            let dt: DateTimeL;
            let dt_beg: LineIndex;
            let dt_end: LineIndex;
            (dt_beg, dt_end, dt) =
                match bytes_to_regex_to_datetime(slice_, index, year_opt, tz_offset) {
                    None => continue,
                    Some(val) => val,
            };
            defx!("return Ok({}, {}, {}, {});", dt_beg, dt_end, &dt, index);
            return ResultFindDateTime::Ok((dt_beg, dt_end, dt, *index));
        } // end for(pattern, …)

        defx!("return Err(ErrorKind::NotFound); tried {} DateTimeParseInstr", _attempts);
        ResultFindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found in Line!"))
    }

    /// Update the two statistic `DateTimeL` of
    /// `self.dt_first` and `self.dt_last`.
    fn dt_first_last_update(
        &mut self,
        datetime: &DateTimeL,
    ) {
        defñ!("({:?})", datetime);
        // TODO: the `dt_first` and `dt_last` are only for `--summary`,
        //       no need to always copy datetimes.
        //       Would be good to only run this when `if self.do_summary {...}`
        match self.dt_first {
            Some(dt_first_) => {
                if &dt_first_ > datetime {
                    self.dt_first_prev = self.dt_first;
                    self.dt_first = Some(*datetime);
                }
            }
            None => {
                self.dt_first = Some(*datetime);
            }
        }
        match self.dt_last {
            Some(dt_last_) => {
                if &dt_last_ < datetime {
                    self.dt_last_prev = self.dt_last;
                    self.dt_last = Some(*datetime);
                }
            }
            None => {
                self.dt_last = Some(*datetime);
            }
        }
    }

    /// current count of `DateTimeParseInstrs` that have been used as tracked
    /// by `self.dt_patterns`.
    pub(crate) fn dt_patterns_counts_in_use(&self) -> usize {
        self.dt_patterns_counts.iter().filter(|(_index, count)| count > &&0).count()
    }

    /// Helper function to update `parse_datetime_in_line`.
    fn dt_patterns_update(
        &mut self,
        index: DateTimeParseInstrsIndex,
    ) {
        match self.dt_patterns_counts.get_mut(&index) {
            Some(counter) => {
                *counter += 1;
                defñ!("dt_patterns_counts({:?}) at {}", index, counter);
            }
            None => {
                panic!("index {} not present in self.dt_patterns_counts", index);
            }
        }
        // refresh the indexes every time until `dt_patterns_analysis` is called
        if self.analyzed {
            return;
        }
        self.dt_patterns_indexes_refresh();
    }

    /// Refresh the `self.dt_patterns_indexes` from `self.dt_patterns_counts`.
    /// This is an expensive operation; it is only expected to run during
    /// blockzero analysis.
    /// Only useful during the blockzero analysis stage before one final
    /// `DateTimeParseInstr` is chosen.
    fn dt_patterns_indexes_refresh(&mut self) {
        self.dt_patterns_indexes
            .clear();
        // get copy of pattern indexes sorted by value,
        // this makes the most-used parse_data more likely to be used again
        self.dt_patterns_indexes
            .extend(
                self.dt_patterns_counts
                    .iter()
                    .sorted_by(
                        |a, b| Ord::cmp(&b.1, &a.1), // sort by value (second tuple item)
                    )
                    .map(|(k, _v)| k), // copy only the key (first tuple item) which is an index
            );
        defñ!("dt_patterns_indexes {:?}", self.dt_patterns_indexes);
    }

    /// Analyze `Sysline`s gathered.
    ///
    /// When a threshold of `Sysline`s or bytes has been processed, then
    /// this function narrows down datetime formats to try for future
    /// datetime-parsing attempts.
    /// Further calls to function `SyslineReader::find_datetime_in_line`
    /// use far less resources.
    pub(crate) fn dt_patterns_analysis(&mut self) -> bool {
        defn!();
        debug_assert!(!self.analyzed, "already called dt_patterns_analysis()");
        // XXX: DT_PATERN_MAX > 1 is unimplemented
        const_assert!(SyslineReader::DT_PATTERN_MAX == 1);

        #[cfg(any(debug_assertions, test))]
        {
            for (k, v) in self.dt_patterns_counts.iter() {
                let data_: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC
                    .get(*k)
                    .unwrap();
                defo!("self.dt_patterns_counts[{:?}]={:?} is {:?}, {:?}", k, v, data_, data_rex_);
            }
        }
        defo!("dt_patterns_counts.len() {}", self.dt_patterns_counts.len());

        // get maximum value in `dt_patterns_counts`
        // ripped from https://stackoverflow.com/a/60134450/471376
        // test https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b8eb53f40fd89461c9dad9c976746cc3
        let max_ = self.dt_patterns_counts
            .iter()
            .fold(std::u64::MIN, |a, b| a.max(*(b.1)));
        if max_ == 0 {
            // no datetime patterns were found
            defx!("return false");
            return false;
        }
        // remove all items < maximum value in `dt_patterns_counts`
        defo!("dt_patterns_counts.retain(v >= {:?})", max_);
        self.dt_patterns_counts
            .retain(|_, v| *v >= max_);
        // if there is a tie for the most-used pattern, then `pop_last` until
        // only `DT_PATTERN_MAX` remains.
        // XXX: Note that this removal chooses by list ordering preferring to
        //      keep `DTPD` near the front of `DATETIME_PARSE_DATAS`. It might
        //      choose the wrong pattern. This should only be a problem for
        //      very short files that happen to have some equal part of
        //      datetime patterns that alternate on lines.
        while self.dt_patterns_counts.len() > SyslineReader::DT_PATTERN_MAX {
            // TODO: use `pop_last` which is experimental until MSRV 1.66.0
            //       see https://github.com/rust-lang/rust/issues/62924
            let rm_key: DateTimeParseInstrsIndex = *self.dt_patterns_counts.iter().last().unwrap().0;
            self.dt_patterns_counts.remove(&rm_key);
        }
        defo!("dt_patterns_counts.len() {}", self.dt_patterns_counts.len());

        #[cfg(any(debug_assertions, test))]
        {
            if self.dt_patterns_counts.len() != SyslineReader::DT_PATTERN_MAX {
                de_wrn!(
                    "dt_patterns_analysis: self.dt_patterns_counts.len() {}, expected {}",
                    self.dt_patterns_counts.len(),
                    SyslineReader::DT_PATTERN_MAX,
                );
            }
        }

        self.dt_patterns_indexes_refresh();

        #[cfg(any(debug_assertions, test))]
        {
            for (k, v) in self.dt_patterns_counts.iter() {
                let data_: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC
                    .get(*k)
                    .unwrap();
                defo!("self.dt_patterns_counts[index {:?}]={:?} is {:?}, {:?}", k, v, data_, data_rex_);
            }
        }

        self.analyzed = true;
        defx!("return true");

        true
    }

    /*
    /// get `DateTimeParseInstrsIndex` from SyslineReader. `rank` is zero-based with
    /// zero being the most important rank.
    ///
    /// Passing `rank` value `0` will return the `DateTimeParseInstrsIndex` for the
    /// most-used `DateTimeParseInstr` (i.e. the regex and strftime patterns used to extract
    /// `DateTimeL` from `Line`s).
    pub(crate) fn dt_pattern_index_at(&self, rank: usize) -> DateTimeParseInstrsIndex {
        *(self.dt_patterns_indexes.iter().skip(rank).next().unwrap())
    }
    */

    /// Return most-used `DateTimeParseInstrsIndex`.
    pub(crate) fn dt_pattern_index_max_count(&self) -> DateTimeParseInstrsIndex {
        #[cfg(debug_assertions)]
        {
            for (k, v) in self.dt_patterns_counts.iter() {
                let data: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*k];
                let data_rex: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC
                    .get(*k)
                    .unwrap();
                defo!("self.dt_patterns_counts[index {:?}]={:?} is {:?}, {:?}", k, v, data, data_rex);
            }
            for val in self
                .dt_patterns_indexes
                .iter()
            {
                defo!("self.dt_patterns_indexes {:?}", val);
            }
        }
        if !self.analyzed {
            defo!("before analysis");
            // before analysis, the uses of all `DateTimeParseInstr` are tracked
            // return index to maximum value
            // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=85ac85f48e6ddff04dc938b742872dc1
            let max_key_value: Option<(&DateTimeParseInstrsIndex, &Count)> = self
                .dt_patterns_counts
                .iter()
                .reduce(|accum, item| if accum.1 >= item.1 { accum } else { item });
            *max_key_value.unwrap().0
        } else {
            defo!("after analysis");
            // after analysis, only one `DateTimeParseInstr` is used
            debug_assert_eq!(
                self.dt_patterns_indexes.len(),
                SyslineReader::DT_PATTERN_MAX,
                "self.dt_patterns_indexes length {}, expected {}",
                self.dt_patterns_indexes.len(),
                SyslineReader::DT_PATTERN_MAX
            );
            // the first and only element is the chosen dt_pattern
            *self
                .dt_patterns_indexes
                .iter()
                .next()
                .unwrap()
        }
    }

    /// Attempt to parse a DateTime substring in the passed `Line`.
    ///
    /// Wraps call to `self.find_datetime_in_line` according to status of
    /// `self.dt_patterns`.<br/>
    /// If `self.dt_patterns` is `None`, will set `self.dt_patterns`.
    // TODO: [2022/08] having `dt_patterns_update` embedded into this is an unexpected side-affect
    //       the user should have more control over when `dt_patterns_update` is called.
    //       i.e. this approach is hacky
    fn parse_datetime_in_line(
        &mut self,
        line: &Line,
        charsz: &CharSz,
        year_opt: &Option<Year>,
    ) -> ResultParseDateTime {
        defn!("(…, {}, year_opt {:?}) line: {:?}", charsz, year_opt, line.to_String_noraw());

        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats
        // `self.dt_patterns`
        //
        // TODO: [2022/06/26] cost-savings: create the `indexes` once in an analysis update function
        //       or somewhere else
        let mut indexes: DateTimeParseDatasIndexes = DateTimeParseDatasIndexes::new();
        // get copy of indexes sorted by value
        indexes.extend(
            self.dt_patterns_counts
                .iter()
                .sorted_by(
                    |a, b| Ord::cmp(&b.1, &a.1), // sort by value (second tuple item)
                )
                .map(|(k, _v)| k), // copy only the key (first tuple item) which is an index
        );
        defo!("indexes {:?}", indexes);
        let result: ResultFindDateTime = SyslineReader::find_datetime_in_line(
            line,
            &indexes,
            charsz,
            year_opt,
            &self.tz_offset,
            &mut self.get_boxptrs_singleptr,
            &mut self.get_boxptrs_doubleptr,
            &mut self.get_boxptrs_multiptr,
            &mut self.ezcheck12_hit,
            &mut self.ezcheck12_miss,
            &mut self.ezcheck12_hit_max,
            &mut self.ezcheckd2_hit,
            &mut self.ezcheckd2_miss,
            &mut self.ezcheckd2_hit_max,
        );
        let data: FindDateTimeData = match result {
            Ok(val) => val,
            Err(err) => {
                defx!("return Err {};", err);
                return ResultParseDateTime::Err(err);
            }
        };
        self.dt_patterns_update(data.3);
        self.dt_first_last_update(&data.2);
        defx!("return {:?}", data);

        ResultParseDateTime::Ok(data)
    }

    /// Helper function to `find_sysline`.
    ///
    /// Call `self.parse_datetime_in_line` with help of LRU cache
    /// `self.parse_datetime_in_line_lru_cache`.
    fn parse_datetime_in_line_cached(
        &mut self,
        linep: &LineP,
        charsz: &CharSz,
        year_opt: &Option<Year>,
    ) -> ResultParseDateTime {
        if self.parse_datetime_in_line_lru_cache_enabled {
            match self
                .parse_datetime_in_line_lru_cache
                .get(&linep.fileoffset_begin())
            {
                Some(val) => {
                    self.parse_datetime_in_line_lru_cache_hit += 1;
                    return ResultParseDateTime::Ok(*val);
                }
                _ => {
                    self.parse_datetime_in_line_lru_cache_miss += 1;
                }
            }
        }
        defn!("(…, {:?}, {:?})", charsz, year_opt);
        let result: ResultParseDateTime = self.parse_datetime_in_line(&*linep, charsz, year_opt);
        if self.parse_datetime_in_line_lru_cache_enabled {
            match result {
                Ok(val) => {
                    match self
                        .parse_datetime_in_line_lru_cache
                        .put(linep.fileoffset_begin(), val)
                    {
                        Some(val_prev) => {
                            panic!(
                                "self.parse_datetime_in_line_lru_cache already had key {:?}, value {:?}",
                                linep.fileoffset_begin(),
                                val_prev
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        defx!("return {:?}", result);

        result
    }

    /// wrapper for a verbose debug check for `find_sysline`.
    #[inline]
    fn debug_assert_gt_fo_syslineend(fo: &FileOffset, syslinep: &SyslineP) {
        debug_assert_gt!(
            fo,
            &(*syslinep).fileoffset_end(),
            "fo {} ≯ {} syslinep.fileoffset_end()",
            fo,
            (*syslinep).fileoffset_end(),
        );
    }

    /// Check various internal storage for already processed
    /// [`Sysline`] at [`FileOffset`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`FileOffset`]: crate::common::FileOffset
    fn check_store(
        &mut self,
        fileoffset: FileOffset,
    ) -> Option<ResultS3SyslineFind> {
        defn!("({})", fileoffset);

        if self.find_sysline_lru_cache_enabled {
            // check if `fileoffset` is already known about in LRU cache
            match self
                .find_sysline_lru_cache
                .get(&fileoffset)
            {
                Some(result) => {
                    self.find_sysline_lru_cache_hit += 1;
                    defo!("found LRU cached for fileoffset {}", fileoffset);
                    // the `.get` returns a reference `&ResultS3SyslineFind` so must return a new `ResultS3SyslineFind`
                    match result {
                        ResultS3SyslineFind::Found(val) => {
                            defx!(
                                "return ResultS3SyslineFind::Found(({}, …)) @[{}, {}] from LRU cache",
                                val.0,
                                val.1.fileoffset_begin(),
                                val.1.fileoffset_end()
                            );
                            return Some(ResultS3SyslineFind::Found((val.0, val.1.clone())));
                        }
                        ResultS3SyslineFind::Done => {
                            defx!("return ResultS3SyslineFind::Done from LRU cache");
                            return Some(ResultS3SyslineFind::Done);
                        }
                        ResultS3SyslineFind::Err(err) => {
                            defo!("Error {}", err);
                            eprintln!(
                                "ERROR: unexpected value store in self._find_line_lru_cache.get({}) error {}",
                                fileoffset, err
                            );
                        }
                    }
                }
                None => {
                    self.find_sysline_lru_cache_miss += 1;
                    defo!("fileoffset {} not found in LRU cache", fileoffset);
                }
            }
        }

        // check if the offset is already in a known range
        match self
            .syslines_by_range
            .get_key_value(&fileoffset)
        {
            Some(range_fo) => {
                defo!(
                    "hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    fileoffset,
                    range_fo.0
                );
                self.syslines_by_range_hit += 1;
                let fo: &FileOffset = range_fo.1;
                let syslinep: SyslineP = self.syslines[fo].clone();
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                let fo_next: FileOffset = (*syslinep).fileoffset_next();
                if self.is_sysline_last(&syslinep) {
                    defx!(
                        "is_sysline_last() true; return ResultS3SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        fo_next,
                        &syslinep,
                        (*syslinep).fileoffset_begin(),
                        (*syslinep).fileoffset_end(),
                        (*syslinep).to_String_noraw()
                    );
                    self.find_sysline_lru_cache_put += 1;
                    self.find_sysline_lru_cache
                        .put(fileoffset, ResultS3SyslineFind::Found((fo_next, syslinep.clone())));
                    SyslineReader::debug_assert_gt_fo_syslineend(&fo_next, &syslinep);
                    return Some(ResultS3SyslineFind::Found((fo_next, syslinep)));
                }
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS3SyslineFind::Found((fo_next, syslinep.clone())));
                defx!(
                    "is_sysline_last() false; return ResultS3SyslineFind::Found(({}, @{:p})) @[{}, {}] from self.syslines_by_range {:?}",
                    fo_next,
                    &syslinep,
                    (*syslinep).fileoffset_begin(),
                    (*syslinep).fileoffset_end(),
                    (*syslinep).to_String_noraw()
                );
                SyslineReader::debug_assert_gt_fo_syslineend(&fo_next, &syslinep);
                return Some(ResultS3SyslineFind::Found((fo_next, syslinep)));
            }
            None => {
                self.syslines_by_range_miss += 1;
                defo!("fileoffset {} not found in self.syslines_by_range", fileoffset);
            }
        }

        // check if there is a Sysline already known at this fileoffset
        // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
        if self
            .syslines
            .contains_key(&fileoffset)
        {
            debug_assert!(self.syslines_by_range.contains_key(&fileoffset), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}) returned None (syslines_by_range out of synch)", fileoffset, fileoffset);
            self.syslines_hit += 1;
            defo!("hit self.syslines for FileOffset {}", fileoffset);
            let syslinep: SyslineP = self.syslines[&fileoffset].clone();
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            let fo_next: FileOffset = (*syslinep).fileoffset_end() + (self.charsz() as FileOffset);
            if self.is_sysline_last(&syslinep) {
                defo!(
                    "return ResultS3SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                    fo_next,
                    &syslinep,
                    (*syslinep).fileoffset_begin(),
                    (*syslinep).fileoffset_end(),
                    (*syslinep).to_String_noraw()
                );
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS3SyslineFind::Found((fo_next, syslinep.clone())));
                return Some(ResultS3SyslineFind::Found((fo_next, syslinep)));
            }
            if self.find_sysline_lru_cache_enabled {
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS3SyslineFind::Found((fo_next, syslinep.clone())));
            }
            defx!(
                "return ResultS3SyslineFind::Found(({}, @{:p})) @[{}, {}] from self.syslines {:?}",
                fo_next,
                &syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
            return Some(ResultS3SyslineFind::Found((fo_next, syslinep)));
        } else {
            self.syslines_miss += 1;
            defo!("fileoffset {} not found in self.syslines", fileoffset);
        }
        defx!("return None");

        None
    }

    /// Find [`Sysline`] at `FileOffset` within the same `Block`
    /// (does not cross `Block` boundaries).
    ///
    /// This does a linear search over the `Block`, _O(n)_.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub fn find_sysline_in_block(
        &mut self,
        fileoffset: FileOffset,
    ) -> (ResultS3SyslineFind, bool) {
        self.find_sysline_in_block_year(fileoffset, &None)
    }

    /// Implementation of find [`Sysline`] at fileoffset within the same `Block`
    /// (does not cross block boundaries).
    ///
    /// This does a linear search over the `Block`, _O(n)_.
    ///
    /// Optional `Year` is the filler year for datetime patterns that do not include a year.
    /// e.g. `"Jan 1 12:00:00 this is a syslog message"`
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    // XXX: similar to `find_sysline`:
    //      This function `find_sysline_in_block_year` is large and cumbersome.
    //      Changes require extensive retesting.
    //      Extensive debug prints are left in place to aid this.
    //      It could use some improvements but for now it gets the job done.
    //      You've been warned.
    //
    pub fn find_sysline_in_block_year(
        &mut self,
        fileoffset: FileOffset,
        year_opt: &Option<Year>,
    ) -> (ResultS3SyslineFind, bool) {
        defn!("({}, {:?})", fileoffset, year_opt);

        if let Some(result) = self.check_store(fileoffset) {
            defx!("({}): return {:?}, false", fileoffset, result);
            return (result, false);
        }

        defo!("({}): searching for first sysline datetime A …", fileoffset);

        let mut _fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sysline = Sysline::new();
        loop {
            defo!("({}): self.linereader.find_line_in_block({})", fileoffset, fo1);
            let result_ = self
                .linereader
                .find_line_in_block(fo1);
            let result: ResultS3LineFind = result_.0;
            let partial = result_.1;
            let (fo2, linep) = match result {
                ResultS3LineFind::Found((fo_, linep_)) => {
                    defo!(
                        "({}): A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );

                    (fo_, linep_)
                }
                ResultS3LineFind::Done => {
                    defo!(
                        "({}): A from LineReader.find_line_in_block({}), partial {:?}",
                        fileoffset,
                        fo1,
                        partial,
                    );
                    match partial {
                        Some(line) => {
                            // received Done but also a partial Line (the terminating Newline was
                            // not found in the current Block). Try to match a datetime on this
                            // partial Line. Do not store this Line, do not create a new Sysline.
                            let result = self.parse_datetime_in_line_cached(&LineP::new(line), &self.charsz(), year_opt);
                            defo!("({}): partial parse_datetime_in_line_cached returned {:?}", fileoffset, result);
                            match result {
                                Err(_) => {},
                                Ok((_dt_beg, _dt_end, _dt, _index)) => {
                                    defo!("({}): partial Line datetime found: {:?}", fileoffset, _dt);
                                    defx!("({}): return ResultS3SyslineFind::Done, true", fileoffset);
                                    return (ResultS3SyslineFind::Done, true);
                                }
                            }
                        }
                        None => {}
                    }
                    defx!("({}): return ResultS3SyslineFind::Done, false", fileoffset);
                    return (ResultS3SyslineFind::Done, false);
                }
                ResultS3LineFind::Err(err) => {
                    de_err!("LineReader.find_line_in_block({}) returned {}", fo1, err);
                    defx!(
                        "({}): return ResultS3SyslineFind::Err({}), false; A from LineReader.find_line_in_block({})",
                        fileoffset,
                        err,
                        fo1
                    );
                    return (ResultS3SyslineFind::Err(err), false);
                }
            };
            let result: ResultParseDateTime =
                self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            defo!("({}): A parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {}
                // FindDateTimeData:
                // (LineIndex, LineIndex, DateTimeL, DateTimeParseInstrsIndex);
                Ok((dt_beg, dt_end, dt, _index)) => {
                    // a datetime was found! beginning of a sysline
                    _fo_a = fo1;
                    sysline.dt_beg = dt_beg;
                    sysline.dt_end = dt_end;
                    sysline.dt = Some(dt);
                    defo!(
                        "({}): A sl.dt_beg {}, sl.dt_end {}, sl.push({:?})",
                        fileoffset,
                        dt_beg,
                        dt_end,
                        (*linep).to_String_noraw()
                    );
                    sysline.push(linep);
                    fo1 = sysline.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_le!(
                        dt_end,
                        fo1 as usize,
                        "bad dt_end {} fileoffset+charsz {}",
                        dt_end,
                        fo1 as usize
                    );
                    if self.is_sysline_last(&sysline) {
                        let syslinep: SyslineP = self.insert_sysline(sysline);
                        if self.find_sysline_lru_cache_enabled {
                            self.find_sysline_lru_cache_put += 1;
                            defo!("({}): LRU cache put({}, Found({}, …))", fileoffset, fileoffset, fo1);
                            self.find_sysline_lru_cache
                                .put(fileoffset, ResultS3SyslineFind::Found((fo1, syslinep.clone())));
                        }
                        defx!(
                            "({}): return ResultS3SyslineFind::Found({}, {:p}), false; @[{}, {}]; A found here and LineReader.find_line({})",
                            fileoffset,
                            fo1,
                            &(*syslinep),
                            (*syslinep).fileoffset_begin(),
                            (*syslinep).fileoffset_end(),
                            fo1,
                        );
                        SyslineReader::debug_assert_gt_fo_syslineend(&fo1, &syslinep);
                        return (ResultS3SyslineFind::Found((fo1, syslinep)), false);
                    }
                    break;
                }
            }
            defo!("({}): A skip push Line {:?}", fileoffset, (*linep).to_String_noraw());
            fo1 = fo2;
        }

        defo!(
            "({}): found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            fileoffset,
            _fo_a,
            fo1,
        );

        //
        // find line with datetime B
        //

        let fo_b: FileOffset;
        loop {
            defo!("({}): self.linereader.find_line_in_block({})", fileoffset, fo1);
            let result_ = self
                .linereader
                .find_line_in_block(fo1);
            let result: ResultS3LineFind = result_.0;
            let (fo2, linep) = match result {
                ResultS3LineFind::Found((fo_, linep_)) => {
                    defo!(
                        "({}): B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );

                    (fo_, linep_)
                }
                ResultS3LineFind::Done => {
                    defo!("({}): B got Done", fileoffset);
                    if fo1 < self.fileoffset_last() {
                        // Line search did not find a whole Line *before* getting to end
                        // of the block. Cannot be sure if where this Sysline ends until a
                        // proceeding Sysline is found (or end of file reached). Must return Done.
                        defx!(
                            "({}): return ResultS3SyslineFind::Done, true; B from LineReader.find_line_in_block({})",
                            fileoffset,
                            fo1
                        );
                        return (ResultS3SyslineFind::Done, true);
                    }
                    // line search is exhausted, force `fo_b` to "point" to
                    // the known `sysline.fileoffset_end()`
                    fo_b = sysline.fileoffset_end() + self.charsz() as FileOffset;
                    break;
                }
                ResultS3LineFind::Err(err) => {
                    de_err!("LineReader.find_line_in_block({}) returned {}", fo1, err);
                    defx!(
                        "({}): return ResultS3SyslineFind::Err({}), false; B from LineReader.find_line_in_block({})",
                        fileoffset,
                        err,
                        fo1
                    );
                    return (ResultS3SyslineFind::Err(err), false);
                }
            };

            let result: ResultParseDateTime =
                self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            defo!("({}): B parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {
                    defo!(
                        "({}): B append found Line to this Sysline sl.push({:?})",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    sysline.push(linep);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    defo!(
                        "({}): B found datetime; end of this Sysline. Do not append found Line {:?}",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        } // loop

        defo!(
            "({}): found sysline with datetime B at FileOffset {} dt {:?} {:?}",
            fileoffset,
            fo_b,
            sysline.dt,
            sysline.to_String_noraw(),
        );

        let syslinep: SyslineP = self.insert_sysline(sysline);
        if self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_put += 1;
            defo!("({}): LRU cache put({}, Found({}, …))", fileoffset, fileoffset, fo_b);
            self.find_sysline_lru_cache
                .put(fileoffset, ResultS3SyslineFind::Found((fo_b, syslinep.clone())));
        }
        defx!(
            "({}): return ResultS3SyslineFind::Found(({}, SyslineP@{:p})), false; @[{}, {}] E {:?}",
            fileoffset,
            fo_b,
            &syslinep,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).to_String_noraw()
        );
        SyslineReader::debug_assert_gt_fo_syslineend(&fo_b, &syslinep);

        (ResultS3SyslineFind::Found((fo_b, syslinep)), false)
    }

    /// Find first [`Sysline`] starting at or after `FileOffset`.
    /// Returns
    /// (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`).
    ///
    /// Similar to `LineReader.find_line`, `BlockReader.read_block`.
    ///
    /// This does a linear search _O(n)_ over the file.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub fn find_sysline(
        &mut self,
        fileoffset: FileOffset,
    ) -> ResultS3SyslineFind {
        self.find_sysline_year(fileoffset, &None)
    }

    /// Find first [`Sysline`] starting at or after `FileOffset`.
    /// Returns
    /// (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`).
    ///
    /// Optional `Year` is the filler year for datetime patterns that do not include a year.
    /// e.g. `"Jan 1 12:00:00 this is a syslog message"`
    ///
    /// Similar to `LineReader.find_line`.
    ///
    /// This does a linear search _O(n)_ over the file.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    //
    // XXX: This function `find_sysline` is large and cumbersome.
    //      Changes require extensive retesting.
    //      Extensive debug prints are left in place to aid this.
    //      It could use a few improvements but for now it gets the job done.
    //      You've been warned.
    //
    pub fn find_sysline_year(
        &mut self,
        fileoffset: FileOffset,
        year_opt: &Option<Year>,
    ) -> ResultS3SyslineFind {
        defn!("({}, {:?})", fileoffset, year_opt);

        if let Some(result) = self.check_store(fileoffset) {
            defx!("({}): return {:?}", fileoffset, result);
            return result;
        }

        let charsz_fo: FileOffset = self.charsz() as FileOffset;

        defo!("({}): searching for first sysline datetime A …", fileoffset);

        //
        // find line with datetime A
        //

        // FileOffset ZERO has been TRIED?
        let mut fo_zero_tried: bool = false;
        // FileOffset A
        let mut _fo_a: FileOffset = 0;
        // FileOffset A MAXimum
        let mut fo_a_max: FileOffset = 0;
        // FileOffset
        let mut fo1: FileOffset = fileoffset;
        // the new Sysline instance
        let mut sysline = Sysline::new();

        loop {
            defo!("({}): self.linereader.find_line({})", fileoffset, fo1);
            let result: ResultS3LineFind = self.linereader.find_line(fo1);
            let (fo2, linep) = match result {
                ResultS3LineFind::Found((fo_, linep_)) => {
                    defo!(
                        "A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    (fo_, linep_)
                }
                ResultS3LineFind::Done => {
                    if self.find_sysline_lru_cache_enabled {
                        self.find_sysline_lru_cache_put += 1;
                        defo!("({}): LRU cache put({}, Done)", fileoffset, fileoffset);
                        self.find_sysline_lru_cache
                            .put(fileoffset, ResultS3SyslineFind::Done);
                    }
                    defx!(
                        "({}): return ResultS3SyslineFind::Done; A from LineReader.find_line({})",
                        fileoffset,
                        fo1
                    );
                    return ResultS3SyslineFind::Done;
                }
                ResultS3LineFind::Err(err) => {
                    de_err!("LineReader.find_line({}) returned {}", fo1, err);
                    defx!(
                        "({}): return ResultS3SyslineFind::Err({}); A from LineReader.find_line({})",
                        fileoffset,
                        err,
                        fo1
                    );
                    return ResultS3SyslineFind::Err(err);
                }
            };
            fo_a_max = std::cmp::max(fo_a_max, fo2);
            let result: ResultParseDateTime =
                self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            defo!("({}): A parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {
                    // a datetime was not found in the Line! (this is normal behavior)
                }
                Ok((dt_beg, dt_end, dt, _index)) => {
                    // a datetime was found! beginning of a sysline
                    _fo_a = fo1;
                    sysline.dt_beg = dt_beg;
                    sysline.dt_end = dt_end;
                    sysline.dt = Some(dt);
                    defo!("({}): A sl.push({:?})", fileoffset, (*linep).to_String_noraw());
                    sysline.push(linep);
                    fo1 = sysline.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}, dt {:?}", dt_beg, dt_end, dt);
                    debug_assert_le!(
                        dt_end,
                        fo1 as usize,
                        "bad dt_end {} fileoffset+charsz {}, dt {:?}",
                        dt_end,
                        fo1 as usize,
                        dt
                    );
                    break;
                }
            }
            defo!("({}): A skip push Line {:?}", fileoffset, (*linep).to_String_noraw());
            let line_beg: FileOffset = (*linep).fileoffset_begin();
            if fo_zero_tried {
                // search forwards...
                // have already searched for a datetime stamp all the way back to zero'th byte of file
                // so switch search direction to go forward. these first few lines without a datetime stamp
                // will be ignored.
                // TODO: [2022/07] somehow inform user that some lines were not processed.
                fo1 = fo_a_max;
            } else if line_beg > charsz_fo {
                // search backwards...
                fo1 = line_beg - charsz_fo;
                // TODO: cost-savings: searching `self.syslines_by_range` is surprisingly expensive.
                //       Consider adding a faster, simpler `HashMap` that only tracks
                //       `sysline.fileoffset_end` keys to `fileoffset_begin` values.
                if self
                    .syslines_by_range
                    .contains_key(&fo1)
                {
                    // ran into prior processed sysline; something is odd. Abandon these lines
                    // and change search direction to go forwards
                    // TODO: Issue #61 enable expression attribute when feature is stable
                    //       #[allow(unused_assignments)]
                    fo_zero_tried = true;
                    fo1 = fo_a_max;
                }
            } else {
                // search from byte zero
                fo1 = 0;
                fo_zero_tried = true;
            }
        } // loop

        defo!(
            "({}): found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            fileoffset,
            _fo_a,
            fo1
        );

        //
        // find line with datetime B
        //

        // TODO: 2022/06/18 uncomment this check?
        { // check if sysline at `fo1` is already known about
             /*
             // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
             // check if there is a Sysline already known at this fileoffset
             if self.syslines.contains_key(&fo1) {
                 debug_assert!(self.syslines_by_range.contains_key(&fo1), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}); syslines_by_range out of synch", fo1, fo1);
                 deo!("find_sysline: hit self.syslines for FileOffset {}", fo1);
                 let syslinep = self.syslines[&fo1].clone();
                 // XXX: Issue #16 only handles UTF-8/ASCII encoding
                 let fo_next = (*syslinep).fileoffset_end() + (self.charsz() as FileOffset);
                 // TODO: determine if `fileoffset` is the last sysline of the file
                 //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                 defx!(
                 "return ResultS3SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                 fo_next,
                 &syslinep,
                 (*syslinep).fileoffset_begin(),
                 (*syslinep).fileoffset_end(),
                 (*syslinep).to_String_noraw()
             );
                 if self.find_sysline_lru_cache_enabled {
                     self.find_sysline_lru_cache_put += 1;
                     self.find_sysline_lru_cache
                         .put(fileoffset, ResultS3SyslineFind::Found((fo_next, syslinep.clone())));
                 }
                 return ResultS3SyslineFind::Found((fo_next, syslinep));
             } else {
                 deo!("find_sysline: fileoffset {} not found in self.syslines", fileoffset);
             }
             // check if the offset is already in a known range
             match self.syslines_by_range.get_key_value(&fo1) {
                 Some(range_fo) => {
                     let range = range_fo.0;
                     defo!(
                     "hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                     fo1,
                     range
                 );
                     self.syslines_by_range_hit += 1;
                     let fo = range_fo.1;
                     let syslinep = self.syslines[fo].clone();
                     // XXX: Issue #16 only handles UTF-8/ASCII encoding
                     let fo_next = (*syslinep).fileoffset_next() + (self.charsz() as FileOffset);
                     if self.find_sysline_lru_cache_enabled {
                         self.find_sysline_lru_cache_put += 1;
                         self.find_sysline_lru_cache
                             .put(fileoffset, ResultS3SyslineFind::Found((fo_next, syslinep.clone())));
                     }
                     defx!(
                         "return ResultS3SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                         fo_next,
                         &syslinep
                         (*syslinep).fileoffset_begin(),
                         (*syslinep).fileoffset_end(),
                         (*syslinep).to_String_noraw()
                     );
                     return ResultS3SyslineFind::Found((fo_next, syslinep));
                 }
                 None => {
                     self.syslines_by_range_miss += 1;
                     deo!("find_sysline: fileoffset {} not found in self.syslines_by_range", fileoffset);
                 }
             }
             deo!("find_sysline: searching for first sysline datetime B …");
             */
        }

        let mut fo_b: FileOffset = fo1;
        defo!("({}): fo_b {:?}", fileoffset, fo_b);
        loop {
            defo!("({}): self.linereader.find_line({})", fileoffset, fo1);
            let result: ResultS3LineFind = self.linereader.find_line(fo1);
            let (fo2, linep) = match result {
                ResultS3LineFind::Found((fo_, linep_)) => {
                    defo!(
                        "({}): B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );

                    (fo_, linep_)
                }
                ResultS3LineFind::Done => {
                    defo!("({}): break; B", fileoffset);
                    break;
                }
                ResultS3LineFind::Err(err) => {
                    de_err!("LineReader.find_line({}) returned {}", fo1, err);
                    defx!(
                        "({}): return ResultS3SyslineFind::Err({}); B from LineReader.find_line({})",
                        fileoffset,
                        err,
                        fo1
                    );
                    return ResultS3SyslineFind::Err(err);
                }
            };
            let result: ResultParseDateTime =
                self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            defo!("({}): B parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {
                    // a datetime was not found in the Line! This line is also part of this sysline
                    defo!(
                        "({}): B append found Line to this Sysline sl.push({:?})",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    sysline.push(linep);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    defo!(
                        "({}): B found datetime; end of this Sysline. Do not append found Line {:?}",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
            fo_b = fo1;
        }

        defo!(
            "({}): found sysline with datetime B at FileOffset {} {:?} {:?}",
            fileoffset,
            fo_b,
            sysline.dt,
            sysline.to_String_noraw(),
        );

        let syslinep: SyslineP = self.insert_sysline(sysline);

        if self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_put += 1;
            defo!("({}): LRU cache put({}, Found({}, …))", fileoffset, fileoffset, fo_b);
            self.find_sysline_lru_cache
                .put(fileoffset, ResultS3SyslineFind::Found((fo_b, syslinep.clone())));
        }
        defx!(
            "({}): return ResultS3SyslineFind::Found(({}, SyslineP@{:p})) @[{}, {}] F {:?}",
            fileoffset,
            fo_b,
            &syslinep,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).to_String_noraw()
        );
        SyslineReader::debug_assert_gt_fo_syslineend(&fo_b, &syslinep);

        ResultS3SyslineFind::Found((fo_b, syslinep))
    }

    /// Find first [`Sysline`] at or after [`FileOffset`] that is at or
    /// after `dt_filter`.
    ///
    /// This does a binary search over the file, _O(log(n))_.
    ///
    /// For example, given syslog file with datetimes:
    ///
    /// ```text
    /// 20010101
    /// 20010102
    /// 20010103
    /// ```
    ///
    /// where the newline ending the first line is the ninth byte
    /// (fileoffset 9). So calling
    ///
    /// ```text
    /// syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))
    /// ```
    ///
    /// will return
    ///
    /// ```text
    /// ResultS3::Found(19, SyslineP(data='20010102␊'))
    /// ```
    ///
    /// For syslog files where the datetime does not include a year, prior
    /// processing must occur to make guesstimates for each `Sysline`'s
    /// real year.
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    /// [`Sysline`]: crate::data::sysline::Sysline
    //
    // XXX: This function is large, cumbersome, and messy.
    //      Changes require extensive retesting.
    //      Extensive debug prints are left in place to aid this.
    //      You've been warned.
    //
    // TODO: [2022/06] rename function to `find_next_sysline_at_datetime_filter`, rename all `find_` functions to either
    //       `find_..._between_`, `find_...at_`, or `find_next`,
    //       `between` and `at` should mean binary search over the file
    //       `next` should mean linear sequantial search
    //
    // TODO: [2022/07/25] enforce by assertion
    //       if the in-use datetime pattern has no year, then year processing has already occurred.
    //
    pub fn find_sysline_at_datetime_filter(
        &mut self,
        fileoffset: FileOffset,
        // TODO: [2023/02/26] type alias for `Option<DateTimeL>` that are meant
        //       as broad filters versus meant as instance values.
        dt_filter: &DateTimeLOpt,
    ) -> ResultS3SyslineFind {
        defn!("(SyslineReader@{:p}, {}, {:?})", self, fileoffset, dt_filter);
        let filesz: FileSz = self.filesz();
        let _fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        #[allow(unused_assignments)]
        let mut try_fo_last: FileOffset = try_fo;
        #[allow(unused_assignments)]
        let mut syslinep_opt: Option<SyslineP> = None;
        let mut fo_a: FileOffset = fileoffset; // begin "range cursor" marker
        let mut fo_b: FileOffset = _fo_end; // end "range cursor" marker
        loop {
            // TODO: [2021/09/26]
            //       this could be faster.
            //       currently it narrowing down to a byte offset
            //       but it only needs to narrow down to offsets within range of one sysline
            //       so if `fo_a` and `fo_b` are in same sysline range, then this can return that sysline.
            //       Also, add stats for this function and debug print those stats before exiting.
            //       i.e. count of loops, count of calls to sysline_dt_before_after, etc.
            //       do this before tweaking function so can be compared
            defo!("loop(…)!");
            let result: ResultS3SyslineFind = self.find_sysline(try_fo);
            let done = result.is_done();
            match result {
                ResultS3SyslineFind::Found((fo, syslinep)) => {
                    defo!(
                        "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?} C",
                        fo,
                        &(*syslinep),
                        syslinep.lines.len(),
                        (*syslinep).len(),
                        (*syslinep).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    defo!(
                        "sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        &syslinep,
                        (*syslinep).dt,
                        dt_filter,
                    );
                    match SyslineReader::sysline_dt_after_or_before(&syslinep, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            defo!(
                                "Pass => fo {} try_fo {} try_fo_last {} (fo_end {})",
                                fo,
                                try_fo,
                                try_fo_last,
                                _fo_end,
                            );
                            defx!("return ResultS3SyslineFind::Found(({}, @{:p})); A", fo, &syslinep,);
                            SyslineReader::debug_assert_gt_fo_syslineend(&fo, &syslinep);
                            return ResultS3SyslineFind::Found((fo, syslinep));
                        } // end Pass
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            defo!("OccursAtOrAfter => fo {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", fo, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            // short-circuit a common case, passed fileoffset is past the `dt_filter`, can immediately return
                            // XXX: does this mean my algorithm sucks?
                            if try_fo == fileoffset {
                                // first loop iteration
                                defo!(
                                    "                    try_fo {} == {} try_fo_last; early return",
                                    try_fo,
                                    try_fo_last,
                                );
                                defx!(
                                    "return ResultS3SyslineFind::Found(({}, @{:p})); B fileoffset {} {:?}",
                                    fo,
                                    &syslinep,
                                    (*syslinep).fileoffset_begin(),
                                    (*syslinep).to_String_noraw(),
                                );
                                SyslineReader::debug_assert_gt_fo_syslineend(&fo, &syslinep);
                                return ResultS3SyslineFind::Found((fo, syslinep));
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*syslinep).fileoffset_begin(), try_fo_last);
                            defo!(
                                "                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                fo_a,
                                fo_b,
                                fo_a,
                            );
                            assert_le!(
                                fo_a,
                                fo_b,
                                "Unexpected values for fo_a {} fo_b {}, FPath {:?}",
                                fo_a,
                                fo_b,
                                self.path()
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursAtOrAfter
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            defo!("OccursBefore =>    fo {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", fo, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            let syslinep_foe: FileOffset = (*syslinep).fileoffset_end();
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, syslinep_foe, "Unexpected values try_fo_last {} syslinep_foe {}, last tried offset (passed to self.find_sysline({})) is beyond returned Sysline@{:p}.fileoffset_end() {}!? FPath {:?}", try_fo_last, syslinep_foe, try_fo, syslinep, syslinep_foe, self.path());
                            defo!(
                                "                    ∴ fo_a = min(syslinep_foe {}, fo_b {});",
                                syslinep_foe,
                                fo_b
                            );
                            fo_a = std::cmp::min(syslinep_foe, fo_b);
                            defo!(
                                "                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                fo_a,
                                fo_b,
                                fo_a,
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    defo!("                    fo {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", fo, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                    syslinep_opt = Some(syslinep);
                    // TODO: [2021/09/26]
                    //       I think this could do an early check and potentially skip a few loops.
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Done
                } // end Found
                ResultS3SyslineFind::Done => {
                    defo!("SyslineReader.find_sysline(try_fo: {}) returned Done", try_fo);
                    defo!(
                        "                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                    try_fo_last = try_fo;
                    defo!(
                        "                 ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                        fo_a,
                        fo_b,
                        fo_a
                    );
                    try_fo = fo_a + ((fo_b - fo_a) / 2);
                    defo!(
                        "                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                } // end Done
                ResultS3SyslineFind::Err(_err) => {
                    defo!("SyslineReader.find_sysline(try_fo: {}) returned Err({})", try_fo, _err,);
                    de_err!("{}", _err);
                    break;
                } // end Err
            } // match result
            defo!("next loop will try offset {} (fo_end {})", try_fo, _fo_end);

            // TODO: 2022/03/18 this latter part hints at a check that could be done sooner,
            //       before `try_fo==try_fo_last`, that would result in a bit less loops.
            //       A simpler and faster check is to do
            //           fo_next, syslinep = find_sysline(fileoffset)
            //           _, syslinep_next = find_sysline(fo_next)
            //       do this at the top of the loop. Then call `dt_after_or_before` for each
            //       `.dt` among `syslinep`, `syslinep_next`.

            // `try_fo == try_fo_last` means binary search loop is deciding on the same fileoffset
            // upon each loop. the searching is exhausted.
            if done && try_fo == try_fo_last {
                // reached a dead-end of searching the same fileoffset `find_sysline(try_fo)` and
                // receiving Done, so this function is exhausted too.
                defo!("Done && try_fo {} == {} try_fo_last; break!", try_fo, try_fo_last);
                break;
            } else if try_fo != try_fo_last {
                continue;
            }
            defo!("try_fo {} == {} try_fo_last;", try_fo, try_fo_last);
            let mut syslinep = syslinep_opt.unwrap();
            let fo_beg: FileOffset = syslinep.fileoffset_begin();
            if self.is_sysline_last(&syslinep) && fo_beg < try_fo {
                // binary search stopped at fileoffset past start of last Sysline in file
                // so entirely past all acceptable syslines
                defx!("return ResultS3SyslineFind::Done; C binary searched ended after beginning of last sysline in the file");
                return ResultS3SyslineFind::Done;
            }
            // binary search loop is deciding on the same fileoffset upon each loop. That
            // fileoffset must refer to an acceptable sysline. However, if that fileoffset is past
            // `syslinep.fileoffset_begin` than the threshold change of datetime for the
            // `dt_filter` is the *next* Sysline.
            let fo_next: FileOffset = syslinep.fileoffset_next();
            if fo_beg < try_fo {
                defo!("syslinep.fileoffset_begin() {} < {} try_fo;", fo_beg, try_fo);
                let syslinep_next: SyslineP = match self.find_sysline(fo_next) {
                    ResultS3SyslineFind::Found((_, syslinep_)) => {
                        defo!(
                            "SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                            fo_next,
                            syslinep_
                        );
                        syslinep_
                    }
                    ResultS3SyslineFind::Done => {
                        defo!("SyslineReader.find_sysline(fo_next1: {}) unexpectedly returned Done", fo_next);
                        break;
                    }
                    ResultS3SyslineFind::Err(_err) => {
                        defo!("SyslineReader.find_sysline(fo_next1: {}) returned Err({})", fo_next, _err,);
                        de_err!("{}", _err);
                        break;
                    }
                };
                defo!("dt_filter:                   {:?}", dt_filter);
                defo!(
                    "syslinep      : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    fo_beg,
                    (*syslinep).fileoffset_end(),
                    (*syslinep).dt.unwrap(),
                    (*syslinep).to_String_noraw()
                );
                defo!(
                    "syslinep_next : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    (*syslinep_next).fileoffset_begin(),
                    (*syslinep_next).fileoffset_end(),
                    (*syslinep_next).dt.unwrap(),
                    (*syslinep_next).to_String_noraw()
                );
                let syslinep_compare = dt_after_or_before(&(*syslinep).dt.unwrap(), dt_filter);
                let syslinep_next_compare = dt_after_or_before(&(*syslinep_next).dt.unwrap(), dt_filter);
                defo!("match({:?}, {:?})", syslinep_compare, syslinep_next_compare);
                syslinep = match (syslinep_compare, syslinep_next_compare) {
                    (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                        defo!("unexpected Result_Filter_DateTime1::Pass");
                        eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                        break;
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                        defo!("choosing syslinep_next");
                        syslinep_next
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        defo!("choosing syslinep_next");
                        syslinep_next
                    }
                    (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        defo!("choosing syslinep");
                        syslinep
                    }
                    _ => {
                        defo!("unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                        eprintln!(
                            "ERROR: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple"
                        );
                        break;
                    }
                };
            } else {
                defo!("syslinep.fileoffset_begin() {} >= {} try_fo; use syslinep", fo_beg, try_fo);
            }
            let fo_: FileOffset = syslinep.fileoffset_next();
            defx!(
                "return ResultS3SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}",
                fo_,
                &syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).to_String_noraw()
            );
            SyslineReader::debug_assert_gt_fo_syslineend(&fo_, &syslinep);
            return ResultS3SyslineFind::Found((fo_, syslinep));
        } // end loop

        defx!("return ResultS3SyslineFind::Done; E");

        ResultS3SyslineFind::Done
    }

    /// Wrapper function for [`dt_after_or_before`].
    ///
    /// [`dt_after_or_before`]: crate::data::datetime::dt_after_or_before
    pub fn sysline_dt_after_or_before(
        syslinep: &SyslineP,
        dt_filter: &DateTimeLOpt,
    ) -> Result_Filter_DateTime1 {
        defñ!(
            "(Sysline@[{:?}, {:?}], {:?})",
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            dt_filter,
        );
        assert!((*syslinep).dt.is_some(), "Sysline@{:p} does not have a datetime set.", &syslinep);

        let dt: &DateTimeL = (*syslinep)
            .dt
            .as_ref()
            .unwrap();

        dt_after_or_before(dt, dt_filter)
    }

    /// Wrapper function for [`dt_pass_filters`].
    ///
    /// [`dt_pass_filters`]: crate::data::datetime::dt_pass_filters
    pub fn sysline_pass_filters(
        syslinep: &SyslineP,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> Result_Filter_DateTime2 {
        defn!(
            "(Sysline[{:?}, {:?}], {:?}, {:?})",
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            dt_filter_after,
            dt_filter_before,
        );
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &syslinep);
        let dt: &DateTimeL = (*syslinep)
            .dt
            .as_ref()
            .unwrap();
        let result: Result_Filter_DateTime2 = dt_pass_filters(dt, dt_filter_after, dt_filter_before);
        defx!("(…) return {:?};", result);

        result
    }

    /// Find the first [`Sysline`], starting at `FileOffset`,
    /// that is at or after datetime filter
    /// `dt_filter_after` and before datetime filter `dt_filter_before`.
    ///
    /// This uses `self.find_sysline_at_datetime_filter`
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub fn find_sysline_between_datetime_filters(
        &mut self,
        fileoffset: FileOffset,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> ResultS3SyslineFind {
        defn!("({}, {:?}, {:?})", fileoffset, dt_filter_after, dt_filter_before);

        match self.find_sysline_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS3SyslineFind::Found((fo, syslinep)) => {
                defo!("returned ResultS3SyslineFind::Found(({}, {:?}))", fo, syslinep);
                match Self::sysline_pass_filters(&syslinep, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::InRange => {
                        defo!("sysline_pass_filters(…) returned InRange;");
                        defx!("return ResultS3SyslineFind::Found(({}, {:?}))", fo, syslinep);
                        SyslineReader::debug_assert_gt_fo_syslineend(&fo, &syslinep);
                        return ResultS3SyslineFind::Found((fo, syslinep));
                    }
                    Result_Filter_DateTime2::BeforeRange => {
                        defo!("sysline_pass_filters(…) returned BeforeRange;");
                        eprintln!("ERROR: sysline_pass_filters(Sysline@{:p}, {:?}, {:?}) returned BeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  syslinep, dt_filter_after, dt_filter_before,
                                  fileoffset, dt_filter_after
                        );
                        defx!("return ResultS3SyslineFind::Done (not sure what to do here)");
                        return ResultS3SyslineFind::Done;
                    }
                    Result_Filter_DateTime2::AfterRange => {
                        defo!("sysline_pass_filters(…) returned AfterRange;");
                        defx!("return ResultS3SyslineFind::Done");
                        return ResultS3SyslineFind::Done;
                    }
                };
            }
            ResultS3SyslineFind::Done => {}
            ResultS3SyslineFind::Err(err) => {
                defo!("({}, dt_after: {:?}) returned Err({})", fileoffset, dt_filter_after, err);
                de_err!("{}", err);
                defx!("return ResultS3SyslineFind::Err({})", err);
                return ResultS3SyslineFind::Err(err);
            }
        };

        defx!("return ResultS3SyslineFind::Done");

        ResultS3SyslineFind::Done
    }

    pub fn summary(&self) -> SummarySyslineReader {
        let syslinereader_syslines = self
            .count_syslines_processed();
        let syslinereader_syslines_stored_highest = self
            .syslines_stored_highest();
        let syslinereader_syslines_hit = self
            .syslines_hit;
        let syslinereader_syslines_miss = self
            .syslines_miss;
        let syslinereader_syslines_by_range_hit = self
            .syslines_by_range_hit;
        let syslinereader_syslines_by_range_miss = self
            .syslines_by_range_miss;
        let syslinereader_syslines_by_range_put = self
            .syslines_by_range_put;
        // only print patterns with use count > 0, sorted by count
        let mut syslinereader_patterns_ = DateTimePatternCounts::new();
        syslinereader_patterns_.extend(
            self
                .dt_patterns_counts
                .iter()
                .filter(|&(_k, v)| v > &mut 0),
        );
        let mut syslinereader_patterns = DateTimePatternCounts::new();
        syslinereader_patterns.extend(
            syslinereader_patterns_
                .into_iter()
                .sorted_by(|a, b| Ord::cmp(&b.1, &a.1)),
        );
        let syslinereader_find_sysline_lru_cache_hit = self
            .find_sysline_lru_cache_hit;
        let syslinereader_find_sysline_lru_cache_miss = self
            .find_sysline_lru_cache_miss;
        let syslinereader_find_sysline_lru_cache_put = self
            .find_sysline_lru_cache_put;
        let syslinereader_parse_datetime_in_line_lru_cache_hit = self
            .parse_datetime_in_line_lru_cache_hit;
        let syslinereader_parse_datetime_in_line_lru_cache_miss = self
            .parse_datetime_in_line_lru_cache_miss;
        let syslinereader_parse_datetime_in_line_lru_cache_put = self
            .parse_datetime_in_line_lru_cache_put;
        let syslinereader_get_boxptrs_singleptr = self
            .get_boxptrs_singleptr;
        let syslinereader_get_boxptrs_doubleptr = self
            .get_boxptrs_doubleptr;
        let syslinereader_get_boxptrs_multiptr = self
            .get_boxptrs_multiptr;
        let syslinereader_drop_sysline_ok = self
            .drop_sysline_ok;
        let syslinereader_drop_sysline_errors = self
            .drop_sysline_errors;
        let syslinereader_ezcheck12_hit = self.ezcheck12_hit;
        let syslinereader_ezcheck12_miss = self.ezcheck12_miss;
        let syslinereader_ezcheck12_hit_max = self.ezcheck12_hit_max;
        let syslinereader_ezcheckd2_hit = self.ezcheckd2_hit;
        let syslinereader_ezcheckd2_miss = self.ezcheckd2_miss;
        let syslinereader_ezcheckd2_hit_max = self.ezcheckd2_hit_max;
        let syslinereader_datetime_first = self.dt_first;
        let syslinereader_datetime_last = self.dt_last;

        SummarySyslineReader {
            syslinereader_syslines,
            syslinereader_syslines_stored_highest,
            syslinereader_syslines_hit,
            syslinereader_syslines_miss,
            syslinereader_syslines_by_range_hit,
            syslinereader_syslines_by_range_miss,
            syslinereader_syslines_by_range_put,
            syslinereader_patterns,
            syslinereader_find_sysline_lru_cache_hit,
            syslinereader_find_sysline_lru_cache_miss,
            syslinereader_find_sysline_lru_cache_put,
            syslinereader_parse_datetime_in_line_lru_cache_hit,
            syslinereader_parse_datetime_in_line_lru_cache_miss,
            syslinereader_parse_datetime_in_line_lru_cache_put,
            syslinereader_get_boxptrs_singleptr,
            syslinereader_get_boxptrs_doubleptr,
            syslinereader_get_boxptrs_multiptr,
            syslinereader_drop_sysline_ok,
            syslinereader_drop_sysline_errors,
            syslinereader_ezcheck12_hit,
            syslinereader_ezcheck12_miss,
            syslinereader_ezcheck12_hit_max,
            syslinereader_ezcheckd2_hit,
            syslinereader_ezcheckd2_miss,
            syslinereader_ezcheckd2_hit_max,
            syslinereader_datetime_first,
            syslinereader_datetime_last,
        }
    }
}
