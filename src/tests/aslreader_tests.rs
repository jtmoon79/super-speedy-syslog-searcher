//! Tests for `AslReader` event and rendered-byte APIs.

use std::io::{Cursor, ErrorKind, Read};
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;

use crate::common::{FileType, FileTypeArchive, LogMessageType, summary_stats_enabled};
use crate::data::asl::Asl;
use crate::data::datetime::ymdhmsl;
use crate::readers::aslreader::{AslReader, AslSource};
use crate::readers::summary::SummaryReaderData;
use crate::tests::common::{ASL_FIXTURES, FO_0, NTF_LOG_EMPTY_FPATH, path_id_generator};

const ASL_FILETYPE: FileType = FileType::Asl {
    archival_type: FileTypeArchive::Normal,
};

fn reader_from_bytes(bytes: Vec<u8>) -> AslReader<Cursor<Vec<u8>>> {
    let source = AslSource {
        path_id: path_id_generator(),
        path: "test.asl".into(),
        filetype: ASL_FILETYPE,
        filesz: bytes.len() as u64,
        mtime: SystemTime::UNIX_EPOCH,
        fixed_offset: FO_0,
    };
    AslReader::from_reader(Cursor::new(bytes), source).unwrap()
}

#[test]
fn aslreader_real_files_and_summaries() {
    for &(path, count, first_id, last_id) in &ASL_FIXTURES {
        let mut reader = AslReader::new(path_id_generator(), path.into(), ASL_FILETYPE, FO_0).unwrap();
        let stats_enabled = summary_stats_enabled();
        reader
            .analyze(&None, &None, &AtomicBool::new(false))
            .unwrap();
        let mut events: Vec<Asl> = Vec::new();
        while let Some(event) = reader.next_event().unwrap() {
            if let Some(previous) = events.last() {
                assert!(previous.dt() <= event.dt(), "{path}");
            }
            events.push(event);
        }
        assert_eq!(events.len(), count, "{path}");
        assert!(String::from_utf8_lossy(events[0].as_bytes()).contains(&format!("  id={first_id}  ")), "{path}");
        assert!(String::from_utf8_lossy(events[count - 1].as_bytes()).contains(&format!("  id={last_id}  ")), "{path}");
        let (first_timestamp, last_timestamp) = match first_id {
            208037 => ("2024-03-05T16:03:24.000000000+00:00", "2024-03-24T17:38:34.545788000+00:00"),
            425 => ("2024-03-05T15:34:24.000000000+00:00", "2024-03-24T19:11:50.348826000+00:00"),
            2 => ("2026-05-29T18:44:57.000000000+00:00", "2026-05-30T06:50:27.000000000+00:00"),
            _ => panic!("unexpected ASL fixture {path}"),
        };
        assert!(
            events[0]
                .as_bytes()
                .starts_with(first_timestamp.as_bytes()),
            "{path}"
        );
        assert!(
            events[count - 1]
                .as_bytes()
                .starts_with(last_timestamp.as_bytes()),
            "{path}"
        );
        assert!(
            reader
                .next_event()
                .unwrap()
                .is_none()
        );

        let summary = reader.summary_complete();
        assert_eq!(summary.logmessagetype, LogMessageType::Asl, "{path}");
        assert!(summary.error.is_none(), "{path}");
        let SummaryReaderData::Asl(stats) = summary.readerdata else {
            panic!("expected ASL summary for {path}")
        };
        assert_eq!(
            stats.aslreader_filesz,
            std::fs::metadata(path)
                .unwrap()
                .len(),
            "{path}"
        );
        if stats_enabled {
            assert_eq!(stats.aslreader_events_processed, count as u64, "{path}");
            assert_eq!(stats.aslreader_events_accepted, count as u64, "{path}");
            assert_eq!(stats.aslreader_datetime_first_accepted, Some(*events[0].dt()), "{path}");
            assert_eq!(stats.aslreader_datetime_last_accepted, Some(*events[count - 1].dt()), "{path}");
        }
    }
}

#[test]
fn aslreader_read_in_small_chunks_and_exclusive_modes() {
    let mut reader = reader_from_bytes(std::fs::read(ASL_FIXTURES[0].0).unwrap());
    assert_eq!(reader.read(&mut []).unwrap(), 0);
    let mut first = [0; 7];
    assert_eq!(
        reader
            .read(&mut first)
            .unwrap(),
        7
    );
    assert_eq!(&first, b"2024-03");
    let mut rest = Vec::new();
    reader
        .read_to_end(&mut rest)
        .unwrap();
    let output = [
        first.as_slice(),
        rest.as_slice(),
    ]
    .concat();
    let output = std::str::from_utf8(&output).unwrap();
    assert_eq!(output.lines().count(), 5);
    assert!(output.starts_with("2024-03-05T16:03:24.000000000+00:00  id=208037  "));
    assert!(
        output.contains("  sender=loginwindow  facility=com.apple.system.lastlog  message='USER_PROCESS: 138 console'")
    );
    assert_eq!(
        reader
            .read(&mut first)
            .unwrap(),
        0
    );
    assert_eq!(
        reader
            .next_event()
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidInput
    );

    let mut reader = reader_from_bytes(std::fs::read(ASL_FIXTURES[0].0).unwrap());
    assert_eq!(
        reader
            .next_event()
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidInput
    );
    reader
        .analyze(&None, &None, &AtomicBool::new(false))
        .unwrap();
    assert!(
        reader
            .next_event()
            .unwrap()
            .is_some()
    );
    assert_eq!(
        reader
            .read(&mut first)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        reader
            .analyze(&None, &None, &AtomicBool::new(false))
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidInput
    );
}

#[test]
fn aslreader_inclusive_datetime_filter() {
    let path = ASL_FIXTURES[0].0;
    let mut reader = AslReader::new(path_id_generator(), path.into(), ASL_FILETYPE, FO_0).unwrap();
    let stats_enabled = summary_stats_enabled();
    let after = Some(ymdhmsl(&FO_0, 2024, 3, 5, 16, 4, 17, 288));
    let before = Some(ymdhmsl(&FO_0, 2024, 3, 5, 16, 4, 17, 289));
    reader
        .analyze(&after, &before, &AtomicBool::new(false))
        .unwrap();
    let event = reader
        .next_event()
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(event.as_bytes()).contains("  id=208329  "));
    assert!(
        reader
            .next_event()
            .unwrap()
            .is_none()
    );
    let SummaryReaderData::Asl(stats) = reader
        .summary_complete()
        .readerdata
    else {
        panic!("expected ASL summary")
    };
    if stats_enabled {
        assert_eq!(stats.aslreader_events_processed, 5);
        assert_eq!(stats.aslreader_events_accepted, 1);
        assert_eq!(stats.aslreader_datetime_first_accepted, Some(*event.dt()));
        assert_eq!(stats.aslreader_datetime_last_accepted, Some(*event.dt()));
    }
}

#[test]
fn aslreader_empty_file_and_bad_signature() {
    let mut empty = AslReader::new(path_id_generator(), (*NTF_LOG_EMPTY_FPATH).clone(), ASL_FILETYPE, FO_0).unwrap();
    let error = empty
        .analyze(&None, &None, &AtomicBool::new(false))
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "ASL header is truncated");
    assert_eq!(
        empty
            .summary_complete()
            .error
            .as_deref(),
        Some("ASL header is truncated")
    );

    let mut empty = reader_from_bytes(Vec::new());
    let error = empty
        .read(&mut [0])
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "ASL header is truncated");

    let mut sidecar = reader_from_bytes(std::fs::read("./logs/MacOS11/asl/._2023.10.26.G80.asl").unwrap());
    let error = sidecar
        .read(&mut [0])
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "invalid ASL database signature \"␀␅␖␇␀␂␀␀Mac \"");
}

#[test]
fn aslreader_empty_database_and_cancellation() {
    let mut empty_database = vec![0; 80];
    empty_database[..12].copy_from_slice(b"ASL DB\0\0\0\0\0\0");
    empty_database[12..16].copy_from_slice(&2u32.to_be_bytes());
    let mut reader = reader_from_bytes(empty_database);
    assert_eq!(reader.read(&mut [0]).unwrap(), 0);
    let SummaryReaderData::Asl(stats) = reader
        .summary_complete()
        .readerdata
    else {
        panic!("expected ASL summary")
    };
    assert_eq!(stats.aslreader_events_processed, 0);

    let mut reader = reader_from_bytes(std::fs::read(ASL_FIXTURES[0].0).unwrap());
    let error = reader
        .analyze(&None, &None, &AtomicBool::new(true))
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Interrupted);
    assert_eq!(
        reader
            .summary_complete()
            .error
            .as_deref(),
        Some("ASL analysis cancelled")
    );
}

#[test]
fn aslreader_rejects_non_asl_filetype() {
    let source = AslSource {
        path_id: path_id_generator(),
        path: "test.etl".into(),
        filetype: FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
        filesz: 0,
        mtime: SystemTime::UNIX_EPOCH,
        fixed_offset: FO_0,
    };
    let error = match AslReader::from_reader(Cursor::new(Vec::<u8>::new()), source) {
        Ok(_) => panic!("expected filetype error"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    assert_eq!(error.to_string(), "AslReader requires FileType::Asl");
}
