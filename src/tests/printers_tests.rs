// src/tests/printers_tests.rs

//! tests for [`src/printer/printers.rs`]
//!
//! [`src/printer/printers.rs`]: crate::printer::printers

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use ::const_format::concatcp;
use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
};
use ::test_case::test_case;

use crate::common::{
    AllocatorChosen,
    Count,
    FPath,
    FileOffset,
    FileType,
    FileTypeArchive,
    FileTypeFixedStruct,
    FileTypeTextEncoding,
    LogMessageType,
    RegexId,
    SetPathId,
    summary_stats_enable,
};
use crate::data::datetime::{
    FixedOffset,
    Local,
    Utc,
    regex_id_compiled,
};
use crate::data::fixedstruct::ENTRY_SZ_MAX;
use crate::debug::helpers::{
    create_temp_file,
    ntf_fpath,
    NamedTempFile,
};
use crate::libload::systemd_dlopen2::load_library_systemd;
use crate::printer::printers::{
    Color,
    ColorChoice,
    PrinterLogMessage,
    PrinterLogMessageResult,
    fpath_to_prependname,
    fpath_to_prependpath,
};
use crate::printer::summary::{
    print_summary,
    summary_largest_evtx_event,
    summary_largest_journal_event,
    summary_longest_line_sysline,
    MapPathIdSummary,
    MapPathIdSummaryPrint,
    MapPathIdToColor,
    MapPathIdToFPath,
    MapPathIdToFileProcessingResultBlockZero,
    MapPathIdToFileType,
    MapPathIdToLogMessageType,
    MapPathIdToModifiedTime,
    MapPathIdToProcessPathResult,
    MapPathIdToProcessPathResultOrdered,
    MapPathIdToStackSize,
    SummaryPrinted,
};
use crate::readers::blockreader::{
    BlockSz,
    SummaryBlockReader,
};
use crate::readers::evtxreader::{
    EvtxReader,
    SummaryEvtxReader,
};
use crate::readers::filehandlemanager::FILE_HANDLE_MANAGER;
use crate::readers::filepreprocessor::{
    fpath_to_filetype,
    PathToFiletypeResult,
};
use crate::readers::fixedstructreader::{
    FixedStructReader,
    ResultFindFixedStruct,
    ResultFixedStructReaderNew,
};
use crate::readers::journalreader::{
    JournalOutput,
    JournalReader,
    ResultNext,
    SummaryJournalReader,
};
use crate::readers::linereader::SummaryLineReader;
use crate::readers::summary::{
    Summary,
    SummaryReaderData,
};
use crate::readers::syslinereader::{
    ResultFindSysline,
    SummarySyslineReader,
    SyslineReader,
};
use crate::readers::syslogprocessor::SummarySyslogProcessor;
use crate::tests::common::{
    path_id_generator,
    EVTX_KPNP_EVENT_COUNT,
    EVTX_KPNP_FPATH,
    FILE_UTF16BE_BOM_DTF56B_FPATH,
    FILE_UTF16BE_DTF56B_FPATH,
    FILE_UTF16LE_BOM_DTF56B_FPATH,
    FILE_UTF16LE_DTF56B_FPATH,
    FILE_UTF32BE_BOM_DTF56B_FPATH,
    FILE_UTF32BE_DTF56B_FPATH,
    FILE_UTF32LE_BOM_DTF56B_FPATH,
    FILE_UTF32LE_DTF56B_FPATH,
    FILE_UTF8_BOM_DTF56B_FPATH,
    FILE_UTF8_DTF56B_FPATH,
    FO_0,
    FO_P8,
    JOURNAL_FILE_RHE_91_SYSTEM_EVENT_COUNT,
    JOURNAL_FILE_RHE_91_SYSTEM_FPATH,
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH,
    REGEX_ID_DTF56B,
};

// XXX: copied from `syslinereader_tests.rs`
const NTF5_DATA_LINE0: &str = "[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.\n";
const NTF5_DATA_LINE1: &str = "[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!\n";
const NTF5_DATA_LINE2: &str = "[20200113-11:03:08] [INFO ] starting xrdp with pid 23198\n";
const NTF5_DATA_LINE3: &str = "[20200113-11:13:59] [DEBUG] Certification found
    FOUND CERTIFICATE!\n";
const NTF5_DATA_LINE4: &str = "[20200113-11:13:59] [DEBUG] Certification complete.\n";
const NTF5_DATA: &str = concatcp!(NTF5_DATA_LINE0, NTF5_DATA_LINE1, NTF5_DATA_LINE2, NTF5_DATA_LINE3, NTF5_DATA_LINE4,);

const REGEX_ID_NTF5: RegexId = 141;

const FT_EVTX_NORM: FileType = FileType::Evtx { archival_type: FileTypeArchive::Normal };

lazy_static! {
    static ref NTF5: NamedTempFile = create_temp_file(NTF5_DATA);
    static ref NTF5_PATH: FPath = ntf_fpath(&NTF5);
}

/// helper to `test_summary_longest_line_sysline`
fn summary_with_longest_values(
    line_longest: Count,
    sysline_longest: Count,
) -> Summary {
    Summary {
        readerdata: SummaryReaderData::Syslog((
            SummaryBlockReader::default(),
            SummaryLineReader {
                linereader_line_longest_processed: line_longest,
                ..Default::default()
            },
            SummarySyslineReader {
                syslinereader_sysline_longest: sysline_longest,
                ..Default::default()
            },
            SummarySyslogProcessor::default(),
        )),
        ..Default::default()
    }
}

/// helper to `test_summary_largest_evtx_event`
fn summary_with_largest_evtx_values(
    event_largest_processed: Count,
    event_largest_accepted: Count,
) -> Summary {
    Summary {
        readerdata: SummaryReaderData::Etvx(
            SummaryEvtxReader {
                evtxreader_event_largest_processed: event_largest_processed,
                evtxreader_event_largest_accepted: event_largest_accepted,
                ..Default::default()
            }
        ),
        ..Default::default()
    }
}

/// helper to `test_summary_largest_journal_event`
fn summary_with_largest_journal_event_values(
    journal_event_largest_processed: Count,
    journal_event_largest_accepted: Count,
) -> Summary {
    Summary {
        readerdata: SummaryReaderData::Journal(
            SummaryJournalReader {
                journalreader_journal_event_largest_processed: journal_event_largest_processed,
                journalreader_journal_event_largest_accepted: journal_event_largest_accepted,
                ..Default::default()
            }
        ),
        ..Default::default()
    }
}

#[test]
fn test_summary_largest_journal_event() {
    let mut map_pathid_summary = MapPathIdSummary::new();
    map_pathid_summary.insert(1, Summary::default());
    assert_eq!((0, 0), summary_largest_journal_event(&map_pathid_summary));

    map_pathid_summary.insert(2, summary_with_largest_journal_event_values(123, 55));
    map_pathid_summary.insert(3, summary_with_largest_journal_event_values(80, 456));
    assert_eq!((123, 456), summary_largest_journal_event(&map_pathid_summary));
}

#[test]
fn test_summary_largest_evtx_event() {
    let mut map_pathid_summary = MapPathIdSummary::new();
    map_pathid_summary.insert(1, Summary::default());
    assert_eq!((0, 0), summary_largest_evtx_event(&map_pathid_summary));

    map_pathid_summary.insert(2, summary_with_largest_evtx_values(123, 55));
    map_pathid_summary.insert(3, summary_with_largest_evtx_values(80, 456));
    assert_eq!((123, 456), summary_largest_evtx_event(&map_pathid_summary));
}

#[test]
fn test_summary_longest_line_sysline() {
    let mut map_pathid_summary = MapPathIdSummary::new();
    map_pathid_summary.insert(1, Summary::default());
    assert_eq!((0, 0), summary_longest_line_sysline(&map_pathid_summary));

    map_pathid_summary.insert(2, summary_with_longest_values(123, 55));
    map_pathid_summary.insert(3, summary_with_longest_values(80, 456));
    assert_eq!((123, 456), summary_longest_line_sysline(&map_pathid_summary));
}

#[test]
fn test_PrinterLogMessage_new() {
    PrinterLogMessage::new(
        ColorChoice::Never,
        Color::Red,
        FileTypeTextEncoding::Utf8Ascii,
        None,
        None,
        FO_0,
    );
}

fn new_SyslineReader(
    path: &FPath,
    blocksz: BlockSz,
    tzo: FixedOffset,
) -> SyslineReader {
    let result = fpath_to_filetype(path, true);
    let filetype = match result {
        PathToFiletypeResult::Filetype(ft) => ft,
        PathToFiletypeResult::Archive(..) => {
            panic!("ERROR: fpath_to_filetype({:?}) returned an PathToFiletypeResult::Archive", path);
        }
    };
    match SyslineReader::new(path_id_generator(), path.clone(), filetype, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

fn new_PrinterLogMessage(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
) -> PrinterLogMessage {
    let pf = prepend_file.map(String::from);
    let pd = prepend_date.map(String::from);

    PrinterLogMessage::new(
        colorchoice,
        color,
        FileTypeTextEncoding::Utf8Ascii,
        pf,
        pd,
        prepend_offset.unwrap_or(FO_0),
    )
}

const CCA: ColorChoice = ColorChoice::Always;
const CCU: ColorChoice = ColorChoice::Auto;
const CCN: ColorChoice = ColorChoice::Never;

const CLR: Color = Color::Green;
const FILEN: &str = "foo.log";
const DATE: &str = "20000101T000000";

#[test_case(CCA, CLR, None, None, None, 333, 37; "a")]
#[test_case(CCU, CLR, None, None, None, 333, 37; "b")]
#[test_case(CCN, CLR, None, None, None, 333, 5; "c")]
#[test_case(CCA, CLR, Some(FILEN), None, None, 382, 49; "d")]
#[test_case(CCU, CLR, None, Some(DATE), None, 438, 49; "e")]
#[test_case(CCN, CLR, None, None, Some(FO_P8), 333, 5; "f")]
#[test_case(CCA, CLR, Some(FILEN), Some(DATE), None, 487, 49; "g")]
#[test_case(CCN, CLR, None, Some(DATE), Some(FO_P8), 438, 5; "h")]
#[test_case(CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 487, 49; "i")]
#[test_case(CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 487, 49; "j")]
#[test_case(CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 487, 5; "k")]
fn test_PrinterLogMessage_print_sysline_NTF5(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    if !regex_id_compiled(REGEX_ID_NTF5) {
        eprintln!("Regex #{} not compiled; skip test", REGEX_ID_NTF5);
        return;
    }
    summary_stats_enable();
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut fo: FileOffset = 0;
    let mut slr = new_SyslineReader(&NTF5_PATH, 1024, FO_P8);
    let mut prints: usize = 0;
    let mut printed_bytes: usize = 0;
    let mut printed_flushed: usize = 0;
    loop {
        let result = slr.find_sysline(fo);
        match result {
            ResultFindSysline::Found((fo_, syslinep)) => {
                fo = fo_;
                match plm.print_sysline(&syslinep) {
                    Ok((bytes_, flushed_)) => {
                        prints += 1;
                        printed_bytes += bytes_;
                        printed_flushed += flushed_;
                    }
                    Err(err) => {
                        panic!("ERROR: plm.print_sysline({:?}) returned Err({})", fo_, err);
                    }
                }
            }
            ResultFindSysline::Done => {
                break;
            }
            ResultFindSysline::Err(err) => {
                panic!("ERROR: slr.find_sysline({}) returned Err({})", fo, err);
            }
        }
    }
    eprintln!("prints={}, bytes={}, flushes={}", prints, printed_bytes, printed_flushed);
    assert_eq!(prints, 5, "Expected 5 prints, got {}", prints);
    assert_eq!(
        printed_bytes, expected_printed_bytes,
        "Expected {} printed bytes, got {}", expected_printed_bytes, printed_bytes,
    );
    assert_eq!(
        printed_flushed, expected_flushed,
        "Expected {} flushed, got {}", expected_flushed, printed_flushed,
    );
}

const ENC_UTF8: FileTypeTextEncoding = FileTypeTextEncoding::Utf8Ascii;
const ENC_UTF8_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf8BOM;
const ENC_UTF16LE: FileTypeTextEncoding = FileTypeTextEncoding::Utf16le;
const ENC_UTF16LE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf16leBOM;
const ENC_UTF16BE: FileTypeTextEncoding = FileTypeTextEncoding::Utf16be;
const ENC_UTF16BE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf16beBOM;
const ENC_UTF32LE: FileTypeTextEncoding = FileTypeTextEncoding::Utf32le;
const ENC_UTF32LE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf32leBOM;
const ENC_UTF32BE: FileTypeTextEncoding = FileTypeTextEncoding::Utf32be;
const ENC_UTF32BE_BOM: FileTypeTextEncoding = FileTypeTextEncoding::Utf32beBOM;

#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCA, CLR, None, None, None, 6, 249, 42; "UTF8 a")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCU, CLR, None, None, None, 6, 249, 42; "UTF8 b")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCN, CLR, None, None, None, 6, 249, 6; "UTF8 c")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCA, CLR, Some(FILEN), None, None, 6, 333, 67; "UTF8 d")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCU, CLR, None, Some(DATE), None, 6, 429, 67; "UTF8 e")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCN, CLR, None, None, Some(FO_P8), 6, 249, 6; "UTF8 f")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 513, 67; "UTF8 g")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCN, CLR, None, Some(DATE), Some(FO_P8), 6, 429, 6; "UTF8 h")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 67; "UTF8 i")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 67; "UTF8 j")]
#[test_case(&FILE_UTF8_DTF56B_FPATH, ENC_UTF8, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 6; "UTF8 k")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCA, CLR, None, None, None, 6, 249, 42; "UTF8 BOM a")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCU, CLR, None, None, None, 6, 249, 42; "UTF8 BOM b")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCN, CLR, None, None, None, 6, 249, 6; "UTF8 BOM c")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCA, CLR, Some(FILEN), None, None, 6, 333, 67; "UTF8 BOM d")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCU, CLR, None, Some(DATE), None, 6, 429, 67; "UTF8 BOM e")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCN, CLR, None, None, Some(FO_P8), 6, 249, 6; "UTF8 BOM f")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 513, 67; "UTF8 BOM g")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 67; "UTF8 BOM h")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 6; "UTF8 BOM i")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 67; "UTF8 BOM j")]
#[test_case(&FILE_UTF8_BOM_DTF56B_FPATH, ENC_UTF8_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 513, 6; "UTF8 BOM k")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCA, CLR, None, None, None, 6, 498, 42; "UTF16BE BOM a")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCU, CLR, None, None, None, 6, 498, 42; "UTF16BE BOM b")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCN, CLR, None, None, None, 6, 498, 6; "UTF16BE BOM c")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCA, CLR, Some(FILEN), None, None, 6, 582, 67; "UTF16BE BOM d")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCU, CLR, None, Some(DATE), None, 6, 678, 67; "UTF16BE BOM e")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCN, CLR, None, None, Some(FO_P8), 6, 498, 6; "UTF16BE BOM f")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 762, 67; "UTF16BE BOM g")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16BE BOM h")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCN, CLR, None, Some(DATE), Some(FO_P8), 6, 678, 6; "UTF16BE BOM i")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16BE BOM j")]
#[test_case(&FILE_UTF16BE_BOM_DTF56B_FPATH, ENC_UTF16BE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16BE BOM k")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCA, CLR, None, None, None, 6, 498, 42; "UTF16BE a")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCU, CLR, None, None, None, 6, 498, 42; "UTF16BE b")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCN, CLR, None, None, None, 6, 498, 6; "UTF16BE c")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCA, CLR, Some(FILEN), None, None, 6, 582, 67; "UTF16BE d")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCU, CLR, None, Some(DATE), None, 6, 678, 67; "UTF16BE e")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCN, CLR, None, None, Some(FO_P8), 6, 498, 6; "UTF16BE f")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 762, 67; "UTF16BE g")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16BE h")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16BE i")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16BE j")]
#[test_case(&FILE_UTF16BE_DTF56B_FPATH, ENC_UTF16BE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16BE k")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCA, CLR, None, None, None, 6, 498, 42; "UTF16LE BOM a")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCU, CLR, None, None, None, 6, 498, 42; "UTF16LE BOM b")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCN, CLR, None, None, None, 6, 498, 6; "UTF16LE BOM c")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCA, CLR, Some(FILEN), None, None, 6, 582, 67; "UTF16LE BOM d")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCU, CLR, None, Some(DATE), None, 6, 678, 67; "UTF16LE BOM e")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCN, CLR, None, None, Some(FO_P8), 6, 498, 6; "UTF16LE BOM f")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 762, 67; "UTF16LE BOM g")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16LE BOM h")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16LE BOM i")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16LE BOM j")]
#[test_case(&FILE_UTF16LE_BOM_DTF56B_FPATH, ENC_UTF16LE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16LE BOM k")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCA, CLR, None, None, None, 6, 498, 42; "UTF16LE a")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCU, CLR, None, None, None, 6, 498, 42; "UTF16LE b")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCN, CLR, None, None, None, 6, 498, 6; "UTF16LE c")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCA, CLR, Some(FILEN), None, None, 6, 582, 67; "UTF16LE d")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCU, CLR, None, Some(DATE), None, 6, 678, 67; "UTF16LE e")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCN, CLR, None, None, Some(FO_P8), 6, 498, 6; "UTF16LE f")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 762, 67; "UTF16LE g")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16LE h")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16LE i")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 67; "UTF16LE j")]
#[test_case(&FILE_UTF16LE_DTF56B_FPATH, ENC_UTF16LE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 762, 6; "UTF16LE k")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCA, CLR, None, None, None, 6, 998, 42; "UTF32LE BOM a")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCU, CLR, None, None, None, 6, 998, 42; "UTF32LE BOM b")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCN, CLR, None, None, None, 6, 998, 6; "UTF32LE BOM c")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCA, CLR, Some(FILEN), None, None, 6, 1082, 67; "UTF32LE BOM d")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCU, CLR, None, Some(DATE), None, 6, 1178, 67; "UTF32LE BOM e")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCN, CLR, None, None, Some(FO_P8), 6, 998, 6; "UTF32LE BOM f")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 1262, 67; "UTF32LE BOM g")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1262, 67; "UTF32LE BOM h")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1262, 6; "UTF32LE BOM i")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1262, 67; "UTF32LE BOM j")]
#[test_case(&FILE_UTF32LE_BOM_DTF56B_FPATH, ENC_UTF32LE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1262, 6; "UTF32LE BOM k")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCA, CLR, None, None, None, 6, 996, 42; "UTF32LE a")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCU, CLR, None, None, None, 6, 996, 42; "UTF32LE b")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCN, CLR, None, None, None, 6, 996, 6; "UTF32LE c")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCA, CLR, Some(FILEN), None, None, 6, 1080, 67; "UTF32LE d")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCU, CLR, None, Some(DATE), None, 6, 1176, 67; "UTF32LE e")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCN, CLR, None, None, Some(FO_P8), 6, 996, 6; "UTF32LE f")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 1260, 67; "UTF32LE g")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 67; "UTF32LE h")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 6; "UTF32LE i")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 67; "UTF32LE j")]
#[test_case(&FILE_UTF32LE_DTF56B_FPATH, ENC_UTF32LE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 6; "UTF32LE k")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCA, CLR, None, None, None, 6, 996, 42; "UTF32BE BOM a")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCU, CLR, None, None, None, 6, 996, 42; "UTF32BE BOM b")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCN, CLR, None, None, None, 6, 996, 6; "UTF32BE BOM c")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCA, CLR, Some(FILEN), None, None, 6, 1080, 67; "UTF32BE BOM d")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCU, CLR, None, Some(DATE), None, 6, 1176, 67; "UTF32BE BOM e")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCN, CLR, None, None, Some(FO_P8), 6, 996, 6; "UTF32BE BOM f")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 1260, 67; "UTF32BE BOM g")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 67; "UTF32BE BOM h")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 6; "UTF32BE BOM i")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 67; "UTF32BE BOM j")]
#[test_case(&FILE_UTF32BE_BOM_DTF56B_FPATH, ENC_UTF32BE_BOM, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 6; "UTF32BE BOM k")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCA, CLR, None, None, None, 6, 996, 42; "UTF32BE a")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCU, CLR, None, None, None, 6, 996, 42; "UTF32BE b")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCN, CLR, None, None, None, 6, 996, 6; "UTF32BE c")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCA, CLR, Some(FILEN), None, None, 6, 1080, 67; "UTF32BE d")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCU, CLR, None, Some(DATE), None, 6, 1176, 67; "UTF32BE e")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCN, CLR, None, None, Some(FO_P8), 6, 996, 6; "UTF32BE f")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCA, CLR, Some(FILEN), Some(DATE), None, 6, 1260, 67; "UTF32BE g")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCU, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 67; "UTF32BE h")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 6; "UTF32BE i")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCA, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 67; "UTF32BE j")]
#[test_case(&FILE_UTF32BE_DTF56B_FPATH, ENC_UTF32BE, CCN, CLR, Some(FILEN), Some(DATE), Some(FO_P8), 6, 1260, 6; "UTF32BE k")]
fn test_PrinterLogMessage_print_sysline_UTF(
    fpath: &FPath,
    encoding_type: FileTypeTextEncoding,
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    expected_prints: usize,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    if !regex_id_compiled(REGEX_ID_DTF56B) {
        eprintln!("Regex #{} not compiled; skip test", REGEX_ID_DTF56B);
        return;
    }
    summary_stats_enable();
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut fo: FileOffset = 0;
    let mut slr = new_SyslineReader(fpath, 1024, FO_P8);
    slr.filetype_text_encoding_update(encoding_type);
    let mut prints: usize = 0;
    let mut printed_bytes: usize = 0;
    let mut printed_flushed: usize = 0;
    loop {
        let result = slr.find_sysline(fo);
        match result {
            ResultFindSysline::Found((fo_, syslinep)) => {
                fo = fo_;
                match plm.print_sysline(&syslinep) {
                    Ok((bytes_, flushed_)) => {
                        prints += 1;
                        printed_bytes += bytes_;
                        printed_flushed += flushed_;
                    }
                    Err(err) => {
                        panic!("ERROR: plm.print_sysline({:?}) returned Err({})", fo_, err);
                    }
                }
            }
            ResultFindSysline::Done => {
                break;
            }
            ResultFindSysline::Err(err) => {
                panic!("ERROR: slr.find_sysline({}) returned Err({})", fo, err);
            }
        }
    }
    eprintln!("prints={}, bytes={}, flushes={}", prints, printed_bytes, printed_flushed);
    assert_eq!(prints, expected_prints, "Expected {} prints, got {}", expected_prints, prints);
    assert_eq!(
        printed_bytes, expected_printed_bytes,
        "Expected {} printed bytes, got {}", expected_printed_bytes, printed_bytes,
    );
    assert_eq!(
        printed_flushed, expected_flushed,
        "Expected {} flushed, got {}", expected_flushed, printed_flushed,
    );
}

const FILEU: &str = "foo.utmp";

#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, None, None, None, 378, 14; "a")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, None, None, None, 378, 14; "b")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, None, None, 378, 2; "c")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, Some(FILEU), None, None, 394, 17; "d")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, None, Some(DATE), None, 408, 17; "e")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, None, Some(FO_P8), 378, 2; "f")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, Some(FILEU), Some(DATE), None, 424, 17; "g")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, Some(FILEU), Some(DATE), Some(FO_P8), 424, 17; "h")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, Some(DATE), Some(FO_P8), 408, 2; "i")]
#[test_case(&*NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH, 1, CCN, CLR, None, Some(DATE), Some(FO_P8), 71, 1; "j")]
fn test_PrinterLogMessage_print_fixedstruct(
    path: &FPath,
    print_count_expect: usize,
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    summary_stats_enable();
    let mut plm: PrinterLogMessage = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let buffer: &mut [u8] = &mut [0; ENTRY_SZ_MAX];
    let mut fixedstructreader = match FixedStructReader::new(
        path_id_generator(),
        path.clone(),
        FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: FileTypeFixedStruct::Utmpx },
        ENTRY_SZ_MAX as BlockSz,
        FO_P8,
        None,
        None,
    ) {
        ResultFixedStructReaderNew::FileOk(val) => val,
        _ => panic!("ERROR: FixedStructReader::new() failed"),
    };
    let mut fo: FileOffset = fixedstructreader.fileoffset_first().unwrap();
    let mut prints: usize = 0;
    let mut printed_bytes: usize = 0;
    let mut printed_flushed: usize = 0;
    loop {
        let (fo_next, fs) = match fixedstructreader.process_entry_at(fo, buffer) {
            ResultFindFixedStruct::Found((fo_, fixedstruct_)) => {
                defo!("ResultFindFixedStruct::Found({}, …)", fo_);
                (fo_, fixedstruct_)
            }
            ResultFindFixedStruct::Done => {
                defo!("ResultFindFixedStruct::Done");
                break;
            }
            ResultFindFixedStruct::Err((_fo_opt, err)) => {
                panic!("ResultFindFixedStruct::Err({})", err);
            }
        };
        fo = fo_next;
        match plm.print_fixedstruct(&fs, buffer) {
            Ok((bytes_, flushed_)) => {
                prints += 1;
                printed_bytes += bytes_;
                printed_flushed += flushed_;
            }
            Err(err) => {
                panic!("ERROR: plm.print_fixedstruct({:?}, …) returned Err({})", fs, err);
            }
        }
    }
    eprintln!("prints={}, bytes={}, flushes={}", prints, printed_bytes, printed_flushed);
    assert_eq!(
        prints, print_count_expect, "Expected {} prints, got {}",
        print_count_expect, prints,
    );
    assert_eq!(
        printed_bytes, expected_printed_bytes,
        "Expected {} printed bytes, got {}", expected_printed_bytes, printed_bytes,
    );
    assert_eq!(
        printed_flushed, expected_flushed,
        "Expected {} flushed, got {}", expected_flushed, printed_flushed,
    );
}

#[test_case(CCA, CLR, None, None, None, 321782, 1589; "a")]
#[test_case(CCU, CLR, None, None, None, 321782, 1589; "b")]
#[test_case(CCN, CLR, None, None, None, 321782, 227; "c")]
#[test_case(CCA, CLR, Some(FILEU), None, None, 388782, 34409; "d")]
#[test_case(CCU, CLR, None, Some(DATE), None, 447407, 34409; "e")]
#[test_case(CCN, CLR, None, None, Some(FO_P8), 321782, 227; "f")]
#[test_case(CCA, CLR, Some(FILEU), Some(DATE), None, 514407, 34409; "g")]
#[test_case(CCU, CLR, Some(FILEU), Some(DATE), Some(FO_P8), 514407, 34409; "h")]
#[test_case(CCN, CLR, None, Some(DATE), Some(FO_P8), 447407, 339; "i")]
fn test_PrinterLogMessage_print_evtx(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    summary_stats_enable();
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut er = EvtxReader::new(
        path_id_generator(),
        EVTX_KPNP_FPATH.clone(),
        FT_EVTX_NORM,
        FO_P8,
    ).unwrap();
    let mut prints: usize = 0;
    let mut printed_bytes: usize = 0;
    let mut printed_flushed: usize = 0;
    er.analyze(&None, &None);
    while let Some(evtx) = er.next() {
        match plm.print_evtx(&evtx) {
            Ok((bytes_, flushed_)) => {
                prints += 1;
                printed_bytes += bytes_;
                printed_flushed += flushed_;
            }
            Err(err) => {
                panic!("ERROR: plm.print_evtx({:?}) returned Err({})", evtx, err);
            }
        }
    }
    eprintln!("prints={}, bytes={}, flushes={}", prints, printed_bytes, printed_flushed);
    let expect_prints: usize = *EVTX_KPNP_EVENT_COUNT as usize;
    assert_eq!(prints, expect_prints,
        "Expected {} prints, got {}", expect_prints, prints);
    assert_eq!(
        printed_bytes, expected_printed_bytes,
        "Expected {} printed bytes, got {}", expected_printed_bytes, printed_bytes,
    );
    assert_eq!(
        printed_flushed, expected_flushed,
        "Expected {} flushed, got {}", expected_flushed, printed_flushed,
    );
}

const FILEJ: &str = "foo.JOURNAL";

// TODO: [2026/04] the `JOURNAL_FILE_RHE_91_SYSTEM_FPATH` has *way* too much
//        data and it's all printed; test with a .journal with less data

#[test_case(CCA, CLR, None, None, None, JournalOutput::Short, 197149, 12486; "a")]
#[test_case(CCU, CLR, None, None, None, JournalOutput::ShortPrecise, 211716, 12486; "b")]
#[test_case(CCN, CLR, None, None, None, JournalOutput::ShortIso, 205473, 2081; "c")]
#[test_case(CCA, CLR, Some(FILEJ), None, None, JournalOutput::ShortIsoPrecise, 253424, 14600; "d")]
#[test_case(CCU, CLR, None, Some(DATE), None, JournalOutput::ShortFull, 259699, 14600; "e")]
#[test_case(CCN, CLR, None, None, Some(FO_P8), JournalOutput::ShortMonotonic, 195068, 2081; "f")]
#[test_case(CCA, CLR, Some(FILEJ), Some(DATE), None, JournalOutput::ShortUnix, 255625, 14600; "g")]
#[test_case(CCU, CLR, Some(FILEJ), Some(DATE), Some(FO_P8), JournalOutput::Verbose, 3234685, 205840; "h")]
#[test_case(CCN, CLR, None, Some(DATE), Some(FO_P8), JournalOutput::Export, 2574170, 2085; "i")]
#[test_case(CCN, CLR, None, Some(DATE), Some(FO_P8), JournalOutput::Cat, 154102, 2081; "j")]
fn test_PrinterLogMessage_print_journal(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    journal_output: JournalOutput,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    if !cfg!(target_os = "linux") {
        return;
    }
    summary_stats_enable();
    load_library_systemd().is_ok();
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut jr: JournalReader = JournalReader::new(
        path_id_generator(),
        (*JOURNAL_FILE_RHE_91_SYSTEM_FPATH).clone(),
        journal_output,
        FO_0,
        FileType::Journal{ archival_type: FileTypeArchive::Normal },
    ).unwrap();
    let mut prints: usize = 0;
    let mut printed_bytes: usize = 0;
    let mut printed_flushed: usize = 0;
    assert!(jr.analyze(&None).is_ok());
    loop {
        match jr.next(&None) {
            ResultNext::Done => {
                break;
            }
            ResultNext::Err(err) => {
                panic!("ERROR: jr.next() returned Err({})", err);
            }
            ResultNext::ErrIgnore(err) => {
                panic!("ERROR: jr.next() returned ErrIgnore({})", err);
            }
            ResultNext::Found(journal_entry) => {
                match plm.print_journalentry(&journal_entry) {
                    PrinterLogMessageResult::Ok((printed, flushed)) => {
                        prints += 1;
                        printed_bytes += printed;
                        printed_flushed += flushed;
                    }
                    _ => {
                        panic!(
                            "ERROR: plm.print_journalentry({:?}) returned ResultNext::Done",
                            journal_entry,
                        );
                    }
                }
            }
        }
    }
    eprintln!("prints={}, bytes={}, flushes={}", prints, printed_bytes, printed_flushed);
    let expect_prints: usize = *JOURNAL_FILE_RHE_91_SYSTEM_EVENT_COUNT as usize;
    assert_eq!(
        prints, expect_prints,
        "Expected {} prints, got {}", expect_prints, prints,
    );
    assert_eq!(
        printed_bytes, expected_printed_bytes,
        "Expected {} printed bytes, got {}", expected_printed_bytes, printed_bytes,
    );
    assert_eq!(
        printed_flushed, expected_flushed,
        "Expected {} flushed, got {}", expected_flushed, printed_flushed,
    );
}

#[test]
fn test_print_summary_empty() {
    summary_stats_enable();
    let utc_ = Utc::now();
    let local_ = Local::now();
    print_summary(
        MapPathIdToProcessPathResult::new(),
        MapPathIdToProcessPathResultOrdered::new(),
        MapPathIdToFPath::new(),
        MapPathIdToModifiedTime::new(),
        MapPathIdToFileProcessingResultBlockZero::new(),
        MapPathIdToFileType::new(),
        MapPathIdToStackSize::new(),
        MapPathIdToLogMessageType::new(),
        MapPathIdToColor::new(),
        MapPathIdSummary::new(),
        MapPathIdSummaryPrint::new(),
        FILE_HANDLE_MANAGER.summary(),
        ColorChoice::Always,
        Color::Black,
        0,
        SetPathId::new(),
        SummaryPrinted::new(LogMessageType::All),
        &None,
        &None,
        &local_,
        &utc_,
        0,
        0,
        std::time::Instant::now(),
        0,
        1,
        0,
        AllocatorChosen::System,
    );
}

#[test_case("bar", "bar")]
#[test_case("/foo/bar", "bar")]
#[test_case("/foo/bar/baz", "baz")]
#[test_case("/foo/bar.tar\0baz", "baz")]
#[test_case("/foo/bar.tar\0b\0az", "b\0az")]
fn test_fpath_to_prependname(
    path_s: &str,
    expect: &str,
) {
    const S: char = std::path::MAIN_SEPARATOR;
    let fpath: FPath = FPath::from(path_s).replace("/", &S.to_string());
    let result: FPath = fpath_to_prependname(&fpath);
    assert_eq!(
        result,
        expect,
        "\nGiven    {:?}\nExpected {:?}\nActual   {:?}\n",
        path_s,
        expect,
        result
    );
}

#[test_case("bar", "bar")]
#[test_case("/foo/bar", "/foo/bar")]
#[test_case("/foo/bar/baz", "/foo/bar/baz")]
#[test_case("/foo/bar.tar\0baz", "/foo/bar.tar|baz")]
#[test_case("/foo/bar.tar\0b\0az", "/foo/bar.tar|b\0az")]
fn test_fpath_to_prependpath(
    path_s: &str,
    expect: &str,
) {
    let fpath: FPath = FPath::from(path_s);
    let result: FPath = fpath_to_prependpath(&fpath);
    assert_eq!(
        result,
        expect,
        "\nGiven    {:?}\nExpected {:?}\nActual   {:?}\n",
        path_s,
        expect,
        result
    );
}
