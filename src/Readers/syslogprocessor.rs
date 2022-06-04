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

use crate::Readers::datetime::{
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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SyslogProcessor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
    filetype: FileType,
}

impl std::fmt::Debug for SyslogProcessor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SyslogProcessor")
            .field("Path", &self.path)
            .field("Processing Mode", &self.processingmode)
            .field("BlockSz", &self.blocksz)
            .field("TimeOffset", &self.tz_offset)
            .field("ZeroBlockAnalysis done?", &self.blockzero_analysis_done)
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

    pub fn new(path: FPath, blocksz: BlockSz, tz_offset: FixedOffset) -> Result<SyslogProcessor> {
        let path_ = path.clone();
        let mut slr = match SyslineReader::new(path, blocksz, tz_offset) {
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
                filetype: FileType::_FILE_UNSET,
            }
        )
    }

    pub fn lines_count(&self) -> u64 {
        self.syslinereader.linereader.lines_count
    }

    pub fn blocksz(&self) -> BlockSz {
        self.syslinereader.blocksz()
    }

    pub fn filesz(&self) -> u64 {
        self.syslinereader.filesz()
    }

    pub fn path(&self) -> &FPath {
        self.syslinereader.path()
    }

    /// return nearest preceding `BlockOffset` for given `FileOffset` (file byte offset)
    pub fn block_offset_at_file_offset(&self, fileoffset: FileOffset) -> BlockOffset {
        self.syslinereader.block_offset_at_file_offset(fileoffset)
    }

    /// return file_offset (file byte offset) at given `BlockOffset`
    pub fn file_offset_at_block_offset(&self, blockoffset: BlockOffset) -> FileOffset {
        self.syslinereader.file_offset_at_block_offset(blockoffset)
    }

    /// return file_offset (file byte offset) at blockoffset+blockindex
    pub fn file_offset_at_block_offset_index(&self, blockoffset: BlockOffset, blockindex: BlockIndex) -> FileOffset {
        self.syslinereader
            .file_offset_at_block_offset_index(blockoffset, blockindex)
    }

    /// return block index at given `FileOffset`
    pub fn block_index_at_file_offset(&self, fileoffset: FileOffset) -> BlockIndex {
        self.syslinereader.block_index_at_file_offset(fileoffset)
    }

    /// return count of blocks in a file, also, the last blockoffset + 1
    pub fn file_blocks_count(&self) -> u64 {
        self.syslinereader.file_blocks_count()
    }

    /// last valid `BlockOffset` of the file
    pub fn blockoffset_last(&self) -> BlockOffset {
        self.syslinereader.blockoffset_last()
    }

    /// smallest size character in bytes
    pub fn charsz(&self) -> usize {
        self.syslinereader.charsz()
    }

    /// wrapper to `self.syslinereader.linereader.blockreader.mimeguess`
    pub fn mimeguess(&self) -> MimeGuess {
        self.syslinereader.linereader.blockreader.mimeguess
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
    pub fn process_stage0_valid_file_check(&mut self) -> FileProcessingResult {
        debug_eprintln!("{}syslogprocessor.process_stage0_valid_file_check", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage0_valid_file_check,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage0_valid_file_check,
        );
        self.processingmode = ProcessingMode::stage1_blockzero_analysis;

        FileProcessingResult::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage1_blockzero_analysis(&mut self) -> FileProcessingResult {
        debug_eprintln!("{}syslogprocessor.process_stage1_blockzero_analysis", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage1_blockzero_analysis,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage1_blockzero_analysis,
        );
        self.processingmode = ProcessingMode::stage2_find_dt;

        FileProcessingResult::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage2_find_dt(&mut self) -> FileProcessingResult {
        debug_eprintln!("{}syslogprocessor.process_stage2_find_dt", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage2_find_dt,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage2_find_dt,
        );
        self.processingmode = ProcessingMode::stage3_stream_syslines;

        FileProcessingResult::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage3_stream_syslines(&mut self) -> FileProcessingResult {
        debug_eprintln!("{}syslogprocessor.process_stage3_stream_syslines", snx());
        assert_eq!(
            self.processingmode, ProcessingMode::stage3_stream_syslines,
            "Unexpected Processing Mode {:?}, expected Processing Mode {:?}",
            self.processingmode, ProcessingMode::stage3_stream_syslines,
        );
        self.processingmode = ProcessingMode::stage4_summary;

        FileProcessingResult::FILE_OK
    }

    /// TODO: complete this
    pub fn process_stage4_summary(&mut self) -> FileProcessingResult {
        debug_eprintln!("{}syslogprocessor.process_stage4_summary", snx());
        self.processingmode = ProcessingMode::stage4_summary;

        FileProcessingResult::FILE_OK
    }

    // TODO: this should return `FileProcessingResult`
    pub fn blockzero_analysis(&mut self) -> std::io::Result<bool> {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis", sn());
        match self.blockzero_analysis_lines() {
            Ok(val) => {
                if !val {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis: syslinereader.blockzero_analysis() was false, return Ok(false)", sx());
                    return Ok(false);
                };
            },
            Err(err) => {
                debug_eprintln!("{}syslogprocessor.blockzero_analysis: return Err({:?})", sx(), err);
                return Err(err);
            }
        }

        debug_eprintln!("{}syslogprocessor.blockzero_analysis", sx());

        self.blockzero_analysis_syslines()
    }

    /// read block zero (the first data block of the file), do necessary analysis
    /// 
    /// TODO: this should return `FileProcessingResult`
    pub(crate) fn blockzero_analysis_syslines(&mut self) -> std::io::Result<bool> {
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
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: found {} syslines, Ok({})", sx(), at, at != 0);
                    return Ok(at != 0);
                }, ResultS4_SyslineFind::Err(err) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: return Err({:?})", sx(), err);
                    return Result::Err(err);
                },
            };
            if 0 != self.syslinereader.block_offset_at_file_offset(fo) {
                break;
            }
            at += 1;
        }

        debug_eprintln!("{}syslogprocessor.blockzero_analysis_syslines: found {} syslines, return Ok(true)", sx(), at);
        Ok(true)
    }

    // LAST WORKING HERE [2022/06/02 23:45:00] see TODO in `fileprocessor.rs`

    /// map `MimeGuess` into a `FileType`
    /// (i.e. call `find_line`)
    pub fn parseable_mimeguess_str(mimeguess_str: &str) -> FileType {
        // see https://docs.rs/mime/latest/mime/
        // see https://docs.rs/mime/latest/src/mime/lib.rs.html#572-575
        debug_eprintln!("{}LineReader::parseable_mimeguess_str: mimeguess {:?}", snx(), mimeguess_str);
        match mimeguess_str {
            "plain"
            | "text"
            | "text/plain"
            | "text/*"
            | "utf-8" => {FileType::FILE},
            _ => {FileType::FILE_UNKNOWN},
        }
    }

    /// should `LineReader` attempt to parse this file/MIME type?
    /// (i.e. call `find_line`)
    pub fn parseable_mimeguess(mimeguess: &MimeGuess) -> FileType {
        for mimeguess_ in mimeguess.iter() {
            match SyslogProcessor::parseable_mimeguess_str(mimeguess_.as_ref()) {
                FileType::FILE_UNKNOWN
                | FileType::_FILE_UNSET => {},
                val => { return val; }
            }
        }

        FileType::FILE_UNKNOWN
    }

    pub(crate) fn mimesniff_analysis(&mut self) -> Result<bool> {
        let bo_zero: FileOffset = 0;
        debug_eprintln!("{}linereader.mimesniff_analysis: self.blockreader.read_block({:?})", sn(), bo_zero);
        let bptr: BlockP = match self.syslinereader.linereader.blockreader.read_block(bo_zero) {
            ResultS3_ReadBlock::Found(val) => val,
            ResultS3_ReadBlock::Done => {
                debug_eprintln!("{}linereader.mimesniff_analysis: read_block({}) returned Done for {:?}, return Error(UnexpectedEof)", sx(), bo_zero, self.path());
                assert_eq!(self.filesz(), 0, "readblock(0) returned Done for file with size {}", self.filesz());
                return Ok(false);
            },
            ResultS3_ReadBlock::Err(err) => {
                debug_eprintln!("{}linereader.mimesniff_analysis: read_block({}) returned Err {:?}", sx(), bo_zero, err);
                return Result::Err(err);
            },
        };

        let sniff: String = String::from((*bptr).as_slice().sniff_mime_type().unwrap_or(""));
        debug_eprintln!("{}linereader.mimesniff_analysis: sniff_mime_type {:?}", so(), sniff);
        // TODO: this function should be moved to filepreprocssor.rs and modified
        //let is_parseable: bool = SyslogProcessor::parseable_mimeguess_str(sniff.as_ref());
        let is_parseable = false;

        debug_eprintln!("{}linereader.mimesniff_analysis: return Ok({:?})", sx(), is_parseable);
        Ok(is_parseable)
    }

    pub(crate) fn mimeguess_analysis(&mut self) -> bool {
        let mimeguess_ = self.mimeguess();
        debug_eprintln!("{}linereader.mimeguess_analysis: mimeguess is {:?}", sn(), mimeguess_);
        let mut is_parseable: bool = false;

        if !mimeguess_.is_empty() {
            // TODO: this function should be moved to filepreprocssor.rs and modified
            //is_parseable = SyslogProcessor::parseable_mimeguess(&mimeguess_);
            debug_eprintln!("{}linereader.mimeguess_analysis: parseable_mimeguess {:?}", sx(), is_parseable);
            return is_parseable;
        }
        debug_eprintln!("{}linereader.mimeguess_analysis: {:?}", sx(), is_parseable);

        is_parseable
    }

    /// helper to `blockzero_analysis_lines`
    ///
    /// attempt to find `Line` within the first block (block zero).
    /// if enough `Line` found then return `Ok(true)` else `Ok(false)`.
    /// 
    /// TODO: this should return `FileProcessingResult`
    fn blockzero_analysis_lines_readlines(&mut self) -> Result<bool> {
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
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() found {} lines, return Ok(true)", sx(), at);
                    return Ok(true);
                },
                ResultS4_LineFind::Done => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() found {} lines, return Ok({})", sx(), at, at != 0);
                    return Ok(at != 0);
                },
                ResultS4_LineFind::Err(err) => {
                    debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() return Err({:?})", sx(), err);
                    return Result::Err(err);
                },
            };
            if 0 != self.syslinereader.linereader.block_offset_at_file_offset(fo) {
                break;
            }
            at += 1;
        }

        debug_eprintln!("{}syslogprocessor.blockzero_analysis_readlines() found {} lines, return Ok(true)", at, sx());

        Ok(true)
    }

    /// Given a file of an unknown MIME type (`self.blockreader.mimeguess.is_empty()`),
    /// analyze block 0 (the first block, the "zero block") and make best guesses
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
    ///
    /// TODO: this should return `FileProcessingResult`
    pub(crate) fn blockzero_analysis_lines(&mut self) -> Result<bool> {
        debug_eprintln!("{}syslogprocessor.blockzero_analysis()", sn());
        assert!(!self.blockzero_analysis_done, "blockzero_analysis should only be completed once.");

        self.blockzero_analysis_done = true;

        // XXX: this is not used
        let mimeguess_ = self.mimeguess_analysis();
        debug_eprintln!("{}syslogprocessor.blockzero_analysis() mimeguess_analysis() {}", so(), mimeguess_);

        // XXX: this is not used
        let mimesniff_ = match self.mimesniff_analysis() {
            Ok(val) => val,
            Err(err) => {
                debug_eprintln!("{}syslogprocessor.blockzero_analysis() return Err({})", sx(), err);
                return Err(err);
            }
        };
        debug_eprintln!("{}syslogprocessor.blockzero_analysis() mimesniff_analysis() {}", so(), mimesniff_);

        debug_eprintln!("{}syslogprocessor.blockzero_analysis() return …", sx());

        self.blockzero_analysis_lines_readlines()
    }

    /// return an up-to-date `Summary` instance for this `SyslogProcessor`
    pub fn summary(&self) -> Summary {
        let BlockReader_bytes = self.syslinereader.linereader.blockreader.count_bytes();
        let BlockReader_bytes_total = self.syslinereader.linereader.blockreader.filesz as u64;
        let BlockReader_blocks = self.syslinereader.linereader.blockreader.count();
        let BlockReader_blocks_total = self.syslinereader.linereader.blockreader.blockn;
        let BlockReader_blocksz = self.blocksz();
        let LineReader_lines = self.syslinereader.linereader.count();
        let SyslineReader_syslines = self.syslinereader.count();
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
        let BlockReader_read_block_cache_lru_hit = self.syslinereader.linereader.blockreader._read_block_cache_lru_hit;
        let BlockReader_read_block_cache_lru_miss = self.syslinereader.linereader.blockreader._read_block_cache_lru_miss;
        let BlockReader_read_block_cache_lru_put = self.syslinereader.linereader.blockreader._read_block_cache_lru_put;
        let BlockReader_read_blocks_hit = self.syslinereader.linereader.blockreader._read_blocks_hit;
        let BlockReader_read_blocks_miss = self.syslinereader.linereader.blockreader._read_blocks_miss;
        let BlockReader_read_blocks_insert = self.syslinereader.linereader.blockreader._read_blocks_insert;

        Summary::new(
            BlockReader_bytes,
            BlockReader_bytes_total,
            BlockReader_blocks,
            BlockReader_blocks_total,
            BlockReader_blocksz,
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
            BlockReader_read_block_cache_lru_hit,
            BlockReader_read_block_cache_lru_miss,
            BlockReader_read_block_cache_lru_put,
            BlockReader_read_blocks_hit,
            BlockReader_read_blocks_miss,
            BlockReader_read_blocks_insert,
        )
    }
}

const_assert!(
    SyslogProcessor::BLOCKZERO_ANALYSIS_LINE_COUNT > SyslogProcessor::BLOCKZERO_ANALYSIS_SYSLINE_COUNT
);

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
