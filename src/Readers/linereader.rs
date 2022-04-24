// Readers/linereader.rs
//

pub use crate::common::{
    FPath,
    FileOffset,
    CharSz,
    NLu8,
};

use crate::Readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    BlockP,
    Slices,
    BlockReader,
    ResultS3_ReadBlock,
};

use crate::common::{
    Bytes,
    ResultS4,
};

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    byte_to_char_noraw,
    buffer_to_String_noraw,
    char_to_char_noraw,
};

use crate::dbgpr::stack::{
    sn,
    snx,
    so,
    sx,
};

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::io::{
    Error,
    Result,
};
use std::io::prelude::*;
use std::sync::Arc;

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};
#[allow(unused_imports)]
use debug_print::{debug_print, debug_println};

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
    //debug_assert_ge,
    debug_assert_gt,
};

/// A sequence to track a `Line`.
/// A "line" may span multiple `Block`s. One `LinePart` is needed for each `Block`.
pub type LineParts = Vec<LinePart>;
/// A sequence to track one or more `LineP` that make up a `Sysline` 
pub type Lines = Vec<LineP>;
/// An offset into a `Line`
pub type LineIndex = usize;
/// thread-safe Atomic Reference Counting pointer to a `Line`
pub type LineP = Arc<Line>;
/// storage for Lines found from the underlying `BlockReader`
/// FileOffset key is the first byte/offset that begins the `Line`
pub type FoToLine = BTreeMap<FileOffset, LineP>;
pub type FoToFo = BTreeMap<FileOffset, FileOffset>;
/// `LineReader.find_line` searching results
#[allow(non_camel_case_types)]
pub type ResultS4_LineFind = ResultS4<(FileOffset, LineP), Error>;
pub type LinesLRUCache = LruCache<FileOffset, ResultS4_LineFind>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Struct describing a part or all of a line within a `Block`
/// A "line" can span more than one `Block`. This tracks part or all of a line within
/// one `Block`. One `LinePart` to one `Block`.
/// But one or more `LinePart` are necessary to represent an entire "line".
pub struct LinePart {
    /// index into the `blockp`, index at beginning
    /// used as-is in slice notation
    pub blocki_beg: BlockIndex,
    /// index into the `blockp`, index at one after ending '\n' (may refer to one past end of `Block`)
    /// used as-is in slice notation
    pub blocki_end: BlockIndex,
    /// the `Block` pointer
    pub blockp: BlockP,
    /// the byte offset into the file where this `LinePart` begins
    pub fileoffset: FileOffset,
    /// debug helper, might be good to get rid of this?
    pub blockoffset: BlockOffset,
    /// debug helper, might be good to get rid of this?
    pub blocksz: BlockSz,
    // TODO: add size of *this* block
}

impl fmt::Debug for LinePart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LinePart")
            .field("LinePart @", &format_args!("{:p}", &self))
            .field("blocki_beg", &self.blocki_beg)
            .field("blocki_end", &self.blocki_end)
            .field("len", &self.len())
            .field("blockp @", &format_args!("{:p}", &(*self.blockp)))
            .field("fileoffset", &self.fileoffset)
            .field("blockoffset", &self.blockoffset)
            .finish()
    }
}

impl LinePart {
    // XXX: does not handle multi-byte encodings
    const CHARSZ: usize = 1;

    pub fn new(
        blocki_beg: BlockIndex,
        blocki_end: BlockIndex,
        blockp: BlockP,
        fileoffset: FileOffset,
        blockoffset: BlockOffset,
        blocksz: BlockSz,
    ) -> LinePart {
        debug_eprintln!(
            "{}LinePart::new(blocki_beg {}, blocki_end {}, Block @{:p}, fileoffset {}, blockoffset {}, blocksz {}) (blockp.len() {})",
            so(),
            blocki_beg,
            blocki_end,
            &*blockp,
            fileoffset,
            blockoffset,
            blocksz,
            (*blockp).len(),
        );
        // some sanity checks
        assert_ne!(fileoffset, FileOffset::MAX, "Bad fileoffset MAX");
        assert_ne!(blockoffset, BlockOffset::MAX, "Bad blockoffset MAX");
        let fo1 = BlockReader::file_offset_at_block_offset(blockoffset, blocksz);
        assert_le!(fo1, fileoffset, "Bad FileOffset {}, must ≥ {} (based on file_offset_at_block_offset(BlockOffset {}, BlockSz {}))", fileoffset, fo1, blockoffset, blocksz);
        let fo2 = BlockReader::file_offset_at_block_offset(blockoffset + 1, blocksz);
        assert_le!(fileoffset, fo2, "Bad FileOffset {}, must ≤ {} (based on file_offset_at_block_offset(BlockOffset {}, BlockSz {}))", fileoffset, fo2, blockoffset + 1, blocksz);
        let bo = BlockReader::block_offset_at_file_offset(fileoffset, blocksz);
        assert_eq!(blockoffset, bo, "Bad BlockOffset {}, expected {} (based on block_offset_at_file_offset(FileOffset {}, BlockSz {}))", blockoffset, bo, fileoffset, blocksz);
        let bi = BlockReader::block_index_at_file_offset(fileoffset, blocksz);
        assert_eq!(
            blocki_beg, bi,
            "blocki_beg {} ≠ {} block_index_at_file_offset({}, {})",
            blocki_beg, bi, fileoffset, blocksz
        );
        assert_ne!(blocki_end, 0, "Bad blocki_end 0, expected > 0");
        assert_lt!(blocki_beg, blocki_end, "blocki_beg {} should be < blocki_end {}", blocki_beg, blocki_end);
        assert_lt!((blocki_beg as BlockSz), blocksz, "blocki_beg {} should be < blocksz {}", blocki_beg, blocksz);
        assert_le!((blocki_end as BlockSz), blocksz, "blocki_end {} should be ≤ blocksz {}", blocki_end, blocksz);
        assert_le!(((*blockp).len() as BlockSz), blocksz, "block.len() {} should be ≤ blocksz {}", (*blockp).len(), blocksz);
        assert_ge!((*blockp).len(), blocki_end - blocki_beg, "block.len() {} should be ≥ {} (blocki_end {} - {} blocki_beg)", (*blockp).len(), blocki_end - blocki_beg, blocki_end, blocki_beg);
        LinePart {
            blocki_beg,
            blocki_end,
            blockp,
            fileoffset,
            blockoffset,
            blocksz,
        }
    }

    /// length of line starting at index `blocki_beg`
    pub fn len(&self) -> usize {
        (self.blocki_end - self.blocki_beg) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// count of bytes of this `LinePart`
    /// XXX: `count_bytes` and `len` is overlapping and confusing.
    pub fn count_bytes(&self) -> u64 {
        (self.len() * LinePart::CHARSZ) as u64
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub(self) fn _to_String_raw(self: &LinePart, raw: bool) -> String {
        // XXX: intermixing byte lengths and character lengths
        // XXX: does not handle multi-byte
        let s1: String;
        let slice_ = &(*self.blockp)[self.blocki_beg..self.blocki_end];
        if raw {
            unsafe {
                s1 = String::from_utf8_unchecked(Vec::<u8>::from(slice_));
            }
            return s1;
        }
        s1 = buffer_to_String_noraw(slice_);
        s1
    }

    pub fn contains(self: &LinePart, byte_: &u8) -> bool {
        (*self.blockp).contains(byte_)
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &LinePart) -> String {
        self._to_String_raw(false)
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &LinePart) -> String {
        self._to_String_raw(true)
    }

    /// return Box pointer to slice of bytes that make up this `LinePart`
    pub fn block_boxptr(&self) -> Box<&[u8]> {
        let slice_ = &(*self.blockp).as_slice()[self.blocki_beg..self.blocki_end];
        //let slice_ptr: *const &[u8] = **slice_;
        Box::new(slice_)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from `a` to end
    pub fn block_boxptr_a(&self, a: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..self.blocki_end];
        //let slice2 = &slice1[*a..];
        Box::new(slice1)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from beginning to `b`
    pub fn block_boxptr_b(&self, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[..self.blocki_beg+b];
        //let slice2 = &slice1[..*b];
        Box::new(slice1)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from `a` to `b`
    pub fn block_boxptr_ab(&self, a: &LineIndex, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(a, b, "bad LineIndex");
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        debug_assert_lt!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        debug_assert_lt!(b - a, self.len(), "Passed LineIndex {}..{} (diff {}) are larger than this LinePart 'slice' {}", a, b, b - a, self.len());
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..(self.blocki_beg+b)];
        //let slice2 = &slice1[*a..*b];
        Box::new(slice1)
    }
}

/// A `Line` has information about a "line" that may or may not span more than one `Block`
pub struct Line {
    pub(crate) lineparts: LineParts,
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut li_s = String::new();
        for li in self.lineparts.iter() {
            li_s.push_str(&format!(
                " @{:p} (blocki_beg {}, blocki_end {}, len() {}, BlockP.len() {}, fileoffset {}, blockoffset {})",
                &li,
                &li.blocki_beg,
                &li.blocki_end,
                &li.len(),
                &li.blockp.len(),
                &li.fileoffset,
                &li.blockoffset
            ));
        }
        let mut fo_b = 0;
        if !self.lineparts.is_empty() {
            fo_b = self.lineparts[0].fileoffset;
        }
        let mut fo_e = 0;
        if !self.lineparts.is_empty() {
            let last_li = self.lineparts.len() - 1;
            fo_e = self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1;
        }
        f.debug_struct("Line")
            .field("line.fileoffset_begin()", &fo_b)
            .field("line.fileoffset_end()", &fo_e)
            .field("lineparts @", &format_args!("{:p}", &self))
            .field("lineparts.len", &self.lineparts.len())
            .field("lineparts", &li_s)
            .finish()
    }
}

/// return value for `Line::get_boxptrs`
pub enum enum_BoxPtrs <'a> {
    SinglePtr(Box<&'a [u8]>),
    MultiPtr(Vec<Box<&'a [u8]>>),
}

impl Default for Line {
    fn default() -> Self {
        Self {
            lineparts: LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY),
        }
    }
}

impl Line {
    /// default `with_capacity` for a `LineParts`, most often will only need 1 capacity
    /// as the found "line" will likely reside within one `Block`
    const LINE_PARTS_WITH_CAPACITY: usize = 1;

    pub fn new() -> Line {
        Line::default()
    }

    pub fn new_from_linepart(linepart: LinePart) -> Line {
        let mut v = LineParts::with_capacity(Line::LINE_PARTS_WITH_CAPACITY);
        v.push(linepart);
        Line {
            lineparts: v,
        }
    }

    /// insert `linepart` at back of `self.lineparts`
    /// TODO: rename to `append`
    pub fn push(&mut self, linepart: LinePart) {
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li = &self.lineparts[l_ - 1];
            assert_le!(
                li.blockoffset,
                linepart.blockoffset,
                "Line.push: Prior stored LinePart at blockoffset {} is after passed LinePart at blockoffset {}",
                li.blockoffset,
                linepart.blockoffset,
            );
            assert_lt!(
                li.fileoffset,
                linepart.fileoffset,
                "Line.push: Prior stored LinePart at fileoffset {} is at or after passed LinePart at fileoffset {}",
                li.fileoffset,
                linepart.fileoffset,
            );
        }
        self.lineparts.push(linepart);
    }

    /// insert `linepart` at front of `self.lineparts`
    pub fn prepend(&mut self, linepart: LinePart) {
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li: &LinePart = &self.lineparts[0];
            assert_ge!(
                li.blockoffset,
                linepart.blockoffset,
                "Line.prepend: Prior stored LinePart at blockoffset {} is before passed LinePart at blockoffset {} (should be after)",
                li.blockoffset,
                linepart.blockoffset,
            );
            assert_gt!(
                li.fileoffset,
                linepart.fileoffset,
                "Line.prepend: Prior stored LinePart at fileoffset {} is at or before passed LinePart at fileoffset {} (should be after)",
                li.fileoffset,
                linepart.fileoffset,
            );
        }
        self.lineparts.insert(0, linepart);
    }

    /// the byte offset into the file where this `Line` begins
    /// "points" to first character of `Line`
    pub fn fileoffset_begin(self: &Line) -> FileOffset {
        debug_assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        self.lineparts[0].fileoffset
    }

    /// the byte offset into the file where this `Line` ends, inclusive (not one past ending)
    pub fn fileoffset_end(self: &Line) -> FileOffset {
        debug_assert_ne!(self.lineparts.len(), 0, "This Line has no `LinePart`");
        let last_li = self.lineparts.len() - 1;
        self.lineparts[last_li].fileoffset + (self.lineparts[last_li].len() as FileOffset) - 1
    }

    /// length of this `Line` in bytes
    pub fn len(self: &Line) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `LinePart` in `self.lineparts.len()`
    /// XXX: redundant, need to decide which to keep.
    pub fn count(self: &Line) -> usize {
        self.lineparts.len()
    }

    /// sum of `LinePart.count_bytes`
    pub fn count_bytes(self: &Line) -> u64 {
        let mut cb: u64 = 0;
        for lp in self.lineparts.iter() {
            cb += lp.count_bytes();
        }
        cb
    }

    pub fn get_linepart(self: &Line, mut a: LineIndex) -> &LinePart {
        for linepart in self.lineparts.iter() {
            let len_ = linepart.len();
            if a < len_ {
                return linepart;
            }
            a -= len_;
        }
        // XXX: not sure if this is the best choice
        self.lineparts.last().unwrap()
    }

    /// does the `Line` contain the byte value?
    pub fn contains(self: &Line, byte_: &u8) -> bool {
        for linepart in self.lineparts.iter() {
            if linepart.contains(byte_) {
                return true;
            }
        }
        false
    }

    /// does the `Line` contain the byte value?
    pub fn contains_at(self: &Line, byte_: &u8, a: &LineIndex, b: &LineIndex) -> bool {
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        for linepart in self.lineparts.iter() {
            if linepart.contains(byte_) {
                return true;
            }
        }
        false
    }

    /// return all slices that make up this `Line`
    /// CANDIDATE FOR REMOVAL?
    pub fn get_slices(self: &Line) -> Slices {
        // short-circuit this case
        let sz = self.lineparts.len();
        let mut slices = Slices::with_capacity(sz);
        for linepart in self.lineparts.iter() {
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            slices.push(slice);
        }

        slices
    }

    /// return a count of slices that would be returned by `get_slices`
    /// CANDIDATE FOR REMOVAL?
    pub fn get_slices_count(self: &Line) -> usize {
        self.lineparts.len()
    }


    /// get Box pointers to the underlying `&[u8]` slice that makes up this `Line`.
    /// There may be more than one slice as the `Line` may cross block boundaries. So
    /// return the sequence of Box pointers in a `Vec`.
    /// TODO: the `Vec<Box<&[u8]>>` creation is expensive
    ///       consider allowing a mut &Vec to be passed in. However, this will require declaring lifetimes!
    ///       LAST WORKING HERE 2022/04/03 23:54:00
    // TODO: due to unstable feature `Sized` in `Box`, cannot do
    //           fn get_boxptrs(...) -> either::Either<Box<&[u8]>, Vec<Box<&[u8]>>>
    //       causes error `experimental Sized`
    pub fn get_boxptrs(self: &Line, mut a: LineIndex, mut b: LineIndex) -> enum_BoxPtrs<'_> {
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        // do the simple case first (single `Box` pointer required)
        // doing this here, as opposed to intermixing with multiple case, avoids compiler complaint of "use of possibly-uninitialized `ptrs`"
        let mut a1: LineIndex = a;
        let mut b1: LineIndex = b;
        for linepart_ in &self.lineparts {
            let len_ = linepart_.len();
            if a1 < len_ && b1 < len_ {
                return enum_BoxPtrs::SinglePtr(linepart_.block_boxptr_ab(&a1, &b1));
            } else if a1 < len_ && len_ <= b1 {
                break;
            }
            a1 -= len_;
            b1 -= len_;
        }
        // do the harder case (multiple `Box` pointers required)
        let mut a_found = false;
        let mut b_search = false;
        let mut ptrs: Vec<Box<&[u8]>> = Vec::<Box::<&[u8]>>::new();
        for linepart_ in &self.lineparts {
            debug_eprintln!("{}get_boxptrs: linepart {:?}", so(), linepart_.to_String_noraw());
            let len_ = linepart_.len();
            if !a_found && a < len_ {
                a_found = true;
                b_search = true;
                if b < len_ {
                    debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_ab({}, {}))", so(), a, b);
                    ptrs.push(linepart_.block_boxptr_ab(&a, &b));  // store [a..b]  (entire slice, entire `Line`)
                    debug_assert_gt!(ptrs.len(), 1, "ptrs is {} elements, expected >= 1; this should have been handled earlier", ptrs.len());
                    return enum_BoxPtrs::MultiPtr(ptrs);
                }
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_a({}))", so(), a);
                ptrs.push(linepart_.block_boxptr_a(&a));  // store [a..]  (first slice of `Line`)
                b -= len_;
                continue;
            } else if !a_found {
                a -= len_;
                continue;
            }
            if b_search && b < len_ {
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr_b({}))", so(), b);
                ptrs.push(linepart_.block_boxptr_b(&b));  // store [..b] (last slice of `Line`)
                break;
            } else  {
                debug_eprintln!("{}get_boxptrs: ptrs.push(linepart_.block_boxptr())", so());
                ptrs.push(linepart_.block_boxptr());  // store [..] (entire slice, middle part of `Line`)
                b -= len_;
            }
        }
        enum_BoxPtrs::MultiPtr(ptrs)
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    /// XXX: `raw==false` does not handle multi-byte encodings
    #[cfg(any(debug_assertions,test))]
    pub fn print(self: &Line, raw: bool) {
        // is this an expensive command? should `stdout` be cached?
        let stdout = io::stdout();
        let mut stdout_lock = stdout.lock();
        for linepart in &self.lineparts {
            // TODO: I'm somewhat sure this is not creating anything new but I should verify with `gdb-rust`.
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            if raw {
                match stdout_lock.write(slice) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!(
                            "ERROR: StdoutLock.write(@{:p}[{}‥{}]) error {}",
                            &*linepart.blockp, linepart.blocki_beg, linepart.blocki_end, err
                        );
                    }
                }
            } else {
                // XXX: only handle single-byte encodings
                // XXX: this is not efficient
                //let s = match str::from_utf8_lossy(slice) {
                let s = match std::str::from_utf8(slice) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("ERROR: Invalid UTF-8 sequence during from_utf8: {:?}", err);
                        continue;
                    }
                };
                let mut dst: [u8; 4] = [0, 0, 0, 0];
                for c in s.chars() {
                    let c_ = char_to_char_noraw(c);
                    let _cs = c_.encode_utf8(&mut dst);
                    match stdout_lock.write(&dst) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("ERROR: StdoutLock.write({:?}) error {}", &dst, err);
                        }
                    }
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("ERROR: stdout flushing error {}", err);
            }
        }
    }

    /// create `String` from known bytes referenced by `self.lineparts`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    /// XXX: not efficient!
    /// TODO: this would be more efficient returning `&str`
    ///       https://bes.github.io/blog/rust-strings
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub(crate) fn _to_String_raw(self: &Line, raw: bool) -> String {
        let mut sz: usize = 0;
        for linepart in &self.lineparts {
            sz += linepart.len();
        }
        let mut s1 = String::with_capacity(sz);

        for linepart in &self.lineparts {
            if raw {
                // transform slices to `str`, can this be done more efficiently?
                // XXX: here is a good place to use `bstr`
                let s2 = &(&*linepart.blockp)[linepart.blocki_beg..linepart.blocki_end];
                let s3 = match std::str::from_utf8(s2) {
                    Ok(val) => val,
                    Err(err) => {
                        let fo1 = self.fileoffset_begin() + (linepart.blocki_beg as FileOffset);
                        let fo2 = self.fileoffset_begin() + (linepart.blocki_end as FileOffset);
                        eprintln!("ERROR: failed to convert [u8] at FileOffset[{}‥{}] to utf8 str; {}", fo1, fo2, err);
                        continue;
                    }
                };
                s1.push_str(s3);
            } else {
                // copy u8 as char to `s1`
                let stop = linepart.len();
                let block_iter = (&*linepart.blockp).iter();
                for (bi, b) in block_iter.skip(linepart.blocki_beg).enumerate() {
                    if bi >= stop {
                        break;
                    }
                    let c = byte_to_char_noraw(*b);
                    s1.push(c);
                }
            }
        }

        s1
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(dead_code, non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Line) -> String {
        self._to_String_raw(true)
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Line) -> String {
        self._to_String_raw(false)
    }

    /// slice that represents the entire `Line`
    /// if `Line` does not cross a Block then this returns slice into the `Block`,
    /// otherwise it requires a copy of `Block`s data
    /// XXX: naive implementation
    /// XXX: cannot return slice because
    ///      1. size not known at compile time so cannot place on stack
    ///      2. slice is an array which is not an "owned type"
    /// TODO: add tests
    /// CANDIDATE FOR REMOVAL
    pub(crate) fn as_bytes(self: &Line) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            let bi_beg = self.lineparts[0].blocki_beg;
            let bi_end = self.lineparts[0].blocki_end;
            assert_eq!(bi_end - bi_beg, self.len(), "bi_end-bi_beg != line.len()");
            return Bytes::from(&(*(self.lineparts[0].blockp))[bi_beg..bi_end]);
        }
        // not efficient case, Line crosses stored Blocks so have to create a new vec
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_bytes: data.len() != self.len()");

        data
    }

    /// do be do
    /// CANDIDATE FOR REMOVAL
    //pub fn as_vec(self: &Line, beg: LineIndex, end: LineIndex) -> Bytes {
    #[allow(unreachable_code)]
    pub(crate) fn as_vec(self: &Line, beg: LineIndex, end: LineIndex) -> Bytes {
        assert_gt!(self.lineparts.len(), 0, "This Line has no LineParts");
        // efficient case, Line does not cross any Blocks
        if self.lineparts.len() == 1 {
            //let bi_beg = self.lineparts[0].blocki_beg;
            //let bi_end = self.lineparts[0].blocki_end;
            assert_le!(end - beg, self.len(), "end-beg > line.len()");

            return Bytes::from(&(*(self.lineparts[0].blockp))[beg as usize..end as usize]);
        }
        unreachable!("as_vec does not handle multiple lineparts yet");
        // XXX: incredibly inefficient case, Line crosses stored Blocks so have to create a new vec
        let sz = self.len();
        assert_ne!(sz, 0, "self.len() is zero!?");
        let mut data: Bytes = Bytes::with_capacity(sz);
        for lp in self.lineparts.iter() {
            let bi_beg = lp.blocki_beg;
            let bi_end = lp.blocki_end;
            data.extend_from_slice(&(*(lp.blockp))[bi_beg..bi_end]);
        }
        assert_eq!(data.len(), self.len(), "Line.as_vec: data.len() != self.len()");
        data
    }
}

/// Specialized Reader that uses BlockReader to find FoToLine
pub struct LineReader<'linereader> {
    pub(crate) blockreader: BlockReader<'linereader>,
    /// track `Line` found among blocks in `blockreader`, tracked by line beginning `FileOffset`
    /// key value `FileOffset` should agree with `(*LineP).fileoffset_begin()`
    pub lines: FoToLine,
    /// for all `Lines`, map `Line.fileoffset_end` to `Line.fileoffset_beg`
    foend_to_fobeg: FoToFo,
    /// count of `Line`s. Tracked outside of `self.lines.len()` as that may
    /// have contents removed when --streaming
    lines_count: u64,
    /// char size in bytes
    /// TODO: handle char sizes > 1 byte
    /// TODO: handle multi-byte encodings
    _charsz: CharSz,
    /// internal LRU cache for `find_line`
    _find_line_lru_cache: LinesLRUCache,
    /// internal stats
    pub(crate) _find_line_lru_cache_hit: u64,
    /// internal stats
    pub(crate) _find_line_lru_cache_miss: u64,
    // internal stats
    //pub(crate) _bytes_processed: u64,
}

impl fmt::Debug for LineReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //let f_ = match &self.file_metadata {
        //    None => format!("None"),
        //    Some(val) => format!("{:?}", val.file_type()),
        //};
        f.debug_struct("LineReader")
            //.field("@", format!("{:p}", &self))
            .field("blockreader", &self.blockreader)
            .field("_charsz", &self._charsz)
            .field("lines", &self.lines)
            .finish()
    }
}

// XXX: cannot place these within `impl LineReader`?
/// minimum char storage size in bytes
static CHARSZ_MIN: CharSz = 1;
/// maximum char storage size in bytes
static CHARSZ_MAX: CharSz = 4;
/// default char storage size in bytes
/// XXX: does not handle multi-byte encodings (e.g. UTF-8) or multi-byte character storage (e.g. UTF-32)
static CHARSZ: CharSz = CHARSZ_MIN;

/// implement the LineReader things
impl<'linereader> LineReader<'linereader> {
    const FIND_LINE_LRC_CACHE_SZ: usize = 8;

    pub fn new(path: &'linereader FPath, blocksz: BlockSz) -> Result<LineReader<'linereader>> {
        // XXX: multi-byte
        assert_ge!(
            blocksz,
            (CHARSZ_MIN as BlockSz),
            "BlockSz {} is too small, must be greater than or equal {}",
            blocksz,
            CHARSZ_MAX
        );
        assert_ne!(blocksz, 0, "BlockSz is zero");
        let mut br = BlockReader::new(path, blocksz);
        if let Err(err) = br.open() {
            return Err(err);
        };
        Ok(LineReader {
            blockreader: br,
            lines: FoToLine::new(),
            foend_to_fobeg: FoToFo::new(),
            lines_count: 0,
            _charsz: CHARSZ,
            _find_line_lru_cache: LinesLRUCache::new(LineReader::FIND_LINE_LRC_CACHE_SZ),
            _find_line_lru_cache_hit: 0,
            _find_line_lru_cache_miss: 0,
        })
    }

    /// smallest size character in bytes
    #[inline]
    pub fn charsz(&self) -> usize {
        self._charsz
    }

    /// `Block` size in bytes
    #[inline]
    pub fn blocksz(&self) -> BlockSz {
        self.blockreader.blocksz
    }

    /// File Size in bytes
    #[inline]
    pub fn filesz(&self) -> BlockSz {
        self.blockreader.filesz
    }

    /// File path
    #[inline]
    pub fn path(&self) -> &FPath {
        &self.blockreader.path
    }

    /// Testing helper only
    /// print `Line` at `fileoffset`
    /// return `false` if `fileoffset` not found
    #[cfg(any(debug_assertions, test))]
    pub fn print(&self, fileoffset: &FileOffset) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let lp = &self.lines[fileoffset];
        lp.print(true);
        true
    }

    /// Testing helper only
    /// print all known `Line`s
    #[cfg(any(debug_assertions, test))]
    pub(crate) fn print_all(&self) {
        for fo in self.lines.keys() {
            self.print(fo);
        }
    }

    /// Testing helper only
    /// copy `Line`s at `fileoffset` to String buffer
    #[cfg(test)]
    pub(crate) fn copy_line(&self, fileoffset: &FileOffset, buffer: &mut String) -> bool {
        if !self.lines.contains_key(fileoffset) {
            return false;
        }
        let line: &Line = &self.lines[fileoffset];
        let s: String = line.to_String();
        eprintln!("{}LineReader.copy_line({:2}) {:?}", snx(), fileoffset, s);
        buffer.push_str(s.as_str());
        true
    }

    /// Testing helper only
    /// copy all `Line`s to String buffer
    #[cfg(test)]
    pub(crate) fn copy_all_lines(&self, buffer: &mut String) {
        // reserve capacity in buffer
        let mut sz: usize = 0;
        for fo in self.lines.keys() {
            sz += &(self.lines[fo]).len();
        }
        sz += 1;
        buffer.clear();
        if buffer.capacity() < sz {
            eprintln!("{}LineReader.copy_all_lines() buffer.reserve({:?})", sn(), sz);
            buffer.reserve(sz);
        }
        for fo in self.lines.keys() {
            if !self.copy_line(fo, buffer){
                panic!("copy_line({}, ...) failed", fo);
            }
        }
        eprintln!("{}LineReader.copy_all_lines()", sx());
    }


    /// count of lines processed by this LineReader
    #[inline]
    pub fn count(&self) -> u64 {
        self.lines_count
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    #[inline]
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(fileoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    #[inline]
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        BlockReader::file_offset_at_block_offset(blockoffset, self.blocksz())
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    #[inline]
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        BlockReader::file_offset_at_block_offset_index(blockoffset, self.blocksz(), blockindex)
    }

    /// return block index at given `FileOffset`
    #[inline]
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        BlockReader::block_index_at_file_offset(fileoffset, self.blocksz())
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    #[inline]
    pub fn file_blocks_count(&self) -> u64 {
        BlockReader::file_blocks_count(self.filesz(), self.blocksz())
    }

    /// last valid `BlockOffset` for the file (inclusive)
    pub fn blockoffset_last(&self) -> BlockOffset {
        self.blockreader.blockoffset_last()
    }
    
    fn insert_line(&mut self, line: Line) -> LineP {
        debug_eprintln!("{}LineReader.insert_line(Line @{:p})", sn(), &line);
        let fo_beg = line.fileoffset_begin();
        let fo_end = line.fileoffset_end();
        let rl = LineP::new(line);
        debug_eprintln!("{}LineReader.insert_line: lines.insert({}, Line @{:p})", so(), fo_beg, &(*rl));
        debug_assert!(!self.lines.contains_key(&fo_beg), "self.lines already contains key {}", fo_beg);
        self.lines.insert(fo_beg, rl.clone());
        debug_eprintln!("{}LineReader.insert_line: foend_to_fobeg.insert({}, {})", so(), fo_end, fo_beg);
        debug_assert!(!self.foend_to_fobeg.contains_key(&fo_end), "self.foend_to_fobeg already contains key {}", fo_end);
        self.foend_to_fobeg.insert(fo_end, fo_beg);
        self.lines_count += 1;
        debug_eprintln!("{}LineReader.insert_line() returning @{:p}", sx(), rl);
        rl
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
    pub(crate) fn get_linep(&self, fileoffset: &FileOffset) -> Option<LineP> {
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

    /// find next `Line` starting from `fileoffset`
    /// in the process of finding, creates and stores the `Line` from underlying `Block` data
    /// returns `Found`(`FileOffset` of beginning of the _next_ line, found `LineP`)
    /// reaching end of file (and no new line) returns `Found_EOF`
    /// reaching end of file returns `FileOffset` value that is one byte past the actual end of file (and should not be used)
    /// otherwise `Err`, all other `Result::Err` errors are propagated
    /// not idempotent. 
    /// 
    /// similar to `find_sysline`, `read_block`
    ///
    /// XXX: presumes single-byte to one '\n', does not handle UTF-16 or UTF-32 or other (`charsz` hardcoded to 1)
    /// TODO: [2021/08/30] handle different encodings
    /// XXX: this function is fragile and cumbersome, any tweaks require extensive retesting
    pub fn find_line(&mut self, fileoffset: FileOffset) -> ResultS4_LineFind {
        debug_eprintln!("{}find_line(LineReader@{:p}, {})", sn(), self, fileoffset);

        // some helpful constants
        let charsz_fo = self._charsz as FileOffset;
        let charsz_bi = self._charsz as BlockIndex;
        let filesz = self.filesz();
        let blockoffset_last = self.blockoffset_last();
        let bsz: BlockSz = self.blocksz();

        // check LRU cache first (this is very fast)
        match self._find_line_lru_cache.get(&fileoffset) {
            Some(rlp) => {
                debug_eprint!("{}find_line({}): found LRU cached for offset {}", sx(), fileoffset, fileoffset);
                self._find_line_lru_cache_hit += 1;
                match rlp {
                    ResultS4_LineFind::Found(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found(({}, …)) @[{}, {}] {:?}", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end(), val.1.to_String_noraw());
                        return ResultS4_LineFind::Found((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Found_EOF(val) => {
                        debug_eprintln!(" return ResultS4_LineFind::Found_EOF(({}, …)) @[{}, {}] {:?}", val.0, val.1.fileoffset_begin(), val.1.fileoffset_end(), val.1.to_String_noraw());
                        return ResultS4_LineFind::Found_EOF((val.0, val.1.clone()));
                    }
                    ResultS4_LineFind::Done => {
                        debug_eprintln!(" return ResultS4_LineFind::Done");
                        return ResultS4_LineFind::Done;
                    }
                    _ => {
                        debug_eprintln!(" Err");
                        eprintln!("ERROR: unexpected value store in _find_line_lru_cache, fileoffset {}", fileoffset);
                    }
                }
            }
            None => {
                self._find_line_lru_cache_miss += 1;
                debug_eprintln!("{}find_line: fileoffset {} not found in LRU cache", so(), fileoffset);
            }
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

        // search containers of `Line`s
        {
            // first check if there is a `Line` already known at this fileoffset
            if self.lines.contains_key(&fileoffset) {
                debug_eprintln!("{}find_line: hit self.lines for FileOffset {}", so(), fileoffset);
                debug_assert!(self.lines_contains(&fileoffset), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fileoffset);
                let lp = self.lines[&fileoffset].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …))", so(), fileoffset, fo_next);
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, lp));
            }
            // second check if there is a `Line` at a preceding offset
            match self.get_linep(&fileoffset) {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line: self.get_linep({}) returned @{:p}",
                        so(),
                        fileoffset,
                        lp
                    );
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    debug_eprintln!("{}find_line: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*lp).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p}) @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).to_String_noraw());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    debug_eprintln!("{}find_line: fileoffset {} not found in self.lines_by_range", so(), fileoffset);
                }
            }
            debug_eprintln!("{}find_line: fileoffset {} not found in self.lines", so(), fileoffset);
            debug_eprintln!("{}find_line: searching for first newline A …", so());
        }

        //
        // could not find `fileoffset` from prior saved information so...
        // walk through blocks and bytes looking for beginning of a line (a newline character)
        // newline search part A
        //

        // reached beginning of file?
        let mut bof = false;
        // block pointer to the current block of interest
        let mut bp: BlockP;
        // found newline part A? Line begins after that newline
        let mut found_nl_a = false;
        // found newline part B? Line ends at this
        let mut found_nl_b: bool = false;
        // `fo_nl_a` should eventually "point" to beginning of `Line` (one char after found newline A)
        let mut fo_nl_a: FileOffset = fileoffset;
        // `fo_nl_b` should eventually "point" to end of `Line` including the newline char
        let mut fo_nl_b: FileOffset = fileoffset + charsz_fo;
        // if at first byte of file no need to search for first newline
        if fileoffset == 0 {
            found_nl_a = true;
            bof = true;
            debug_eprintln!("{}find_line: newline A0 is {} because at beginning of file!", so(), fo_nl_a);
        }
        // (`bin_beg`, `fo`, `bo`, `bp`, `bsz`)
        type LinePart_Mid = (BlockIndex, FileOffset, BlockOffset, BlockP, BlockSz);
        type LinePart_Mid_Opt = Option<LinePart_Mid>;
        // remember the first half of this "middle" `LinePart` (partial information for a `LinePart`)
        let mut mid_info: LinePart_Mid_Opt = LinePart_Mid_Opt::None;
        // append new `LinePart`s to this `Line`
        let mut line: Line = Line::new();

        if !found_nl_a {
            // if prior char at fileoffset-1 has newline then use that.
            // Background: caller's commonly call this function `find_line` in a sequence so it's an
            // easy check with likely success.
            // XXX: single-byte encoding, does not handle multi-byte
            let fo1 = fileoffset - charsz_fo;
            if self.foend_to_fobeg.contains_key(&fo1) {
                found_nl_a = true;
                debug_eprintln!(
                    "{}find_line A0: found newline A {} from lookup of passed fileoffset-1 {}",
                    so(),
                    fo1,
                    fileoffset - 1
                );
                // `fo_nl_a` should refer to first char past newline A
                fo_nl_a = fileoffset;
                debug_eprintln!("{}find_line A0: fo_nl_a set {}", so(), fo_nl_a);
                let bin_beg: BlockIndex = self.block_index_at_file_offset(fo_nl_a);
                let bo: BlockOffset = self.block_offset_at_file_offset(fo_nl_a);
                let bp: BlockP = match self.blockreader.read_block(bo) {
                    ResultS3_ReadBlock::Found(val) => { val },
                    _ => {
                        panic!("Uhhh..");
                    }
                };
                debug_eprintln!(
                    "{}find_line A0: set aside mid_info(BlockIndex beg {}, FileOffset beg {}, BlockOffset {}, BlockSz {})",
                    so(),
                    bin_beg,
                    fo_nl_a,
                    bo,
                    bsz,
                );
                mid_info = Some((bin_beg, fo_nl_a, bo, bp, bsz));
            } else {
                debug_eprintln!(
                    "{}find_line A0: did not find newline A in lookup of passed fileoffset-1 {}",
                    so(),
                    fileoffset - 1
                );
            }
        }

        //
        // walk backwards looking for line-beginning newline '\n' newline A
        // that is, the newline that terminates a preceding line (not included in this found Line)
        //
        if !found_nl_a {
            let mut bo = self.block_offset_at_file_offset(fileoffset);
            let bo_1 = bo;
            let mut bin_beg: BlockIndex = self.block_index_at_file_offset(fileoffset);
            let mut bin_end: BlockIndex;
            let mut bo_prior: BlockOffset = 0;
            let mut first_check = true;

            // walk backwards though partial "middle" block (wherever `fileoffset` refers) (loop once)
            #[allow(clippy::never_loop)]
            '_loop_nl_a1: loop {  // this "loop" only occurs once
                debug_eprintln!("{}find_line A1: self.blockreader.read_block({}) (one time while searching for newline A)", so(), bo);
                match self.blockreader.read_block(bo) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line A1: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                            so(),
                            bo,
                            &(*val),
                            (*val).len()
                        );
                        bp = val;
                        bin_end = bp.len() as BlockIndex;
                        // XXX: multi-byte
                        //bin_beg = bin_end - charsz_bi;
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!("{}find_line A1: read_block({}) returned Done {:?}", so(), bo, self.path());
                        return ResultS4_LineFind::Done;
                    },
                    ResultS3_ReadBlock::Err(err) => {
                        //debug_eprintln!("{}find_line A1: LRU cache put({}, Done)", so(), fileoffset);
                        //self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
                        //debug_eprintln!("{}find_line A1: return ResultS4_LineFind::Done", sx());
                        debug_eprintln!("{}find_line({}) A1: read_block({}) returned Err, return ResultS4_LineFind::Err({:?})", sx(), fileoffset, bo, err);
                        return ResultS4_LineFind::Err(err);
                    }
                }
                // special case where very first byte check is newline, use that as newline B
                if first_check && (*bp)[bin_beg] == NLu8 {
                    first_check = false;
                    found_nl_b = true;
                    fo_nl_b = fileoffset;
                    debug_eprintln!("{}find_line A1b: very first byte was newline, this is newline B at fileoffset {}", so(), fo_nl_b);
                    // move search back one char
                    let fo_ = fileoffset - charsz_fo;
                    bo = self.block_offset_at_file_offset(fo_);
                    bin_beg = self.block_index_at_file_offset(fo_);
                    if fo_ == 0 {
                        fo_nl_a = fo_;
                        found_nl_a = true;
                        bof = true;
                        debug_eprintln!("{}find_line A1b: newline A1 restarting search at beginning of file!", so());
                    }
                    debug_eprintln!("{}find_line A1b: restart search for newline A at blockoffset {} blockindex {} (fileoffset {})", so(), bo, bin_beg, fo_);
                    continue;
                }
                debug_eprintln!("{}find_line A1: scan block {} backwards, starting from blockindex {} (fileoffset {}) searching for newline A", so(), bo, bin_beg, self.file_offset_at_block_offset_index(bo, bin_beg));
                loop {
                    // XXX: single-byte encoding
                    if (*bp)[bin_beg] == NLu8 {
                        found_nl_a = true;
                        debug_eprintln!(
                            "{}find_line A1: found newline A during byte search, blockoffset {} blockindex {} (fileoffset {}) {:?}",
                            so(),
                            bo,
                            bin_beg,
                            self.file_offset_at_block_offset_index(bo, bin_beg),
                            byte_to_char_noraw((*bp)[bin_beg]),
                        );
                        if bin_beg >= bin_end - 1 {
                            // if at last blockindex of current block then move to preceding block at blockindex 0
                            debug_eprintln!(
                                "{}find_line A1: at last blockindex {} of block {} (fileoffset {})",
                                so(),
                                bin_beg,
                                bo,
                                self.file_offset_at_block_offset_index(bo, bin_beg),
                            );
                            // special case at end of file
                            if bo == blockoffset_last {
                                let fo_beg = self.file_offset_at_block_offset_index(bo, bin_beg);
                                debug_eprintln!("{}find_line A1: newline A is at end of file (fileoffset {}); create LinePart and Line then return", so(), fo_beg);
                                let li = LinePart::new(bin_beg, bin_end, bp.clone(), fo_beg, bo, bsz);
                                debug_eprintln!("{}find_line A1: Line.prepend({:?}) {:?}", so(), &li, li.to_String_noraw());
                                line.prepend(li);
                                let linep = self.insert_line(line);
                                let fo_next = fileoffset + charsz_fo;
                                debug_assert_eq!(fo_next, (*linep).fileoffset_end() + charsz_fo, "mismatching fo_next (*linep).fileoffset_end()+1");
                                debug_eprintln!("{}find_line A1: LRU cache put({}, Found_EOF(({}, @{:p})))", so(), fileoffset, fo_next, linep);
                                self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Found_EOF((fo_next, linep.clone())));
                                debug_eprintln!("{}find_line({}) A1: return ResultS4_LineFind::Found_EOF(({}, @{:p})) @[{}, {}] {:?}", sx(), fileoffset, fo_next, linep, (*linep).fileoffset_begin(), (*linep).fileoffset_end(), (*linep).to_String_noraw());
                                return ResultS4_LineFind::Found_EOF((fo_next, linep.clone()));
                            }
                            bo += 1;
                            bin_beg = 0;
                            fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                            debug_eprintln!("{}find_line A1a: fo_nl_a set {}", so(), fo_nl_a);
                            mid_info = None;
                        } else {
                            // `fo_nl_a` should refer to first char past newline A
                            // XXX: single-byte encoding
                            bin_beg += charsz_bi;
                            fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                            debug_eprintln!("{}find_line A1b: fo_nl_a set {}", so(), fo_nl_a);
                            debug_eprintln!(
                                "{}find_line A1a: set aside mid_info(BlockIndex beg {}, FileOffset beg {}, BlockOffset {}, BlockSz {})",
                                so(),
                                bin_beg,
                                fo_nl_a,
                                bo,
                                bsz,
                            );
                            // (BlockIndex, FileOffset, BlockOffset, BlockSz)
                            mid_info = Some((bin_beg, fo_nl_a, bo, bp.clone(), bsz));
                        }
                        break;
                    } else {
                        if bin_beg == 0 {
                            fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                            debug_eprintln!("{}find_line A1c: fo_nl_a set {}", so(), fo_nl_a);
                            debug_eprintln!(
                                "{}find_line A1b: set aside mid_info(BlockIndex beg {}, FileOffset beg {}, BlockOffset {})",
                                so(),
                                bin_beg,
                                fo_nl_a,
                                bo,
                            );
                            // (BlockIndex, FileOffset, BlockOffset, BlockSz)
                            mid_info = Some((bin_beg, fo_nl_a, bo, bp.clone(), bsz));
                            break;
                        }
                        bin_beg -= charsz_bi;
                    }
                }  // end loop
                if found_nl_a {
                    break;
                }
                if bo != 0 {
                    bo -= 1;
                    bin_beg = bin_end - charsz_bi;
                } else {
                    debug_eprintln!("{}find_line A1: walked all the way back to the beginning of file", so());
                    // getting here means walked all the back to the beginning of the file
                    // XXX: does not handle Byte Order Mark
                    found_nl_a = true;
                    fo_nl_a = 0;
                    debug_eprintln!("{}find_line A1d: fo_nl_a set {}", so(), fo_nl_a);
                    bof = true;
                }
                debug_eprintln!("{}find_line A1: done with A1 once loop", so());
                break;  // only loop once
            }

            // search backwards to beginning of file (loop zero or more times)
            let mut save_linepart = true;
           '_loop_nl_aN: while !found_nl_a {
                debug_eprintln!("{}find_line A2: self.blockreader.read_block({}) (one or more times while searching for newline A)", so(), bo);
                match self.blockreader.read_block(bo) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line A2: read_block({}) returned Found Block @{:p} len {} while searching for newline A",
                            so(),
                            bo,
                            &(*val),
                            (*val).len()
                        );
                        bp = val;
                        bin_end = bp.len() as BlockIndex;
                        if bo != bo_1 {  // if not first loop iteration
                            bin_beg = bin_end - charsz_bi;
                        }
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!("{}find_line A2: read_block({}) returned Done {:?} searching for found_nl_a failed", so(), bo, self.path());
                        debug_eprintln!("{}find_line({}) A2: return ResultS4_LineFind::Done", sx(), fileoffset);
                        return ResultS4_LineFind::Done;
                    }
                    ResultS3_ReadBlock::Err(err) => {
                        //debug_eprintln!("{}find_line A2: LRU cache put({}, Done)", so(), fileoffset);
                        //self._find_line_lru_cache.put(fileoffset, ResultS4_LineFind::Done);
                        debug_eprintln!("{}find_line({}) A2: return ResultS4_LineFind::Err({:?})", sx(), fileoffset, err);
                        return ResultS4_LineFind::Err(err);
                    }
                }
                // walk backwards though a single `Block`
                debug_eprintln!("{}find_line A2: scan block {} backwards, starting from blockindex {} (fileoffset {})", so(), bo, bin_beg, self.file_offset_at_block_offset_index(bo, bin_beg));
                loop {
                    // XXX: single-byte encoding
                    if (*bp)[bin_beg] == NLu8 {
                        found_nl_a = true;
                        let _fo_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                        debug_eprintln!(
                            "{}find_line A2: found newline A at fileoffset {} during byte search, blockoffset {} blockindex {} {:?}",
                            so(),
                            _fo_a,
                            bo,
                            bin_beg,
                            byte_to_char_noraw((*bp)[bin_beg]),
                        );
                        if bin_beg == bin_end - 1 {
                            // if at last blockindex of current block then move to next block at blockindex 0
                            bo += 1;
                            bin_beg = 0;
                            fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                            debug_eprintln!("{}find_line A2a: fo_nl_a set {}", so(), fo_nl_a);
                        } else {
                            // `fo_nl_a` should refer to first char past newline A
                            // XXX: single-byte encoding
                            bin_beg += charsz_bi;
                            fo_nl_a = self.file_offset_at_block_offset_index(bo, bin_beg);
                            debug_eprintln!("{}find_line A2b: fo_nl_a set {}", so(), fo_nl_a);
                        }
                        debug_eprintln!(
                            "{}find_line A2: fo_nl_a set to fileoffset {}, blockoffset {} blockindex {}",
                            so(),
                            fo_nl_a,
                            bo,
                            bin_beg,
                        );
                        break;
                    }
                    // XXX: single-byte encoding
                    fo_nl_a -= charsz_fo;
                    debug_eprintln!("{}find_line A2c: fo_nl_a set {}", so(), fo_nl_a);
                    if bin_beg == 0 {
                        break;
                    }
                    // XXX: single-byte encoding
                    bin_beg -= charsz_bi;
                }
                if mid_info.as_ref().is_some() && mid_info.as_ref().unwrap().2 == bo {
                    // if still at "middle" linepart (fileoffset reference) then wait to search for newline B to create a new `LinePart`
                    debug_eprintln!("{}find_line A2: skip new `LinePart`", so());
                } else if bo != bo_prior {
                    // remember this (possibly entire) `LinePart` (done zero or more times)
                    let fo_beg: FileOffset = self.file_offset_at_block_offset_index(bo, bin_beg);
                    let li = LinePart::new(bin_beg, bin_end, bp.clone(), fo_beg, bo, bsz);
                    debug_eprintln!("{}find_line A2: Line.prepend({:?}) {:?}", so(), &li, li.to_String_noraw());
                    line.prepend(li);
                    bo_prior = bo;
                } else {
                    debug_eprintln!("{}find_line A2: skip new `LinePart`, this block {} was previously saved in a LinePart", so(), bo);
                }
                if found_nl_a {
                    break;
                }
                if bo != 0 {
                    bo -= 1;
                    bin_beg = bin_end - charsz_bi;
                } else {
                    debug_eprintln!("{}find_line A2: walked all the way back to the beginning of file", so());
                    // getting here means walked all the back to the beginning of the file
                    // XXX: does not handle Byte Order Mark
                    found_nl_a = true;
                    fo_nl_a = 0;
                    debug_eprintln!("{}find_line A2d: fo_nl_a set {}", so(), fo_nl_a);
                    bof = true;
                    break;
                }
            }  // '_loop_nl_aN: while ! found_nl_a
        } else {  
            debug_eprintln!("{}find_line: skip backwards search for newline A2", so());
        }  // if ! found_nl_a

        //
        // walk through blocks and bytes looking for ending of line (a newline character; part B)
        //
        debug_eprintln!(
            "{}find_line: found first newline A at FileOffset {}, searching for second B newline starting at FileOffset {} …",
            so(),
            fo_nl_a,
            fileoffset + charsz_fo,
        );

        if !found_nl_b {
            // …but before doing work of discovering a new `Line` (part B), first checks various
            // maps in `self` to see if this `Line` has already been discovered and processed
            if self.lines.contains_key(&fo_nl_a) {
                debug_eprintln!("{}find_line AB: hit in self.lines for FileOffset {} (before part B)", so(), fo_nl_a);
                debug_assert!(self.lines_contains(&fo_nl_a), "self.lines and self.lines_by_range are out of synch on key {} (before part A)", fo_nl_a);
                let lp = self.lines[&fo_nl_a].clone();
                let fo_next = (*lp).fileoffset_end() + charsz_fo;
                debug_eprintln!("{}find_line AB: LRU Cache put({}, Found({}, …)) {:?}", so(), fileoffset, fo_next, (*lp).to_String_noraw());
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found((fo_next, lp.clone())));
                debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p})  @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).to_String_noraw());
                return ResultS4_LineFind::Found((fo_next, lp));
            } else {
                debug_eprintln!("{}find_line AB: miss in self.lines for FileOffset {} (before part B)", so(), fo_nl_a);
            }
            match self.get_linep(&fo_nl_a) {
                Some(lp) => {
                    debug_eprintln!(
                        "{}find_line AB: self.get_linep({}) returned {:p}",
                        so(),
                        fo_nl_a,
                        lp
                    );
                    let fo_next = (*lp).fileoffset_end() + charsz_fo;
                    debug_eprintln!("{}find_line AB: LRU Cache put({}, Found({}, …)) {:?}", so(), fo_nl_a, fo_next, (*lp).to_String_noraw());
                    self._find_line_lru_cache
                        .put(fo_nl_a, ResultS4_LineFind::Found((fo_next, lp.clone())));
                    debug_eprintln!("{}find_line({}): return ResultS4_LineFind::Found({}, {:p}) @[{}, {}] {:?}", sx(), fileoffset, fo_next, &*lp, (*lp).fileoffset_begin(), (*lp).fileoffset_end(), (*lp).to_String_noraw());
                    return ResultS4_LineFind::Found((fo_next, lp));
                }
                None => {
                    debug_eprintln!("{}find_line AB: self.get_linep({}) returned None (before part B)", so(), fo_nl_a);
                }
            }
        }

        //
        // getting here means this function is discovering a brand new `Line` (part B)
        // walk *forwards* to find line-terminating newline (part B)
        //

        /// wrapper to handle possible combining of `LinePart_Mid` into a new `LinePart` then do `Line.push`
        /// TODO: this should use Either wrapper
        #[allow(clippy::too_many_arguments)]
        fn create_linepart_B1(
            mid_info_: &mut LinePart_Mid_Opt,
            bin_beg_: &BlockIndex,
            bin_end_: &BlockIndex,
            fo_: &FileOffset,
            bo_: &BlockOffset,
            bp_: &BlockP,
            bsz_: &BlockSz,
            line_: &mut Line,
        ) {
            if mid_info_.is_some() {
                // remember this middle `LinePart`, combining with saved `mid_info` (done once)
                #[allow(clippy::unnecessary_unwrap)]
                let mi: &LinePart_Mid = mid_info_.as_ref().unwrap();
                debug_eprintln!("{}find_line Bx: create_linepart_B1: combine mid_info set aside: {:?}", so(), mi);
                // sanity checks
                //let bo_at_fo = BlockReader::block_offset_at_file_offset(*mi.1, *bsz_);
                //assert_eq!(mi.2, bo_at_fo, "unexpected mid_info.blockoffset {} ≠ {} block_offset_at_file_offset({})", mi.2, bo_at_fo, fo_);
                // save the "middle" `LinePart`
                let li = LinePart::new(mi.0, *bin_end_, mi.3.clone(), mi.1, mi.2, mi.4);
                debug_eprintln!("{}find_line Bx: create_linepart_B1: Line.push({:?}) {:?}", so(), &li, li.to_String_noraw());
                line_.push(li);
                *mid_info_ = LinePart_Mid_Opt::None;
            } else {
                // remember this (possibly entire) `LinePart` (done zero or more times)
                let li = LinePart::new(*bin_beg_, *bin_end_, bp_.clone(), *fo_, *bo_, *bsz_);
                debug_eprintln!("{}find_line Bx: create_linepart_B1: Line.push({:?}) {:?}", so(), &li, li.to_String_noraw());
                line_.push(li);
            }
        }

        if found_nl_b {
            debug_eprintln!("{}find_line B_: newline B is already found at fileoffset {}", so(), fo_nl_b);
            let bin_beg = self.block_index_at_file_offset(fo_nl_a);
            let mut bin_end = self.block_index_at_file_offset(fo_nl_b + charsz_fo);
            let bo = self.block_offset_at_file_offset(fo_nl_a);
            if bin_end == bin_beg && (fo_nl_b + charsz_fo) == filesz {
                bin_end += charsz_bi;
                debug_eprintln!("{}find_line B_: newline B fileoffset {} is at end of file (filesz {}), force increase of bi_end {}", so(), fo_nl_b, filesz, bin_end);
            }
            debug_eprintln!("{}find_line B_: self.blockreader.read_block({})", so(), bo);
            let bp = match self.blockreader.read_block(bo) {
                ResultS3_ReadBlock::Found(val) => {
                    debug_eprintln!(
                        "{}find_line B_: read_block({}) returned Found Block @{:p} len {} while creating Line for newline B",
                        so(),
                        bo,
                        &(*val),
                        (*val).len()
                    );
                    val
                },
                ResultS3_ReadBlock::Done => {
                    debug_eprintln!(
                        "{}find_line B_: read_block({}) returned Done {:?} while creating Line for newline B",
                        so(),
                        bo,
                        self.path(),
                    );
                    debug_eprintln!("{}find_line({}): B_ return ResultS3_ReadBlock::Done;", sx(), fileoffset);
                    return ResultS4_LineFind::Done;
                },
                ResultS3_ReadBlock::Err(err) => {
                    debug_eprintln!("{}find_line({}): B_ return ResultS3_ReadBlock::Err({:?});", sx(), fileoffset, err);
                    return ResultS4_LineFind::Err(err);
                },
            };
            let bsz = self.blocksz();
            create_linepart_B1(&mut mid_info, &bin_beg, &bin_end, &fo_nl_a, &bo, &bp, &bsz, &mut line);
        }

        // `fo_nl_b` is effectively the cursor that is being analyzed
        //let mut fo_nl_b: FileOffset = fileoffset + charsz_fo;
        if !found_nl_b {
            // found newline part B? Line ends at this
            //let mut found_nl_b: bool = false;
            let bo_1 = self.block_offset_at_file_offset(fileoffset);
            let mut bo = self.block_offset_at_file_offset(fo_nl_b);

            //if found_nl_b {
            //    let bin_end = self.block_index_at_file_offset(fo_nl_b);
            //    let bsz = self.blocksz();
            //    create_linepart_B1(&mut mid_info, &bin_beg, &bin_end, &fo_nl_b, &bo, &bp, &bsz, &mut line);
            //}

            // XXX: does this check make sense?
            if bo > blockoffset_last {
                found_nl_b = true;
                debug_eprintln!(
                    "{}find_line B0:  BlockOffset {:?} (fileoffset {}) past end of file, while searching for newline B",
                    so(),
                    bo,
                    fo_nl_b,
                );
                //create_linepart_B1(&mut mid_info, &0, &0, &fo_nl_b, &bo, &bp, bsz, &mut line);
                assert!(mid_info.is_none(), "mid_info is still Some {:?}, it should have been used (and set to None)", mid_info);
                let lp_ = self.insert_line(line);
                let fo_ = (*lp_).fileoffset_end() + charsz_fo;
                debug_eprintln!("{}find_line B0: LRU Cache put({}, Found_EOF({}, …)) {:?}", so(), fileoffset, fo_, lp_.to_String_noraw());
                self._find_line_lru_cache
                    .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_, lp_.clone())));
                debug_eprintln!(
                    "{}find_line({}) B0: return ResultS4_LineFind::Found_EOF(({}, {:p})) @[{} , {}]; {:?}",
                    sx(),
                    fileoffset,
                    fo_,
                    &*lp_,
                    (*lp_).fileoffset_begin(),
                    (*lp_).fileoffset_end(),
                    (*lp_).to_String_noraw()
                );
                return ResultS4_LineFind::Found_EOF((fo_, lp_));
            }

            if mid_info.is_some() && mid_info.as_mut().unwrap().2 < bo {
                let bo_: BlockOffset = mid_info.as_mut().unwrap().2;
                let bp_: BlockP = mid_info.as_mut().unwrap().3.clone();
                let bsz_: BlockSz = self.blocksz();
                let bin_end_ = (*bp_).len() as BlockIndex;
                debug_eprintln!("{}find_line B0: transitioned from block {} to block {}, so create linepart with midinfo for block {} data", so(), bo_, bo, bo_);
                create_linepart_B1(&mut mid_info, &0, &bin_end_, &0, &0, &bp_, &bsz_, &mut line);
            }

            // handle middle partial block (where `fileoffset` refers) (loop once)
            #[allow(clippy::never_loop)]
            '_loop_nl_b1: while !found_nl_b && bo <= blockoffset_last {
                debug_eprintln!("{}find_line B1: self.blockreader.read_block({})", so(), bo);
                match self.blockreader.read_block(bo) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line B1: read_block({}) returned Found Block @{:p} len {} while searching for newline B",
                            so(),
                            bo,
                            &(*val),
                            (*val).len()
                        );
                        bp = val;
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!(
                            "{}find_line B1: read_block({}) returned Done {:?} while searching for newline B",
                            so(),
                            bo,
                            self.path(),
                        );
                        assert!(mid_info.is_none(), "mid_info is still Some {:?}, it should have been used (and set to None)", mid_info);
                        let lp = self.insert_line(line);
                        let fo_ = (*lp).fileoffset_end() + charsz_fo;
                        debug_eprintln!("{}find_line B1: LRU Cache put({}, Found_EOF({}, …)) {:?}", so(), fileoffset, fo_, lp.to_String_noraw());
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_, lp.clone())));
                        debug_eprintln!(
                            "{}find_line({}) B1: return ResultS4_LineFind::Found_EOF(({}, {:p})) @[{} , {}]; {:?}",
                            sx(),
                            fileoffset,
                            fo_,
                            &*lp,
                            (*lp).fileoffset_begin(),
                            (*lp).fileoffset_end(),
                            (*lp).to_String_noraw()
                        );
                        return ResultS4_LineFind::Found_EOF((fo_, lp));
                    },
                    ResultS3_ReadBlock::Err(err) => {
                        debug_eprintln!("{}find_line({}): B1 return ResultS4_LineFind::Err({:?});", sx(), fileoffset, err);
                        return ResultS4_LineFind::Err(err);
                    }
                }
                let blen = (*bp).len() as BlockIndex;
                let bin_beg: BlockIndex;
                // LAST WORKING HERE 2022/04/22 00:09:55
                // Almost got this working... ./tools/rust-test.sh 'test_LineReader_precise_order_2__0_44__0xF'
                // Another 'start from scratch' attempt at this function would need to determine early-on
                // if passed `fileoffset` is at block boundary (and char boundary). then use that to
                // walk through the flattened-out state machine.
                // is next line correct thing to do????????
                if mid_info.is_some() {
                    bin_beg = self.block_index_at_file_offset(fo_nl_a);
                } else {
                    bin_beg = 0;
                }
                let fo_beg = fo_nl_a;
                let mut bin_end = bin_beg + charsz_bi;
                while bin_end < blen {
                    // XXX: single-byte encoding
                    if (*bp)[bin_end] == NLu8 {
                        found_nl_b = true;
                        fo_nl_b = self.file_offset_at_block_offset_index(bo, bin_end);
                        debug_eprintln!(
                            "{}find_line B1: newline B found by byte search at fileoffset {} ≟ blockoffset {} blockindex {}",
                            so(),
                            fo_nl_b,
                            bo,
                            bin_end,
                        );
                        bin_end += charsz_bi;
                        break;
                    }
                    // XXX: single-byte encoding
                    bin_end += charsz_bi;
                }
                // sanity check
                //if fo_nl_b == filesz {
                 //   assert_eq!(bin_end - bin_beg, 0, "newline B fileoffset {} is at end of file offset {}, yet found a linepart of length {} (expected zero)", fo_nl_b, filesz, bin_end - bin_beg);
                //}
                // sanity check
                //if bin_end - bin_beg == 0 {
                //    assert_eq!(fo_nl_b, filesz, "found a linepart of length {} (expected zero) yet fileoffset is {}", bin_end - bin_beg, fo_beg);
                //}
                // at end of file and "zero length" LinePart then skip creating a `LinePart`
                if bin_end == bin_beg && fo_nl_b == filesz {
                    debug_eprintln!("{}find_line B1: no newline B, at end of file, do not create zero length LinePart", so());
                    break;
                }
                // at last char in file
                if bin_end == bin_beg && (fo_nl_b + charsz_fo) == filesz {
                    found_nl_b = true;
                    bin_end += charsz_bi;
                    debug_eprintln!("{}find_line B1: newline B fileoffset {} is at end of file (filesz {}), force increase of bi_end {}", so(), fo_nl_b, filesz, bin_end);
                }
                create_linepart_B1(&mut mid_info, &bin_beg, &bin_end, &fo_beg, &bo, &bp, &bsz, &mut line);
                if found_nl_b {
                    break;
                }
                bo += 1;
                break;
            }  // '_loop_nl_b1: loop

            // walk forwards through remainder of file (loop zero or more times)
            '_loop_nl_bn: while !found_nl_b && bo <= blockoffset_last {
                debug_eprintln!("{}find_line B2: self.blockreader.read_block({})", so(), bo);
                match self.blockreader.read_block(bo) {
                    ResultS3_ReadBlock::Found(val) => {
                        debug_eprintln!(
                            "{}find_line B2: read_block({}) returned Found Block @{:p} len {} while searching for newline B",
                            so(),
                            bo,
                            &(*val),
                            (*val).len()
                        );
                        bp = val;
                    },
                    ResultS3_ReadBlock::Done => {
                        debug_eprintln!(
                            "{}find_line B2: read_block({}) returned Done {:?} while searching for newline B",
                            so(),
                            bo,
                            self.path()
                        );
                        assert!(mid_info.is_none(), "mid_info is still Some {:?}, it should have been used (and set to None)", mid_info);
                        let lp = self.insert_line(line);
                        let fo_ = (*lp).fileoffset_end() + charsz_fo;
                        debug_eprintln!("{}find_line B2: LRU Cache put({}, Found_EOF({}, …))", so(), fileoffset, fo_);
                        self._find_line_lru_cache
                            .put(fileoffset, ResultS4_LineFind::Found_EOF((fo_, lp.clone())));
                        debug_eprintln!(
                            "{}find_line({}) B2: return ResultS4_LineFind::Found_EOF(({}, {:p})) @[{} , {}]; {:?}",
                            sx(),
                            fileoffset,
                            fo_,
                            &*lp,
                            (*lp).fileoffset_begin(),
                            (*lp).fileoffset_end(),
                            (*lp).to_String_noraw(),
                        );
                        return ResultS4_LineFind::Found_EOF((fo_, lp));
                    }
                    ResultS3_ReadBlock::Err(err) => {
                        debug_eprintln!("{}find_line({}) B2: return ResultS4_LineFind::Err({:?});", sx(), fileoffset, err);
                        return ResultS4_LineFind::Err(err);
                    }
                }
                let blen = (*bp).len() as BlockIndex;
                let bin_beg = 0;
                let mut bin_end = bin_beg;
                while bin_end < blen {
                    // XXX: single-byte encoding
                    if (*bp)[bin_end] == NLu8 {
                        found_nl_b = true;
                        fo_nl_b = self.file_offset_at_block_offset_index(bo, bin_end);
                        bin_end += charsz_bi; // refer to one past end
                        debug_eprintln!(
                            "{}find_line B2: newline B found by byte search at fileoffset {} ≟ blockoffset {} blockindex {}",
                            so(),
                            fo_nl_b,
                            bo,
                            bin_end,
                        );
                        break;
                    }
                    // XXX: single-byte encoding
                    bin_end += charsz_bi;
                }
                let fo_beg = self.file_offset_at_block_offset_index(bo, bin_beg);
                // sanity check
                if fo_beg == filesz {
                    assert_eq!(bin_end - bin_beg, 0, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
                }
                // sanity check
                if bin_end - bin_beg == 0 {
                    assert_eq!(fo_beg, filesz, "fileoffset of beginning of line {} is at end of file, yet found a linepart of length {} (expected zero)", fo_beg, bin_end - bin_beg);
                }
                // at end of file, "zero length" LinePart, skip creating a `LinePart`
                if bin_end - bin_beg == 0 && fo_beg == filesz {
                    debug_eprintln!("{}find_line B2: no newline B, at end of file", so());
                    break;
                }
                // remember this (possibly entire) `LinePart` (done zero or more times)
                let li = LinePart::new(bin_beg, bin_end, bp.clone(), fo_beg, bo, bsz);
                debug_eprintln!("{}find_line B2: Line.push({:?}) {:?}", so(), &li, li.to_String_noraw());
                line.push(li);
                assert!(mid_info.is_none(), "mid_info is still Some {:?}, it should have been used (and set to None)", mid_info);
                if found_nl_b {
                    break;
                }
                bo += 1;
            } // while ! found_nl_b
        }

        // may occur in files ending on a single newline
        if line.count() == 0 {
            debug_eprintln!("{}find_line C: LRU Cache put({}, Done)", so(), fileoffset);
            self._find_line_lru_cache
                .put(fileoffset, ResultS4_LineFind::Done);
            debug_eprintln!("{}find_line({}) C: return ResultS4_LineFind::Done;", sx(), fileoffset);
            return ResultS4_LineFind::Done;
        }

        // sanity check
        assert!(mid_info.is_none(), "mid_info should be None, it is {:?}", mid_info);
        debug_eprintln!("{}find_line D: return {:?};", so(), line);
        let fo_end = line.fileoffset_end();
        let lp = self.insert_line(line);
        debug_eprintln!("{}find_line D: LRU Cache put({}, Found({}, …))", so(), fileoffset, fo_end + 1);
        self._find_line_lru_cache
            .put(fileoffset, ResultS4_LineFind::Found((fo_end + 1, lp.clone())));
        debug_eprintln!(
            "{}find_line({}) D: return ResultS4_LineFind::Found(({}, @{:p})) @[{}, {}] {:?}",
            sx(),
            fileoffset,
            fo_end + 1,
            &*lp,
            (*lp).fileoffset_begin(),
            (*lp).fileoffset_end(),
            (*lp).to_String_noraw()
        );

        ResultS4_LineFind::Found((fo_end + 1, lp))
    }
}
