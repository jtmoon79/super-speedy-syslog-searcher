// src/readers/linereader.rs

//! Implements a [`LineReader`],
//! the driver of deriving [`Line`s] using a [`BlockReader`].
//!
//! [`Line`s]: crate::data::line::Line
//! [`BlockReader`]: crate::readers::blockreader::BlockReader

use crate::common::{CharSz, Count, FPath, FileOffset, FileSz, FileType, NLu8, ResultS3};
use crate::data::line::{Line, LineP, LinePart, Lines};
use crate::data::datetime::SystemTime;
use crate::readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockP,
    BlockReader,
    BlockSz,
    ResultS3ReadBlock,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::byte_to_char_noraw;

use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::HashSet;
use std::fmt;
use std::io::{Error, Result};
use std::sync::Arc;

use ::lru::LruCache;
use ::more_asserts::debug_assert_ge;
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, def1n, def1ñ, def1x, den, deo, dex, deñ};


// ----------
// LineReader

/// Map [`FileOffset`] To [`Line`].
///
/// Storage for Lines found from the underlying `BlockReader`
/// FileOffset key is the first byte/offset that begins the `Line`
///
/// [`Line`]: crate::data::line::Line
pub type FoToLine = BTreeMap<FileOffset, LineP>;

/// Map [`FileOffset`] To `FileOffset`
pub type FoToFo = BTreeMap<FileOffset, FileOffset>;

/// [`LineReader.find_line()`] searching results.
///
/// [`LineReader.find_line()`]: self::LineReader#method.find_line
// TODO: rename `ResultS3LineFind` with `ResultLineFind`
pub type ResultS3LineFind = ResultS3<(FileOffset, LineP), Error>;

/// Internal LRU cache for [`LineReader.find_line()`].
///
/// [`LineReader.find_line()`]: self::LineReader#method.find_line
pub type LinesLRUCache = LruCache<FileOffset, ResultS3LineFind>;

#[cfg(test)]
pub type SetDroppedLines = HashSet<FileOffset>;

/// A specialized reader that uses [`BlockReader`] to find [`Lines`] in a file.
/// A `LineReader` knows how to process sequences of bytes of data among
/// different `Block`s to create a `Line`.
///
/// The `LineReader` does most of the `[u8]` to `char` interpretation for
/// text log files. [`SyslineReader`] also does a little.
///
/// A `LineReader` stores past lookups of data in `self.lines`.
///
/// _XXX: not a rust "Reader"; does not implement trait [`Read`]._
///
/// [`BlockReader`]: crate::readers::blockreader::BlockReader
/// [`Lines`]: crate::data::line::Line
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
/// [`Read`]: std::io::Read
pub struct LineReader {
    pub(crate) blockreader: BlockReader,
    /// Track [`Line`] found among blocks in `blockreader`, tracked by line
    /// beginning `FileOffset` key value `FileOffset` should agree
    /// with `(*LineP).fileoffset_begin()`.
    ///
    /// [`Line`]: crate::data::line::Line
    pub(crate) lines: FoToLine,
    /// Internal stats - "high watermark" of Lines stored in `self.lines`
    lines_stored_highest: usize,
    /// Internal stats - hits of `self.lines` in `find_line()`
    /// and other functions.
    pub(super) lines_hits: Count,
    /// Internal stats - misses of `self.lines` in `find_line()`
    /// and other functions.
    pub(super) lines_miss: Count,
    /// For all `Lines`, map `Line.fileoffset_end` to `Line.fileoffset_beg`.
    foend_to_fobeg: FoToFo,
    /// `Count` of `Line`s processed.
    ///
    /// Distinct from `self.lines.len()` as that may have contents removed
    /// during "[streaming stage]".
    ///
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    pub(super) lines_processed: Count,
    /// Smallest size character in bytes.
    // XXX: Issue #16 only handles UTF-8/ASCII encoding
    charsz_: CharSz,
    /// Enable internal [LRU cache] for `find_line` (default `true`).
    ///
    /// [LRU cache]: https://docs.rs/lru/0.7.8/lru/index.html
    find_line_lru_cache_enabled: bool,
    /// Internal LRU cache for function [`find_line`].
    ///
    /// [`find_line`]: self::LineReader#method.find_line
    pub(super) find_line_lru_cache: LinesLRUCache,
    /// Internal LRU cache `Count` of lookup hit.
    pub(super) find_line_lru_cache_hit: Count,
    /// Internal LRU cache `Count` of lookup miss.
    pub(super) find_line_lru_cache_miss: Count,
    /// Internal LRU cache `Count` of `.put`.
    pub(super) find_line_lru_cache_put: Count,
    /// Count of Ok to Arc::try_unwrap(linep), effectively `Count` of
    /// dropped `Line`.
    pub(super) drop_line_ok: Count,
    /// `Count` of failures to Arc::try_unwrap(linep).
    /// A failure does not mean an error.
    pub(super) drop_line_errors: Count,
    /// testing-only tracker of successfully dropped `Line`
    #[cfg(test)]
    pub(crate) dropped_lines: SetDroppedLines,
}

impl fmt::Debug for LineReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("LineReader")
            .field("LRU cache enabled?", &self.find_line_lru_cache_enabled)
            .field("charsz", &self.charsz())
            .field("lines", &self.lines)
            .field("blockreader", &self.blockreader)
            .finish()
    }
}

// TODO: [2023/04] remove redundant variable prefix name `linereader_`
// TODO: [2023/05] instead of having 1:1 manual copying of `LineReader`
//       fields to `SummaryLineReader` fields, just store a
//       `SummaryLineReader` in `LineReader` and update directly.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SummaryLineReader {
    /// `Count` of `Lines` processed by `LineReader`
    pub linereader_lines: Count,
    /// "high watermark" of Lines stored in `LineReader.lines`
    pub linereader_lines_stored_highest: usize,
    /// `LineReader::lines_hits` for `self.lines`
    pub linereader_lines_hits: Count,
    /// `LineReader::lines_miss` for `self.lines`
    pub linereader_lines_miss: Count,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub linereader_find_line_lru_cache_hit: Count,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub linereader_find_line_lru_cache_miss: Count,
    /// `LineReader::find_line()` `self._find_line_lru_cache`
    pub linereader_find_line_lru_cache_put: Count,
    /// `LineReader::drop_line_ok`
    pub linereader_drop_line_ok: Count,
    /// `LineReader::drop_line_errors`
    pub linereader_drop_line_errors: Count,
}

// XXX: cannot place these within `impl LineReader`?

/// Minimum char storage size in bytes.
const CHARSZ_MIN: CharSz = 1;
/// Maximum char storage size in bytes.
const CHARSZ_MAX: CharSz = 4;
/// Default char storage size in bytes.
// XXX: Issue #16 only handles UTF-8/ASCII encoding
const CHARSZ: CharSz = CHARSZ_MIN;

/// Implement the LineReader.
impl LineReader {
    /// Internal LRU cache size (entries).
    const FIND_LINE_LRU_CACHE_SZ: usize = 8;

    /// Default state of LRU cache.
    const CACHE_ENABLE_DEFAULT: bool = true;

    // `LineReader::blockzero_analysis` must find at least this many `Line` within
    // block zero (first block) for the file to be considered a text file.
    // If the file has only one block then different considerations apply.

    /// Create a new `LineReader`.
    ///
    /// **NOTE:** should not attempt any block reads here,
    /// similar to other `*Readers::new()`
    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz: BlockSz,
    ) -> Result<LineReader> {
        def1n!("({:?}, {:?}, {:?})", path, filetype, blocksz);
        // XXX: Issue #16 only handles UTF-8/ASCII encoding
        debug_assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        debug_assert!(blocksz != 0, "BlockSz is zero");
        let blockreader = match BlockReader::new(path, filetype, blocksz) {
            Ok(br) => br,
            Err(err) => {
                def1x!();
                return Err(err);
            }
        };
        def1x!("return Ok(LineReader)");

        Ok(LineReader {
            blockreader,
            lines: FoToLine::new(),
            lines_stored_highest: 0,
            lines_hits: 0,
            lines_miss: 0,
            foend_to_fobeg: FoToFo::new(),
            lines_processed: 0,
            charsz_: CHARSZ,
            find_line_lru_cache_enabled: LineReader::CACHE_ENABLE_DEFAULT,
            find_line_lru_cache: LinesLRUCache::new(
                std::num::NonZeroUsize::new(LineReader::FIND_LINE_LRU_CACHE_SZ).unwrap(),
            ),
            find_line_lru_cache_hit: 0,
            find_line_lru_cache_miss: 0,
            find_line_lru_cache_put: 0,
            drop_line_ok: 0,
            drop_line_errors: 0,
            #[cfg(test)]
            dropped_lines: SetDroppedLines::new(),
        })
    }

    /// Smallest size character in bytes.
    #[inline(always)]
    pub const fn charsz(&self) -> usize {
        self.charsz_
    }

    /// See [`BlockReader::blocksz`].
    ///
    /// [`BlockReader::blocksz`]: crate::readers::blockreader::BlockReader#method.blocksz
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz()
    }

    /// See [`BlockReader::filesz`].
    ///
    /// [`BlockReader::filesz`]: crate::readers::blockreader::BlockReader#method.filesz
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.blockreader.filesz()
    }

    /// See [`BlockReader::filetype`].
    ///
    /// [`BlockReader::filetype`]: crate::readers::blockreader::BlockReader#method.filetype
    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.blockreader.filetype()
    }

    /// See [`BlockReader::path`].
    ///
    /// [`BlockReader::path`]: crate::readers::blockreader::BlockReader#method.path
    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        self.blockreader.path()
    }

    /// Enable internal LRU cache used by `find_line`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_enable(&mut self) {
        if self.find_line_lru_cache_enabled {
            return;
        }
        self.find_line_lru_cache_enabled = true;
        self.find_line_lru_cache
            .clear();
        self.find_line_lru_cache
            .resize(std::num::NonZeroUsize::new(LineReader::FIND_LINE_LRU_CACHE_SZ).unwrap());
    }

    /// Disable internal LRU cache used by `find_line`.
    #[allow(non_snake_case)]
    pub fn LRU_cache_disable(&mut self) {
        self.find_line_lru_cache_enabled = false;
        self.find_line_lru_cache
            .clear();
    }

    /// See [`BlockReader::mtime`].
    ///
    /// [`BlockReader::mtime`]: crate::readers::blockreader::BlockReader#method.mtime
    pub fn mtime(&self) -> SystemTime {
        self.blockreader.mtime()
    }

    /// `Count` of `Line`s processed by this `LineReader`
    /// (i.e. `self.lines_processed`).
    #[inline(always)]
    pub const fn count_lines_processed(&self) -> Count {
        self.lines_processed
    }

    /// "high watermark" of Lines stored in `self.lines`
    #[inline(always)]
    pub const fn lines_stored_highest(&self) -> usize {
        self.lines_stored_highest
    }

    /// See [`BlockReader::block_offset_at_file_offset`].
    ///
    /// [`BlockReader::block_offset_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_offset_at_file_offset
    #[inline(always)]
    pub const fn block_offset_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// See [`BlockReader::file_offset_at_block_offset`].
    ///
    /// [`BlockReader::file_offset_at_block_offset`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset
    #[inline(always)]
    pub const fn file_offset_at_block_offset(
        &self,
        blockoffset: BlockOffset,
    ) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// See [`BlockReader::file_offset_at_block_offset_index`].
    ///
    /// [`BlockReader::file_offset_at_block_offset_index`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset_index
    #[inline(always)]
    pub const fn file_offset_at_block_offset_index(
        &self,
        blockoffset: BlockOffset,
        blockindex: BlockIndex,
    ) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// See [`BlockReader::block_index_at_file_offset`].
    ///
    /// [`BlockReader::block_index_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_index_at_file_offset
    #[inline(always)]
    pub const fn block_index_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// See [`BlockReader::count_blocks`].
    ///
    /// [`BlockReader::count_blocks`]: crate::readers::blockreader::BlockReader#method.count_blocks
    #[inline(always)]
    pub const fn count_blocks(&self) -> Count {
        BlockReader::count_blocks(self.filesz(), self.blocksz()) as Count
    }

    /// See [`BlockReader::blockoffset_last`].
    ///
    /// [`BlockReader::blockoffset_last`]: crate::readers::blockreader::BlockReader#method.blockoffset_last
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader
            .blockoffset_last()
    }

    /// See [`BlockReader::fileoffset_last`].
    ///
    /// [`BlockReader::fileoffset_last`]: crate::readers::blockreader::BlockReader#method.fileoffset_last
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.blockreader
            .fileoffset_last()
    }

    /// Is the passed `FileOffset` the last byte of the file?
    pub const fn is_fileoffset_last(
        &self,
        fileoffset: FileOffset,
    ) -> bool {
        self.fileoffset_last() == fileoffset
    }

    /// Is the passed `LineP` the last `Line` of the file?
    pub fn is_line_last(
        &self,
        linep: &LineP,
    ) -> bool {
        self.is_fileoffset_last((*linep).fileoffset_end())
    }

    /// Return all currently stored `FileOffset` in `self.lines`.
    ///
    /// Only intended to aid testing.
    #[cfg(test)]
    pub fn get_fileoffsets(&self) -> Vec<FileOffset> {
        self.lines
            .keys()
            .cloned()
            .collect()
    }

    /// Store information about a single line in a file.
    /// Returns a `Line` pointer `LineP`.
    ///
    /// Should only be called by `self.find_line` and `self.find_line_in_block`.
    fn insert_line(
        &mut self,
        line: Line,
    ) -> LineP {
        defn!("Line @[{}‥{}]", line.fileoffset_begin(), line.fileoffset_end());
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end: FileOffset = line.fileoffset_end();
        let linep: LineP = LineP::new(line);
        deo!("lines.insert({})", fo_beg);
        debug_assert!(
            !self
                .lines
                .contains_key(&fo_beg),
            "self.lines already contains key {}",
            fo_beg
        );
        self.lines
            .insert(fo_beg, linep.clone());
        deo!("foend_to_fobeg.insert({}, {})", fo_end, fo_beg);
        self.lines_stored_highest = std::cmp::max(self.lines_stored_highest, self.lines.len());
        debug_assert!(
            !self
                .foend_to_fobeg
                .contains_key(&fo_end),
            "self.foend_to_fobeg already contains key {}",
            fo_end
        );
        self.foend_to_fobeg
            .insert(fo_end, fo_beg);
        self.lines_processed += 1;
        defx!("returning LineP");

        linep
    }

    /// See [`BlockReader::is_streamed_file`].
    ///
    /// [`BlockReader::is_streamed_file`]: crate::readers::blockreader::BlockReader#method.is_streamed_file
    pub const fn is_streamed_file(&self) -> bool {
        self.blockreader.is_streamed_file()
    }

    /// Return `drop_data` value.
    pub const fn is_drop_data(&self) -> bool {
        self.blockreader.is_drop_data()
    }

    /// Proactively `drop` the [`Lines`]. For "[streaming stage]".
    /// This calls [`LineReader::drop_line`].
    ///
    /// _The caller must know what they are doing!_
    ///
    /// [`Lines`]: crate::data::line::Lines
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    pub fn drop_lines(
        &mut self,
        lines: Lines,
    ) -> bool {
        defn!();
        if ! self.is_drop_data() {
            defx!("is_drop_data is false");
            return false;
        }
        let mut ret = false;
        for linep in lines.into_iter() {
            if self.drop_line(linep) {
                ret = true;
            }
        }
        defx!("return {}", ret);

        ret
    }

    /// Proactively `drop` the [`Line`]. For "[streaming stage]".
    /// This calls [`BlockReader::drop_block`].
    ///
    /// _The caller must know what they are doing!_
    ///
    /// [`Line`s]: crate::data::line::Line
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    pub fn drop_line(
        &mut self,
        linep: LineP,
    ) -> bool {
        defn!("Line @[{}‥{}]", (*linep).fileoffset_begin(), (*linep).fileoffset_end());
        let mut ret = false;
        let fo_key: FileOffset = (*linep).fileoffset_begin();
        self.find_line_lru_cache
            .pop(&fo_key);
        self.lines.remove(&fo_key);
        match Arc::try_unwrap(linep) {
            Ok(line) => {
                defo!(
                    "Arc::try_unwrap(linep) dropped Line, Block @[{}‥{}]",
                    line.blockoffset_first(),
                    line.blockoffset_last()
                );
                self.drop_line_ok += 1;
                #[cfg(test)]
                {
                    self.dropped_lines
                        .insert(line.fileoffset_begin());
                }
                // drop blocks referenced by lineparts except the last linepart
                let take_ = match line.lineparts.len() {
                    0 => 0,
                    val => val - 1,
                };
                for linepart in line
                    .lineparts
                    .into_iter()
                    .take(take_)
                {
                    let bo = linepart.blockoffset();
                    drop(linepart);
                    if self
                        .blockreader
                        .drop_block(bo)
                    {
                        ret = true;
                    }
                }
            }
            Err(_linep) => {
                defo!(
                    "Arc::try_unwrap(linep) failed to drop Line, strong_count {}",
                    Arc::strong_count(&_linep)
                );
                self.drop_line_errors += 1;
            }
        }
        defx!("return {}", ret);

        ret
    }

    /// Does `self` "contain" this `fileoffset`? That is, already know about it?
    ///
    /// The `FileOffset` can be any value (does not have to be beginning
    /// or ending of a `Line`).
    fn lines_contains(
        &self,
        fileoffset: &FileOffset,
    ) -> bool {
        let fo_beg: &FileOffset = match self
            .foend_to_fobeg
            .range(fileoffset..)
            .next()
        {
            Some((_, fo_beg_)) => fo_beg_,
            None => {
                return false;
            }
        };
        if fileoffset < fo_beg {
            return false;
        }
        self.lines
            .contains_key(fo_beg)
    }

    /// For any `FileOffset`, get the `Line` (if available).
    ///
    /// The passed `FileOffset` can be any value (does not have to be
    /// beginning or ending of a `Line`).
    ///
    /// _O(log(n))_
    pub fn get_linep(
        &self,
        fileoffset: &FileOffset,
    ) -> Option<LineP> {
        // I'm somewhat sure this is O(log(n))
        let fo_beg: &FileOffset = match self
            .foend_to_fobeg
            .range(fileoffset..)
            .next()
        {
            Some((_, fo_beg_)) => fo_beg_,
            None => {
                return None;
            }
        };
        if fileoffset < fo_beg {
            return None;
        }
        #[allow(clippy::manual_map)]
        match self.lines.get(fo_beg) {
            Some(lp) => Some(lp.clone()),
            None => None,
        }
    }

    /// Check the internal LRU cache if this `FileOffset` has a known return
    /// value for `find_line`.
    #[inline(always)]
    #[allow(non_snake_case)]
    fn check_store_LRU(
        &mut self,
        fileoffset: FileOffset,
    ) -> Option<ResultS3LineFind> {
        // check LRU cache first (this is very fast)
        if !self.find_line_lru_cache_enabled {
            return None;
        }
        match self
            .find_line_lru_cache
            .get(&fileoffset)
        {
            Some(rlp) => {
                defn!("({}): found LRU cached for offset {}", fileoffset, fileoffset);
                self.find_line_lru_cache_hit += 1;
                // `find_line_lru_cache.get(&fileoffset)` returns reference so must create new `ResultS3LineFind` here
                // and return that
                match rlp {
                    ResultS3LineFind::Found(val) => {
                        defx!(
                            "return ResultS3LineFind::Found(({}, …)) @[{}, {}] {:?}",
                            val.0,
                            val.1.fileoffset_begin(),
                            val.1.fileoffset_end(),
                            val.1.to_String_noraw()
                        );
                        return Some(ResultS3LineFind::Found((val.0, val.1.clone())));
                    }
                    ResultS3LineFind::Done => {
                        defx!("return ResultS3LineFind::Done");
                        return Some(ResultS3LineFind::Done);
                    }
                    ResultS3LineFind::Err(_err) => {
                        defx!("Err {}", _err);
                        eprintln!(
                            "ERROR: unexpected Error store in find_line_lru_cache, fileoffset {}, file {:?}",
                            fileoffset, self.path(),
                        );
                    }
                }
            }
            None => {
                self.find_line_lru_cache_miss += 1;
                defñ!("fileoffset {} not found in LRU cache", fileoffset);
            }
        }

        None
    }

    /// Check the internal storage if this `FileOffset` has a known return
    /// value for `find_line`.
    #[inline(always)]
    fn check_store(
        &mut self,
        fileoffset: FileOffset,
    ) -> Option<ResultS3LineFind> {
        // TODO: [2022/06/18] add a counter for hits and misses for `self.lines`
        let charsz_fo: FileOffset = self.charsz_ as FileOffset;
        // search containers of `Line`s
        // first, check if there is a `Line` already known at this fileoffset
        if self
            .lines
            .contains_key(&fileoffset)
        {
            self.lines_hits += 1;
            deo!("hit self.lines for FileOffset {}", fileoffset);
            debug_assert!(
                self.lines_contains(&fileoffset),
                "self.lines and self.lines_by_range are out of synch on key {} (before part A)",
                fileoffset
            );
            let linep: LineP = self.lines[&fileoffset].clone();
            let fo_next: FileOffset = (*linep).fileoffset_end() + charsz_fo;
            if self.is_line_last(&linep) {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    deo!(
                        "LRU Cache put({}, Found({}, …)) {:?}",
                        fileoffset,
                        fo_next,
                        (*linep).to_String_noraw()
                    );
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                dex!(
                    "return ResultS3LineFind::Found({}, LineP) @[{}, {}] {:?}",
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return Some(ResultS3LineFind::Found((fo_next, linep)));
            }
            if self.find_line_lru_cache_enabled {
                self.find_line_lru_cache_put += 1;
                deo!("LRU Cache put({}, Found({}, …))", fileoffset, fo_next);
                self.find_line_lru_cache
                    .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
            }
            dex!(
                "return ResultS3LineFind::Found({}, LineP)  @[{}, {}] {:?}",
                fo_next,
                (*linep).fileoffset_begin(),
                (*linep).fileoffset_end(),
                (*linep).to_String_noraw()
            );
            return Some(ResultS3LineFind::Found((fo_next, linep)));
        } else {
            self.lines_miss += 1;
        }
        // second, check if there is a `Line` at a preceding offset
        match self.get_linep(&fileoffset) {
            Some(linep) => {
                deo!("self.get_linep({}) returned @{:p}", fileoffset, linep);
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                let fo_next: FileOffset = (*linep).fileoffset_end() + charsz_fo;
                if self.is_line_last(&linep) {
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        deo!(
                            "LRU Cache put({}, Found({}, …)) {:?}",
                            fileoffset,
                            fo_next,
                            (*linep).to_String_noraw()
                        );
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                    }
                    defx!(
                        "return ResultS3LineFind::Found({}, LineP) @[{}, {}] {:?}",
                        fo_next,
                        (*linep).fileoffset_begin(),
                        (*linep).fileoffset_end(),
                        (*linep).to_String_noraw()
                    );
                    return Some(ResultS3LineFind::Found((fo_next, linep)));
                }
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    deo!(
                        "LRU Cache put({}, Found({}, …)) {:?}",
                        fileoffset,
                        fo_next,
                        (*linep).to_String_noraw()
                    );
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "return ResultS3LineFind::Found({}, LineP) @[{}, {}] {:?}",
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return Some(ResultS3LineFind::Found((fo_next, linep)));
            }
            None => {
                deo!("fileoffset {} not found in self.lines_by_range", fileoffset);
            }
        }
        defx!("fileoffset {} not found in self.lines", fileoffset);

        None
    }

    /// Find the [`Line`] at [`FileOffset`] within the same [`Block`].
    /// This does a linear search over the `Block`, _O(n)_.
    ///
    /// If a `Line` extends before or after the `Block` then [`Done`] is
    /// returned.
    ///
    /// Returned `ResultS3LineFind(fileoffset, …)` may refer to a different
    /// proceeding `Block`.
    ///
    /// The second parameter of the returned tuple is a "partial" `Line`.
    /// That is, a `Line` that starts in `Block` but extends to the next
    /// `Block`. In this case, the result, the first tuple parameter,
    /// will be `ResultS3LineFind::Done`. The returned `Line` is not stored
    /// by this `LineReader` and should be quickly dropped by the caller. If
    /// the "partial" `Line` is held too long then the underlying `Block`
    /// cannot be dropped during the "[streaming stage]".
    ///
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`Found`]: crate::common::ResultS3
    /// [`Done`]: crate::common::ResultS3
    /// [`Line`]: crate::data::line::Line
    /// [`FileOffset`]: crate::common::FileOffset
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    //
    // TODO: [2022/05] add test for this:
    //       Keep in mind, a `Line` with terminating-newline as the last byte a `Block`
    //       may be allowed. However, a `Line` with preceding `Line` newline in prior `Block`
    //       may not be found, since the preceding `Line` terminating-newline must be found.
    //       In other words, last byte of `Line` may be last byte of `Block` and the `Line`
    //       will be found. However, if first byte of `Line` is first byte of `Block` then
    //       it will not be found.
    //
    // XXX: This function `find_line` is large and cumbersome.
    //      Changes require extensive retesting.
    //      Extensive debug prints are left in place to aid this.
    //      It could use some improvements but for now it gets the job done.
    //      You've been warned.
    //
    pub fn find_line_in_block(
        &mut self,
        fileoffset: FileOffset,
    ) -> (ResultS3LineFind, Option<Line>) {
        defn!("({})", fileoffset);

        // XXX: using cache can result in non-idempotent behavior
        // check fast LRU
        if let Some(result) = self.check_store_LRU(fileoffset) {
            defx!("({}): return {:?}, None", fileoffset, result);
            return (result, None);
        }

        let filesz: FileSz = self.filesz();

        // handle special cases
        if filesz == 0 {
            defx!("({}): return ResultS3LineFind::Done, None; file is empty", fileoffset);
            return (ResultS3LineFind::Done, None);
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            defx!(
                "({}): WARNING: return ResultS3LineFind::Done, None; fileoffset {} was too big filesz {}!",
                fileoffset,
                fileoffset,
                filesz
            );
            return (ResultS3LineFind::Done, None);
        } else if fileoffset == filesz {
            defx!(
                "({}): return ResultS3LineFind::Done(), None; fileoffset {} is at end of file {}!",
                fileoffset,
                fileoffset,
                filesz
            );
            return (ResultS3LineFind::Done, None);
        }

        // check container of `Line`s
        // XXX: using cache can result in non-idempotent behavior
        if let Some(result) = self.check_store(fileoffset) {
            defx!("({}): return {:?}, None", fileoffset, result);
            return (result, None);
        }

        //
        // Could not find `fileoffset` from prior saved information so…
        // 1. search "part B" or "newline B"
        //    walk through this one block
        //    start from `fileoffset` and walk forwards
        //    look for line terminating '\n' or end of file
        // 2. then do search "part A" or "newline A"
        //    start from passed `fileoffset` and walk backwards
        //    look for line terminating '\n' of previous Line or beginning of file
        //
        // After step 1., do not access anything after `fileoffset`
        //
        // There is a special-case "partial_line" where newline B is not found
        // but a newline A is found. In this case, return `(Done, Line)`. The `Line`
        // is temporary and should not be stored.
        //

        defo!("searching for first newline B (line terminator) …");

        let charsz_fo: FileOffset = self.charsz_ as FileOffset;
        let charsz_bi: BlockIndex = self.charsz_ as BlockIndex;
        let blockoffset_last: BlockOffset = self.blockoffset_last();

        // if NewLine part A cannot be found maybe we can find NewLine part B
        let mut partial_line= false;
        // FOUND NewLine part A? Line begins after that newline
        let mut found_nl_a = false;
        // FOUND NewLine part B? Line ends at this.
        let mut found_nl_b: bool = false;
        // FileOffset NewLine A
        // `fo_nl_a` should eventually "point" to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = fileoffset;
        // FileOffset NewLine B
        // `fo_nl_b` should eventually "point" to end of `Line` including the newline char.
        // if  line is terminated by end-of-file then "points" to last char of file.
        let mut fo_nl_b: FileOffset = fileoffset;
        // BlockIndex NewLine B
        //let mut bi_nl_b: BlockIndex;
        // NewLine B EOF?
        // was newline B actually the end of file?
        let mut nl_b_eof: bool = false;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            defo!("newline A0 is {} because fileoffset {} is beginning of file!", fo_nl_a, fileoffset);
        }
        // append new `LinePart`s to this `Line`
        let mut line: Line = Line::new();

        // The "middle" block is block referred to by `fileoffset` and could be the inexact "middle"
        // of the eventually found `Line`. In other words, `Line.fileoffset_begin` could be before it (or in it)
        // and `Line.fileoffset_end` could be after it (or in it).
        let bo_middle: BlockOffset = self.block_offset_at_file_offset(fileoffset);
        let bi_middle: BlockIndex = self.block_index_at_file_offset(fileoffset);
        let mut bi_middle_end: BlockIndex = bi_middle;
        // search within "middle" block for newline B
        let bptr_middle: BlockP = match self
            .blockreader
            .read_block(bo_middle)
        {
            ResultS3ReadBlock::Found(val) => {
                defo!(
                    "B1: read_block({}) returned Found Block len {} while searching for newline A",
                    bo_middle,
                    (*val).len()
                );
                val
            }
            ResultS3ReadBlock::Done => {
                defx!("({}) B1: read_block({}) returned Done {:?}, None", fileoffset, bo_middle, self.path());
                return (ResultS3LineFind::Done, None);
            }
            ResultS3ReadBlock::Err(err) => {
                defx!(
                    "({}) B1: read_block({}) returned Err; return ResultS3LineFind::Err({:?}), None",
                    fileoffset,
                    bo_middle,
                    err
                );
                return (ResultS3LineFind::Err(err), None);
            }
        };

        let mut bi_at: BlockIndex = bi_middle;
        let bi_stop: BlockIndex = bptr_middle.len() as BlockIndex;
        debug_assert_ge!(bi_stop, charsz_bi, "bi_stop is less than charsz; not yet handled");

        // XXX: only handle UTF-8/ASCII encoding
        defo!("({}) B1: scan middle block {} forwards, starting from blockindex {} (fileoffset {}) searching for newline B",
            fileoffset,
            bo_middle,
            bi_at,
            self.file_offset_at_block_offset_index(bo_middle, bi_at)
        );
        loop {
            // XXX: only handle UTF-8/ASCII encoding
            if (*bptr_middle)[bi_at] == NLu8 {
                found_nl_b = true;
                fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                //bi_nl_b = bi_at;
                bi_middle_end = bi_at;
                defo!("B1: bi_middle_end {:?} fo_nl_b {:?}", bi_middle_end, fo_nl_b);
                defo!(
                    "B1: found newline B in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                    bo_middle,
                    bi_at,
                    fo_nl_b,
                    byte_to_char_noraw((*bptr_middle)[bi_at]),
                );
                break;
            } else {
                bi_at += charsz_bi;
            }
            if bi_at >= bi_stop {
                break;
            }
        } // end loop
          // if (newline B not found and the "middle" block was the last block) then eof is newline B
        if !found_nl_b && bo_middle == blockoffset_last {
            found_nl_b = true;
            debug_assert_ge!(bi_at, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B1 at end of file for file {:?}", bi_at, charsz_bi, self.path());
            let bi_: usize = bi_at - charsz_bi;
            fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_);
            //bi_nl_b = bi_;
            bi_middle_end = bi_;
            defo!(
                "B1: bi_middle_end {:?} fo_nl_b {:?} blockoffset_last {:?}",
                bi_middle_end,
                fo_nl_b,
                blockoffset_last,
            );
            nl_b_eof = true;
            debug_assert_eq!(
                fo_nl_b, filesz - 1,
                "newline B1 fileoffset {} is at end of file, yet filesz is {}; there was a bad calculation of newline B1 from blockoffset {} blockindex {} (blockoffset last {}), for file {:?}",
                fo_nl_b,
                filesz,
                bo_middle,
                bi_,
                blockoffset_last,
                self.path(),
            );
        } else if found_nl_b && self.is_fileoffset_last(fo_nl_b) {
            debug_assert_eq!(
                bo_middle, blockoffset_last,
                "blockoffset 'middle' {}, blockoffset last {}, yet newline B FileOffset {} is last byte of filesz {}, for file {:?}",
                bo_middle, blockoffset_last, fo_nl_b, self.filesz(), self.path(),
            );
            nl_b_eof = true;
        }
        if !found_nl_b {
            partial_line = true;
            /*
            defx!(
                "({}): failed to find newline B in block {}; return Done {:?}, Some({:?})",
                fileoffset,
                bo_middle,
                self.path(),
                line,
            );
            return (ResultS3LineFind::Done, Some(line));
            */
        }

        if partial_line {
            defo!(
                "({}): did not find first newline B, searching for preceding newline A starting at {}, for possible partial Line …",
                fileoffset,
                fileoffset,
            );
        } else {
            defo!(
                "({}): found first newline B at FileOffset {}, searching for preceding newline A. Search starts at FileOffset {} …",
                fileoffset,
                fo_nl_b,
                fileoffset,
            );
        }

        // if found_nl_a was already found then this function can return
        if found_nl_a {
            defo!("({}) A0: already found newline A and newline B, return early", fileoffset);
            debug_assert_eq!(fo_nl_a, 0, "newline A is {}, only reason newline A should be found at this point was if passed fileoffset 0, (passed fileoffset {}), for file {:?}", fo_nl_a, fileoffset, self.path());
            let li: LinePart = LinePart::new(
                bptr_middle,
                self.block_index_at_file_offset(fo_nl_a),
                bi_middle_end + 1,
                fo_nl_a,
                self.block_offset_at_file_offset(fo_nl_a),
                self.blocksz(),
            );
            line.prepend(li);

            if partial_line {
                // NewLine B or EOF was *not* found! So only return a special-case "partial" `Line`.
                // This temporary `Line` will not be stored in this `LineReader` and
                // should not be stored by the caller, either.
                defx!(
                    "({}): found newline A, failed to find newline B in block {}, partial Line; return Done {:?}, Some(@{:p}) {:?}",
                    fileoffset,
                    bo_middle,
                    self.path(),
                    &line,
                    line.to_String_noraw(),
                );
                return (ResultS3LineFind::Done, Some(line));
            }

            let linep: LineP = self.insert_line(line);
            let fo_next: FileOffset = fo_nl_b + charsz_fo;
            debug_assert_eq!(
                fo_next,
                (*linep).fileoffset_end() + charsz_fo,
                "mismatching fo_next {} != (*linep).fileoffset_end()+1, for file {:?}",
                fo_next,
                self.path()
            );
            if !nl_b_eof {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    defo!(
                        "({}) A0: LRU cache put({}, Found(({}, @{:p})))",
                        fileoffset,
                        fileoffset,
                        fo_next,
                        linep
                    );
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "({}) A0: return ResultS3LineFind::Found(({}, LineP)), None; @[{}, {}] {:?}",
                    fileoffset,
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return (ResultS3LineFind::Found((fo_next, linep)), None);
            } else {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    defo!(
                        "({}) A0: LRU cache put({}, Found(({}, @{:p})))",
                        fileoffset,
                        fileoffset,
                        fo_next,
                        linep
                    );
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "({}) A0: return ResultS3LineFind::Found(({}, LineP)), None; @[{}, {}] {:?}",
                    fileoffset,
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return (ResultS3LineFind::Found((fo_next, linep)), None);
            };
        }

        assert!(
            !found_nl_a,
            "already found newline A; was finding it once not good enough? {:?}",
            self.path()
        );
        assert!(
            // XOR check
            !found_nl_b ^ !partial_line,
            "found newline A, found_nl_b {} yet {} partial_line, expected only one; bird with one wing. {:?}",
            found_nl_b, partial_line,
            self.path(),
        );

        if fileoffset >= charsz_fo {
            let fo_: FileOffset = fileoffset - charsz_fo;
            if !partial_line && self.lines.contains_key(&fo_) {
                self.lines_hits += 1;
                defo!("({}) A1a: hit in self.lines for FileOffset {} (before part A)", fileoffset, fo_);
                fo_nl_a = fo_;
                let linep_prev: LineP = self.lines[&fo_nl_a].clone();
                debug_assert_eq!(
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                    "get_linep({}) returned Line with fileoffset_end() {}; these should match for file {:?}",
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                    self.path(),
                );
                let li: LinePart = LinePart::new(
                    bptr_middle,
                    self.block_index_at_file_offset(fileoffset),
                    bi_middle_end + 1,
                    fileoffset,
                    self.block_offset_at_file_offset(fileoffset),
                    self.blocksz(),
                );
                line.prepend(li);
                let linep: LineP = self.insert_line(line);
                let fo_next: FileOffset = fo_nl_b + charsz_fo;
                if nl_b_eof {
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        defo!(
                            "({}) A1a: LRU Cache put({}, Found({}, …)) {:?}",
                            fileoffset,
                            fileoffset,
                            fo_next,
                            (*linep).to_String_noraw()
                        );
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                    }
                    defx!(
                        "({}): return ResultS3LineFind::Found({}, LineP), None;  @[{}, {}] {:?}",
                        fileoffset,
                        fo_next,
                        (*linep).fileoffset_begin(),
                        (*linep).fileoffset_end(),
                        (*linep).to_String_noraw()
                    );
                    return (ResultS3LineFind::Found((fo_next, linep)), None);
                }
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    defo!(
                        "({}) A1a: LRU Cache put({}, Found({}, …)) {:?}",
                        fileoffset,
                        fileoffset,
                        fo_next,
                        (*linep).to_String_noraw()
                    );
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "({}): return ResultS3LineFind::Found({}, LineP), None;  @[{}, {}] {:?}",
                    fileoffset,
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return (ResultS3LineFind::Found((fo_next, linep)), None);
            } else {
                self.lines_miss += 1;
                defo!(
                    "({}) A1a: miss in self.lines for FileOffset {} (quick check before part A)",
                    fileoffset,
                    fo_
                );
            }

            if !partial_line {
                // quick-check if this Line is already known
                match self.get_linep(&fo_) {
                    Some(linep_prev) => {
                        defo!("({}) A1b: self.get_linep({}) returned Line", fileoffset, fo_);
                        // TODO: Issue #61 enable expression attribute when feature is stable
                        //       #[allow(unused_assignments)]
                        //found_nl_a = true;
                        fo_nl_a = (*linep_prev).fileoffset_end();
                        debug_assert_eq!(
                            fo_nl_a, fo_,
                            "get_linep({}) returned Line with fileoffset_end() {}; these should match for file {:?}",
                            fo_,
                            fo_nl_a,
                            self.path(),
                        );
                        let li: LinePart = LinePart::new(
                            bptr_middle,
                            self.block_index_at_file_offset(fileoffset),
                            bi_middle_end + 1,
                            fileoffset,
                            self.block_offset_at_file_offset(fileoffset),
                            self.blocksz(),
                        );
                        line.prepend(li);
                        let linep: LineP = self.insert_line(line);
                        let fo_next: FileOffset = fo_nl_b + charsz_fo;
                        if nl_b_eof {
                            debug_assert!(
                                self.is_line_last(&linep),
                                "nl_b_eof true yet !is_line_last(linep) file {:?}",
                                self.path()
                            );
                            if self.find_line_lru_cache_enabled {
                                self.find_line_lru_cache_put += 1;
                                defo!(
                                    "({}) A1b: LRU Cache put({}, Found({}, …)) {:?}",
                                    fileoffset,
                                    fileoffset,
                                    fo_next,
                                    (*linep).to_String_noraw()
                                );
                                self.find_line_lru_cache
                                    .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                            }
                            defx!(
                                "({}): return ResultS3LineFind::Found(({}, LineP)), None;  @[{}, {}] {:?}",
                                fileoffset,
                                fo_next,
                                (*linep).fileoffset_begin(),
                                (*linep).fileoffset_end(),
                                (*linep).to_String_noraw()
                            );
                            return (ResultS3LineFind::Found((fo_next, linep)), None);
                        }
                        debug_assert!(!self.is_line_last(&linep), "nl_b_eof true yet !is_line_last(linep)");
                        if self.find_line_lru_cache_enabled {
                            self.find_line_lru_cache_put += 1;
                            defo!(
                                "({}) A1b: LRU Cache put({}, Found({}, …)) {:?}",
                                fileoffset,
                                fileoffset,
                                fo_next,
                                (*linep).to_String_noraw()
                            );
                            self.find_line_lru_cache
                                .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                        }
                        defx!(
                            "({}): return ResultS3LineFind::Found(({}, LineP)), None;  @[{}, {}] {:?}",
                            fileoffset,
                            fo_next,
                            (*linep).fileoffset_begin(),
                            (*linep).fileoffset_end(),
                            (*linep).to_String_noraw()
                        );
                        return (ResultS3LineFind::Found((fo_next, linep)), None);
                    }
                    None => {
                        defo!(
                            "({}) A1b: self.get_linep({}) returned None (quick check before part A)",
                            fileoffset,
                            fo_
                        );
                    }
                }
            }
        }

        //
        // getting here means this function is discovering a brand new `Line` (searching for newline A)
        // walk *backwards* to find line-terminating newline of the preceding line (or beginning of file)
        //

        // FileOffset NewLine A SEARCH START
        let fo_nl_a_search_start: FileOffset = std::cmp::max(fileoffset, charsz_fo) - charsz_fo;
        // BlockOFfset
        let bof: BlockOffset = self.block_offset_at_file_offset(fo_nl_a_search_start);
        // BEGinning OFfset?
        let mut begof: bool = false; // run into beginning of file (as in first byte)?
                                     // newline A plus one (one charsz past preceding Line terminating '\n')
        // FileOffset NewLine A1
        let mut fo_nl_a1: FileOffset = 0;

        if bof != bo_middle {
            defx!(
                "({}): failed to find newline A within block {}, return Done {:?}, None",
                fileoffset,
                bo_middle,
                self.path()
            );
            return (ResultS3LineFind::Done, None);
        }

        // search for newline A starts within "middle" block
        let mut bi_at: BlockIndex = self.block_index_at_file_offset(fo_nl_a_search_start);
        const BI_STOP: BlockIndex = 0;
        defo!(
            "({}) A2a: scan middle block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
            fileoffset,
            bo_middle,
            bi_at,
            self.file_offset_at_block_offset_index(bo_middle, bi_at),
            BI_STOP,
        );
        loop {
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            if (*bptr_middle)[bi_at] == NLu8 {
                found_nl_a = true;
                fo_nl_a = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                defo!(
                    "({}) A2a: found newline A in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                    fileoffset,
                    bo_middle,
                    bi_at,
                    fo_nl_a,
                    byte_to_char_noraw((*bptr_middle)[bi_at]),
                );
                // adjust offsets one forward
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                fo_nl_a1 = fo_nl_a + charsz_fo;
                bi_at += charsz_bi;
                break;
            }
            if bi_at == BI_STOP {
                break;
            }
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            bi_at -= charsz_bi;
        }

        if bof == 0 {
            defo!("({}) A2a: run into beginning of file", fileoffset);
            begof = true;
        }
        if !found_nl_a && begof {
            found_nl_a = true;
            //#[allow(unused_assignments)]
            //fo_nl_a = 0;
            fo_nl_a1 = 0;
        }
        if !found_nl_a {
            defo!("({}) A2a: newline A not found in middle block {}", fileoffset, bo_middle);
            defx!("find_line_in_block({}): return Done {:?}, None", fileoffset, self.path());
            return (ResultS3LineFind::Done, None);
        }

        let li: LinePart =
            LinePart::new(bptr_middle, bi_at, bi_middle_end + 1, fo_nl_a1, bo_middle, self.blocksz());
        line.prepend(li);

        if partial_line {
            // return the bytes from newline A to end of this Block as a special "partial" `Line`
            defx!(
                "({}): return ResultS3LineFind::Done, partial {:?}; @[{}, {}] {:?}",
                fileoffset,
                line,
                line.fileoffset_begin(),
                line.fileoffset_end(),
                line.to_String_noraw()
            );
            return (ResultS3LineFind::Done, Some(line));
        }

        let linep: LineP = LineP::new(line);
        let fo_next: FileOffset = fo_nl_b + charsz_fo;

        if nl_b_eof {
            defx!(
                "({}): return ResultS3LineFind::Found({}, LineP), None;  @[{}, {}] {:?}",
                fileoffset,
                fo_next,
                (*linep).fileoffset_begin(),
                (*linep).fileoffset_end(),
                (*linep).to_String_noraw()
            );
            return (ResultS3LineFind::Found((fo_next, linep)), None);
        }

        defx!(
            "({}): return ResultS3LineFind::Found({}, LineP), None; @[{}, {}] {:?}",
            fileoffset,
            fo_next,
            (*linep).fileoffset_begin(),
            (*linep).fileoffset_end(),
            (*linep).to_String_noraw()
        );

        (ResultS3LineFind::Found((fo_next, linep)), None)
    }

    /// Find next [`Line`] starting from passed [`FileOffset`].
    /// This does a linear search over the file, _O(n)_.
    ///
    /// During the process of finding, this creates and stores the `Line` from
    /// underlying [`Block`] data.
    /// Returns [`Found`] (`FileOffset` of beginning of the _next_ line, found
    /// `LineP`)
    /// Reaching end of file returns `FileOffset` value that is one byte past
    /// the actual end of file (and should not be used).
    /// Otherwise returns [`Err`], all other `Result::Err`
    /// errors are propagated.
    ///
    /// This function has the densest number of byte↔char handling and
    /// transitions within this program.
    ///
    /// Correllary to functions `find_sysline`, `read_block`.
    ///
    /// Throughout this function, _newline A_ points to the line beginning,
    /// _newline B_ points to line ending. Both are inclusive.
    ///
    /// Here are two defining cases of this function:
    ///
    /// Given a file of four newlines:
    ///
    /// ```text
    ///     byte: 0123
    ///     char: ␊␊␊␊
    /// ```
    ///
    /// Calls to `find_line` would result in a `Line`
    ///
    /// ```text
    ///     A=Line.fileoffset_begin();
    ///     B=Line.fileoffset_end();
    ///     Val=Line.to_string();
    ///
    ///                     A,B  Val
    ///     find_line(0) -> 0,0 "␊"
    ///     find_line(1) -> 1,1 "␊"
    ///     find_line(2) -> 2,2 "␊"
    ///     find_line(3) -> 3,3 "␊"
    /// ```
    ///
    /// Given a file with two alphabet chars and one newline:
    ///
    /// ```text
    ///     012
    ///     x␊y
    ///
    ///                     A,B  Val
    ///     fine_line(0) -> 0,1 "x␊"
    ///     fine_line(1) -> 0,1 "x␊"
    ///     fine_line(2) -> 2,2 "y"
    /// ```
    ///
    /// XXX: returning the "next fileoffset (along with `LineP`) is jenky. Just return the `LineP`.
    ///      and/or add `iter` capabilities to `Line` that will hide tracking the "next fileoffset".
    ///
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`Found`]: crate::common::ResultS3
    /// [`Err`]: crate::common::ResultS3
    /// [`Line`]: crate::data::line::Line
    /// [`FileOffset`]: crate::common::FileOffset
    // XXX: This function `find_line` is large and cumbersome.
    //      Changes require extensive retesting.
    //      Extensive debug prints are left in place to aid this.
    //      You've been warned.
    //
    // XXX: Issue #16 only handles UTF-8/ASCII encoding
    pub fn find_line(
        &mut self,
        fileoffset: FileOffset,
    ) -> ResultS3LineFind {
        defn!("({})", fileoffset);

        // some helpful constants
        let charsz_fo: FileOffset = self.charsz_ as FileOffset;
        let charsz_bi: BlockIndex = self.charsz_ as BlockIndex;
        let filesz: FileSz = self.filesz();
        let blockoffset_last: BlockOffset = self.blockoffset_last();

        // check fast LRU first
        if let Some(result) = self.check_store_LRU(fileoffset) {
            defx!("({}): return {:?}", fileoffset, result);
            return result;
        }

        // handle special cases
        if filesz == 0 {
            defx!("({}): return ResultS3LineFind::Done; file is empty", fileoffset);
            return ResultS3LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            defx!(
                "({}): return ResultS3LineFind::Done; fileoffset {} was too big filesz {}!",
                fileoffset,
                fileoffset,
                filesz
            );
            return ResultS3LineFind::Done;
        } else if fileoffset == filesz {
            defx!(
                "({}): return ResultS3LineFind::Done(); fileoffset {} is at end of file, filesz {}!",
                fileoffset,
                fileoffset,
                filesz
            );
            return ResultS3LineFind::Done;
        }

        // check container of `Line`s
        if let Some(result) = self.check_store(fileoffset) {
            defx!("({}): return {:?}", fileoffset, result);
            return result;
        }

        //
        // could not find `fileoffset` from prior saved information so…
        // walk through blocks and bytes looking for beginning of a line (a newline character)
        // start with newline search "part B" (look for line terminating '\n' or end of file)
        // then do search "part A" (look for line terminating '\n' of previous Line or beginning
        // of file)
        //

        defo!("searching for first newline B (line terminator) …");

        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // found newline part B? Line ends at this.
        let mut found_nl_b: bool = false;
        // `fo_nl_a` should eventually "point" to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = fileoffset;
        // `fo_nl_b` should eventually "point" to end of `Line` including the newline char.
        // if  line is terminated by end-of-file then "points" to last char of file.
        let mut fo_nl_b: FileOffset = fileoffset;
        let mut fo_nl_b_in_middle: bool = false;
        // was newline B actually the end of file?
        let mut nl_b_eof: bool = false;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            defo!("newline A0 is {} because fileoffset {} is beginning of file!", fo_nl_a, fileoffset);
        }
        // append new `LinePart`s to this `Line`
        let mut line: Line = Line::new();

        // The "middle" block is block referred to by `fileoffset` and could be the inexact "middle"
        // of the eventually found `Line`. In other words, `Line.fileoffset_begin` could be before it (or in it)
        // and `Line.fileoffset_end` could be after it (or in it).
        let bo_middle: BlockOffset = self.block_offset_at_file_offset(fileoffset);
        let bi_middle: BlockIndex = self.block_index_at_file_offset(fileoffset);
        let mut bi_middle_end: BlockIndex = bi_middle;
        let bptr_middle: BlockP;

        // search within "middle" block for newline B
        {
            // arbitrary statement block
            bptr_middle = match self
                .blockreader
                .read_block(bo_middle)
            {
                ResultS3ReadBlock::Found(val) => {
                    defo!(
                        "B1: read_block({}) returned Found Block len {} while searching for newline A",
                        bo_middle,
                        (*val).len()
                    );
                    val
                }
                ResultS3ReadBlock::Done => {
                    defx!("B1: read_block({}) returned Done {:?}", bo_middle, self.path());
                    return ResultS3LineFind::Done;
                }
                ResultS3ReadBlock::Err(err) => {
                    defx!(
                        "B1: read_block({}) returned Err, return ResultS3LineFind::Err({:?})",
                        bo_middle,
                        err
                    );
                    return ResultS3LineFind::Err(err);
                }
            };
            let mut bi_at: BlockIndex = bi_middle;
            let bi_stop: BlockIndex = bptr_middle.len() as BlockIndex;
            debug_assert_ge!(bi_stop, charsz_bi, "bi_stop is less than charsz; not yet handled");
            // XXX: Issue #16 only handles UTF-8/ASCII encoding
            //bi_beg = bi_stop - charsz_bi;
            defo!("B1: scan middle block {} forwards (block len {}), starting from blockindex {} (fileoffset {}) searching for newline B", bo_middle, (*bptr_middle).len(), bi_at, self.file_offset_at_block_offset_index(bo_middle, bi_at));
            loop {
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                if (*bptr_middle)[bi_at] == NLu8 {
                    found_nl_b = true;
                    fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                    bi_middle_end = bi_at;
                    defo!("B1: bi_middle_end {:?} fo_nl_b {:?}", bi_middle_end, fo_nl_b);
                    fo_nl_b_in_middle = true;
                    defo!(
                        "B1: found newline B in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                        bo_middle,
                        bi_at,
                        fo_nl_b,
                        byte_to_char_noraw((*bptr_middle)[bi_at]),
                    );
                    break;
                } else {
                    bi_at += charsz_bi;
                }
                if bi_at >= bi_stop {
                    break;
                }
            } // end loop
              // if (newline B not found and the "middle" block was the last block) then eof is newline B
            if !found_nl_b && bo_middle == blockoffset_last {
                found_nl_b = true;
                debug_assert_ge!(bi_at, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B1 at end of file {:?}", bi_at, charsz_bi, self.path());
                let bi_: BlockIndex = bi_at - charsz_bi;
                fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_);
                bi_middle_end = bi_;
                defo!(
                    "B1: bi_middle_end {:?} fo_nl_b {:?} blockoffset_last {:?}",
                    bi_middle_end,
                    fo_nl_b,
                    blockoffset_last
                );
                fo_nl_b_in_middle = true;
                nl_b_eof = true;
                debug_assert_eq!(
                    fo_nl_b, filesz - 1,
                    "newline B1 fileoffset {} is at end of file, yet filesz is {}; there was a bad calculation of newline B1 from blockoffset {} blockindex {} (blockoffset last {}) for file {:?}",
                    fo_nl_b,
                    filesz,
                    bo_middle,
                    bi_,
                    blockoffset_last,
                    self.path(),
                );
            } else if !found_nl_b {
                bi_middle_end = bi_stop - charsz_bi;
                defo!("B1: bi_middle_end {:?}", bi_middle_end);
            }
        }

        if found_nl_b {
            defo!("B2: skip continued backwards search for newline B (already found)");
        } else {
            // search within proceeding blocks for newline B
            const BI_UNINIT: BlockIndex = usize::MAX;
            let mut bi_beg: BlockIndex = BI_UNINIT; // XXX: value BI_UNINIT is a hacky "uninitialized" signal
            let mut bi_end: BlockIndex = BI_UNINIT; // XXX: value BI_UNINIT is a hacky "uninitialized" signal
            let mut bof = bo_middle + 1;
            while !found_nl_b && bof <= blockoffset_last {
                let bptr: BlockP = match self
                    .blockreader
                    .read_block(bof)
                {
                    ResultS3ReadBlock::Found(val) => {
                        defo!(
                            "B2: read_block({}) returned Found Block len {} while searching for newline B",
                            bof,
                            (*val).len()
                        );
                        val
                    }
                    ResultS3ReadBlock::Done => {
                        defx!("B2: read_block({}) returned Done {:?}", bof, self.path());
                        return ResultS3LineFind::Done;
                    }
                    ResultS3ReadBlock::Err(err) => {
                        defx!(
                            "B2: read_block({}) returned Err, return ResultS3LineFind::Err({:?})",
                            bof,
                            err
                        );
                        return ResultS3LineFind::Err(err);
                    }
                };
                bi_beg = 0;
                bi_end = (*bptr).len() as BlockIndex;
                debug_assert_ge!(
                    bi_end,
                    charsz_bi,
                    "blockindex bi_end {} is less than charsz; not yet handled, file {:?}",
                    bi_end,
                    self.path()
                );
                debug_assert_ne!(
                    bi_end, 0,
                    "blockindex bi_end is zero; Block at blockoffset {}, BlockP @0x{:p}, has len() zero",
                    bof, bptr
                );
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                //bi_beg = bi_end - charsz_bi;
                defo!(
                    "B2: scan block {} forwards, starting from blockindex {} (fileoffset {}) up to blockindex {} searching for newline B",
                    bof,
                    bi_beg,
                    self.file_offset_at_block_offset_index(bof, bi_beg),
                    bi_end,
                );
                loop {
                    // XXX: Issue #16 only handles UTF-8/ASCII encoding
                    if (*bptr)[bi_beg] == NLu8 {
                        found_nl_b = true;
                        fo_nl_b = self.file_offset_at_block_offset_index(bof, bi_beg);
                        assert!(
                            !fo_nl_b_in_middle,
                            "fo_nl_b_in_middle should be false, file {:?}",
                            self.path()
                        );
                        defo!(
                            "B2: found newline B during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            bof,
                            bi_beg,
                            fo_nl_b,
                            byte_to_char_noraw((*bptr)[bi_beg]),
                        );
                        let li: LinePart = LinePart::new(
                            bptr.clone(),
                            0,
                            bi_beg + 1,
                            self.file_offset_at_block_offset_index(bof, 0),
                            bof,
                            self.blocksz(),
                        );
                        line.append(li);
                        break;
                    } else {
                        bi_beg += charsz_bi;
                    }
                    if bi_beg >= bi_end {
                        break;
                    }
                } // end loop
                if found_nl_b {
                    break;
                }
                // newline B was not found in this `Block`, but must save this `Block` information as a `LinePart.
                let li: LinePart = LinePart::new(
                    bptr.clone(),
                    0,
                    bi_beg,
                    self.file_offset_at_block_offset_index(bof, 0),
                    bof,
                    self.blocksz(),
                );
                line.append(li);
                bof += 1;
            } // end while bof <= blockoffset_last
              // if newline B not found and last checked block was the last block
              // then EOF is newline B
            if !found_nl_b && bof > blockoffset_last {
                bof = blockoffset_last;
                found_nl_b = true;
                debug_assert_ne!(bi_beg, BI_UNINIT, "blockindex begin is uninitialized");
                debug_assert_ne!(bi_end, BI_UNINIT, "blockindex end is uninitialized");
                debug_assert_ge!(bi_beg, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B2 at end of file {:?}", bi_beg, charsz_bi, self.path());
                debug_assert_eq!(bi_beg, bi_end, "blockindex begin {} != {} blockindex end, yet entire last block was searched (last blockoffset {}) file {:?}", bi_beg, bi_end, blockoffset_last, self.path());
                let bi_: BlockIndex = bi_beg - charsz_bi;
                fo_nl_b = self.file_offset_at_block_offset_index(bof, bi_);
                nl_b_eof = true;
                defo!(
                    "B2: newline B is end of file; blockoffset {} blockindex {} fileoffset {} (blockoffset last {})",
                    bof,
                    bi_,
                    fo_nl_b,
                    blockoffset_last,
                );
                debug_assert_eq!(
                    fo_nl_b, filesz - 1,
                    "newline B2 fileoffset {} is supposed to be the end of file, yet filesz is {}; bad calculation of newline B2 from blockoffset {} blockindex {} (last blockoffset {}) (bi_beg {} bi_end {}) (charsz {}) file {:?}",
                    fo_nl_b,
                    filesz,
                    bof,
                    bi_,
                    blockoffset_last,
                    bi_beg,
                    bi_end,
                    charsz_bi,
                    self.path(),
                );
            }
        } // end if !found_nl_b

        //
        // walk backwards through blocks and bytes looking for newline A (line terminator of preceding Line or beginning of file)
        //

        defo!(
            "found first newline B at FileOffset {}, searching for preceding newline A. Search starts at FileOffset {} …",
            fo_nl_b,
            fileoffset,
        );

        // if found_nl_a was already found then this function can return
        if found_nl_a {
            defo!("A0: already found newline A and newline B, return early");
            debug_assert_eq!(fo_nl_a, 0, "newline A is {}, only reason newline A should be found at this point was if passed fileoffset 0, (passed fileoffset {}) file {:?}", fo_nl_a, fileoffset, self.path());
            let li: LinePart = LinePart::new(
                bptr_middle,
                self.block_index_at_file_offset(fo_nl_a),
                bi_middle_end + 1,
                fo_nl_a,
                self.block_offset_at_file_offset(fo_nl_a),
                self.blocksz(),
            );
            line.prepend(li);
            let linep: LineP = self.insert_line(line);
            let fo_next: FileOffset = fo_nl_b + charsz_fo;
            debug_assert_eq!(
                fo_next,
                (*linep).fileoffset_end() + charsz_fo,
                "mismatching fo_next {} != (*linep).fileoffset_end()+1, file {:?}",
                fo_next,
                self.path()
            );
            if !nl_b_eof {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    defo!("A0: LRU cache put({}, Found(({}, @{:p})))", fileoffset, fo_next, linep);
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "({}) A0: return ResultS3LineFind::Found(({}, LineP)) @[{}, {}] {:?}",
                    fileoffset,
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return ResultS3LineFind::Found((fo_next, linep));
            } else {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    defo!("A0: LRU cache put({}, Found(({}, @{:p})))", fileoffset, fo_next, linep);
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "({}) A0: return ResultS3LineFind::Found(({}, LineP)) @[{}, {}] {:?}",
                    fileoffset,
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return ResultS3LineFind::Found((fo_next, linep));
            };
        }
        assert!(
            !found_nl_a,
            "already found newline A; was finding it once not good enough? file {:?}",
            self.path()
        );
        assert!(
            found_nl_b,
            "found newline A, have not found newline B; bird with one wing. file {:?}",
            self.path()
        );
        // …but before doing work of discovering a new `Line` (newline A),
        // check various maps at `fileoffset + 1` to see if the preceding
        // `Line` has already been discovered and processed.
        // This is common for sequential calls to this function.
        if fileoffset >= charsz_fo {
            let fo_: FileOffset = fileoffset - charsz_fo;
            if self.lines.contains_key(&fo_) {
                self.lines_hits += 1;
                defo!("A1a: hit in self.lines for FileOffset {} (before part A)", fo_);
                fo_nl_a = fo_;
                let linep_prev: LineP = self.lines[&fo_nl_a].clone();
                debug_assert_eq!(
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                    "get_linep({}) returned Line with fileoffset_end() {}; these should match; file {:?}",
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                    self.path(),
                );
                let li: LinePart = LinePart::new(
                    bptr_middle,
                    self.block_index_at_file_offset(fileoffset),
                    bi_middle_end + 1,
                    fileoffset,
                    self.block_offset_at_file_offset(fileoffset),
                    self.blocksz(),
                );
                line.prepend(li);
                let linep: LineP = self.insert_line(line);
                let fo_next: FileOffset = fo_nl_b + charsz_fo;
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    defo!(
                        "A1a: LRU Cache put({}, Found({}, …)) {:?}",
                        fileoffset,
                        fo_next,
                        (*linep).to_String_noraw()
                    );
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                }
                defx!(
                    "({}): return ResultS3LineFind::Found({}, LineP)  @[{}, {}] {:?}",
                    fileoffset,
                    fo_next,
                    (*linep).fileoffset_begin(),
                    (*linep).fileoffset_end(),
                    (*linep).to_String_noraw()
                );
                return ResultS3LineFind::Found((fo_next, linep));
            } else {
                self.lines_miss += 1;
                defo!("A1a: miss in self.lines for FileOffset {} (quick check before part A)", fo_);
            }
            match self.get_linep(&fo_) {
                Some(linep_prev) => {
                    defo!("A1b: self.get_linep({}) returned Line", fo_);
                    // TODO: Issue #61 enable expression attribute when feature is stable
                    //       #[allow(unused_assignments)]
                    //found_nl_a = true;
                    fo_nl_a = (*linep_prev).fileoffset_end();
                    debug_assert_eq!(
                        fo_nl_a,
                        fo_,
                        "get_linep({}) returned Line with fileoffset_end() {}; these should match; file {:?}",
                        fo_,
                        fo_nl_a,
                        self.path(),
                    );
                    let li: LinePart = LinePart::new(
                        bptr_middle,
                        self.block_index_at_file_offset(fileoffset),
                        bi_middle_end + 1,
                        fileoffset,
                        self.block_offset_at_file_offset(fileoffset),
                        self.blocksz(),
                    );
                    line.prepend(li);
                    let linep: LineP = self.insert_line(line);
                    let fo_next: FileOffset = fo_nl_b + charsz_fo;
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        defo!(
                            "A1b: LRU Cache put({}, Found({}, …)) {:?}",
                            fileoffset,
                            fo_next,
                            (*linep).to_String_noraw()
                        );
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS3LineFind::Found((fo_next, linep.clone())));
                    }
                    defx!(
                        "({}): return ResultS3LineFind::Found({}, LineP)  @[{}, {}] {:?}",
                        fileoffset,
                        fo_next,
                        (*linep).fileoffset_begin(),
                        (*linep).fileoffset_end(),
                        (*linep).to_String_noraw()
                    );
                    return ResultS3LineFind::Found((fo_next, linep));
                }
                None => {
                    defo!("A1b: self.get_linep({}) returned None (quick check before part A)", fo_);
                }
            }
        }

        //
        // getting here means this function is discovering a brand new `Line` (searching for newline A)
        // walk *backwards* to find line-terminating newline of the preceding line (or beginning of file)
        //

        let fo_nl_a_search_start: FileOffset = std::cmp::max(fileoffset, charsz_fo) - charsz_fo;
        let mut bof: BlockOffset = self.block_offset_at_file_offset(fo_nl_a_search_start);
        let mut begof: bool = false; // run into beginning of file (as in first byte)?
                                     // newline A plus one (one charsz past preceding Line terminating '\n')
        let mut fo_nl_a1: FileOffset = 0;

        if bof == bo_middle {
            // search for newline A starts within "middle" block
            let mut bi_at: BlockIndex = self.block_index_at_file_offset(fo_nl_a_search_start);
            const BI_STOP: BlockIndex = 0;
            defo!(
                "A2a: scan middle block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
                bo_middle, bi_at, self.file_offset_at_block_offset_index(bo_middle, bi_at), BI_STOP,
            );
            loop {
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                if (*bptr_middle)[bi_at] == NLu8 {
                    found_nl_a = true;
                    fo_nl_a = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                    defo!(
                        "A2a: found newline A in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                        bo_middle,
                        bi_at,
                        fo_nl_a,
                        byte_to_char_noraw((*bptr_middle)[bi_at]),
                    );
                    // adjust offsets one forward
                    // XXX: Issue #16 only handles UTF-8/ASCII encoding
                    fo_nl_a1 = fo_nl_a + charsz_fo;
                    bi_at += charsz_bi;
                    break;
                }
                if bi_at == BI_STOP {
                    break;
                }
                // XXX: Issue #16 only handles UTF-8/ASCII encoding
                bi_at -= charsz_bi;
            }
            let fo_: FileOffset = if found_nl_a {
                fo_nl_a1
            } else {
                defo!("A2a: newline A not found in middle block {} but store middle block", bo_middle);
                self.file_offset_at_block_offset_index(bo_middle, bi_at)
            };
            let li: LinePart =
                LinePart::new(bptr_middle.clone(), bi_at, bi_middle_end + 1, fo_, bo_middle, self.blocksz());
            line.prepend(li);
            if bof != 0 {
                defo!("A2a: blockoffset set to {}", bof);
                bof -= 1;
            } else {
                defo!("A2a: run into beginning of file");
                begof = true;
            }
        } else {
            defo!("A2b: search for newline A crossed block boundary {} -> {}, save LinePart", bo_middle, bof);
            // the charsz shift backward to begin search for newline A crossed block boundary
            // so save linepart from "middle" block before searching further
            let li: LinePart = LinePart::new(
                bptr_middle.clone(),
                0,
                bi_middle_end + 1,
                self.file_offset_at_block_offset_index(bo_middle, 0),
                bo_middle,
                self.blocksz(),
            );
            line.prepend(li);
        }

        if !found_nl_a && begof {
            found_nl_a = true;
            // TODO: Issue #61 enable expression attribute when feature is stable
            //       #[allow(unused_assignments)]
            //fo_nl_a = 0;
            // TODO: Issue #61 enable expression attribute when feature is stable
            //       #[allow(unused_assignments)]
            //fo_nl_a1 = 0;
        }

        if !found_nl_a && !begof {
            let mut bptr_prior: BlockP;
            let mut bptr: BlockP = bptr_middle;
            let mut bi_start_prior: BlockIndex;
            let mut bi_start: BlockIndex = bi_middle;
            while !found_nl_a && !begof {
                // "middle" block should have been handled by now
                // remainder is to just walk backwards chedcking for first newline or beginning of file
                defo!("A4: searching blockoffset {} …", bof);
                bptr_prior = bptr;
                bptr = match self
                    .blockreader
                    .read_block(bof)
                {
                    ResultS3ReadBlock::Found(val) => {
                        defo!(
                            "A4: read_block({}) returned Found Block len {} while searching for newline A",
                            bof,
                            (*val).len()
                        );
                        val
                    }
                    ResultS3ReadBlock::Done => {
                        defx!("A4: read_block({}) returned Done {:?}", bof, self.path());
                        return ResultS3LineFind::Done;
                    }
                    ResultS3ReadBlock::Err(err) => {
                        defx!(
                            "({}) A4: read_block({}) returned Err, return ResultS3LineFind::Err({:?})",
                            fileoffset,
                            bof,
                            err
                        );
                        return ResultS3LineFind::Err(err);
                    }
                };
                let blen: BlockIndex = bptr.len() as BlockIndex;
                debug_assert_ge!(
                    blen,
                    charsz_bi,
                    "blen is less than charsz; not yet handled, file {:?}",
                    self.path()
                );
                bi_start_prior = bi_start;
                bi_start = blen - charsz_bi;
                let mut bi_at: BlockIndex = bi_start;
                const BI_STOP: BlockIndex = 0;
                defo!(
                    "A5: scan block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
                    bof, bi_at, self.file_offset_at_block_offset_index(bof, bi_at), BI_STOP,
                );
                loop {
                    // XXX: Issue #16 only handles UTF-8/ASCII encoding
                    if (*bptr)[bi_at] == NLu8 {
                        found_nl_a = true;
                        fo_nl_a = self.file_offset_at_block_offset_index(bof, bi_at);
                        defo!(
                            "A5: found newline A during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            bof,
                            bi_at,
                            fo_nl_a,
                            byte_to_char_noraw((*bptr)[bi_at]),
                        );
                        // adjust offsets one forward
                        // XXX: Issue #16 only handles UTF-8/ASCII encoding
                        fo_nl_a1 = fo_nl_a + charsz_fo;
                        bi_at += charsz_bi;
                        let bof_a1 = self.block_offset_at_file_offset(fo_nl_a1);
                        if bof_a1 == bof {
                            // newline A and first line char does not cross block boundary
                            defo!("A5: store current blockoffset {}", bof);
                            let li = LinePart::new(
                                bptr.clone(),
                                bi_at,
                                bi_start + 1,
                                fo_nl_a1,
                                bof,
                                self.blocksz(),
                            );
                            line.prepend(li);
                        } else if !line.stores_blockoffset(bof_a1) {
                            // newline A and first line char does cross block boundary
                            defo!("A5: store prior blockoffset {}", bof_a1);
                            // use prior block data
                            let li = LinePart::new(
                                bptr_prior,
                                0,
                                bi_start_prior + 1,
                                fo_nl_a1,
                                bof_a1,
                                self.blocksz(),
                            );
                            line.prepend(li);
                        } else {
                            // newline A and first line char does cross block boundary
                            defo!("A5: blockoffset {} was already stored", bof_a1);
                        }
                        break;
                    }
                    if bi_at == BI_STOP {
                        break;
                    }
                    // XXX: Issue #16 only handles UTF-8/ASCII encoding
                    bi_at -= charsz_bi;
                }
                if found_nl_a {
                    break;
                }
                defo!("A5: store blockoffset {}", bof);
                let li = LinePart::new(
                    bptr.clone(),
                    BI_STOP,
                    bi_start + 1,
                    self.file_offset_at_block_offset_index(bof, 0),
                    bof,
                    self.blocksz(),
                );
                line.prepend(li);
                if bof != 0 {
                    // newline A not found
                    defo!("A5: newline A not found in block {}", bof);
                    bof -= 1;
                } else {
                    // hit beginning of file, "newline A" is the beginning of the file (not a newline char)
                    // store that first block
                    defo!("A5: ran into beginning of file");
                    found_nl_a = true;
                    begof = true;
                    debug_assert!(
                        line.stores_blockoffset(0),
                        "block 0 was not stored but ran into beginning of file {:?}",
                        self.path()
                    );
                }
            } // end while !found_nl_a !begof
        } // end if !found_nl_a !begof

        // may occur in files ending on a single newline
        defo!("C: line.count() is {}", line.count_lineparts());
        if line.count_lineparts() == 0 {
            if self.find_line_lru_cache_enabled {
                self.find_line_lru_cache_put += 1;
                defo!("C: LRU Cache put({}, Done)", fileoffset);
                self.find_line_lru_cache
                    .put(fileoffset, ResultS3LineFind::Done);
            }
            defx!("({}) C: return ResultS3LineFind::Done;", fileoffset);
            return ResultS3LineFind::Done;
        }

        defo!("D: return {:?};", line);
        let fo_end: FileOffset = line.fileoffset_end();
        let linep: LineP = self.insert_line(line);
        if self.find_line_lru_cache_enabled {
            self.find_line_lru_cache_put += 1;
            defo!("D: LRU Cache put({}, Found({}, …))", fileoffset, fo_end + 1);
            self.find_line_lru_cache
                .put(fileoffset, ResultS3LineFind::Found((fo_end + 1, linep.clone())));
        }
        defx!(
            "({}) D: return ResultS3LineFind::Found(({}, LineP)) @[{}, {}] {:?}",
            fileoffset,
            fo_end + 1,
            (*linep).fileoffset_begin(),
            (*linep).fileoffset_end(),
            (*linep).to_String_noraw()
        );

        ResultS3LineFind::Found((fo_end + 1, linep))
    }

    pub fn summary(&self) -> SummaryLineReader {
        let linereader_lines = self
            .count_lines_processed();
        let linereader_lines_stored_highest = self
            .lines_stored_highest();
        let linereader_lines_hits = self
            .lines_hits;
        let linereader_lines_miss = self
            .lines_miss;
        let linereader_find_line_lru_cache_hit = self
            .find_line_lru_cache_hit;
        let linereader_find_line_lru_cache_miss = self
            .find_line_lru_cache_miss;
        let linereader_find_line_lru_cache_put = self
            .find_line_lru_cache_put;
        let linereader_drop_line_ok = self
            .drop_line_ok;
        let linereader_drop_line_errors = self
            .drop_line_errors;

        SummaryLineReader {
            linereader_lines,
            linereader_lines_stored_highest,
            linereader_lines_hits,
            linereader_lines_miss,
            linereader_find_line_lru_cache_hit,
            linereader_find_line_lru_cache_miss,
            linereader_find_line_lru_cache_put,
            linereader_drop_line_ok,
            linereader_drop_line_errors,
        }
    }
}
