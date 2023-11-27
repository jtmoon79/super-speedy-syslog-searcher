// src/readers/syslogprocessor.rs
// …

//! Implements a [`SyslogProcessor`], the driver of the processing stages for
//! a "syslog" file using a [`SyslineReader`].
//!
//! A "syslog" file in this context means any text-based file with logged
//! messages with a datetime stamp.
//! The file may use a formally defined log message format (e.g. RFC 5424)
//! or an ad-hoc log message format (most log files).<br/>
//! The two common assumptions are that:
//! 1. each log message has a datetime stamp on the first line
//! 2. log messages are in chronological order
//!
//! Sibling of [`UtmpxReader`]. But far more complicated due to the
//! ad-hoc nature of log files.
//! 
//! This is an _s4lib_ structure used by the binary program _s4_.
//!
//! [`UtmpxReader`]: crate::readers::utmpxreader::UtmpxReader
//! [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
//! [`SyslogProcessor`]: SyslogProcessor

#![allow(non_snake_case)]

use crate::common::{
    Count,
    FPath,
    FileOffset,
    FileProcessingResult,
    FileSz,
    FileType,
    SYSLOG_SZ_MAX,
    filetype_to_logmessagetype,
};
use crate::data::datetime::{
    dt_after_or_before,
    systemtime_to_datetime,
    DateTimeL,
    DateTimeLOpt,
    Duration,
    FixedOffset,
    Result_Filter_DateTime1,
    SystemTime,
    Year,
};
use crate::data::sysline::SyslineP;
use crate::{e_err, de_err, de_wrn};
use crate::readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockP,
    BlockSz,
    ResultS3ReadBlock,
};
#[cfg(test)]
use crate::readers::blockreader::SetDroppedBlocks;
#[cfg(test)]
use crate::readers::linereader::SetDroppedLines;
#[cfg(test)]
use crate::readers::syslinereader::SetDroppedSyslines;
#[doc(hidden)]
pub use crate::readers::linereader::ResultS3LineFind;
#[doc(hidden)]
pub use crate::readers::syslinereader::{
    DateTimePatternCounts,
    ResultS3SyslineFind,
    SummarySyslineReader,
    SyslineReader,
};
use crate::readers::summary::Summary;

use std::fmt;
use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result};

use ::chrono::Datelike;
use ::lazy_static::lazy_static;
use ::mime_guess::MimeGuess;
use ::rangemap::RangeMap;
use ::si_trace_print::{def1n, def1x, def1ñ, defn, defo, defx, defñ};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// `SYSLOG_SZ_MAX` as a `BlockSz`.
pub(crate) const SYSLOG_SZ_MAX_BSZ: BlockSz = SYSLOG_SZ_MAX as BlockSz;

/// Typed [`FileProcessingResult`] for "block zero analysis".
///
/// [`FileProcessingResult`]: crate::common::FileProcessingResult
pub type FileProcessingResultBlockZero = FileProcessingResult<std::io::Error>;

/// Enum for the [`SyslogProcessor`] processing stages. Each file processed
/// advances through these stages. Sometimes stages may be skipped.
///
/// [`SyslogProcessor`]: self::SyslogProcessor
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProcessingStage {
    /// Does the file exist and is it a parseable type?
    Stage0ValidFileCheck,
    /// Check file can be parsed by trying to parse it. Determine the
    /// datetime patterns of any found [`Sysline`s].<br/>
    /// If no `Sysline`s are found then advance to `Stage4Summary`.
    ///
    /// [`Sysline`s]: crate::data::sysline::Sysline
    Stage1BlockzeroAnalysis,
    /// Find the first [`Sysline`] in the syslog file.<br/>
    /// If passed CLI option `--after` then find the first `Sysline` with
    /// datetime at or after the user-passed [`DateTimeL`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    Stage2FindDt,
    /// Advanced through the syslog file to the end.<br/>
    /// If passed CLI option `--before` then find the last [`Sysline`] with
    /// datetime at or before the user-passed [`DateTimeL`].
    ///
    /// While advancing, try to `drop` previously processed data `Block`s,
    /// `Line`s, and `Sysline`s to lessen memory allocated.
    /// a.k.a. "_streaming stage_".
    /// Also see function [`find_sysline`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    /// [`find_sysline`]: self::SyslogProcessor#method.find_sysline
    Stage3StreamSyslines,
    /// If passed CLI option `--summary` then print a summary of
    /// various information about the processed file.
    Stage4Summary,
}

/// [`BlockSz`] in a [`Range`].
///
/// [`Range`]: std::ops::Range
/// [`BlockSz`]: crate::readers::blockreader::BlockSz
type BszRange = std::ops::Range<BlockSz>;

/// Map [`BlockSz`] to a [`Count`].
///
/// [`BlockSz`]: crate::readers::blockreader::BlockSz
/// [`Count`]: crate::common::Count
type MapBszRangeToCount = RangeMap<u64, Count>;

lazy_static! {
    /// For files in `blockzero_analyis`, the number of [`Line`]s needed to
    /// be found within block zero.
    ///
    /// [`Line`]: crate::data::line::Line
    pub static ref BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP: MapBszRangeToCount = {
        let mut m = MapBszRangeToCount::new();
        m.insert(BszRange{start: 0, end: SYSLOG_SZ_MAX_BSZ}, 1);
        m.insert(BszRange{start: SYSLOG_SZ_MAX_BSZ, end: SYSLOG_SZ_MAX_BSZ * 3}, 3);
        m.insert(BszRange{start: SYSLOG_SZ_MAX_BSZ * 3, end: BlockSz::MAX}, 3);

        m
    };

    /// For files in `blockzero_analyis`, the number of [`Sysline`]s needed to
    /// be found within block zero.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub static ref BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP: MapBszRangeToCount = {
        let mut m = MapBszRangeToCount::new();
        m.insert(BszRange{start: 0, end: SYSLOG_SZ_MAX_BSZ}, 1);
        m.insert(BszRange{start: SYSLOG_SZ_MAX_BSZ, end: BlockSz::MAX}, 2);

        m
    };

    /// 25 hours.
    /// For processing syslog files without a year.
    /// If there is a datetime jump backwards more than this value then
    /// a year rollover happened.
    ///
    /// e.g. given log messages
    ///     Dec 31 23:59:59 [INFO] One!
    ///     Jan 1 00:00:00 [INFO] Happy New Year!!!
    /// These messages interpreted as the same year would be a jump backwards
    /// in time.
    /// Of course, this apparent "jump backwards" means the year changed.
    // XXX: cannot make `const` because `secs` is a private field
    static ref BACKWARDS_TIME_JUMP_MEANS_NEW_YEAR: Duration = Duration::seconds(60 * 60 * 25);
}

/// The `SyslogProcessor` uses [`SyslineReader`] to find [`Sysline`s] in a file.
///
/// A `SyslogProcessor` has knowledge of:
/// - the different stages of processing a syslog file
/// - stores optional datetime filters and searches with them
/// - handles special cases of a syslog file with a datetime format without a
///   year
///
/// A `SyslogProcessor` is driven by a thread to fully process one syslog file.
///
/// During "[streaming stage]", the `SyslogProcessor` will proactively `drop`
/// data that has been processed and printed. It does so by calling
/// private function `drop_block` during function [`find_sysline`].
///
/// [`Sysline`s]: crate::data::sysline::Sysline
/// [`SyslineReader`]: crate::readers::syslinereader::SyslineReader
/// [`LineReader`]: crate::readers::linereader::LineReader
/// [`BlockReader`]: crate::readers::blockreader::BlockReader
/// [`find_sysline`]: self::SyslogProcessor#method.find_sysline
/// [streaming stage]: self::ProcessingStage#variant.Stage3StreamSyslines
pub struct SyslogProcessor {
    syslinereader: SyslineReader,
    /// Current `ProcessingStage`.
    processingstage: ProcessingStage,
    /// `FPath`.
    // TODO: remove this, use the `BlockReader` path, (DRY)
    path: FPath,
    // TODO: remove this, use the `BlockReader` blocksz, (DRY)
    blocksz: BlockSz,
    /// `FixedOffset` timezone for datetime formats without a timezone.
    tz_offset: FixedOffset,
    /// Optional filter, syslines _after_ this `DateTimeL`.
    filter_dt_after_opt: DateTimeLOpt,
    /// Optional filter, syslines _before_ this `DateTimeL`.
    filter_dt_before_opt: DateTimeLOpt,
    /// Internal sanity check, has `self.blockzero_analysis()` completed?
    blockzero_analysis_done: bool,
    /// Internal tracking of last `blockoffset` passed to `drop_block`.
    drop_block_last: BlockOffset,
    /// Optional `Year` value used to start `process_missing_year()`.
    /// Only needed for syslog files with datetime format without a year.
    missing_year: Option<Year>,
    /// The last [`Error`], if any, as a `String`. Set by [`set_error`].
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    /// [`set_error`]: self::SyslogProcessor#method.set_error
    error: Option<String>,
}

impl Debug for SyslogProcessor {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
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
            .field("Error?", &self.error)
            .finish()
    }
}

// TODO: [2023/04] remove redundant variable prefix name `syslogprocessor_`
#[derive(Clone, Default)]
pub struct SummarySyslogProcessor {
    /// `SyslogProcessor::missing_year`
    pub SyslogProcessor_missing_year: Option<Year>,
}

impl SyslogProcessor {
    /// `SyslogProcessor` has it's own miminum requirements for `BlockSz`.
    ///
    /// Necessary for `blockzero_analysis` functions to have chance at success.
    #[doc(hidden)]
    #[cfg(any(debug_assertions, test))]
    pub const BLOCKSZ_MIN: BlockSz = 0x2;

    /// Maximum number of datetime patterns for matching the remainder of a syslog file.
    const DT_PATTERN_MAX: usize = SyslineReader::DT_PATTERN_MAX;

    /// `SyslogProcessor` has it's own miminum requirements for `BlockSz`.
    ///
    /// Necessary for `blockzero_analysis` functions to have chance at success.
    #[cfg(not(any(debug_assertions, test)))]
    pub const BLOCKSZ_MIN: BlockSz = 0x40;

    /// Minimum number of bytes needed to perform `blockzero_analysis_bytes`.
    ///
    /// Pretty sure this is smaller than the smallest possible timestamp that
    /// can be processed by the `DTPD!` in `DATETIME_PARSE_DATAS`.
    /// In other words, a file that only has a datetimestamp followed by an
    /// empty log message.
    ///
    /// It's okay if this is too small as the later processing stages will
    /// be certain of any possible datetime patterns.
    pub const BLOCKZERO_ANALYSIS_BYTES_MIN: BlockSz = 6;

    /// If the first number of bytes are zero bytes (NULL bytes) then
    /// stop processing the file. It's extremely unlikely this is a syslog
    /// file and more likely it's some sort of binary data file.
    pub const BLOCKZERO_ANALYSIS_BYTES_NULL_MAX: usize = 128;

    /// Allow "streaming stage" to drop data?
    /// Compile-time "option" to aid manual debugging.
    #[doc(hidden)]
    const STREAM_STAGE_DROP: bool = true;

    /// Use LRU caches in underlying components?
    ///
    /// XXX: For development and testing experiments!
    #[doc(hidden)]
    const LRU_CACHE_ENABLE: bool = true;

    /// Create a new `SyslogProcessor`.
    // NOTE: should not attempt any block reads here, similar to other `*Readers`
    pub fn new(
        path: FPath,
        filetype: FileType,
        blocksz: BlockSz,
        tz_offset: FixedOffset,
        filter_dt_after_opt: DateTimeLOpt,
        filter_dt_before_opt: DateTimeLOpt,
    ) -> Result<SyslogProcessor> {
        def1n!("({:?}, {:?}, {:?}, {:?})", path, filetype, blocksz, tz_offset);
        if blocksz < SyslogProcessor::BLOCKSZ_MIN {
            return Result::Err(
                Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "BlockSz {0} (0x{0:08X}) is too small, SyslogProcessor has BlockSz minimum {1} (0x{1:08X}) file {2:?}",
                        blocksz, SyslogProcessor::BLOCKSZ_MIN, &path,
                    )
                )
            );
        }
        let path_ = path.clone();
        let mut slr = match SyslineReader::new(path, filetype, blocksz, tz_offset) {
            Ok(val) => val,
            Err(err) => {
                def1x!();
                return Result::Err(err);
            }
        };

        if !SyslogProcessor::LRU_CACHE_ENABLE {
            slr.LRU_cache_disable();
            slr.linereader
                .LRU_cache_disable();
            slr.linereader
                .blockreader
                .LRU_cache_disable();
        }

        def1x!("return Ok(SyslogProcessor)");

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
                missing_year: None,
                error: None,
            }
        )
    }

    /// `Count` of [`Line`s] processed.
    ///
    /// [`Line`s]: crate::data::line::Line
    #[inline(always)]
    #[allow(dead_code)]
    pub fn count_lines(&self) -> Count {
        self.syslinereader
            .linereader
            .count_lines_processed()
    }

    /// See [`Sysline::count_syslines_stored`].
    ///
    /// [`Sysline::count_syslines_stored`]: crate::data::sysline::Sysline::count_syslines_stored
    #[cfg(test)]
    pub fn count_syslines_stored(&self) -> Count {
        self.syslinereader.count_syslines_stored()
    }

    /// See [`BlockReader::blocksz`].
    ///
    /// [`BlockReader::blocksz`]: crate::readers::blockreader::BlockReader#method.blocksz
    #[inline(always)]
    pub const fn blocksz(&self) -> BlockSz {
        self.syslinereader.blocksz()
    }

    /// See [`BlockReader::filesz`].
    ///
    /// [`BlockReader::filesz`]: crate::readers::blockreader::BlockReader#method.filesz
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.syslinereader.filesz()
    }

    /// See [`BlockReader::filetype`].
    ///
    /// [`BlockReader::filetype`]: crate::readers::blockreader::BlockReader#method.filetype
    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        self.syslinereader.filetype()
    }

    /// See [`BlockReader::path`].
    ///
    /// [`BlockReader::path`]: crate::readers::blockreader::BlockReader#method.path
    #[inline(always)]
    #[allow(dead_code)]
    pub const fn path(&self) -> &FPath {
        self.syslinereader.path()
    }

    /// See [`BlockReader::block_offset_at_file_offset`].
    ///
    /// [`BlockReader::block_offset_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_offset_at_file_offset
    #[allow(dead_code)]
    pub const fn block_offset_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockOffset {
        self.syslinereader
            .block_offset_at_file_offset(fileoffset)
    }

    /// See [`BlockReader::file_offset_at_block_offset`].
    ///
    /// [`BlockReader::file_offset_at_block_offset`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset
    #[allow(dead_code)]
    pub const fn file_offset_at_block_offset(
        &self,
        blockoffset: BlockOffset,
    ) -> FileOffset {
        self.syslinereader
            .file_offset_at_block_offset(blockoffset)
    }

    /// See [`BlockReader::file_offset_at_block_offset_index`].
    ///
    /// [`BlockReader::file_offset_at_block_offset_index`]: crate::readers::blockreader::BlockReader#method.file_offset_at_block_offset_index
    #[allow(dead_code)]
    pub const fn file_offset_at_block_offset_index(
        &self,
        blockoffset: BlockOffset,
        blockindex: BlockIndex,
    ) -> FileOffset {
        self.syslinereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// See [`BlockReader::block_index_at_file_offset`].
    ///
    /// [`BlockReader::block_index_at_file_offset`]: crate::readers::blockreader::BlockReader#method.block_index_at_file_offset
    #[allow(dead_code)]
    pub const fn block_index_at_file_offset(
        &self,
        fileoffset: FileOffset,
    ) -> BlockIndex {
        self.syslinereader
            .block_index_at_file_offset(fileoffset)
    }

    /// See [`BlockReader::count_blocks`].
    ///
    /// [`BlockReader::count_blocks`]: crate::readers::blockreader::BlockReader#method.count_blocks
    #[allow(dead_code)]
    pub const fn count_blocks(&self) -> Count {
        self.syslinereader
            .count_blocks()
    }

    /// See [`BlockReader::blockoffset_last`].
    ///
    /// [`BlockReader::blockoffset_last`]: crate::readers::blockreader::BlockReader#method.blockoffset_last
    #[allow(dead_code)]
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.syslinereader
            .blockoffset_last()
    }

    /// See [`BlockReader::fileoffset_last`].
    ///
    /// [`BlockReader::fileoffset_last`]: crate::readers::blockreader::BlockReader#method.fileoffset_last
    pub const fn fileoffset_last(&self) -> FileOffset {
        self.syslinereader
            .fileoffset_last()
    }

    /// See [`LineReader::charsz`].
    ///
    /// [`LineReader::charsz`]: crate::readers::linereader::LineReader#method.charsz
    #[allow(dead_code)]
    pub const fn charsz(&self) -> usize {
        self.syslinereader.charsz()
    }

    /// See [`BlockReader::mimeguess`].
    ///
    /// [`BlockReader::mimeguess`]: crate::readers::blockreader::BlockReader#method.mimeguess
    pub const fn mimeguess(&self) -> MimeGuess {
        self.syslinereader.mimeguess()
    }

    /// See [`BlockReader::mtime`].
    ///
    /// [`BlockReader::mtime`]: crate::readers::blockreader::BlockReader#method.mtime
    pub fn mtime(&self) -> SystemTime {
        self.syslinereader.mtime()
    }

    /// Did this `SyslogProcessor` run `process_missing_year()` ?
    fn did_process_missing_year(&self) -> bool {
        self.missing_year.is_some()
    }

    /// store an `Error` that occurred. For later printing during `--summary`.
    // XXX: duplicates `UtmpxReader.set_error`
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
            de_wrn!("skip overwrite of previous Error {:?} with Error ({:?})", _err, error);
            return;
        }
        self.error = Some(error_string);
    }

    /// Syslog files wherein the datetime format that does not include a year
    /// must have special handling.
    ///
    /// The last [`Sysline`] in the file is presumed to share the same year as
    /// the `mtime` (stored by the underlying [`BlockReader`] instance).
    /// The entire file is read from end to beginning (in reverse) (unless
    /// a `filter_dt_after_opt` is passed that coincides with the found
    /// syslines). The year is tracked and updated for each sysline.
    /// If there is jump backwards in time, that is presumed to be a
    /// year changeover.
    ///
    /// For example, given syslog contents
    ///
    /// ```text
    /// Nov 1 12:00:00 hello
    /// Dec 1 12:00:00 good morning
    /// Jan 1 12:00:00 goodbye
    /// ```
    ///
    /// and file `mtime` that is datetime _January 1 12:00:00 2015_,
    /// then the last `Sysline` "Jan 1 12:00:00 goodbye" is presumed to be in
    /// year 2015.
    /// The preceding `Sysline` "Dec 1 12:00:00 goodbye" is then processed.
    /// An apparent backwards jump is seen _Jan 1_ to _Dec 1_.
    /// From this, it can be concluded the _Dec 1_ refers to a prior year, 2014.
    ///
    /// Typically, when a datetime filter is passed, a special binary search is
    /// done to find the desired syslog line, reducing resource usage. Whereas,
    /// files processed here must be read linearly and in their entirety
    /// Or, if `filter_dt_after_opt` is passed then the file is read to the
    /// first `sysline.dt()` (datetime) that is
    /// `Result_Filter_DateTime1::OccursBefore` the
    /// `filter_dt_after_opt`.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`BlockReader`]: crate::readers::blockreader::BlockReader
    /// [`DateTimeL`]: crate::data::datetime::DateTimeL
    pub fn process_missing_year(
        &mut self,
        mtime: SystemTime,
        filter_dt_after_opt: &DateTimeLOpt,
    ) -> FileProcessingResultBlockZero {
        defn!("({:?}, {:?})", mtime, filter_dt_after_opt);
        debug_assert!(!self.did_process_missing_year(), "process_missing_year() must only be called once");
        let dt_mtime: DateTimeL = systemtime_to_datetime(&self.tz_offset, &mtime);
        let year: Year = dt_mtime.date_naive().year() as Year;
        self.missing_year = Some(year);
        let mut year_opt: Option<Year> = Some(year);
        let charsz_fo: FileOffset = self.charsz() as FileOffset;

        // The previously stored `Sysline`s have a filler year that is most likely incorrect.
        // The underlying `Sysline` instance cannot be updated behind an `Arc`.
        // Those syslines must be dropped and the entire file processed again.
        // However, underlying `Line` and `Block` are still valid; do not reprocess those.
        self.syslinereader
            .clear_syslines();

        // read all syslines in reverse
        let mut fo_prev: FileOffset = self.fileoffset_last();
        let mut syslinep_prev_opt: Option<SyslineP> = None;
        loop {
            let syslinep: SyslineP = match self
                .syslinereader
                .find_sysline_year(fo_prev, &year_opt)
            {
                ResultS3SyslineFind::Found((_fo, syslinep)) => {
                    defo!(
                        "Found {} Sysline @[{}, {}] datetime: {:?})",
                        _fo,
                        (*syslinep).fileoffset_begin(),
                        (*syslinep).fileoffset_end(),
                        (*syslinep).dt()
                    );
                    syslinep
                }
                ResultS3SyslineFind::Done => {
                    defo!("Done, break;");
                    break;
                }
                ResultS3SyslineFind::Err(err) => {
                    self.set_error(&err);
                    defx!("return FileErrIo({:?})", err);
                    return FileProcessingResultBlockZero::FileErrIoPath(err);
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
                    // normally `dt_cur` should have a datetime *before or equal* to `dt_prev`
                    // but if not, then there was probably a year rollover
                    if (*syslinep).dt() > (*syslinep_prev).dt() {
                        let diff: Duration = *(*syslinep).dt() - *(*syslinep_prev).dt();
                        if diff > *BACKWARDS_TIME_JUMP_MEANS_NEW_YEAR {
                            year_opt = Some(year_opt.unwrap() - 1);
                            defo!("year_opt updated {:?}", year_opt);
                            self.syslinereader
                                .remove_sysline(fo_prev);
                            fo_prev = fo_prev_prev;
                            syslinep_prev_opt = Some(syslinep_prev.clone());
                            continue;
                        }
                    }
                }
                None => {}
            }
            if fo_prev < charsz_fo {
                defo!("fo_prev {} break;", fo_prev);
                // fileoffset is at the beginning of the file (or, cannot be moved back any more)
                break;
            }
            // if user-passed `--dt-after` and the sysline is prior to that filter then
            // stop processing
            match dt_after_or_before(syslinep.dt(), filter_dt_after_opt) {
                Result_Filter_DateTime1::OccursBefore => {
                    defo!("dt_after_or_before({:?},  {:?}) returned OccursBefore; break", syslinep.dt(), filter_dt_after_opt);
                    break;
                }
                Result_Filter_DateTime1::OccursAtOrAfter | Result_Filter_DateTime1::Pass => {},
            }
            // search for preceding sysline
            fo_prev -= charsz_fo;
            if fo_prev >= fo_prev_prev {
                // This will happen in case where the very first line of the file
                // holds a sysline with datetime pattern without a year, and that
                // sysline datetime pattern is different than all
                // proceeding syslines that have a year. (and it should only happen then)
                // Elicited by example in Issue #74
                de_err!("fo_prev {} ≥ {} fo_prev_prev, expected <; something is wrong", fo_prev, fo_prev_prev);
                // must break otherwise end up in an infinite loop
                break;
            }
            syslinep_prev_opt = Some(syslinep.clone());
        } // end loop
        defx!("return FileOk");

        FileProcessingResultBlockZero::FileOk
    }

    /// See [`SyslineReader::is_sysline_last`].
    ///
    /// [`SyslineReader::is_sysline_last`]: crate::readers::syslinereader::SyslineReader#method.is_sysline_last
    pub fn is_sysline_last(
        &self,
        syslinep: &SyslineP,
    ) -> bool {
        self.syslinereader
            .is_sysline_last(syslinep)
    }

    /// Try to `drop` data associated with the [`Block`] at [`BlockOffset`].
    /// This includes dropping associated [`Sysline`]s and [`Line`]s.
    ///
    /// Caller must know what they are doing!
    ///
    /// [`BlockOffset`]: crate::common::BlockOffset
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`Line`]: crate::data::line::Line
    /// [`Block`]: crate::readers:blockreader::Block
    fn drop_data(
        &mut self,
        blockoffset: BlockOffset,
    ) -> bool {
        def1n!("({})", blockoffset);
        self.assert_stage(ProcessingStage::Stage3StreamSyslines);

        // `syslinereader.drop_data` is an expensive function, skip if possible.
        if blockoffset == self.drop_block_last {
            def1x!("({}) skip block, return true", blockoffset);
            return false;
        }

        if self
            .syslinereader
            .drop_data(blockoffset)
        {
            self.drop_block_last = blockoffset;
            def1x!("({}) return true", blockoffset);
            return true;
        }

        def1x!("({}) return false", blockoffset);
        false
    }

    /// Call [`drop_data`] for the [`Block`] *preceding* the first block of the
    /// passed [`Sysline`].
    ///
    /// [`drop_data`]: Self#method.drop_data
    /// [`Block`]: crate::readers::blockreader::Block
    /// [`Sysline`]: crate::data::sysline::Sysline
    pub fn drop_data_try(
        &mut self,
        syslinep: &SyslineP,
    ) -> bool {
        if !SyslogProcessor::STREAM_STAGE_DROP {
            return false;
        }
        let bo_first: BlockOffset = (*syslinep).blockoffset_first();
        if bo_first > 1 {
            def1ñ!();
            return self.drop_data(bo_first - 2);
        }

        false
    }

    /// Calls [`self.syslinereader.find_sysline(fileoffset)`],
    /// and in some cases calls private function `drop_block` to drop
    /// previously processed [`Sysline`], [`Line`], and [`Block`s].
    ///
    /// This is what implements the "streaming" in "[streaming stage]".
    ///
    /// [`self.syslinereader.find_sysline(fileoffset)`]: crate::readers::syslinereader::SyslineReader#method.find_sysline
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`Line`]: crate::data::line::Line
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [streaming stage]: crate::readers::syslogprocessor::ProcessingStage#variant.Stage3StreamSyslines
    pub fn find_sysline(
        &mut self,
        fileoffset: FileOffset,
    ) -> ResultS3SyslineFind {
        defn!("({})", fileoffset);
        let result: ResultS3SyslineFind = self
            .syslinereader
            .find_sysline(fileoffset);
        match result {
            ResultS3SyslineFind::Found(_) => {}
            ResultS3SyslineFind::Done => {}
            ResultS3SyslineFind::Err(ref err) => {
                self.set_error(err);
            }
        }
        defx!();

        result
    }

    /// Wrapper function for [`SyslineReader::find_sysline_between_datetime_filters`].
    /// Keeps a custom copy of any returned `Error` at `self.error`.
    ///
    /// [`SyslineReader::find_sysline_between_datetime_filters`]: crate::readers::syslinereader::SyslineReader#method.find_sysline_between_datetime_filters
    //
    // TODO: [2022/06/20] the `find` functions need consistent naming,
    //       `find_next`, `find_between`, `find_…` . The current design has
    //       the public-facing `find_` functions falling back on potential file-wide binary-search
    //       The binary-search only needs to be done during the stage 2. During stage 3, a simpler
    //       linear sequential search is more suitable, and more intuitive.
    //       More refactoring is in order.
    //       Also, a linear search can better detect rollover (i.e. when sysline datetime is missing year).
    // TODO: [2023/03/06] add stats tracking in `find` functions for number of
    //       "jumps" or bounces or fileoffset changes to confirm big-O
    #[inline(always)]
    pub fn find_sysline_between_datetime_filters(
        &mut self,
        fileoffset: FileOffset,
    ) -> ResultS3SyslineFind {
        defn!("({})", fileoffset);

        let result = match self
            .syslinereader
            .find_sysline_between_datetime_filters(
                fileoffset,
                &self.filter_dt_after_opt,
                &self.filter_dt_before_opt,
            ) {
            ResultS3SyslineFind::Err(err) => {
                self.set_error(&err);

                ResultS3SyslineFind::Err(err)
            }
            val => val,
        };

        defx!("({})", fileoffset);

        result
    }

    /// Wrapper function for a recurring sanity check.
    ///
    /// Good for checking functions `process_stage…` are called in
    /// the correct order.
    // XXX: is there a rust-ic way to enforce stage procession behavior
    //      at compile-time? It's a fairly simple enumerated type. Could a
    //      `match` tree (or something like that) be used?
    //      run-time checks of rust enum values seems hacky.
    #[inline(always)]
    fn assert_stage(
        &self,
        stage_expact: ProcessingStage,
    ) {
        debug_assert_eq!(
            self.processingstage, stage_expact,
            "Unexpected Processing Stage {:?}, expected Processing Stage {:?}",
            self.processingstage, stage_expact,
        );
    }

    /// Stage 0 does some sanity checks on the file.
    // TODO: this is redundant and has already been performed by functions in
    //       `filepreprocessor` and `BlockReader::new`.
    pub fn process_stage0_valid_file_check(&mut self) -> FileProcessingResultBlockZero {
        defn!();
        // sanity check calls are in correct order
        self.assert_stage(ProcessingStage::Stage0ValidFileCheck);
        self.processingstage = ProcessingStage::Stage0ValidFileCheck;

        if self.filesz() == 0 {
            defx!("filesz 0; return {:?}", FileProcessingResultBlockZero::FileErrEmpty);
            return FileProcessingResultBlockZero::FileErrEmpty;
        }
        defx!("return {:?}", FileProcessingResultBlockZero::FileOk);

        FileProcessingResultBlockZero::FileOk
    }

    /// Stage 1: Can [`Line`s] and [`Sysline`s] be parsed from the first block
    /// (block zero)?
    ///
    /// [`Sysline`s]: crate::data::sysline::Sysline
    /// [`Line`s]: crate::data::line::Line
    pub fn process_stage1_blockzero_analysis(&mut self) -> FileProcessingResultBlockZero {
        defn!();
        self.assert_stage(ProcessingStage::Stage0ValidFileCheck);
        self.processingstage = ProcessingStage::Stage1BlockzeroAnalysis;

        let result: FileProcessingResultBlockZero = self.blockzero_analysis();
        // stored syslines may be zero if a "partial" `Line` was examined
        // e.g. an incomplete and temporary `Line` instance was examined.
        defo!(
            "blockzero_analysis() stored syslines {}",
            self.syslinereader
                .count_syslines_stored()
        );
        match result {
            FileProcessingResult::FileOk => {}
            // skip further processing if not `FileOk`
            _ => {
                defx!("return {:?}", result);
                return result;
            }
        }

        defx!("return {:?}", result);

        result
    }

    /// Stage 2: Given the an optional datetime filter (user-passed
    /// `--dt-after`), can a log message with a datetime after that filter be
    /// found?
    pub fn process_stage2_find_dt(
        &mut self,
        filter_dt_after_opt: &DateTimeLOpt,
    ) -> FileProcessingResultBlockZero {
        defn!();
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);
        self.processingstage = ProcessingStage::Stage2FindDt;

        // datetime formats without a year requires special handling
        if !self
            .syslinereader
            .dt_pattern_has_year()
        {
            defo!("!dt_pattern_has_year()");
            let mtime: SystemTime = self.mtime();
            match self.process_missing_year(mtime, filter_dt_after_opt) {
                FileProcessingResultBlockZero::FileOk => {}
                result => {
                    defx!("Bad result {:?}", result);
                    return result;
                }
            }
        }

        defx!();

        FileProcessingResultBlockZero::FileOk
    }

    /// Stage 3: during "[streaming]", processed and printed data stored by
    /// underlying "Readers" is proactively dropped
    /// (removed from process memory).
    ///
    /// Also see [`find_sysline`].
    ///
    /// [streaming]: ProcessingStage#variant.Stage3StreamSyslines
    /// [`find_sysline`]: self::SyslogProcessor#method.find_sysline
    pub fn process_stage3_stream_syslines(&mut self) -> FileProcessingResultBlockZero {
        defñ!();
        self.assert_stage(ProcessingStage::Stage2FindDt);
        self.processingstage = ProcessingStage::Stage3StreamSyslines;

        FileProcessingResultBlockZero::FileOk
    }

    /// Stage 4: no more [`Sysline`s] to process. Create and return a
    /// [`Summary`].
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    /// [`Sysline`s]: crate::data::sysline::Sysline
    pub fn process_stage4_summary(&mut self) -> Summary {
        defñ!();
        // XXX: this can be called from various stages, no need to assert
        self.processingstage = ProcessingStage::Stage4Summary;

        self.summary_complete()
    }

    /// Review bytes in the first block ("zero block").
    /// If enough `Line` found then return [`FileOk`]
    /// else return [`FileErrNoLinesFound`].
    ///
    /// [`FileOk`]: self::FileProcessingResultBlockZero
    /// [`FileErrNoLinesFound`]: self::FileProcessingResultBlockZero
    pub(super) fn blockzero_analysis_bytes(&mut self) -> FileProcessingResultBlockZero {
        defn!();
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        let blockp: BlockP = match self
            .syslinereader
            .linereader
            .blockreader
            .read_block(0)
        {
            ResultS3ReadBlock::Found(blockp_) => blockp_,
            ResultS3ReadBlock::Done => {
                defx!("return FileErrEmpty");
                return FileProcessingResultBlockZero::FileErrEmpty;
            }
            ResultS3ReadBlock::Err(err) => {
                self.set_error(&err);
                defx!("return FileErrIo({:?})", err);
                return FileProcessingResultBlockZero::FileErrIoPath(err);
            }
        };
        // if the first block is too small then there will not be enough
        // data to parse a `Line` or `Sysline`
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let require_sz: BlockSz = std::cmp::min(Self::BLOCKZERO_ANALYSIS_BYTES_MIN, self.blocksz());
        defo!("blocksz0 {} < {} require_sz", blocksz0, require_sz);
        if blocksz0 < require_sz {
            defx!("return FileErrTooSmall");
            return FileProcessingResultBlockZero::FileErrTooSmall;
        }
        // if the first `BLOCKZERO_ANALYSIS_BYTES_NULL_MAX` bytes are all
        // zero then this is not a text file and processing should stop.
        if (*blockp).iter().take(Self::BLOCKZERO_ANALYSIS_BYTES_NULL_MAX).all(|&b| b == 0) {
            defx!("return FileErrNullBytes");
            return FileProcessingResultBlockZero::FileErrNullBytes;
        }

        defx!("return FileOk");

        FileProcessingResultBlockZero::FileOk
    }

    /// Attempt to find a minimum number of [`Line`s] within the first block
    /// (block zero).
    /// If enough `Line` found then return [`FileOk`]
    /// else return [`FileErrNoLinesFound`].
    ///
    /// [`Line`s]: crate::data::line::Line
    /// [`FileOk`]: self::FileProcessingResultBlockZero
    /// [`FileErrNoLinesFound`]: self::FileProcessingResultBlockZero
    pub(super) fn blockzero_analysis_lines(&mut self) -> FileProcessingResultBlockZero {
        defn!();
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        let blockp: BlockP = match self
            .syslinereader
            .linereader
            .blockreader
            .read_block(0)
        {
            ResultS3ReadBlock::Found(blockp_) => blockp_,
            ResultS3ReadBlock::Done => {
                defx!("return FileErrEmpty");
                return FileProcessingResultBlockZero::FileErrEmpty;
            }
            ResultS3ReadBlock::Err(err) => {
                self.set_error(&err);
                defx!("return FileErrIo({:?})", err);
                return FileProcessingResultBlockZero::FileErrIoPath(err);
            }
        };
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let mut _partial_found = false;
        let mut fo: FileOffset = 0;
        // how many lines have been found?
        let mut found: Count = 0;
        // must find at least this many lines in block zero to be FileOk
        let found_min: Count = *BLOCKZERO_ANALYSIS_LINE_COUNT_MIN_MAP
            .get(&blocksz0)
            .unwrap();
        defx!("block zero blocksz {} found_min {}", blocksz0, found_min);
        // find `found_min` Lines or whatever can be found within block 0
        while found < found_min {
            fo = match self
                .syslinereader
                .linereader
                .find_line_in_block(fo)
            {
                (ResultS3LineFind::Found((fo_next, _linep)), _) => {
                    found += 1;

                    fo_next
                }
                (ResultS3LineFind::Done, partial) => {
                    match partial {
                        Some(_) => {
                            found += 1;
                            _partial_found = true;
                        },
                        None => {}
                    }
                    break;
                }
                (ResultS3LineFind::Err(err), _) => {
                    self.set_error(&err);
                    defx!("return FileErrIo({:?})", err);
                    return FileProcessingResultBlockZero::FileErrIoPath(err);
                }
            };
            if 0 != self
                .syslinereader
                .linereader
                .block_offset_at_file_offset(fo)
            {
                break;
            }
        }

        let fpr: FileProcessingResultBlockZero = match found >= found_min {
            true => FileProcessingResultBlockZero::FileOk,
            false => FileProcessingResultBlockZero::FileErrNoLinesFound,
        };

        defx!("found {} lines, partial_found {}, require {} lines, return {:?}", found, _partial_found, found_min, fpr);

        fpr
    }

    /// Attempt to find a minimum number of [`Sysline`] within the first block.
    /// If enough `Sysline` found then return [`FileOk`]
    /// else return [`FileErrNoSyslinesFound`].
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    /// [`FileOk`]: self::FileProcessingResultBlockZero
    /// [`FileErrNoSyslinesFound`]: self::FileProcessingResultBlockZero
    pub(super) fn blockzero_analysis_syslines(&mut self) -> FileProcessingResultBlockZero {
        defn!();
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        let blockp: BlockP = match self
            .syslinereader
            .linereader
            .blockreader
            .read_block(0)
        {
            ResultS3ReadBlock::Found(blockp_) => blockp_,
            ResultS3ReadBlock::Done => {
                defx!("return FileErrEmpty");
                return FileProcessingResultBlockZero::FileErrEmpty;
            }
            ResultS3ReadBlock::Err(err) => {
                self.set_error(&err);
                defx!("return FileErrIo({:?})", err);
                return FileProcessingResultBlockZero::FileErrIoPath(err);
            }
        };
        let blocksz0: BlockSz = (*blockp).len() as BlockSz;
        let mut fo: FileOffset = 0;
        // how many syslines have been found?
        let mut found: Count = 0;
        // must find at least this many syslines in block zero to be FileOk
        let found_min: Count = *BLOCKZERO_ANALYSIS_SYSLINE_COUNT_MIN_MAP
            .get(&blocksz0)
            .unwrap();
        defo!("block zero blocksz {} found_min {:?}", blocksz0, found_min);

        // find `at_max` Syslines within block zero
        while found < found_min
            && self.syslinereader.block_offset_at_file_offset(fo) == 0
        {
            fo = match self
                .syslinereader
                .find_sysline_in_block(fo)
            {
                (ResultS3SyslineFind::Found((fo_next, _slinep)), _) => {
                    found += 1;
                    defo!("Found; found {} syslines, fo_next {}", found, fo_next);

                    fo_next
                }
                (ResultS3SyslineFind::Done, partial_found) => {
                    defo!("Done; found {} syslines, partial_found {}", found, partial_found);
                    if partial_found {
                        found += 1;
                    }
                    break;
                }
                (ResultS3SyslineFind::Err(err), _) => {
                    self.set_error(&err);
                    defx!("return FileErrIo({:?})", err);
                    return FileProcessingResultBlockZero::FileErrIoPath(err);
                }
            };
        }

        if found == 0 {
            defx!("found {} syslines, require {} syslines, return FileErrNoSyslinesFound", found, found_min);
            return FileProcessingResultBlockZero::FileErrNoSyslinesFound;
        }

        let patt_count_a = self.syslinereader.dt_patterns_counts_in_use();
        defo!("dt_patterns_counts_in_use {}", patt_count_a);

        if !self.syslinereader.dt_patterns_analysis() {
            de_err!("dt_patterns_analysis() failed which is unexpected; return FileErrNoSyslinesFound");
            return FileProcessingResultBlockZero::FileErrNoSyslinesFound;
        }

        let _patt_count_b = self.syslinereader.dt_patterns_counts_in_use();
        debug_assert_eq!(
            _patt_count_b,
            SyslogProcessor::DT_PATTERN_MAX,
            "expected patterns to be reduced to {}, found {:?}",
            SyslogProcessor::DT_PATTERN_MAX,
            _patt_count_b,
        );

        // if more than one `DateTimeParseInstr` was used then the syslines
        // must be reparsed using the one chosen `DateTimeParseInstr`
        if patt_count_a > 1 {
            defo!("must reprocess all syslines using limited patterns (used {} DateTimeParseInstr; must only use {})!", patt_count_a, 1);

            self.syslinereader.clear_syslines();
            // find `at_max` Syslines within block zero
            found = 0;
            fo = 0;
            while found < found_min
                && self.syslinereader.block_offset_at_file_offset(fo) == 0
            {
                fo = match self
                    .syslinereader
                    .find_sysline_in_block(fo)
                {
                    (ResultS3SyslineFind::Found((fo_next, _slinep)), _) => {
                        found += 1;
                        defo!("Found; found {} syslines, fo_next {}", found, fo_next);

                        fo_next
                    }
                    (ResultS3SyslineFind::Done, partial_found) => {
                        defo!("Done; found {} syslines, partial_found {}", found, partial_found);
                        if partial_found {
                            found += 1;
                        }
                        break;
                    }
                    (ResultS3SyslineFind::Err(err), _) => {
                        self.set_error(&err);
                        defx!("return FileErrIo({:?})", err);
                        return FileProcessingResultBlockZero::FileErrIoPath(err);
                    }
                };
            }
            defo!("done reprocessing.");
        } else {
            defo!("no reprocess needed ({} DateTimeParseInstr)!", patt_count_a);
        }

        let fpr: FileProcessingResultBlockZero = match found >= found_min {
            true => FileProcessingResultBlockZero::FileOk,
            false => FileProcessingResultBlockZero::FileErrNoSyslinesFound,
        };

        defx!("found {} syslines, require {} syslines, return {:?}", found, found_min, fpr);

        fpr
    }

    /// Call `self.blockzero_analysis_lines`.
    /// If that passes then call `self.blockzero_analysis_syslines`.
    pub(super) fn blockzero_analysis(&mut self) -> FileProcessingResultBlockZero {
        defn!();
        assert!(!self.blockzero_analysis_done, "blockzero_analysis_lines should only be completed once.");
        self.blockzero_analysis_done = true;
        self.assert_stage(ProcessingStage::Stage1BlockzeroAnalysis);

        if self.syslinereader.filesz() == 0 {
            defx!("return FileErrEmpty");
            return FileProcessingResultBlockZero::FileErrEmpty;
        }

        let result: FileProcessingResultBlockZero = self.blockzero_analysis_bytes();
        if !result.is_ok() {
            defx!("syslinereader.blockzero_analysis_bytes() was !is_ok(), return {:?}", result);
            return result;
        };

        let result: FileProcessingResultBlockZero = self.blockzero_analysis_lines();
        if !result.is_ok() {
            defx!("syslinereader.blockzero_analysis() was !is_ok(), return {:?}", result);
            return result;
        };

        let result: FileProcessingResultBlockZero = self.blockzero_analysis_syslines();
        defx!("return {:?}", result);

        result
    }

    #[cfg(test)]
    pub(crate) fn dropped_blocks(&self) -> SetDroppedBlocks {
        self.syslinereader
            .linereader
            .blockreader
            .dropped_blocks
            .clone()
    }

    #[cfg(test)]
    pub(crate) fn dropped_lines(&self) -> SetDroppedLines {
        self.syslinereader
            .linereader
            .dropped_lines
            .clone()
    }

    #[cfg(test)]
    pub(crate) fn dropped_syslines(&self) -> SetDroppedSyslines {
        self.syslinereader
            .dropped_syslines
            .clone()
    }

    pub fn summary(&self) -> SummarySyslogProcessor {
        let SyslogProcessor_missing_year = self.missing_year;

        SummarySyslogProcessor {
            SyslogProcessor_missing_year,
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `SyslogProcessor`.
    ///
    /// Probably not useful or interesting before
    /// `ProcessingStage::Stage4Summary`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let filetype = self.filetype();
        let logmessagetype = filetype_to_logmessagetype(filetype);
        let summaryblockreader = self.syslinereader.linereader.blockreader.summary();
        let summarylinereader = self.syslinereader.linereader.summary();
        let summarysyslinereader = self.syslinereader.summary();
        let summarysyslogprocessor = self.summary();
        let error: Option<String> = self.error.clone();

        Summary::new(
            path,
            filetype,
            logmessagetype,
            Some(summaryblockreader),
            Some(summarylinereader),
            Some(summarysyslinereader),
            Some(summarysyslogprocessor),
            None,
            None,
            None,
            error,
        )
    }
}
