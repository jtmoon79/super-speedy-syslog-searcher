// src/tests/pyeventreader_tests.rs

//! tests for [`src/readers/pyeventreader.rs`]
//!
//! [`src/readers/pyeventreader.rs`]: crate::readers::pyeventreader

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use ::const_format::concatcp;
#[allow(unused_imports)]
use ::si_trace_print::printers::{
    defn,
    defo,
    defx,
};
use ::test_case::test_case;

use crate::common::{
    FPath,
    FileType,
    FileTypeArchive,
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
use crate::tests::common::FO_0;

pub const LOG_ETL_FILE_1: &str = concatcp!(
    env!("CARGO_MANIFEST_DIR"),
    "/logs/programs/Event_Trace_Log/waasmedic.20251005_113019_195.etl"
);
pub const LOG_ETL_FILE_1_SZ: usize = 16384;
pub const LOG_ETL_FILE_1_EVENTS_COUNT: usize = 21;

#[test_case(LOG_ETL_FILE_1.to_string(), 1024; "etl1")]
fn test_PyEventReader_new_etl(path: FPath, pipe_sz: PipeSz) {
    let per = PyEventReader::new(
        path,
        EtlParserUsed::DissectEtl,
        FileType::Etl { archival_type: FileTypeArchive::Normal },
        FO_0,
        pipe_sz,
    ).unwrap();
    defo!("per: {:?}", per);
    assert_eq!(per.filesz() as usize, LOG_ETL_FILE_1_SZ);
    defo!("per.mtime(): {:?}", per.mtime());
    defo!("per.path(): {:?}", per.path());
    defo!("per.pipe_sz_stdout(): {:?}", per.pipe_sz_stdout());
    defo!("per.pipe_sz_stderr(): {:?}", per.pipe_sz_stderr());
    assert_eq!(per.pipe_sz_stdout(), pipe_sz);
    assert_eq!(per.pipe_sz_stderr(), pipe_sz);
}

#[test]
fn test_PyEventReader_ts_data_to_datetime_ok() {
    let per = PyEventReader::new(
        LOG_ETL_FILE_1.to_string(),
        EtlParserUsed::DissectEtl,
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
}

#[test]
fn test_PyEventReader_ts_data_to_datetime_none() {
    let per = PyEventReader::new(
        LOG_ETL_FILE_1.to_string(),
        EtlParserUsed::DissectEtl,
        FileType::Etl { archival_type: FileTypeArchive::Normal },
        FO_0,
        1,
    ).unwrap();

    let ts_data = b"-"; // May 25, 2020 16:19:15.554 UTC
    let dt_ts = per.ts_data_to_datetime(ts_data);
    assert!(dt_ts.is_none());
}

#[test_case(
    LOG_ETL_FILE_1.to_string(),
    8,
    EtlParserUsed::DissectEtl,
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    &DateTimeLOpt::None,
    LOG_ETL_FILE_1_EVENTS_COUNT;
    "etl1 pipesz 8 events all"
)]
#[test_case(
    LOG_ETL_FILE_1.to_string(),
    2056,
    EtlParserUsed::DissectEtl,
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    &DateTimeLOpt::None,
    LOG_ETL_FILE_1_EVENTS_COUNT;
    "etl1 pipesz 2056 events all"
)]
#[test_case(
    LOG_ETL_FILE_1.to_string(),
    64,
    EtlParserUsed::DissectEtl,
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    // 2025-10-05 11:30:19.300+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2025, 10, 5, 11, 30, 19, 300)),
    &DateTimeLOpt::None,
    13;
    "etl1 pipesz 64 events 13, after 2025-10-05T11:30:19.300"
)]
#[test_case(
    LOG_ETL_FILE_1.to_string(),
    64,
    EtlParserUsed::DissectEtl,
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    &DateTimeLOpt::None,
    // 2025-10-05 11:30:19.300+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2025, 10, 5, 11, 30, 19, 300)),
    8;
    "etl1 pipesz 64 events 8, before 2025-10-05T11:30:19.300"
)]
#[test_case(
    LOG_ETL_FILE_1.to_string(),
    64,
    EtlParserUsed::DissectEtl,
    FileType::Etl { archival_type: FileTypeArchive::Normal },
    // 2030-01-01 12:00:00.000+00:00
    &DateTimeLOpt::Some(ymdhmsl(&FO_0, 2030, 1, 1, 12, 0, 0, 0)),
    &DateTimeLOpt::None,
    0;
    "etl1 pipesz 64 events 0 after 2030-01-01T12:00:00.000"
)]
#[test_case(
    LOG_ETL_FILE_1.to_string(),
    64,
    EtlParserUsed::DissectEtl,
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
    etl_parser_used: EtlParserUsed,
    file_type: FileType,
    dt_filter_after: &DateTimeLOpt,
    dt_filter_before: &DateTimeLOpt,
    events_expected: usize,
) {
    defn!(
        "test_PyEventReader_next: path={:?}, pipe_sz={:?}, etl_parser_used={:?}, file_type={:?}, dt_filter_after={:?}, dt_filter_before={:?}, events_expected={}",
        path, pipe_sz, etl_parser_used, file_type, dt_filter_after,  dt_filter_before, events_expected);

    let mut per = PyEventReader::new(
        path,
        etl_parser_used,
        file_type,
        FO_0,
        pipe_sz,
    ).unwrap();

    let mut count: usize = 0;
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
}
