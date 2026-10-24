// src/tests/etlreader_tests.rs

//! tests for `etlreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use ::test_case::test_case;

use crate::common::{
    FileType,
    FileTypeArchive,
    summary_stats_enable,
};
use crate::data::datetime::FixedOffset;
use crate::data::etl::Etl;
use crate::debug::helpers::{create_temp_file_data, ntf_fpath};
use crate::readers::etlreader::EtlReader;
use crate::readers::summary::SummaryReaderData;
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
#[test_case(false; "unfiltered")]
#[test_case(true; "filtered")]
fn test_etlreader_file1(filtered: bool) {
    summary_stats_enable();
    let mut reader = EtlReader::new(
        path_id_generator(),
        ETL_FILE1_PATH.to_string(),
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
        FO_m7,
    )
    .unwrap();
    assert_eq!(reader.out_of_order_max, None);
    assert_eq!(reader.summary().etlreader_out_of_order_max, None);
    let after = filtered.then(|| *ETL_FILE1_DATA[7].dt());
    reader.analyze(&after, &None);
    let maximum = None;
    assert_eq!(reader.out_of_order, 0);
    assert_eq!(reader.out_of_order_max, maximum);
    let stats = reader.summary();
    assert_eq!(stats.etlreader_events_processed, ETL_FILE1_DATA.len() as u64);
    assert_eq!(stats.etlreader_events_accepted, if filtered { 5 } else { 12 });
    assert_eq!(stats.etlreader_out_of_order, 0);
    assert_eq!(stats.etlreader_out_of_order_max, maximum);
    let SummaryReaderData::Etl(stats) = reader.summary_complete().readerdata else {
        panic!("expected ETL summary")
    };
    assert_eq!(stats.etlreader_out_of_order_max, maximum);

    let mut actual: Vec<Etl> = Vec::new();
    while let Some(event) = reader.next() {
        actual.push(event);
    }

    let mut expected: Vec<Etl> = ETL_FILE1_DATA.clone();
    expected.retain(|event| after.is_none_or(|after| *event.dt() >= after));
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

#[test_case(false; "unfiltered")]
#[test_case(true; "filtered")]
fn test_etlreader_out_of_order_max(filtered: bool) {
    summary_stats_enable();
    let mut bytes = std::fs::read(ETL_FILE1_PATH).unwrap();
    // Swap record timestamps to create a 1.0251439s jump followed by a smaller jump.
    for (first, second) in [(4864 + 16, 5080 + 16), (6008 + 16, 6352 + 16)] {
        for index in 0..8 {
            bytes.swap(first + index, second + index);
        }
    }
    let file = create_temp_file_data(&bytes);
    let mut reader = EtlReader::new(
        path_id_generator(),
        ntf_fpath(&file),
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
        FO_m7,
    )
    .unwrap();
    let after = filtered.then(|| *ETL_FILE1_DATA[7].dt());
    reader.analyze(&after, &None);
    let maximum = Some(Duration::new(1, 25_143_900));
    assert_eq!(reader.out_of_order, 2);
    assert_eq!(reader.out_of_order_max, maximum);
    let stats = reader.summary();
    assert_eq!(stats.etlreader_events_processed, 12);
    assert_eq!(stats.etlreader_events_accepted, if filtered { 5 } else { 12 });
    assert_eq!(stats.etlreader_out_of_order, 2);
    assert_eq!(stats.etlreader_out_of_order_max, maximum);
    let summary = reader.summary_complete();
    assert_eq!(summary.error, None);
    let SummaryReaderData::Etl(stats) = summary.readerdata else {
        panic!("expected ETL summary")
    };
    assert_eq!(stats.etlreader_out_of_order_max, maximum);
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
    assert_eq!(reader.out_of_order_max, None);
    assert_eq!(reader.summary().etlreader_out_of_order_max, None);
    assert_eq!(
        reader
            .summary_complete()
            .error
            .as_deref(),
        Some("ETL file has no buffers")
    );
}
