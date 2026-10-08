// src/tests/odlreader_tests.rs

//! tests for `odlreader.rs`

#![allow(non_snake_case)]

use std::io::ErrorKind;
use std::sync::atomic::AtomicBool;

use crate::common::{
    FileType,
    FileTypeArchive,
    OdlSubType,
};
use crate::data::datetime::ymdhms;
use crate::data::odl::Odl;
use crate::readers::odlreader::OdlReader;
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
fn test_odlreader_rejects_non_odl_filetype() {
    let error = match OdlReader::new(
        path_id_generator(),
        (*NTF_LOG_EMPTY_FPATH).clone(),
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
        FO_M7,
    ) {
        Ok(_) => panic!("expected OdlReader to reject a non-ODL filetype"),
        Err(error) => error,
    };

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(error.to_string(), "OdlReader requires FileType::Odl");
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
