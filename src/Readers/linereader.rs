// src/Readers/linereader.rs
//

use crate::common::{
    Count,
    FPath,
    FileOffset,
    FileType,
    FileSz,
    CharSz,
    NLu8,
};

use crate::common::{
    ResultS4,
};

use crate::Data::line::{
    Line,
    LineP,
    LinePart,
    Lines,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    BlockP,
    BlockReader,
    ResultS3_ReadBlock,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::printers::{
    byte_to_char_noraw,
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
    HashSet,
};
use std::fmt;
use std::io::{
    Error,
    Result,
};
use std::sync::Arc;

extern crate debug_print;
use debug_print::debug_eprintln;
#[allow(unused_imports)]
use debug_print::{
    debug_eprint,
    debug_print,
    debug_println
};

extern crate lru;
use lru::LruCache;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate more_asserts;
use more_asserts::{
    assert_ge,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// storage for Lines found from the underlying `BlockReader`
/// FileOffset key is the first byte/offset that begins the `Line`
pub type FoToLine = BTreeMap<FileOffset, LineP>;
pub type FoToFo = BTreeMap<FileOffset, FileOffset>;

/// `LineReader.find_line()` searching results
#[allow(non_camel_case_types)]
pub type ResultS4_LineFind = ResultS4<(FileOffset, LineP), Error>;
pub type LinesLRUCache = LruCache<FileOffset, ResultS4_LineFind>;

/// Specialized reader that uses `BlockReader` to find `Lines` in a file.
///
/// The `LineReader` does much `[u8]` to `char` interpretation. It does the most
/// work in this regard (`SyslineReader` does a little).
///
/// A `LineReader` stores past lookups of data.
///
/// XXX: not a rust "Reader"; does not implement trait `Read`
pub struct LineReader {
    pub(crate) blockreader: BlockReader,
    /// track `Line` found among blocks in `blockreader`, tracked by line beginning `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_begin()`
    pub lines: FoToLine,
    /// internal stats - hits in `find_line()` and other
    pub(crate) _lines_hits: Count,
    /// internal stats - misses in `find_line()` and other
    pub(crate) _lines_miss: Count,
    /// for all `Lines`, map `Line.fileoffset_end` to `Line.fileoffset_beg`
    foend_to_fobeg: FoToFo,
    /// count of `Line`s processed.
    ///
    /// Distinct from `self.lines.len()` as that may have contents removed when --streaming
    pub (crate) lines_processed: Count,
    /// smallest size character in bytes
    /// TODO: handle char sizes > 1 byte, multi-byte encodings
    charsz_: CharSz,
    /// enable internal LRU cache for `find_line` (default `true`)
    find_line_lru_cache_enabled: bool,
    /// internal LRU cache for `find_line`
    /// TODO: remove `pub(crate)`
    pub(crate) find_line_lru_cache: LinesLRUCache,
    /// internal LRU cache count of lookup hit
    pub(crate) find_line_lru_cache_hit: Count,
    /// internal LRU cache count of lookup miss
    pub(crate) find_line_lru_cache_miss: Count,
    /// internal LRU cache count of `.put`
    pub(crate) find_line_lru_cache_put: Count,
    /// count of Ok to Arc::try_unwrap(linep), effectively count of dropped `Line`
    pub(crate) drop_line_ok: Count,
    /// count of failures to Arc::try_unwrap(linep). A failure does not mean an error
    pub(crate) drop_line_errors: Count,
}

impl fmt::Debug for LineReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("LineReader")
            //.field("@", format!("{:p}", &self))
            .field("LRU cache enabled?", &self.find_line_lru_cache_enabled)
            .field("charsz", &self.charsz())
            .field("lines", &self.lines)
            .field("blockreader", &self.blockreader)
            .finish()
    }
}

// XXX: cannot place these within `impl LineReader`?
/// minimum char storage size in bytes
const CHARSZ_MIN: CharSz = 1;
/// maximum char storage size in bytes
const CHARSZ_MAX: CharSz = 4;
/// default char storage size in bytes
/// XXX: does not handle multi-byte encodings (e.g. UTF-8) or multi-byte character storage (e.g. UTF-32)
const CHARSZ: CharSz = CHARSZ_MIN;

/// implement the LineReader things
impl LineReader {
    const FIND_LINE_LRC_CACHE_SZ: usize = 8;
    /// `LineReader::blockzero_analysis` must find at least this many `Line` within
    /// block zero (first block) for the file to be considered a text file.
    /// If the file has only one block then different considerations apply.
    ///

    pub fn new(path: FPath, filetype: FileType, blocksz: BlockSz) -> Result<LineReader> {
        debug_eprintln!("{}LineReader::new({:?}, {:?}, {:?})", snx(), path, filetype, blocksz);
        // XXX: multi-byte
        assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        assert_ne!(blocksz, 0, "BlockSz is zero");
        let blockreader = BlockReader::new(path, filetype, blocksz)?;
        Ok(
            LineReader {
                blockreader,
                lines: FoToLine::new(),
                _lines_hits: 0,
                _lines_miss: 0,
                foend_to_fobeg: FoToFo::new(),
                lines_processed: 0,
                charsz_: CHARSZ,
                find_line_lru_cache_enabled: true,
                find_line_lru_cache: LinesLRUCache::new(LineReader::FIND_LINE_LRC_CACHE_SZ),
                find_line_lru_cache_hit: 0,
                find_line_lru_cache_miss: 0,
                find_line_lru_cache_put: 0,
                drop_line_ok: 0,
                drop_line_errors: 0,
            }
        )
    }

    /// smallest size character in bytes
    #[inline(always)]
    pub const fn charsz(&self) -> usize {
        self.charsz_
    }

    /// `Block` size in bytes
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    /// File Size in bytes
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.blockreader.filesz()
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.blockreader.filetype()
    }

    /// File path
    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        &self.blockreader.path
    }

    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.blockreader.mimeguess()
    }

    /// enable internal LRU cache used by `find_line`
    #[allow(dead_code)]
    pub fn LRU_cache_enable(&mut self) {
        if self.find_line_lru_cache_enabled {
            return;
        }
        self.find_line_lru_cache_enabled = true;
        self.find_line_lru_cache.clear();
        self.find_line_lru_cache.resize(LineReader::FIND_LINE_LRC_CACHE_SZ);
    }

    /// disable internal LRU cache used by `find_line`
    /// intended for testing
    pub fn LRU_cache_disable(&mut self) {
        self.find_line_lru_cache_enabled = false;
        self.find_line_lru_cache.resize(0);
    }

    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    ///
    /// Testing helper only
    // TODO: [2022/07] remove this
    #[cfg(any(debug_assertions, test))]
    pub fn print(&self, fileoffset: &FileOffset) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let linep = &self.lines[fileoffset];
        linep.print(true);
        true
    }

    /// count of lines processed by this `LineReader` (i.e. `self.lines_processed`)
    #[inline(always)]
    pub fn count_lines_processed(&self) -> Count {
        self.lines_processed
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    #[inline(always)]
    pub const fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    #[inline(always)]
    #[allow(dead_code)]
    pub const fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    #[inline(always)]
    pub const fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// return block index at given `FileOffset`
    #[inline(always)]
    pub const fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    #[inline(always)]
    pub const fn count_blocks(&self) -> Count {
        BlockReader::count_blocks(self.filesz(), self.blocksz()) as Count
    }

    /// last valid `BlockOffset` for the file (inclusive)
    /// (expected largest `BlockOffset` value, no relation to `Block`s processed)
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader.blockoffset_last()
    }

    /// get the last byte index of the file
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.blockreader.fileoffset_last()
    }

    /// is `FileOffset` the last byte of the file?
    pub const fn is_FileOffset_last(&self, fileoffset: FileOffset) -> bool {
        self.fileoffset_last() == fileoffset
    }

    /// is `Line` the last of the file?
    pub fn is_Line_last(&self, linep: &LineP) -> bool {
        self.is_FileOffset_last((*linep).fileoffset_end())
    }

    /// return all currenty stored `FileOffset` in `self.lines`
    ///
    /// only intended to aid testing
    pub fn get_fileoffsets(&self) -> Vec<FileOffset> {
        self.lines.keys().cloned().collect()
    }

    /// store information about a single line in a file
    /// returns a `Line` pointer `LineP`
    ///
    /// should only be called by `self.find_line` and `self.find_line_in_block`
    fn insert_line(&mut self, line: Line) -> LineP {
        debug_eprintln!("{}LineReader.insert_line(Line @{:p})", sn(), &line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let linep = LineP::new(line);
        debug_eprintln!("{}LineReader.insert_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*linep));
        debug_assert!(!self.lines.contains_key(&fo_beg), "self.lines already contains key {}", fo_beg);
        self.lines.insert(fo_beg, linep.clone());
        debug_eprintln!("{}LineReader.insert_line: foend_to_fobeg.insert({}, {})", so(), fo_end, fo_beg);
        debug_assert!(!self.foend_to_fobeg.contains_key(&fo_end), "self.foend_to_fobeg already contains key {}", fo_end);
        self.foend_to_fobeg.insert(fo_end, fo_beg);
        self.lines_processed += 1;
        debug_eprintln!("{}LineReader.insert_line() returning @{:p}", sx(), linep);

        linep
    }

    pub fn drop_lines(&mut self, lines: Lines, bo_dropped: &mut HashSet<BlockOffset>) {
        debug_eprintln!("{}linereader.drop_lines", sn());
        for linep in lines.into_iter() {
            self.drop_line(linep, bo_dropped);
        }
        debug_eprintln!("{}linereader.drop_lines", sx());
    }

    pub fn drop_line(&mut self, linep: LineP, bo_dropped: &mut HashSet<BlockOffset>) {
        let fo_key = (*linep).fileoffset_begin();
        self.find_line_lru_cache.pop(&fo_key);
        self.lines.remove(&fo_key);
        match Arc::try_unwrap(linep) {
            Ok(line) => {
                debug_eprintln!("{}linereader.drop_line: Arc::try_unwrap(linep) processing Line @[{}‥{}] Block @[{}‥{}]", sn(), line.fileoffset_begin(), line.fileoffset_end(), line.blockoffset_first(), line.blockoffset_last());
                self.drop_line_ok += 1;
                for linepart in line.lineparts.into_iter() {
                    self.blockreader.drop_block(linepart.blockoffset(), bo_dropped);
                }
            }
            Err(_linep) => {
                debug_eprintln!("{}linereader.drop_line: Arc::try_unwrap(linep) Err strong_count {}", sn(), Arc::strong_count(&_linep));
                self.drop_line_errors += 1;
            }
        }
        debug_eprintln!("{}linereader.drop_lines", sx());
    }

    /// does `self` "contain" this `fileoffset`? That is, already know about it?
    /// the `fileoffset` can be any value (does not have to be begining or ending of
    /// a `Line`).
    fn lines_contains(&self, fileoffset: &FileOffset) -> bool {
        let fo_beg = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return false; },
        };
        if fileoffset < fo_beg {
            return false;
        }
        self.lines.contains_key(fo_beg)
    }

    /// for any `FileOffset`, get the `Line` (if available)
    /// The passed `FileOffset` can be any value (does not have to be begining or ending of
    /// a `Line`).
    /// O(log(n))
    // XXX: this fails `pub(in crate::Readers::linereader_tests)`
    pub fn get_linep(&self, fileoffset: &FileOffset) -> Option<LineP> {
        // I'm somewhat sure this is O(log(n))
        let fo_beg: &FileOffset = match self.foend_to_fobeg.range(fileoffset..).next() {
            Some((_, fo_beg_)) => {
                fo_beg_
            },
            None => { return None; },
        };
        if fileoffset < fo_beg {
            return None;
        }
        #[allow(clippy::manual_map)]
        match self.lines.get(fo_beg) {
            Some(lp) => { Some(lp.clone()) }
            None => { None }
        }
    }

    #[inline(always)]
    fn check_store_LRU(&mut self, fileoffset: FileOffset) -> Option<ResultS4_LineFind> {
        // check LRU cache first (this is very fast)
        if self.find_line_lru_cache_enabled {
            match self.find_line_lru_cache.get(&fileoffset) {
                Some(rlp) => {
                    debug_eprintln!("{}check_store_LRU({}): found LRU cached for offset {}", sn(), fileoffset, fileoffset);
                    self.find_line_lru_cache_hit += 1;
                    // `find_line_lru_cache.get(&fileoffset)` returns refernce so must create new `ResultS4_LineFind` here
                    // and return that
                    match rlp {
                        ResultS4_LineFind::Found(val) => {
                            debug_eprintln!(
                                "{}check_store_LRU({}): return ResultS4_LineFind::Found(({}, …)) @[{}, {}] {:?}",
                                sx(),
                                fileoffset,
                                val.0,
                                val.1.fileoffset_begin(),
                                val.1.fileoffset_end(),
                                val.1.to_String_noraw()
                            );
                            return Some(ResultS4_LineFind::Found((val.0, val.1.clone())));
                        }
                        ResultS4_LineFind::Found_EOF(val) => {
                            debug_eprintln!("{}check_store_LRU({}): return ResultS4_LineFind::Found_EOF(({}, …)) @[{}, {}] {:?}", sx(), fileoffset, val.0, val.1.fileoffset_begin(), val.1.fileoffset_end(), val.1.to_String_noraw());
                            return Some(ResultS4_LineFind::Found_EOF((val.0, val.1.clone())));
                        }
                        ResultS4_LineFind::Done => {
                            debug_eprintln!("{}check_store_LRU({}): return ResultS4_LineFind::Done", sx(), fileoffset);
                            return Some(ResultS4_LineFind::Done);
                        }
                        ResultS4_LineFind::Err(err) => {
                            debug_eprintln!("{}check_store_LRU({}): Err {}", sx(), fileoffset, err);
                            eprintln!("ERROR: unexpected Error store in find_line_lru_cache, fileoffset {}", fileoffset);
                        }
                    }
                }
                None => {
                    self.find_line_lru_cache_miss += 1;
                    debug_eprintln!("{}find_line: fileoffset {} not found in LRU cache", snx(), fileoffset);
                }
            }
        }

        None
    }

    #[inline(always)]
    fn check_store(&mut self, fileoffset: FileOffset) -> Option<ResultS4_LineFind> {
        // TODO: [2022/06/18] add a counter for hits and misses for `self.lines`
        let charsz_fo: FileOffset = self.charsz_ as FileOffset;
        // search containers of `Line`s
        // first, check if there is a `Line` already known at this fileoffset
        if self.lines.contains_key(&fileoffset) {
            self._lines_hits += 1;
            debug_eprintln!("{}check_store: hit self.lines for FileOffset {}", so(), fileoffset);
            debug_assert!(self.lines_contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
            let linep = self.lines[&fileoffset].clone();
            let fo_next = (*linep).fileoffset_end() + charsz_fo;
            if self.is_Line_last(&linep) {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}check_store: LRU Cache put({}, Found_EOF({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                }
                debug_eprintln!("{}check_store({}): return ResultS4_LineFind::Found_EOF({}, {:p}) @[{}, {}] {:?}", so(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return Some(ResultS4_LineFind::Found_EOF((fo_next, linep)));    
            }
            if self.find_line_lru_cache_enabled {
                self.find_line_lru_cache_put += 1;
                debug_eprintln!("{}check_store: LRU Cache put({}, Found({}, …))", so(), fileoffset, fo_next);
                self.find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
            }
            debug_eprintln!("{}check_store({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", so(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
            return Some(ResultS4_LineFind::Found((fo_next, linep)));
        } else {
            self._lines_miss += 1;
        }
        // second, check if there is a `Line` at a preceding offset
        match self.get_linep(&fileoffset) {
            Some(linep) => {
                debug_eprintln!(
                    "{}check_store: self.get_linep({}) returned @{:p}",
                    sn(),
                    fileoffset,
                    linep,
                );
                // XXX: does not handle multi-byte
                let fo_next = (*linep).fileoffset_end() + charsz_fo;
                if self.is_Line_last(&linep) {
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        debug_eprintln!("{}check_store: LRU Cache put({}, Found_EOF({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                    }
                    debug_eprintln!("{}check_store({}): return ResultS4_LineFind::Found_EOF({}, {:p}) @[{}, {}] {:?}", so(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                    return Some(ResultS4_LineFind::Found_EOF((fo_next, linep)));    
                }
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}check_store: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}check_store({}): return ResultS4_LineFind::Found({}, {:p}) @[{}, {}] {:?}", so(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return Some(ResultS4_LineFind::Found((fo_next, linep)));
            }
            None => {
                debug_eprintln!("{}check_store: fileoffset {} not found in self.lines_by_range", so(), fileoffset);
            }
        }
        debug_eprintln!("{}check_store: fileoffset {} not found in self.lines", so(), fileoffset);

        None
    }

    /// Find the `Line` at `fileoffset` within the same `Block`.
    /// This does a linear search over the `Block`, O(n).
    ///
    /// If a `Line` extends before or after the `Block` then `Done` is returned.
    ///
    /// Returned `ResultS4_LineFind(fileoffset, …)` may refer to a different
    /// proceeding `Block`.
    ///
    /// TODO: [2022/05] add test for this:
    ///       Keep in mind, a `Line` with terminating-newline as the last byte a `Block`
    ///       may be allowed. However, a `Line` with preceding `Line` newline in prior `Block`
    ///       may not be found, since the preceding `Line` terminating-newline must be found.
    ///       In other words, last byte of `Line` may be last byte of `Block` and the `Line`
    ///       will be found. However, if first byte of `Line` is first byte of `Block` then
    ///       it will not be found.
    ///
    /// XXX: This function `find_line` is large and cumbersome and needs some cleanup of warnings.
    ///      It could definitely use some improvements, but for now it gets the job done.
    ///      Changes require extensive retesting.
    ///      You've been warned.
    pub fn find_line_in_block(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line_in_block({})", sn(), fileoffset);

        // some helpful constants
        let charsz_fo = self.charsz_ as FileOffset;
        let charsz_bi = self.charsz_ as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // XXX: using cache can result in non-idempotent behavior
        // check fast LRU
        if let Some(results4) = self.check_store_LRU(fileoffset) {
            debug_eprintln!("{}find_line_in_block({}): return {:?}", sx(), fileoffset, results4);
            return results4;
        }

        // handle special cases
        if filesz == 0 {
            debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Done; file is empty", sx(), fileoffset);
            return ResultS4_LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Done; fileoffset {} was too big filesz {}!", sx(), fileoffset, fileoffset, filesz);
            return ResultS4_LineFind::Done;
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Done(); fileoffset {} is at end of file {}!", sx(), fileoffset, fileoffset, filesz);
            return ResultS4_LineFind::Done;
        }

        // XXX: using cache can result in non-idempotent behavior
        // check container of `Line`s
        if let Some(results4) = self.check_store(fileoffset) {
            debug_eprintln!("{}find_line_in_block({}): return {:?}", sx(), fileoffset, results4);
            return results4;
        }

        //
        // could not find `fileoffset` from prior saved information so...
        // walk through blocks and bytes looking for beginning of a line (a newline character)
        // start with newline search "part B" (look for line terminating '\n' or end of file)
        // then do search "part A" (look for line terminating '\n' of previous Line or beginning
        // of file)
        //

        debug_eprintln!("{}find_line_in_block({}): searching for first newline B (line terminator) …", so(), fileoffset);

        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // found newline part B? Line ends at this.
        let mut found_nl_b: bool = false;
        // `fo_nl_a` should eventually "point" to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = fileoffset;
        // `fo_nl_b` should eventually "point" to end of `Line` including the newline char.
        // if  line is terminated by end-of-file then "points" to last char of file.
        let mut fo_nl_b: FileOffset = fileoffset;
        let mut bi_nl_b: BlockIndex;
        let mut fo_nl_b_in_middle: bool = false;
        // was newline B actually the end of file?
        let mut nl_b_eof: bool = false;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            debug_eprintln!("{}find_line_in_block({}): newline A0 is {} because fileoffset {} is beginning of file!", so(), fileoffset, fo_nl_a, fileoffset);
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
        let bptr_middle: BlockP = match self.blockreader.read_block(bo_middle) {
            ResultS3_ReadBlock::Found(val) => {
                debug_eprintln!(
                    "{}find_line_in_block({}) B1: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                    so(),
                    fileoffset,
                    bo_middle,
                    &(*val),
                    (*val).len()
                );
                val
            },
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}find_line_in_block({}) B1: read_block({}) returned Done {:?}", sx(), fileoffset, bo_middle, self.path());
                return ResultS4_LineFind::Done;
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}find_line_in_block({}) B1: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bo_middle, err);
                return ResultS4_LineFind::Err(err);
            }
        };

        let mut bi_at: BlockIndex = bi_middle;
        let bi_stop: BlockIndex = bptr_middle.len() as BlockIndex;
        assert_ge!(bi_stop, charsz_bi, "bi_stop is less than charsz; not yet handled");

        // XXX: multi-byte
        //bi_beg = bi_stop - charsz_bi;
        debug_eprintln!("{}find_line_in_block({}) B1: scan middle block {} forwards, starting from blockindex {} (fileoffset {}) searching for newline B",
            so(),
            fileoffset,
            bo_middle,
            bi_at,
            self.file_offset_at_block_offset_index(bo_middle, bi_at)
        );
        loop {
            // XXX: single-byte encoding
            if (*bptr_middle)[bi_at] == NLu8 {
                found_nl_b = true;
                fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                bi_nl_b = bi_at;
                bi_middle_end = bi_at;
                debug_eprintln!("{}find_line_in_block({}) B1: bi_middle_end {:?} bi_nl_b {:?} fo_nl_b {:?}", so(), fileoffset, bi_middle_end, bi_nl_b, fo_nl_b);
                fo_nl_b_in_middle = true;
                debug_eprintln!(
                    "{}find_line_in_block({}) B1: found newline B in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                    so(),
                    fileoffset,
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
        }  // end loop
        
        // if (newline B not found and the "middle" block was the last block) then eof is newline B
        if !found_nl_b && bo_middle == blockoffset_last {
            found_nl_b = true;
            assert_ge!(bi_at, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B1 at end of file for file {:?}", bi_at, charsz_bi, self.path());
            let bi_ = bi_at - charsz_bi;
            fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_);
            bi_nl_b = bi_;
            bi_middle_end = bi_;
            debug_eprintln!(
                "{}find_line_in_block({}) B1: bi_middle_end {:?} bi_nl_b {:?} fo_nl_b {:?} blockoffset_last {:?}",
                so(),
                fileoffset,
                bi_middle_end,
                bi_nl_b,
                fo_nl_b,
                blockoffset_last,
            );
            fo_nl_b_in_middle = true;
            nl_b_eof = true;
            assert_eq!(
                fo_nl_b, filesz - 1,
                "newline B1 fileoffset {} is at end of file, yet filesz is {}; there was a bad calcuation of newline B1 from blockoffset {} blockindex {} (blockoffset last {}), for file {:?}",
                fo_nl_b,
                filesz,
                bo_middle,
                bi_,
                blockoffset_last,
                self.path(),
            );
        } else if found_nl_b && self.is_FileOffset_last(fo_nl_b) {
            assert_eq!(
                bo_middle, blockoffset_last,
                "blockoffset 'middle' {}, blockoffset last {}, yet newline B FileOffset {} is last byte of filesz {}, for file {:?}",
                bo_middle, blockoffset_last, fo_nl_b, self.filesz(), self.path(),
            );
            nl_b_eof = true;
        }
        if !found_nl_b {
            debug_eprintln!("{}find_line_in_block({}): failed to find newline B in block {} return Done {:?}", sx(), fileoffset, bo_middle, self.path());
            return ResultS4_LineFind::Done;
        }

        debug_eprintln!(
            "{}find_line_in_block({}): found first newline B at FileOffset {}, searching for preceding newline A. Search starts at FileOffset {} …",
            so(),
            fileoffset,
            fo_nl_b,
            fileoffset,
        );

        // if found_nl_a was already found then this function can return
        if found_nl_a {
            debug_eprintln!("{}find_line_in_block({}) A0: already found newline A and newline B, return early", so(), fileoffset);
            assert_eq!(fo_nl_a, 0, "newline A is {}, only reason newline A should be found at this point was if passed fileoffset 0, (passed fileoffset {}), for file {:?}", fo_nl_a, fileoffset, self.path());
            let li = LinePart::new(
                bptr_middle,
                self.block_index_at_file_offset(fo_nl_a),
                bi_middle_end + 1,
                fo_nl_a,
                self.block_offset_at_file_offset(fo_nl_a),
                self.blocksz(),
            );
            line.prepend(li);
            let linep = self.insert_line(line);
            let fo_next = fo_nl_b + charsz_fo;
            debug_assert_eq!(fo_next, (*linep).fileoffset_end() + charsz_fo, "mismatching fo_next {} != (*linep).fileoffset_end()+1, for file {:?}", fo_next, self.path());
            if !nl_b_eof {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}find_line_in_block({}) A0: LRU cache put({}, Found(({}, @{:p})))", so(), fileoffset, fileoffset, fo_next, linep);
                    self.find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line_in_block({}) A0: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, linep.clone()));
            } else {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}find_line_in_block({}) A0: LRU cache put({}, Found_EOF(({}, @{:p})))", so(), fileoffset, fileoffset, fo_next, linep);
                    self.find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line_in_block({}) A0: return ResultS4_LineFind::Found_EOF(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found_EOF((fo_next, linep.clone()));
            };
        }
        assert!(!found_nl_a, "already found newline A; was finding it once not good enough? {:?}", self.path());
        assert!(found_nl_b, "found newline A, have not found newline B; bird with one wing. {:?}", self.path());

        if fileoffset >= charsz_fo {
            let fo_ = fileoffset - charsz_fo;
            if self.lines.contains_key(&fo_) {
                self._lines_hits += 1;
                debug_eprintln!("{}find_line_in_block({}) A1a: hit in self.lines for FileOffset {} (before part A)", so(), fileoffset, fo_);
                fo_nl_a = fo_;
                let linep_prev = self.lines[&fo_nl_a].clone();
                assert_eq!(
                    fo_nl_a, (*linep_prev).fileoffset_end(),
                    "get_linep({}) returned Line with fileoffset_end() {}; these should match for file {:?}",
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                    self.path(),
                );
                let li = LinePart::new(
                    bptr_middle,
                    self.block_index_at_file_offset(fileoffset),
                    bi_middle_end + 1,
                    fileoffset,
                    self.block_offset_at_file_offset(fileoffset),
                    self.blocksz(),
                );
                line.prepend(li);
                let linep = self.insert_line(line);
                let fo_next = fo_nl_b + charsz_fo;
                if nl_b_eof {
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        debug_eprintln!("{}find_line_in_block({}) A1a: LRU Cache put({}, Found_EOF({}, …)) {:?}", so(), fileoffset, fileoffset, fo_next, (*linep).to_String_noraw());
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                    }
                    debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Found_EOF({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                    return ResultS4_LineFind::Found_EOF((fo_next, linep));
    
                }
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}find_line_in_block({}) A1a: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fileoffset, fo_next, (*linep).to_String_noraw());
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, linep));
            } else {
                self._lines_miss += 1;
                debug_eprintln!("{}find_line_in_block({}) A1a: miss in self.lines for FileOffset {} (quick check before part A)", so(), fileoffset, fo_);
            }

            match self.get_linep(&fo_) {
                Some(linep_prev) => {
                    debug_eprintln!(
                        "{}find_line_in_block({}) A1b: self.get_linep({}) returned {:p}",
                        so(),
                        fileoffset,
                        fo_,
                        linep_prev,
                    );
                    found_nl_a = true;
                    fo_nl_a = (*linep_prev).fileoffset_end();
                    assert_eq!(
                        fo_nl_a, fo_,
                        "get_linep({}) returned Line with fileoffset_end() {}; these should match for file {:?}",
                        fo_,
                        fo_nl_a,
                        self.path(),
                    );
                    let li = LinePart::new(
                        bptr_middle,
                        self.block_index_at_file_offset(fileoffset),
                        bi_middle_end + 1,
                        fileoffset,
                        self.block_offset_at_file_offset(fileoffset),
                        self.blocksz(),
                    );
                    line.prepend(li);
                    let linep = self.insert_line(line);
                    let fo_next = fo_nl_b + charsz_fo;
                    if nl_b_eof {
                        debug_assert!(self.is_Line_last(&linep), "nl_b_eof true yet !is_Line_last(linep) file {:?}", self.path());
                        if self.find_line_lru_cache_enabled {
                            self.find_line_lru_cache_put += 1;
                            debug_eprintln!("{}find_line_in_block({}) A1b: LRU Cache put({}, Found_EOF({}, …)) {:?}", so(), fileoffset, fileoffset, fo_next, (*linep).to_String_noraw());
                            self.find_line_lru_cache
                                .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                        }
                        debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Found_EOF({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                        return ResultS4_LineFind::Found_EOF((fo_next, linep));    
                    }
                    debug_assert!(!self.is_Line_last(&linep), "nl_b_eof true yet !is_Line_last(linep)");
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        debug_eprintln!("{}find_line_in_block({}) A1b: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fileoffset, fo_next, (*linep).to_String_noraw());
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                    }
                    debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                    return ResultS4_LineFind::Found((fo_next, linep));
                },
                None => {
                    debug_eprintln!("{}find_line_in_block({}) A1b: self.get_linep({}) returned None (quick check before part A)", so(), fileoffset, fo_);
                },
            }
        }

        //
        // getting here means this function is discovering a brand new `Line` (searching for newline A)
        // walk *backwards* to find line-terminating newline of the preceding line (or beginning of file)
        //

        let fo_nl_a_search_start = std::cmp::max(fileoffset, charsz_fo) - charsz_fo;
        let bof: BlockOffset = self.block_offset_at_file_offset(fo_nl_a_search_start);
        let mut begof: bool = false;  // run into beginning of file (as in first byte)?
        // newline A plus one (one charsz past preceding Line terminating '\n')
        let mut fo_nl_a1: FileOffset = 0;

        if bof != bo_middle {
            debug_eprintln!("{}find_line_in_block({}): failed to find newline A within block {} return Done {:?}", sx(), fileoffset, bo_middle, self.path());
            return ResultS4_LineFind::Done;
        }

        // search for newline A starts within "middle" block
        let mut bi_at: BlockIndex = self.block_index_at_file_offset(fo_nl_a_search_start);
        let bi_stop: BlockIndex = 0;
        debug_eprintln!(
            "{}find_line_in_block({}) A2a: scan middle block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
            so(),
            fileoffset,
            bo_middle,
            bi_at,
            self.file_offset_at_block_offset_index(bo_middle, bi_at),
            bi_stop,
        );
        loop {
            // XXX: single-byte encoding
            if (*bptr_middle)[bi_at] == NLu8 {
                found_nl_a = true;
                fo_nl_a = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                debug_eprintln!(
                    "{}find_line_in_block({}) A2a: found newline A in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                    so(),
                    fileoffset,
                    bo_middle,
                    bi_at,
                    fo_nl_a,
                    byte_to_char_noraw((*bptr_middle)[bi_at]),
                );
                // adjust offsets one forward
                // XXX: single-byte encoding
                fo_nl_a1 = fo_nl_a + charsz_fo;
                bi_at += charsz_bi;
                break;
            }
            if bi_at == 0 {
                break;
            }
            // XXX: single-byte encoding
            bi_at -= charsz_bi;
            if bi_at < bi_stop {
                break;
            }
        }

        if bof == 0 {
            debug_eprintln!("{}find_line_in_block({}) A2a: run into beginning of file", so(), fileoffset);
            begof = true;
        }
        if !found_nl_a && begof {
            found_nl_a = true;
            fo_nl_a = 0;
            fo_nl_a1 = 0;
        }
        if !found_nl_a {
            debug_eprintln!("{}find_line_in_block({}) A2a: newline A not found in middle block {}", so(), fileoffset, bo_middle);
            debug_eprintln!("{}find_line_in_block({}): return Done {:?}", sx(), fileoffset, self.path());
            return ResultS4_LineFind::Done;
        }

        let li = LinePart::new(
            bptr_middle.clone(),
            bi_at,
            bi_middle_end + 1,
            fo_nl_a1,
            bo_middle,
            self.blocksz(),
        );
        line.prepend(li);

        let linep = LineP::new(line);
        let fo_next: FileOffset = fo_nl_b + charsz_fo;
        if nl_b_eof {
            debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Found_EOF({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
            return ResultS4_LineFind::Found_EOF((fo_next, linep));
        }

        debug_eprintln!("{}find_line_in_block({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());

        ResultS4_LineFind::Found((fo_next, linep))
    }

    /// Find next `Line` starting from `fileoffset`.
    /// This does a linear search over the file, O(n).
    ///
    /// During the process of finding, creates and stores the `Line` from underlying `Block` data.
    /// Returns `Found`(`FileOffset` of beginning of the _next_ line, found `LineP`)
    /// Reaching end of file (and no new line) returns `Found_EOF`.
    /// Reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used).
    /// Otherwise returns `Err`, all other `Result::Err` errors are propagated.
    ///
    /// This function has the densest number of byte↔char handling and transitions within this program.
    ///
    /// Correllary to `find_sysline`, `read_block`.
    ///
    /// Throughout this function, newline A points to the line beginning, newline B
    /// points to line ending. Both are inclusive.
    ///
    /// Here are two defining cases of this function:
    ///
    /// given a file of four newlines:
    ///
    /// ```
    ///     byte: 0123
    ///     char: ␊␊␊␊
    /// ```
    ///
    /// calls to `find_line` would result in a `Line`
    ///
    /// ```
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
    /// given a file with two alphabet chars and one newline:
    ///
    /// ```
    ///     012
    ///     x␊y
    ///
    ///                     A,B  Val
    ///     fine_line(0) -> 0,1 "x␊"
    ///     fine_line(1) -> 0,1 "x␊"
    ///     fine_line(2) -> 2,2 "y"
    /// ```
    ///
    /// XXX: presumes a single-byte can represent a '\n'; i.e. does not handle UTF-16 or UTF-32 or other.
    ///
    /// TODO: [2021/08/30] handle different encodings
    ///
    /// XXX: returning the "next fileoffset (along with `LineP`) is jenky. Just return the `LineP`.
    ///      and/or do not return "fileoffset next" for `Found_EOF` (doesn't make sense).
    ///      and/or add `iter` capabilities to `Line` that will hide tracking the "next fileoffset".
    ///
    /// XXX: This function `find_line` is large and cumbersome and needs some cleanup of warnings.
    ///      It could definitely use some improvements, but for now it gets the job done.
    ///      Changes require extensive retesting.
    ///      You've been warned.
    ///
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self.charsz_ as FileOffset;
        let charsz_bi = self.charsz_ as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();

        // check fast LRU first
        if let Some(results4) = self.check_store_LRU(fileoffset) {
            debug_eprintln!("{}find_line({}): return {:?}", sx(), fileoffset, results4);
            return results4;
        }

        // handle special cases
        if filesz == 0 {
            debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Done; file is empty", sx(), fileoffset);
            return ResultS4_LineFind::Done;
        } else if fileoffset > filesz {
            // TODO: [2021/10] need to decide on consistent behavior for passing fileoffset > filesz
            //       should it really Error or be Done?
            //       Make that consisetent among all LineReader and SyslineReader `find_*` functions
            debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Done; fileoffset {} was too big filesz {}!", sx(), fileoffset, fileoffset, filesz);
            return ResultS4_LineFind::Done;
        } else if fileoffset == filesz {
            debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Done(); fileoffset {} is at end of file {}!", sx(), fileoffset, fileoffset, filesz);
            return ResultS4_LineFind::Done;
        }

        // check container of `Line`s
        if let Some(results4) = self.check_store(fileoffset) {
            debug_eprintln!("{}find_line({}): return {:?}", sx(), fileoffset, results4);
            return results4;
        }

        //
        // could not find `fileoffset` from prior saved information so...
        // walk through blocks and bytes looking for beginning of a line (a newline character)
        // start with newline search "part B" (look for line terminating '\n' or end of file)
        // then do search "part A" (look for line terminating '\n' of previous Line or beginning
        // of file)
        //

        debug_eprintln!("{}find_line: searching for first newline B (line terminator) …", so());

        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // found newline part B? Line ends at this.
        let mut found_nl_b: bool = false;
        // `fo_nl_a` should eventually "point" to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = fileoffset;
        // `fo_nl_b` should eventually "point" to end of `Line` including the newline char.
        // if  line is terminated by end-of-file then "points" to last char of file.
        let mut fo_nl_b: FileOffset = fileoffset;
        let mut bi_nl_b: BlockIndex;
        let mut fo_nl_b_in_middle: bool = false;
        // was newline B actually the end of file?
        let mut nl_b_eof: bool = false;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            debug_eprintln!("{}find_line: newline A0 is {} because fileoffset {} is beginning of file!", so(), fo_nl_a, fileoffset);
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
        {  // arbitrary statement block
            bptr_middle = match self.blockreader.read_block(bo_middle) {
                ResultS3_ReadBlock::Found(val) => {
                    debug_eprintln!(
                        "{}find_line B1: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                        so(),
                        bo_middle,
                        &(*val),
                        (*val).len()
                    );
                    val
                },
                ResultS3_ReadBlock::Done => {
                    debug_eprintln!("{}find_line B1: read_block({}) returned Done {:?}", so(), bo_middle, self.path());
                    return ResultS4_LineFind::Done;
                },
                ResultS3_ReadBlock::Err(err) => {
                    debug_eprintln!("{}find_line({}) B1: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bo_middle, err);
                    return ResultS4_LineFind::Err(err);
                }
            };
            let mut bi_at: BlockIndex = bi_middle;
            let bi_stop: BlockIndex = bptr_middle.len() as BlockIndex;
            assert_ge!(bi_stop, charsz_bi, "bi_stop is less than charsz; not yet handled");
            // XXX: multi-byte
            //bi_beg = bi_stop - charsz_bi;
            debug_eprintln!("{}find_line B1: scan middle block {} forwards (block len {}), starting from blockindex {} (fileoffset {}) searching for newline B", so(), bo_middle, (*bptr_middle).len(), bi_at, self.file_offset_at_block_offset_index(bo_middle, bi_at));
            loop {
                // XXX: single-byte encoding
                if (*bptr_middle)[bi_at] == NLu8 {
                    found_nl_b = true;
                    fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                    bi_nl_b = bi_at;
                    bi_middle_end = bi_at;
                    debug_eprintln!("{}find_line B1: bi_middle_end {:?} bi_nl_b {:?} fo_nl_b {:?}", so(), bi_middle_end, bi_nl_b, fo_nl_b);
                    fo_nl_b_in_middle = true;
                    debug_eprintln!(
                        "{}find_line B1: found newline B in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                        so(),
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
            }  // end loop
            // if (newline B not found and the "middle" block was the last block) then eof is newline B
            if !found_nl_b && bo_middle == blockoffset_last {
                found_nl_b = true;
                assert_ge!(bi_at, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B1 at end of file {:?}", bi_at, charsz_bi, self.path());
                let bi_ = bi_at - charsz_bi;
                fo_nl_b = self.file_offset_at_block_offset_index(bo_middle, bi_);
                bi_nl_b = bi_;
                bi_middle_end = bi_;
                debug_eprintln!("{}find_line B1: bi_middle_end {:?} bi_nl_b {:?} fo_nl_b {:?} blockoffset_last {:?}", so(), bi_middle_end, bi_nl_b, fo_nl_b, blockoffset_last);
                fo_nl_b_in_middle = true;
                nl_b_eof = true;
                assert_eq!(
                    fo_nl_b, filesz - 1,
                    "newline B1 fileoffset {} is at end of file, yet filesz is {}; there was a bad calcuation of newline B1 from blockoffset {} blockindex {} (blockoffset last {}) for file {:?}",
                    fo_nl_b,
                    filesz,
                    bo_middle,
                    bi_,
                    blockoffset_last,
                    self.path(),
                );
            } else if !found_nl_b {
                bi_middle_end = bi_stop - charsz_bi;
                debug_eprintln!("{}find_line B1: bi_middle_end {:?}", so(), bi_middle_end);
            }
        }

        if found_nl_b {
            debug_eprintln!("{}find_line B2: skip continued backwards search for newline B (already found)", so());
        } else {
            // search within proceeding blocks for newline B
            const BI_UNINIT: BlockIndex = usize::MAX;
            let mut bi_beg: BlockIndex = BI_UNINIT;  // XXX: value BI_UNINIT is a hacky "uninitialized" signal
            let mut bi_end: BlockIndex = BI_UNINIT;  // XXX: value BI_UNINIT is a hacky "uninitialized" signal
            let mut bof = bo_middle + 1;
            while !found_nl_b && bof <= blockoffset_last {
                let bptr: BlockP = match self.blockreader.read_block(bof) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line B2: read_block({}) returned Found Block @{:p} len {} while searching for newline B",
                            so(),
                            bof,
                            &(*val),
                            (*val).len()
                        );
                        val
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!("{}find_line B2: read_block({}) returned Done {:?}", so(), bof, self.path());
                        return ResultS4_LineFind::Done;
                    },
                    ResultS3_ReadBlock::Err(err) => {
                        debug_eprintln!("{}find_line({}) B2: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bof, err);
                        return ResultS4_LineFind::Err(err);
                    },
                };
                bi_beg = 0;
                bi_end = (*bptr).len() as BlockIndex;
                assert_ge!(bi_end, charsz_bi, "blockindex bi_end {} is less than charsz; not yet handled, file {:?}", bi_end, self.path());
                assert_ne!(bi_end, 0, "blockindex bi_end is zero; Block at blockoffset {}, BlockP @0x{:p}, has len() zero", bof, bptr);
                // XXX: multi-byte
                //bi_beg = bi_end - charsz_bi;
                debug_eprintln!(
                    "{}find_line B2: scan block {} forwards, starting from blockindex {} (fileoffset {}) up to blockindex {} searching for newline B",
                    so(),
                    bof,
                    bi_beg,
                    self.file_offset_at_block_offset_index(bof, bi_beg),
                    bi_end,
                );
                loop {
                    // XXX: single-byte encoding
                    if (*bptr)[bi_beg] == NLu8 {
                        found_nl_b = true;
                        fo_nl_b = self.file_offset_at_block_offset_index(bof, bi_beg);
                        bi_nl_b = bi_beg;
                        assert!(!fo_nl_b_in_middle, "fo_nl_b_in_middle should be false, file {:?}", self.path());
                        debug_eprintln!(
                            "{}find_line B2: found newline B during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            so(),
                            bof,
                            bi_beg,
                            fo_nl_b,
                            byte_to_char_noraw((*bptr)[bi_beg]),
                        );
                        let li = LinePart::new(
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
                }  // end loop
                if found_nl_b {
                    break;
                }
                // newline B was not found in this `Block`, but must save this `Block` information as a `LinePart.
                let li = LinePart::new(
                    bptr.clone(),
                    0,
                    bi_beg,
                    self.file_offset_at_block_offset_index(bof, 0),
                    bof,
                    self.blocksz(),
                );
                line.append(li);
                bof += 1;
            }  // end while bof <= blockoffset_last
            // if newline B not found and last checked block was the last block
            // then eof is newline B
            if !found_nl_b && bof > blockoffset_last {
                bof = blockoffset_last;
                found_nl_b = true;
                assert_ne!(bi_beg, BI_UNINIT, "blockindex begin is uninitialized");
                assert_ne!(bi_end, BI_UNINIT, "blockindex end is uninitialized");
                assert_ge!(bi_beg, charsz_bi, "blockindex begin {} is less than charsz {} before attempt to subtract to determine newline B2 at end of file {:?}", bi_beg, charsz_bi, self.path());
                assert_eq!(bi_beg, bi_end, "blockindex begin {} != {} blockindex end, yet entire last block was searched (last blockoffset {}) file {:?}", bi_beg, bi_end, blockoffset_last, self.path());
                let bi_ = bi_beg - charsz_bi;
                fo_nl_b = self.file_offset_at_block_offset_index(bof, bi_);
                bi_nl_b = bi_;
                nl_b_eof = true;
                debug_eprintln!(
                    "{}find_line B2: newline B is end of file; blockoffset {} blockindex {} fileoffset {} (blockoffset last {})",
                    so(),
                    bof,
                    bi_,
                    fo_nl_b,
                    blockoffset_last,
                );
                assert_eq!(
                    fo_nl_b, filesz - 1,
                    "newline B2 fileoffset {} is supposed to be the end of file, yet filesz is {}; bad calcuation of newline B2 from blockoffset {} blockindex {} (last blockoffset {}) (bi_beg {} bi_end {}) (charsz {}) file {:?}",
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
        }  // end if ! found_nl_b

        //
        // walk backwards through blocks and bytes looking for newline A (line terminator of preceding Line or beginning of file)
        //

        debug_eprintln!(
            "{}find_line: found first newline B at FileOffset {}, searching for preceding newline A. Search starts at FileOffset {} …",
            so(),
            fo_nl_b,
            fileoffset,
        );

        // if found_nl_a was already found then this function can return
        if found_nl_a {
            debug_eprintln!("{}find_line A0: already found newline A and newline B, return early", so());
            assert_eq!(fo_nl_a, 0, "newline A is {}, only reason newline A should be found at this point was if passed fileoffset 0, (passed fileoffset {}) file {:?}", fo_nl_a, fileoffset, self.path());
            let li = LinePart::new(
                bptr_middle,
                self.block_index_at_file_offset(fo_nl_a),
                bi_middle_end + 1,
                fo_nl_a,
                self.block_offset_at_file_offset(fo_nl_a),
                self.blocksz(),
            );
            line.prepend(li);
            let linep = self.insert_line(line);
            let fo_next = fo_nl_b + charsz_fo;
            debug_assert_eq!(fo_next, (*linep).fileoffset_end() + charsz_fo, "mismatching fo_next {} != (*linep).fileoffset_end()+1, file {:?}", fo_next, self.path());
            if !nl_b_eof {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}find_line A0: LRU cache put({}, Found(({}, @{:p})))", so(), fileoffset, fo_next, linep);
                    self.find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line({}) A0: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, linep.clone()));
            } else {
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}find_line A0: LRU cache put({}, Found_EOF(({}, @{:p})))", so(), fileoffset, fo_next, linep);
                    self.find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line({}) A0: return ResultS4_LineFind::Found_EOF(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found_EOF((fo_next, linep.clone()));
            };
        }
        assert!(!found_nl_a, "already found newline A; was finding it once not good enough? file {:?}", self.path());
        assert!(found_nl_b, "found newline A, have not found newline B; bird with one wing. file {:?}", self.path());
    
        // …but before doing work of discovering a new `Line` (newline A),
        // check various maps at `fileoffset + 1` to see if the preceding
        // `Line` has already been discovered and processed.
        // This is common for sequential calls to this function.
        if fileoffset >= charsz_fo {
            let fo_ = fileoffset - charsz_fo;
            if self.lines.contains_key(&fo_) {
                self._lines_hits += 1;
                debug_eprintln!("{}find_line A1a: hit in self.lines for FileOffset {} (before part A)", so(), fo_);
                fo_nl_a = fo_;
                let linep_prev = self.lines[&fo_nl_a].clone();
                assert_eq!(
                    fo_nl_a, (*linep_prev).fileoffset_end(),
                    "get_linep({}) returned Line with fileoffset_end() {}; these should match; file {:?}",
                    fo_nl_a,
                    (*linep_prev).fileoffset_end(),
                    self.path(),
                );
                let li = LinePart::new(
                    bptr_middle,
                    self.block_index_at_file_offset(fileoffset),
                    bi_middle_end + 1,
                    fileoffset,
                    self.block_offset_at_file_offset(fileoffset),
                    self.blocksz(),
                );
                line.prepend(li);
                let linep = self.insert_line(line);
                let fo_next = fo_nl_b + charsz_fo;
                if self.find_line_lru_cache_enabled {
                    self.find_line_lru_cache_put += 1;
                    debug_eprintln!("{}find_line A1a: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                    self.find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                }
                debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, linep));
            } else {
                self._lines_miss += 1;
                debug_eprintln!("{}find_line A1a: miss in self.lines for FileOffset {} (quick check before part A)", so(), fo_);
            }
            match self.get_linep(&fo_) {
                Some(linep_prev) => {
                    debug_eprintln!(
                        "{}find_line A1b: self.get_linep({}) returned {:p}",
                        so(),
                        fo_,
                        linep_prev,
                    );
                    found_nl_a = true;
                    fo_nl_a = (*linep_prev).fileoffset_end();
                    assert_eq!(
                        fo_nl_a, fo_,
                        "get_linep({}) returned Line with fileoffset_end() {}; these should match; file {:?}",
                        fo_,
                        fo_nl_a,
                        self.path(),
                    );
                    let li = LinePart::new(
                        bptr_middle,
                        self.block_index_at_file_offset(fileoffset),
                        bi_middle_end + 1,
                        fileoffset,
                        self.block_offset_at_file_offset(fileoffset),
                        self.blocksz(),
                    );
                    line.prepend(li);
                    let linep = self.insert_line(line);
                    let fo_next = fo_nl_b + charsz_fo;
                    if self.find_line_lru_cache_enabled {
                        self.find_line_lru_cache_put += 1;
                        debug_eprintln!("{}find_line A1b: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*linep).to_String_noraw());
                        self.find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found((fo_next, linep.clone())));
                    }
                    debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                    return ResultS4_LineFind::Found((fo_next, linep));
                },
                None => {
                    debug_eprintln!("{}find_line A1b: self.get_linep({}) returned None (quick check before part A)", so(), fo_);
                },
            }
        }

        //
        // getting here means this function is discovering a brand new `Line` (searching for newline A)
        // walk *backwards* to find line-terminating newline of the preceding line (or beginning of file)
        //

        let fo_nl_a_search_start = std::cmp::max(fileoffset, charsz_fo) - charsz_fo;
        let mut bof: BlockOffset = self.block_offset_at_file_offset(fo_nl_a_search_start);
        let mut begof: bool = false;  // run into beginning of file (as in first byte)?
        // newline A plus one (one charsz past preceding Line terminating '\n')
        let mut fo_nl_a1: FileOffset = 0;

        if bof == bo_middle {
            // search for newline A starts within "middle" block
            let mut bi_at: BlockIndex = self.block_index_at_file_offset(fo_nl_a_search_start);
            let bi_stop: BlockIndex = 0;
            debug_eprintln!(
                "{}find_line A2a: scan middle block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
                so(), bo_middle, bi_at, self.file_offset_at_block_offset_index(bo_middle, bi_at), bi_stop,
            );
            loop {
                // XXX: single-byte encoding
                if (*bptr_middle)[bi_at] == NLu8 {
                    found_nl_a = true;
                    fo_nl_a = self.file_offset_at_block_offset_index(bo_middle, bi_at);
                    debug_eprintln!(
                        "{}find_line A2a: found newline A in middle block during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                        so(),
                        bo_middle,
                        bi_at,
                        fo_nl_a,
                        byte_to_char_noraw((*bptr_middle)[bi_at]),
                    );
                    // adjust offsets one forward
                    // XXX: single-byte encoding
                    fo_nl_a1 = fo_nl_a + charsz_fo;
                    bi_at += charsz_bi;
                    break;
                }
                if bi_at == 0 {
                    break;
                }
                // XXX: single-byte encoding
                bi_at -= charsz_bi;
                if bi_at < bi_stop {
                    break;
                }
            }
            let fo_: FileOffset;
            if found_nl_a {
                fo_ = fo_nl_a1;
            } else {
                debug_eprintln!("{}find_line A2a: newline A not found in middle block {} but store middle block", so(), bo_middle);
                fo_ = self.file_offset_at_block_offset_index(bo_middle, bi_at);
            }
            let li = LinePart::new(
                bptr_middle.clone(),
                bi_at,
                bi_middle_end + 1,
                fo_,
                bo_middle,
                self.blocksz(),
            );
            line.prepend(li);
            if bof != 0 {
                debug_eprintln!("{}find_line A2a: blockoffset set to {}", so(), bof);
                bof -= 1;
            } else {
                debug_eprintln!("{}find_line A2a: run into beginning of file", so());
                begof = true;
            }
        } else {
            debug_eprintln!("{}find_line A2b: search for newline A crossed block boundary {} -> {}, save LinePart", so(), bo_middle, bof);
            // the charsz shift backward to begin search for newline A crossed block boundary
            // so save linepart from "middle" block before searching further
            let li = LinePart::new(
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
            fo_nl_a = 0;
            fo_nl_a1 = 0;
        }

        if !found_nl_a && !begof {
            let mut bptr_prior: BlockP;
            let mut bptr: BlockP = bptr_middle.clone();
            let mut bi_start_prior: BlockIndex;
            let mut bi_start: BlockIndex = bi_middle;
            while !found_nl_a && !begof {
                // "middle" block should have been handled by now
                // remainder is to just walk backwards chedcking for first newline or beginning of file
                debug_eprintln!("{}find_line A4: searching blockoffset {} …", so(), bof);
                bptr_prior = bptr;
                bptr = match self.blockreader.read_block(bof) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line A4: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                            so(),
                            bof,
                            &(*val),
                            (*val).len()
                        );
                        val
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!("{}find_line A4: read_block({}) returned Done {:?}", so(), bof, self.path());
                        return ResultS4_LineFind::Done;
                    },
                    ResultS3_ReadBlock::Err(err) => {
                        debug_eprintln!("{}find_line({}) A4: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bof, err);
                        return ResultS4_LineFind::Err(err);
                    }
                };
                let blen: BlockIndex = bptr.len() as BlockIndex;
                assert_ge!(blen, charsz_bi, "blen is less than charsz; not yet handled, file {:?}", self.path());
                bi_start_prior = bi_start;
                bi_start = blen - charsz_bi;
                let mut bi_at: BlockIndex = bi_start;
                let bi_stop: BlockIndex = 0;
                debug_eprintln!(
                    "{}find_line A5: scan block {} backwards, starting from blockindex {} (fileoffset {}) down to blockindex {} searching for newline A",
                    so(), bof, bi_at, self.file_offset_at_block_offset_index(bof, bi_at), bi_stop,
                );
                loop {
                    // XXX: single-byte encoding
                    if (*bptr)[bi_at] == NLu8 {
                        found_nl_a = true;
                        fo_nl_a = self.file_offset_at_block_offset_index(bof, bi_at);
                        debug_eprintln!(
                            "{}find_line A5: found newline A during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            so(),
                            bof,
                            bi_at,
                            fo_nl_a,
                            byte_to_char_noraw((*bptr)[bi_at]),
                        );
                        // adjust offsets one forward
                        // XXX: single-byte encoding
                        fo_nl_a1 = fo_nl_a + charsz_fo;
                        bi_at += charsz_bi;
                        let bof_a1 = self.block_offset_at_file_offset(fo_nl_a1);
                        if bof_a1 == bof {
                            // newline A and first line char does not cross block boundary
                            debug_eprintln!("{}find_line A5: store current blockoffset {}", so(), bof);
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
                            debug_eprintln!("{}find_line A5: store prior blockoffset {}", so(), bof_a1);
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
                            debug_eprintln!("{}find_line A5: blockoffset {} was already stored", so(), bof_a1);
                        }
                        break;
                    }
                    if bi_at == 0 {
                        break;
                    }
                    // XXX: single-byte encoding
                    bi_at -= charsz_bi;
                    if bi_at < bi_stop {
                        break;
                    }
                }
                if found_nl_a {
                    break;
                }
                debug_eprintln!("{}find_line A5: store blockoffset {}", so(), bof);
                let li = LinePart::new(
                    bptr.clone(),
                    bi_stop,
                    bi_start + 1,
                    self.file_offset_at_block_offset_index(bof, 0),
                    bof,
                    self.blocksz(),
                );
                line.prepend(li);                
                if bof != 0 {
                    // newline A not found
                    debug_eprintln!("{}find_line A5: newline A not found in block {}", so(), bof);
                    bof -= 1;
                } else {
                    // hit beginning of file, "newline A" is the beginning of the file (not a newline char)
                    // store that first block
                    debug_eprintln!("{}find_line A5: ran into beginning of file", so());
                    found_nl_a = true;
                    begof = true;
                    debug_assert!(line.stores_blockoffset(0), "block 0 was not stored but ran into beginning of file {:?}", self.path());
                }
            }  // end while !found_nl_a !begof
        }// end if !found_nl_a !begof

        // may occur in files ending on a single newline
        debug_eprintln!("{}find_line C: line.count() is {}", so(), line.count_lineparts());
        if line.count_lineparts() == 0 {
            if self.find_line_lru_cache_enabled {
                self.find_line_lru_cache_put += 1;
                debug_eprintln!("{}find_line C: LRU Cache put({}, Done)", so(), fileoffset);
                self.find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Done);
            }
            debug_eprintln!("{}find_line({}) C: return ResultS4_LineFind::Done;", sx(), fileoffset);
            return ResultS4_LineFind::Done;
        }

        debug_eprintln!("{}find_line D: return {:?};", so(), line);
        let fo_end = line.fileoffset_end();
        let linep = self.insert_line(line);
        if self.find_line_lru_cache_enabled {
            self.find_line_lru_cache_put += 1;
            debug_eprintln!("{}find_line D: LRU Cache put({}, Found({}, …))", so(), fileoffset, fo_end + 1);
            self.find_line_lru_cache
                .put(fileoffset, ResultS4_LineFind::Found((fo_end + 1, linep.clone())));
        }
        debug_eprintln!(
            "{}find_line({}) D: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}] {:?}",
            sx(),
            fileoffset,
            fo_end + 1,
            &*linep,
            (*linep).fileoffset_begin(),
            (*linep).fileoffset_end(),
            (*linep).to_String_noraw()
        );

        ResultS4_LineFind::Found((fo_end + 1, linep))
    }
}
