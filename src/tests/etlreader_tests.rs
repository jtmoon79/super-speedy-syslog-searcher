// src/tests/etlreader_tests.rs

//! tests for `etlreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use crate::common::{
    FileType,
    FileTypeArchive,
};
use crate::data::datetime::FixedOffset;
use crate::data::etl::Etl;
use crate::readers::etlreader::EtlReader;
use crate::tests::common::{
    ETL_FILE1_DATA,
    ETL_FILE1_PATH,
    NTF_LOG_EMPTY_FPATH,
    path_id_generator,
};

#[allow(non_upper_case_globals)]
const FO_m7: FixedOffset = FixedOffset::east_opt(-7 * 3600).unwrap();

/// create a `EtlReader` for the file at `ETL_FILE1_PATH`
/// process the Events, compare them against expected results
#[test]
fn test_etlreader_file1() {
    let mut reader = EtlReader::new(
        path_id_generator(),
        ETL_FILE1_PATH.to_string(),
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
        FO_m7,
    )
    .unwrap();
    reader.analyze(&None, &None);

    let mut actual: Vec<Etl> = Vec::new();
    while let Some(event) = reader.next() {
        actual.push(event);
    }

    let mut expected: Vec<Etl> = ETL_FILE1_DATA.clone();
    expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(actual.len(), expected.len(), "event count");
    for (actual, expected) in actual
        .iter()
        .zip(expected.iter())
    {
        assert_eq!(actual.dt(), expected.dt());
        assert_eq!(actual.dt_beg_end(), expected.dt_beg_end());
        assert_eq!(
            actual
                .as_bytes()
                .strip_suffix(b"\n"),
            Some(expected.as_bytes())
        );
    }
    assert_eq!(
        reader
            .summary_complete()
            .error,
        None
    );
}

#[test]
fn test_etlreader_file_empty() {
    let mut reader = EtlReader::new(
        path_id_generator(),
        (*NTF_LOG_EMPTY_FPATH).clone(),
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
        FO_m7,
    )
    .unwrap();
    reader.analyze(&None, &None);

    assert_eq!(reader.next(), None);
    assert_eq!(
        reader
            .summary_complete()
            .error
            .as_deref(),
        Some("ETL file has no buffers")
    );
}
