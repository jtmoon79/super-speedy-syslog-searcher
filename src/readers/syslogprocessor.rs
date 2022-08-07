// src/readers/syslogprocessor.rs
//
// …

#![allow(non_snake_case)]

use crate::common::{
    Count,
    FPath,
    FileOffset,
    FileProcessingResult,
    FileType,
    FileSz,
    SYSLOG_SZ_MAX,
};

use crate::readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockSz,
    BlockP,
    ResultS3ReadBlock,
};

use crate::data::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeLOpt,
    Duration,
    systemtime_to_datetime,
    SystemTime,
    Year,
};

use crate::data::sysline::{
    SyslineP,
};

pub use crate::readers::linereader::{
    ResultS4LineFind,
};

pub use crate::readers::syslinereader::{
    SyslineReader,
    ResultS4SyslineFind,
    DateTimePatternCounts,
};

use crate::readers::summary::{
    Summary,
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
};

use std::collections::HashSet;
use std::fmt;
use std::io::{
    Error,
    Result,
    ErrorKind,
};

extern crate chrono;
use chrono::Datelike;

extern crate itertools;
use itertools::Itertools;  // attaches `sorted_by`

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate more_asserts;
use more_asserts::{
    assert_lt,
    debug_assert_lt,
};

extern crate rangemap;
use rangemap::RangeMap;

extern crate walkdir;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub type FileProcessingResultBlockZero = FileProcessingResult<std::io::Error>;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProcessingStage {
    /// does the file exist?
    Stage0ValidFileCheck,
    /// check file can be parsed
    Stage1BlockzeroAnalysis,
    /// find the sysline with datetime that is allowed by the datetime filters
    Stage2FindDt,
    /// no more searching backwards in a file, and thus, previously processed data can be dropped
    Stage3StreamSyslines,
    /// for CLI option --summary, print a summary about the file processing
    Stage4Summary,
}

type BszRange = std::ops::Range<BlockSz>;
type MapBszRangeToCount = RangeMap<u64, Count>;

lazy_static! {
    /// for files in blockzero_analyis, the number `Line` needed to found within
    /// block zero will vary depending on the blocksz
    pub static ref BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP: MapBszRangeToCount = {
        let mut m = MapBszRangeToCount::new();
        m.insert(BszRange{start: 0, end: SYSLOG_SZ_MAX as BlockSz}, 1);
        m.insert(BszRange{start: SYSLOG_SZ_MAX as BlockSz, end: BlockSz::MAX}, 2);

        m
    };
    /// for files in blockzero_analyis, the number `Sysline` needed to found within
    /// block zero will vary depending on the blocksz
    pub static ref BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP: MapBszRangeToCount = {
        let mut m = MapBszRangeToCount::new();
        m.insert(BszRange{start: 0, end: SYSLOG_SZ_MAX as BlockSz}, 1);
        m.insert(BszRange{start: SYSLOG_SZ_MAX as BlockSz, end: BlockSz::MAX}, 2);

        m
    };
}

/// The `SyslogProcessor` uses `SyslineReader` to find `Sysline`s in a file.
///
/// A `SyslogProcessor` has knowledge of:
/// - the different stages of processing a syslog file
/// - stores optional datetime filters
///
/// A `SyslogProcessor` will drop processed data stored by it's `SyslineReader`
/// (and underlying `LineReader` and `BlockReader`). During streaming mode,
/// the `SyslogProcessor` will proactively `drop` data that has been processed
/// and printed. It does so by calling `self.syslinereader.drop` when
pub struct SyslogProcessor {
    syslinereader: SyslineReader,
    processingstage: ProcessingStage,
    path: FPath,
    blocksz: BlockSz,
    tz_offset: FixedOffset,
    filter_dt_after_opt: DateTimeLOpt,
    filter_dt_before_opt: DateTimeLOpt,
    /// internal sanity check, has `self.blockzero_analysis()` completed?
    blockzero_analysis_done: bool,
    /// internal tracking of last `blockoffset` passed to `drop_block`
    drop_block_last: BlockOffset,
    /// internal memory of blocks dropped
    bo_dropped: HashSet<BlockOffset>,
    /// Year value used to start `process_missing_year()`
    missing_year: Option<Year>,
    /// last IO Error, if any
    Error_: Option<String>,
}

impl std::fmt::Debug for SyslogProcessor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SyslogProcessor")
            .field("Path", &self.path)
            .field("Processing Stage", &self.processingstage)
            .field("BlockSz", &self.blocksz)
            .field("TimeOffset", &self.tz_offset)
            .field("filter_dt_after_opt", &self.filter_dt_after_opt)
            .field("filter_dt_before_opt", &self.filter_dt_before_opt)
            .field("BO Analysis done?", &self.blockzero_analysis_done)
            .field("filetype", &self.filetype())
            .field("MimeGuess", &self.mimeguess())
            .field("Reprocessed missing year?", &self.did_process_missing_year())
            .field("Missing Year", &self.missing_year)
            .finish()
    }
}

impl SyslogProcessor {
    /// `SyslogProcessor` has it's own miminum requirements for `BlockSz`.
    /// Necessary for `blockzero_analysis` functions to have chance at success.
    #[cfg(any(debug_assertions,test))]
    pub const BLOCKSZ_MIN: BlockSz = 0x2;
    #[cfg(not(any(debug_assertions,test)))]
    pub const BLOCKSZ_MIN: BlockSz = 0x40;
    /// allow "streaming" (`drop`ping data in calls to `find_sysline`)?
    const STREAM_STAGE_DROP: bool = true;
    /// use LRU caches in underlying components. For development and testing experiments
    const LRU_CACHE_ENABLE: bool = true;

    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz: BlockSz,
        tz_offset: FixedOffset,
        filter_dt_after_opt: DateTimeLOpt,
        filter_dt_before_opt: DateTimeLOpt,
    ) -> Result<SyslogProcessor> {
        dpnx!("SyslogProcessor::new({:?}, {:?}, {:?}, {:?})", path, filetype, blocksz, tz_offset);
        if blocksz < SyslogProcessor::BLOCKSZ_MIN {
            return Result::Err(
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("BlockSz {0} (0x{0:08X}) is too small, SyslogProcessor has BlockSz minumum {1} (0x{1:08X})", blocksz, SyslogProcessor::BLOCKSZ_MIN)
                )
            );
        }
        let path_ = path.clone();
        let mut slr = match SyslineReader::new(path, filetype, blocksz, tz_offset) {
            Ok(val) => val,
            Err(err) => {
                return Result::Err(err);
            }
        };

        if ! SyslogProcessor::LRU_CACHE_ENABLE {
            slr.LRU_cache_disable();
            slr.linereader.LRU_cache_disable();
            slr.linereader.blockreader.LRU_cache_disable();
        }

        let bo_dropped_sz: usize = slr.blockoffset_last() as usize;

        Result::Ok(
            SyslogProcessor {
                syslinereader: slr,
                processingstage: ProcessingStage::Stage0ValidFileCheck,
                path: path_,
                blocksz,
                tz_offset,
                filter_dt_after_opt,
                filter_dt_before_opt,
                blockzero_analysis_done: false,
                drop_block_last: 0,
                bo_dropped: HashSet::<BlockOffset>::with_capacity(bo_dropped_sz),
                missing_year: None,
                Error_: None,
            }
        )
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn count_lines(&self) -> Count {
        self.syslinereader.linereader.count_lines_processed()
    }

    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.syslinereader.blocksz()
    }

    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.syslinereader.filesz()
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.syslinereader.filetype()
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub const fn path(&self) -> &FPath {
        self.syslinereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    #[allow(dead_code)]
    pub const fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.syslinereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    #[allow(dead_code)]
    pub const fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.syslinereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    #[allow(dead_code)]
    pub const fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.syslinereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    #[allow(dead_code)]
    pub const fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.syslinereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    #[allow(dead_code)]
    pub const fn count_blocks(&self) -> Count {
        self.syslinereader.count_blocks()
    }

    /// last valid `BlockOffset` of the file
    #[allow(dead_code)]
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.syslinereader.blockoffset_last()
    }

    /// get the last byte index of the file
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.syslinereader.fileoffset_last()
    }

    /// smallest size character in bytes
    #[allow(dead_code)]
    pub const fn charsz(&self) -> usize {
        self.syslinereader.charsz()
    }

    /// wrapper to `self.syslinereader.linereader.blockreader.mimeguess`
    pub const fn mimeguess(&self) -> MimeGuess {
        self.syslinereader.mimeguess()
    }

    /// did this SyslogProcessor run `process_missing_year()` ?
    fn did_process_missing_year(&self) -> bool {
        self.missing_year.is_some()
    }

    /// syslog files wherein the datetime format that does not include a year
    /// must have special handling:
    ///
    /// The last sysline in the file is presumed to share the same year as the `mtime`.
    /// The entire file is read from end to beginning (in reverse). The year is tracked
    /// and updated for each sysline. If there is jump backwards in time, that is presumed to
    /// be a year changeover.
    ///
    /// For example,
    /// TODO: fill this example
    ///
    pub fn process_missing_year(&mut self, mtime: SystemTime) -> FileProcessingResultBlockZero {
        dpnf!("syslogprocessor.process_missing_year({:?})", mtime);
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);
        debug_assert!(!self.did_process_missing_year(), "process_missing_year() must only be called once");
        let dt_mtime: DateTimeL = systemtime_to_datetime(&self.tz_offset, &mtime);
        let year: Year = dt_mtime.date().year() as Year;
        self.missing_year = Some(year);
        let mut year_opt: Option<Year> = Some(year);
        let charsz_fo: FileOffset = self.charsz() as FileOffset;
        // 25 hours
        // if there is a datetime jump backwards of more than `min_diff` then a year rollover
        // happened
        let min_diff: Duration = Duration::seconds(60 * 60 * 25);

        // The previously stored `Sysline`s have a filler year that is most likely incorrect.
        // The underlying `Sysline` instance cannot be updated behind an `Arc`.
        // Those syslines must be dropped and the entire file processed again.
        self.syslinereader.clear_syslines();

        // read all syslines in reverse
        let mut fo_prev: FileOffset = self.fileoffset_last();
        let mut syslinep_prev_opt: Option<SyslineP> = None;
        loop {
            let syslinep: SyslineP = match self.syslinereader.find_sysline_year(fo_prev, &year_opt) {
                ResultS4SyslineFind::Found((_fo, syslinep))
                => {
                    dpo!("syslogprocessor.process_missing_year Found {} Sysline @[{}, {}] datetime: {:?})", _fo, (*syslinep).fileoffset_begin(), (*syslinep).fileoffset_end(), (*syslinep).dt());
                    syslinep
                }
                ResultS4SyslineFind::Done => {
                    dpo!("syslogprocessor.process_missing_year Done, break;");
                    break;
                }
                ResultS4SyslineFind::Err(err) => {
                    self.Error_ = Some(err.to_string());
                    dpxf!("syslogprocessor.process_missing_year: return FileErrIo({:?})", err);
                    return FileProcessingResultBlockZero::FileErrIo(err);
                }
            };
            // TODO: [2022/07/27] add fn `syslinereader.find_sysline_year_rev` to hide these char offset
            //       details (put them into a struct that is meant to understand these details)
            let fo_prev_prev: FileOffset = fo_prev;
            fo_prev = (*syslinep).fileoffset_begin();
            // check if datetime has suddenly jumped backwards.
            // if date has jumped backwards, then remove sysline, update the year, and process the file
            // from that fileoffset again
            match syslinep_prev_opt {
                Some(syslinep_prev) => {
                    match (*syslinep_prev).dt() {
                        Some(dt_prev) => {
                            let dt_cur_opt: &Option<DateTimeL> = &(*syslinep).dt();
                            match dt_cur_opt {
                                Some(dt_cur) => {
                                    // normally `dt_cur` should have a datetime *before or equal* to `dt_prev`
                                    // but if not, then there was probably a year rollover
                                    if dt_cur > dt_prev {
                                        let diff: Duration = *dt_cur - *dt_prev;
                                        if diff > min_diff {
                                            year_opt = Some(year_opt.unwrap() - 1);
                                            dpo!("syslogprocessor.process_missing_year year_opt updated {:?}", year_opt);
                                            self.syslinereader.remove_sysline(fo_prev);
                                            fo_prev = fo_prev_prev;
                                            syslinep_prev_opt = Some(syslinep_prev.clone());
                                            continue;
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                        None => {}
                    }
                }
                None => {}
            }
            if fo_prev < charsz_fo {
                dpo!("syslogprocessor.process_missing_year fo_prev {} break;", fo_prev);
                // fileoffset is at the beginning of the file (or, cannot be moved back any more)
                break;
            }
            // search for preceding sysline
            fo_prev -= charsz_fo;
            // sanity check
            debug_assert_lt!(fo_prev, fo_prev_prev, "fo_prev {} ≥ {} fo_prev_prev, expected <; something is wrong", fo_prev, fo_prev_prev);
            syslinep_prev_opt = Some(syslinep.clone());
        }
        dpxf!("syslogprocessor.process_missing_year(): return FileOk");

        FileProcessingResultBlockZero::FileOk
    }

    /// wrapper to `self.syslinereader.find_sysline`
    ///
    /// This is where data is `drop`ped during streaming stage.
    //
    // TODO: [2022/06/18] store IO errors from this, for later use with `Summary` printing
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4SyslineFind {
        if self.processingstage == ProcessingStage::Stage3StreamSyslines && SyslogProcessor::STREAM_STAGE_DROP {
            dpnf!("syslogprocesser.find_sysline({})", fileoffset);
            // if processing stage is `stage3_stream_syslines`
            // then any prior processed syslines (and underlying data `Line`, `Block`, etc.)
            // can be dropped.
            let result: ResultS4SyslineFind =
                self.syslinereader.find_sysline(fileoffset);
            match result {
                ResultS4SyslineFind::Found((ref _fo, ref syslinep))
                => {
                    let bo_first: BlockOffset = (*syslinep).blockoffset_first();
                    if bo_first > 0 {
                        self.drop_block(bo_first - 1);
                    }
                }
                ResultS4SyslineFind::Done => {}
                ResultS4SyslineFind::Err(ref err) => {
                    self.Error_ = Some(err.to_string());
                }
            }
            return result;
        }
        dpnxf!("syslogprocesser.find_sysline({})", fileoffset);

        self.syslinereader.find_sysline(fileoffset)
    }

    /// wrapper to `self.syslinereader.is_sysline_last`
    pub fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        self.syslinereader.is_sysline_last(syslinep)
    }

    /// drop all data at and before `blockoffset` (drop as much as possible)
    /// this includes underyling `Block`, `LineParts`, `Line`, `Sysline`
    ///
    /// Presumes the caller knows what they are doing!
    fn drop_block(&mut self, blockoffset: BlockOffset) {
        // `drop_block_impl` is an expensive function. only run it when needed
        if blockoffset <= self.drop_block_last {
            dpnxf!("syslogprocesser.drop_block({}) skip", blockoffset);
            return;
        }
        self.drop_block_last = blockoffset;

        self.drop_block_impl(blockoffset)
    }

    fn drop_block_impl(&mut self, blockoffset: BlockOffset) {
        dpnf!("syslogprocesser.drop_block({})", blockoffset);
        debug_assert!(SyslogProcessor::STREAM_STAGE_DROP, "STREAM_STAGE_DROP is false yet call to drop_block");
        self.syslinereader.drop_block(blockoffset, &mut self.bo_dropped);
        dpxf!("syslogprocesser.drop_block({})", blockoffset);
    }

    /// Wrapper for `self.syslinereader.find_sysline_between_datetime_filters`
    //
    // TODO: [2022/06/20] the `find` functions need consistent naming,
    //       `find_next`, `find_between`, `find_...` . The current design has
    //       the public-facing `find_` functions falling back on potentail file-wide binary-search
    //       The binary-search only needs to be done during the stage 2. During stage 3, a simpler
    //       linear sequential search is more suitable, and more intuitive.
    //       More refactoring is in order.
    //       Also, a linear search can better detect rollover (i.e. when sysline datetime is missing year).
    #[inline(always)]
    pub fn find_sysline_between_datetime_filters(&mut self, fileoffset: FileOffset) -> ResultS4SyslineFind {
        dpnxf!("syslogprocesser.find_sysline_between_datetime_filters({})", fileoffset);

        match self.syslinereader.find_sysline_between_datetime_filters(
            fileoffset, &self.filter_dt_after_opt, &self.filter_dt_before_opt,
        ) {
            ResultS4SyslineFind::Err(err) => {
                self.Error_ = Some(err.to_string());

                ResultS4SyslineFind::Err(err)
            },
            val => val,
        }
    }

    /// wrapper for a recurring sanity check
    /// good for checking `process_stageX` function calls are in correct order
    #[inline(always)]
    fn assert_stage(&self, stage_expact: ProcessingStage) {
        assert_eq!(
            self.processingstage, stage_expact,
            "Unexpected Processing Stage {:?}, expected Processing Stage {:?}",
            self.processingstage, stage_expact,
        );
    }

    /// stage 0 does some sanity checks on the file
    /// XXX: this is redundant and has already been performed by functions in
    ///      `filepreprocessor` and `BlockReader::new`.
    pub fn process_stage0_valid_file_check(&mut self) -> FileProcessingResultBlockZero {
        dpnf!("syslogprocessor.process_stage0_valid_file_check");
        // sanity check calls are in correct order
        self.assert_stage(ProcessingStage::Stage0ValidFileCheck);
        self.processingstage = ProcessingStage::Stage0ValidFileCheck;

        if self.filesz() == 0 {
            dpxf!("syslogprocessor.process_stage0_valid_file_check: filesz 0; return {:?}", FileProcessingResultBlockZero::FileErrEmpty);
            return FileProcessingResultBlockZero::FileErrEmpty;
        }
        dpxf!("syslogprocessor.process_stage0_valid_file_check: return {:?}", FileProcessingResultBlockZero::FileOk);

        FileProcessingResultBlockZero::FileOk
    }

    /// stage 1: Can `Line`s and `Sysline`s be parsed from the first block (block zero)?
    pub fn process_stage1_blockzero_analysis(&mut self) -> FileProcessingResultBlockZero {
        dpnf!("syslogprocessor.process_stage1_blockzero_analysis");
        self.assert_stage(ProcessingStage::Stage0ValidFileCheck);
        self.processingstage = ProcessingStage::Stage1BlockzeroAnalysis;

        let result: FileProcessingResultBlockZero = self.blockzero_analysis();
        dpo!("syslogprocessor.process_stage1_blockzero_analysis blockzero_analysis() returned syslines {}", self.syslinereader.count_syslines_stored());
        match result {
            FileProcessingResult::FileOk => {}
            // skip further processing if not `FileOk`
            _ => {
                dpxf!("syslogprocessor.process_stage1_blockzero_analysis: return {:?}", result);
                return result;
            }
        }

        if ! self.syslinereader.dt_pattern_has_year() {
            dpo!("syslogprocessor.process_stage1_blockzero_analysis !dt_pattern_has_year()");
            let mtime: SystemTime = self.syslinereader.linereader.blockreader.mtime();
            // TODO: return any errors
            self.process_missing_year(mtime);
        }

        dpxf!("syslogprocessor.process_stage1_blockzero_analysis: return {:?}", result);

        result
    }

    /// stage 2: Given the two optional datetime filters, can a datetime be
    /// found between those filters?
    pub fn process_stage2_find_dt(&mut self) -> FileProcessingResultBlockZero {
        dpnxf!("syslogprocessor.process_stage2_find_dt");
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);
        self.processingstage = ProcessingStage::Stage2FindDt;

        FileProcessingResultBlockZero::FileOk
    }

    /// stage 3: during streaming, processed and printed data stored by underlying
    /// "Readers" is proactively dropped (removed from process memory).
    pub fn process_stage3_stream_syslines(&mut self) -> FileProcessingResultBlockZero {
        dpnxf!("syslogprocessor.process_stage3_stream_syslines");
        self.assert_stage(ProcessingStage::Stage2FindDt);
        self.processingstage = ProcessingStage::Stage3StreamSyslines;

        FileProcessingResultBlockZero::FileOk
    }

    /// stage 4: no more syslines to process, only interested in the `self.summary()`
    pub fn process_stage4_summary(&mut self) -> Summary {
        dpnxf!("syslogprocessor.process_stage4_summary");
        self.processingstage = ProcessingStage::Stage4Summary;

        self.summary()
    }

    /// Attempt to find a minimum number of `Sysline` within the first block.
    /// If enough `Sysline` found then return `FileOk` else `FileErrNoSyslinesFound`.
    pub(crate) fn blockzero_analysis_syslines(&mut self) -> FileProcessingResultBlockZero {
        dpnf!("syslogprocessor.blockzero_analysis_syslines");
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        let blockp: BlockP = match self.syslinereader.linereader.blockreader.read_block(0) {
            ResultS3ReadBlock::Found(blockp_) => blockp_,
            ResultS3ReadBlock::Done => {
                dpxf!("syslogprocessor.blockzero_analysis_syslines: return FileErrEmpty");
                return FileProcessingResultBlockZero::FileErrEmpty;
            },
            ResultS3ReadBlock::Err(err) => {
                dpxf!("syslogprocessor.blockzero_analysis_syslines: return FileErrIo({:?})", err);
                self.Error_ = Some(err.to_string());
                return FileProcessingResultBlockZero::FileErrIo(err);
            },
        };
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let mut fo: FileOffset = 0;
        // how many syslines have been found?
        let mut found: Count = 0;
        // must find at least this many syslines in block zero to be FileOk
        let found_min: Count = *BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP.get(&blocksz0).unwrap();
        dpo!("syslogprocessor.blockzero_analysis_syslines: block zero blocksz {} found_min {:?}", blocksz0, found_min);
        // find `at_max` Syslines within block zero
        while found < found_min {
            fo = match self.syslinereader.find_sysline_in_block(fo) {
                ResultS4SyslineFind::Found((fo_next, _slinep)) => {
                    found += 1;

                    fo_next
                }
                ResultS4SyslineFind::Done => {
                    //found += 1;
                    break;
                }
                ResultS4SyslineFind::Err(err) => {
                    self.Error_ = Some(err.to_string());
                    dpxf!("syslogprocessor.blockzero_analysis_syslines: return FileErrIo({:?})", err);
                    return FileProcessingResultBlockZero::FileErrIo(err);
                }
            };
            if 0 != self.syslinereader.block_offset_at_file_offset(fo) {
                break;
            }
        }

        let fpr: FileProcessingResultBlockZero = match found >= found_min {
            true => FileProcessingResultBlockZero::FileOk,
            false => FileProcessingResultBlockZero::FileErrNoSyslinesFound,
        };

        dpxf!("syslogprocessor.blockzero_analysis_syslines() found {} syslines, require {} syslines, return {:?}", found, found_min, fpr);

        fpr
    }

    /// Attempt to find a minimum number of `Line`s within the first block (block zero).
    /// If enough `Line` found then return `FileOk` else `FileErrNoLinesFound`.
    #[inline(always)]
    pub(crate) fn blockzero_analysis_lines(&mut self) -> FileProcessingResultBlockZero {
        dpnf!("syslogprocessor.blockzero_analysis_lines()");
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        let blockp: BlockP = match self.syslinereader.linereader.blockreader.read_block(0) {
            ResultS3ReadBlock::Found(blockp_) => blockp_,
            ResultS3ReadBlock::Done => {
                dpxf!("syslogprocessor.blockzero_analysis_lines: return FileErrEmpty");
                return FileProcessingResultBlockZero::FileErrEmpty;
            },
            ResultS3ReadBlock::Err(err) => {
                self.Error_ = Some(err.to_string());
                dpxf!("syslogprocessor.blockzero_analysis_lines: return FileErrIo({:?})", err);
                return FileProcessingResultBlockZero::FileErrIo(err);
            },
        };
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let mut fo: FileOffset = 0;
        // how many lines have been found?
        let mut found: Count = 0;
        // must find at least this many lines in block zero to be FileOk
        let found_min: Count = *BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP.get(&blocksz0).unwrap();
        dpxf!("syslogprocessor.blockzero_analysis_lines: block zero blocksz {} found_min {}", blocksz0, found_min);
        // find `found_min` Lines or whatever can be found within block 0
        while found < found_min {
            fo = match self.syslinereader.linereader.find_line_in_block(fo) {
                ResultS4LineFind::Found((fo_next, _linep)) => {
                    found += 1;

                    fo_next
                },
                ResultS4LineFind::Done => {
                    found += 1;
                    break;
                },
                ResultS4LineFind::Err(err) => {
                    self.Error_ = Some(err.to_string());
                    dpxf!("syslogprocessor.blockzero_analysis_lines: return FileErrIo({:?})", err);
                    return FileProcessingResultBlockZero::FileErrIo(err);
                },
            };
            if 0 != self.syslinereader.linereader.block_offset_at_file_offset(fo) {
                break;
            }
        }

        let fpr: FileProcessingResultBlockZero = match found >= found_min {
            true => FileProcessingResultBlockZero::FileOk,
            false => FileProcessingResultBlockZero::FileErrNoSyslinesFound,
        };

        dpxf!("syslogprocessor.blockzero_analysis_lines: found {} lines, require {} lines, return {:?}", found, found_min, fpr);

        fpr
    }

    /// Call `self.blockzero_analysis_lines`.
    /// If that passes then call `self.blockzero_analysis_syslines`.
    pub fn blockzero_analysis(&mut self) -> FileProcessingResultBlockZero {
        dpnf!("syslogprocessor.blockzero_analysis");
        assert!(!self.blockzero_analysis_done, "blockzero_analysis_lines should only be completed once.");
        self.blockzero_analysis_done = true;
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        let result: FileProcessingResultBlockZero = self.blockzero_analysis_lines();
        if ! result.is_ok() {
            dpxf!("syslogprocessor.blockzero_analysis: syslinereader.blockzero_analysis() was !is_ok(), return {:?}", result);
            return result;
        };

        let result: FileProcessingResultBlockZero = self.blockzero_analysis_syslines();
        dpxf!("syslogprocessor.blockzero_analysis() return {:?}", result);

        result
    }

    /// return an up-to-date `Summary` instance for this `SyslogProcessor`
    pub fn summary(&self) -> Summary {
        let filetype = self.filetype();
        let BlockReader_bytes = self.syslinereader.linereader.blockreader.count_bytes();
        let BlockReader_bytes_total = self.filesz() as FileSz;
        let BlockReader_blocks = self.syslinereader.linereader.blockreader.count_blocks_processed();
        let BlockReader_blocks_total = self.syslinereader.linereader.blockreader.blockn;
        let BlockReader_blocksz = self.blocksz();
        let BlockReader_filesz = self.syslinereader.linereader.blockreader.filesz;
        let BlockReader_filesz_actual = self.syslinereader.linereader.blockreader.filesz_actual;
        let LineReader_lines = self.syslinereader.linereader.count_lines_processed();
        let SyslineReader_syslines = self.syslinereader.count_syslines_processed();
        let SyslineReader_syslines_hit = self.syslinereader.syslines_hit;
        let SyslineReader_syslines_miss = self.syslinereader.syslines_miss;
        let SyslineReader_syslines_by_range_hit = self.syslinereader.syslines_by_range_hit;
        let SyslineReader_syslines_by_range_miss = self.syslinereader.syslines_by_range_miss;
        let SyslineReader_syslines_by_range_put = self.syslinereader.syslines_by_range_put;
        // only print patterns with use count > 0, sorted by count
        let mut SyslineReader_patterns_ = DateTimePatternCounts::new();
        SyslineReader_patterns_.extend(
            self.syslinereader.dt_patterns_counts.iter().filter(|&(_k, v)| v > &mut 0)
        );
        let mut SyslineReader_patterns = DateTimePatternCounts::new();
        SyslineReader_patterns.extend(SyslineReader_patterns_.into_iter().sorted_by(|a, b| Ord::cmp(&b.1, &a.1)));
        let SyslineReader_datetime_first = self.syslinereader.dt_first;
        let SyslineReader_datetime_last = self.syslinereader.dt_last;
        let SyslineReader_find_sysline_lru_cache_hit = self.syslinereader.find_sysline_lru_cache_hit;
        let SyslineReader_find_sysline_lru_cache_miss = self.syslinereader.find_sysline_lru_cache_miss;
        let SyslineReader_find_sysline_lru_cache_put = self.syslinereader.find_sysline_lru_cache_put;
        let SyslineReader_parse_datetime_in_line_lru_cache_hit = self.syslinereader.parse_datetime_in_line_lru_cache_hit;
        let SyslineReader_parse_datetime_in_line_lru_cache_miss = self.syslinereader.parse_datetime_in_line_lru_cache_miss;
        let SyslineReader_parse_datetime_in_line_lru_cache_put = self.syslinereader.parse_datetime_in_line_lru_cache_put;
        let SyslineReader_get_boxptrs_singleptr = self.syslinereader.get_boxptrs_singleptr;
        let SyslineReader_get_boxptrs_doubleptr = self.syslinereader.get_boxptrs_doubleptr;
        let SyslineReader_get_boxptrs_multiptr = self.syslinereader.get_boxptrs_multiptr;
        let LineReader_lines_hit = self.syslinereader.linereader._lines_hits;
        let LineReader_lines_miss = self.syslinereader.linereader._lines_miss;
        let LineReader_find_line_lru_cache_hit = self.syslinereader.linereader.find_line_lru_cache_hit;
        let LineReader_find_line_lru_cache_miss = self.syslinereader.linereader.find_line_lru_cache_miss;
        let LineReader_find_line_lru_cache_put = self.syslinereader.linereader.find_line_lru_cache_put;
        let BlockReader_read_block_lru_cache_hit = self.syslinereader.linereader.blockreader.read_block_cache_lru_hit;
        let BlockReader_read_block_lru_cache_miss = self.syslinereader.linereader.blockreader.read_block_cache_lru_miss;
        let BlockReader_read_block_lru_cache_put = self.syslinereader.linereader.blockreader.read_block_cache_lru_put;
        let BlockReader_read_blocks_hit = self.syslinereader.linereader.blockreader.read_blocks_hit;
        let BlockReader_read_blocks_miss = self.syslinereader.linereader.blockreader.read_blocks_miss;
        let BlockReader_read_blocks_put = self.syslinereader.linereader.blockreader.read_blocks_put;
        let LineReader_drop_line_ok = self.syslinereader.linereader.drop_line_ok;
        let LineReader_drop_line_errors = self.syslinereader.linereader.drop_line_errors;
        let SyslineReader_drop_sysline_ok = self.syslinereader.drop_sysline_ok;
        let SyslineReader_drop_sysline_errors = self.syslinereader.drop_sysline_errors;
        let SyslogProcessor_missing_year = self.missing_year;
        let Error_: Option<String> = self.Error_.clone();

        Summary::new(
            filetype,
            BlockReader_bytes,
            BlockReader_bytes_total,
            BlockReader_blocks,
            BlockReader_blocks_total,
            BlockReader_blocksz,
            BlockReader_filesz,
            BlockReader_filesz_actual,
            LineReader_lines,
            SyslineReader_syslines,
            SyslineReader_syslines_hit,
            SyslineReader_syslines_miss,
            SyslineReader_syslines_by_range_hit,
            SyslineReader_syslines_by_range_miss,
            SyslineReader_syslines_by_range_put,
            SyslineReader_patterns,
            SyslineReader_datetime_first,
            SyslineReader_datetime_last,
            SyslineReader_find_sysline_lru_cache_hit,
            SyslineReader_find_sysline_lru_cache_miss,
            SyslineReader_find_sysline_lru_cache_put,
            SyslineReader_parse_datetime_in_line_lru_cache_hit,
            SyslineReader_parse_datetime_in_line_lru_cache_miss,
            SyslineReader_parse_datetime_in_line_lru_cache_put,
            SyslineReader_get_boxptrs_singleptr,
            SyslineReader_get_boxptrs_doubleptr,
            SyslineReader_get_boxptrs_multiptr,
            LineReader_lines_hit,
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
            LineReader_drop_line_ok,
            LineReader_drop_line_errors,
            SyslineReader_drop_sysline_ok,
            SyslineReader_drop_sysline_errors,
            SyslogProcessor_missing_year,
            Error_,
        )
    }
}
