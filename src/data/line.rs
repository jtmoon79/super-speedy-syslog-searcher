// src/Data/line.rs
//

#![allow(non_camel_case_types)]

pub use crate::common::{
    Bytes,
    Count,
    FPath,
    FileOffset,
    CharSz,
    NLu8,
    ResultS3,
};

use crate::readers::blockreader::{
    BlockSz,
    BlockOffset,
    BlockIndex,
    BlockP,
    Slices,
    BlockReader,
};

#[cfg(any(debug_assertions,test))]
use crate::printer_debug::printers::{
    buffer_to_String_noraw,
    char_to_char_noraw,
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
    p_err,
    p_wrn,
};

#[cfg(any(debug_assertions,test))]
use std::borrow::Cow;

use std::fmt;

#[cfg(any(debug_assertions,test))]
use std::io::prelude::*;

use std::sync::Arc;

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_gt,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A sequence to track the `[u8]` that make up a `Line`.
/// A "line" may span multiple `Block`s. One `LinePart` refers to one `Block`.
pub type LineParts = Vec<LinePart>;
/// A sequence to track one or more `LineP` that make up a `Sysline`
pub type Lines = Vec<LineP>;
/// An offset into a `Line` (not related to underlying `Block` offset or indexes)
pub type LineIndex = usize;
/// half-open `Range` of `LineIndex`
pub type Range_LineIndex = std::ops::Range<LineIndex>;
/// thread-safe Atomic Reference Counting pointer to a `Line`
pub type LineP = Arc<Line>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinePart, Line, and LineReader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A `LinePart` is some or all of a line within a `Block`.
/// The purpose of a `LinePart` is to help create a slice into a `Block`.
///
/// A "line" can span more than one `Block`. A `LinePart` tracks the line data
/// residing in one `Block`. One `LinePart` to one `Block`.
///
/// One or more `LinePart`s are required for a `Line`.
pub struct LinePart {
    /// the `Block` pointer
    pub blockp: BlockP,
    /// index into the `blockp`, index at beginning
    /// used as-is in slice notation (inclusive)
    pub blocki_beg: BlockIndex,
    /// index into the `blockp`, index at one after ending '\n' (may refer to one past end of `Block`)
    /// used as-is in slice notation (exclusive)
    pub blocki_end: BlockIndex,
    /// the byte offset into the file where this `LinePart` begins
    fileoffset: FileOffset,
    /// `blockoffset` of underlying `Block` to which `self.blockp` points.
    ///
    /// XXX: debug helper
    // TODO: [2022] is this used?
    blockoffset: BlockOffset,
    /// the file-designated BlockSz, _not_ necessarily the `len()` of the `Block` at `blockp`
    // TODO: [2022] is this used?
    pub blocksz: BlockSz,
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
    const _CHARSZ: usize = 1;

    /// create a new `LinePart`. Remember that `blocki_end` points to one byte past
    /// because it used directly in byte array slice notation (exclusive).
    /// i.e. `(*blockp)[blocki_beg..blocki_end]`
    pub fn new(
        blockp: BlockP,
        blocki_beg: BlockIndex,
        blocki_end: BlockIndex,
        fileoffset: FileOffset,
        blockoffset: BlockOffset,
        blocksz: BlockSz,
    ) -> LinePart {
        dpnf!(
            "LinePart(blocki_beg {}, blocki_end {}, Block @{:p}, fileoffset {}, blockoffset {}, blocksz {}) (blockp.len() {})",
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
            blockp,
            blocki_beg,
            blocki_end,
            fileoffset,
            blockoffset,
            blocksz,
        }
    }

    /// length of `LinePart` starting at index `blocki_beg` in bytes
    pub fn len(&self) -> usize {
        (self.blocki_end - self.blocki_beg) as usize
    }

    /// clippy recommends `fn is_empty` since there is already `len()`
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// fileoffset at beginning of the `LinePart` (inclusive)
    pub fn fileoffset_begin(&self) -> FileOffset {
        self.fileoffset
    }

    /// fileoffset at one byte past ending of the `LinePart` (exclusive)
    pub fn fileoffset_end(&self) -> FileOffset {
        self.fileoffset + (self.blocki_end as FileOffset)
    }

    /// `blockoffset` of underlying `Block` to which `self.blockp` points
    pub fn blockoffset(&self) -> BlockOffset {
        self.blockoffset
    }

    /// count of bytes of this `LinePart`
    /// XXX: `count_bytes` and `len` is overlapping and confusing.
    ///
    /// TODO: this should be removed
    pub fn count_bytes(&self) -> Count {
        self.len() as Count
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub(self) fn impl_to_String_raw(self: &LinePart, raw: bool) -> String {
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
        self.impl_to_String_raw(false)
    }

    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &LinePart) -> String {
        self.impl_to_String_raw(true)
    }

    /// return Box pointer to slice of bytes that make up this `LinePart`
    pub fn block_boxptr(&self) -> Box<&[u8]> {
        let slice_ = &(*self.blockp).as_slice()[self.blocki_beg..self.blocki_end];

        Box::new(slice_)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from `a` (inclusive) to end
    pub fn block_boxptr_a(&self, a: &LineIndex) -> Box<&[u8]> {
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        debug_assert_le!(self.blocki_end, (*self.blockp).as_slice().len(), "self.blocki_end {} past end of slice.len() {}", self.blocki_end, (*self.blockp).as_slice().len());
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..self.blocki_end];

        Box::new(slice1)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from beginning to `b` (exclusive)
    pub fn block_boxptr_b(&self, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_le!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        let slice1 = &(*self.blockp).as_slice()[self.blocki_beg..(self.blocki_beg+b)];

        Box::new(slice1)
    }

    /// return Box pointer to slice of bytes in this `LinePart` from `a` (inclusive) to `b` (exclusive)
    pub fn block_boxptr_ab(&self, a: &LineIndex, b: &LineIndex) -> Box<&[u8]> {
        debug_assert_le!(a, b, "bad LineIndex");
        debug_assert_lt!(self.blocki_beg+a, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed a {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, a, self.blocki_beg + a, self.blocki_end);
        debug_assert_le!(self.blocki_beg+b, self.blocki_end, "LinePart occupies Block slice [{}…{}], with passed b {} creates invalid slice [{}…{}]", self.blocki_beg, self.blocki_end, b, self.blocki_beg + b, self.blocki_end);
        debug_assert_le!(b - a, self.len(), "Passed LineIndex {}‥{} (diff {}) are larger than this LinePart len {}", a, b, b - a, self.len());
        let slice1 = &(*self.blockp).as_slice()[(self.blocki_beg+a)..(self.blocki_beg+b)];

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
#[derive(Debug)]
pub enum LinePartPtrs<'a> {
    /// empty line or some other null-like or false-like condition
    NoPtr,
    /// one box pointer needed represent the entire `Line`
    SinglePtr(Box<&'a [u8]>),
    /// two box pointers needed represent the entire `Line`
    DoublePtr(Box<&'a [u8]>, Box<&'a [u8]>),
    /// three or more box pointers needed to represent the entire `Line`
    MultiPtr(Vec<Box<&'a [u8]>>),
}

impl<'a> LinePartPtrs<'a> {
    /// to aid testing
    pub fn is_no_ptr(&self) -> bool {
        match self {
            LinePartPtrs::NoPtr => true,
            _ => false,
        }
    }
    /// to aid testing
    pub fn is_single_ptr(&self) -> bool {
        match self {
            LinePartPtrs::SinglePtr(_) => true,
            _ => false,
        }
    }
    /// to aid testing
    pub fn is_double_ptr(&self) -> bool {
        match self {
            LinePartPtrs::DoublePtr(_, _) => true,
            _ => false,
        }
    }
    /// to aid testing
    pub fn is_multi_ptr(&self) -> bool {
        match self {
            LinePartPtrs::MultiPtr(_) => true,
            _ => false,
        }
    }
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
    pub fn append(&mut self, linepart: LinePart) {
        dpo!("Line.append({:?}) {:?}", &linepart, linepart.to_String_noraw());
        let l_ = self.lineparts.len();
        if l_ > 0 {
            // sanity checks; each `LinePart` should be stored in same order as it appears in the file
            // only need to compare to last `LinePart`
            let li = &self.lineparts[l_ - 1];
            assert_le!(
                li.blockoffset,
                linepart.blockoffset,
                "Line.append: Prior stored LinePart at blockoffset {} is after passed LinePart at blockoffset {}\n{:?}\n{:?}\n",
                li.blockoffset,
                linepart.blockoffset,
                li,
                linepart,
            );
            assert_lt!(
                li.fileoffset,
                linepart.fileoffset,
                "Line.append: Prior stored LinePart at fileoffset {} is at or after passed LinePart at fileoffset {}\n{:?}\n{:?}\n",
                li.fileoffset,
                linepart.fileoffset,
                li,
                linepart,
            );
        }
        self.lineparts.push(linepart);
    }

    /// insert `linepart` at front of `self.lineparts`
    pub fn prepend(&mut self, linepart: LinePart) {
        dpo!("Line.prepend({:?}) {:?}", &linepart, linepart.to_String_noraw());
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

    /// return the first `BlockOffset`s on which data for this `Line` resides.
    ///
    /// Presumes underlying `LinePart` hold data else panic!
    pub fn blockoffset_first(self: &Line) -> BlockOffset {
        self.lineparts[0].blockoffset
    }

    /// Return the last `BlockOffset`s on which data for this `Line` resides.
    ///
    /// Presumes underlying `LinePart` hold data else panic!
    pub fn blockoffset_last(self: &Line) -> BlockOffset {
        self.lineparts[self.lineparts.len() - 1].blockoffset
    }

    /// do the bytes of this `Line` reside on one `Block`?
    pub fn occupies_one_block(self: &Line) -> bool {
        self.blockoffset_first() == self.blockoffset_last()
    }

    /// length of this `Line` in bytes as calcuated from stored fileoffsets
    pub fn len(self: &Line) -> usize {
        (self.fileoffset_end() - self.fileoffset_begin() + 1) as usize
    }

    /// count of `LinePart` in `self.lineparts.len()`
    /// XXX: redundant, need to decide which to keep.
    pub fn count_lineparts(self: &Line) -> usize {
        self.lineparts.len()
    }

    /// sum of `LinePart.count_bytes`
    pub fn count_bytes(self: &Line) -> Count {
        let mut cb: Count = 0;
        for lp in self.lineparts.iter() {
            cb += lp.count_bytes();
        }
        cb
    }

    /// does this `Line` store a `LinePart.blockoffset == bo`?
    ///
    /// O(n)
    pub fn stores_blockoffset(self: &Line, bo: BlockOffset) -> bool {
        for linepart in self.lineparts.iter() {
            if linepart.blockoffset == bo {
                return true;
            }
        }
        false
    }

    /// return all slices that make up this `Line` within a `Vec`
    ///
    /// Only for testing
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

    /// return a count of slices that make up this `Line`
    pub fn count_slices(self: &Line) -> Count {
        self.lineparts.len() as Count
    }

    // XXX: due to unstable feature `Sized` in `Box`, cannot do
    //           fn get_boxptrs(...) -> either::Either<Box<&[u8]>, Vec<Box<&[u8]>>>
    //      causes error `experimental Sized`
    //
    // TODO: use `&Range_LineIndex` instead of `a` `b`
    //
    /// get Box pointer(s) to an underlying `&[u8]` slice that is part of this `Line`.
    /// `a` is inclusive, `b` is exclusive.
    ///
    /// If slice is refers to one `Linepart` then return a single `Box` pointer.
    ///
    /// If slice is composed of multiple `Linepart` then return a
    /// `Vec` of `Box` pointers to each part.
    ///
    /// The purpose of this function and `LinePartPtrs` is to provide fast access to
    /// some underlying slice(s) of a `Line` while hiding complexities of crossing
    /// `Block` boundaries (and not being lazy and copying lots of bytes around).
    ///
    pub fn get_boxptrs(self: &Line, mut a: LineIndex, mut b: LineIndex) -> LinePartPtrs<'_> {
        dpnf!("(…, {}, {}), lineparts {} line.len() {} {:?}", a, b, self.lineparts.len(), self.len(), self.to_String_noraw());
        debug_assert_le!(a, b, "passed bad LineIndex pair");
        // simple case: `a, b` are past end of `Line`
        if self.len() <= a {
            dpxf!("return NoPtr");
            return LinePartPtrs::NoPtr;
        }
        // ideal case: `a, b` are within one `linepart`
        // harder case: `a, b` are among two `linepart`s
        let mut a_found = false;
        let mut a1: LineIndex = a;
        let mut b1: LineIndex = b;
        // Box ptr to first `a` slice of `linepart`, also a flag for special case
        let mut bptr_a: Option<Box::<&[u8]>> = None;
        for linepart in &self.lineparts {
            let len_ = linepart.len();
            dpo!("next: a {}, b {}, len_ {}", a1, b1, len_);
            if a1 < len_ && b1 <= len_ && !a_found {
                // ideal case, very efficient
                dpxf!("return SinglePtr({}, {})", a1, b1);
                return LinePartPtrs::SinglePtr(linepart.block_boxptr_ab(&a1, &b1));
            } else if a1 < len_ && len_ < b1 && !a_found {
                a_found = true;
                bptr_a = Some(linepart.block_boxptr_a(&a1));
                b1 -= len_;
                dpo!("a_found: bptr_a = block_boxptr_a({})", a1);
            } else if b1 <= len_ && a_found {
                // harder case, pretty efficient
                dpxf!("return DoublePtr({}, {})", a1, b1);
                return LinePartPtrs::DoublePtr(bptr_a.unwrap(), linepart.block_boxptr_b(&b1));
            } else if len_ < b1 && a_found {
                dpo!("break: a {} < {} && {} < {} b && a_found", a1, len_, len_, b1);
                bptr_a = None;
                break;
            } else if a_found {
                dpo!("break: a_found");
                bptr_a = None;
                break;
            } else {
                a1 -= len_;
                b1 -= len_;
            }
        }
        // handle special case where `b` is beyond last `lineparts` but `a` data is within last `linepart`
        if bptr_a.is_some() {
            dpxf!("special case: return SinglePtr({})", a1);
            return LinePartPtrs::SinglePtr(bptr_a.unwrap());
        }

        // previous searches failed, so it must be the hardest case.
        // hardest case: `a, b` are among many `lineparts` (>=3 `Box` pointers required)
        //               less efficient (requires a new `Vec`)
        // TODO: cost-savings: vec capacity will often be less than `lineparts.len()`
        dpo!("Vec::with_capacity({})", self.lineparts.len());
        let mut a_found = false;
        let mut b_search = false;
        let mut ptrs: Vec<Box<&[u8]>> = Vec::<Box::<&[u8]>>::with_capacity(self.lineparts.len());
        for linepart in &self.lineparts {
            let len_ = linepart.len();
            if !a_found && a < len_ {
                a_found = true;
                b_search = true;
                if b < len_ {
                    dpo!("ptrs.push(linepart.block_boxptr_ab({}, {})) @Block[{:?}‥{:?}] @[{:?}‥{:?}]", a, b, linepart.blocki_beg, linepart.blocki_end, linepart.fileoffset_begin(), linepart.fileoffset_end());
                    ptrs.push(linepart.block_boxptr_ab(&a, &b));  // store [a..b]  (entire slice, entire `Line`)
                    debug_assert_gt!(ptrs.len(), 1, "ptrs is {} elements, expected >= 1; this should have been handled earlier", ptrs.len());
                    dpxf!("return MultiPtr {} ptrs", ptrs.len());
                    return LinePartPtrs::MultiPtr(ptrs);
                }
                dpo!("ptrs.push(linepart.block_boxptr_a({})) @Block[{:?}‥{:?}] @[{:?}‥{:?}]", a, linepart.blocki_beg, linepart.blocki_end, linepart.fileoffset_begin(), linepart.fileoffset_end());
                ptrs.push(linepart.block_boxptr_a(&a));  // store [a..]  (first slice of `Line`)
                b -= len_;
                continue;
            } else if !a_found {
                dpo!("next: !a_found, a {}, {} linepart.len(), a becomes {}", a, len_, a - len_);
                a -= len_;
                continue;
            }
            if b_search && b < len_ {
                dpo!("ptrs.push(linepart.block_boxptr_b({})) @Block[{:?}‥{:?}] @[{:?}‥{:?}]", b, linepart.blocki_beg, linepart.blocki_end, linepart.fileoffset_begin(), linepart.fileoffset_end());
                ptrs.push(linepart.block_boxptr_b(&b));  // store [..b] (last slice of `Line`)
                break;
            } else  {
                dpo!("ptrs.push(linepart.block_boxptr()) @Block[{:?}‥{:?}] @[{:?}‥{:?}]", linepart.blocki_beg, linepart.blocki_end, linepart.fileoffset_begin(), linepart.fileoffset_end());
                ptrs.push(linepart.block_boxptr());  // store [..] (entire slice, middle part of `Line`)
                b -= len_;
            }
        }
        debug_assert_gt!(ptrs.len(), 1, "Ptrs is length {}, expected >1; parsing algorithm missed this case", ptrs.len());

        LinePartPtrs::MultiPtr(ptrs)
    }

    /// `raw` true will write directly to stdout from the stored `Block`
    ///
    /// `raw` false will write transcode each byte to a character and use pictoral representations
    ///
    /// XXX: `raw==false` does not handle multi-byte encodings
    #[cfg(any(debug_assertions,test))]
    pub fn print(self: &Line, raw: bool) {
        // is this an expensive command? should `stdout` be cached?
        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        for linepart in &self.lineparts {
            // TODO: I'm somewhat sure this is not creating anything new but I should verify with `gdb-rust`.
            let slice = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            if raw {
                match stdout_lock.write(slice) {
                    Ok(_) => {}
                    Err(err) => {
                        p_err!(
                            "StdoutLock.write(@{:p}[{}‥{}]) error {}",
                            &*linepart.blockp, linepart.blocki_beg, linepart.blocki_end, err
                        );
                    }
                }
            } else {
                // XXX: only handle single-byte encodings
                // XXX: this is not efficient
                let s = match std::str::from_utf8(slice) {
                    Ok(val) => val,
                    Err(err) => {
                        p_err!("Invalid UTF-8 sequence during from_utf8: {:?}", err);
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
                            p_err!("StdoutLock.write({:?}) error {}", &dst, err);
                        }
                    }
                }
            }
        }
        match stdout_lock.flush() {
            Ok(_) => {}
            Err(err) => {
                p_err!("stdout flushing error {}", err);
            }
        }
    }

    /// create `String` from known bytes referenced by `self.lineparts`
    /// `raw` is `true` means use byte characters as-is
    /// `raw` is `false` means replace formatting characters or non-printable characters
    /// with pictoral representation (i.e. `byte_to_char_noraw`)
    ///
    /// XXX: very inefficient and not always correct! *only* intended to help humans visually
    ///      inspect stderr output
    ///
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub(crate) fn impl_to_String_raw(self: &Line, raw: bool) -> String {
        // get capacity
        let mut sz: usize = 0;
        for linepart in &self.lineparts {
            sz += linepart.len();
        }
        let mut buf = Bytes::with_capacity(sz);
        // copy lineparts to a buffer
        for linepart in &self.lineparts {
            let bptr = linepart.block_boxptr();
            for byte_ in (*bptr).iter() {
                buf.push(*byte_);
            }
        }
        // transform buffer to a `String`
        let s1: Cow<str> = String::from_utf8_lossy(&buf);
        let s3: String;
        if !raw {
            // replace "raw" formatting characters with associated glyphs
            let mut s2 = String::with_capacity(s1.len());
            for c_ in s1.chars() {
                if c_.is_ascii() {
                    s2.push(char_to_char_noraw(c_));
                } else {
                    s2.push(c_);
                }
            }
            s3 = s2;
        } else {
            s3 = String::from(s1);
        }

        s3
    }

    // XXX: rust does not support function overloading which is really surprising and disappointing
    /// `Line` to `String`
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String(self: &Line) -> String {
        self.impl_to_String_raw(true)
    }

    /// `Line` to `String` but using printable chars for non-printable and/or formatting characters
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions,test))]
    pub fn to_String_noraw(self: &Line) -> String {
        self.impl_to_String_raw(false)
    }

}