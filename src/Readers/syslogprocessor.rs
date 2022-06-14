// Readers/syslogprocessor.rs
//

use crate::common::{
    FPath,
    FileOffset,
    FileProcessingResult,
    FileType,
};

use crate::Readers::blockreader::{
    BlockIndex,
    BlockOffset,
    BlockSz,
    BlockP,
    ResultS3_ReadBlock,
};

use crate::printer::printers::{
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

use crate::Data::datetime::{
    FixedOffset,
    DateTimeL,
    DateTimeL_Opt,
};

pub use crate::Readers::linereader::{
    ResultS4_LineFind,
};

pub use crate::Readers::syslinereader::{
    ResultS4_SyslineFind,
    Sysline,
    SyslineP,
    SyslineReader,
};

use crate::Readers::summary::{
    Summary,
};

use std::fmt;
use std::io::{
    Error,
    Result,
    ErrorKind,
};

extern crate debug_print;
use debug_print::{debug_eprint, debug_eprintln};

extern crate mime_guess;
use mime_guess::MimeGuess;

extern crate mime_sniffer;
use mime_sniffer::MimeTypeSniffer;  // adds extension method `sniff_mime_type` to `[u8]`

extern crate more_asserts;
use more_asserts::{
    assert_le,
    assert_lt,
    assert_ge,
    assert_gt,
    debug_assert_le,
    debug_assert_lt,
    debug_assert_ge,
};

extern crate static_assertions;
use static_assertions::{
    const_assert,
};

extern crate walkdir;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub type FileProcessingResult_BlockZero = FileProcessingResult<std::io::Error>;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProcessingMode {
    stage0_valid_file_check,
    stage1_blockzero_analysis,
    stage2_find_dt,
    stage3_stream_syslines,
    stage4_summary,
}

/// A `Sysline` has information about a "syslog line" that spans one or more `Line`s.
/// A "syslog line" or `Sysline` is one or more `Line`s, where the first line contains a
/// datetime stamp. That datetime stamp is consistent format of other nearby syslog lines.
pub struct SyslogProcessor {
    syslinereader: SyslineReader,
    processingmode: ProcessingMode,
    path: FPath,
    blocksz: BlockSz,
    tz_offset: FixedOffset,
    /// has `self.blockzero_analysis()` completed?
    blockzero_analysis_done: bool,
}

impl std::fmt::Debug for SyslogProcessor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SyslogProcessor")
            .field("Path", &self.path)
            .field("Processing Mode", &self.processingmode)
            .field("BlockSz", &self.blocksz)
            .field("TimeOffset", &self.tz_offset)
            .field("BO Analysis done?", &self.blockzero_analysis_done)
            .field("filetype", &self.filetype())
            .field("MimeGuess", &self.mimeguess())
            .finish()
    }
}

impl SyslogProcessor {

    /// TODO: [2022/06/01] this should be predefined mapping of key range to value integer,
    ///       where blocksz keys to count of expected line.
    ///       e.g. blocksz [2, 64] expect 1 line, blocksz [64, 1024] expect 5 lines, etc.
    /// `SyslogProcessor::blockzero_analysis_lines` must find this many `Line` for the
    /// file to be considered a text file
    pub (crate) const BLOCKZERO_ANALYSIS_LINE_COUNT: u64 = 15;

    /// `SyslogProcessor::blockzero_analysis_syslines` must find this many `Sysline` for the
    /// file to be considered a syslog file
    pub (crate) const BLOCKZERO_ANALYSIS_SYSLINE_COUNT: u64 = 2;

    pub fn new(path: FPath, filetype: FileType, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslogProcessor> {
        debug_eprintln!("{}SyslogProcessor::new({:?}, {:?}, {:?}, {:?})", snx(), path, filetype, blocksz, tz_offset);
        let path_ = path.clone();
        let slr = match SyslineReader::new(path, filetype, blocksz, tz_offset) {
            Ok(val) => val,
            Err(err) => {
                return Result::Err(err);
            }
        };
        Result::Ok(
            SyslogProcessor {
                syslinereader: slr,
                processingmode: ProcessingMode::stage0_valid_file_check,
                path: path_,
                blocksz,
                tz_offset,
                blockzero_analysis_done: false,
            }
        )
    }

    #[inline]
    pub fn lines_count(&self) -> u64 {
        self.syslinereader.linereader.lines_count
    }

    #[inline]
    pub const fn blocksz(&self) -> BlockSz {
        self.syslinereader.blocksz()
    }

    #[inline]
    pub const fn filesz(&self) -> u64 {
        self.syslinereader.filesz()
    }

    #[inline]
    pub const fn filetype(&self) -> FileType {
        self.syslinereader.filetype()
    }

    #[inline]
    pub const fn path(&self) -> &FPath {
        self.syslinereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub const fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.syslinereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub const fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.syslinereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub const fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.syslinereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub const fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.syslinereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub const fn file_blocks_count(&self) -> u64 {
        self.syslinereader.file_blocks_count()
    }

    /// last valid `BlockOffset` of the file
    pub const fn blockoffset_last(&self) -> BlockOffset {
        self.syslinereader.blockoffset_last()
    }

    /// smallest size character in bytes
    pub const fn charsz(&self) -> usize {
        self.syslinereader.charsz()
    }

    /// wrapper to `self.syslinereader.linereader.blockreader.mimeguess`
    pub const fn mimeguess(&self) -> MimeGuess {
        self.syslinereader.mimeguess()
    }

    /// wrapper to `self.syslinereader.find_sysline`
    pub fn find_sysline(&mut self, fileoffset: FileOffset) -> ResultS4_SyslineFind {
        self.syslinereader.find_sysline(fileoffset)
    }

    /// wrapper to `self.syslinereader.is_sysline_last`
    pub(crate) fn is_sysline_last(&self, syslinep: &SyslineP) -> bool {
        self.syslinereader.is_sysline_last(syslinep)
    }

    /// wrapper to `self.syslinereader.find_sysline_between_datetime_filters`
    pub fn find_sysline_between_datetime_filters(
        &mut self, fileoffset: FileOffset, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
    ) -> ResultS4_SyslineFind {
        self.syslinereader.find_sysline_between_datetime_filters(
            fileoffset, dt_filter_after, dt_filter_before,
        )
    }

    /// TODO: complete this
    pub fn process_stage0_valid_file_check(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage0_valid_file_check", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage0_valid_file_check,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage0_valid_file_check,
        );
        self.processingmode = ProcessingMode::stage1_blockzero_analysis;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage1_blockzero_analysis(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage1_blockzero_analysis", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage1_blockzero_analysis,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage1_blockzero_analysis,
        );
        self.processingmode = ProcessingMode::stage2_find_dt;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage2_find_dt(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage2_find_dt", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage2_find_dt,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage2_find_dt,
        );
        self.processingmode = ProcessingMode::stage3_stream_syslines;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage3_stream_syslines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage3_stream_syslines", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage3_stream_syslines,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage3_stream_syslines,
        );
        self.processingmode = ProcessingMode::stage4_summary;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage4_summary(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.process_stage4_summary", snx());
        self.processingmode = ProcessingMode::stage4_summary;

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// read block zero (the first data block of the file), do necessary analysis
    pub(crate) fn blockzero_analysis_syslines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines", sn());

        let mut fo: FileOffset = 0;
        let mut at: u64 = 0;
        let at_max: u64 = SyslogProcessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT;
        // TODO: `at_max` should be adjusted based on `blocksz` and `filesz`
        //       small `blocksz` should mean small `at_max`, etc.
        while at < at_max {
            fo = match self.syslinereader.find_sysline_in_block(fo) {
                ResultS4_SyslineFind::Found((fo_next, _slinep)) => {
                    fo_next
                },
                ResultS4_SyslineFind::Found_EOF((_fo_next, _slinep)) => {
                    break;
                }, ResultS4_SyslineFind::Done => {
                    let mut fpr = FileProcessingResult_BlockZero::FILE_ERR_NO_SYSLINES_FOUND;
                    if at != 0 {
                        fpr = FileProcessingResult_BlockZero::FILE_OK;
                    }
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: found {} syslines, {:?}", sx(), at, fpr);
                    return fpr;
                }, ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: return FILE_ERR_IO({:?})", sx(), err);
                    return FileProcessingResult_BlockZero::FILE_ERR_IO(err);
                },
            };
            if 0 != self.syslinereader.block_offset_at_file_offset(fo) {
                break;
            }
            at += 1;
        }

        debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: found {} syslines, return FILE_OK", sx(), at);
        FileProcessingResult_BlockZero::FILE_OK
    }

    /// helper to `blockzero_analysis_lines`
    ///
    /// attempt to find `Line` within the first block (block zero).
    /// if enough `Line` found then return `Ok(true)` else `Ok(false)`.
    /// 
    fn blockzero_analysis_lines_readlines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines()", sn());
        // could not guess suitability based on MIME type
        // so try to parse BLOCKZERO_ANALYSIS_LINE_COUNT `Lines`

        let mut fo: FileOffset = 0;
        let mut at: u64 = 0;
        let at_max: u64 = SyslogProcessor::BLOCKZERO_ANALYSIS_LINE_COUNT;
        // find max 15 Lines or whatever can be found within block 0
        // TODO: `at_max` should be adjusted based on `blocksz` and `filesz`
        //       small blocksz should lower value of `at_max`, etc.
        while at < at_max {
            fo = match self.syslinereader.linereader.find_line_in_block(fo) {
                ResultS4_LineFind::Found((fo_, _linep)) => {
                    fo_
                },
                ResultS4_LineFind::Found_EOF((_fo, _linep)) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() found {} lines, return FILE_OK", sx(), at);
                    return FileProcessingResult_BlockZero::FILE_OK;
                },
                ResultS4_LineFind::Done => {
                    let mut fpr = FileProcessingResult_BlockZero::FILE_ERR_NO_LINES_FOUND;
                    if at != 0 {
                        fpr = FileProcessingResult_BlockZero::FILE_OK;
                    }
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() found {} lines, return {:?}", sx(), at, fpr);
                    return fpr;
                },
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() return FILE_ERR_IO({:?})", sx(), err);
                    return FileProcessingResult_BlockZero::FILE_ERR_IO(err);
                },
            };
            if 0 != self.syslinereader.linereader.block_offset_at_file_offset(fo) {
                break;
            }
            at += 1;
        }

        debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() found {} lines, return FILE_OK", sx(), at);

        FileProcessingResult_BlockZero::FILE_OK
    }

    /// Analyze block 0 (the first block, the "zero block") and make best guesses
    /// about the file.
    ///
    /// Return `true` if enough is known about the file to proceed with further analysis
    /// (e.g. calls to `linereader.find_line`).
    /// Else return `false`.
    /// Calls `blockreader.read_block(0)`
    ///
    /// Should only call to completion once per `SyslogProcessor` instance.
    ///
    /// TODO: mime analysis not currently used, either use it or remove it.
    pub(crate) fn blockzero_analysis_lines(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis()", sn());
        assert!(!self.blockzero_analysis_done, "blockzero_analysis should only be completed once.");
        self.blockzero_analysis_done = true;

        // TODO: something!

        debug_eprintln!("{}syslogprocessor.blockzero_analysis() return …", sx());
        self.blockzero_analysis_lines_readlines()
    }

    pub fn blockzero_analysis(&mut self) -> FileProcessingResult_BlockZero {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis", sn());
        if self.filesz() == 0 {
            debug_eprintln!("{}syslogprocessor.blockzero_analysis: filesz 0; return {:?}", sx(), FileProcessingResult_BlockZero::FILE_ERR_EMPTY);
            return FileProcessingResult_BlockZero::FILE_ERR_EMPTY;
        }
        let result = self.blockzero_analysis_lines();
        if ! result.is_ok() {
            debug_eprintln!("{}syslogprocessor.blockzero_analysis: syslinereader.blockzero_analysis() was !is_ok(), return {:?}", sx(), result);
            return result;
        };

        debug_eprintln!("{}syslogprocessor.blockzero_analysis", sx());

        self.blockzero_analysis_syslines()
    }

    /// return an up-to-date `Summary` instance for this `SyslogProcessor`
    pub fn summary(&self) -> Summary {
        let filetype = self.filetype();
        let BlockReader_bytes = self.syslinereader.linereader.blockreader.count_bytes();
        let BlockReader_bytes_total = self.filesz() as u64;
        let BlockReader_blocks = self.syslinereader.linereader.blockreader.count_blocks();
        let BlockReader_blocks_total = self.syslinereader.linereader.blockreader.blockn;
        let BlockReader_blocksz = self.blocksz();
        let BlockReader_filesz = self.syslinereader.linereader.blockreader.filesz;
        let BlockReader_filesz_actual = self.syslinereader.linereader.blockreader.filesz_actual;
        let LineReader_lines = self.syslinereader.linereader.count_lines_processed();
        let SyslineReader_syslines = self.syslinereader.count_syslines_processed();
        let SyslineReader_syslines_by_range_hit = self.syslinereader._syslines_by_range_hit;
        let SyslineReader_syslines_by_range_miss = self.syslinereader._syslines_by_range_miss;
        let SyslineReader_syslines_by_range_insert = self.syslinereader._syslines_by_range_insert;
        let SyslineReader_patterns = self.syslinereader.dt_patterns.clone();
        let SyslineReader_find_sysline_lru_cache_hit = self.syslinereader._find_sysline_lru_cache_hit;
        let SyslineReader_find_sysline_lru_cache_miss = self.syslinereader._find_sysline_lru_cache_miss;
        let SyslineReader_find_sysline_lru_cache_put = self.syslinereader._find_sysline_lru_cache_put;
        let SyslineReader_parse_datetime_in_line_lru_cache_hit = self.syslinereader._parse_datetime_in_line_lru_cache_hit;
        let SyslineReader_parse_datetime_in_line_lru_cache_miss = self.syslinereader._parse_datetime_in_line_lru_cache_miss;
        let SyslineReader_parse_datetime_in_line_lru_cache_put = self.syslinereader._parse_datetime_in_line_lru_cache_put;
        let LineReader_find_line_lru_cache_hit = self.syslinereader.linereader._find_line_lru_cache_hit;
        let LineReader_find_line_lru_cache_miss = self.syslinereader.linereader._find_line_lru_cache_miss;
        let LineReader_find_line_lru_cache_put = self.syslinereader.linereader._find_line_lru_cache_put;
        let BlockReader_read_block_lru_cache_hit = self.syslinereader.linereader.blockreader._read_block_cache_lru_hit;
        let BlockReader_read_block_lru_cache_miss = self.syslinereader.linereader.blockreader._read_block_cache_lru_miss;
        let BlockReader_read_block_lru_cache_put = self.syslinereader.linereader.blockreader._read_block_cache_lru_put;
        let BlockReader_read_blocks_hit = self.syslinereader.linereader.blockreader._read_blocks_hit;
        let BlockReader_read_blocks_miss = self.syslinereader.linereader.blockreader._read_blocks_miss;
        let BlockReader_read_blocks_insert = self.syslinereader.linereader.blockreader._read_blocks_insert;

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
            SyslineReader_syslines_by_range_hit,
            SyslineReader_syslines_by_range_miss,
            SyslineReader_syslines_by_range_insert,
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
            BlockReader_read_block_lru_cache_hit,
            BlockReader_read_block_lru_cache_miss,
            BlockReader_read_block_lru_cache_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
        )
    }
}

const_assert!(
    SyslogProcessor::BLOCKZERO_ANALYSIS_LINE_COUNT > SyslogProcessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT
);


// TODO: [2022/06/02] AFAICT, this doens't need to be a long-lived object,
// only a series of functions... thinking about it... this series of functions could
// be placed within `syslogprocessor.rs`:
//    pub fn generate_syslogprocessor(path: FPath) -> Vec<(ProcessPathResult, Option<SyslogProcessor>)>
// with helper function:
//    pub fn process_path(path: FPath) -> Vec<ProcessPathResult>
//
// type ProcessPathResult = (Path, Option<SubPath>, FileType);
//
// The algorithm for analyzing a path would be:
//    if directory the recurse directory for more paths.
//    if not file then eprintln and (if --summary the save error) and return.
//    (must be plain file so)
//    if file name implies obvious file type then presume mimeguess to be correct.
//       example, `messages.gz`, is very likely a gzipped text file. Try to gunzip. If gunzip fails then give up on it. (`FILE_ERR_DECOMPRESS_FAILED`)
//       example, `logs.tar`, is very likely multiple tarred text files. Try to untar. If untar fails then give up on it. (`FILE_ERR_UNARCHIVE_FAILED`)
//    else if mime analysis has likely answer then presume that to be correct.
//        example, `messages`, is very likely a text file.
//    else try blockzero analysis (attempt to parse Lines and Syslines).
// Failures to process paths should be:
//    eprintln() at time of opening failure.
//    if --summary then printed with the closing summary.
//
// That algorithm should be correct in 99% of cases.
//

//pub fn generate_syslogprocessor(path: FPath) -> Vec<(ProcessPathResult, Option<SyslogProcessor>)> {
//    vec![]
//}

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
