// src/readers/fixedstructreader.rs

//! Implements a [`FixedStructReader`],
//! the driver of deriving [`FixedStruct`s] from a fixed C-struct format file
//! using a [`BlockReader`].
//!
//! Sibling of [`SyslogProcessor`]. But simpler in a number of ways due to
//! predictable format of the fixedsturct files. Also, a `FixedStructReader`
//! does not presume entries are in chronological order.
//! Whereas `SyslogProcessor` presumes entries are in chronological order.
//! This makes a big difference for implementations.
//!
//! This is an _s4lib_ structure used by the binary program _s4_.
//!
//! Implements [Issue #70].
//!
//! [`FixedStructReader`]: self::FixedStructReader
//! [`FixedStruct`s]: crate::data::fixedstruct::FixedStruct
//! [`BlockReader`]: crate::readers::blockreader::BlockReader
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [Issue #70]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/70

// TODO: ask question on SO about difference in
//       `e_termination` and `e_exit` in `struct exit_status`
//       https://elixir.bootlin.com/glibc/glibc-2.37/source/bits/utmp.h#L48

use crate::{e_err, de_err, de_wrn};
use crate::common::{
    debug_panic,
    Count,
    FPath,
    FileOffset,
    FileSz,
    FileType,
    FixedStructFileType,
    filetype_to_logmessagetype,
    ResultS3,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    SystemTime,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    dt_after_or_before,
    dt_pass_filters,
};
use crate::data::fixedstruct::{
    buffer_to_fixedstructptr,
    convert_datetime_tvpair,
    filesz_to_types,
    FixedStruct,
    FixedStructDynPtr,
    FixedStructType,
    FixedStructTypeSet,
    ENTRY_SZ_MAX,
    ENTRY_SZ_MIN,
    Score,
    TIMEVAL_SZ_MAX,
    tv_pair_type,
};
use crate::readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockReader,
    BlockSz,
    ResultReadDataToBuffer,
};
use crate::readers::summary::Summary;

use std::collections::{BTreeMap, LinkedList};
use std::fmt;
use std::io::{Error, ErrorKind, Result};

use ::mime_guess::MimeGuess;
use ::more_asserts::{debug_assert_ge, debug_assert_le};
#[allow(unused_imports)]
use ::si_trace_print::{
    de,
    defn,
    defo,
    defx,
    defñ,
    def1ñ,
    def1n,
    def1o,
    def1x,
    den,
    deo,
    dex,
    deñ,
    pfo,
    pfn,
    pfx,
};


// -----------------
// FixedStructReader

/// Map [`FileOffset`] To [`FixedStruct`].
///
/// Storage for `FixedStruct` found from the underlying `BlockReader`.
/// FileOffset key is the first byte/offset that begins the `FixedStruct`.
///
/// [`FileOffset`]: crate::common::FileOffset
/// [`FixedStruct`]: crate::data::fixedstruct::FixedStruct
pub type FoToEntry = BTreeMap<FileOffset, FixedStruct>;

/// Map [`FileOffset`] To `FileOffset`
///
/// [`FileOffset`]: crate::common::FileOffset
pub type FoToFo = BTreeMap<FileOffset, FileOffset>;

pub type FoList = LinkedList<FileOffset>;

type MapTvPairToFo = BTreeMap<tv_pair_type, FileOffset>;

/// [`FixedStructReader.find`*] functions results.
///
/// [`FixedStructReader.find`*]: self::FixedStructReader#method.find_entry
pub type ResultS3FixedStructFind = ResultS3<(FileOffset, FixedStruct), (Option<FileOffset>, Error)>;

pub type ResultS3FixedStructProcZeroBlock = ResultS3<(FixedStructType, Score, ListFileOffsetFixedStructPtr), Error>;

pub type ResultTvFo = Result<(usize, usize, usize, usize, MapTvPairToFo)>;

#[cfg(test)]
pub type DroppedBlocks = LinkedList<BlockOffset>;

type ListFileOffsetFixedStructPtr = LinkedList<(FileOffset, FixedStructDynPtr)>;

/// Enum return value for [`FixedStructReader::new`].
#[derive(Debug)]
pub enum ResultFixedStructReaderNew<E> {
    /// `FixedStructReader::new` was successful and returns the
    /// `FixedStructReader`
    FileOk(FixedStructReader),
    FileErrEmpty,
    FileErrTooSmall(String),
    /// No valid fixedstruct
    FileErrNoValidFixedStruct,
    /// No fixedstruct within the datetime filters
    FileErrNoFixedStructWithinDtFilters,
    /// Carries the `E` error data. This is how an [`Error`] is carried between
    /// a processing thread and the main printing thread
    FileErrIo(E),
}

pub type ResultFixedStructReaderNewError = ResultFixedStructReaderNew<Error>;

#[derive(Debug)]
pub enum ResultFixedStructReaderScoreFile<E> {
    /// `score_file` was successful; return the `FixedStructType`, `Score`,
    /// already processed `FixedStrcutDynPtr` entries (with associated offsets)
    FileOk(FixedStructType, Score, ListFileOffsetFixedStructPtr),
    FileErrEmpty,
    /// No valid fixedstruct
    FileErrNoValidFixedStruct,
    /// No high score for the file
    FileErrNoHighScore,
    /// Carries the `E` error data. This is how an [`Error`] is carried between
    /// a processing thread and the main printing thread
    FileErrIo(E),
}

pub type ResultFixedStructReaderScoreFileError = ResultFixedStructReaderScoreFile<Error>;

/// A specialized reader that uses [`BlockReader`] to read [`FixedStruct`]
/// entries in a file.
///
/// The `FixedStructReader` converts `\[u8\]` to `FixedStruct` in
/// [`buffer_to_fixedstructptr`].
///
/// ## Summary of operation
///
/// A `FixedStructReader` first deteremines the `FixedStructType` of the file
/// in [`preprocess_fixedstructtype`].
/// Then it scans all ***t***ime ***v***alues in each entry to determine the
/// order to process the entries in [`preprocess_timevalues`]. This implies the
/// `blockreader` must read the entire file into memory. So far, "in the wild"
/// user accounting records are only a few kilobytes at most. So reading the
/// entire file into memory should not put too much strain on memory usage.
/// The processing of time values is done first and for the entire file
/// because records may not be stored in chronological order.
/// Then the caller makes repeated calls to [`process_entry_at`] which processes
/// the `FixedStruct`s found in the file.
///
/// 0x00 byte and 0xFF byte fixedstructs are considered a null entry and
/// ignored.
///
/// _XXX: not a rust "Reader"; does not implement trait [`Read`]._
///
/// [`buffer_to_fixedstructptr`]: crate::data::fixedstruct::buffer_to_fixedstructptr
/// [`BlockReader`]: crate::readers::blockreader::BlockReader
/// [`Read`]: std::io::Read
/// [`preprocess_fixedstructtype`]: FixedStructReader::preprocess_fixedstructtype
/// [`preprocess_timevalues`]: FixedStructReader::preprocess_timevalues
/// [`process_entry_at`]: FixedStructReader::process_entry_at
pub struct FixedStructReader
{
    pub(crate) blockreader: BlockReader,
    fixedstruct_type: FixedStructType,
    fixedstructfiletype: FixedStructFileType,
    /// Size of a single [`FixedStruct`] entry.
    fixedstruct_size: usize,
    /// The highest score found during `preprocess_file`.
    /// Used to determine the `FixedStructType` of the file.
    high_score: Score,
    /// Timezone to use for conversions using function
    /// [`convert_tvpair_to_datetime`].
    ///
    /// [`convert_tvpair_to_datetime`]: crate::data::fixedstruct::convert_tvpair_to_datetime
    tz_offset: FixedOffset,
    /// A temporary hold for [`FixedStruct`] entries found
    /// by [`preprocess_fixedstructtype`]. Use `insert_cache_entry` and `remove_entry`
    /// to manage this cache.
    ///
    /// [`preprocess_fixedstructtype`]: FixedStructReader::preprocess_fixedstructtype
    pub(crate) cache_entries: FoToEntry,
    /// A mapping of all entries in the entire file (that pass the datetime
    /// filters), mapped by [`tv_pair_type`] to [`FileOffset`]. Created by
    /// [`preprocess_timevalues`].
    ///
    /// [`preprocess_timevalues`]: FixedStructReader::preprocess_timevalues
    pub(crate) map_tvpair_fo: MapTvPairToFo,
    pub(crate) block_use_count: BTreeMap<BlockOffset, usize>,
    /// The first entry found in the file, by `FileOffset`
    pub(crate) first_entry_fileoffset: FileOffset,
    /// "high watermark" of `FixedStruct` stored in `self.cache_entries`
    pub(crate) entries_stored_highest: usize,
    pub(crate) entries_out_of_order: usize,
    /// Internal stats - hits of `self.cache_entries` in `find_entry*` functions.
    pub(super) entries_hits: Count,
    /// Internal stats - misses of `self.cache_entries` in `find_entry*` functions.
    pub(super) entries_miss: Count,
    /// `Count` of `FixedStruct`s processed.
    ///
    /// Distinct from `self.cache_entries.len()` as that may have contents removed.
    pub(super) entries_processed: Count,
    /// First (soonest) processed [`DateTimeL`] (not necessarily printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub(super) dt_first: DateTimeLOpt,
    /// Last (latest) processed [`DateTimeL`] (not necessarily printed,
    /// not representative of the entire file).
    ///
    /// Intended for `--summary`.
    ///
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub(super) dt_last: DateTimeLOpt,
    /// `Count` of dropped `FixedStruct`.
    pub(super) drop_entry_ok: Count,
    /// `Count` of failed drop attempts of `FixedStruct`.
    pub(super) drop_entry_errors: Count,
    /// Largest `BlockOffset` of successfully dropped blocks.
    pub(super) blockoffset_drop_last: BlockOffset,
    /// testing-only tracker of successfully dropped `FixedStruct`
    #[cfg(test)]
    pub(crate) dropped_blocks: DroppedBlocks,
    pub(super) map_tvpair_fo_max_len: usize,
    /// The last [`Error`], if any, as a `String`. Set by [`set_error`].
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    /// [`set_error`]: self::FixedStructReader#method.set_error
    // TRACKING: https://github.com/rust-lang/rust/issues/24135
    error: Option<String>,
}

impl fmt::Debug for FixedStructReader
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("FixedStructReader")
            .field("Path", &self.path())
            .field("Entries", &self.cache_entries.len())
            .field("tz_offset", &self.tz_offset)
            .field("dt_first", &self.dt_first)
            .field("dt_last", &self.dt_last)
            .field("Error?", &self.error)
            .finish()
    }
}

// TODO: [2023/04] remove redundant variable prefix name `fixedstructreader_`
// TODO: [2023/05] instead of having 1:1 manual copying of `FixedStructReader`
//       fields to `SummaryFixedStructReader` fields, just store a
//       `SummaryFixedStructReader` in `FixedStructReader` and update directly.
#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryFixedStructReader {
    pub fixedstructreader_fixedstructtype_opt: Option<FixedStructType>,
    pub fixedstructreader_fixedstructfiletype_opt: Option<FixedStructFileType>,
    pub fixedstructreader_fixedstruct_size: usize,
    pub fixedstructreader_high_score: Score,
    pub fixedstructreader_utmp_entries: Count,
    pub fixedstructreader_first_entry_fileoffset: FileOffset,
    pub fixedstructreader_entries_out_of_order: usize,
    pub fixedstructreader_utmp_entries_max: Count,
    pub fixedstructreader_utmp_entries_hit: Count,
    pub fixedstructreader_utmp_entries_miss: Count,
    pub fixedstructreader_drop_entry_ok: Count,
    pub fixedstructreader_drop_entry_errors: Count,
    /// datetime soonest seen (not necessarily reflective of entire file)
    pub fixedstructreader_datetime_first: DateTimeLOpt,
    /// datetime latest seen (not necessarily reflective of entire file)
    pub fixedstructreader_datetime_last: DateTimeLOpt,
    pub fixedstructreader_map_tvpair_fo_max_len: usize,
}

/// Implement the FixedStructReader.
impl FixedStructReader
{
    /// Create a new `FixedStructReader`.
    ///
    /// **NOTE:** this `new()` calls [`BlockerReader.read_block`],
    /// dissimilar from other
    /// `*Readers::new()` which try to avoid calls to `read_block`.
    /// This means the reading may return `Done` (like if the file is empty) and
    /// `Done` must be reflected in the return value of `new`. Hence this
    /// function has a specialized return value.
    ///
    /// [`BlockerReader.read_block`]: crate::readers::blockreader::BlockReader#method.read_block
    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz: BlockSz,
        tz_offset: FixedOffset,
        dt_filter_after: DateTimeLOpt,
        dt_filter_before: DateTimeLOpt,
    ) -> ResultFixedStructReaderNewError {
        def1n!(
            "({:?}, filetype={:?}, blocksz={:?}, {:?}, {:?}, {:?})",
            path, filetype, blocksz, tz_offset, dt_filter_after, dt_filter_before,
        );
        let mut blockreader = match BlockReader::new(
            path.clone(), filetype, blocksz
        ) {
            Ok(blockreader_) => blockreader_,
            Err(err) => {
                def1x!("return Err {}", err);
                //return Some(Result::Err(err));
                return ResultFixedStructReaderNew::FileErrIo(err);
            }
        };
        let fixedstructfiletype = match filetype {
            FileType::FixedStruct{type_: fixedstructfiletype_} => fixedstructfiletype_,
            _ => {
                debug_panic!("Unexpected FileType: {:?}", filetype);
                return ResultFixedStructReaderNew::FileErrIo(
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("Unexpected FileType {:?}", filetype),
                    )
                );
            }
        };

        const ENTRY_SZ_MIN_FSZ: FileSz = ENTRY_SZ_MIN as FileSz;
        if blockreader.filesz() == 0 {
            def1x!("return FileErrEmpty");
            return ResultFixedStructReaderNew::FileErrEmpty;
        } else if blockreader.filesz() < ENTRY_SZ_MIN_FSZ {
            def1x!(
                "return FileErrTooSmall; {} < {} (ENTRY_SZ_MIN)",
                blockreader.filesz(), ENTRY_SZ_MIN_FSZ
            );
            return ResultFixedStructReaderNew::FileErrTooSmall(
                format!(
                    "file size {} < {} (ENTRY_SZ_MIN), file {:?}",
                    blockreader.filesz(), ENTRY_SZ_MIN_FSZ, path,
                )
            );
        }

        // preprocess the file, pass `oneblock=false` to process
        // the entire file. This is because `lastlog` files are often nearly
        // entirely null bytes until maybe one entry near the end, so this should
        // search past the first block of data.
        let (
            fixedstruct_type,
            high_score,
            list_entries,
        ) = match FixedStructReader::preprocess_fixedstructtype(
            &mut blockreader, &fixedstructfiletype, false,
        ) {
            ResultFixedStructReaderScoreFileError::FileOk(
                fixedstruct_type_, high_score_, list_entries_,
            ) => (fixedstruct_type_, high_score_, list_entries_),
            ResultFixedStructReaderScoreFileError::FileErrEmpty => {
                def1x!("return FileErrEmpty");
                return ResultFixedStructReaderNew::FileErrEmpty;
            }
            ResultFixedStructReaderScoreFileError::FileErrNoHighScore => {
                def1x!("return FileErrNoHighScore");
                return ResultFixedStructReaderNew::FileErrNoValidFixedStruct;
            }
            ResultFixedStructReaderScoreFileError::FileErrNoValidFixedStruct => {
                def1x!("return FileErrNoValidFixedStruct");
                return ResultFixedStructReaderNew::FileErrNoValidFixedStruct;
            }
            ResultFixedStructReaderScoreFileError::FileErrIo(err) => {
                de_err!("FixedStructReader::preprocess_fixedstructtype Error {}; file {:?}",
                        err, blockreader.path());
                def1x!("return Err {:?}", err);
                return ResultFixedStructReaderNew::FileErrIo(err);
            }
        };

        let (total, invalid, valid_no_filter, out_of_order, map_tvpair_fo) = 
            match FixedStructReader::preprocess_timevalues(
                &mut blockreader,
                fixedstruct_type,
                &dt_filter_after,
                &dt_filter_before,
            )
        {
            ResultTvFo::Err(err) => {
                de_err!("FixedStructReader::preprocess_timevalues Error {}; file {:?}",
                        err, blockreader.path());
                def1x!("return Err {:?}", err);
                return ResultFixedStructReaderNew::FileErrIo(err);
            }
            ResultTvFo::Ok(
                (total_, invalid_, valid_no_filter_, out_of_order_, map_tvpair_fo_)
            ) =>
                (total_, invalid_, valid_no_filter_, out_of_order_, map_tvpair_fo_),
        };
        def1o!("total: {}, invalid: {}, valid_no_filter: {}, out_of_order: {}",
               total, invalid, valid_no_filter, out_of_order);
        #[cfg(debug_assertions)]
        {
            def1o!("map_tvpair_fo has {} entries", map_tvpair_fo.len());
            for (_tv_pair, _fo) in map_tvpair_fo.iter() {
                def1o!("map_tvpair_fo: [tv_pair: {:?}] = fo: {}", _tv_pair, _fo);
            }
        }
        debug_assert_ge!(total, invalid);
        debug_assert_ge!(total, valid_no_filter);

        if map_tvpair_fo.is_empty() {
            if valid_no_filter > 0 {
                def1x!("return FileErrNoFixedStructWithinDtFilters");
                return ResultFixedStructReaderNew::FileErrNoFixedStructWithinDtFilters;
            }
            def1x!("return FileErrNoValidFixedStruct");
            return ResultFixedStructReaderNew::FileErrNoValidFixedStruct;
        }

        // set `first_entry_fileoffset` to the first entry by fileoffset
        let mut first_entry_fileoffset: FileOffset = blockreader.filesz();
        for (_tv_pair, fo) in map_tvpair_fo.iter() {
            if &first_entry_fileoffset > fo {
                first_entry_fileoffset = *fo;
            }
        }
        debug_assert_ne!(
            first_entry_fileoffset, blockreader.filesz(),
            "failed to update first_entry_fileoffset"
        );

        // create a mapping of blocks to not-yet-processed entries
        // later on, this will be used to proactively drop blocks
        let mut block_use_count: BTreeMap<BlockOffset, usize> = BTreeMap::new();
        def1o!("block_use_count create");
        for (_tv_pair, fo) in map_tvpair_fo.iter() {
            let bo_beg: BlockOffset = BlockReader::block_offset_at_file_offset(*fo, blocksz);
            let fo_end: FileOffset = *fo + fixedstruct_type.size() as FileOffset;
            let bo_end: BlockOffset = BlockReader::block_offset_at_file_offset(fo_end, blocksz);
            def1o!("blocksz = {}", blocksz);
            for bo in bo_beg..bo_end+1 {
                match block_use_count.get_mut(&bo) {
                    Some(count) => {
                        let count_ = *count + 1;
                        def1o!(
                            "block_use_count[{}] += 1 ({}); [{}‥{}]; total span [{}‥{})",
                            bo, count_, *fo, fo_end,
                            BlockReader::file_offset_at_block_offset(bo_beg, blocksz),
                            BlockReader::file_offset_at_block_offset(bo_end + 1, blocksz),
                        );
                        *count = count_;
                    }
                    None => {
                        def1o!(
                            "block_use_count[{}] = 1; [{}‥{}]; total span [{}‥{})",
                            bo, *fo, fo_end,
                            BlockReader::file_offset_at_block_offset(bo_beg, blocksz),
                            BlockReader::file_offset_at_block_offset(bo_end + 1, blocksz),
                        );
                        block_use_count.insert(bo, 1);
                    }
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            for (bo, count) in block_use_count.iter() {
                def1o!(
                    "block_use_count[{}] = {}; total span [{}‥{}]",
                    bo, count,
                    BlockReader::file_offset_at_block_offset(*bo, blocksz),
                    BlockReader::file_offset_at_block_offset(*bo + 1, blocksz),
                );
            }
        }

        let map_max_len = map_tvpair_fo.len();
        // now that the `fixedstruct_type` is known, create the FixedStructReader
        let mut fixedstructreader = FixedStructReader
        {
            blockreader,
            fixedstruct_type,
            fixedstructfiletype,
            fixedstruct_size: fixedstruct_type.size(),
            high_score,
            tz_offset,
            cache_entries: FoToEntry::new(),
            map_tvpair_fo,
            block_use_count,
            first_entry_fileoffset,
            entries_stored_highest: 0,
            entries_out_of_order: out_of_order,
            entries_hits: 0,
            entries_miss: 0,
            entries_processed: 0,
            dt_first: DateTimeLOpt::None,
            dt_last: DateTimeLOpt::None,
            drop_entry_ok: 0,
            drop_entry_errors: 0,
            blockoffset_drop_last: 0,
            #[cfg(test)]
            dropped_blocks: DroppedBlocks::new(),
            map_tvpair_fo_max_len: map_max_len,
            error: None,
        };

        // store the entries found and processed during `preprocess_file` into
        // `fixedstructreader.cache_entries`, to avoid duplicating work later on
        for (fo, fixedstructptr) in list_entries.into_iter() {
            // TODO: cost-savings: if `FixedStructTrait` had a `tv_pair` function then that time
            //       value could be checked against `map_tvpair_fo` *before* creating a new
            //       `FixedStruct`. However, the maximum number of `FixedStruct` entries
            //       created and then discarded will be very few so this is a marginal
            //       improvement.
            match FixedStruct::from_fixedstructptr(
                fo, &tz_offset, fixedstructptr
            ) {
                Ok(fixedstruct) => {
                    if fixedstructreader.map_tvpair_fo.iter().find(|(_tv_pair, fo2)| &fo == *fo2).is_some() {
                        def1o!("insert entry at fo {}", fo);
                        fixedstructreader.insert_cache_entry(fixedstruct);
                    } else {
                        def1o!("skip entry at fo {}; not in map_tvpair_fo", fo);
                    }
                }
                Err(err) => {
                    de_err!("FixedStruct::from_fixedstructptr Error {}; file {:?}",
                            err, fixedstructreader.path());
                    fixedstructreader.set_error(&err);
                }
            }
        }

        def1x!("return FileOk(FixedStructReader)");

        ResultFixedStructReaderNew::FileOk(fixedstructreader)
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

    /// See [`BlockReader::mimeguess`].
    ///
    /// [`BlockReader::mimeguess`]: crate::readers::blockreader::BlockReader#method.mimeguess
    #[inline(always)]
    pub const fn mimeguess(&self) -> MimeGuess {
        self.blockreader.mimeguess()
    }

    /// See [`BlockReader::mtime`].
    ///
    /// [`BlockReader::mtime`]: crate::readers::blockreader::BlockReader#method.mtime
    pub fn mtime(&self) -> SystemTime {
        self.blockreader.mtime()
    }

    /// `Count` of `FixedStruct`s processed by this `FixedStructReader`
    /// (i.e. `self.entries_processed`).
    #[inline(always)]
    pub fn count_entries_processed(&self) -> Count {
        self.entries_processed
    }

    /// "_High watermark_" of `FixedStruct` stored in `self.cache_entries`.
    #[inline(always)]
    pub fn entries_stored_highest(&self) -> usize {
        self.entries_stored_highest
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

    /// Is the passed `FixedStruct` the last of the file?
    #[inline(always)]
    pub fn is_last(
        &self,
        fixedstruct: &FixedStruct,
    ) -> bool {
        self.is_fileoffset_last(fixedstruct.fileoffset_end() - 1)
    }

    /// Return the `FileOffset` that is adjusted to the beginning offset of
    /// a `fixedstruct` entry.
    #[inline(always)]
    pub const fn fileoffset_to_fixedstructoffset (
        &self,
        fileoffset: FileOffset,
    ) -> FileOffset {
        (fileoffset / self.fixedstruct_size_fo()) * self.fixedstruct_size_fo()
    }

    /// Return the first file offset from `self.map_tvpair_fo` for the first
    /// entry as sorted by `tv_pair_type` (datetime); i.e. the earliest entry.
    /// Ties are broken by `FileOffset`.
    pub fn fileoffset_first(&self) -> Option<FileOffset> {
        match self.map_tvpair_fo.iter().min_by_key(|(tv_pair, fo)| (*tv_pair, *fo)) {
            Some((_tv_pair, fo_)) => Some(*fo_),
            None => None,
        }
    }

    /// The size in bytes of the `FixedStruct` entries managed by this
    /// `FixedStructReader`.
    #[inline(always)]
    pub const fn fixedstruct_size(&self) -> usize {
        self.fixedstruct_size
    }

    /// [`fixedstruct_size`] as a `FileOffset`.
    ///
    /// [`fixedstruct_size`]: self::FixedStructReader#method.fixedstruct_size
    #[inline(always)]
    pub const fn fixedstruct_size_fo(&self) -> FileOffset {
        self.fixedstruct_size() as FileOffset
    }

    /// The `FixedStructType` of the file.
    #[inline(always)]
    pub const fn fixedstruct_type(&self) -> FixedStructType {
        self.fixedstruct_type
    }

    /// Return all currently stored `FileOffset` in `self.cache_entries`.
    ///
    /// Only for testing.
    #[cfg(test)]
    pub fn get_fileoffsets(&self) -> Vec<FileOffset> {
        self.cache_entries
            .keys()
            .cloned()
            .collect()
    }

    /// store an `Error` that occurred. For later printing during `--summary`.
    // XXX: duplicates `SyslogProcessor.set_error`
    fn set_error(
        &mut self,
        error: &Error,
    ) {
        def1ñ!("{:?}", error);
        let mut error_string: String = error.kind().to_string();
        error_string.push_str(": ");
        error_string.push_str(error.kind().to_string().as_str());
        // print the error but avoid printing the same error more than once
        // XXX: This is somewhat a hack as it's possible the same error, with the
        //      the same error message, could occur more than once.
        //      Considered another way, this function `set_error` may get called
        //      too often. The responsibility for calling `set_error` is haphazard.
        match &self.error {
            Some(err_s) => {
                if err_s != &error_string {
                    e_err!("{}", error);
                }
            }
            None => {
                e_err!("{}", error);
            }
        }
        if let Some(ref _err) = self.error {
            de_wrn!("skip overwrite of previous Error ({:?}) with Error ({:?})", _err, error);
            return;
        }
        self.error = Some(error_string);
    }

    /// Store information about a single [`FixedStruct`] entry.
    ///
    /// Should only be called by `FixedStructReader::new`
    ///
    /// [`FixedStruct`]: crate::data::fixedstruct::FixedStruct
    fn insert_cache_entry(
        &mut self,
        entry: FixedStruct,
    ) {
        defn!("@{}", entry.fileoffset_begin());
        let fo_beg: FileOffset = entry.fileoffset_begin();
        debug_assert!(
            !self.cache_entries.contains_key(&fo_beg),
            "self.cache_entries already contains key {}",
            fo_beg
        );

        // update some stats and (most importantly) `self.cache_entries`
        self.cache_entries
            .insert(fo_beg, entry);
        self.entries_stored_highest = std::cmp::max(
            self.entries_stored_highest, self.cache_entries.len()
        );
        self.entries_processed += 1;
        defo!("entries_processed = {}", self.entries_processed);

        defx!();
    }

    /// Update the statistic `DateTimeL` of
    /// `self.dt_first` and `self.dt_last`
    fn dt_first_last_update(
        &mut self,
        datetime: &DateTimeL,
    ) {
        defñ!("({:?})", datetime);
        // TODO: cost-savings: the `dt_first` and `dt_last` are only for `--summary`,
        //       no need to always copy datetimes.
        //       Would be good to only run this when `if self.do_summary {...}`
        match self.dt_first {
            Some(dt_first_) => {
                if &dt_first_ > datetime {
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
                    self.dt_last = Some(*datetime);
                }
            }
            None => {
                self.dt_last = Some(*datetime);
            }
        }
    }

    /// Proactively `drop` the [`Block`s] associated with the
    /// passed [`FixedStruct`]. Return count of dropped entries (0 or 1).
    ///
    /// _The caller must know what they are doing!_
    ///
    /// [`FixedStruct`]: crate::data::fixedstruct::FixedStruct
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`FileOffset`]: crate::common::FileOffset
    fn drop_entry(
        &mut self,
        fixedstruct: &FixedStruct,
    ) -> usize {
        let bsz: BlockSz = self.blocksz();
        defn!(
            "(fixedstruct@{}); offsets [{}‥{}), blocks [{}‥{}]",
            fixedstruct.fileoffset_begin(),
            fixedstruct.fileoffset_begin(),
            fixedstruct.fileoffset_end(),
            fixedstruct.blockoffset_begin(bsz),
            fixedstruct.blockoffset_end(bsz),
        );
        let mut dropped_ok: usize = 0;
        let mut dropped_err: usize = 0;
        let mut bo_at: BlockOffset = fixedstruct.blockoffset_begin(bsz);
        let bo_end: BlockOffset = fixedstruct.blockoffset_end(bsz);
        debug_assert_le!(bo_at, bo_end);
        while bo_at <= bo_end {
            defo!("block_use_count.get_mut({})", bo_at);
            match self.block_use_count.get_mut(&bo_at) {
                Some(count) => {
                    if *count <= 1 {
                        defo!(
                            "block_use_count[{}] found; count=={}; total span [{}‥{})",
                            bo_at, count,
                            BlockReader::file_offset_at_block_offset(bo_at, bsz),
                            BlockReader::file_offset_at_block_offset(bo_end + 1, bsz),
                        );
                        if self
                            .blockreader
                            .drop_block(bo_at)
                        {
                            defo!(
                                "dropped block {}; total span [{}‥{})",
                                bo_at,
                                BlockReader::file_offset_at_block_offset(bo_at, bsz),
                                BlockReader::file_offset_at_block_offset(bo_end + 1, bsz),
                            );
                            // the largest blockoffset that has been dropped should also
                            // imply that all prior blockoffsets have been dropped
                            self.blockoffset_drop_last = std::cmp::max(bo_at, self.blockoffset_drop_last);
                            self.block_use_count.remove(&bo_at);
                            #[cfg(test)]
                            self.dropped_blocks.push_back(bo_at);
                            dropped_ok += 1;
                        } else {
                            defo!("failed to drop block {}", bo_at);
                            dropped_err += 1;
                        }
                    } else {
                        *count -= 1;
                        defo!("block_use_count[{}] found; count-=1=={}", bo_at, *count);
                    }
                }
                None => {
                    defo!("block_use_count[{}] not found", bo_at);
                }
            }
            bo_at += 1;
        }
        if dropped_ok > 0 {
            self.drop_entry_ok += 1;
        }
        if dropped_err > 0 {
            self.drop_entry_errors += 1;
        }
        defx!("return {}", dropped_ok);

        dropped_ok
    }

    /// Check the internal storage `self.cache_entries`.
    /// Remove the entry and return it if found.
    #[inline(always)]
    fn remove_cache_entry(
        &mut self,
        fileoffset: FileOffset,
    ) -> Option<FixedStruct> {
        match self.cache_entries.remove(&fileoffset) {
            Some(fixedstruct) => {
                defñ!("({}): found in store", fileoffset);
                self.entries_hits += 1;

                Some(fixedstruct)
            }
            None => {
                defñ!("({}): not found in store", fileoffset);
                self.entries_miss += 1;

                None
            }
        }
    }

    /// Process entries for the file managed by the `BlockReader`.
    /// Find the entry with the highest "score" as judged by `score_fixedstruct`.
    ///
    /// Returns the highest scored `FixedStructType`, that highest score,
    /// and any processed [`FixedStruct`] entries (referenced by a
    /// [`FixedStructDynPtr`]) in a list.
    ///
    /// Each list entry is a tuple of the
    /// the entries `FileOffset` and the `FixedStructDynPtr`.
    /// The entries will presumably be stored in the
    /// `FixedStructReader`'s `cache_entries`.
    pub fn score_file(
        blockreader: &mut BlockReader,
        oneblock: bool,
        types_to_bonus: FixedStructTypeSet,
    ) -> ResultFixedStructReaderScoreFileError {
        def1n!("(oneblock={}, types_to_bonus len {})", oneblock, types_to_bonus.len());
        #[cfg(debug_assertions)]
        {
            for (fixedstructtype, bonus) in types_to_bonus.iter() {
                def1o!(
                    "types_to_bonus: ({:<30?}, {:2}) size {}",
                    fixedstructtype, bonus, fixedstructtype.size(),
                );
            }
        }
        // allocate largest possible buffer needed on the stack
        let mut buffer: [u8; ENTRY_SZ_MAX] = [0; ENTRY_SZ_MAX];
        // only score this number of entries, i.e. don't walk the
        // entire file scoring all entries
        #[cfg(not(test))]
        const COUNT_FOUND_ENTRIES_MAX: usize = 5;
        #[cfg(test)]
        const COUNT_FOUND_ENTRIES_MAX: usize = 2;
        let mut _count_total: usize = 0;
        let mut highest_score: Score = 0;
        let mut highest_score_type: Option<FixedStructType> = None;
        let mut highest_score_entries = ListFileOffsetFixedStructPtr::new();

        for (fixedstructtype, bonus) in types_to_bonus.into_iter() {
            let mut _count_loop: usize = 0;
            let mut count_found_entries: usize = 0;
            let mut high_score: Score = 0;
            let mut fo: FileOffset = 0;
            let mut found_entries = ListFileOffsetFixedStructPtr::new();

            loop {
                if count_found_entries >= COUNT_FOUND_ENTRIES_MAX {
                    def1o!("count_found_entries {} >= COUNT_FOUND_ENTRIES_MAX {}", count_found_entries, COUNT_FOUND_ENTRIES_MAX);
                    break;
                }
                _count_total += 1;
                _count_loop += 1;
                let utmp_sz: usize = fixedstructtype.size();
                let fo_end = fo + utmp_sz as FileOffset;
                def1o!(
                    "loop try {} (total {}), fixedstructtype {:?}, zero the buffer (size {}), looking at fileoffset {}‥{} (0x{:08X}‥0x{:08X})",
                    _count_loop, _count_total, fixedstructtype, buffer.len(), fo, fo_end, fo, fo_end
                );
                // zero out the buffer
                // XXX: not strictly necessary to zero buffer but it helps humans
                //      manually reviewing debug logs
                // this compiles down to a single `memset` call
                // See https://godbolt.org/#g:!((g:!((g:!((h:codeEditor,i:(filename:'1',fontScale:14,fontUsePx:'0',j:1,lang:rust,selection:(endColumn:1,endLineNumber:5,positionColumn:1,positionLineNumber:5,selectionStartColumn:1,selectionStartLineNumber:5,startColumn:1,startLineNumber:5),source:'pub+fn+foo()+%7B%0A++++const+ENTRY_SZ_MAX:+usize+%3D+400%3B%0A++++let+mut+buffer:+%5Bu8%3B+ENTRY_SZ_MAX%5D+%3D+%5B0%3B+ENTRY_SZ_MAX%5D%3B%0A++++buffer.iter_mut().for_each(%7Cm%7C+*m+%3D+0)%3B%0A%0A++++std::hint::black_box(buffer)%3B%0A%7D'),l:'5',n:'0',o:'Rust+source+%231',t:'0')),k:41.18316477421653,l:'4',n:'0',o:'',s:0,t:'0'),(g:!((h:compiler,i:(compiler:r1750,filters:(b:'0',binary:'1',binaryObject:'1',commentOnly:'0',debugCalls:'1',demangle:'0',directives:'0',execute:'1',intel:'0',libraryCode:'1',trim:'1'),flagsViewOpen:'1',fontScale:14,fontUsePx:'0',j:1,lang:rust,libs:!(),options:'-O',overrides:!(),selection:(endColumn:1,endLineNumber:1,positionColumn:1,positionLineNumber:1,selectionStartColumn:1,selectionStartLineNumber:1,startColumn:1,startLineNumber:1),source:1),l:'5',n:'0',o:'+rustc+1.75.0+(Editor+%231)',t:'0')),k:42.788718063415104,l:'4',n:'0',o:'',s:0,t:'0'),(g:!((h:output,i:(editorid:1,fontScale:14,fontUsePx:'0',j:1,wrap:'1'),l:'5',n:'0',o:'Output+of+rustc+1.75.0+(Compiler+%231)',t:'0')),k:16.028117162368364,l:'4',n:'0',o:'',s:0,t:'0')),l:'2',n:'0',o:'',t:'0')),version:4
                // See https://godbolt.org/z/KcxW9hWYb
                buffer.iter_mut().for_each(|m| *m = 0);
                // read data into buffer
                let buffer_read: usize = match blockreader.read_data_to_buffer(
                    fo,
                    fo_end,
                    oneblock,
                    &mut buffer,
                ) {
                    ResultReadDataToBuffer::Found(buffer_read) => buffer_read,
                    ResultReadDataToBuffer::Err(err) => {
                        def1x!("return Err");
                        return ResultFixedStructReaderScoreFileError::FileErrIo(err);
                    }
                    ResultReadDataToBuffer::Done => {
                        // reached end of block (if `oneblock` is `true`) or end of file
                        break;
                    }
                };
                if buffer_read < utmp_sz {
                    def1o!(
                        "read_data_to_buffer read bytes {} < {} requested fixedstruct size bytes; break",
                        buffer_read, utmp_sz,
                    );
                    break;
                }
                // advance the file offset for the next loop
                let fo2 = fo;
                fo += utmp_sz as FileOffset;
                // grab the slice of interest
                let slice_ = &buffer[..buffer_read];
                // convert buffer to fixedstruct
                let fixedstructptr: FixedStructDynPtr = match buffer_to_fixedstructptr(slice_, fixedstructtype) {
                    Some(val) => val,
                    None => {
                        def1o!(
                            "buffer_to_fixedstructptr(buf len {}, {:?}) returned None; continue",
                            buffer.len(), fixedstructtype,
                        );
                        continue;
                    }
                };
                count_found_entries += 1;
                // score the newly create fixedstruct
                let score: Score = FixedStruct::score_fixedstruct(&fixedstructptr, bonus);
                def1o!("score {} for entry type {:?} @[{}‥{}]",
                       score, fixedstructptr.fixedstruct_type(), fo2, fo_end);
                // update the high score
                let _fs_type: FixedStructType = fixedstructptr.fixedstruct_type();
                found_entries.push_back((fo2, fixedstructptr));
                if score <= high_score {
                    def1o!(
                        "score {} ({:?}) not higher than high score {}",
                        score,
                        _fs_type,
                        high_score,
                    );
                    // score is not higher than high score so continue
                    continue;
                }
                // there is a new high score for this type
                def1o!("new high score {} for entry type {:?} @[{}‥{}]",
                       score, _fs_type, fo2, fo_end);
                high_score = score;
            }
            // finished with that fixedstructtype
            // so check if it's high score beat any previous high score of a different fixedstructtype
            if high_score > highest_score {
                // a new high score was found for a different type so throw away
                // linked lists of entries for the previous type
                match highest_score_type {
                    None => {
                        def1o!(
                            "new highest score {} entry type {:?} with {} entries",
                            high_score, fixedstructtype, found_entries.len(),
                        );
                    }
                    Some(_highest_score_type) => {
                        def1o!(
                            "new highest score {} entry type {:?} with {} entries; replaces old high score {} entry type {:?} with {} entries (entries dropped)",
                            high_score, fixedstructtype, found_entries.len(),
                            highest_score, _highest_score_type, highest_score_entries.len(),
                        );
                    }
                }
                highest_score = high_score;
                highest_score_type = Some(fixedstructtype);
                highest_score_entries = found_entries;
            } else {
                def1o!(
                    "no new highest score: score {} entry type {:?} with {} entries. old high score remains: score {} entry type {:?} with {} entries",
                    high_score, fixedstructtype, found_entries.len(),
                    highest_score, highest_score_type, highest_score_entries.len(),
                );
            }
        }

        match highest_score_type {
            None => {
                def1x!("return Err {:?}", ResultFixedStructReaderScoreFileError::FileErrNoHighScore);
                return ResultFixedStructReaderScoreFileError::FileErrNoHighScore;
            }
            Some(highest_score_type) => {
                def1x!("return Ok(({:?}, {}, found_entries))", highest_score_type, highest_score);

                ResultFixedStructReaderScoreFileError::FileOk(highest_score_type, highest_score, highest_score_entries)
            }
        }
    }

    /// Determine the `FixedStructType` based on the file size and data.
    ///
    /// 1. Makes best guess about file structure based on size by calling
    ///    `filesz_to_types`
    /// 2. Searches for a valid struct within the first block of the file (if
    ///    `oneblock` is `true`, else searches all available blocks).
    /// 3. Creates a [`FixedStruct`] from the found struct.
    ///
    /// Call this before calling `process_entry_at`.
    ///
    /// [OpenBSD file `w.c`] processes `/var/log/utmp`. Reviewing the code you
    /// get some idea of how the file is determined to be valid.
    ///
    /// [OpenBSD file `w.c`]: https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/usr.bin/w/w.c
    pub(crate) fn preprocess_fixedstructtype(
        blockreader: &mut BlockReader,
        fixedstructfiletype: &FixedStructFileType,
        oneblock: bool,
    ) -> ResultFixedStructReaderScoreFileError
    {
        def1n!("({:?}, oneblock={})", fixedstructfiletype, oneblock);

        // short-circuit special case of empty file
        if blockreader.filesz() == 0 {
            def1x!("empty file; return FileErrEmpty");
            return ResultFixedStructReaderScoreFileError::FileErrEmpty;
        }

        let types_to_bonus: FixedStructTypeSet = match filesz_to_types(
            blockreader.filesz(),
            fixedstructfiletype,
        ) {
            Some(set) => set,
            None => {
                de_wrn!("FixedStructReader::filesz_to_types({}) failed; file {:?}",
                        blockreader.filesz(), blockreader.path());
                def1x!("filesz_to_types returned None; return FileErrNoValidFixedStruct");
                return ResultFixedStructReaderScoreFileError::FileErrNoValidFixedStruct;
            }
        };
        def1o!("filesz_to_types returned {} types: {:?}", types_to_bonus.len(), types_to_bonus);

        match FixedStructReader::score_file(blockreader, oneblock, types_to_bonus) {
            ret => {
                def1x!("score_file returned {:?}", ret);

                ret
            }
        }
    }

    /// Jump to each entry offset, convert the raw bytes to a `tv_pair_type`.
    /// If the value is within the passed filters than save the value and the
    /// file offset of the entry.
    /// Return a count of out of order entries, and map of filtered tv_pair to
    /// fileoffsets. The fileoffsets in the map are the fixedstruct offsets
    /// (not timevalue offsets).
    pub(crate) fn preprocess_timevalues(
        blockreader: &mut BlockReader,
        fixedstruct_type: FixedStructType,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> ResultTvFo
    {
        defn!();
        // allocate largest possible buffer needed on the stack
        let mut buffer: [u8; TIMEVAL_SZ_MAX] = [0; TIMEVAL_SZ_MAX];
        // map of time values to file offsets
        let mut map_tv_pair_fo: MapTvPairToFo = MapTvPairToFo::new();
        // count of out of order entries
        let mut out_of_order: usize = 0;
        // valid fixedstruct but does not pass the time value filters
        let mut valid_no_pass_filter: usize = 0;
        // null fixedstruct or non-sense time values
        let mut invalid: usize = 0;
        // total number of entries processed
        let mut total_entries: usize = 0;

        let tv_filter_after: Option<tv_pair_type> = match dt_filter_after {
            Some(dt) => Some(convert_datetime_tvpair(dt)),
            None => None,
        };
        defo!("tv_filter_after: {:?}", tv_filter_after);
        let tv_filter_before: Option<tv_pair_type> = match dt_filter_before {
            Some(dt) => Some(convert_datetime_tvpair(dt)),
            None => None,
        };
        defo!("tv_filter_before: {:?}", tv_filter_before);

        // 1. get offsets
        let entry_sz: FileOffset = fixedstruct_type.size() as FileOffset;
        debug_assert_eq!(blockreader.filesz() % entry_sz, 0, "file not a multiple of entry size {}", entry_sz);
        let tv_sz: usize = fixedstruct_type.size_tv();
        let tv_offset: usize = fixedstruct_type.offset_tv();
        let slice_: &mut [u8] = &mut buffer[..tv_sz];
        let mut fo: FileOffset = 0;
        let mut tv_pair_prev: Option<tv_pair_type> = None;
        loop {
            // 2. jump to each offset, grab datetime bytes,
            let beg: FileOffset = fo + tv_offset as FileOffset;
            let end: FileOffset = beg + tv_sz as FileOffset;
            match blockreader.read_data_to_buffer(
                beg,
                end,
                false,
                slice_,
            ) {
                ResultReadDataToBuffer::Found(_readn) => {
                    defo!("read {} bytes at fileoffset {}", _readn, beg);
                    debug_assert_eq!(
                        _readn, tv_sz as usize,
                        "read {} bytes, expected {} bytes (size of a time value)",
                        _readn, tv_sz,
                    );
                }
                ResultReadDataToBuffer::Err(err) => {
                    defx!("return Err");
                    return ResultTvFo::Err(err);
                }
                ResultReadDataToBuffer::Done => {
                    defo!("return Done");
                    break;
                }
            }
            // 3. convert bytes to tv_sec, tv_usec
            let tv_pair: tv_pair_type = match fixedstruct_type.tv_pair_from_buffer(
                slice_,
            ) {
                Some(pair) => pair,
                None => {
                    de_err!("invalid entry at fileoffset {}", fo);
                    defo!("invalid entry at fileoffset {}", fo);
                    fo += entry_sz;
                    invalid += 1;
                    continue;
                }
            };
            defo!("tv_pair: {:?}", tv_pair);
            if tv_pair == tv_pair_type(0, 0) {
                defo!("tv_pair is (0, 0); continue");
                fo += entry_sz;
                continue;
            }
            match tv_pair_prev {
                Some(tv_pair_prev) => {
                    if tv_pair < tv_pair_prev {
                        out_of_order += 1;
                        defo!(
                            "out_of_order = {}; tv_pair = {:?}, tv_pair_prev = {:?}",
                            out_of_order, tv_pair, tv_pair_prev,
                        );
                    }
                }
                None => {}
            }
            tv_pair_prev = Some(tv_pair);
            total_entries += 1;
            // 4. compare to time value filters
            if let Some(tv_filter) = tv_filter_after {
                if tv_pair < tv_filter {
                    defo!("tv_pair {:?} < {:?} tv_filter_after; continue", tv_pair, tv_filter);
                    fo += entry_sz;
                    valid_no_pass_filter += 1;
                    continue;
                }
            }
            if let Some(tv_filter) = tv_filter_before {
                if tv_pair > tv_filter {
                    defo!("tv_pair {:?} > {:?} tv_filter_before; continue", tv_pair, tv_filter);
                    fo += entry_sz;
                    valid_no_pass_filter += 1;
                    // continue to check _all_ entries as the entries may not be
                    // in chronological order
                    continue;
                }
            }
            // 5. save entries that pass the time value filters
            defo!("tv_pair {:?} @{} passes time value filters", tv_pair, fo);
            map_tv_pair_fo.insert(tv_pair, fo);

            fo += entry_sz;
        }
        // 6. return list of valid entries
        defx!("map_tv_pair_fo len {}", map_tv_pair_fo.len());

        ResultTvFo::Ok(
            (total_entries, invalid, valid_no_pass_filter, out_of_order, map_tv_pair_fo)
        )
    }

    /// Process the data at FileOffset `fo`. Transform it into a `FixedStruct`
    /// using [`FixedStruct::new`].
    /// But before that, check private `self.cache_entries`
    /// in case the data at the fileoffset was already processed (transformed)
    /// during `FixedStructReader::new`.
    ///
    /// Let the caller pass a `buffer` to avoid this function having allocate.
    ///
    /// This function does the bulk of file processing after the
    /// `FixedStructReader` has been initialized during
    /// [`FixedStructReader::new`].
    pub fn process_entry_at(&mut self, fo: FileOffset, buffer: &mut [u8])
        -> ResultS3FixedStructFind
    {
        defn!("({})", fo);

        let sz: FileOffset = self.fixedstruct_size_fo();
        debug_assert_eq!(
            fo % sz, 0,
            "fileoffset {} not multiple of {}", fo, sz,
        );
        let fileoffset: FileOffset = fo - (fo % sz);

        if fileoffset >= self.filesz() {
            defx!(
                "return ResultS3FixedStructFind::Done; fileoffset {} >= filesz {}",
                fileoffset, self.filesz()
            );
            return ResultS3FixedStructFind::Done;
        }

        // The `map_tvpair_fo` is the oracle listing of entries ordered by
        // `tv_pair` (it was  fully created during `preprocess_timevalues`).
        // Search `map_tvpair_fo` for the passed `fo`, then return the
        // fileoffset of the entry *after that* which will be returned in `Found`.
        // If no next fileoffset is found it means `map_tvpair_fo` is empty.
        // In that case, the `fo_next` will be the value of `filesz()` and the
        // next call to `process_entry_at` will return `Done`.
        let fo_next: FileOffset = {
            let mut fo_next_: FileOffset = self.filesz();
            let mut next_pair: bool = false;
            let mut tv_pair_at_opt: Option<tv_pair_type> = None;
            // TODO: is there a rustic iterator way to
            //       "find something and return the next thing"?
            for (tv_pair_at, fo_at) in self.map_tvpair_fo.iter() {
                if next_pair {
                    defo!("set fo_next = {}", fo_at);
                    fo_next_ = *fo_at;
                    break;
                }
                if &fileoffset == fo_at {
                    defo!(
                        "found fileoffset {} with key {:?} in map_tvpair_fo",
                        fileoffset, tv_pair_at,
                    );
                    tv_pair_at_opt = Some(*tv_pair_at);
                    next_pair = true;
                }
            }
            // remove the `tv_pair` from `map_tvpair_fo`
            match tv_pair_at_opt {
                Some(tv_pair_at) => {
                    self.map_tvpair_fo.remove(&tv_pair_at);
                    defo!(
                        "remove tv_pair {:?}; map_tvpair_fo size {}",
                        tv_pair_at, self.map_tvpair_fo.len()
                    );
                }
                None => {
                    defo!("no map_tvpair_fo found!");
                }
            }

            fo_next_
        };
        defo!("fo_next = {}", fo_next);

        // check if the entry is already stored
        if let Some(fixedstruct) = self.remove_cache_entry(fileoffset) {
            self.dt_first_last_update(fixedstruct.dt());
            // try to drop blocks associated with the entry
            self.drop_entry(&fixedstruct);
            defx!(
                "remove_cache_entry found fixedstruct at fileoffset {}; return Found({}, …)",
                fileoffset, fo_next,
            );
            return ResultS3FixedStructFind::Found((fo_next, fixedstruct));
        }

        // the entry was not in the cache so read the raw bytes from the file
        // and transform them into a `FixedStruct`

        // check the buffer size
        if buffer.len() < sz as usize {
            defx!("return ResultS3FixedStructFind::Err");
            return ResultS3FixedStructFind::Err((
                None,
                Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "buffer size {} less than fixedstruct size {} at fileoffset {}, file {:?}",
                        buffer.len(), sz, fileoffset, self.path(),
                    ),
                )
            ));
        }

        // zero out the slice
        defo!("zero buffer[‥{}]", sz);
        let slice_: &mut [u8] = &mut buffer[..sz as usize];
        slice_.iter_mut().for_each(|m| *m = 0);

        // read raw bytes into the slice
        let _readn = match self.blockreader.read_data_to_buffer(
            fileoffset,
            fileoffset + sz,
            false,
            slice_,
        ) {
            ResultReadDataToBuffer::Found(val) => val,
            ResultReadDataToBuffer::Done => {
                defx!("return ResultS3FixedStructFind::Done; read_data_to_buffer returned Done");
                return ResultS3FixedStructFind::Done;
            }
            ResultReadDataToBuffer::Err(err) => {
                self.set_error(&err);
                defx!("return ResultS3FixedStructFind::Err({:?})", err);
                // an error from `blockreader.read_data_to_buffer` is unlikely to improve
                // with a retry so return `None` signifying no more processing of the file
                return ResultS3FixedStructFind::Err((None, err));
            }
        };
        debug_assert_eq!(_readn, sz as usize, "read {} bytes, expected {} bytes", _readn, sz);

        // create a FixedStruct from the slice
        let fs: FixedStruct = match FixedStruct::new(
            fileoffset,
            &self.tz_offset,
            &slice_,
            self.fixedstruct_type(),
        ) {
            Ok(val) => val,
            Err(err) => {
                defx!("return ResultS3FixedStructFind::Done; FixedStruct::new returned Err({:?})", err);
                return ResultS3FixedStructFind::Err((Some(fo_next), err));
            }
        };
        // update various statistics/counters
        self.entries_processed += 1;
        defo!("entries_processed = {}", self.entries_processed);
        self.dt_first_last_update(fs.dt());
        // try to drop blocks associated with the entry
        self.drop_entry(&fs);

        defx!("return ResultS3FixedStructFind::Found((fo_next={}, …))", fo_next);

        ResultS3FixedStructFind::Found((fo_next, fs))
    }

    /// Wrapper function for call to [`datetime::dt_after_or_before`] using the
    /// [`FixedStruct::dt`] of the `entry`.
    ///
    /// [`datetime::dt_after_or_before`]: crate::data::datetime::dt_after_or_before
    /// [`FixedStruct::dt`]: crate::data::fixedstruct::FixedStruct::dt
    pub fn entry_dt_after_or_before(
        entry: &FixedStruct,
        dt_filter: &DateTimeLOpt,
    ) -> Result_Filter_DateTime1 {
        defñ!("({:?})", dt_filter);

        dt_after_or_before(entry.dt(), dt_filter)
    }

    /// Wrapper function for call to [`datetime::dt_pass_filters`] using the
    /// [`FixedStruct::dt`] of the `entry`.
    ///
    /// [`datetime::dt_pass_filters`]: crate::data::datetime::dt_pass_filters
    /// [`FixedStruct::dt`]: crate::data::fixedstruct::FixedStruct::dt
    #[inline(always)]
    pub fn entry_pass_filters(
        entry: &FixedStruct,
        dt_filter_after: &DateTimeLOpt,
        dt_filter_before: &DateTimeLOpt,
    ) -> Result_Filter_DateTime2 {
        defn!("({:?}, {:?})", dt_filter_after, dt_filter_before);

        let result: Result_Filter_DateTime2 = dt_pass_filters(
            entry.dt(),
            dt_filter_after,
            dt_filter_before
        );
        defx!("(…) return {:?};", result);

        result
    }

    /// Return an up-to-date [`SummaryFixedStructReader`] instance for this
    /// `FixedStructReader`.
    ///
    /// [`SummaryFixedStructReader`]: SummaryFixedStructReader
    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryFixedStructReader {
        let fixedstructreader_fixedstructtype_opt = Some(self.fixedstruct_type());
        let fixedstructreader_fixedstructfiletype_opt = Some(self.fixedstructfiletype);
        let fixedstructreader_high_score: Score = self.high_score;
        let fixedstructreader_utmp_entries: Count = self.entries_processed;
        let fixedstructreader_first_entry_fileoffset: FileOffset = self.first_entry_fileoffset;
        let fixedstructreader_entries_out_of_order: usize = self.entries_out_of_order;
        let fixedstructreader_utmp_entries_max: Count = self.entries_stored_highest as Count;
        let fixedstructreader_utmp_entries_hit: Count = self.entries_hits as Count;
        let fixedstructreader_utmp_entries_miss: Count = self.entries_miss as Count;
        let fixedstructreader_drop_entry_ok: Count = self.drop_entry_ok;
        let fixedstructreader_drop_entry_errors: Count = self.drop_entry_errors;
        let fixedstructreader_datetime_first = self.dt_first;
        let fixedstructreader_datetime_last = self.dt_last;
        let fixedstructreader_map_tvpair_fo_max_len: usize = self.map_tvpair_fo_max_len;

        SummaryFixedStructReader {
            fixedstructreader_fixedstructtype_opt,
            fixedstructreader_fixedstructfiletype_opt,
            fixedstructreader_fixedstruct_size: self.fixedstruct_size(),
            fixedstructreader_high_score,
            fixedstructreader_utmp_entries,
            fixedstructreader_first_entry_fileoffset,
            fixedstructreader_entries_out_of_order,
            fixedstructreader_utmp_entries_max,
            fixedstructreader_utmp_entries_hit,
            fixedstructreader_utmp_entries_miss,
            fixedstructreader_drop_entry_ok,
            fixedstructreader_drop_entry_errors,
            fixedstructreader_datetime_first,
            fixedstructreader_datetime_last,
            fixedstructreader_map_tvpair_fo_max_len,
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `FixedStructReader`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let filetype = self.filetype();
        let logmessagetype = filetype_to_logmessagetype(filetype);
        let summaryblockreader = self.blockreader.summary();
        let summaryutmpreader = self.summary();
        let error: Option<String> = self.error.clone();

        Summary::new(
            path,
            filetype,
            logmessagetype,
            Some(summaryblockreader),
            None,
            None,
            None,
            Some(summaryutmpreader),
            None,
            None,
            error,
        )
    }
}
