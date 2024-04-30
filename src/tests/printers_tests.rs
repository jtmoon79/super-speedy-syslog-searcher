// src/tests/printers_tests.rs

//! tests for [`src/printer/printers.rs`]
//!
//! [`src/printer/printers.rs`]: ../../printer/printers.rs

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
use crate::printer::printers::{
    Color,
    ColorChoice,
    PrinterLogMessage,
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
use crate::readers::syslinereader::{ResultS3SyslineFind, SyslineReader};
use crate::tests::common::{
    FO_0,
    FO_P8,
    NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH,
    NTF_LINUX_X86_UTMPX_2ENTRY_FPATH,
    EVTX_KPNP_FPATH,
    EVTX_KPNP_EVENT_COUNT,
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
        PathToFiletypeResult::Archive(_) => {
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

#[test_case(CCA, CLR, None, None, None; "a")]
#[test_case(CCU, CLR, None, None, None; "b")]
#[test_case(CCN, CLR, None, None, None; "c")]
#[test_case(CCA, CLR, Some(FILEN), None, None; "d")]
#[test_case(CCU, CLR, None, Some(DATE), None; "e")]
#[test_case(CCN, CLR, None, None, Some(*FO_P8); "f")]
#[test_case(CCA, CLR, Some(FILEN), Some(DATE), None; "g")]
#[test_case(CCU, CLR, Some(FILEN), Some(DATE), Some(*FO_P8); "h")]
#[test_case(CCN, CLR, None, Some(DATE), Some(*FO_P8); "i")]
fn test_PrinterLogMessage_print_sysline_NTF5(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
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
    loop {
        let result = slr.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, syslinep)) => {
                fo = fo_;
                match plm.print_sysline(&syslinep) {
                    Ok(_) => {
                        prints += 1;
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
    assert_eq!(prints, 5, "Expected 5 prints, got {}", prints);
}

const FILEU: &str = "foo.utmp";

#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, None, None, None; "u_a")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, None, None, None; "u_b")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, None, None; "u_c")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, Some(FILEU), None, None; "u_d")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, None, Some(DATE), None; "u_e")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, None, Some(*FO_P8); "u_f")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCA, CLR, Some(FILEU), Some(DATE), None; "u_g")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCU, CLR, Some(FILEU), Some(DATE), Some(*FO_P8); "u_h")]
#[test_case(&*NTF_LINUX_X86_UTMPX_2ENTRY_FPATH, 2, CCN, CLR, None, Some(DATE), Some(*FO_P8); "u_i")]
#[test_case(&*NTF_LINUX_X86_LASTLOG_1ENTRY_FPATH, 1, CCN, CLR, None, Some(DATE), Some(*FO_P8); "l_i")]
fn test_PrinterLogMessage_print_fixedstruct(
    path: &FPath,
    print_count_expect: usize,
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
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
            Ok(_) => {
                prints += 1;
            }
            Err(err) => {
                panic!("ERROR: plm.print_fixedstruct({:?}, …) returned Err({})", fs, err);
            }
        }
    }

    assert_eq!(
        prints, print_count_expect, "Expected {} prints, got {}",
        print_count_expect, prints,
    );
}

#[test_case(CCA, CLR, None, None, None; "u_a")]
#[test_case(CCU, CLR, None, None, None; "u_b")]
#[test_case(CCN, CLR, None, None, None; "u_c")]
#[test_case(CCA, CLR, Some(FILEU), None, None; "u_d")]
#[test_case(CCU, CLR, None, Some(DATE), None; "u_e")]
#[test_case(CCN, CLR, None, None, Some(*FO_P8); "u_f")]
#[test_case(CCA, CLR, Some(FILEU), Some(DATE), None; "u_g")]
#[test_case(CCU, CLR, Some(FILEU), Some(DATE), Some(*FO_P8); "u_h")]
#[test_case(CCN, CLR, None, Some(DATE), Some(*FO_P8); "u_i")]
fn test_PrinterLogMessage_print_evtx(
    colorchoice: ColorChoice,
    color: Color,
    prepend_file: Option<&str>,
    prepend_date: Option<&str>,
    prepend_offset: Option<FixedOffset>,
) {
    let mut plm = new_PrinterLogMessage(
        colorchoice,
        color,
        prepend_file,
        prepend_date,
        prepend_offset,
    );

    let mut er = EvtxReader::new(EVTX_KPNP_FPATH.clone()).unwrap();
    let mut prints: usize = 0;
    er.analyze(&None, &None);
    while let Some(evtx) = er.next() {
        match plm.print_evtx(&evtx) {
            Ok(_) => {
                prints += 1;
            }
            Err(err) => {
                panic!("ERROR: plm.print_evtx({:?}) returned Err({})", evtx, err);
            }
        }
    }
    let expect_prints: usize = *EVTX_KPNP_EVENT_COUNT as usize;
    assert_eq!(prints, expect_prints,
        "Expected {} prints, got {}", expect_prints, prints);
}
