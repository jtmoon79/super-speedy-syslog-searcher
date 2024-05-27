// src/tests/evtxreader_tests.rs

//! tests for `evtxreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use crate::common::{
    Count,
    FileSz,
    FileType,
    FileTypeArchive,
    FPath,
    LogMessageType,
};
use crate::data::datetime::DateTimeLOpt;
use crate::data::common::DtBegEndPairOpt;
use crate::data::evtx::Evtx;
use crate::readers::summary::SummaryReaderData;
use crate::readers::evtxreader::EvtxReader;
use crate::tests::common::{
    NTF_LOG_EMPTY_FPATH,
    EVTX_NE_FPATH,
    EVTX_KPNP_FPATH,
    EVTX_KPNP_ENTRY1_DT,
    EVTX_KPNP_ENTRY227_DT,
    EVTX_KPNP_EVENT_COUNT,
    EVTX_KPNP_DATA1_S,
    EVTX_KPNP_GZ_FPATH,
    EVTX_KPNP_GZ_ENTRY1_DT,
    EVTX_KPNP_GZ_ENTRY227_DT,
    EVTX_KPNP_GZ_EVENT_COUNT,
    EVTX_KPNP_LZ4_FPATH,
    EVTX_KPNP_LZ4_ENTRY1_DT,
    EVTX_KPNP_LZ4_ENTRY227_DT,
    EVTX_KPNP_LZ4_EVENT_COUNT,
    EVTX_KPNP_TAR_FPATH,
    EVTX_KPNP_TAR_ENTRY1_DT,
    EVTX_KPNP_TAR_ENTRY227_DT,
    EVTX_KPNP_TAR_EVENT_COUNT,
    EVTX_KPNP_XZ_FPATH,
    EVTX_KPNP_XZ_ENTRY1_DT,
    EVTX_KPNP_XZ_ENTRY227_DT,
    EVTX_KPNP_XZ_EVENT_COUNT,
};

use ::criterion::black_box;
use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::more_asserts::{assert_gt, assert_ge};
#[allow(unused_imports)]
use ::si_trace_print::printers::{defn, defo, defx, defñ};
#[allow(unused_imports)]
use ::si_trace_print::stack::stack_offset_set;
use ::test_case::test_case;


/// Error, broken data
pub const EVTX_KPNP_DATA1_S_E: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <TimeCreated SystemTime="2023-03-10T03:49:43"#;

lazy_static! {
    /// EVTX #1
    static ref EVTX_1: Evtx = Evtx::new_(
        1,
        *EVTX_KPNP_ENTRY1_DT,
        String::from(EVTX_KPNP_DATA1_S),
        DtBegEndPairOpt::Some((420, 447)),
    );
}

const FT_NORM: FileType = FileType::Evtx {
    archival_type: FileTypeArchive::Normal,
};
const FT_GZ: FileType = FileType::Evtx {
    archival_type: FileTypeArchive::Gz,
};
const FT_LZ4: FileType = FileType::Evtx {
    archival_type: FileTypeArchive::Lz4,
};
const FT_TAR: FileType = FileType::Evtx {
    archival_type: FileTypeArchive::Tar,
};
const FT_XZ: FileType = FileType::Evtx {
    archival_type: FileTypeArchive::Xz,
};

/// test creating a new `EvtxReader`
#[test_case(&EVTX_NE_FPATH, true)]
#[test_case(&EVTX_KPNP_FPATH, true)]
#[test_case(&NTF_LOG_EMPTY_FPATH, false)]
#[test_case(&FPath::from("BAD PATH"), false)]
fn test_EvtxReader_new(path: &FPath, ok: bool) {
    match EvtxReader::new(
        path.clone(),
        FT_NORM,
    ) {
        Ok(_) => {
            assert!(ok, "EvtxReader::new({:?}) should have failed", path);
        }
        Err(_err) => {
            assert!(!ok, "EvtxReader::new({:?}) should have succeeded", path);
        }
    }
}

#[test_case(&EVTX_NE_FPATH)]
#[test_case(&EVTX_KPNP_FPATH)]
fn test_mtime(path: &FPath) {
    let er1 = EvtxReader::new(
        path.clone(),
        FT_NORM,
    ).unwrap();
    // merely run the function
    _ = er1.mtime();
}

/// test `EvtxReader::summary` and `EvtxReader::summary_complete`
/// before doing any processing
#[test_case(&EVTX_NE_FPATH)]
#[test_case(&EVTX_KPNP_FPATH)]
fn test_EvtxReader_summary_empty(
    path: &FPath,
) {
    let evtxreader = EvtxReader::new(
        path.clone(),
        FT_NORM,
    ).unwrap();
    _ = evtxreader.summary();
    _ = evtxreader.summary_complete();
}

/// test `EvtxReader::next_between_datetime_filters` and
/// `EvtxReader::summary` and `EvtxReader::summary_complete`
#[test_case(
    &EVTX_NE_FPATH,
    FT_NORM,
    0,
    0,
    69632,
    0,
    None,
    None,
    None,
    None;
    "no events"
)]
#[test_case(
    &EVTX_KPNP_FPATH,
    FT_NORM,
    *EVTX_KPNP_EVENT_COUNT,
    *EVTX_KPNP_EVENT_COUNT,
    1052672,
    1,
    Some(*EVTX_KPNP_ENTRY1_DT),
    Some(*EVTX_KPNP_ENTRY227_DT),
    Some(*EVTX_KPNP_ENTRY1_DT),
    Some(*EVTX_KPNP_ENTRY227_DT);
    ".evtx"
)]
#[test_case(
    &EVTX_KPNP_GZ_FPATH,
    FT_GZ,
    *EVTX_KPNP_GZ_EVENT_COUNT,
    *EVTX_KPNP_GZ_EVENT_COUNT,
    1052672,
    1,
    Some(*EVTX_KPNP_GZ_ENTRY1_DT),
    Some(*EVTX_KPNP_GZ_ENTRY227_DT),
    Some(*EVTX_KPNP_GZ_ENTRY1_DT),
    Some(*EVTX_KPNP_GZ_ENTRY227_DT);
    ".evtx.gz"
)]
#[test_case(
    &EVTX_KPNP_LZ4_FPATH,
    FT_LZ4,
    *EVTX_KPNP_LZ4_EVENT_COUNT,
    *EVTX_KPNP_LZ4_EVENT_COUNT,
    1052672,
    1,
    Some(*EVTX_KPNP_LZ4_ENTRY1_DT),
    Some(*EVTX_KPNP_LZ4_ENTRY227_DT),
    Some(*EVTX_KPNP_LZ4_ENTRY1_DT),
    Some(*EVTX_KPNP_LZ4_ENTRY227_DT);
    ".evtx.lz4"
)]
#[test_case(
    &EVTX_KPNP_TAR_FPATH,
    FT_TAR,
    *EVTX_KPNP_TAR_EVENT_COUNT,
    *EVTX_KPNP_TAR_EVENT_COUNT,
    1052672,
    1,
    Some(*EVTX_KPNP_TAR_ENTRY1_DT),
    Some(*EVTX_KPNP_TAR_ENTRY227_DT),
    Some(*EVTX_KPNP_TAR_ENTRY1_DT),
    Some(*EVTX_KPNP_TAR_ENTRY227_DT);
    ".tar"
)]
#[test_case(
    &EVTX_KPNP_XZ_FPATH,
    FT_XZ,
    *EVTX_KPNP_XZ_EVENT_COUNT,
    *EVTX_KPNP_XZ_EVENT_COUNT,
    1052672,
    1,
    Some(*EVTX_KPNP_XZ_ENTRY1_DT),
    Some(*EVTX_KPNP_XZ_ENTRY227_DT),
    Some(*EVTX_KPNP_XZ_ENTRY1_DT),
    Some(*EVTX_KPNP_XZ_ENTRY227_DT);
    ".evtx.xz"
)]
fn test_EvtxReader_next_summary(
    path: &FPath,
    filetype: FileType,
    events_processed: Count,
    events_accepted: Count,
    filesz: FileSz,
    out_of_order: Count,
    datetime_first_accepted: DateTimeLOpt,
    datetime_last_accepted: DateTimeLOpt,
    datetime_first_processed: DateTimeLOpt,
    datetime_last_processed: DateTimeLOpt,
) {
    match filetype {
        FileType::Evtx { archival_type } => {
            match archival_type {
                // XXX: forces error if new FileTypeArchive is added
                FileTypeArchive::Normal
                | FileTypeArchive::Gz
                | FileTypeArchive::Lz4
                | FileTypeArchive::Tar
                | FileTypeArchive::Xz
                => {}
            }
        }
        _ => {
            panic!("filetype should be FileType::Evtx");
        }
    }

    let mut evtxreader = EvtxReader::new(
        path.clone(),
        filetype,
    ).unwrap();
    evtxreader.analyze(&None, &None);
    while let Some(evtx_) = evtxreader.next() {
        black_box(evtx_);
    }

    // assert EvtxReader
    assert_eq!(evtxreader.count_events_processed(), events_processed,
        "count_events_processed");
    assert_eq!(evtxreader.count_events_accepted(), events_accepted,
        "count_events_accepted");
    assert_eq!(evtxreader.filesz(), filesz, "filesz");

    // assert SummaryEvtxReader
    let summary = evtxreader.summary();
    assert_eq!(summary.evtxreader_events_processed, events_processed,
        "summary.count_events_processed");
    assert_eq!(summary.evtxreader_events_accepted, events_accepted,
        "summary.count_events_accepted");
    assert_eq!(summary.evtxreader_filesz, filesz, "summary.filesz");
    assert_eq!(summary.evtxreader_out_of_order, out_of_order,
        "summary.out_of_order");
    assert_eq!(summary.evtxreader_datetime_first_accepted, datetime_first_accepted,
        "summary.datetime_first_accepted");
    assert_eq!(summary.evtxreader_datetime_last_accepted, datetime_last_accepted,
        "summary.datetime_last_accepted");
    assert_eq!(summary.evtxreader_datetime_first_processed, datetime_first_processed,
        "summary.datetime_first_processed");
    assert_eq!(summary.evtxreader_datetime_last_processed, datetime_last_processed,
        "summary.datetime_last_processed");

    // assert Summary
    let summary_c = evtxreader.summary_complete();
    assert!(summary_c.filetype.unwrap().is_evtx(), "summary_c.filetype");
    assert_eq!(summary_c.logmessagetype, LogMessageType::Evtx,
        "summary_c.logmessagetype");
    assert!(summary_c.blockreader().is_none());
    assert_eq!(summary_c.datetime_first(), &datetime_first_accepted);
    assert_eq!(summary_c.datetime_last(), &datetime_last_accepted);
    assert_eq!(summary_c.max_drop(), 0);
    assert_eq!(summary_c.max_hit_miss(), events_processed);
    match summary_c.readerdata {
        SummaryReaderData::Etvx(_summary_evtx) => {}
        _ => {
            panic!("summary_c.readerdata() should be SummaryReaderData::Etvx");
        }
    }
}
