// src/tests/printers_tests.rs

//! tests for [`src/printer/printers.rs`]
//!
//! [`src/printer/printers.rs`]: crate::printer::printers

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use crate::common::{
    FileOffset,
    FileType,
    FileTypeArchive,
    FileTypeFixedStruct,
    FPath,
};
use crate::debug::helpers::{
    create_temp_file,
    ntf_fpath,
    NamedTempFile,
};
use crate::data::datetime::FixedOffset;
use crate::data::fixedstruct::ENTRY_SZ_MAX;
use crate::libload::systemd_dlopen2::load_library_systemd;
use crate::printer::printers::{
    Color,
    ColorChoice,
    PrinterLogMessage,
    PrinterLogMessageResult,
};
use crate::readers::blockreader::BlockSz;
use crate::readers::evtxreader::EvtxReader;
use crate::readers::filepreprocessor::{
    fpath_to_filetype,
    PathToFiletypeResult,
};
use crate::readers::fixedstructreader::{
    FixedStructReader,
    ResultFixedStructReaderNew,
    ResultS3FixedStructFind,
};
use crate::readers::journalreader::{
    JournalOutput,
    JournalReader,
    ResultNext,
};
use crate::readers::syslinereader::{ResultS3SyslineFind, SyslineReader};
use crate::tests::common::{
    FO_0,
    FO_P8,
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH,
    EVTX_KPNP_FPATH,
    EVTX_KPNP_EVENT_COUNT,
    JOURNAL_FILE_RHE_91_SYSTEM_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_EVENT_COUNT,
};

use ::const_format::concatcp;
use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::si_trace_print::printers::{defn, defo, defx};
use ::test_case::test_case;


// XXX: copied from `syslinereader_tests.rs`
const NTF5_DATA_LINE0: &str =
    "[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.\n";
const NTF5_DATA_LINE1: &str = "[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!\n";
const NTF5_DATA_LINE2: &str = "[20200113-11:03:08] [INFO ] starting xrdp with pid 23198\n";
const NTF5_DATA_LINE3: &str = "[20200113-11:13:59] [DEBUG] Certification found
    FOUND CERTIFICATE!\n";
const NTF5_DATA_LINE4: &str = "[20200113-11:13:59] [DEBUG] Certification complete.\n";
const NTF5_DATA: &str =
    concatcp!(NTF5_DATA_LINE0, NTF5_DATA_LINE1, NTF5_DATA_LINE2, NTF5_DATA_LINE3, NTF5_DATA_LINE4,);

const FT_EVTX_NORM: FileType = FileType::Evtx { archival_type: FileTypeArchive::Normal };

lazy_static! {
    static ref NTF5: NamedTempFile = create_temp_file(NTF5_DATA);
    static ref NTF5_PATH: FPath = ntf_fpath(&NTF5);
}

#[test]
fn test_PrinterLogMessage_new() {
    PrinterLogMessage::new(
        ColorChoice::Never,
        Color::Red,
        None,
        None,
        *FO_0,
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
    match SyslineReader::new(path.clone(), filetype, blocksz, tzo) {
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
) -> PrinterLogMessage{
    let pf = prepend_file.map(String::from);
    let pd = prepend_date.map(String::from);

    PrinterLogMessage::new(
        colorchoice,
        color,
        pf,
        pd,
        prepend_offset.unwrap_or(*FO_0),
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
#[test_case(CCN, CLR, None, None, Some(*FO_P8), 333, 5; "f")]
#[test_case(CCA, CLR, Some(FILEN), Some(DATE), None, 487, 49; "g")]
#[test_case(CCN, CLR, None, Some(DATE), Some(*FO_P8), 438, 5; "h")]
#[test_case(CCA, CLR, Some(FILEN), Some(DATE), Some(*FO_P8), 487, 49; "i")]
#[test_case(CCU, CLR, Some(FILEN), Some(DATE), Some(*FO_P8), 487, 49; "j")]
#[test_case(CCN, CLR, Some(FILEN), Some(DATE), Some(*FO_P8), 487, 5; "k")]
fn test_PrinterLogMessage_print_sysline_NTF5(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut fo: FileOffset = 0;
    let mut slr = new_SyslineReader(&NTF5_PATH, 1024, *FO_P8);
    let mut prints: usize = 0;
    let mut printed_bytes: usize = 0;
    let mut printed_flushed: usize = 0;
    loop {
        let result = slr.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, syslinep)) => {
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
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
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

const FILEU: &str = "foo.utmp";

#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, None, None, None, 378, 14; "a")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, None, None, None, 378, 14; "b")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, None, None, 378, 2; "c")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, Some(FILEU), None, None, 394, 17; "d")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, None, Some(DATE), None, 408, 17; "e")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, None, Some(*FO_P8), 378, 2; "f")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, Some(FILEU), Some(DATE), None, 424, 17; "g")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, Some(FILEU), Some(DATE), Some(*FO_P8), 424, 17; "h")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, Some(DATE), Some(*FO_P8), 408, 2; "i")]
#[test_case(&*NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH, 1, CCN, CLR, None, Some(DATE), Some(*FO_P8), 71, 1; "j")]
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
    let mut plm: PrinterLogMessage = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut buffer: &mut [u8] = &mut [0; ENTRY_SZ_MAX];
    let mut fixedstructreader = match FixedStructReader::new(
        path.clone(),
        FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: FileTypeFixedStruct::Utmpx },
        ENTRY_SZ_MAX as BlockSz,
        *FO_P8,
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
        let (fo_next, fs) = match fixedstructreader.process_entry_at(fo, &mut buffer) {
            ResultS3FixedStructFind::Found((fo_, fixedstruct_)) => {
                defo!("ResultS3FixedStructFind::Found({}, …)", fo_);
                (fo_, fixedstruct_)
            }
            ResultS3FixedStructFind::Done => {
                defo!("ResultS3FixedStructFind::Done");
                break;
            }
            ResultS3FixedStructFind::Err((_fo_opt, err)) => {
                panic!("ResultS3FixedStructFind::Err({})", err);
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
#[test_case(CCN, CLR, None, None, Some(*FO_P8), 321782, 227; "f")]
#[test_case(CCA, CLR, Some(FILEU), Some(DATE), None, 514407, 34409; "g")]
#[test_case(CCU, CLR, Some(FILEU), Some(DATE), Some(*FO_P8), 514407, 34409; "h")]
#[test_case(CCN, CLR, None, Some(DATE), Some(*FO_P8), 447407, 339; "i")]
fn test_PrinterLogMessage_print_evtx(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
    expected_printed_bytes: usize,
    expected_flushed: usize,
) {
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut er = EvtxReader::new(
        EVTX_KPNP_FPATH.clone(),
        FT_EVTX_NORM,
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

#[test_case(CCA, CLR, None, None, None, JournalOutput::Short, 197149, 12486; "a")]
#[test_case(CCU, CLR, None, None, None, JournalOutput::ShortPrecise, 211716, 12486; "b")]
#[test_case(CCN, CLR, None, None, None, JournalOutput::ShortIso, 205473, 2081; "c")]
#[test_case(CCA, CLR, Some(FILEJ), None, None, JournalOutput::ShortIsoPrecise, 253424, 14600; "d")]
#[test_case(CCU, CLR, None, Some(DATE), None, JournalOutput::ShortFull, 259699, 14600; "e")]
#[test_case(CCN, CLR, None, None, Some(*FO_P8), JournalOutput::ShortMonotonic, 195068, 2081; "f")]
#[test_case(CCA, CLR, Some(FILEJ), Some(DATE), None, JournalOutput::ShortUnix, 255625, 14600; "g")]
#[test_case(CCU, CLR, Some(FILEJ), Some(DATE), Some(*FO_P8), JournalOutput::Verbose, 3234685, 205840; "h")]
#[test_case(CCN, CLR, None, Some(DATE), Some(*FO_P8), JournalOutput::Export, 2574170, 2085; "i")]
#[test_case(CCN, CLR, None, Some(DATE), Some(*FO_P8), JournalOutput::Cat, 154102, 2081; "j")]
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
    load_library_systemd().is_ok();
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut jr: JournalReader = JournalReader::new(
        (*JOURNAL_FILE_RHE_91_SYSTEM_FPATH).clone(),
        journal_output,
        *FO_0,
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

}
