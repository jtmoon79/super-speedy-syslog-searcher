// Readers/syslinereader.rs
//

use crate::common::{
    FPath,
    FileOffset,
    FileType,
    NLu8,
    CharSz,
};

use crate::common::{
    Count,
    FileSz,
    Bytes,
    ResultS4,
};

use crate::Data::sysline::{
    Sysline,
    SyslineP,
    SyslineP_Opt,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
};

use crate::Data::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeL_Opt,
    DateTime_Parse_Data,
    DateTimeRegex,
    DateTime_Parse_Datas_Regex_vec,
    DateTime_Parse_Datas_Index,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
    DATETIME_PARSE_DATAS_REGEX_VEC,
    bytes_to_regex_to_datetime,
    dt_pass_filters,
    dt_after_or_before,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    slice_contains_X_2,
    u8_to_str,
};

use crate::Data::line::{
    LineIndex,
    Line,
    LineP,
    LinePartPtrs,
};

use crate::Readers::linereader::{
    LineReader,
    ResultS4_LineFind,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::printers::{
    str_to_String_noraw,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::stack::{
    sn,
    snx,
    so,
    sx,
};

use std::collections::{
    BTreeMap,
    BTreeSet,
    HashMap,
    HashSet,
};
use std::fmt;
use std::io::{
    Error,
    Result,
    ErrorKind,
};
use std::str;
use std::sync::Arc;

extern crate debug_print;
use debug_print::debug_eprintln;
#[cfg(debug_assertions)]
use debug_print::debug_eprint;

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
    assert_ge,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
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

/// count of datetime format strings used
///
/// key is index into global `DATETIME_PARSE_DATAS_VEC` and `DATETIME_PARSE_DATAS_REGEX_VEC`
///
/// value is count of use of those "pattern rules" to find datetimes in a `Line`
pub type DateTime_Pattern_Counts = BTreeMap<DateTime_Parse_Datas_Index, Count>;
pub type DateTime_Parse_Data_Indexes = BTreeSet<DateTime_Parse_Datas_Index>;
/// data returned by `SyslineReader::find_datetime_in_line` and
/// `SyslineReader::parse_datetime_in_line`
///
/// - datetime substring index begin
/// - datetime substring index end
/// - the datetime found
/// - index into global `DATETIME_PARSE_DATAS_VEC` and `DATETIME_PARSE_DATAS_REGEX_VEC` for the
///   "pattern rules" used to find the datetime.
pub type FindDateTimeData = (LineIndex, LineIndex, DateTimeL, DateTime_Parse_Datas_Index);
/// return type for `SyslineReader::find_datetime_in_line`
pub type Result_FindDateTime = Result<FindDateTimeData>;
/// return type for `SyslineReader::parse_datetime_in_line`
pub type Result_ParseDateTime = Result<FindDateTimeData>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Sysline Searching result
#[allow(non_camel_case_types)]
pub type ResultS4_SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;
/// storage for `Sysline`
pub type Syslines = BTreeMap<FileOffset, SyslineP>;
/// range map where key is sysline begin to end `[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`). Use the value to lookup associated `Syslines` map
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;
/// used internally by `SyslineReader`
type SyslinesLRUCache = LruCache<FileOffset, ResultS4_SyslineFind>;
/// used internally by `SyslineReader`
type LineParsedCache = LruCache<FileOffset, FindDateTimeData>;

/// Specialized reader that uses `LineReader` to find `Sysline`s in a file.
///
/// A `SyslineReader` does some `[u8]` to `char` interpretation.
///
/// A `SyslineReader` stores past lookups of data.
///
/// XXX: not a rust "Reader"; does not implement trait `Read`
pub struct SyslineReader {
    pub(crate) linereader: LineReader,
    /// Syslines keyed by fileoffset_begin
    pub(crate) syslines: Syslines,
    /// count of Syslines processed
    syslines_count: Count,
    /// internal stats for `self.find_sysline()` use of `self.syslines`
    pub(crate) syslines_hit: Count,
    /// internal stats for `self.find_sysline()` use of `self.syslines`
    pub(crate) syslines_miss: Count,
    /// Syslines fileoffset by sysline fileoffset range, i.e. `[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`
    /// the stored value can be used as a key for `self.syslines`
    syslines_by_range: SyslinesRangeMap,
    /// count of `self.syslines_by_range` lookup hit
    pub(crate) syslines_by_range_hit: Count,
    /// count of `self.syslines_by_range` lookup miss
    pub(crate) syslines_by_range_miss: Count,
    /// count of `self.syslines_by_range.insert`
    pub(crate) syslines_by_range_put: Count,
    /// datetime formatting patterns, for finding datetime strings from Lines
    /// TODO: change to a `Set`
    //pub(crate) dt_patterns: DateTime_Parse_Datas_vec,
    /// first (soonest) processed DateTimeL (not necessarly printed, not representative of the entire file)
    ///
    /// intended for `--summary`
    pub(crate) dt_first: DateTimeL_Opt,
    /// last (latest) processed DateTimeL (not necessarly printed, not representative of the entire file)
    ///
    /// intended for `--summary``
    pub(crate) dt_last: DateTimeL_Opt,
    /// counts found patterns stored in `dt_patterns`
    /// not used after `self.analyzed` becomes `true`
    pub(crate) dt_patterns_counts: DateTime_Pattern_Counts,
    /// keys of `dt_patterns_counts` sorted by value. updated in `dt_patterns_indexes_refresh`
    dt_patterns_indexes: DateTime_Parse_Data_Indexes,
    /// default FixedOffset for a found `DateTime` without timezone
    tz_offset: FixedOffset,
    /// enable or disable the internal LRU cache for `find_sysline()`
    find_sysline_lru_cache_enabled: bool,
    /// internal LRU cache for `find_sysline()`. maintained in `SyslineReader::find_sysline`
    /// TODO: remove `pub(crate)`
    pub(crate) find_sysline_lru_cache: SyslinesLRUCache,
    /// count of internal LRU cache lookup hits
    pub(crate) find_sysline_lru_cache_hit: Count,
    /// count of internal LRU cache lookup misses
    pub(crate) find_sysline_lru_cache_miss: Count,
    /// count of internal LRU cache lookup `.put`
    pub(crate) find_sysline_lru_cache_put: Count,
    /// enable/disable `parse_datetime_in_line_lru_cache`
    parse_datetime_in_line_lru_cache_enabled: bool,
    /// internal cache of calls to `SyslineReader::parse_datetime_in_line()`. maintained in `SyslineReader::find_sysline()`
    parse_datetime_in_line_lru_cache: LineParsedCache,
    /// count of `self.parse_datetime_in_line_lru_cache` lookup hit
    pub(crate) parse_datetime_in_line_lru_cache_hit: Count,
    /// count of `self.parse_datetime_in_line_lru_cache` lookup miss
    pub(crate) parse_datetime_in_line_lru_cache_miss: Count,
    /// count of `self.parse_datetime_in_line_lru_cache.put`
    pub(crate) parse_datetime_in_line_lru_cache_put: Count,
    /// count of `line.get_boxptrs` returning `SinglePtr`
    pub(crate) get_boxptrs_singleptr: Count,
    /// count of `line.get_boxptrs` returning `DoublePtr`
    pub(crate) get_boxptrs_doubleptr: Count,
    /// count of `line.get_boxptrs` returning `MultiPtr`
    pub(crate) get_boxptrs_multiptr: Count,
    /// has `self.file_analysis` completed?
    ///
    /// Initially `false`. During `parse_datetime_in_line` all patterns in
    /// `DATETIME_PARSE_DATAS_VEC` may be used for a search. During this time,
    /// `dt_pattern_counts` is updated in `dt_patterns_indexes_refresh`.
    ///
    /// Once `true` then only a subset of patterns that successfully matched syslines are used.
    /// This avoids likely fruitless or misleading searches for datetime in a `Line`. Those
    /// searches are resource expensive.
    analyzed: bool,
    /// count of Ok to Arc::try_unwrap(syslinep), effectively a count of
    /// `Sysline` dropped
    pub(crate) drop_sysline_ok: Count,
    /// count of failures to Arc::try_unwrap(syslinep). A failure does not mean an error.
    pub(crate) drop_sysline_errors: Count,
}

// TODO: [2021/09/19]
//       put all filter data into one struct `SyslineFilter`, simpler to pass around

impl fmt::Debug for SyslineReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SyslineReader")
            .field("linereader", &self.linereader)
            .field("syslines", &self.syslines)
            .finish()
    }
}

/// quick debug helper
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
    debug_eprint!("[");
    for (key, val) in cache.iter() {
        debug_eprint!(" Key: {:?}, Value: {:?};", key, val);
    }
    debug_eprint!("]");
}

/// implement SyslineReader things
impl SyslineReader {
    /// maximum number of datetime patterns to match when first reading a file (before settling on one)
    const DT_PATTERN_MAX_PRE_ANALYSIS: usize = 4;
    /// maximum number of datetime patterns for matching the remainder of a syslog file
    const DT_PATTERN_MAX: usize = 1;
    /// when this number of syslines has been processed then reduce use of all
    /// patterns from all patterns in `DATETIME_PARSE_DATAS_VEC` to one pattern
    #[cfg(any(debug_assertions,test))]
    const DT_PATTERN_ANALYSIS_THRESHOLD: Count = 1;
    #[cfg(not(any(debug_assertions,test)))]
    const DT_PATTERN_ANALYSIS_THRESHOLD: Count = 5;
    /// capacity of internal LRU cache
    const FIND_SYSLINE_LRU_CACHE_SZ: usize = 4;
    /// capacity of internal LRU cache
    const PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ: usize = 8;
    /// A `Line.len()` must at least this value to proceed with a datetime search, if `Line.len()`
    /// is less then it is presumed no datetime string could fit on the line.
    ///
    /// This allows skipping a few datetime searches that would fail.
    const DATETIME_STR_MIN: usize = 8;

    pub fn new(path: FPath, filetype: FileType, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslineReader> {
        debug_eprintln!("{}SyslineReader::new({:?}, {:?}, {:?}, {:?})", snx(), path, filetype, blocksz, tz_offset);
        let lr = match LineReader::new(path, filetype, blocksz) {
            Ok(val) => val,
            Err(err) => {
                //eprintln!("ERROR: LineReader::new({}, {}) failed {}", path, blocksz, err);
                return Err(err);
            }
        };
        let mut dt_patterns_counts = DateTime_Pattern_Counts::new();
        let mut dt_patterns_indexes = DateTime_Parse_Data_Indexes::new();
        let mut index = 0;
        while index < DATETIME_PARSE_DATAS_LEN {
            dt_patterns_counts.insert(index as DateTime_Parse_Datas_Index, 0);
            dt_patterns_indexes.insert(index as DateTime_Parse_Datas_Index);
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
                //dt_patterns: DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
                dt_first: None,
                dt_last: None,
                dt_patterns_counts,
                dt_patterns_indexes,
                tz_offset,
                find_sysline_lru_cache_enabled: true,
                find_sysline_lru_cache: SyslinesLRUCache::new(SyslineReader::FIND_SYSLINE_LRU_CACHE_SZ),
                find_sysline_lru_cache_hit: 0,
                find_sysline_lru_cache_miss: 0,
                find_sysline_lru_cache_put: 0,
                parse_datetime_in_line_lru_cache_enabled: true,
                parse_datetime_in_line_lru_cache: LineParsedCache::new(SyslineReader::PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ),
                parse_datetime_in_line_lru_cache_hit: 0,
                parse_datetime_in_line_lru_cache_miss: 0,
                parse_datetime_in_line_lru_cache_put: 0,
                get_boxptrs_singleptr: 0,
                get_boxptrs_doubleptr: 0,
                get_boxptrs_multiptr: 0,
                analyzed: false,
                //drop_block_fo_keys: Vec::<FileOffset>::with_capacity(drop_block_fo_keys_sz),
                drop_sysline_ok: 0,
                drop_sysline_errors: 0,
            }
        )
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.linereader.filetype()
    }

    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.linereader.filesz()
    }

    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.linereader.mimeguess()
    }

    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        self.linereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub const fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.linereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub const fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.linereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub const fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub const fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.linereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub const fn count_blocks(&self) -> Count {
        self.linereader.count_blocks()
    }

    /// last valid `BlockOffset` of the file
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.linereader.blockoffset_last()
    }

    /// smallest size character in bytes
    pub const fn charsz(&self) -> usize {
        self.linereader.charsz()
    }

    /// count of `Sysline`s processed so far, i.e. `self.syslines_count`
    pub fn count_syslines_processed(&self) -> Count {
        self.syslines_count
    }

    /// count of `Sysline`s processed so far, i.e. `self.syslines_count`
    pub fn count_syslines_stored(&self) -> Count {
        self.syslines.len() as Count
    }

    /// count underlying `Line`s processed so far
    #[inline(always)]
    pub fn count_lines_processed(&self) -> Count {
        self.linereader.count_lines_processed()
    }

    /// enable internal LRU cache used by `find_sysline` and `parse_datetime_in_line`
    /// intended to aid testing and debugging
    #[allow(dead_code)]
    pub fn LRU_cache_enable(&mut self) {
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
    }

    /// disable internal LRU cache used by `find_sysline` and `parse_datetime_in_line`
    /// intended to aid testing and debugging
    pub fn LRU_cache_disable(&mut self) {
        self.find_sysline_lru_cache_enabled = false;
        self.find_sysline_lru_cache.resize(0);
        self.parse_datetime_in_line_lru_cache_enabled = false;
        self.parse_datetime_in_line_lru_cache.resize(0);
    }

    /// print Sysline at `fileoffset`
    ///
    /// Testing helper only
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

    /// is this `SyslineP` the last `Sysline` of the entire file?
    /// (not the same as last Sysline within the optional datetime filters).
    pub fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        let filesz = self.filesz();
        let fo_end = (*syslinep).fileoffset_end();
        if (fo_end + 1) == filesz {
            debug_eprintln!("{}is_sysline_last(…); return true, fo_end+1 {} == {} filesz", snx(), fo_end + 1, filesz);
            return true;
        }
        assert_lt!(fo_end + 1, filesz, "fileoffset_end {}+1 = {}, is at or after filesz() {}", fo_end, fo_end + 1, filesz);
        debug_eprintln!("{}is_sysline_last(…); return false, fo_end+1 {} != {} filesz", snx(), fo_end + 1, filesz);

        false
    }

    /// store passed `Sysline` in `self.syslines`, update other fields
    fn insert_sysline(&mut self, line: Sysline) -> SyslineP {
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let syslinep = SyslineP::new(line);
        debug_eprintln!("{}SyslineReader.insert_sysline: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*syslinep);
        self.syslines.insert(fo_beg, syslinep.clone());
        self.syslines_count += 1;
        // XXX: multi-byte character
        let fo_end1 = fo_end + (self.charsz() as FileOffset);
        debug_eprintln!(
            "{}SyslineReader.insert_sysline: syslines_by_range.insert(({}‥{}], {})",
            so(),
            fo_beg,
            fo_end1,
            fo_beg
        );
        self.syslines_by_range.insert(fo_beg..fo_end1, fo_beg);
        self.syslines_by_range_put += 1;
        syslinep
    }

    /// drop as much data as possible that uses the referred `Block`
    pub fn drop_block(&mut self, blockoffset: BlockOffset, bo_dropped: &mut HashSet<BlockOffset>) {
        debug_eprintln!("{}syslinereader.drop_block({})", sn(), blockoffset);

        // TODO: [2022/06/18] make this a "one time" creation that is reused
        //       this is challenging, as it runs into borrow errors during `.iter()`
        let mut drop_block_fo_keys: Vec<FileOffset> = Vec::<FileOffset>::with_capacity(self.syslines.len());

        for fo_key in self.syslines.keys() {
            drop_block_fo_keys.push(*fo_key);
        }
        // vec of `fileoffset` must be ordered which is guaranteed by `syslines: BTreeMap`

        debug_eprintln!("{}syslinereader.drop_block: collected keys {:?}", so(), drop_block_fo_keys);

        // XXX: using `sylines.value_mut()` would be cleaner.
        //      But `sylines.value_mut()` causes a clone of the `SyslineP`, which then
        //      increments the `Arc` "strong_count". That in turn prevents `Arc::get_mut(&SyslineP)`
        //      from returning the original `Sysline`.
        //      Instead of `syslines.values_mut()`, use `syslines.keys()` and then `syslines.get_mut`
        //      to get a `&SyslineP`. This does not increase the "strong_count".

        for fo_key in drop_block_fo_keys.iter() {
            let bo_last = self.syslines[fo_key].blockoffset_last();
            if bo_last > blockoffset {
                debug_eprintln!("{}syslinereader.drop_block: blockoffset_last {} > {} blockoffset, continue;", so(), bo_last, blockoffset);
                // presume all proceeding `Sysline.blockoffset_last()` will be after `blockoffset`
                break;
            }
            // XXX: copy `fo_key` to avoid borrowing error
            self.drop_sysline(fo_key, bo_dropped);
            debug_eprintln!("{}syslinereader.drop_block: bo_dropped {:?}", so(), bo_dropped);
        }

        debug_eprintln!("{}syslinereader.drop_block({})", sx(), blockoffset);
    }

    /// drop all data associated with `Sysline` at `fileoffset` (or at least, drop as much
    /// as possible).
    ///
    /// Caller must know what they are doing!
    pub fn drop_sysline(&mut self, fileoffset: &FileOffset, bo_dropped: &mut HashSet<BlockOffset>) {
        debug_eprintln!("{}syslinereader.drop_sysline({})", sn(), fileoffset);
        let syslinep: SyslineP = match self.syslines.remove(fileoffset) {
            Some(syslinep_) => syslinep_,
            None => {
                debug_eprintln!("syslinereader.drop_sysline: syslines.remove({}) returned None which is unexpected", fileoffset);
                return;
            }
        };
        debug_eprintln!("{}syslinereader.drop_sysline: Processing SyslineP @[{}‥{}], Block @[{}‥{}] strong_count {}", so(), (*syslinep).fileoffset_begin(), (*syslinep).fileoffset_end(), (*syslinep).blockoffset_first(), (*syslinep).blockoffset_last(), Arc::strong_count(&syslinep));
        self.find_sysline_lru_cache.pop(&(*syslinep).fileoffset_begin());
        match Arc::try_unwrap(syslinep) {
            Ok(sysline) => {
                debug_eprintln!("{}syslinereader.drop_sysline: Arc::try_unwrap(syslinep) Ok Sysline @[{}‥{}] Block @[{}‥{}]", so(), sysline.fileoffset_begin(), sysline.fileoffset_end(), sysline.blockoffset_first(), sysline.blockoffset_last());
                self.drop_sysline_ok += 1;
                self.linereader.drop_lines(sysline.lines, bo_dropped);
            }
            Err(_syslinep) => {
                debug_eprintln!("{}syslinereader.drop_sysline: Arc::try_unwrap(syslinep) Err strong_count {}", so(), Arc::strong_count(&_syslinep));
                self.drop_sysline_errors += 1;
            }
        }
    }

    /// if datetime found in `Line` returns `Ok` around
    /// indexes into `line` of found datetime string `(start of string, end of string)`
    ///
    /// else returns `Err`
    pub fn find_datetime_in_line(
        line: &Line,
        parse_data_indexes: &DateTime_Parse_Data_Indexes,
        charsz: &CharSz,
        tz_offset: &FixedOffset,
        get_boxptrs_singleptr: &mut Count,
        get_boxptrs_doubleptr: &mut Count,
        get_boxptrs_multiptr: &mut Count,
    ) -> Result_FindDateTime {
        debug_eprintln!("{}find_datetime_in_line:(Line, {:?})", sn(), line.to_String_noraw());
        debug_eprintln!("{}find_datetime_in_line: parse_data_indexes.len() {} {:?}", so(), parse_data_indexes.len(), parse_data_indexes);

        // skip an easy case; no possible datetime
        if line.len() < SyslineReader::DATETIME_STR_MIN {
            debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::InvalidInput);", sx());
            return Result_FindDateTime::Err(Error::new(ErrorKind::InvalidInput, "Line is too short to hold a datetime"));
        }

        const HACK12: &[u8; 2] = b"12";
        // `sie` and `siea` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        for (at, index) in parse_data_indexes.iter().enumerate() {
            let dtpd: &DateTime_Parse_Data = &DATETIME_PARSE_DATAS[*index];
            debug_eprintln!("{}find_datetime_in_line: pattern data try {} index {} dtpd.line_num {}", so(), at, index, dtpd._line_num);
            // XXX: does not support multi-byte string; assumes single-byte
            let line_end: usize;
            if line.len() > dtpd.range_regex.len() {
                line_end = dtpd.range_regex.len();
            } else {
                line_end = line.len() - 1;
            }
            if dtpd.range_regex.start >= line_end {
                debug_eprintln!("{}find_datetime_in_line: bad line slice indexes [{}, {}); continue", so(), dtpd.range_regex.start, line_end);
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // this is to force the parsing function `Local.datetime_from_str` to constrain where it
            // searches within the `Line`
            let mut hack_slice: Bytes;
            let slice_: &[u8];
            match line.get_boxptrs(dtpd.range_regex.start as LineIndex, line_end as LineIndex) {
                LinePartPtrs::NoPtr => {
                    panic!("line.get_boxptrs({}, {}) returned NoPtr which means it was passed non-sense values", dtpd.range_regex.start, line_end);
                    continue;
                }
                LinePartPtrs::SinglePtr(box_slice) => {
                    slice_ = *box_slice;
                    *get_boxptrs_singleptr += 1;
                }
                LinePartPtrs::DoublePtr(box_slice2) => {
                    hack_slice = Bytes::with_capacity(box_slice2.0.len() + box_slice2.1.len());
                    hack_slice.extend_from_slice(*box_slice2.0);
                    hack_slice.extend_from_slice(*box_slice2.1);
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
            // hack efficiency improvement, presumes all found years will have a '1' or a '2' in them
            if charsz == &1 && dtpd.dtfs.has_year() && !slice_contains_X_2(slice_, HACK12) {
                debug_eprintln!("{}find_datetime_in_line: skip slice, does not have '1' or '2'", so());
                continue;
            }
            // found the datetime string using `regex`, convert to a `DateTimeL`
            let dt: DateTimeL;
            let dt_beg: LineIndex;
            let dt_end: LineIndex;
            (dt_beg, dt_end, dt) = 
                //match str_to_regex_to_datetime(dts, index, tz_offset) {
                match bytes_to_regex_to_datetime(slice_, index, tz_offset) {
                    None => continue,
                    Some(val) => val,
            };
            debug_eprintln!("{}find_datetime_in_line: return Ok({}, {}, {}, {});", sx(), dt_beg, dt_end, &dt, index);
            return Result_FindDateTime::Ok((dt_beg, dt_end, dt, *index));
        }  // end for(pattern, ...)

        debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::NotFound);", sx());
        Result_FindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found in Line!"))
    }

    /// update the two statistic `DateTimeL` of `self.dt_first` and `self.dt_last`
    fn dt_first_last_update(&mut self, datetime: &DateTimeL) {
        debug_eprintln!("{}syslinereader.dt_first_last_update({:?})", snx(), datetime);
        // TODO: the `dt_first` and `dt_last` are only for `--summary`, would be good to only run this
        //       when `if self.do_summary {...}`
        match self.dt_first {
            Some(dt_first_) => {
                if &dt_first_ > datetime {
                    self.dt_first = Some(*datetime);
                }
            },
            None => { self.dt_first = Some(*datetime); }
        }
        match self.dt_last {
            Some(dt_last_) => {
                if &dt_last_ < datetime {
                    self.dt_last = Some(*datetime);
                }
            },
            None => { self.dt_last = Some(*datetime); }
        }

    }

    /// helper function to update `parse_datetime_in_line`
    fn dt_patterns_update(&mut self, index: DateTime_Parse_Datas_Index) {
        debug_eprintln!("{}syslinereader.dt_patterns_update({:?})", snx(), index);
        if let std::collections::btree_map::Entry::Vacant(entry_) = self.dt_patterns_counts.entry(index) {
            // first count of this index so insert it
            //entry_.insert(1);
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
        // this makes the most-used parse_data more likley to be used again
        self.dt_patterns_indexes.extend(
            self.dt_patterns_counts.iter().sorted_by(
                |a, b| Ord::cmp(&b.1, &a.1) // sort by value (second tuple item)
            ).map(|(k, _v)| k) // copy only the key (first tuple item) which is an index
        );
        debug_eprintln!("{}syslinereader.dt_patterns_indexes_refresh() dt_patterns_indexes {:?}", snx(), self.dt_patterns_indexes);
    }

    /// analyze syslines gathered
    ///
    /// When a threshold of syslines or bytes has been processed, then
    /// this function narrows down datetime formats to try for future datetime-parsing attempts.
    /// This is a very large reduction of resources/time used by
    /// `SyslineReader::find_datetime_in_line`.
    fn dt_patterns_analysis(&mut self) {
        if self.count_syslines_processed() < SyslineReader::DT_PATTERN_ANALYSIS_THRESHOLD {
            return;
        }
        debug_eprintln!("{}dt_patterns_analysis()", sn());
        // XXX: DT_PATERN_MAX > 1 is unimplemented
        const_assert!(SyslineReader::DT_PATTERN_MAX == 1);
        if cfg!(debug_assertions) {
            for (_k, _v) in self.dt_patterns_counts.iter() {
                let data_: &DateTime_Parse_Data = &DATETIME_PARSE_DATAS[*_k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC.get(*_k).unwrap();
                debug_eprintln!("{}dt_patterns_analysis: self.dt_patterns_counts[{:?}]={:?} is {:?}, {:?}", so(), _k, _v, data_, data_rex_);
            }
        }
        // ripped from https://stackoverflow.com/a/60134450/471376
        // test https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b8eb53f40fd89461c9dad9c976746cc3
        let max_ = (&self.dt_patterns_counts).iter().fold(
            std::u64::MIN, |a,b| a.max(*(b.1))
        );
        debug_eprintln!("{}dt_patterns_analysis: dt_patterns_counts.retain(v >= {:?})", so(), max_);
        self.dt_patterns_counts.retain(|_, v| *v >= max_);
        if self.dt_patterns_counts.len() != 1 {
            debug_eprintln!("ERROR: dt_patterns_analysis: self.dt_patterns_counts.len() {}, expected 1", self.dt_patterns_counts.len());
        }
        self.dt_patterns_indexes_refresh();
        if cfg!(debug_assertions) {
            for (_k, _v) in self.dt_patterns_counts.iter() {
                let data_: &DateTime_Parse_Data = &DATETIME_PARSE_DATAS[*_k];
                let data_rex_: &DateTimeRegex = DATETIME_PARSE_DATAS_REGEX_VEC.get(*_k).unwrap();
                debug_eprintln!("{}dt_patterns_analysis: self.dt_patterns_counts[{:?}]={:?} is {:?}, {:?}", so(), _k, _v, data_, data_rex_);
            }
        }
        self.analyzed = true;
        debug_eprintln!("{}dt_patterns_analysis()", sx());
    }

    /// attempt to parse a DateTime substring in the passed `Line`
    /// wraps call to `self.find_datetime_in_line` according to status of `self.dt_patterns`
    /// if `self.dt_patterns` is `None`, will set `self.dt_patterns`
    fn parse_datetime_in_line(&mut self, line: &Line, charsz: &CharSz) -> Result_ParseDateTime {
        // XXX: would prefer this at the end of this function but borrow error occurs
        if !self.analyzed {
            self.dt_patterns_analysis();
        }
        debug_eprintln!("{}parse_datetime_in_line({}, {:?})", sn(), charsz, line.to_String_noraw());

        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats
        // `self.dt_patterns`
        // TODO: [2022/06/26] cost-savings: create the `indexes` once in an analysis update function or somewhere else
        let mut indexes: DateTime_Parse_Data_Indexes = DateTime_Parse_Data_Indexes::new();
        // get copy of indexes sorted by value
        indexes.extend(
            self.dt_patterns_counts.iter().sorted_by(
                |a, b| Ord::cmp(&b.1, &a.1) // sort by value (second tuple item)
            ).map(|(k, _v)| k) // copy only the key (first tuple item) which is an index
        );
        debug_eprintln!("{}parse_datetime_in_line indexes {:?}", so(), indexes);
        let result: Result_FindDateTime = SyslineReader::find_datetime_in_line(
            line,
            &indexes,
            charsz,
            &self.tz_offset,
            &mut self.get_boxptrs_singleptr,
            &mut self.get_boxptrs_doubleptr,
            &mut self.get_boxptrs_multiptr,
        );
        let data: FindDateTimeData = match result {
            Ok(val) => val,
            Err(err) => {
                debug_eprintln!("{}parse_datetime_in_line return Err {};", sx(), err);
                return Result_ParseDateTime::Err(err);
            }
        };
        self.dt_patterns_update(data.3);
        self.dt_first_last_update(&data.2);
        debug_eprintln!("{}parse_datetime_in_line return {:?}", sx(), data);

        Result_ParseDateTime::Ok(data)
    }

    /// helper to `find_sysline`
    ///
    /// call `self.parse_datetime_in_line` with help of LRU cache `self.parse_datetime_in_line_lru_cache`
    fn parse_datetime_in_line_cached(&mut self, linep: &LineP, charsz: &CharSz) -> Result_ParseDateTime {
        if self.parse_datetime_in_line_lru_cache_enabled {
            match self.parse_datetime_in_line_lru_cache.get(&linep.fileoffset_begin()) {
                Some(val) => {
                    self.parse_datetime_in_line_lru_cache_hit +=1;
                    return Result_ParseDateTime::Ok(*val);
                }
                _ => {
                    self.parse_datetime_in_line_lru_cache_miss += 1;
                },
            }
        }
        // LAST WORKING HERE 2022/06/26 01:56:15
        // It's far better to just stored indexes into the global VECs.
        // Continue implemeting new approach which is to only use `self.dt_patterns_counts` for tracking
        // patterns (and eventaully rid of `self.dt_patterns`).
        // Need to finish correcting statements in this file then do a first run, see
        // how first few sysline proceses go.
        let result: Result_ParseDateTime = self.parse_datetime_in_line(&*linep, charsz);
        if self.parse_datetime_in_line_lru_cache_enabled {
            //#[allow(clippy::single_match)]
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

        result
    }

    /// check various internal storage for already processed `Sysline` at `fileoffset`
    fn check_store(&mut self, fileoffset: FileOffset) -> Option<ResultS4_SyslineFind> {
        debug_eprintln!("{}check_store({})", sn(), fileoffset);

        if self.find_sysline_lru_cache_enabled {
            // check if `fileoffset` is already known about in LRU cache
            match self.find_sysline_lru_cache.get(&fileoffset) {
                Some(results4) => {
                    self.find_sysline_lru_cache_hit += 1;
                    debug_eprintln!("{}check_store: found LRU cached for fileoffset {}", so(), fileoffset);
                    // the `.get` returns a reference `&ResultS4_SyslineFind` so must return a new `ResultS4_SyslineFind`
                    match results4 {
                        ResultS4_SyslineFind::Found(val) => {
                            debug_eprintln!("{}check_store: return ResultS4_SyslineFind::Found(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return Some(ResultS4_SyslineFind::Found((val.0, val.1.clone())));
                        }
                        ResultS4_SyslineFind::Found_EOF(val) => {
                            debug_eprintln!("{}check_store: return ResultS4_SyslineFind::Found_EOF(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return Some(ResultS4_SyslineFind::Found_EOF((val.0, val.1.clone())));
                        }
                        ResultS4_SyslineFind::Done => {
                            debug_eprintln!("{}check_store: return ResultS4_SyslineFind::Done from LRU cache", sx());
                            return Some(ResultS4_SyslineFind::Done);
                        }
                        ResultS4_SyslineFind::Err(err) => {
                            debug_eprintln!("{}check_store: Error {}", so(), err);
                            eprintln!("ERROR: unexpected value store in self._find_line_lru_cache.get({}) error {}", fileoffset, err);
                        }
                    }
                }
                None => {
                    self.find_sysline_lru_cache_miss += 1;
                    debug_eprintln!("{}check_store: fileoffset {} not found in LRU cache", so(), fileoffset);
                }
            }
        }

        // check if the offset is already in a known range
        match self.syslines_by_range.get_key_value(&fileoffset) {
            Some(range_fo) => {
                let range = range_fo.0;
                debug_eprintln!(
                "{}check_store: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                so(),
                fileoffset,
                range
            );
                self.syslines_by_range_hit += 1;
                let fo = range_fo.1;
                let syslinep = self.syslines[fo].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*syslinep).fileoffset_next() + (self.charsz() as FileOffset);
                if self.is_sysline_last(&syslinep) {
                    debug_eprintln!(
                        "{}check_store: is_sysline_last() true; return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*syslinep,
                        (*syslinep).fileoffset_begin(),
                        (*syslinep).fileoffset_end(),
                        (*syslinep).to_String_noraw()
                    );
                    self.find_sysline_lru_cache_put += 1;
                    self.find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, syslinep.clone())));
                    return Some(ResultS4_SyslineFind::Found_EOF((fo_next, syslinep)));
                }
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, syslinep.clone())));
                debug_eprintln!(
                    "{}check_store: is_sysline_last() false; return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] from self.syslines_by_range {:?}",
                    sx(),
                    fo_next,
                    &*syslinep,
                    (*syslinep).fileoffset_begin(),
                    (*syslinep).fileoffset_end(),
                    (*syslinep).to_String_noraw()
                );
                return Some(ResultS4_SyslineFind::Found((fo_next, syslinep)));
            }
            None => {
                self.syslines_by_range_miss += 1;
                debug_eprintln!("{}check_store: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
            }
        }

        // check if there is a Sysline already known at this fileoffset
        // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
        if self.syslines.contains_key(&fileoffset) {
            debug_assert!(self.syslines_by_range.contains_key(&fileoffset), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}) returned None (syslines_by_range out of synch)", fileoffset, fileoffset);
            self.syslines_hit += 1;
            debug_eprintln!("{}check_store: hit self.syslines for FileOffset {}", so(), fileoffset);
            let syslinep = self.syslines[&fileoffset].clone();
            // XXX: multi-byte character encoding
            let fo_next = (*syslinep).fileoffset_end() + (self.charsz() as FileOffset);
            if self.is_sysline_last(&syslinep) {
                debug_eprintln!(
                    "{}check_store: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                    sx(),
                    fo_next,
                    &*syslinep,
                    (*syslinep).fileoffset_begin(),
                    (*syslinep).fileoffset_end(),
                    (*syslinep).to_String_noraw()
                );
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, syslinep.clone())));
                return Some(ResultS4_SyslineFind::Found_EOF((fo_next, syslinep)));
            }
            if self.find_sysline_lru_cache_enabled {
                self.find_sysline_lru_cache_put += 1;
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, syslinep.clone())));
            }
            debug_eprintln!(
                "{}check_store: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] from self.syslines {:?}",
                sx(),
                fo_next,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
            return Some(ResultS4_SyslineFind::Found((fo_next, syslinep)));
        } else {
            self.syslines_miss += 1;
            debug_eprintln!("{}check_store: fileoffset {} not found in self.syslines", so(), fileoffset);
        }
        debug_eprintln!("{}check_store: return None", sx());

        None
    }

    /// Find sysline at fileoffset within the same `Block` (does not cross block boundaries).
    ///
    /// This does a linear search over the `Block`, O(n).
    ///
    /// XXX: similar to `find_sysline`...
    ///      This function `find_sysline_in_block` is large and cumbersome and needs some cleanup of warnings.
    ///      It could definitely use some improvements, but for now it gets the job done.
    ///      You've been warned.
    ///
    pub fn find_sysline_in_block(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline_in_block({})", sn(), fileoffset);

        if let Some(results4) = self.check_store(fileoffset) {
            debug_eprintln!("{}find_sysline_in_block({}): return {:?}", sx(), fileoffset, results4);
            return results4;
        }

        debug_eprintln!("{}find_sysline_in_block({}): searching for first sysline datetime A …", so(), fileoffset);

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sysline = Sysline::new();
        loop {
            debug_eprintln!("{}find_sysline_in_block({}): self.linereader.find_line_in_block({})", so(), fileoffset, fo1);
            let result: ResultS4_LineFind = self.linereader.find_line_in_block(fo1);
            let eof = result.is_eof();
            let (fo2, linep) = match result {
                ResultS4_LineFind::Found((fo_, linep_)) | ResultS4_LineFind::Found_EOF((fo_, linep_)) => {
                    debug_eprintln!(
                        "{}find_sysline_in_block({}): A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        so(),
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    (fo_, linep_)
                }
                ResultS4_LineFind::Done => {
                    debug_eprintln!("{}find_sysline_in_block({}): return ResultS4_SyslineFind::Done; A from LineReader.find_line_in_block({})", sx(), fileoffset, fo1);
                    return ResultS4_SyslineFind::Done;
                }
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("ERROR: LineReader.find_line_in_block({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline_in_block({}): return ResultS4_SyslineFind::Err({}); A from LineReader.find_line_in_block({})", sx(), fileoffset, err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result: Result_ParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz());
            debug_eprintln!("{}find_sysline_in_block({}): A parse_datetime_in_line_cached returned {:?}", so(), fileoffset, result);
            match result {
                Err(_) => {},
                //FindDateTimeData
                Ok((dt_beg, dt_end, dt, _index)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sysline.dt_beg = dt_beg;
                    sysline.dt_end = dt_end;
                    sysline.dt = Some(dt);
                    debug_eprintln!("{}find_sysline_in_block({}): A sl.dt_beg {}, sl.dt_end {}, sl.push({:?})", so(), fileoffset, dt_beg, dt_end, (*linep).to_String_noraw());
                    sysline.push(linep);
                    fo1 = sysline.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}", dt_end, fo1 as usize);
                    if eof {
                        let syslinep = SyslineP::new(sysline);
                        if self.find_sysline_lru_cache_enabled {
                            self.find_sysline_lru_cache_put += 1;
                            debug_eprintln!("{}find_sysline_in_block({}): LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fileoffset, fo1);
                            self.find_sysline_lru_cache
                                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo1, syslinep.clone())));
                        }
                        debug_eprintln!(
                            "{}find_sysline_in_block({}): return ResultS4_SyslineFind::Found_EOF({}, {:p}) @[{}, {}]; A found here and LineReader.find_line({})",
                            sx(),
                            fileoffset,
                            fo1,
                            &(*syslinep),
                            (*syslinep).fileoffset_begin(),
                            (*syslinep).fileoffset_end(),
                            fo1,
                        );
                        return ResultS4_SyslineFind::Found_EOF((fo1, syslinep));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline_in_block({}): A skip push Line {:?}", so(), fileoffset, (*linep).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline_in_block({}): found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            so(),
            fileoffset,
            fo_a,
            fo1,
        );

        //
        // find line with datetime B
        //

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        loop {
            debug_eprintln!("{}find_sysline_in_block({}): self.linereader.find_line_in_block({})", so(), fileoffset, fo1);
            let result = self.linereader.find_line_in_block(fo1);
            let (fo2, linep) = match result {
                ResultS4_LineFind::Found((fo_, linep_)) => {
                    debug_eprintln!(
                        "{}find_sysline_in_block({}): B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    (fo_, linep_)
                },
                ResultS4_LineFind::Found_EOF((fo_, linep_)) => {
                    debug_eprintln!(
                        "{}find_sysline_in_block({}): B got Found_EOF(FileOffset {} Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    eof = true;
                    (fo_, linep_)
                },
                ResultS4_LineFind::Done => {
                    debug_eprintln!("{}find_sysline_in_block({}): B got Done", so(), fileoffset);
                    debug_eprintln!("{}find_sysline_in_block({}): return ResultS4_SyslineFind::Done", sx(), fileoffset);
                    return ResultS4_SyslineFind::Done;
                },
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("ERROR: LineReader.find_line_in_block({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline_in_block({}): return ResultS4_SyslineFind::Err({}); B from LineReader.find_line_in_block({})", sx(), fileoffset, err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                },
            };

            let result: Result_ParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz());
            debug_eprintln!("{}find_sysline_in_block({}): B parse_datetime_in_line_cached returned {:?}", so(), fileoffset, result);
            match result {
                Err(_) => {
                    debug_eprintln!(
                        "{}find_sysline_in_block({}): B append found Line to this Sysline sl.push({:?})",
                        so(),
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    sysline.push(linep);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!(
                        "{}find_sysline_in_block({}): B found datetime; end of this Sysline. Do not append found Line {:?}",
                        so(),
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }
        

        debug_eprintln!("{}find_sysline_in_block({}): found line with datetime B at FileOffset {} {:?}", so(), fileoffset, fo_b, sysline.to_String_noraw());

        let syslinep = self.insert_sysline(sysline);
        // XXX: hack fix, would be better to remove the notion of `Found` and `Found_EOF` (just `Found` or `Done`)
        if eof && !self.is_sysline_last(&syslinep) {
            eof = false;
        }
        if eof {
            if self.find_sysline_lru_cache_enabled {
                self.find_sysline_lru_cache_put += 1;
                debug_eprintln!("{}find_sysline_in_block({}): LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fileoffset, fo_b);
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_b, syslinep.clone())));
            }
            debug_eprintln!(
                "{}find_sysline_in_block({}): return ResultS4_SyslineFind::Found_EOF(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
                sx(),
                fileoffset,
                fo_b,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found_EOF((fo_b, syslinep));
        }
        if self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_put += 1;
            debug_eprintln!("{}find_sysline_in_block({}): LRU cache put({}, Found({}, …))", so(), fileoffset, fileoffset, fo_b);
            self.find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found((fo_b, syslinep.clone())));
        }
        debug_eprintln!(
            "{}find_sysline_in_block({}): return ResultS4_SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            sx(),
            fileoffset,
            fo_b,
            &*syslinep,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).to_String_noraw()
        );

        ResultS4_SyslineFind::Found((fo_b, syslinep))
    }

    /// Find first sysline starting at or after `fileoffset`.
    /// return (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`).
    ///
    /// Similar to `LineReader.find_line`, `BlockReader.read_block`.
    ///
    /// This does a linear search O(n) over the file.
    // XXX: This function `find_sysline` is large and cumbersome and needs some
    //      cleanup of warnings.
    //      It could definitely use improvements but for now it gets the job done.
    //      You've been warned.
    //
    // TODO: separate the caching into wrapper function `find_sysline_cached`
    //
    // TODO: test that retrieving by cache always returns the same ResultS4 enum value as without a cache
    //
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline({})", sn(), fileoffset);

        if let Some(results4) = self.check_store(fileoffset) {
            debug_eprintln!("{}find_sysline({}): return {:?}", sx(), fileoffset, results4);
            return results4;
        }

        debug_eprintln!("{}find_sysline({}): searching for first sysline datetime A …", so(), fileoffset);

        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sysline = Sysline::new();
        loop {
            debug_eprintln!("{}find_sysline({}): self.linereader.find_line({})", so(), fileoffset, fo1);
            let result: ResultS4_LineFind = self.linereader.find_line(fo1);
            let eof = result.is_eof();
            let (fo2, linep) = match result {
                ResultS4_LineFind::Found((fo_, linep_)) | ResultS4_LineFind::Found_EOF((fo_, linep_)) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    (fo_, linep_)
                }
                ResultS4_LineFind::Done => {
                    if self.find_sysline_lru_cache_enabled {
                        self.find_sysline_lru_cache_put += 1;
                        debug_eprintln!("{}find_sysline({}): LRU cache put({}, Done)", so(), fileoffset, fileoffset);
                        self.find_sysline_lru_cache.put(fileoffset, ResultS4_SyslineFind::Done);
                    }
                    debug_eprintln!("{}find_sysline({}): return ResultS4_SyslineFind::Done; A from LineReader.find_line({})", sx(), fileoffset, fo1);
                    return ResultS4_SyslineFind::Done;
                }
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline({}): return ResultS4_SyslineFind::Err({}); A from LineReader.find_line({})", sx(), fileoffset, err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result: Result_ParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz());
            debug_eprintln!("{}find_sysline({}): A parse_datetime_in_line_cached returned {:?}", so(), fileoffset, result);
            match result {
                Err(_) => {
                    // a datetime was not found in the Line!
                }
                Ok((dt_beg, dt_end, dt, _index)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sysline.dt_beg = dt_beg;
                    sysline.dt_end = dt_end;
                    sysline.dt = Some(dt);
                    debug_eprintln!("{}find_sysline({}): A sl.push({:?})", so(), fileoffset, (*linep).to_String_noraw());
                    sysline.push(linep);
                    fo1 = sysline.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}, dt {:?}", dt_beg, dt_end, dt);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}, dt {:?}", dt_end, fo1 as usize, dt);
                    if eof {
                        let syslinep = SyslineP::new(sysline);
                        if self.find_sysline_lru_cache_enabled {
                            self.find_sysline_lru_cache_put += 1;
                            debug_eprintln!("{}find_sysline({}): LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fileoffset, fo1);
                            self.find_sysline_lru_cache
                                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo1, syslinep.clone())));
                        }
                        debug_eprintln!(
                            "{}find_sysline({}): A return ResultS4_SyslineFind::Found_EOF({}, {:p}) @[{}, {}]; A found here and LineReader.find_line({})",
                            sx(),
                            fileoffset,
                            fo1,
                            &(*syslinep),
                            (*syslinep).fileoffset_begin(),
                            (*syslinep).fileoffset_end(),
                            fo1,
                        );
                        return ResultS4_SyslineFind::Found_EOF((fo1, syslinep));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline({}): A skip push Line {:?}", so(), fileoffset, (*linep).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline({}): found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            so(),
            fileoffset,
            fo_a,
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
                debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fo1);
                let syslinep = self.syslines[&fo1].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*syslinep).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
                if self.find_sysline_lru_cache_enabled {
                    self.find_sysline_lru_cache_put += 1;
                    self.find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, syslinep.clone())));
                }
                return ResultS4_SyslineFind::Found((fo_next, syslinep));
            } else {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
            }
            
            // check if the offset is already in a known range
            match self.syslines_by_range.get_key_value(&fo1) {
                Some(range_fo) => {
                    let range = range_fo.0;
                    debug_eprintln!(
                    "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                    so(),
                    fo1,
                    range
                );
                    self.syslines_by_range_hit += 1;
                    let fo = range_fo.1;
                    let syslinep = self.syslines[fo].clone();
                    // XXX: multi-byte character encoding
                    let fo_next = (*syslinep).fileoffset_next() + (self.charsz() as FileOffset);
                    if self.is_sysline_last(&syslinep) {
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                            sx(),
                            fo_next,
                            &*syslinep,
                            (*syslinep).fileoffset_begin(),
                            (*syslinep).fileoffset_end(),
                            (*syslinep).to_String_noraw()
                        );
                        if self.find_sysline_lru_cache_enabled {
                            self.find_sysline_lru_cache_put += 1;
                            self.find_sysline_lru_cache
                                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, syslinep.clone())));
                        }
                        return ResultS4_SyslineFind::Found_EOF((fo_next, syslinep));
                    }
                    if self.find_sysline_lru_cache_enabled {
                        self.find_sysline_lru_cache_put += 1;
                        self.find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, syslinep.clone())));
                    }
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*syslinep,
                        (*syslinep).fileoffset_begin(),
                        (*syslinep).fileoffset_end(),
                        (*syslinep).to_String_noraw()
                    );
                    return ResultS4_SyslineFind::Found((fo_next, syslinep));
                }
                None => {
                    self.syslines_by_range_miss += 1;
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_sysline: searching for first sysline datetime B …", so());
            */
        }

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        debug_eprintln!("{}find_sysline({}): eof {:?}, fo_b {:?}", so(), fileoffset, eof, fo_b);
        loop {
            debug_eprintln!("{}find_sysline({}): self.linereader.find_line({})", so(), fileoffset, fo1);
            let result = self.linereader.find_line(fo1);
            let (fo2, linep) = match result {
                ResultS4_LineFind::Found((fo_, linep_)) => {
                    debug_eprintln!(
                        "{}find_sysline({}): B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found()");
                    (fo_, linep_)
                }
                ResultS4_LineFind::Found_EOF((fo_, linep_)) => {
                    debug_eprintln!(
                        "{}find_sysline({}): B got Found_EOF(FileOffset {} Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fileoffset,
                        fo_,
                        &*linep_,
                        (*linep_).len(),
                        (*linep_).count_lineparts(),
                        (*linep_).to_String_noraw()
                    );
                    //eof = true;
                    (fo_, linep_)
                }
                ResultS4_LineFind::Done => {
                    //debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; B", sx());
                    debug_eprintln!("{}find_sysline({}): eof=true; break; B", so(), fileoffset);
                    eof = true;
                    break;
                }
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline({}): return ResultS4_SyslineFind::Err({}); B from LineReader.find_line({})", sx(), fileoffset, err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let result: Result_ParseDateTime = self.parse_datetime_in_line_cached(&linep, &self.charsz());
            debug_eprintln!("{}find_sysline({}): B parse_datetime_in_line_cached returned {:?}", so(), fileoffset, result);
            match result {
                Err(_) => {
                    // a datetime was not found in the Line! This line is also part of this sysline
                    debug_eprintln!(
                        "{}find_sysline({}): B append found Line to this Sysline sl.push({:?})",
                        so(),
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    sysline.push(linep);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!(
                        "{}find_sysline({}): B found datetime; end of this Sysline. Do not append found Line {:?}",
                        so(),
                        fileoffset,
                        (*linep).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline({}): found line with datetime B at FileOffset {} {:?}", so(), fileoffset, fo_b, sysline.to_String_noraw());

        let syslinep = self.insert_sysline(sysline);
        if eof {
            if self.find_sysline_lru_cache_enabled {
                self.find_sysline_lru_cache_put += 1;
                debug_eprintln!("{}find_sysline({}): LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fileoffset, fo_b);
                self.find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_b, syslinep.clone())));
            }
            debug_eprintln!(
                "{}find_sysline({}): return ResultS4_SyslineFind::Found_EOF(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
                sx(),
                fileoffset,
                fo_b,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).fileoffset_end(),
                (*syslinep).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found_EOF((fo_b, syslinep));
        }
        if self.find_sysline_lru_cache_enabled {
            self.find_sysline_lru_cache_put += 1;
            debug_eprintln!("{}find_sysline({}): LRU cache put({}, Found({}, …))", so(), fileoffset, fileoffset, fo_b);
            self.find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found((fo_b, syslinep.clone())));
        }
        debug_eprintln!(
            "{}find_sysline({}): return ResultS4_SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            sx(),
            fileoffset,
            fo_b,
            &*syslinep,
            (*syslinep).fileoffset_begin(),
            (*syslinep).fileoffset_end(),
            (*syslinep).to_String_noraw()
        );

        ResultS4_SyslineFind::Found((fo_b, syslinep))
    }

    /// Find first sysline at or after `fileoffset` that is at or after `dt_filter`.
    ///
    /// This does a binary search over the file, O(log(n)).
    ///
    /// For example, given syslog file with datetimes:
    ///
    /// ```
    ///     20010101
    ///     20010102
    ///     20010103
    /// ```
    ///
    /// where the newline ending the first line is the ninth byte (fileoffset 9)
    ///
    /// calling
    ///
    ///     `syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))`
    ///
    /// will return
    ///
    ///     `ResultS4::Found(19, SyslineP(data='20010102␊'))`
    ///
    /// XXX: this function is large, cumbersome, and messy. Changes require extensive retesting.
    //
    // TODO: rename this to `find_next_sysline_at_datetime_filter`, rename all `find_` functions to either
    //       `find_..._between_`, `find_...at_`, or `find_next`
    //       `between` and `at` mean binary search over the file, `next` means linear sequantial search
    pub fn find_sysline_at_datetime_filter(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        const _fname: &str = "find_sysline_at_datetime_filter";
        debug_eprintln!("{}{}(SyslineReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter);
        let filesz = self.filesz();
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
            debug_eprintln!("{}{}: loop(…)!", so(), _fname);
            let result = self.find_sysline(try_fo);
            let eof = result.is_eof();
            let done = result.is_done();
            match result {
                ResultS4_SyslineFind::Found((fo, syslinep)) | ResultS4_SyslineFind::Found_EOF((fo, syslinep)) => {
                    if !eof {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found({}, …) A",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    } else {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(try_fo: {}) returned ResultS4_SyslineFind::Found_EOF({}, …) B",
                            so(),
                            _fname,
                            try_fo,
                            fo
                        );
                    }
                    debug_eprintln!(
                        "{}{}: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?} C",
                        so(),
                        _fname,
                        fo,
                        &(*syslinep),
                        syslinep.lines.len(),
                        (*syslinep).len(),
                        (*syslinep).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    debug_eprintln!(
                        "{}{}: sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        so(),
                        _fname,
                        &*syslinep,
                        (*syslinep).dt,
                        dt_filter,
                    );
                    match SyslineReader::sysline_dt_after_or_before(&syslinep, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            debug_eprintln!(
                                "{}{}: Pass => fo {} fo_last {} try_fo {} try_fo_last {} (fo_end {})",
                                so(),
                                _fname,
                                fo,
                                fo_last,
                                try_fo,
                                try_fo_last,
                                _fo_end,
                            );
                            debug_eprintln!(
                                "{}{}: return ResultS4_SyslineFind::{}(({}, @{:p})); A",
                                sx(),
                                _fname,
                                if eof { "Found_EOF" } else { "Found" },
                                fo,
                                &*syslinep,
                            );
                            if !eof {
                                return ResultS4_SyslineFind::Found((fo, syslinep));
                            } else {
                                return ResultS4_SyslineFind::Found_EOF((fo, syslinep));
                            }
                        } // end Pass
                        Result_Filter_DateTime1::OccursAtOrAfter => {
                            // the Sysline found by `find_sysline(try_fo)` occurs at or after filter `dt_filter`, so search backward
                            // i.e. move end marker `fo_b` backward
                            debug_eprintln!("{}{}: OccursAtOrAfter => fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            // short-circuit a common case, passed fileoffset is past the `dt_filter`, can immediately return
                            // XXX: does this mean my algorithm sucks?
                            if try_fo == fileoffset {
                                // first loop iteration
                                debug_eprintln!(
                                    "{}{}:                    try_fo {} == {} try_fo_last; early return",
                                    so(),
                                    _fname,
                                    try_fo,
                                    try_fo_last,
                                );
                                debug_eprintln!(
                                    "{}{}: return ResultS4_SyslineFind::{}(({}, @{:p})); B fileoffset {} {:?}",
                                    sx(),
                                    _fname,
                                    if eof { "Found_EOF" } else { "Found" },
                                    fo,
                                    &*syslinep,
                                    (*syslinep).fileoffset_begin(),
                                    (*syslinep).to_String_noraw(),
                                );
                                if !eof {
                                    return ResultS4_SyslineFind::Found((fo, syslinep));    
                                } else {
                                    return ResultS4_SyslineFind::Found_EOF((fo, syslinep));
                                }    
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*syslinep).fileoffset_begin(), try_fo_last);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
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
                            debug_eprintln!("{}{}: OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            let syslinep_foe = (*syslinep).fileoffset_end();
                            // XXX: [2022/03/25] why was this `assert_le` here? It seems wrong.
                            //assert_le!(syslinep_foe, fo, "unexpected values (SyslineP@{:p}).fileoffset_end() {}, fileoffset returned by self.find_sysline({}) was {} FPath {:?}", syslinep, syslinep_foe, try_fo, fo, self.path());
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, syslinep_foe, "Unexpected values try_fo_last {} syslinep_foe {}, last tried offset (passed to self.find_sysline({})) is beyond returned Sysline@{:p}.fileoffset_end() {}!? FPath {:?}", try_fo_last, syslinep_foe, try_fo, syslinep, syslinep_foe, self.path());
                            debug_eprintln!(
                                "{}{}:                    ∴ fo_a = min(syslinep_foe {}, fo_b {});",
                                so(),
                                _fname,
                                syslinep_foe,
                                fo_b,
                            );
                            // LAST WORKING HERE [2021/10/06 00:05:00]
                            // LAST WORKING HERE [2022/03/16 01:15:00]
                            // this code passes all tests, but runs strangely. I think the problem is the first found sysline
                            // (that may or may not satisfy the passed filter) is placed into a queue and then printed by the waiting main thread.
                            fo_a = std::cmp::min(syslinep_foe, fo_b);
                            //fo_a = std::cmp::max(syslinep_foe, fo_b);
                            //fo_a = syslinep_foe;
                            //assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a,
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    debug_eprintln!("{}{}:                    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                    fo_last = fo;
                    syslinep_opt = Some(syslinep);
                    // TODO: [2021/09/26]
                    //       I think could do an early check and skip a few loops:
                    //       if `fo_a` and `fo_b` are offsets into the same Sysline
                    //       then that Sysline is the candidate, so return Ok(...)
                    //       unless `fo_a` and `fo_b` are past last Sysline.fileoffset_begin of the file then return Done
                } // end Found | Found_EOF
                ResultS4_SyslineFind::Done => {
                    debug_eprintln!("{}{}: SyslineReader.find_sysline(try_fo: {}) returned Done", so(), _fname, try_fo);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                    try_fo_last = try_fo;
                    debug_eprintln!(
                        "{}{}:                 ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                        so(),
                        _fname,
                        fo_a,
                        fo_b,
                        fo_a
                    );
                    try_fo = fo_a + ((fo_b - fo_a) / 2);
                    debug_eprintln!(
                        "{}{}:                 try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})",
                        so(),
                        _fname,
                        try_fo,
                        try_fo_last,
                        fo_a,
                        fo_b,
                        _fo_end
                    );
                } // end Done
                ResultS4_SyslineFind::Err(_err) => {
                    debug_eprintln!(
                        "{}{}: SyslineReader.find_sysline(try_fo: {}) returned Err({})",
                        so(),
                        _fname,
                        try_fo,
                        _err,
                    );
                    debug_eprintln!("ERROR: {}", _err);
                    break;
                } // end Err
            } // match result
            debug_eprintln!("{}{}: next loop will try offset {} (fo_end {})", so(), _fname, try_fo, _fo_end);

            // TODO: 2022/03/18 this latter part hints at a check that could be done sooner,
            //       before `try_fo==try_fo_last`, that would result in a bit less loops.
            //       A simpler and faster check is to do
            //           fo_next, syslinep = find_sysline(fileoffset)
            //           _, syslinep_next = find_sysline(fo_next)
            //       do this at the top of the loop. Then call `dt_after_or_before` for each
            //       `.dt` among `syslinep`, `syslinep_next`.

            // `try_fo == try_fo_last` means binary search loop is deciding on the same fileoffset upon each loop.
            // the searching is exhausted.
            if done && try_fo == try_fo_last {
                // reached a dead-end of searching the same fileoffset `find_sysline(try_fo)` and receiving Done
                // so this function is exhausted too.
                debug_eprintln!("{}{}: Done && try_fo {} == {} try_fo_last; break!", so(), _fname, try_fo, try_fo_last);
                break;
            } else if try_fo != try_fo_last {
                continue;
            }
            debug_eprintln!("{}{}: try_fo {} == {} try_fo_last;", so(), _fname, try_fo, try_fo_last);
            let mut syslinep = syslinep_opt.unwrap();
            let fo_beg = syslinep.fileoffset_begin();
            if self.is_sysline_last(&syslinep) && fo_beg < try_fo {
                // binary search stopped at fileoffset past start of last Sysline in file
                // so entirely past all acceptable syslines
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; C binary searched ended after beginning of last sysline in the file", sx(), _fname,);
                return ResultS4_SyslineFind::Done;
            }
            // binary search loop is deciding on the same fileoffset upon each loop. That fileoffset must refer to
            // an acceptable sysline. However, if that fileoffset is past `syslinep.fileoffset_begin` than the threshold
            // change of datetime for the `dt_filter` is the *next* Sysline.
            let fo_next = syslinep.fileoffset_next();
            // XXX: sanity check
            //debug_assert_eq!(fo_last, fo_next, "fo {} != {} syslinep.fileoffset_next()", fo_last, fo_next);
            if fo_beg < try_fo {
                debug_eprintln!("{}{}: syslinep.fileoffset_begin() {} < {} try_fo;", so(), _fname, fo_beg, try_fo);
                let syslinep_next = match self.find_sysline(fo_next) {
                    ResultS4_SyslineFind::Found_EOF((_, syslinep_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found_EOF(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            syslinep_
                        );
                        syslinep_
                    }
                    ResultS4_SyslineFind::Found((_, syslinep_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            syslinep_
                        );
                        syslinep_
                    }
                    ResultS4_SyslineFind::Done => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) unexpectedly returned Done",
                            so(),
                            _fname,
                            fo_next
                        );
                        break;
                    }
                    ResultS4_SyslineFind::Err(_err) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Err({})",
                            so(),
                            _fname,
                            fo_next,
                            _err,
                        );
                        debug_eprintln!("ERROR: {}", _err);
                        break;
                    }
                };
                debug_eprintln!("{}{}: dt_filter:                   {:?}", so(), _fname, dt_filter);
                debug_eprintln!(
                    "{}{}: syslinep      : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    fo_beg,
                    (*syslinep).fileoffset_end(),
                    (*syslinep).dt.unwrap(),
                    (*syslinep).to_String_noraw()
                );
                debug_eprintln!(
                    "{}{}: syslinep_next : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    (*syslinep_next).fileoffset_begin(),
                    (*syslinep_next).fileoffset_end(),
                    (*syslinep_next).dt.unwrap(),
                    (*syslinep_next).to_String_noraw()
                );
                let syslinep_compare = dt_after_or_before(&(*syslinep).dt.unwrap(), dt_filter);
                let syslinep_next_compare = dt_after_or_before(&(*syslinep_next).dt.unwrap(), dt_filter);
                debug_eprintln!("{}{}: match({:?}, {:?})", so(), _fname, syslinep_compare, syslinep_next_compare);
                syslinep = match (syslinep_compare, syslinep_next_compare) {
                    (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                        debug_eprintln!("{}{}: unexpected Result_Filter_DateTime1::Pass", so(), _fname);
                        eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                        break;
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                        debug_eprintln!("{}{}: choosing syslinep_next", so(), _fname);
                        syslinep_next
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing syslinep_next", so(), _fname);
                        syslinep_next
                    }
                    (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing syslinep", so(), _fname);
                        syslinep
                    }
                    _ => {
                        debug_eprintln!(
                            "{}{}: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple",
                            so(),
                            _fname
                        );
                        eprintln!("ERROR: unhandled (Result_Filter_DateTime1, Result_Filter_DateTime1) tuple");
                        break;
                    }
                };
            } else {
                debug_eprintln!(
                    "{}{}: syslinep.fileoffset_begin() {} >= {} try_fo; use syslinep",
                    so(),
                    _fname,
                    fo_beg,
                    try_fo
                );
            }
            let fo_ = syslinep.fileoffset_next();
            debug_eprintln!(
                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}",
                sx(),
                _fname,
                fo_,
                &*syslinep,
                (*syslinep).fileoffset_begin(),
                (*syslinep).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found((fo_, syslinep));
        } // end loop

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; E", sx(), _fname);

        ResultS4_SyslineFind::Done
    }

    /// convenience wrapper for `dt_after_or_before`
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        debug_eprintln!("{}sysline_dt_after_or_before(SyslineP@{:p}, {:?})", snx(), &*syslinep, dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline@{:p} does not have a datetime set.", &*syslinep);

        let dt: &DateTimeL = (*syslinep).dt.as_ref().unwrap();

        dt_after_or_before(dt, dt_filter)
    }

    /// wrapper for call to `dt_pass_filters`
    pub fn sysline_pass_filters(
        syslinep: &SyslineP, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> Result_Filter_DateTime2 {
        debug_eprintln!(
            "{}sysline_pass_filters(SyslineP@{:p}, {:?}, {:?})",
            sn(),
            &*syslinep,
            dt_filter_after,
            dt_filter_before,
        );
        assert!((*syslinep).dt.is_some(), "Sysline @{:p} does not have a datetime set.", &*syslinep);
        let dt: &DateTimeL = (*syslinep).dt.as_ref().unwrap();
        let result = dt_pass_filters(dt, dt_filter_after, dt_filter_before);
        debug_eprintln!("{}sysline_pass_filters(…) return {:?};", sx(), result);

        result
    }

    /// find the first `Sysline`, starting at `fileoffset`, that is at or after datetime filter
    /// `dt_filter_after` and before datetime filter `dt_filter_before`.
    ///
    /// This uses `self.find_sysline_at_datetime_filter`
    ///
    pub fn find_sysline_between_datetime_filters(
        &mut self, fileoffset: FileOffset, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        const _fname: &str = "find_sysline_between_datetime_filters";
        debug_eprintln!("{}{}({}, {:?}, {:?})", sn(), _fname, fileoffset, dt_filter_after, dt_filter_before);

        match self.find_sysline_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS4_SyslineFind::Found((fo, syslinep)) => {
                debug_eprintln!(
                "{}{}: find_sysline_at_datetime_filter returned ResultS4_SyslineFind::Found(({}, {:?})); call sysline_pass_filters",
                    so(),
                    _fname,
                    fo,
                    syslinep,
                );
                match Self::sysline_pass_filters(&syslinep, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::InRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned InRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found(({}, {:?}))", sx(), _fname, fo, syslinep);
                        return ResultS4_SyslineFind::Found((fo, syslinep));
                    },
                    Result_Filter_DateTime2::BeforeRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned BeforeRange;", so(), _fname);
                        eprintln!("ERROR: sysline_pass_filters(Sysline@{:p}, {:?}, {:?}) returned BeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  syslinep, dt_filter_after, dt_filter_before,
                                  fileoffset, dt_filter_after
                        );
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done (not sure what to do here)", sx(), _fname);
                        return ResultS4_SyslineFind::Done; 
                    },
                    Result_Filter_DateTime2::AfterRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned AfterRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done", sx(), _fname);
                        return ResultS4_SyslineFind::Done;
                    },
                };
            },
            ResultS4_SyslineFind::Found_EOF((fo, syslinep)) => {
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found_EOF(({}, {:?}))", sx(), _fname, fo, syslinep);
                return ResultS4_SyslineFind::Found_EOF((fo, syslinep));
            },
            ResultS4_SyslineFind::Done => {},
            ResultS4_SyslineFind::Err(err) => {
                debug_eprintln!(
                    "{}{}: find_sysline_at_datetime_filter({}, dt_after: {:?}) returned Err({})",
                    so(),
                    _fname,
                    fileoffset,
                    dt_filter_after,
                    err,
                );
                debug_eprintln!("ERROR: {}", err);
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Err({})", sx(), _fname, err);
                return ResultS4_SyslineFind::Err(err);
            },
        };

        debug_eprintln!("{}{} return ResultS4_SyslineFind::Done", sx(), _fname);

        ResultS4_SyslineFind::Done
    }
}
