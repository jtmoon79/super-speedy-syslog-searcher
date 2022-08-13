// src/readers/syslinereader.rs
// …

//! Implements `SyslineReader`, the driver of deriving [`Sysline`]s using a
//! [`LineReader`].
//!
//! [`Sysline`]: crate::data::sysline::Sysline
//! [`LineReader`]: crate::readers::linereader::LineReader

use crate::common::{
    FPath,
    FileOffset,
    FileType,
    CharSz,
};

use crate::common::{
    Count,
    FileSz,
    Bytes,
    ResultS4,
};

use crate::data::sysline::{
    Sysline,
    SyslineP,
};

use crate::readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
};

use crate::data::datetime::{
    Year,
    FixedOffset,
    DateTimeL,
    DateTimeLOpt,
    DateTimeParseInstr,
    DateTimeRegex,
    DateTimeParseInstrsIndex,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
    DATETIME_PARSE_DATAS_REGEX_VEC,
    bytes_to_regex_to_datetime,
    dt_pass_filters,
    dt_after_or_before,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    slice_contains_X_2,
};

use crate::data::line::{
    LineIndex,
    Line,
    LineP,
    LinePartPtrs,
};

use crate::readers::linereader::{
    LineReader,
    ResultS4LineFind,
};

#[allow(unused_imports)]
use crate::printer_debug::printers::{
    dpo,
    dpn,
    dpx,
    dpnx,
    dpof,
    dpnf,
    dpxf,
    dpnxf,
    dp_err,
    dp_wrn,
    dp,
    p_err,
    p_wrn,
    p,
};

use std::collections::{
    BTreeMap,
    BTreeSet,
    HashSet,
};
use std::fmt;
use std::io::{
    Error,
    Result,
    ErrorKind,
};
use std::sync::Arc;

extern crate itertools;
use itertools::Itertools;  // brings in `sorted_by`

extern crate lru;
use lru::LruCache;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    debug_assert_lt,
};

extern crate rangemap;
use rangemap::RangeMap;

extern crate static_assertions;
use static_assertions::{
    const_assert,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime typing, strings, and formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// `Count` of datetime format strings used.
///
/// Key is index into global [`DATETIME_PARSE_DATAS`]
/// (and [`static@DATETIME_PARSE_DATAS_REGEX_VEC]).
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// [`Sysline`] Searching result
///
/// [`Sysline`]: crate::data::sysline::Sysline
pub type ResultS4SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;

/// storage for `Sysline`.
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
type SyslinesLRUCache = LruCache<FileOffset, ResultS4SyslineFind>;

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
    /// internal stats `Count` for `self.find_sysline()` use of `self.syslines`.
    pub(super) syslines_hit: Count,
    /// internal stats `Count` for `self.find_sysline()` use of `self.syslines`.
    pub(super) syslines_miss: Count,
    /// Syslines fileoffset by sysline fileoffset range,
    /// i.e. `\[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`.
    /// The stored value can be used as a key for `self.syslines`.
    syslines_by_range: SyslinesRangeMap,
    /// `Count` of `self.syslines_by_range` lookup hit
    pub(super) syslines_by_range_hit: Count,
    /// `Count` of `self.syslines_by_range` lookup miss
    pub(super) syslines_by_range_miss: Count,
    /// `Count` of `self.syslines_by_range.insert`
    pub(super) syslines_by_range_put: Count,
    /// First (soonest) processed [`DateTimeL`] (not necessarly printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    // TODO: [2022/07/27] cost-savings: save the ref
    pub(super) dt_first: DateTimeLOpt,
    pub(super) dt_first_prev: DateTimeLOpt,
    /// Last (latest) processed [`DateTimeL`] (not necessarly printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    // TODO: [2022/07/27] cost-savings: save the ref
    pub(super) dt_last: DateTimeLOpt,
    pub(super) dt_last_prev: DateTimeLOpt,
    /// `Count`s found patterns stored in `dt_patterns`.
    /// "mirrors" the global [`DATETIME_PARSE_DATAS`].
    /// Keys are indexes into `DATETIME_PARSE_DATAS`,
    /// values are counts of successful pattern match at that index.
    ///
    /// Initialized once in `fn SyslineReader::new`.
    ///
    /// Not used after `self.analyzed` becomes `true`.
    ///
    /// [`DATETIME_PARSE_DATAS`]: crate::data::datetime::DATETIME_PARSE_DATAS
    pub(super) dt_patterns_counts: DateTimePatternCounts,
    /// Keys of `self.dt_patterns_counts` sorted by value.
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
    /// [`FixedOffset`]: https://docs.rs/chrono/0.4.21/chrono/offset/struct.FixedOffset.html
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
    /// A failure does not mean an error.
    ///
    /// [`Arc::try_unwrap(syslinep)`]: std::sync::Arc#method.try_unwrap
    pub(super) drop_sysline_errors: Count,
}

impl fmt::Debug for SyslineReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    if !cfg!(debug_assertions) {
        return;
    }
    dp!("[");
    for (key, val) in cache.iter() {
        dp!(" Key: {:?}, Value: {:?};", key, val);
    }
    dp!("]");
}

/// Implement the `SyslineReader`
impl SyslineReader {
    /// Maximum number of datetime patterns to match when first reading a
    /// file (before settling on one).
    const DT_PATTERN_MAX_PRE_ANALYSIS: usize = 4;

    /// Maximum number of datetime patterns for matching the remainder of a syslog file
    const DT_PATTERN_MAX: usize = 1;

    /// When this number of syslines has been processed then reduce use of all
    /// patterns from all patterns in `DATETIME_PARSE_DATAS_REGEX_VEC` to one
    /// pattern.
    // TODO: this should not vary among different builds, fixing requires user-controlled
    //       call to `dt_patterns_analysis`
    //#[cfg(any(debug_assertions,test))]
    //const DT_PATTERN_ANALYSIS_THRESHOLD: Count = 1;
    //#[cfg(not(any(debug_assertions,test)))]
    const DT_PATTERN_ANALYSIS_THRESHOLD: Count = 5;

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

    pub fn new(path: FPath, filetype: FileType, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslineReader> {
        dpnx!("SyslineReader::new({:?}, {:?}, {:?}, {:?})", path, filetype, blocksz, tz_offset);
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
        Ok(
            SyslineReader {
                linereader: lr,
                syslines: Syslines::new(),
                syslines_count: 0,
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
                find_sysline_lru_cache: SyslinesLRUCache::new(SyslineReader::FIND_SYSLINE_LRU_CACHE_SZ),
                find_sysline_lru_cache_hit: 0,
                find_sysline_lru_cache_miss: 0,
                find_sysline_lru_cache_put: 0,
                parse_datetime_in_line_lru_cache_enabled: SyslineReader::CACHE_ENABLE_DEFAULT,
                parse_datetime_in_line_lru_cache: LineParsedCache::new(SyslineReader::PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ),
                parse_datetime_in_line_lru_cache_hit: 0,
                parse_datetime_in_line_lru_cache_miss: 0,
                parse_datetime_in_line_lru_cache_put: 0,
                get_boxptrs_singleptr: 0,
                get_boxptrs_doubleptr: 0,
                get_boxptrs_multiptr: 0,
                analyzed: false,
                drop_sysline_ok: 0,
                drop_sysline_errors: 0,
            }
        )
    }

    /// Return a copy of the `FileType`.
    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.linereader.filetype()
    }

    /// Return a copy of the `BlockSz`.
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    /// Return a copy of the File Size `FileSz`.
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.linereader.filesz()
    }

    /// Return a copy of the `MimeGuess`.
    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.linereader.mimeguess()
    }

    /// Return a reference to the `FPath`.
    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        self.linereader.path()
    }

    /// Return nearest preceding `BlockOffset` for given `FileOffset`
    /// (file byte offset).
    pub const fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.linereader.block_offset_at_file_offset(fileoffset)
    }

    /// Return `FileOffset` (file byte offset) at given `BlockOffset`.
    pub const fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.linereader.file_offset_at_block_offset(blockoffset)
    }

    /// Return `FileOffset` (file byte offset) at blockoffset+blockindex.
    pub const fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// Get the last byte index of the file (inclusive).
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.linereader.fileoffset_last()
    }

    /// Return block index at given `FileOffset`.
    pub const fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.linereader.block_index_at_file_offset(fileoffset)
    }

    /// Return `Count` of `Block`s in a file.
    ///
    /// Equivalent to the _last `BlockOffset` + 1_.
    ///
    /// Not a count of `Block`s that have been read; the calculated
    /// count of `Block`s based on the `FileSz`.
    pub const fn count_blocks(&self) -> Count {
        self.linereader.count_blocks()
    }

    /// The last valid `BlockOffset` of the file.
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.linereader.blockoffset_last()
    }

    /// Smallest size character in bytes,
    pub const fn charsz(&self) -> usize {
        self.linereader.charsz()
    }

    /// `Count` of `Sysline`s processed so far, i.e. `self.syslines_count`.
    pub fn count_syslines_processed(&self) -> Count {
        self.syslines_count
    }

    /// `Count` of `Sysline`s currently stored, i.e. `self.syslines.len()`
    pub fn count_syslines_stored(&self) -> Count {
        self.syslines.len() as Count
    }

    /// `Count` underlying `Line`s processed so far
    #[inline(always)]
    pub fn count_lines_processed(&self) -> Count {
        self.linereader.count_lines_processed()
    }

    /// Does the `dt_pattern` have a year? e.g. specificer `%Y` or `%y`
    pub fn dt_pattern_has_year(&self) -> bool {
        debug_assert!(!self.syslines.is_empty(), "called dt_pattern_has_year() without having processed some syslines");
        let dtpd: &DateTimeParseInstr = self.datetime_parse_data();
        dpnxf!("dtpd line {:?}", dtpd._line_num);

        dtpd.dtfs.has_year()
    }

    /// eEable internal LRU cache used by `find_sysline` and
    /// `parse_datetime_in_line`.
    ///
    /// Returns prior value of `find_sysline_lru_cache_enabled`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_enable(&mut self) -> bool {
        let ret = self.find_sysline_lru_cache_enabled;
        debug_assert_eq!(self.find_sysline_lru_cache_enabled, self.parse_datetime_in_line_lru_cache_enabled, "cache enables disagree");
        if !self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_enabled = true;
            self.find_sysline_lru_cache.clear();
            self.find_sysline_lru_cache.resize(SyslineReader::FIND_SYSLINE_LRU_CACHE_SZ);
        }
        if !self.parse_datetime_in_line_lru_cache_enabled {
            self.parse_datetime_in_line_lru_cache_enabled = true;
            self.parse_datetime_in_line_lru_cache.clear();
            self.parse_datetime_in_line_lru_cache.resize(SyslineReader::PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ);
        }

        dpnxf!("return {}", ret);

        ret
    }

    /// Disable internal LRU cache used by `find_sysline` and
    /// `parse_datetime_in_line`
    ///
    /// Returns prior value of `find_sysline_lru_cache_enabled`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_disable(&mut self) -> bool {
        let ret = self.find_sysline_lru_cache_enabled;
        debug_assert_eq!(self.find_sysline_lru_cache_enabled, self.parse_datetime_in_line_lru_cache_enabled, "cache enables disagree");
        self.find_sysline_lru_cache_enabled = false;
        self.find_sysline_lru_cache.resize(0);
        self.parse_datetime_in_line_lru_cache_enabled = false;
        self.parse_datetime_in_line_lru_cache.resize(0);

        dpnxf!("return {}", ret);

        ret
    }

    /// Print `Sysline` at `FileOffset`.
    ///
    /// Testing helper only
    #[doc(hidden)]
    #[cfg(test)]
    pub fn print(&self, fileoffset: FileOffset, raw: bool) {
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

    /// Is this `Sysline` the last `Sysline` of the entire file?
    /// (not the same as last Sysline within the optional datetime filters).
    pub fn is_sysline_last(&self, sysline: &Sysline) -> bool {
        let fo_end: FileOffset = sysline.fileoffset_end();
        if fo_end == self.fileoffset_last() {
            dpnxf!("return true");
            return true;
        }
        assert_lt!(fo_end, self.filesz(), "fileoffset_end {} is at or after filesz() {}", fo_end, self.filesz());
        dpnxf!("return false");

        false
    }

    /// Is this `SyslineP` the last `Sysline` of the entire file?
    /// (not the same as last Sysline within the optional datetime filters).
    pub fn is_syslinep_last(&self, syslinep: &SyslineP) -> bool {
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
        dpnxf!();
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
    /// Only intended to aid post-procesing year updates.
    ///
    /// Users must know what they are doing with this.
    pub(crate) fn remove_sysline(&mut self, fileoffset: FileOffset) -> bool {
        dpnf!("({:?})", fileoffset);
        let cache_enable = self.LRU_cache_disable();
        let syslinep_opt: Option<SyslineP> = self.syslines.remove(&fileoffset);
        let mut ret = true;
        match syslinep_opt {
            Some(syslinep) => {
                let fo_beg: FileOffset = (*syslinep).fileoffset_begin();
                let fo_end: FileOffset = (*syslinep).fileoffset_end();
                dpof!("sysline at {} removed; {:?} {:?}", fileoffset, (*syslinep).dt(), (*syslinep).to_String_noraw());
                debug_assert_eq!(fileoffset, fo_beg, "mismatching fileoffset {}, fileoffset_begin {}", fileoffset, fo_beg);
                let fo_end1: FileOffset = fo_end + (self.charsz() as FileOffset);
                let range: SyslineRange = SyslineRange { start: fo_beg, end: fo_end1 };
                dpof!("syslines_by_range remove {:?}", range);
                self.syslines_by_range.remove(range);
                self.dt_first = self.dt_first_prev;
                self.dt_last = self.dt_last_prev;
            }
            None => {
                dpof!("syslines failed to remove {}", fileoffset);
                ret = false;
            }
        }
        if cache_enable {
            self.LRU_cache_enable();
        }
        dpxf!("({:?}) return {:?}", fileoffset, ret);

        ret
    }

    /// store passed `Sysline` in `self.syslines`,
    /// update other fields.
    fn insert_sysline(&mut self, sysline: Sysline) -> SyslineP {
        let fo_beg: FileOffset = sysline.fileoffset_begin();
        let fo_end: FileOffset = sysline.fileoffset_end();
        let syslinep: SyslineP = SyslineP::new(sysline);
        dpnf!("syslines.insert({}, Sysline @[{}, {}] datetime: {:?})", fo_beg, (*syslinep).fileoffset_begin(), (*syslinep).fileoffset_end(), (*syslinep).dt());
        self.syslines.insert(fo_beg, syslinep.clone());
        self.syslines_count += 1;
        // XXX: Issue #16 only handles UTF-8/ASCII encoding
        let fo_end1: FileOffset = fo_end + (self.charsz() as FileOffset);
        dpxf!("syslines_by_range.insert(({}‥{}], {})", fo_beg, fo_end1, fo_beg);
        self.syslines_by_range.insert(fo_beg..fo_end1, fo_beg);
        self.syslines_by_range_put += 1;

        syslinep
    }

    /// Drop as much data as possible that uses the referred `Block`.
    pub fn drop_block(&mut self, blockoffset: BlockOffset, bo_dropped: &mut HashSet<BlockOffset>) {
        dpnf!("({})", blockoffset);

        // TODO: [2022/06/18] cost-savings: make this a "one time" creation that is reused
        //       this is challenging, as it runs into borrow errors during `.iter()`
        let mut drop_block_fo_keys: Vec<FileOffset> = Vec::<FileOffset>::with_capacity(self.syslines.len());

        for fo_key in self.syslines.keys() {
            drop_block_fo_keys.push(*fo_key);
        }
        // vec of `fileoffset` must be ordered which is guaranteed by `syslines: BTreeMap`

        dpof!("collected keys {:?}", drop_block_fo_keys);

        // XXX: using `sylines.value_mut()` would be cleaner.
        //      But `sylines.value_mut()` causes a clone of the `SyslineP`, which then
        //      increments the `Arc` "strong_count". That in turn prevents `Arc::get_mut(&SyslineP)`
        //      from returning the original `Sysline`.
        //      Instead of `syslines.values_mut()`, use `syslines.keys()` and then `syslines.get_mut`
        //      to get a `&SyslineP`. This does not increase the "strong_count".

        for fo_key in drop_block_fo_keys.iter() {
            let bo_last: BlockOffset = self.syslines[fo_key].blockoffset_last();
            if bo_last > blockoffset {
                dpof!("blockoffset_last {} > {} blockoffset, continue;", bo_last, blockoffset);
                // presume all proceeding `Sysline.blockoffset_last()` will be after `blockoffset`
                break;
            }
            // XXX: copy `fo_key` to avoid borrowing error
            self.drop_sysline(fo_key, bo_dropped);
            dpof!("bo_dropped {:?}", bo_dropped);
        }

        dpxf!("({})", blockoffset);
    }

    /// Drop all data associated with `Sysline` at `fileoffset`
    /// (or at least, drop as much as possible).
    ///
    /// Caller must know what they are doing!
    pub fn drop_sysline(&mut self, fileoffset: &FileOffset, bo_dropped: &mut HashSet<BlockOffset>) {
        dpnf!("({})", fileoffset);
        let syslinep: SyslineP = match self.syslines.remove(fileoffset) {
            Some(syslinep_) => syslinep_,
            None => {
                dp_wrn!("syslines.remove({}) returned None which is unexpected", fileoffset);
                return;
            }
        };
        dpof!("Processing SyslineP @[{}‥{}], Block @[{}‥{}] strong_count {}", (*syslinep).fileoffset_begin(), (*syslinep).fileoffset_end(), (*syslinep).blockoffset_first(), (*syslinep).blockoffset_last(), Arc::strong_count(&syslinep));
        self.find_sysline_lru_cache.pop(&(*syslinep).fileoffset_begin());
        match Arc::try_unwrap(syslinep) {
            Ok(sysline) => {
                dpof!("Arc::try_unwrap(syslinep) Ok Sysline @[{}‥{}] Block @[{}‥{}]", sysline.fileoffset_begin(), sysline.fileoffset_end(), sysline.blockoffset_first(), sysline.blockoffset_last());
                self.drop_sysline_ok += 1;
                self.linereader.drop_lines(sysline.lines, bo_dropped);
            }
            Err(_syslinep) => {
                dpof!("Arc::try_unwrap(syslinep) Err strong_count {}", Arc::strong_count(&_syslinep));
                self.drop_sysline_errors += 1;
            }
        }
    }

    /// If datetime found in `Line` returns [`Ok`] around
    /// indexes into `Line` of found datetime string 
    /// (start of string, end of string)`
    ///
    /// else returns [`Err`].
    ///
    /// [`Ok`]: self::ResultFindDateTime
    /// [`Err`]: self::ResultFindDateTime
    pub fn find_datetime_in_line(
        line: &Line,
        parse_data_indexes: &DateTimeParseDatasIndexes,
        charsz: &CharSz,
        year_opt: &Option<Year>,
        tz_offset: &FixedOffset,
        get_boxptrs_singleptr: &mut Count,
        get_boxptrs_doubleptr: &mut Count,
        get_boxptrs_multiptr: &mut Count,
    ) -> ResultFindDateTime {
        dpnf!("(…, …, {:?}, year_opt {:?}, {:?}) line {:?}", charsz, year_opt, tz_offset, line.to_String_noraw());
        dpof!("parse_data_indexes.len() {} {:?}", parse_data_indexes.len(), parse_data_indexes);

        // skip an easy case; no possible datetime
        if line.len() < SyslineReader::DATETIME_STR_MIN {
            dpxf!("return Err(ErrorKind::InvalidInput);");
            return ResultFindDateTime::Err(Error::new(ErrorKind::InvalidInput, "Line is too short to hold a datetime"));
        }

        const HACK12: &[u8; 2] = b"12";
        // `sie` and `siea` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        for (at, index) in parse_data_indexes.iter().enumerate() {
            let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*index];
            dpof!("pattern data try {} index {} dtpd.line_num {}", at, index, dtpd._line_num);

            if line.len() <= dtpd.range_regex.start {
                dpof!("line too short {} for  requested start {}; continue", line.len(), dtpd.range_regex.start);
                continue;
            }
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            let slice_end: usize;
            if line.len() > dtpd.range_regex.end {
                slice_end = dtpd.range_regex.end;
            } else {
                slice_end = line.len() - 1;
            }
            if dtpd.range_regex.start >= slice_end {
                dpof!("bad line slice indexes [{}, {}); continue", dtpd.range_regex.start, slice_end);
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // this is to force the parsing function `Local.datetime_from_str` to constrain where it
            // searches within the `Line`
            let mut hack_slice: Bytes;
            let slice_: &[u8];
            match line.get_boxptrs(dtpd.range_regex.start as LineIndex, slice_end as LineIndex) {
                LinePartPtrs::NoPtr => {
                    panic!("line.get_boxptrs({}, {}) returned NoPtr which means it was passed non-sense values", dtpd.range_regex.start, slice_end);
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
            dpof!("slice len {} [{}, {}) (requested [{}, {})) using DTPD from {}, data {:?}", slice_.len(), dtpd.range_regex.start, slice_end, dtpd.range_regex.start, dtpd.range_regex.end, dtpd._line_num, String::from_utf8_lossy(slice_));
            // hack efficiency improvement, presumes all found years will have a '1' or a '2' in them
            if charsz == &1 && dtpd.dtfs.has_year() && !slice_contains_X_2(slice_, HACK12) {
                dpof!("skip slice, does not have '1' or '2'");
                // TODO: add stat for tracking this branch of hack pattern matching
                continue;
            }
            // find the datetime string using `Regex`, convert to a `DateTimeL`
            let dt: DateTimeL;
            let dt_beg: LineIndex;
            let dt_end: LineIndex;
            (dt_beg, dt_end, dt) =
                //match str_to_regex_to_datetime(dts, index, tz_offset) {
                match bytes_to_regex_to_datetime(slice_, index, year_opt, tz_offset) {
                    None => continue,
                    Some(val) => val,
            };
            dpxf!("return Ok({}, {}, {}, {});", dt_beg, dt_end, &dt, index);
            return ResultFindDateTime::Ok((dt_beg, dt_end, dt, *index));
        }  // end for(pattern, ...)

        dpxf!("return Err(ErrorKind::NotFound);");
        ResultFindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found in Line!"))
    }

    /// Update the two statistic `DateTimeL` of
    /// `self.dt_first` and `self.dt_last`.
    fn dt_first_last_update(&mut self, datetime: &DateTimeL) {
        dpnxf!("({:?})", datetime);
        // TODO: the `dt_first` and `dt_last` are only for `--summary`, no need to always copy
        //       datetimes.
        //       Would be good to only run this when `if self.do_summary {...}`
        match self.dt_first {
            Some(dt_first_) => {
                if &dt_first_ > datetime {
                    self.dt_first_prev = self.dt_first;
                    self.dt_first = Some(*datetime);
                }
            },
            None => { self.dt_first = Some(*datetime); }
        }
        match self.dt_last {
            Some(dt_last_) => {
                if &dt_last_ < datetime {
                    self.dt_last_prev = self.dt_last;
                    self.dt_last = Some(*datetime);
                }
            },
            None => { self.dt_last = Some(*datetime); }
        }
    }

    /// Helper function to update `parse_datetime_in_line`.
    fn dt_patterns_update(&mut self, index: DateTimeParseInstrsIndex) {
        dpnxf!("({:?})", index);
        if let std::collections::btree_map::Entry::Vacant(_entry) = self.dt_patterns_counts.entry(index) {
            panic!("index {} not present in self.dt_patterns_counts", index);
        } else {
            // index has been counted, increment it's count
            let counter: &mut Count = self.dt_patterns_counts.get_mut(&index).unwrap();
            *counter += 1;
        }
        // refresh the indexes every time until `dt_patterns_analysis` is called
        if self.analyzed {
            return;
        }
        self.dt_patterns_indexes_refresh();
    }

    fn dt_patterns_indexes_refresh(&mut self) {
        self.dt_patterns_indexes.clear();
        // get copy of pattern indexes sorted by value
        // this makes the most-used parse_data more likely to be used again
        self.dt_patterns_indexes.extend(
            self.dt_patterns_counts.iter().sorted_by(
                |a, b| Ord::cmp(&b.1, &a.1) // sort by value (second tuple item)
            ).map(|(k, _v)| k) // copy only the key (first tuple item) which is an index
        );
        dpnxf!("dt_patterns_indexes {:?}", self.dt_patterns_indexes);
    }

    /// Analyze `Sysline`s gathered.
    ///
    /// When a threshold of `Sysline`s or bytes has been processed, then
    /// this function narrows down datetime formats to try for future
    /// datetime-parsing attempts.
    /// Further calls to function `SyslineReader::find_datetime_in_line`\
    /// use far less resources.
    pub(crate) fn dt_patterns_analysis(&mut self) {
        if self.count_syslines_processed() < SyslineReader::DT_PATTERN_ANALYSIS_THRESHOLD {
            return;
        }
        dpnf!();
        // XXX: DT_PATERN_MAX > 1 is unimplemented
        const_assert!(SyslineReader::DT_PATTERN_MAX == 1);
        if cfg!(debug_assertions) {
            for (k, v) in self.dt_patterns_counts.iter() {
                let data_: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC.get(*k).unwrap();
                dpof!("self.dt_patterns_counts[{:?}]={:?} is {:?}, {:?}", k, v, data_, data_rex_);
            }
        }
        // get maximum value in `dt_patterns_counts`
        // ripped from https://stackoverflow.com/a/60134450/471376
        // test https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b8eb53f40fd89461c9dad9c976746cc3
        let max_ = (&self.dt_patterns_counts).iter().fold(
            std::u64::MIN, |a,b| a.max(*(b.1))
        );
        // remove all items < maximum value in `dt_patterns_counts`
        dpof!("dt_patterns_counts.retain(v >= {:?})", max_);
        self.dt_patterns_counts.retain(|_, v| *v >= max_);
        if self.dt_patterns_counts.len() != SyslineReader::DT_PATTERN_MAX {
            dp_err!("dt_patterns_analysis: self.dt_patterns_counts.len() {}, expected 1", self.dt_patterns_counts.len());
        }
        self.dt_patterns_indexes_refresh();
        if cfg!(debug_assertions) {
            for (k, v) in self.dt_patterns_counts.iter() {
                let data_: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC.get(*k).unwrap();
                dpof!("self.dt_patterns_counts[index {:?}]={:?} is {:?}, {:?}", k, v, data_, data_rex_);
            }
        }
        self.analyzed = true;
        dpxf!();
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
        if cfg!(debug_assertions) {
            for (_k, _v) in self.dt_patterns_counts.iter() {
                let data_: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*_k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC.get(*_k).unwrap();
                dpof!("self.dt_patterns_counts[index {:?}]={:?} is {:?}, {:?}", _k, _v, data_, data_rex_);
            }
            for _val in self.dt_patterns_indexes.iter() {
                dpof!("self.dt_patterns_indexes {:?}", _val);
            }
        }
        if !self.analyzed {
            dpof!("before analysis");
            // before analysis, the uses of all `DateTimeParseInstr` are tracked
            // return index to maximum value
            // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=85ac85f48e6ddff04dc938b742872dc1
            let max_key_value: Option<(&DateTimeParseInstrsIndex, &Count)> = (&self.dt_patterns_counts).iter().reduce(
                |accum, item| {
                    if accum.1 >= item.1 { accum } else { item }
                }
            );
            *max_key_value.unwrap().0
        } else {
            dpof!("after analysis");
            // after analysis, only one `DateTimeParseInstr` is used
            debug_assert_eq!(self.dt_patterns_indexes.len(), SyslineReader::DT_PATTERN_MAX, "self.dt_patterns_indexes length {}, expected {}", self.dt_patterns_indexes.len(), SyslineReader::DT_PATTERN_MAX);
            // the first and only element is the chosen dt_pattern (and had max count)
            *self.dt_patterns_indexes.iter().next().unwrap()
        }
    }

    /// Attempt to parse a DateTime substring in the passed `Line`.
    ///
    /// Wraps call to `self.find_datetime_in_line` according to status of
    /// `self.dt_patterns`.<br/>
    /// If `self.dt_patterns` is `None`, will set `self.dt_patterns`.
    // TODO: [2022/08] having `dt_patterns_update` embedded into this is an unexpected side-affect
    //       the user should have more control over when `dt_patterns_update` is called.
    fn parse_datetime_in_line(&mut self, line: &Line, charsz: &CharSz, year_opt: &Option<Year>) -> ResultParseDateTime {
        // XXX: would prefer this at the end of this function but borrow error occurs
        if !self.analyzed {
            // TODO: [2022/07] `dt_patterns_analysis` should be called by the user, not embedded in the
            //       inner-workings
            self.dt_patterns_analysis();
        }
        dpnf!("(…, {}, year_opt {:?}) line: {:?}", charsz, year_opt, line.to_String_noraw());

        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats
        // `self.dt_patterns`
        //
        // TODO: [2022/06/26] cost-savings: create the `indexes` once in an analysis update function
        //       or somewhere else
        let mut indexes: DateTimeParseDatasIndexes = DateTimeParseDatasIndexes::new();
        // get copy of indexes sorted by value
        indexes.extend(
            self.dt_patterns_counts.iter().sorted_by(
                |a, b| Ord::cmp(&b.1, &a.1) // sort by value (second tuple item)
            ).map(|(k, _v)| k) // copy only the key (first tuple item) which is an index
        );
        dpof!("indexes {:?}", indexes);
        let result: ResultFindDateTime = SyslineReader::find_datetime_in_line(
            line,
            &indexes,
            charsz,
            year_opt,
            &self.tz_offset,
            &mut self.get_boxptrs_singleptr,
            &mut self.get_boxptrs_doubleptr,
            &mut self.get_boxptrs_multiptr,
        );
        let data: FindDateTimeData = match result {
            Ok(val) => val,
            Err(err) => {
                dpxf!("return Err {};", err);
                return ResultParseDateTime::Err(err);
            }
        };
        self.dt_patterns_update(data.3);
        self.dt_first_last_update(&data.2);
        dpxf!("return {:?}", data);

        ResultParseDateTime::Ok(data)
    }

    /// Helper function to `find_sysline`.
    ///
    /// Call `self.parse_datetime_in_line` with help of LRU cache
    /// `self.parse_datetime_in_line_lru_cache`.
    fn parse_datetime_in_line_cached(&mut self, linep: &LineP, charsz: &CharSz, year_opt: &Option<Year>) -> ResultParseDateTime {
        if self.parse_datetime_in_line_lru_cache_enabled {
            match self.parse_datetime_in_line_lru_cache.get(&linep.fileoffset_begin()) {
                Some(val) => {
                    self.parse_datetime_in_line_lru_cache_hit +=1;
                    return ResultParseDateTime::Ok(*val);
                }
                _ => {
                    self.parse_datetime_in_line_lru_cache_miss += 1;
                },
            }
        }
        dpnf!("(…, {:?}, {:?})", charsz, year_opt);
        let result: ResultParseDateTime = self.parse_datetime_in_line(&*linep, charsz, year_opt);
        if self.parse_datetime_in_line_lru_cache_enabled {
            match result {
                Ok(val) => {
                    match self.parse_datetime_in_line_lru_cache.put(linep.fileoffset_begin(), val) {
                        Some(val_prev) => {
                            panic!("self.parse_datetime_in_line_lru_cache already had key {:?}, value {:?}", linep.fileoffset_begin(), val_prev);
                        },
                        _ => {},
                    }
                },
                _ => {},
            }
        }
        dpxf!("return {:?}", result);

        result
    }

    /// Check various internal storage for already processed
    /// [`Sysline`] at [`FileOffset`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`FileOffset`]: crate::common::FileOffset
    fn check_store(&mut self, fileoffset: FileOffset) -> Option<ResultS4SyslineFind> {
        dpnf!("({})", fileoffset);

        if self.find_sysline_lru_cache_enabled {
            // check if `fileoffset` is already known about in LRU cache
            match self.find_sysline_lru_cache.get(&fileoffset) {
                Some(results4) => {
                    self.find_sysline_lru_cache_hit += 1;
                    dpof!("found LRU cached for fileoffset {}", fileoffset);
                    // the `.get` returns a reference `&ResultS4SyslineFind` so must return a new `ResultS4SyslineFind`
                    match results4 {
                        ResultS4SyslineFind::Found(val) => {
                            dpxf!("return ResultS4SyslineFind::Found(({}, …)) @[{}, {}] from LRU cache", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return Some(ResultS4SyslineFind::Found((val.0, val.1.clone())));
                        }
                        ResultS4SyslineFind::Done => {
                            dpxf!("return ResultS4SyslineFind::Done from LRU cache");
                            return Some(ResultS4SyslineFind::Done);
                        }
                        ResultS4SyslineFind::Err(err) => {
                            dpof!("Error {}", err);
                            eprintln!("ERROR: unexpected value store in self._find_line_lru_cache.get({}) error {}", fileoffset, err);
                        }
                    }
                }
                None => {
                    self.find_sysline_lru_cache_miss += 1;
                    dpof!("fileoffset {} not found in LRU cache", fileoffset);
                }
            }
        }

        // check if the offset is already in a known range
        match self.syslines_by_range.get_key_value(&fileoffset) {
            Some(range_fo) => {
                let range: &SyslineRange = range_fo.0;
                dpof!("hit syslines_by_range cache for FileOffset {} (found in range {:?})", fileoffset, range);
                self.syslines_by_range_hit += 1;
                let fo: &FileOffset = range_fo.1;
                let syslinep: SyslineP = self.syslines[fo].clone();
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                let fo_next: FileOffset = (*syslinep).fileoffset_next() + (self.charsz() as FileOffset);
                if self.is_sysline_last(&syslinep) {
                    dpxf!(
                        "is_sysline_last() true; return ResultS4SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        fo_next,
                        &*syslinep,
                        (*syslinep).fileoffset_begin(),
                        (*syslinep).fileoffset_end(),
                        (*syslinep).to_String_noraw()
                    );
                    self.find_sysline_lru_cache_put += 1;
                    self.find_sysline_lru_cache
                        .put(fileoffset, ResultS4SyslineFind::Found((fo_next, syslinep.clone())));
                    return Some(ResultS4SyslineFind::Found((fo_next, syslinep)));
                }
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4SyslineFind::Found((fo_next, syslinep.clone())));
                dpxf!(
                    "is_sysline_last() false; return ResultS4SyslineFind::Found(({}, @{:p})) @[{}, {}] from self.syslines_by_range {:?}",
                    fo_next,
                    &*syslinep,
                    (*syslinep).fileoffset_begin(),
                    (*syslinep).fileoffset_end(),
                    (*syslinep).to_String_noraw()
                );
                return Some(ResultS4SyslineFind::Found((fo_next, syslinep)));
            }
            None => {
                self.syslines_by_range_miss += 1;
                dpof!("fileoffset {} not found in self.syslines_by_range", fileoffset);
            }
        }

        // check if there is a Sysline already known at this fileoffset
        // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
        if self.syslines.contains_key(&fileoffset) {
            debug_assert!(self.syslines_by_range.contains_key(&fileoffset), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}) returned None (syslines_by_range out of synch)", fileoffset, fileoffset);
            self.syslines_hit += 1;
            dpof!("hit self.syslines for FileOffset {}", fileoffset);
            let syslinep: SyslineP = self.syslines[&fileoffset].clone();
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            let fo_next: FileOffset = (*syslinep).fileoffset_end() + (self.charsz() as FileOffset);
            if self.is_sysline_last(&syslinep) {
                dpof!(
                    "return ResultS4SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                    fo_next,
                    &*syslinep,
                    (*syslinep).fileoffset_begin(),
                    (*syslinep).fileoffset_end(),
                    (*syslinep).to_String_noraw()
                );
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4SyslineFind::Found((fo_next, syslinep.clone())));
                return Some(ResultS4SyslineFind::Found((fo_next, syslinep)));
            }
            if self.find_sysline_lru_cache_enabled {
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4SyslineFind::Found((fo_next, syslinep.clone())));
            }
            dpxf!(
                "return ResultS4SyslineFind::Found(({}, @{:p})) @[{}, {}] from self.syslines {:?}",
                fo_next,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
            return Some(ResultS4SyslineFind::Found((fo_next, syslinep)));
        } else {
            self.syslines_miss += 1;
            dpxf!("fileoffset {} not found in self.syslines", fileoffset);
        }
        dpxf!("return None");

        None
    }

    /// Find [`Sysline`] at `FileOffset` within the same `Block`
    /// (does not cross `Block` boundaries).
    ///
    /// This does a linear search over the `Block`, _O(n)_.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub fn find_sysline_in_block(&mut self, fileoffset: FileOffset) -> ResultS4SyslineFind {
        self.find_sysline_in_block_year(fileoffset, &None)
    }

    /// Find [`Sysline`] at fileoffset within the same `Block`
    /// (does not cross block boundaries).
    ///
    /// This does a linear search over the `Block`, _O(n)_.
    ///
    /// Optional `Year` is the filler year for datetime patterns that do not include a year.
    /// e.g. `"Jan 1 12:00:00 this is a sylog message"`
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    // XXX: similar to `find_sysline`:
    //      This function `find_sysline_in_block_year` is large and cumbersome.
    //      Changes require extensive retesting.
    //      Extensive debug prints are left in place to aid this.
    //      It could use some improvements but for now it gets the job done.
    //      You've been warned.
    //
    pub fn find_sysline_in_block_year(&mut self, fileoffset: FileOffset, year_opt: &Option<Year>) -> ResultS4SyslineFind {
        dpnf!("({}, {:?})", fileoffset, year_opt);

        if let Some(results4) = self.check_store(fileoffset) {
            dpxf!("({}): return {:?}", fileoffset, results4);
            return results4;
        }

        dpof!("({}): searching for first sysline datetime A …", fileoffset);

        let mut _fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sysline = Sysline::new();
        loop {
            dpof!("({}): self.linereader.find_line_in_block({})", fileoffset, fo1);
            let result: ResultS4LineFind = self.linereader.find_line_in_block(fo1);
            let (fo2, linep) = match result {
                ResultS4LineFind::Found((fo_, linep_))
                => {
                    dpof!(
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
                ResultS4LineFind::Done => {
                    dpxf!("({}): return ResultS4SyslineFind::Done; A from LineReader.find_line_in_block({})", fileoffset, fo1);
                    return ResultS4SyslineFind::Done;
                }
                ResultS4LineFind::Err(err) => {
                    dp_err!("LineReader.find_line_in_block({}) returned {}", fo1, err);
                    dpxf!("({}): return ResultS4SyslineFind::Err({}); A from LineReader.find_line_in_block({})", fileoffset, err, fo1);
                    return ResultS4SyslineFind::Err(err);
                }
            };
            let result: ResultParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            dpof!("({}): A parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {},
                // FindDateTimeData:
                // (LineIndex, LineIndex, DateTimeL, DateTimeParseInstrsIndex);
                Ok((dt_beg, dt_end, dt, _index)) => {
                    // a datetime was found! beginning of a sysline
                    _fo_a = fo1;
                    sysline.dt_beg = dt_beg;
                    sysline.dt_end = dt_end;
                    sysline.dt = Some(dt);
                    dpof!("({}): A sl.dt_beg {}, sl.dt_end {}, sl.push({:?})", fileoffset, dt_beg, dt_end, (*linep).to_String_noraw());
                    sysline.push(linep);
                    fo1 = sysline.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}", dt_end, fo1 as usize);
                    if self.is_sysline_last(&sysline) {
                        let syslinep: SyslineP = self.insert_sysline(sysline);
                        if self.find_sysline_lru_cache_enabled {
                            self.find_sysline_lru_cache_put += 1;
                            dpof!("({}): LRU cache put({}, Found({}, …))", fileoffset, fileoffset, fo1);
                            self.find_sysline_lru_cache
                                .put(fileoffset, ResultS4SyslineFind::Found((fo1, syslinep.clone())));
                        }
                        dpxf!(
                            "({}): return ResultS4SyslineFind::Found({}, {:p}) @[{}, {}]; A found here and LineReader.find_line({})",
                            fileoffset,
                            fo1,
                            &(*syslinep),
                            (*syslinep).fileoffset_begin(),
                            (*syslinep).fileoffset_end(),
                            fo1,
                        );
                        return ResultS4SyslineFind::Found((fo1, syslinep));
                    }
                    break;
                }
            }
            dpof!("({}): A skip push Line {:?}", fileoffset, (*linep).to_String_noraw());
            fo1 = fo2;
        }

        dpof!(
            "({}): found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            fileoffset,
            _fo_a,
            fo1,
        );

        //
        // find line with datetime B
        //

        let mut fo_b: FileOffset = fo1;
        loop {
            dpof!("({}): self.linereader.find_line_in_block({})", fileoffset, fo1);
            let result: ResultS4LineFind = self.linereader.find_line_in_block(fo1);
            let (fo2, linep) = match result {
                ResultS4LineFind::Found((fo_, linep_)) => {
                    dpof!(
                        "({}): B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    (fo_, linep_)
                },
                ResultS4LineFind::Done => {
                    dpof!("({}): B got Done", fileoffset);
                    break;
                },
                ResultS4LineFind::Err(err) => {
                    dp_err!("LineReader.find_line_in_block({}) returned {}", fo1, err);
                    dpxf!("({}): return ResultS4SyslineFind::Err({}); B from LineReader.find_line_in_block({})", fileoffset, err, fo1);
                    return ResultS4SyslineFind::Err(err);
                },
            };

            let result: ResultParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            dpof!("({}): B parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {
                    dpof!("({}): B append found Line to this Sysline sl.push({:?})", fileoffset, (*linep).to_String_noraw());
                    sysline.push(linep);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    dpof!(
                        "({}): B found datetime; end of this Sysline. Do not append found Line {:?}",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        dpof!("({}): found line with datetime B at FileOffset {} {:?}", fileoffset, fo_b, sysline.to_String_noraw());

        let syslinep: SyslineP = self.insert_sysline(sysline);
        if self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_put += 1;
            dpof!("({}): LRU cache put({}, Found({}, …))", fileoffset, fileoffset, fo_b);
            self.find_sysline_lru_cache
                .put(fileoffset, ResultS4SyslineFind::Found((fo_b, syslinep.clone())));
        }
        dpxf!(
            "({}): return ResultS4SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            fileoffset,
            fo_b,
            &*syslinep,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).to_String_noraw()
        );

        ResultS4SyslineFind::Found((fo_b, syslinep))
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
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4SyslineFind {
        self.find_sysline_year(fileoffset, &None)
    }

    /// Find first [`Sysline`] starting at or after `FileOffset`.
    /// Returns
    /// (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`).
    ///
    /// Optional `Year` is the filler year for datetime patterns that do not include a year.
    /// e.g. `"Jan 1 12:00:00 this is a sylog message"`
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
    pub fn find_sysline_year(&mut self, fileoffset: FileOffset, year_opt: &Option<Year>) -> ResultS4SyslineFind {
        dpnf!("({}, {:?})", fileoffset, year_opt);

        if let Some(results4) = self.check_store(fileoffset) {
            dpxf!("({}): return {:?}", fileoffset, results4);
            return results4;
        }

        let charsz_fo: FileOffset = self.charsz() as FileOffset;

        dpof!("({}): searching for first sysline datetime A …", fileoffset);

        //
        // find line with datetime A
        //

        let mut fo_zero_tried: bool = false;
        let mut _fo_a: FileOffset = 0;
        let mut fo_a_max: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sysline = Sysline::new();
        loop {
            dpof!("({}): self.linereader.find_line({})", fileoffset, fo1);
            let result: ResultS4LineFind = self.linereader.find_line(fo1);
            let (fo2, linep) = match result {
                ResultS4LineFind::Found((fo_, linep_))
                => {
                    dpof!(
                        "A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    (fo_, linep_)
                }
                ResultS4LineFind::Done => {
                    if self.find_sysline_lru_cache_enabled {
                        self.find_sysline_lru_cache_put += 1;
                        dpof!("({}): LRU cache put({}, Done)", fileoffset, fileoffset);
                        self.find_sysline_lru_cache.put(fileoffset, ResultS4SyslineFind::Done);
                    }
                    dpxf!("({}): return ResultS4SyslineFind::Done; A from LineReader.find_line({})", fileoffset, fo1);
                    return ResultS4SyslineFind::Done;
                }
                ResultS4LineFind::Err(err) => {
                    dp_err!("LineReader.find_line({}) returned {}", fo1, err);
                    dpxf!("({}): return ResultS4SyslineFind::Err({}); A from LineReader.find_line({})", fileoffset, err, fo1);
                    return ResultS4SyslineFind::Err(err);
                }
            };
            fo_a_max = std::cmp::max(fo_a_max, fo2);
            let result: ResultParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            dpof!("({}): A parse_datetime_in_line_cached returned {:?}", fileoffset, result);
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
                    dpof!("({}): A sl.push({:?})", fileoffset, (*linep).to_String_noraw());
                    sysline.push(linep);
                    fo1 = sysline.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}, dt {:?}", dt_beg, dt_end, dt);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}, dt {:?}", dt_end, fo1 as usize, dt);
                    break;
                }
            }
            dpof!("({}): A skip push Line {:?}", fileoffset, (*linep).to_String_noraw());
            let line_beg: FileOffset = (*linep).fileoffset_begin();
            if fo_zero_tried {
                // search forwards...
                // have already searched for a datetime stamp all the way back to zero'th byte of file
                // so switch search direction to go forward. these first few lines without a datetime stamp
                // will be ignored.
                // TODO: [2022/07] somehow inform user that some lines were dropped.
                fo1 = fo_a_max;
            } else if line_beg > charsz_fo {
                // search backwards...
                fo1 = line_beg - charsz_fo;
                // TODO: cost-savings: searching `self.syslines_by_range` is surprisingly expensive.
                //       Consider adding a faster, simpler `HashMap` that only tracks
                //       `sysline.fileoffset_end` keys to `fileoffset_begin` values.
                if self.syslines_by_range.contains_key(&fo1) {
                    // ran into prior processed sysline, something is wrong; abandon these lines
                    // and chang search direction to go forwards
                    fo_zero_tried = true;
                    fo1 = fo_a_max;
                    dp_err!("ran into prior processed sysline at fileoffset {}; some lines will be dropped.", fo1);
                    panic!("ERROR: ran into prior processed sysline at fileoffset {}; some lines will be dropped. SHOULD THIS BE FIXED?", fo1);
                }
            } else {
                // search from byte zero
                fo1 = 0;
                fo_zero_tried = true;
            }
        }

        dpof!(
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
                dpo!("find_sysline: hit self.syslines for FileOffset {}", fo1);
                let syslinep = self.syslines[&fo1].clone();
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                let fo_next = (*syslinep).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                dpxf!(
                "return ResultS4SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                fo_next,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
                if self.find_sysline_lru_cache_enabled {
                    self.find_sysline_lru_cache_put += 1;
                    self.find_sysline_lru_cache
                        .put(fileoffset, ResultS4SyslineFind::Found((fo_next, syslinep.clone())));
                }
                return ResultS4SyslineFind::Found((fo_next, syslinep));
            } else {
                dpo!("find_sysline: fileoffset {} not found in self.syslines", fileoffset);
            }
            // check if the offset is already in a known range
            match self.syslines_by_range.get_key_value(&fo1) {
                Some(range_fo) => {
                    let range = range_fo.0;
                    dpof!(
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
                            .put(fileoffset, ResultS4SyslineFind::Found((fo_next, syslinep.clone())));
                    }
                    dpxf!(
                        "return ResultS4SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        fo_next,
                        &*syslinep,
                        (*syslinep).fileoffset_begin(),
                        (*syslinep).fileoffset_end(),
                        (*syslinep).to_String_noraw()
                    );
                    return ResultS4SyslineFind::Found((fo_next, syslinep));
                }
                None => {
                    self.syslines_by_range_miss += 1;
                    dpo!("find_sysline: fileoffset {} not found in self.syslines_by_range", fileoffset);
                }
            }
            dpo!("find_sysline: searching for first sysline datetime B …");
            */
        }

        let mut fo_b: FileOffset = fo1;
        dpo!("find_sysline_year({}): fo_b {:?}", fileoffset, fo_b);
        loop {
            dpof!("({}): self.linereader.find_line({})", fileoffset, fo1);
            let result: ResultS4LineFind = self.linereader.find_line(fo1);
            let (fo2, linep) = match result {
                ResultS4LineFind::Found((fo_, linep_)) => {
                    dpof!(
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
                ResultS4LineFind::Done => {
                    dpof!("({}): break; B", fileoffset);
                    break;
                }
                ResultS4LineFind::Err(err) => {
                    dp_err!("LineReader.find_line({}) returned {}", fo1, err);
                    dpxf!("({}): return ResultS4SyslineFind::Err({}); B from LineReader.find_line({})", fileoffset, err, fo1);
                    return ResultS4SyslineFind::Err(err);
                }
            };
            let result: ResultParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz(), year_opt);
            dpof!("({}): B parse_datetime_in_line_cached returned {:?}", fileoffset, result);
            match result {
                Err(_) => {
                    // a datetime was not found in the Line! This line is also part of this sysline
                    dpof!(
                        "({}): B append found Line to this Sysline sl.push({:?})",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    sysline.push(linep);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    dpof!(
                        "({}): B found datetime; end of this Sysline. Do not append found Line {:?}",
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        dpo!("find_sysline_year({}): found line with datetime B at FileOffset {} {:?}", fileoffset, fo_b, sysline.to_String_noraw());

        let syslinep: SyslineP = self.insert_sysline(sysline);
        if self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_put += 1;
            dpof!("({}): LRU cache put({}, Found({}, …))", fileoffset, fileoffset, fo_b);
            self.find_sysline_lru_cache
                .put(fileoffset, ResultS4SyslineFind::Found((fo_b, syslinep.clone())));
        }
        dpxf!(
            "({}): return ResultS4SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            fileoffset,
            fo_b,
            &*syslinep,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).to_String_noraw()
        );

        ResultS4SyslineFind::Found((fo_b, syslinep))
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
    /// ResultS4::Found(19, SyslineP(data='20010102␊'))
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
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeLOpt,
    ) -> ResultS4SyslineFind {
        dpnf!("(SyslineReader@{:p}, {}, {:?})", self, fileoffset, dt_filter);
        let filesz: FileSz = self.filesz();
        let _fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        let mut try_fo_last: FileOffset = try_fo;
        let mut fo_last: FileOffset = fileoffset;
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
            dpof!("loop(…)!");
            let result: ResultS4SyslineFind = self.find_sysline(try_fo);
            let done = result.is_done();
            match result {
                ResultS4SyslineFind::Found((fo, syslinep))
                => {
                    dpof!(
                        "FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?} C",
                        fo,
                        &(*syslinep),
                        syslinep.lines.len(),
                        (*syslinep).len(),
                        (*syslinep).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    dpof!(
                        "sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        &*syslinep,
                        (*syslinep).dt,
                        dt_filter,
                    );
                    match SyslineReader::sysline_dt_after_or_before(&syslinep, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            dpof!(
                                "Pass => fo {} fo_last {} try_fo {} try_fo_last {} (fo_end {})",
                                fo,
                                fo_last,
                                try_fo,
                                try_fo_last,
                                _fo_end,
                            );
                            dpxf!(
                                "return ResultS4SyslineFind::Found(({}, @{:p})); A",
                                fo,
                                &*syslinep,
                            );
                            return ResultS4SyslineFind::Found((fo, syslinep));
                        } // end Pass
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            dpof!("OccursAtOrAfter => fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            // short-circuit a common case, passed fileoffset is past the `dt_filter`, can immediately return
                            // XXX: does this mean my algorithm sucks?
                            if try_fo == fileoffset {
                                // first loop iteration
                                dpof!(
                                    "                    try_fo {} == {} try_fo_last; early return",
                                    try_fo,
                                    try_fo_last,
                                );
                                dpxf!(
                                    "return ResultS4SyslineFind::Found(({}, @{:p})); B fileoffset {} {:?}",
                                    fo,
                                    &*syslinep,
                                    (*syslinep).fileoffset_begin(),
                                    (*syslinep).to_String_noraw(),
                                );
                                return ResultS4SyslineFind::Found((fo, syslinep));
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*syslinep).fileoffset_begin(), try_fo_last);
                            dpof!(
                                "                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                fo_a,
                                fo_b,
                                fo_a,
                            );
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}, FPath {:?}", fo_a, fo_b, self.path());
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursAtOrAfter
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            dpof!("OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            let syslinep_foe: FileOffset = (*syslinep).fileoffset_end();
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, syslinep_foe, "Unexpected values try_fo_last {} syslinep_foe {}, last tried offset (passed to self.find_sysline({})) is beyond returned Sysline@{:p}.fileoffset_end() {}!? FPath {:?}", try_fo_last, syslinep_foe, try_fo, syslinep, syslinep_foe, self.path());
                            dpof!("                    ∴ fo_a = min(syslinep_foe {}, fo_b {});", syslinep_foe, fo_b);
                            fo_a = std::cmp::min(syslinep_foe, fo_b);
                            dpof!(
                                "                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                fo_a,
                                fo_b,
                                fo_a,
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    dpof!("                    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                    fo_last = fo;
                    syslinep_opt = Some(syslinep);
                    // TODO: [2021/09/26]
                    //       I think this could do an early check and potentially skip a few loops.
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Done
                } // end Found
                ResultS4SyslineFind::Done => {
                    dpof!("SyslineReader.find_sysline(try_fo: {}) returned Done", try_fo);
                    dpof!(
                        "                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                    try_fo_last = try_fo;
                    dpof!(
                        "                 ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                        fo_a,
                        fo_b,
                        fo_a
                    );
                    try_fo = fo_a + ((fo_b - fo_a) / 2);
                    dpof!(
                        "                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                } // end Done
                ResultS4SyslineFind::Err(_err) => {
                    dpof!(
                        "SyslineReader.find_sysline(try_fo: {}) returned Err({})",
                        try_fo,
                        _err,
                    );
                    dp_err!("{}", _err);
                    break;
                } // end Err
            } // match result
            dpof!("next loop will try offset {} (fo_end {})", try_fo, _fo_end);

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
                dpof!("Done && try_fo {} == {} try_fo_last; break!", try_fo, try_fo_last);
                break;
            } else if try_fo != try_fo_last {
                continue;
            }
            dpof!("try_fo {} == {} try_fo_last;", try_fo, try_fo_last);
            let mut syslinep = syslinep_opt.unwrap();
            let fo_beg: FileOffset = syslinep.fileoffset_begin();
            if self.is_sysline_last(&syslinep) && fo_beg < try_fo {
                // binary search stopped at fileoffset past start of last Sysline in file
                // so entirely past all acceptable syslines
                dpxf!("return ResultS4SyslineFind::Done; C binary searched ended after beginning of last sysline in the file");
                return ResultS4SyslineFind::Done;
            }
            // binary search loop is deciding on the same fileoffset upon each loop. That
            // fileoffset must refer to an acceptable sysline. However, if that fileoffset is past
            // `syslinep.fileoffset_begin` than the threshold change of datetime for the
            // `dt_filter` is the *next* Sysline.
            let fo_next: FileOffset = syslinep.fileoffset_next();
            if fo_beg < try_fo {
                dpof!("syslinep.fileoffset_begin() {} < {} try_fo;", fo_beg, try_fo);
                let syslinep_next: SyslineP = match self.find_sysline(fo_next) {
                    ResultS4SyslineFind::Found((_, syslinep_)) => {
                        dpof!(
                            "SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                            fo_next,
                            syslinep_
                        );
                        syslinep_
                    }
                    ResultS4SyslineFind::Done => {
                        dpof!(
                            "SyslineReader.find_sysline(fo_next1: {}) unexpectedly returned Done",
                            fo_next
                        );
                        break;
                    }
                    ResultS4SyslineFind::Err(_err) => {
                        dpof!(
                            "SyslineReader.find_sysline(fo_next1: {}) returned Err({})",
                            fo_next,
                            _err,
                        );
                        dp_err!("{}", _err);
                        break;
                    }
                };
                dpof!("dt_filter:                   {:?}", dt_filter);
                dpof!(
                    "syslinep      : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    fo_beg,
                    (*syslinep).fileoffset_end(),
                    (*syslinep).dt.unwrap(),
                    (*syslinep).to_String_noraw()
                );
                dpof!(
                    "syslinep_next : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    (*syslinep_next).fileoffset_begin(),
                    (*syslinep_next).fileoffset_end(),
                    (*syslinep_next).dt.unwrap(),
                    (*syslinep_next).to_String_noraw()
                );
                let syslinep_compare = dt_after_or_before(&(*syslinep).dt.unwrap(), dt_filter);
                let syslinep_next_compare = dt_after_or_before(&(*syslinep_next).dt.unwrap(), dt_filter);
                dpof!("match({:?}, {:?})", syslinep_compare, syslinep_next_compare);
                syslinep = match (syslinep_compare, syslinep_next_compare) {
                    (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                        dpof!("unexpected Result_Filter_DateTime1::Pass");
                        eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                        break;
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                        dpof!("choosing syslinep_next");
                        syslinep_next
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        dpof!("choosing syslinep_next");
                        syslinep_next
                    }
                    (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        dpof!("choosing syslinep");
                        syslinep
                    }
                    _ => {
                        dpof!("unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                        eprintln!("ERROR: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                        break;
                    }
                };
            } else {
                dpof!("syslinep.fileoffset_begin() {} >= {} try_fo; use syslinep", fo_beg, try_fo);
            }
            let fo_: FileOffset = syslinep.fileoffset_next();
            dpxf!("return ResultS4SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}", fo_, &*syslinep, (*syslinep).fileoffset_begin(), (*syslinep).to_String_noraw());
            return ResultS4SyslineFind::Found((fo_, syslinep));
        } // end loop

        dpxf!("return ResultS4SyslineFind::Done; E");

        ResultS4SyslineFind::Done
    }

    /// Wrapper function for `dt_after_or_before`.
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeLOpt) -> Result_Filter_DateTime1 {
        dpnxf!("(Sysline@[{:?}, {:?}], {:?})", (*syslinep).fileoffset_begin(), (*syslinep).fileoffset_end(), dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline@{:p} does not have a datetime set.", &*syslinep);

        let dt: &DateTimeL = (*syslinep).dt.as_ref().unwrap();

        dt_after_or_before(dt, dt_filter)
    }

    /// Wrapper function for call to `dt_pass_filters`.
    pub fn sysline_pass_filters(
        syslinep: &SyslineP, dt_filter_after: &DateTimeLOpt, dt_filter_before: &DateTimeLOpt,
    ) -> Result_Filter_DateTime2 {
        dpnf!(
            "(Sysline[{:?}, {:?}], {:?}, {:?})",
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            dt_filter_after,
            dt_filter_before,
        );
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);
        let dt: &DateTimeL = (*syslinep).dt.as_ref().unwrap();
        let result: Result_Filter_DateTime2 = dt_pass_filters(dt, dt_filter_after, dt_filter_before);
        dpxf!("(…) return {:?};", result);

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
        &mut self, fileoffset: FileOffset, dt_filter_after: &DateTimeLOpt, dt_filter_before: &DateTimeLOpt,
    ) -> ResultS4SyslineFind {
        dpnf!("({}, {:?}, {:?})", fileoffset, dt_filter_after, dt_filter_before);

        match self.find_sysline_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS4SyslineFind::Found((fo, syslinep)) => {
                dpof!("returned ResultS4SyslineFind::Found(({}, {:?}))", fo, syslinep);
                match Self::sysline_pass_filters(&syslinep, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::InRange => {
                        dpof!("sysline_pass_filters(…) returned InRange;");
                        dpxf!("return ResultS4SyslineFind::Found(({}, {:?}))", fo, syslinep);
                        return ResultS4SyslineFind::Found((fo, syslinep));
                    },
                    Result_Filter_DateTime2::BeforeRange => {
                        dpof!("sysline_pass_filters(…) returned BeforeRange;");
                        eprintln!("ERROR: sysline_pass_filters(Sysline@{:p}, {:?}, {:?}) returned BeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  syslinep, dt_filter_after, dt_filter_before,
                                  fileoffset, dt_filter_after
                        );
                        dpxf!("return ResultS4SyslineFind::Done (not sure what to do here)");
                        return ResultS4SyslineFind::Done;
                    },
                    Result_Filter_DateTime2::AfterRange => {
                        dpof!("sysline_pass_filters(…) returned AfterRange;");
                        dpxf!("return ResultS4SyslineFind::Done");
                        return ResultS4SyslineFind::Done;
                    },
                };
            },
            ResultS4SyslineFind::Done => {},
            ResultS4SyslineFind::Err(err) => {
                dpof!("({}, dt_after: {:?}) returned Err({})", fileoffset, dt_filter_after, err);
                dp_err!("{}", err);
                dpxf!("return ResultS4SyslineFind::Err({})", err);
                return ResultS4SyslineFind::Err(err);
            },
        };

        dpxf!("return ResultS4SyslineFind::Done");

        ResultS4SyslineFind::Done
    }
}
