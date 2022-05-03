// Readers/syslinereader.rs
//

pub use crate::common::{
    FPath,
    FileOffset,
    NLu8,
    CharSz,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    Slices,
};

use crate::Readers::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeL_Opt,
    DateTime_Parse_Data,
    DateTime_Parse_Datas_vec,
    //DateTimePattern,
    //DateTimePattern_str,
    DATETIME_PARSE_DATAS_VEC,
    str_datetime,
    dt_pass_filters,
    dt_after_or_before,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    slice_contains_X_2,
    u8_to_str,
};

use crate::Readers::linereader::{
    LineIndex,
    Line,
    LineP,
    Lines,
    LineReader,
    ResultS4_LineFind,
    enum_BoxPtrs,
};

use crate::Readers::summary::{
    Summary,
};

use crate::common::{
    Bytes,
    ResultS4,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    str_to_String_noraw,
};
use crate::dbgpr::printers::{
    Color,
    ColorSpec,
    WriteColor,
};

use crate::dbgpr::stack::{
    sn,
    snx,
    so,
    sx,
};

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io;
//use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
use std::io::{
    Error,
    Result,
    ErrorKind,
    //Seek,
    //SeekFrom,
};
use std::io::prelude::*;
use std::str;
use std::sync::Arc;

extern crate chain_cmp;
use chain_cmp::chmp;

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate lru;
use lru::LruCache;

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
    //debug_assert_gt,
};

extern crate rangemap;
use rangemap::RangeMap;

extern crate static_assertions;
use static_assertions::{
    const_assert,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sysline
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━


/// Sysline Searching error
/// TODO: does SyslineFind need an `Found_EOF` state? Is it an unnecessary overlap of `Ok` and `Done`?
#[allow(non_camel_case_types)]
pub type ResultS4_SyslineFind = ResultS4<(FileOffset, SyslineP), Error>;

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s.
/// A "syslog line" or `Sysline` is one or more `Line`s, where the first line contains a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslog lines.
pub struct Sysline {
    /// the one or more `Line` that make up a Sysline
    pub(crate) lines: Lines,
    /// index into `Line` where datetime string starts
    /// byte-based count
    /// datetime is presumed to be on first Line
    pub(crate) dt_beg: LineIndex,
    /// index into `Line` where datetime string ends, one char past last character of datetime string
    /// byte-based count
    /// datetime is presumed to be on first Line
    pub(crate) dt_end: LineIndex,
    /// parsed DateTime instance
    pub(crate) dt: DateTimeL_Opt,
}

/// a signifier value for "not set" or "null" - because sometimes Option is a PitA
const LI_NULL: LineIndex = LineIndex::MAX;

impl std::fmt::Debug for Sysline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for lp in self.lines.iter() {
            li_s.push_str(&format!(
                "Line @{:p} (fileoffset_beg {}, fileoffset_end {}, len() {}, count() {}",
                &*lp,
                (*lp).fileoffset_begin(),
                (*lp).fileoffset_end(),
                (*lp).len(),
                (*lp).count()
            ));
        }
        f.debug_struct("Sysline")
            .field("fileoffset_begin()", &self.fileoffset_begin())
            .field("fileoffset_end()", &self.fileoffset_end())
            .field("lines @", &format_args!("{:p}", &self.lines))
            .field("lines.len", &self.lines.len())
            .field("dt_beg", &self.dt_beg)
            .field("dt_end", &self.dt_end)
            .field("dt", &self.dt)
            .field("lines", &li_s)
            .finish()
    }
}

impl Default for Sysline {
    fn default() -> Self {
        Self {
            lines: Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY),
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        }
    }
}

impl Sysline {
    /// default `with_capacity` for a `Lines`, most often will only need 1 capacity
    /// as the found "sysline" will likely be one `Line`
    const SYSLINE_PARTS_WITH_CAPACITY: usize = 1;
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new() -> Sysline {
        Sysline::default()
    }

    pub fn new_from_line(linep: LineP) -> Sysline {
        let mut v = Lines::with_capacity(Sysline::SYSLINE_PARTS_WITH_CAPACITY);
        v.push(linep);
        Sysline {
            lines: v,
            dt_beg: LI_NULL,
            dt_end: LI_NULL,
            dt: None,
        }
    }

    pub fn charsz(self: &Sysline) -> usize {
        Sysline::CHARSZ
    }

    pub fn push(&mut self, linep: LineP) {
        if !self.lines.is_empty() {
            // TODO: sanity check lines are in sequence
        }
        debug_eprintln!(
            "{}SyslineReader.push(@{:p}), self.lines.len() is now {}",
            so(),
            &*linep,
            self.lines.len() + 1
        );
        self.lines.push(linep);
    }

    /// the byte offset into the file where this `Sysline` begins
    /// "points" to first character of `Sysline` (and underlying `Line`)
    pub fn fileoffset_begin(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        (*self.lines[0]).fileoffset_begin()
    }

    /// the byte offset into the file where this `Sysline` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Sysline) -> FileOffset {
        assert_ne!(self.lines.len(), 0, "This Sysline has no Line");
        let last_ = self.lines.len() - 1;
        (*self.lines[last_]).fileoffset_end()
    }

    /// the fileoffset into the next sysline
    /// this Sysline does not know if that fileoffset points to the end of file (one past last actual byte)
    pub fn fileoffset_next(self: &Sysline) -> FileOffset {
        self.fileoffset_end() + (self.charsz() as FileOffset)
    }

    /// length in bytes of this Sysline
    pub fn len(self: &Sysline) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `Line` in `self.lines`
    pub fn count(self: &Sysline) -> u64 {
        self.lines.len() as u64
    }

    /// sum of `Line.count_bytes`
    pub fn count_bytes(self: &Sysline) -> u64 {
        let mut cb = 0;
        for ln in self.lines.iter() {
            cb += ln.count_bytes();
        }
        cb
    }

    /// a `String` copy of the demarcating datetime string found in the Sysline
    #[allow(non_snake_case)]
    pub fn datetime_String(self: &Sysline) -> String {
        assert_ne!(self.dt_beg, LI_NULL, "dt_beg has not been set");
        assert_ne!(self.dt_end, LI_NULL, "dt_end has not been set");
        assert_lt!(self.dt_beg, self.dt_end, "bad values dt_end {} dt_beg {}", self.dt_end, self.dt_beg);
        let slice_ = self.lines[0].as_bytes();
        assert_lt!(
            self.dt_beg,
            slice_.len(),
            "dt_beg {} past end of slice[{}‥{}]?",
            self.dt_beg,
            self.dt_beg,
            self.dt_end
        );
        assert_le!(
            self.dt_end,
            slice_.len(),
            "dt_end {} past end of slice[{}‥{}]?",
            self.dt_end,
            self.dt_beg,
            self.dt_end
        );
        // TODO: here is a place to use `bstr`
        let buf: &[u8] = &slice_[self.dt_beg..self.dt_end];
        let s_ = match str::from_utf8(buf) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Error in datetime_String() during str::from_utf8 {} buffer {:?}", err, buf);
                ""
            }
        };
        String::from(s_)
    }

    /// return all the slices that make up this `Sysline`
    /// TODO: make an iterable trait of this struct
    pub fn get_slices(self: &Sysline) -> Slices {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += lp.get_slices_count();
        }
        let mut slices = Slices::with_capacity(sz);
        for lp in &self.lines {
            slices.extend(lp.get_slices().iter());
        }
        slices
    }

    /// print approach #1, use underlying `Line` to `print`
    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    pub fn print1(self: &Sysline, raw: bool) {
        for lp in &self.lines {
            (*lp).print(raw);
        }
    }

    // TODO: [2022/03/23] implement an `iter_slices` that does not require creating a new `vec`, just
    //       passes `&bytes` back. Call `iter_slices` from `print`

    /// print approach #2, print by slices
    /// prints raw data from underlying `Block`
    /// testing helper
    /// TODO: move this into a `Printer` class
    #[cfg(any(debug_assertions,test))]
    #[allow(dead_code)]
    fn print2(&self) {
        let slices = self.get_slices();
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for slice in slices.iter() {
            match stdout_lock.write(slice) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("ERROR: write: StdoutLock.write(slice@{:p} (len {})) error {}", slice, slice.len(), err);
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: write: stdout flushing error {}", err);
            }
        }
    }

    /// helper to `print_color`
    /// caller must acquire stdout.Lock, and call `stdout.flush()`
    /// TODO: move this into a `Printer` class
    fn print_color_slices(stdclr: &mut termcolor::StandardStream, colors: &[Color], values:&[&[u8]]) -> Result<()> {
        assert_eq!(colors.len(), values.len());
        for (color, value) in colors.iter().zip(values.iter())
        {
            if let Err(err) = stdclr.set_color(ColorSpec::new().set_fg(Some(color.clone()))) {
                eprintln!("ERROR: print_color_slices: stdout.set_color({:?}) returned error {}", color, err);
                //continue;
                return Err(err);
            };
            if let Err(err) = stdclr.write(value) {
                eprintln!("ERROR: print_color_slices: stdout.write(…) returned error {}", err);
                //continue;
                return Err(err);
            };
        }
        Ok(())
    }

    /// print with color
    /// prints raw data from underlying `Block` bytes
    /// XXX: does not handle multi-byte strings
    /// TODO: needs a global mutex... I think...
    /// TODO: move this into a `Printer` class or `Sysline_Printer` class
    /// TODO: [2022/04] would be interesting to benchmark difference of printing using one
    ///       instance of `termcolor::StandStream` versus many (like done here).
    ///       would one instance improve performance?
    pub fn print_color(
        &self,
        color_choice_opt: Option<termcolor::ColorChoice>,
        color_text: Color,
        color_datetime: Color
    ) -> Result<()> {
        let slices = self.get_slices();
        let color_choice: termcolor::ColorChoice = match color_choice_opt {
            Some(choice_) => choice_,
            None => termcolor::ColorChoice::Auto,
        };
        //let mut stdout = io::stdout();
        //let mut stdout_lock = stdout.lock();
        let mut clrout = termcolor::StandardStream::stdout(color_choice);
        let mut at: LineIndex = 0;
        let dtb = self.dt_beg;
        let dte = self.dt_end;
        // 
        for slice in slices.iter() {
            let len_ = slice.len();
            match color_choice {
                // bypass `print_color_slices` for a small speed improvement
                termcolor::ColorChoice::Never => {
                    match clrout.write(slice) {
                        Ok(_) => {},
                        Err(err) => {
                            return Err(err);
                        }
                    }
                },
                // use `print_color_slices` to print the datetime substring and test in differing colors
                _ => {
                    // datetime entirely in this `slice`
                    if chmp!(at <= dtb < dte < (at + len_)) {
                        let a = &slice[..(dtb-at)];
                        let b = &slice[(dtb-at)..(dte-at)];
                        let c = &slice[(dte-at)..];
                        match Sysline::print_color_slices(&mut clrout, &[color_text, color_datetime, color_text], &[a, b, c]) {
                            Ok(_) => {},
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    } else {
                        // TODO: [2022/03] datetime crosses into next slice
                        match Sysline::print_color_slices(&mut clrout, &[color_text], &[slice]) {
                            Ok(_) => {},
                            Err(err) => {
                                return Err(err);
                            }
                        };
                    }
                    at += len_;
                }
            }   
        }
        let mut ret = Ok(());
        if let Err(err) = clrout.flush() {
            eprintln!("ERROR: print_color: stdout.flush() {}", err);
            ret = Err(err);
        }
        if let Err(err) = clrout.reset() {
            eprintln!("ERROR: print_color: stdout.reset() {}", err);
            return Err(err);
        }
        ret
    }

    /// create `String` from `self.lines`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    fn _to_String_raw(self: &Sysline, raw: bool) -> String {
        let mut sz: usize = 0;
        for lp in &self.lines {
            sz += (*lp).len();
        }
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let mut s_ = String::with_capacity(sz + 1);
        for lp in &self.lines {
            s_ += (*lp)._to_String_raw(raw).as_str();
        }
        s_
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Sysline) -> String {
        self._to_String_raw(true)
    }

    /// `Sysline` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        self._to_String_raw(false)
    }

    #[allow(non_snake_case)]
    #[cfg(not(any(debug_assertions,test)))]
    pub fn to_String_noraw(self: &Sysline) -> String {
        panic!("should not call function 'Sysline::to_String_noraw' in release build");
        String::new()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateTime typing, strings, and formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: how to do this a bit more efficiently, and not store an entire copy?
/// count of datetime format strings used
type DateTime_Pattern_Counts = HashMap<DateTime_Parse_Data, u64>;
/// return type for `SyslineReader::find_datetime_in_line`
pub type Result_FindDateTime = Result<(DateTime_Parse_Data, DateTimeL)>;
/// return type for `SyslineReader::parse_datetime_in_line`
pub type Result_ParseDateTime = Result<(LineIndex, LineIndex, DateTimeL)>;
pub type Result_ParseDateTimeP = Arc<Result_ParseDateTime>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// thread-safe Atomic Reference Counting Pointer to a `Sysline`
pub type SyslineP = Arc<Sysline>;
pub type SyslineP_Opt = Option<Arc<Sysline>>;
/// storage for `Sysline`
pub type Syslines = BTreeMap<FileOffset, SyslineP>;
/// range map where key is sysline begin to end `[ Sysline.fileoffset_begin(), Sysline.fileoffset_end()]`
/// and where value is sysline begin (`Sysline.fileoffset_begin()`). Use the value to lookup associated `Syslines` map
type SyslinesRangeMap = RangeMap<FileOffset, FileOffset>;
/// used internally by `SyslineReader`
type SyslinesLRUCache = LruCache<FileOffset, ResultS4_SyslineFind>;
/// used internally by `SyslineReader`
type LineParsedCache = LruCache<FileOffset, Result_ParseDateTimeP>;

/// Specialized Reader that uses `LineReader` to find syslog lines
pub struct SyslineReader<'syslinereader> {
    pub(crate) linereader: LineReader<'syslinereader>,
    /// Syslines by fileoffset_begin
    pub(crate) syslines: Syslines,
    /// count of Syslines processed
    syslines_count: u64,
    // TODO: has `syslines_by_range` ever found a sysline?
    //       would be good to add a test for it.
    /// Syslines fileoffset by sysline fileoffset range, i.e. `[Sysline.fileoffset_begin(), Sysline.fileoffset_end()+1)`
    /// the stored value can be used as a key for `self.syslines`
    syslines_by_range: SyslinesRangeMap,
    /// datetime formatting data, for extracting datetime strings from Lines
    /// TODO: change to Set
    dt_patterns: DateTime_Parse_Datas_vec,
    /// internal use; counts found patterns stored in `dt_patterns`
    /// not used after `analyzed == true`
    dt_patterns_counts: DateTime_Pattern_Counts,
    /// default FixedOffset for found `DateTime` without timezone
    tz_offset: FixedOffset,
    // TODO: is the LRU cache really helping?
    _find_sysline_lru_cache_enabled: bool,
    /// internal LRU cache for `find_sysline`. maintained in `SyslineReader::find_sysline`
    _find_sysline_lru_cache: SyslinesLRUCache,
    // internal stats for `self.find_sysline`
    pub(self) _find_sysline_lru_cache_hit: u64,
    // internal stats for `self.find_sysline`
    pub(self) _find_sysline_lru_cache_miss: u64,
    // internal stats for `self.find_sysline`
    pub(self) _find_sysline_lru_cache_put: u64,
    // enable/disable `_parse_datetime_in_line_lru_cache`
    _parse_datetime_in_line_lru_cache_enabled: bool,
    // internal cache of calls to `SyslineReader::parse_datetime_in_line`. maintained in `SyslineReader::find_sysline`
    _parse_datetime_in_line_lru_cache: LineParsedCache,
    // internal stats for `self._parse_datetime_in_line_lru_cache`
    pub(self) _parse_datetime_in_line_lru_cache_hit: u64,
    // internal stats for `self._parse_datetime_in_line_lru_cache`
    pub(self) _parse_datetime_in_line_lru_cache_miss: u64,
    // internal stats for `self._parse_datetime_in_line_lru_cache`
    pub(self) _parse_datetime_in_line_lru_cache_put: u64,
    /// has `self.file_analysis` completed?
    analyzed: bool,
}

// TODO: [2021/09/19]
//       put all filter data into one struct `SyslineFilter`, simpler to pass around

impl fmt::Debug for SyslineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
impl<'syslinereader> SyslineReader<'syslinereader> {
    const DT_PATTERN_MAX_PRE_ANALYSIS: usize = 4;
    const DT_PATTERN_MAX: usize = 1;
    const ANALYSIS_THRESHOLD: u64 = 5;
    const _FIND_SYSLINE_LRU_CACHE_SZ: usize = 4;
    const _PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ: usize = 8;

    pub fn new(path: &'syslinereader FPath, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslineReader<'syslinereader>> {
        let lr = match LineReader::new(path, blocksz) {
            Ok(val) => val,
            Err(err) => {
                //eprintln!("ERROR: LineReader::new({}, {}) failed {}", path, blocksz, err);
                return Err(err);
            }
        };
        Ok(
            SyslineReader {
                linereader: lr,
                syslines: Syslines::new(),
                syslines_count: 0,
                syslines_by_range: SyslinesRangeMap::new(),
                dt_patterns: DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
                dt_patterns_counts: DateTime_Pattern_Counts::with_capacity(SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS),
                tz_offset,
                _find_sysline_lru_cache_enabled: true,
                _find_sysline_lru_cache: SyslinesLRUCache::new(SyslineReader::_FIND_SYSLINE_LRU_CACHE_SZ),
                _find_sysline_lru_cache_hit: 0,
                _find_sysline_lru_cache_miss: 0,
                _find_sysline_lru_cache_put: 0,
                _parse_datetime_in_line_lru_cache_enabled: true,
                _parse_datetime_in_line_lru_cache: LineParsedCache::new(SyslineReader::_PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ),
                _parse_datetime_in_line_lru_cache_hit: 0,
                _parse_datetime_in_line_lru_cache_miss: 0,
                _parse_datetime_in_line_lru_cache_put: 0,
                analyzed: false,
            }
        )
    }

    pub fn blocksz(&self) -> BlockSz {
        self.linereader.blocksz()
    }

    pub fn filesz(&self) -> BlockSz {
        self.linereader.filesz()
    }

    pub fn path(&self) -> &FPath {
        self.linereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.linereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.linereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.linereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.linereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        self.linereader.file_blocks_count()
    }

    pub fn blockoffset_last(&self) -> BlockOffset {
        self.linereader.blockoffset_last()
    }

    /// smallest size character
    pub fn charsz(&self) -> usize {
        self.linereader.charsz()
    }

    /// count of `Sysline`s processed
    pub fn count(&self) -> u64 {
        self.syslines_count
    }

    /// enable internal LRU cache used by `find_sysline` and `parse_datetime_in_line`
    /// intended to aid testing
    pub fn LRU_cache_enable(&mut self) {
        self._find_sysline_lru_cache_enabled = true;
        self._find_sysline_lru_cache.clear();
        self._find_sysline_lru_cache.resize(SyslineReader::_FIND_SYSLINE_LRU_CACHE_SZ);
        self._parse_datetime_in_line_lru_cache_enabled = true;
        self._parse_datetime_in_line_lru_cache.clear();
        self._parse_datetime_in_line_lru_cache.resize(SyslineReader::_PARSE_DATETIME_IN_LINE_LRU_CACHE_SZ);
    }

    /// disable internal LRU cache used by `find_sysline` and `parse_datetime_in_line`
    /// intended to aid testing
    pub fn LRU_cache_disable(&mut self) {
        self._find_sysline_lru_cache_enabled = false;
        self._find_sysline_lru_cache.clear();
        self._find_sysline_lru_cache.resize(0);
        self._parse_datetime_in_line_lru_cache_enabled = false;
        self._parse_datetime_in_line_lru_cache.clear();
        self._parse_datetime_in_line_lru_cache.resize(0);
    }

    /// read block zero (the first data block of the file), do necessary analysis
    pub fn zeroblock_process(&mut self) -> std::io::Result<bool> {
        self.linereader.zeroblock_process()
        // LAST WORKING HERE 2022/04/29 00:23:29
        // need to add find_sysline calls for block zero, then determine if number found
        // is acceptable to continue. requires `find_syseline` that can avoid new calls to
        // `read_block`... needs some thought to do cleanly.
        // might need to add `find_sysline_in_block` that searches only within one block
        // would be far simpler to restructure `find_sysline`.
        // would need corresponding function `LineReader::find_line`.
    }

    /// print Sysline at `fileoffset`
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

    /// Testing helper only
    /// print all known `Sysline`s
    #[cfg(test)]
    pub fn print_all(&self, raw: bool) {
        debug_eprintln!("{}print_all(true)", sn());
        for fo in self.syslines.keys() {
            self.print(*fo, raw);
        }
        debug_eprintln!("\n{}print_all(true)", sx());
    }

    /// is given `SyslineP` last in the file?
    pub(crate) fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        let filesz = self.filesz();
        let fo_end = (*syslinep).fileoffset_end();
        if (fo_end + 1) == filesz {
            return true;
        }
        assert_lt!(fo_end + 1, filesz, "fileoffset_end() {} is at or after filesz() fileoffset {}", fo_end, filesz);
        false
    }

    /// store passed `Sysline` in `self.syslines`, update other fields
    fn insert_sysline(&mut self, line: Sysline) -> SyslineP {
        let fo_beg: FileOffset = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let slp = SyslineP::new(line);
        debug_eprintln!("{}SyslineReader.insert_sysline: syslines.insert({}, Sysline @{:p})", so(), fo_beg, &*slp);
        self.syslines.insert(fo_beg, slp.clone());
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
        slp
    }

    // wrapper to call `str_datetime` with appropriate arguments
    pub fn str_datetime(
        dts: &str,
        dtpd: &DateTime_Parse_Data,
        tz_offset: &FixedOffset
    ) -> DateTimeL_Opt {
        str_datetime(dts, dtpd.pattern.as_str(), dtpd.tz, tz_offset)
    }

    /// if datetime found in `Line` returns `Ok` around
    /// indexes into `line` of found datetime string `(start of string, end of string)`
    /// else returns `Err`
    /// TODO: assumes Local TZ
    /// TODO: 2022/03/11 14:30:00
    ///      The concept of static pattern lengths (beg_i, end_i, actual_beg_i, actual_end_i) won't work for
    ///      variable length datetime patterns, i.e. full month names 'July 1, 2020' and 'December 1, 2020'.
    ///      Instead of fixing the current problem of unexpected datetime matches,
    ///      fix the concept problem of passing around fixed-length datetime strings. Then redo this.
    pub fn find_datetime_in_line(
        line: &Line,
        parse_data: &'syslinereader DateTime_Parse_Datas_vec,
        fpath: &FPath,
        charsz: &CharSz,
        tz_offset: &FixedOffset,
    ) -> Result_FindDateTime {
        debug_eprintln!("{}find_datetime_in_line:(Line@{:p}, {:?}) {:?}", sn(), &line, line.to_String_noraw(), fpath);
        // skip easy case; no possible datetime
        if line.len() < 4 {
            debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::InvalidInput);", sx());
            return Result_FindDateTime::Err(Error::new(ErrorKind::InvalidInput, "Line is too short"));
        }

        //let longest: usize = *DATETIME_PARSE_DATAS_VEC_LONGEST;
        //let mut dtsS: String = String::with_capacity(longest * (2 as usize));

        let hack12: &[u8; 2] = &b"12";
        let mut i = 0;
        // `sie` and `siea` is one past last char; exclusive.
        // `actual` are more confined slice offsets of the datetime,
        // XXX: it might be faster to skip the special formatting and look directly for the datetime stamp.
        //      calls to chrono are long according to the flamegraph.
        //      however, using the demarcating characters ("[", "]") does give better assurance.
        for dtpd in parse_data.iter() {
            i += 1;
            debug_eprintln!("{}find_datetime_in_line: pattern tuple {} ({:?}, {}, {}, {}, {})", so(), i, dtpd.pattern, dtpd.sib, dtpd.sie, dtpd.siba, dtpd.siea);
            debug_assert_lt!(dtpd.sib, dtpd.sie, "Bad values dtpd.sib dtpd.sie");
            debug_assert_ge!(dtpd.siba, dtpd.sib, "Bad values dtpd.siba dtpd.sib");
            debug_assert_le!(dtpd.siea, dtpd.sie, "Bad values dtpd.siea dtpd.sie");
            //debug_eprintln!("{}find_datetime_in_line searching for pattern {} {:?}", so(), i, dtpd.pattern);
            let len_ = (dtpd.sie - dtpd.sib) as LineIndex;
            // XXX: does not support multi-byte string; assumes single-byte
            if line.len() < dtpd.sie {
                debug_eprintln!(
                    "{}find_datetime_in_line: line len {} is too short for pattern {} len {} @({}, {}] {:?}",
                    so(),
                    line.len(),
                    i,
                    len_,
                    dtpd.sib,
                    dtpd.sie,
                    dtpd.pattern,
                );
                continue;
            }
            // take a slice of the `line_as_slice` then convert to `str`
            // this is to force the parsing function `Local.datetime_from_str` to constrain where it
            // searches within the `Line`
            // TODO: move this remaining loop section into an #[inline] pub function
            // TODO: to make this a bit more efficient, would be good to do a lookahead. Add a funciton like
            //       `Line.crosses_block(a: LineIndex, b: LineIndex) -> bool`. Then could set capacity of things
            //       ahead of time.
            let slice_: &[u8];
            let mut hack_slice: Bytes;
            match line.get_boxptrs(dtpd.sib, dtpd.sie) {
                enum_BoxPtrs::SinglePtr(box_slice) => {
                    slice_ = *box_slice;
                },
                enum_BoxPtrs::MultiPtr(vec_box_slice) => {
                    // XXX: really inefficient! I have no better ideas at this time.
                    hack_slice = Bytes::new();
                    for box_ in vec_box_slice {
                        hack_slice.extend_from_slice(*box_);
                    }
                    slice_ = hack_slice.as_slice();
                },
            };
            // hack efficiency improvement, presumes all found years will have a '1' or a '2' in them
            if charsz == &1 && dtpd.year && !slice_contains_X_2(slice_, hack12) {
            //if charsz == &1 && dtpd.year && !(slice_.contains(&hack12[0]) || slice_.contains(&hack12[1])) {
                debug_eprintln!("{}find_datetime_in_line: skip slice, does not have '1' or '2'", so());
                continue;
            }
            let dts: &str = match u8_to_str(slice_) {
                Some(val) => val,
                None => { continue; }
            };
            debug_eprintln!(
                "{}find_datetime_in_line: searching for pattern {} {:?} in {:?} (slice [{}‥{}] from Line {:?})",
                so(),
                i,
                dtpd.pattern,
                str_to_String_noraw(dts),
                dtpd.sib,
                dtpd.sie,
                line.to_String_noraw(),
            );
            // TODO: [2021/10/03]
            //       according to flamegraph, this function `Local::datetime_from_str` takes a very large amount of
            //       runtime, around 20% to 25% of entire process runtime. How to improve that?
            //       Could I create my own hardcoded parsing for a few common patterns?
            let dt = match SyslineReader::str_datetime(dts, dtpd, tz_offset) {
                Some(val) => {
                    debug_eprintln!("{}find_datetime_in_line: str_datetime returned {:?}", so(), val);
                    val
                }
                None => {
                    debug_eprintln!("{}find_datetime_in_line: str_datetime returned None", so());
                    continue;
                }
            }; // end for(pattern, ...)
            debug_eprintln!("{}find_datetime_in_line: return Ok({}, {}, {});", sx(), dtpd.sib, dtpd.sie, &dt);
            return Result_FindDateTime::Ok((dtpd.clone(), dt));
        }

        debug_eprintln!("{}find_datetime_in_line: return Err(ErrorKind::NotFound);", sx());
        Result_FindDateTime::Err(Error::new(ErrorKind::NotFound, "No datetime found!"))
    }

    /// private helper function to update `self.dt_patterns`
    fn dt_patterns_update(&mut self, datetime_parse_data: DateTime_Parse_Data) {
        if self.analyzed {
            return;
        }
        debug_eprintln!("{}dt_patterns_update(SyslineReader@{:p}, {:?})", sn(), self, datetime_parse_data);
        //
        // update `self.dt_patterns_counts`
        //
        if self.dt_patterns_counts.contains_key(&datetime_parse_data) {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns_counts.get_mut({:?}) += 1",
                so(),
                self,
                datetime_parse_data
            );
            let counter = self.dt_patterns_counts.get_mut(&datetime_parse_data).unwrap();
            *counter += 1;
        } else if self.dt_patterns_counts.len() < SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns_counts.insert({:?}, 0)",
                so(),
                self,
                datetime_parse_data
            );
            self.dt_patterns_counts.insert(datetime_parse_data.clone(), 1);
        }
        //
        // update `self.dt_patterns`
        //
        if self.dt_patterns.len() >= SyslineReader::DT_PATTERN_MAX_PRE_ANALYSIS {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns already DT_PATTERN_MAX_PRE_ANALYSIS length {:?}",
                sx(),
                self,
                &self.dt_patterns.len()
            );
            return;
        }
        if self.dt_patterns.contains(&datetime_parse_data) {
            debug_eprintln!(
                "{}dt_patterns_update(SyslineReader@{:p}) found DateTime_Parse_Data; skip self.dt_patterns.push",
                sx(),
                self
            );
            return;
        }
        debug_eprintln!(
            "{}dt_patterns_update(SyslineReader@{:p}) self.dt_patterns.push({:?})",
            sx(),
            self,
            datetime_parse_data
        );
        self.dt_patterns.push(datetime_parse_data);
    }

    /// analyze syslines gathered
    /// when a threshold of syslines or bytes has been processed, then
    /// 1. narrow down datetime formats used. this greatly reduces resources/time
    ///    used by `SyslineReader::find_datetime_in_line`
    /// 2. TODO: for any prior analyzed syslines using a datetime format that wasn't accepted,
    ///          retry parsing the lines with narrowed set of datetime formats. however, if those reparse attempts fail, keep the prior parse results using the
    ///          "odd man out" format
    /// TODO: will break if DT_PATTERN_MAX > 1
    fn dt_patterns_analysis(&mut self) {
        if self.analyzed || self.count() < SyslineReader::ANALYSIS_THRESHOLD {
            return;
        }
        debug_eprintln!("{}dt_patterns_analysis()", sn());
        // XXX: DT_PATERN_MAX > 1 is unimplemented
        const_assert!(SyslineReader::DT_PATTERN_MAX == 1);
        debug_assert_eq!(self.dt_patterns.len(), self.dt_patterns_counts.len(),
            "dt_patterns.len() != dt_patterns_count.len()\nself.dt_patterns ({})     : {:?}\nself.dt_patterns_counts ({}): {:?}", self.dt_patterns.len(), self.dt_patterns, self.dt_patterns_counts.len(), self.dt_patterns_counts);
        // TODO: change pattern tuple `DateTime_Parse_Data` to use Ranges, currently this is
        //       removing valid patterns in different (beg,end) positions
        // ripped from https://stackoverflow.com/a/60134450/471376
        // test https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b8eb53f40fd89461c9dad9c976746cc3
        let max_ = (&self.dt_patterns_counts).iter().fold(
            std::u64::MIN, |a,b| a.max(*(b.1))
        );
        self.dt_patterns_counts.retain(|_, v| *v >= max_);
        self.dt_patterns_counts.shrink_to(SyslineReader::DT_PATTERN_MAX);
        if cfg!(debug_assertions) {
            for (k, v) in self.dt_patterns_counts.iter() {
                debug_eprintln!("{}dt_patterns_analysis: self.dt_patterns_counts[{:?}]={:?}", so(), k, v);
            }
        }
        // XXX: is there a simpler way to get the first element?
        let datetime_parse_data = match self.dt_patterns_counts.iter().next() {
            Some((k, _)) => { k },
            None => {
                eprintln!("ERROR: self.dt_patterns_counts.values().next() returned None, it is len {}", self.dt_patterns_counts.len());
                self.analyzed = true;
                return;
            }
        };
        debug_eprintln!("{}dt_patterns_analysis: chose dt_pattern", so());
        // effectively remove all elements by index, except for `datetime_parse_data`
        // XXX: what is the rust idiomatic way to remove all but a few elements by index?
        let mut patts = DateTime_Parse_Datas_vec::with_capacity(SyslineReader::DT_PATTERN_MAX);
        let mut index_: usize = 0;
        for datetime_parse_data_ in &self.dt_patterns {
            if datetime_parse_data_.eq(datetime_parse_data) {
                break;
            }
            index_ += 1;
        }
        patts.push(self.dt_patterns.swap_remove(index_));
        self.dt_patterns = patts;
        //self.dt_patterns.retain(|v| v == **patt);
        self.dt_patterns.shrink_to(SyslineReader::DT_PATTERN_MAX);
        if cfg!(debug_assertions) {
            for dtpd in self.dt_patterns.iter() {
                debug_eprintln!("{}dt_patterns_analysis: self.dt_pattern {:?}", so(), dtpd);
            }
        }
        self.dt_patterns_counts.clear();
        self.dt_patterns_counts.shrink_to(0);
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
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}); {:?}", sn(), self, line.to_String_noraw());
        // if no `dt_patterns` have been found then try the default datetime patterns immediately
        if self.dt_patterns.is_empty() {
            debug_eprintln!("{}parse_datetime_in_line self.dt_patterns is empty", sn());
            // this `SyslineReader` has not determined it's own DateTime formatting data `self.dt_patterns`
            // so pass the built-in `DATETIME_PARSE_DATAS`.
            // Use the extra data returned by `find_datetime_in_line` to set `self.dt_patterns` once.
            // This will only happen once per `SyslineReader` (assuming a valid Syslog file)
            let result = SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC, self.path(), charsz, &self.tz_offset);
            let (datetime_parse_data, dt) = match result {
                Ok(val) => val,
                Err(err) => {
                    debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};", sx(), self, err);
                    return Err(err);
                }
            };
            self.dt_patterns_update(datetime_parse_data.clone());
            debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return OK;", sx(), self);
            return Result_ParseDateTime::Ok((datetime_parse_data.siba, datetime_parse_data.siea, dt));
        }
        debug_eprintln!("{}parse_datetime_in_line self.dt_patterns has {} entries", so(), &self.dt_patterns.len());
        // have already determined DateTime formatting for this file, so
        // no need to try *all* built-in DateTime formats, just try the known good formats `self.dt_patterns`
        let result = SyslineReader::find_datetime_in_line(line, &self.dt_patterns, self.path(), charsz, &self.tz_offset);
        let (datetime_parse_data, dt) = match result {
            Ok(val) => val,
            Err(err) => {
                if self.analyzed {
                    debug_eprintln!(
                        "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                        sx(),
                        self,
                        err
                    );
                    return Err(err);
                }
                // The known good format failed and this SyslineReader has not yet run `dt_format_analysis`
                // so now try other default formats. This is a resource expensive operation.
                debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Err {}; try again using default DATETIME_PARSE_DATAS_VEC", so(), self, err);
                match SyslineReader::find_datetime_in_line(line, &DATETIME_PARSE_DATAS_VEC, self.path(), charsz, &self.tz_offset) {
                    Ok((datetime_parse_data_, dt_)) => {
                        self.dt_patterns_update(datetime_parse_data_.clone());
                        (datetime_parse_data_, dt_)
                    }
                    Err(err_) => {
                        debug_eprintln!(
                            "{}parse_datetime_in_line(SyslineReader@{:p}) return Err {};",
                            sx(),
                            self,
                            err_
                        );
                        return Err(err_);
                    }
                }
            }
        };
        debug_eprintln!("{}parse_datetime_in_line(SyslineReader@{:p}) return Ok;", sx(), self);

        Result_ParseDateTime::Ok((datetime_parse_data.sib, datetime_parse_data.sie, dt))
    }

    /// helper to `find_sysline`
    /// call `self.parse_datetime_in_line` with help of `self._parse_datetime_in_line_cache`
    fn parse_datetime_in_line_cached(&mut self, lp: &LineP, charsz: &usize) -> Result_ParseDateTimeP {
        if self._parse_datetime_in_line_lru_cache_enabled {
            match self._parse_datetime_in_line_lru_cache.get(&lp.fileoffset_begin()) {
                Some(val) => {
                    self._parse_datetime_in_line_lru_cache_hit +=1;
                    return val.clone();
                },
                _ => {
                    self._parse_datetime_in_line_lru_cache_miss += 1;
                },
            }
        }
        let result: Result_ParseDateTime = self.parse_datetime_in_line(&*lp, charsz);
        let resultp: Result_ParseDateTimeP = Result_ParseDateTimeP::new(result);
        if self._parse_datetime_in_line_lru_cache_enabled {
            #[allow(clippy::single_match)]
            match self._parse_datetime_in_line_lru_cache.put(lp.fileoffset_begin(), resultp.clone()) {
                Some(val_prev) => {
                    panic!("self._parse_datetime_in_line_lru_cache already had key {:?}, value {:?}", lp.fileoffset_begin(), val_prev);
                },
                _ => {},
            };
        }

        resultp
    }

    /// Find first sysline at or after `fileoffset`.
    /// return (fileoffset of start of _next_ sysline, found Sysline at or after `fileoffset`)
    /// Similar to `LineReader.find_line`, `BlockReader.read_block`.
    ///
    /// This is the heart of the algorithm to find a sysline within a syslog file quickly.
    /// It's simply a binary search.
    /// It could definitely use some improvements, but for now it gets the job done.
    ///
    /// XXX: this function is large and cumbersome. you've been warned.
    ///
    /// TODO: separate caching to wrapper `find_sysline_cached`
    /// TODO: test that retrieving by cache always returns the same ResultS4 enum value as without a cache
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        debug_eprintln!("{}find_sysline(SyslineReader@{:p}, {})", sn(), self, fileoffset);

        // TODO: make these comparison values into consts
        if self.linereader.blockreader.count_bytes() > 0x4000 && self.count() < 3 {
            debug_eprintln!("{}find_sysline(SyslineReader@{:p}); too many bytes analyzed {}, yet too few syslines {}", sn(), self, self.linereader.blockreader.count_bytes(), self.count());
            // TODO: [2022/04/06] need to implement a way to abandon processing a file.
            //return Result_ParseDateTime::Error("");
        }

        if self._find_sysline_lru_cache_enabled {
            // check if `fileoffset` is already known about in LRU cache
            match self._find_sysline_lru_cache.get(&fileoffset) {
                Some(rlp) => {
                    self._find_sysline_lru_cache_hit += 1;
                    debug_eprintln!("{}find_sysline: found LRU cached for fileoffset {}", so(), fileoffset);
                    match rlp {
                        ResultS4_SyslineFind::Found(val) => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Found(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return ResultS4_SyslineFind::Found((val.0, val.1.clone()));
                        }
                        ResultS4_SyslineFind::Found_EOF(val) => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Found_EOF(({}, …)) @[{}, {}] from LRU cache", sx(), val.0, val.1.fileoffset_begin(), val.1.fileoffset_end());
                            return ResultS4_SyslineFind::Found_EOF((val.0, val.1.clone()));
                        }
                        ResultS4_SyslineFind::Done => {
                            debug_eprintln!("{}return ResultS4_SyslineFind::Done from LRU cache", sx());
                            return ResultS4_SyslineFind::Done;
                        }
                        ResultS4_SyslineFind::Err(err) => {
                            debug_eprintln!("{}Error {}", so(), err);
                            eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {} error {}", fileoffset, err);
                        }
                    }
                }
                None => {
                    self._find_sysline_lru_cache_miss += 1;
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in LRU cache", so(), fileoffset);
                }
            }
        }

        // check if the offset is already in a known range
        match self.syslines_by_range.get_key_value(&fileoffset) {
            Some(range_fo) => {
                let range = range_fo.0;
                debug_eprintln!(
                "{}find_sysline: hit syslines_by_range cache for FileOffset {} (found in range {:?})",
                so(),
                fileoffset,
                range
            );
                let fo = range_fo.1;
                let slp = self.syslines[fo].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                if self.is_sysline_last(&slp) {
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                    self._find_sysline_lru_cache_put += 1;
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                    return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                }
                self._find_sysline_lru_cache_put += 1;
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                debug_eprintln!(
                    "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                    sx(),
                    fo_next,
                    &*slp,
                    (*slp).fileoffset_begin(),
                    (*slp).fileoffset_end(),
                    (*slp).to_String_noraw()
                );
                return ResultS4_SyslineFind::Found((fo_next, slp));
            }
            None => {
                debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
            }
        }

        // check if there is a Sysline already known at this fileoffset
        // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
        if self.syslines.contains_key(&fileoffset) {
            debug_assert!(self.syslines_by_range.contains_key(&fileoffset), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}) returned None (syslines_by_range out of synch)", fileoffset, fileoffset);
            debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fileoffset);
            let slp = self.syslines[&fileoffset].clone();
            // XXX: multi-byte character encoding
            let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
            // TODO: determine if `fileoffset` is the last sysline of the file
            //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
            if self._find_sysline_lru_cache_enabled {
                self._find_sysline_lru_cache_put += 1;
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
            }
            return ResultS4_SyslineFind::Found((fo_next, slp));
        } else {
            debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines", so(), fileoffset);
        }

        debug_eprintln!("{}find_sysline: searching for first sysline datetime A …", so());

        //
        // find line with datetime A
        //

        let mut fo_a: FileOffset = 0;
        let mut fo1: FileOffset = fileoffset;
        let mut sl = Sysline::new();
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result: ResultS4_LineFind = self.linereader.find_line(fo1);
            let eof = result.is_eof();
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) | ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: A FileOffset {} Line @{:p} len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    if self._find_sysline_lru_cache_enabled {
                        self._find_sysline_lru_cache_put += 1;
                        debug_eprintln!("{}find_sysline: LRU cache put({}, Done)", so(), fileoffset);
                        self._find_sysline_lru_cache.put(fileoffset, ResultS4_SyslineFind::Done);
                    }
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; A from LineReader.find_line({})", sx(), fo1);
                    return ResultS4_SyslineFind::Done;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); A from LineReader.find_line({})", sx(), err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let resultp = self.parse_datetime_in_line_cached(&lp, &self.charsz());
            debug_eprintln!("{}find_sysline: A find_datetime_in_line returned {:?}", so(), resultp);
            match *resultp {
                Err(_) => {}
                Ok((dt_beg, dt_end, dt)) => {
                    // a datetime was found! beginning of a sysline
                    fo_a = fo1;
                    sl.dt_beg = dt_beg;
                    sl.dt_end = dt_end;
                    sl.dt = Some(dt);
                    debug_eprintln!("{}find_sysline: A sl.push({:?})", so(), (*lp).to_String_noraw());
                    sl.push(lp);
                    fo1 = sl.fileoffset_end() + (self.charsz() as FileOffset);
                    // sanity check
                    debug_assert_lt!(dt_beg, dt_end, "bad dt_beg {} dt_end {}", dt_beg, dt_end);
                    debug_assert_lt!(dt_end, fo1 as usize, "bad dt_end {} fileoffset+charsz {}", dt_end, fo1 as usize);
                    if eof {
                        let slp = SyslineP::new(sl);
                        if self._find_sysline_lru_cache_enabled {
                            self._find_sysline_lru_cache_put += 1;
                            debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo1);
                            self._find_sysline_lru_cache
                                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo1, slp.clone())));
                        }
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF({}, {:p}) @[{}, {}]; A found here and LineReader.find_line({})",
                            sx(),
                            fo1,
                            &(*slp),
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                            fo1,
                        );
                        return ResultS4_SyslineFind::Found_EOF((fo1, slp));
                    }
                    break;
                }
            }
            debug_eprintln!("{}find_sysline: A skip push Line {:?}", so(), (*lp).to_String_noraw());
            fo1 = fo2;
        }

        debug_eprintln!(
            "{}find_sysline: found line with datetime A at FileOffset {}, searching for datetime B starting at fileoffset {} …",
            so(),
            fo_a,
            fo1
        );

        //
        // find line with datetime B
        //

        { // check if sysline at `fo1` is already known about
            /*
            // XXX: not necessary to check `self.syslines` since `self.syslines_by_range` is checked.
            // check if there is a Sysline already known at this fileoffset
            if self.syslines.contains_key(&fo1) {
                debug_assert!(self.syslines_by_range.contains_key(&fo1), "self.syslines.contains_key({}) however, self.syslines_by_range.contains_key({}); syslines_by_range out of synch", fo1, fo1);
                debug_eprintln!("{}find_sysline: hit self.syslines for FileOffset {}", so(), fo1);
                let slp = self.syslines[&fo1].clone();
                // XXX: multi-byte character encoding
                let fo_next = (*slp).fileoffset_end() + (self.charsz() as FileOffset);
                // TODO: determine if `fileoffset` is the last sysline of the file
                //       should add a private helper function for this task `is_sysline_last(FileOffset)` ... something like that
                debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines {:?}",
                sx(),
                fo_next,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
                if self._find_sysline_lru_cache_enabled {
                    self._find_sysline_lru_cache_put += 1;
                    self._find_sysline_lru_cache
                        .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                }
                return ResultS4_SyslineFind::Found((fo_next, slp));
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
                    let fo = range_fo.1;
                    let slp = self.syslines[fo].clone();
                    // XXX: multi-byte character encoding
                    let fo_next = (*slp).fileoffset_next() + (self.charsz() as FileOffset);
                    if self.is_sysline_last(&slp) {
                        debug_eprintln!(
                            "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                            sx(),
                            fo_next,
                            &*slp,
                            (*slp).fileoffset_begin(),
                            (*slp).fileoffset_end(),
                            (*slp).to_String_noraw()
                        );
                        if self._find_sysline_lru_cache_enabled {
                            self._find_sysline_lru_cache_put += 1;
                            self._find_sysline_lru_cache
                                .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_next, slp.clone())));
                        }
                        return ResultS4_SyslineFind::Found_EOF((fo_next, slp));
                    }
                    if self._find_sysline_lru_cache_enabled {
                        self._find_sysline_lru_cache_put += 1;
                        self._find_sysline_lru_cache
                            .put(fileoffset, ResultS4_SyslineFind::Found((fo_next, slp.clone())));
                    }
                    debug_eprintln!(
                        "{}find_sysline: return ResultS4_SyslineFind::Found(({}, @{:p})) @[{}, {}] in self.syslines_by_range {:?}",
                        sx(),
                        fo_next,
                        &*slp,
                        (*slp).fileoffset_begin(),
                        (*slp).fileoffset_end(),
                        (*slp).to_String_noraw()
                    );
                    return ResultS4_SyslineFind::Found((fo_next, slp));
                }
                None => {
                    debug_eprintln!("{}find_sysline: fileoffset {} not found in self.syslines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_sysline: searching for first sysline datetime B …", so());
            */
        }

        let mut fo_b: FileOffset = fo1;
        let mut eof = false;
        loop {
            debug_eprintln!("{}find_sysline: self.linereader.find_line({})", so(), fo1);
            let result = self.linereader.find_line(fo1);
            let (fo2, lp) = match result {
                ResultS4_LineFind::Found((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found(FileOffset {}, Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Found_EOF((fo_, lp_)) => {
                    debug_eprintln!(
                        "{}find_sysline: B got Found_EOF(FileOffset {} Line @{:p}) len {} parts {} {:?}",
                        so(),
                        fo_,
                        &*lp_,
                        (*lp_).len(),
                        (*lp_).count(),
                        (*lp_).to_String_noraw()
                    );
                    eof = true;
                    //assert!(!eof, "ERROR: find_line returned EOF as true yet returned Found_EOF()");
                    (fo_, lp_)
                }
                ResultS4_LineFind::Done => {
                    //debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Done; B", sx());
                    debug_eprintln!("{}find_sysline: break; B", so());
                    eof = true;
                    break;
                }
                ResultS4_LineFind::Err(err) => {
                    eprintln!("ERROR: LineReader.find_line({}) returned {}", fo1, err);
                    debug_eprintln!("{}find_sysline: return ResultS4_SyslineFind::Err({}); B from LineReader.find_line({})", sx(), err, fo1);
                    return ResultS4_SyslineFind::Err(err);
                }
            };
            let resultp = self.parse_datetime_in_line_cached(&lp, &self.charsz());
            debug_eprintln!("{}find_sysline: B find_datetime_in_line returned {:?}", so(), resultp);
            match *resultp {
                Err(_) => {
                    debug_eprintln!(
                        "{}find_sysline: B append found Line to this Sysline sl.push({:?})",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    sl.push(lp);
                }
                Ok(_) => {
                    // a datetime was found! end of this sysline, beginning of a new sysline
                    debug_eprintln!(
                        "{}find_sysline: B found datetime; end of this Sysline. Do not append found Line {:?}",
                        so(),
                        (*lp).to_String_noraw()
                    );
                    fo_b = fo1;
                    break;
                }
            }
            fo1 = fo2;
        }

        debug_eprintln!("{}find_sysline: found line with datetime B at FileOffset {} {:?}", so(), fo_b, sl.to_String_noraw());

        let slp = self.insert_sysline(sl);
        if eof {
            if self._find_sysline_lru_cache_enabled {
                self._find_sysline_lru_cache_put += 1;
                debug_eprintln!("{}find_sysline: LRU cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_b);
                self._find_sysline_lru_cache
                    .put(fileoffset, ResultS4_SyslineFind::Found_EOF((fo_b, slp.clone())));
            }
            debug_eprintln!(
                "{}find_sysline: return ResultS4_SyslineFind::Found_EOF(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
                sx(),
                fo_b,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).fileoffset_end(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found_EOF((fo_b, slp));
        }
        if self._find_sysline_lru_cache_enabled {
            self._find_sysline_lru_cache_put += 1;
            debug_eprintln!("{}find_sysline: LRU cache put({}, Found({}, …))", so(), fileoffset, fo_b);
            self._find_sysline_lru_cache
                .put(fileoffset, ResultS4_SyslineFind::Found((fo_b, slp.clone())));
        }
        debug_eprintln!(
            "{}find_sysline: return ResultS4_SyslineFind::Found(({}, SyslineP@{:p}) @[{}, {}] E {:?}",
            sx(),
            fo_b,
            &*slp,
            (*slp).fileoffset_begin(),
            (*slp).fileoffset_end(),
            (*slp).to_String_noraw()
        );

        ResultS4_SyslineFind::Found((fo_b, slp))
    }

    /// wrapper to call each implementation of `find_sysline_at_datetime_filter`
    pub fn find_sysline_at_datetime_filter(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        self.find_sysline_at_datetime_filter1(fileoffset, dt_filter)
    }

    /// find first sysline at or after `fileoffset` that is at or after `dt_filter`
    ///
    /// for example, given syslog file with datetimes:
    ///     20010101
    ///     20010102
    ///     20010103
    /// where the newline ending the first line is the ninth byte (fileoffset 9)
    ///
    /// calling
    ///     syslinereader.find_sysline_at_datetime_filter(0, Some(20010102 00:00:00-0000))
    /// will return
    ///     ResultS4::Found(19, SyslineP(data='20010102␊'))
    ///
    /// TODO: add more of these examples
    ///
    /// XXX: this function is large, cumbersome, and messy
    fn find_sysline_at_datetime_filter1(
        &mut self, fileoffset: FileOffset, dt_filter: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_at_datetime_filter1";
        debug_eprintln!("{}{}(SyslineReader@{:p}, {}, {:?})", sn(), _fname, self, fileoffset, dt_filter);
        let filesz = self.filesz();
        let _fo_end: FileOffset = filesz as FileOffset;
        let mut try_fo: FileOffset = fileoffset;
        let mut try_fo_last: FileOffset = try_fo;
        let mut fo_last: FileOffset = fileoffset;
        let mut slp_opt: Option<SyslineP> = None;
        let mut slp_opt_last: Option<SyslineP> = None;
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
                ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
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
                        &(*slp),
                        slp.lines.len(),
                        (*slp).len(),
                        (*slp).to_String_noraw(),
                    );
                    // here is the binary search algorithm in action
                    debug_eprintln!(
                        "{}{}: sysline_dt_after_or_before(@{:p} ({:?}), {:?})",
                        so(),
                        _fname,
                        &*slp,
                        (*slp).dt,
                        dt_filter
                    );
                    match SyslineReader::sysline_dt_after_or_before(&slp, dt_filter) {
                        Result_Filter_DateTime1::Pass => {
                            debug_eprintln!(
                                "{}{}: Pass => fo {} fo_last {} try_fo {} try_fo_last {} (fo_end {})",
                                so(),
                                _fname,
                                fo,
                                fo_last,
                                try_fo,
                                try_fo_last,
                                _fo_end
                            );
                            debug_eprintln!(
                                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); A",
                                sx(),
                                _fname,
                                fo,
                                &*slp
                            );
                            return ResultS4_SyslineFind::Found((fo, slp));
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
                                    try_fo_last
                                );
                                debug_eprintln!(
                                    "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); B fileoffset {} {:?}",
                                    sx(),
                                    _fname,
                                    fo,
                                    &*slp,
                                    (*slp).fileoffset_begin(),
                                    (*slp).to_String_noraw()
                                );
                                return ResultS4_SyslineFind::Found((fo, slp));
                            }
                            try_fo_last = try_fo;
                            fo_b = std::cmp::min((*slp).fileoffset_begin(), try_fo_last);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}, FPath {:?}", fo_a, fo_b, self.path());
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursAtOrAfter
                        Result_Filter_DateTime1::OccursBefore => {
                            // the Sysline found by `find_sysline(try_fo)` occurs before filter `dt_filter`, so search forthward
                            // i.e. move begin marker `fo_a` forthward
                            debug_eprintln!("{}{}: OccursBefore =>    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                            let slp_foe = (*slp).fileoffset_end();
                            // XXX: [2022/03/25] why was this `assert_le` here? It seems wrong.
                            //assert_le!(slp_foe, fo, "unexpected values (SyslineP@{:p}).fileoffset_end() {}, fileoffset returned by self.find_sysline({}) was {} FPath {:?}", slp, slp_foe, try_fo, fo, self.path());
                            try_fo_last = try_fo;
                            assert_le!(try_fo_last, slp_foe, "Unexpected values try_fo_last {} slp_foe {}, last tried offset (passed to self.find_sysline({})) is beyond returned Sysline@{:p}.fileoffset_end() {}!? FPath {:?}", try_fo_last, slp_foe, try_fo, slp, slp_foe, self.path());
                            debug_eprintln!(
                                "{}{}:                    ∴ fo_a = min(slp_foe {}, fo_b {});",
                                so(),
                                _fname,
                                slp_foe,
                                fo_b
                            );
                            // LAST WORKING HERE [2021/10/06 00:05:00]
                            // LAST WORKING HERE [2022/03/16 01:15:00]
                            // this code passes all tests, but runs strangely. I think the problem is the first found sysline
                            // (that may or may not satisfy the passed filter) is placed into a queue and then printed by the waiting main thread.
                            fo_a = std::cmp::min(slp_foe, fo_b);
                            //fo_a = std::cmp::max(slp_foe, fo_b);
                            //fo_a = slp_foe;
                            //assert_le!(fo_a, fo_b, "Unexpected values for fo_a {} fo_b {}", fo_a, fo_b);
                            debug_eprintln!(
                                "{}{}:                    ∴ try_fo = fo_a {} + ((fo_b {} - {} fo_a) / 2);",
                                so(),
                                _fname,
                                fo_a,
                                fo_b,
                                fo_a
                            );
                            try_fo = fo_a + ((fo_b - fo_a) / 2);
                        } // end OccursBefore
                    } // end SyslineReader::sysline_dt_after_or_before()
                    debug_eprintln!("{}{}:                    fo {} fo_last {} try_fo {} try_fo_last {} fo_a {} fo_b {} (fo_end {})", so(), _fname, fo, fo_last, try_fo, try_fo_last, fo_a, fo_b, _fo_end);
                    fo_last = fo;
                    slp_opt_last = slp_opt;
                    slp_opt = Some(slp);
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
                ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!(
                        "{}{}: SyslineReader.find_sysline(try_fo: {}) returned Err({})",
                        so(),
                        _fname,
                        try_fo,
                        err
                    );
                    eprintln!("ERROR: {}", err);
                    break;
                } // end Err
            } // match result
            debug_eprintln!("{}{}: next loop will try offset {} (fo_end {})", so(), _fname, try_fo, _fo_end);

            // TODO: 2022/03/18 this latter part hints at a check that could be done sooner,
            //       before `try_fo==try_fo_last`, that would result in a bit less loops.
            //       A simpler and faster check is to do
            //           fo_next, slp = find_sysline(fileoffset)
            //           _, slp_next = find_sysline(fo_next)
            //       do this at the top of the loop. Then call `dt_after_or_before` for each
            //       `.dt` among `slp`, `slp_next`.

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
            let mut slp = slp_opt.unwrap();
            let fo_beg = slp.fileoffset_begin();
            if self.is_sysline_last(&slp) && fo_beg < try_fo {
                // binary search stopped at fileoffset past start of last Sysline in file
                // so entirely past all acceptable syslines
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; C binary searched ended after beginning of last sysline in the file", sx(), _fname,);
                return ResultS4_SyslineFind::Done;
            }
            // binary search loop is deciding on the same fileoffset upon each loop. That fileoffset must refer to
            // an acceptable sysline. However, if that fileoffset is past `slp.fileoffset_begin` than the threshold
            // change of datetime for the `dt_filter` is the *next* Sysline.
            let fo_next = slp.fileoffset_next();
            // XXX: sanity check
            //debug_assert_eq!(fo_last, fo_next, "fo {} != {} slp.fileoffset_next()", fo_last, fo_next);
            if fo_beg < try_fo {
                debug_eprintln!("{}{}: slp.fileoffset_begin() {} < {} try_fo;", so(), _fname, fo_beg, try_fo);
                let slp_next = match self.find_sysline(fo_next) {
                    ResultS4_SyslineFind::Found_EOF((_, slp_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found_EOF(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            slp_
                        );
                        slp_
                    }
                    ResultS4_SyslineFind::Found((_, slp_)) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Found(…, {:?})",
                            so(),
                            _fname,
                            fo_next,
                            slp_
                        );
                        slp_
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
                    ResultS4_SyslineFind::Err(err) => {
                        debug_eprintln!(
                            "{}{}: SyslineReader.find_sysline(fo_next1: {}) returned Err({})",
                            so(),
                            _fname,
                            fo_next,
                            err
                        );
                        eprintln!("ERROR: {}", err);
                        break;
                    }
                };
                debug_eprintln!("{}{}: dt_filter:                   {:?}", so(), _fname, dt_filter);
                debug_eprintln!(
                    "{}{}: slp      : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    fo_beg,
                    (*slp).fileoffset_end(),
                    (*slp).dt.unwrap(),
                    (*slp).to_String_noraw()
                );
                debug_eprintln!(
                    "{}{}: slp_next : fo_beg {:3}, fo_end {:3} {:?} {:?}",
                    so(),
                    _fname,
                    (*slp_next).fileoffset_begin(),
                    (*slp_next).fileoffset_end(),
                    (*slp_next).dt.unwrap(),
                    (*slp_next).to_String_noraw()
                );
                let slp_compare = dt_after_or_before(&(*slp).dt.unwrap(), dt_filter);
                let slp_next_compare = dt_after_or_before(&(*slp_next).dt.unwrap(), dt_filter);
                debug_eprintln!("{}{}: match({:?}, {:?})", so(), _fname, slp_compare, slp_next_compare);
                slp = match (slp_compare, slp_next_compare) {
                    (_, Result_Filter_DateTime1::Pass) | (Result_Filter_DateTime1::Pass, _) => {
                        debug_eprintln!("{}{}: unexpected Result_Filter_DateTime1::Pass", so(), _fname);
                        eprintln!("ERROR: unexpected Result_Filter_DateTime1::Pass result");
                        break;
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursBefore) => {
                        debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                        slp_next
                    }
                    (Result_Filter_DateTime1::OccursBefore, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing slp_next", so(), _fname);
                        slp_next
                    }
                    (Result_Filter_DateTime1::OccursAtOrAfter, Result_Filter_DateTime1::OccursAtOrAfter) => {
                        debug_eprintln!("{}{}: choosing slp", so(), _fname);
                        slp
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
                    "{}{}: slp.fileoffset_begin() {} >= {} try_fo; use slp",
                    so(),
                    _fname,
                    fo_beg,
                    try_fo
                );
            }
            let fo_ = slp.fileoffset_next();
            debug_eprintln!(
                "{}{}: return ResultS4_SyslineFind::Found(({}, @{:p})); D fileoffset {} {:?}",
                sx(),
                _fname,
                fo_,
                &*slp,
                (*slp).fileoffset_begin(),
                (*slp).to_String_noraw()
            );
            return ResultS4_SyslineFind::Found((fo_, slp));
        } // end loop

        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Done; E", sx(), _fname);

        ResultS4_SyslineFind::Done
    }


    /// convenience wrapper for `dt_after_or_before`
    pub fn sysline_dt_after_or_before(syslinep: &SyslineP, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
        debug_eprintln!("{}sysline_dt_after_or_before(SyslineP@{:p}, {:?})", snx(), &*syslinep, dt_filter,);
        assert!((*syslinep).dt.is_some(), "Sysline@{:p} does not have a datetime set.", &*syslinep);

        let dt = (*syslinep).dt.unwrap();

        dt_after_or_before(&dt, dt_filter)
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
        let dt = (*syslinep).dt.unwrap();
        let result = dt_pass_filters(&dt, dt_filter_after, dt_filter_before);
        debug_eprintln!("{}sysline_pass_filters(…) return {:?};", sx(), result);

        result
    }

    /// find the first `Sysline`, starting at `fileoffset`, that is at or after datetime filter
    /// `dt_filter_after` and before datetime filter `dt_filter_before`
    pub fn find_sysline_between_datetime_filters(
        &mut self, fileoffset: FileOffset, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        let _fname = "find_sysline_between_datetime_filters";
        debug_eprintln!("{}{}({}, {:?}, {:?})", sn(), _fname, fileoffset, dt_filter_after, dt_filter_before);

        match self.find_sysline_at_datetime_filter(fileoffset, dt_filter_after) {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                debug_eprintln!(
                "{}{}: find_sysline_at_datetime_filter returned ResultS4_SyslineFind::Found(({}, {:?})); call sysline_pass_filters",
                    so(),
                    _fname,
                    fo,
                    slp,
                );
                match Self::sysline_pass_filters(&slp, dt_filter_after, dt_filter_before) {
                    Result_Filter_DateTime2::InRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned InRange;", so(), _fname);
                        debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found(({}, {:?}))", sx(), _fname, fo, slp);
                        return ResultS4_SyslineFind::Found((fo, slp));
                    },
                    Result_Filter_DateTime2::BeforeRange => {
                        debug_eprintln!("{}{}: sysline_pass_filters(…) returned BeforeRange;", so(), _fname);
                        eprintln!("ERROR: sysline_pass_filters(Sysline@{:p}, {:?}, {:?}) returned BeforeRange, however the prior call to find_sysline_at_datetime_filter({}, {:?}) returned Found; this is unexpected.",
                                  slp, dt_filter_after, dt_filter_before,
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
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Found_EOF(({}, {:?}))", sx(), _fname, fo, slp);
                return ResultS4_SyslineFind::Found_EOF((fo, slp));
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
                eprintln!("ERROR: {}", err);
                debug_eprintln!("{}{}: return ResultS4_SyslineFind::Err({})", sx(), _fname, err);
                return ResultS4_SyslineFind::Err(err);
            },
        };

        debug_eprintln!("{}{} return ResultS4_SyslineFind::Done", sx(), _fname);

        ResultS4_SyslineFind::Done
    }
    
    /// return an up-to-date `Summary` instance for this `SyslineReader`
    pub fn summary(&self) -> Summary {
        let BlockReader_bytes = self.linereader.blockreader.count_bytes();
        let BlockReader_bytes_total = self.linereader.blockreader.filesz as u64;
        let BlockReader_blocks = self.linereader.blockreader.count();
        let BlockReader_blocks_total = self.linereader.blockreader.blockn;
        let BlockReader_blocksz = self.blocksz();
        let LineReader_lines = self.linereader.count();
        let SyslineReader_syslines = self.count();
        let SyslineReader_patterns = self.dt_patterns.clone();
        let SyslineReader_find_sysline_lru_cache_hit = self._find_sysline_lru_cache_hit;
        let SyslineReader_find_sysline_lru_cache_miss = self._find_sysline_lru_cache_miss;
        let SyslineReader_find_sysline_lru_cache_put = self._find_sysline_lru_cache_put;
        let SyslineReader_parse_datetime_in_line_lru_cache_hit = self._parse_datetime_in_line_lru_cache_hit;
        let SyslineReader_parse_datetime_in_line_lru_cache_miss = self._parse_datetime_in_line_lru_cache_miss;
        let SyslineReader_parse_datetime_in_line_lru_cache_put = self._parse_datetime_in_line_lru_cache_put;
        let LineReader_find_line_lru_cache_hit = self.linereader._find_line_lru_cache_hit;
        let LineReader_find_line_lru_cache_miss = self.linereader._find_line_lru_cache_miss;
        let LineReader_find_line_lru_cache_put = self.linereader._find_line_lru_cache_put;
        let BlockReader_read_block_cache_lru_hit = self.linereader.blockreader._read_block_cache_lru_hit;
        let BlockReader_read_block_cache_lru_miss = self.linereader.blockreader._read_block_cache_lru_miss;
        let BlockReader_read_block_cache_lru_put = self.linereader.blockreader._read_block_cache_lru_put;
        let BlockReader_read_blocks_hit = self.linereader.blockreader._read_blocks_hit;
        let BlockReader_read_blocks_miss = self.linereader.blockreader._read_blocks_miss;
        let BlockReader_read_blocks_insert = self.linereader.blockreader._read_blocks_insert;

        Summary::new(
            BlockReader_bytes,
            BlockReader_bytes_total,
            BlockReader_blocks,
            BlockReader_blocks_total,
            BlockReader_blocksz,
            LineReader_lines,
            SyslineReader_syslines,
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
            BlockReader_read_block_cache_lru_hit,
            BlockReader_read_block_cache_lru_miss,
            BlockReader_read_block_cache_lru_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
        )
    }
}


// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogWriter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// XXX: unfinished attempt at `Printer` or `Writer` "class"
/*
type SyslineReaders<'syslogwriter> = Vec<SyslineReader<'syslogwriter>>;

/// Specialized Writer that coordinates writing multiple SyslineReaders
pub struct SyslogWriter<'syslogwriter> {
    syslinereaders: SyslineReaders<'syslogwriter>,
}

impl<'syslogwriter> SyslogWriter<'syslogwriter> {
    pub fn new(syslinereaders: SyslineReaders<'syslogwriter>) -> SyslogWriter<'syslogwriter> {
        assert_gt!(syslinereaders.len(), 0, "Passed zero SyslineReaders");
        SyslogWriter { syslinereaders }
    }

    pub fn push(&mut self, syslinereader: SyslineReader<'syslogwriter>) {
        self.syslinereaders.push(syslinereader);
    }
}
*/
