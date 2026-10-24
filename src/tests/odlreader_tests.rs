// src/tests/odlreader_tests.rs

//! tests for `odlreader.rs`

#![allow(non_snake_case)]

use std::io::{
    Cursor,
    ErrorKind,
};
use std::sync::atomic::AtomicBool;
use std::time::{
    Duration,
    SystemTime,
};

use ::test_case::test_case;

use crate::common::{
    FileType,
    FileTypeArchive,
    OdlSubType,
    summary_stats_enable,
};
use crate::data::datetime::ymdhms;
use crate::data::odl::Odl;
use crate::readers::odlparser::OdlDecodingContext;
use crate::readers::odlreader::{
    OdlReader,
    OdlSource,
};
use crate::readers::summary::SummaryReaderData;
use crate::tests::odlparser_tests::{
    odl_file,
    v3_record,
};
use crate::tests::common::{
    FO_M7,
    NTF_LOG_EMPTY_FPATH,
    ODL_NUCLEUS_ODLGZ_DATA,
    ODL_NUCLEUS_ODLGZ_FILETYPE,
    ODL_NUCLEUS_ODLGZ_PATH,
    ODL_NUCLEUSLOCAL_AODL_DATA,
    ODL_NUCLEUSLOCAL_AODL_FILETYPE,
    ODL_NUCLEUSLOCAL_AODL_PATH,
    ODL_SYNCENGINE_ODL_DATA,
    ODL_SYNCENGINE_ODL_FILETYPE,
    ODL_SYNCENGINE_ODL_PATH,
    OdlExpectedLine,
    path_id_generator,
};

#[test_case(&[100_000, 90_250, 110_000, 80_500, 79_000], 3, Some(Duration::from_millis(29_500)); "largest consecutive backward jump")]
#[test_case(&[79_000, 80_500, 90_250, 110_000, 110_000], 0, None; "ordered and equal")]
#[test_case(&[110_000, 109_999], 1, Some(Duration::from_millis(1)); "millisecond")]
#[test_case(&[], 0, None; "empty")]
fn test_odlreader_out_of_order_max(timestamps: &[u64], count: u64, maximum: Option<Duration>) {
    summary_stats_enable();
    let records: Vec<_> = timestamps.iter().map(|timestamp| v3_record(*timestamp, b"source.cpp", b"function", &[])).collect();
    let slices: Vec<_> = records.iter().map(Vec::as_slice).collect();
    let bytes = odl_file(3, &slices);
    for before in [None, Some(ymdhms(&FO_M7, 1900, 1, 1, 0, 0, 0))] {
        let source = OdlSource {
            path_id: path_id_generator(),
            path: "test.odl".into(),
            filetype: ODL_SYNCENGINE_ODL_FILETYPE,
            filesz: bytes.len() as u64,
            mtime: SystemTime::UNIX_EPOCH,
            fixed_offset: FO_M7,
        };
        let mut reader = OdlReader::from_reader(Cursor::new(bytes.clone()), source, OdlDecodingContext::default()).unwrap();
        assert_eq!(reader.summary().odlreader_out_of_order_max, None);
        reader.analyze(&None, &before, &AtomicBool::new(false)).unwrap();
        let stats = reader.summary();
        assert_eq!(stats.odlreader_events_processed, timestamps.len() as u64);
        assert_eq!(stats.odlreader_events_accepted, if before.is_some() { 0 } else { timestamps.len() as u64 });
        assert_eq!(stats.odlreader_out_of_order, count);
        assert_eq!(stats.odlreader_out_of_order_max, maximum);
        let summary = reader.summary_complete();
        assert_eq!(summary.error, None);
        let SummaryReaderData::Odl(stats) = summary.readerdata else {
            panic!("expected ODL summary")
        };
        assert_eq!(stats.odlreader_out_of_order_max, maximum);
    }
}

fn assert_reader_fixture(
    path: &str,
    filetype: FileType,
    expected: &[OdlExpectedLine],
) {
    let mut reader = OdlReader::new(path_id_generator(), path.to_string(), filetype, FO_M7).unwrap();
    reader
        .analyze(&None, &None, &AtomicBool::new(false))
        .unwrap();

    let mut actual: Vec<Odl> = Vec::new();
    while let Some(event) = reader.next() {
        actual.push(event);
    }

    assert_eq!(actual.len(), expected.len(), "event count");
    for (actual, expected) in actual
        .iter()
        .zip(expected.iter())
    {
        assert_eq!(actual.dt(), &expected.dt);
        assert_eq!(actual.dt_beg_end(), &Some((0, 29)));
        assert_eq!(
            actual
                .as_bytes()
                .strip_suffix(b"\n"),
            Some(expected.line)
        );
    }
    assert_eq!(reader.next(), None);
    assert_eq!(
        reader
            .summary_complete()
            .error,
        None
    );
}

#[test]
fn test_odlreader_nucleus_odlgz() {
    assert_reader_fixture(ODL_NUCLEUS_ODLGZ_PATH, ODL_NUCLEUS_ODLGZ_FILETYPE, &ODL_NUCLEUS_ODLGZ_DATA);
}

#[test]
fn test_odlreader_syncengine_odl() {
    assert_reader_fixture(ODL_SYNCENGINE_ODL_PATH, ODL_SYNCENGINE_ODL_FILETYPE, &ODL_SYNCENGINE_ODL_DATA);
}

#[test]
fn test_odlreader_nucleuslocal_aodl() {
    assert_reader_fixture(ODL_NUCLEUSLOCAL_AODL_PATH, ODL_NUCLEUSLOCAL_AODL_FILETYPE, &ODL_NUCLEUSLOCAL_AODL_DATA);
}

#[test]
fn test_odlreader_file_empty() {
    let mut reader = OdlReader::new(
        path_id_generator(),
        (*NTF_LOG_EMPTY_FPATH).clone(),
        FileType::Odl {
            archival_type: FileTypeArchive::Normal,
            odl_sub_type: OdlSubType::Odl,
        },
        FO_M7,
    )
    .unwrap();
    let error = reader
        .analyze(&None, &None, &AtomicBool::new(false))
        .unwrap_err();

    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    assert_eq!(
        reader
            .summary_complete()
            .error
            .as_deref(),
        Some(error.to_string().as_str())
    );
}

#[test]
fn test_odlreader_analyze_twice() {
    let mut reader =
        OdlReader::new(path_id_generator(), ODL_SYNCENGINE_ODL_PATH.to_string(), ODL_SYNCENGINE_ODL_FILETYPE, FO_M7)
            .unwrap();
    reader
        .analyze(&None, &None, &AtomicBool::new(false))
        .unwrap();
    let error = reader
        .analyze(&None, &None, &AtomicBool::new(false))
        .unwrap_err();

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(error.to_string(), "ODL reader already analyzed");
}

#[test]
fn test_odlreader_datetime_filter_excludes_all() {
    let mut reader =
        OdlReader::new(path_id_generator(), ODL_NUCLEUS_ODLGZ_PATH.to_string(), ODL_NUCLEUS_ODLGZ_FILETYPE, FO_M7)
            .unwrap();
    let before = Some(ymdhms(&FO_M7, 2000, 1, 1, 0, 0, 0));
    reader
        .analyze(&None, &before, &AtomicBool::new(false))
        .unwrap();

    assert_eq!(reader.next(), None);
    assert_eq!(
        reader
            .summary_complete()
            .error,
        None
    );
}

#[test]
fn test_odlreader_analyze_cancelled() {
    let mut reader = OdlReader::new(
        path_id_generator(),
        ODL_NUCLEUSLOCAL_AODL_PATH.to_string(),
        ODL_NUCLEUSLOCAL_AODL_FILETYPE,
        FO_M7,
    )
    .unwrap();
    let error = reader
        .analyze(&None, &None, &AtomicBool::new(true))
        .unwrap_err();

    assert_eq!(error.kind(), ErrorKind::Interrupted);
    assert_eq!(error.to_string(), "ODL analysis cancelled");
}
