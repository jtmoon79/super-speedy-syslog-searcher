// src/tests/pyeventreader_tests.rs

//! tests for [`src/readers/pyeventreader.rs`]
//!
//! [`src/readers/pyeventreader.rs`]: crate::readers::pyeventreader

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
};
use ::test_case::test_case;

use crate::common::{
    Count,
    FPath,
    FileType,
    FileTypeArchive,
    OdlSubType,
};
use crate::data::datetime::{
    DateTimeLOpt,
    ymdhmsl,
};
use crate::data::pydataevent::EtlParserUsed;
use crate::python::pyrunner::PipeSz;
use crate::readers::pyeventreader::{
    PyEventReader,
    ResultNextPyDataEvent,
};
use crate::tests::common::{
    ASL_1_FPATH,
    FO_0,
    ETL_1_FPATH,
    ETL_1_EVENT_COUNT,
    ETL_1_FILESZ,
    ODL_1_FPATH,
};
use crate::tests::venv_tests::venv_setup;

#[test_case(ETL_1_FPATH.clone(), 1024; "etl1")]
fn test_PyEventReader_new_etl(path: FPath, pipe_sz: PipeSz) {
    defn!("path={:?}, pipe_sz={:?}", path, pipe_sz);

    venv_setup();

    let per = PyEventReader::new(
        path,
        Some(EtlParserUsed::DissectEtl),
        FileType::Etl { archival_type: FileTypeArchive::Normal },
        FO_0,
        pipe_sz,
    ).unwrap();
    defo!("per: {:?}", per);
    assert_eq!(per.filesz(), ETL_1_FILESZ);
    defo!("per.mtime(): {:?}", per.mtime());
    defo!("per.path(): {:?}", per.path());
    defo!("per.pipe_sz_stdout(): {:?}", per.pipe_sz_stdout());
    defo!("per.pipe_sz_stderr(): {:?}", per.pipe_sz_stderr());
    assert_eq!(per.pipe_sz_stdout(), pipe_sz);
    assert_eq!(per.pipe_sz_stderr(), pipe_sz);

    defx!();
}

#[test_case(
    ASL_1_FPATH.clone(),
    Some(EtlParserUsed::DissectEtl),
    FileType::Asl { archival_type: FileTypeArchive::Normal } => panics;
    "asl panic"
)]
#[test_case(
    ODL_1_FPATH.clone(),
    Some(EtlParserUsed::DissectEtl),
    FileType::Odl { archival_type: FileTypeArchive::Normal, odl_sub_type: OdlSubType::Odl } => panics;
    "odl panic"
)]
#[test_case(
    ASL_1_FPATH.clone(),
    None,
    FileType::Asl { archival_type: FileTypeArchive::Normal };
    "asl no panic"
)]
#[test_case(
    ODL_1_FPATH.clone(),
    None,
    FileType::Odl { archival_type: FileTypeArchive::Normal, odl_sub_type: OdlSubType::Odl };
    "odl no panic"
)]
fn test_PyEventReader_new_asl_odl_panic(path: FPath, etl_parser_used: Option<EtlParserUsed>, filetype: FileType) {
    PyEventReader::new(
        path,
        etl_parser_used,
        filetype,
        FO_0,
        1024,
    ).unwrap();
}

#[test]
fn test_PyEventReader_ts_data_to_datetime_ok() {
    defn!();
    venv_setup();

    let per = PyEventReader::new(
        ETL_1_FPATH.clone(),
        Some(EtlParserUsed::DissectEtl),
        FileType::Etl { archival_type: FileTypeArchive::Normal },
        FO_0,
        1,
    ).unwrap();

    let ts_data = b"1590429555554"; // 2020-05-25T17:59:15.554+00:00
    let dt_ts = per.ts_data_to_datetime(ts_data).unwrap();
    defo!("dt_ts: {:?}", dt_ts);
    let dt_utc = ymdhmsl(&FO_0,2020, 5, 25, 17, 59, 15, 554);
    defo!("dt_utc: {:?}", dt_utc);
    assert_eq!(dt_utc, dt_ts);

    defx!();
}

#[test]
fn test_PyEventReader_ts_data_to_datetime_none() {
    defn!();
    venv_setup();

    let per = PyEventReader::new(
        ETL_1_FPATH.clone(),
        Some(EtlParserUsed::DissectEtl),
        FileType::Etl { archival_type: FileTypeArchive::Normal },
        FO_0,
        1,
    ).unwrap();

    let ts_data = b"-"; // May 25, 2020 16:19:15.554 UTC
    let dt_ts = per.ts_data_to_datetime(ts_data);
    assert!(dt_ts.is_none());

    defx!();
}

#[test_case(
    ETL_1_FPATH.clone(),
    8,
    Some(EtlParserUsed::DissectEtl),
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    &DateTimeLOpt::None,
    ETL_1_EVENT_COUNT;
    "etl1 pipesz 8 events all"
)]
#[test_case(
    ETL_1_FPATH.clone(),
    2056,
    Some(EtlParserUsed::DissectEtl),
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    &DateTimeLOpt::None,
    ETL_1_EVENT_COUNT;
    "etl1 pipesz 2056 events all"
)]
#[test_case(
    ETL_1_FPATH.clone(),
    64,
    Some(EtlParserUsed::DissectEtl),
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    // 2025-10-05 11:30:19.300+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2025, 10, 5, 11, 30, 19, 300)),
    &DateTimeLOpt::None,
    13;
    "etl1 pipesz 64 events 13, after 2025-10-05T11:30:19.300"
)]
#[test_case(
    ETL_1_FPATH.clone(),
    64,
    Some(EtlParserUsed::DissectEtl),
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    // 2025-10-05 11:30:19.300+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2025, 10, 5, 11, 30, 19, 300)),
    8;
    "etl1 pipesz 64 events 8, before 2025-10-05T11:30:19.300"
)]
#[test_case(
    ETL_1_FPATH.clone(),
    64,
    Some(EtlParserUsed::DissectEtl),
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    // 2030-01-01 12:00:00.000+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2030, 1, 1, 12, 0, 0, 0)),
    &DateTimeLOpt::None,
    0;
    "etl1 pipesz 64 events 0 after 2030-01-01T12:00:00.000"
)]
#[test_case(
    ETL_1_FPATH.clone(),
    64,
    Some(EtlParserUsed::DissectEtl),
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    // 2030-01-01 12:00:00.000+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2030, 1, 1, 12, 0, 0, 0)),
    21;
    "etl1 pipesz 64 events 21 before 2030-01-01T12:00:00.000"
)]
fn test_PyEventReader_next(
    path: FPath,
    pipe_sz: PipeSz,
    etl_parser_used: Option<EtlParserUsed>,
    file_type: FileType,
    dt_filter_after: &DateTimeLOpt,
    dt_filter_before: &DateTimeLOpt,
    events_expected: Count,
) {
    defn!(
        "test_PyEventReader_next: path={:?}, pipe_sz={:?}, etl_parser_used={:?}, file_type={:?}, dt_filter_after={:?}, dt_filter_before={:?}, events_expected={}",
        path, pipe_sz, etl_parser_used, file_type, dt_filter_after,  dt_filter_before, events_expected);

    venv_setup();

    let mut per = PyEventReader::new(
        path,
        etl_parser_used,
        file_type,
        FO_0,
        pipe_sz,
    ).unwrap();

    let mut count: Count = 0;
    loop {
        let pde_result = per.next(dt_filter_after, dt_filter_before);
        match pde_result {
            ResultNextPyDataEvent::Found(pde) => {
                count += 1;
                defo!("pde: {:?}, count is {}", pde, count);
            }
            ResultNextPyDataEvent::Done => {
                defo!("Done");
                break;
            }
            ResultNextPyDataEvent::Err(err) => {
                defo!("Err PyDataEvent: {}", err);
                break;
            }
            ResultNextPyDataEvent::ErrIgnore(err) => {
                defo!("ErrIgnore reading PyDataEvent: {}", err);
                break;
            }
        }
    }
    defo!("total PyDataEvents read: {}", count);
    assert_eq!(count, events_expected, "expected {} PyDataEvents", events_expected);

    defx!();
}
