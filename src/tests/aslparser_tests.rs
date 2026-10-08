//! Tests for the version-2 Apple System Log binary parser.

use std::fs::File;
use std::io::{self, Cursor, ErrorKind};

use compact_str::CompactString;

use crate::data::asl::AslRecord;
use crate::readers::aslparser::AslParser;
use crate::tests::common::ASL_FIXTURES;

fn expect_err<T>(result: io::Result<T>) -> io::Error {
    match result {
        Ok(_) => panic!("expected invalid ASL data"),
        Err(error) => error,
    }
}

fn fixture_bytes() -> Vec<u8> {
    std::fs::read(ASL_FIXTURES[0].0).unwrap()
}

fn first_offset(bytes: &[u8]) -> usize {
    u64::from_be_bytes(
        bytes[16..24]
            .try_into()
            .unwrap(),
    ) as usize
}

fn parser(bytes: Vec<u8>) -> AslParser<Cursor<Vec<u8>>> {
    AslParser::new(Cursor::new(bytes)).unwrap()
}

fn database_with_string_reference(reference: [u8; 8]) -> Vec<u8> {
    let mut bytes = vec![0; 80 + 6 + 132];
    bytes[..12].copy_from_slice(b"ASL DB\0\0\0\0\0\0");
    bytes[12..16].copy_from_slice(&2u32.to_be_bytes());
    bytes[16..24].copy_from_slice(&80u64.to_be_bytes());
    bytes[37..45].copy_from_slice(&80u64.to_be_bytes());
    bytes[82..86].copy_from_slice(&132u32.to_be_bytes());
    let data = &mut bytes[86..];
    data[56..60].copy_from_slice(&2u32.to_be_bytes());
    for field in data[60..124].chunks_exact_mut(8) {
        field.copy_from_slice(&reference);
    }

    bytes
}

fn record_strings(record: &AslRecord) -> [&CompactString; 8] {
    [
        &record.host,
        &record.sender,
        &record.facility,
        &record.message,
        &record.ref_proc,
        &record.session,
        &record.extra[0].0,
        &record.extra[0].1,
    ]
}

fn assert_record_error(
    bytes: Vec<u8>,
    expected: &str,
) {
    let error = expect_err(parser(bytes).next_record());
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(
        error
            .to_string()
            .contains(expected),
        "{error}"
    );
}

#[test]
fn aslparser_real_files() {
    for &(path, count, first_id, last_id) in &ASL_FIXTURES {
        let mut parser = AslParser::new(File::open(path).unwrap()).unwrap();
        let mut records = Vec::new();
        while let Some(record) = parser.next_record().unwrap() {
            assert_eq!(record.ordinal, records.len() as u64, "{path}");
            assert!(record.nanoseconds < 1_000_000_000, "{path}");
            records.push(record);
        }
        assert_eq!(records.len(), count, "{path}");
        assert_eq!(records.first().unwrap().id, first_id, "{path}");
        assert_eq!(records.last().unwrap().id, last_id, "{path}");
        assert_eq!(records[0].offset, if first_id == 2 { 332 } else { 273 }, "{path}");
        assert_eq!(records[0].host, "localhost", "{path}");
        assert_eq!(records[0].level, 5, "{path}");
        assert!(
            parser
                .next_record()
                .unwrap()
                .is_none(),
            "{path}"
        );

        if first_id == 2 {
            assert_eq!(records.last().unwrap().offset, 18861);
            assert_eq!(records[0].sender, "syslogd");
            assert_eq!(records[0].facility, "syslog");
            assert!(
                records[0]
                    .message
                    .starts_with("Configuration Notice:\nASL Module")
            );
            assert_eq!(records[0].extra[0].0, "SenderMachUUID");
            assert_eq!(
                records
                    .last()
                    .unwrap()
                    .message,
                "ASL Sender Statistics"
            );
        } else {
            assert_eq!(records.last().unwrap().offset, if first_id == 425 { 1975 } else { 1990 });
            assert_eq!(records[0].sender, "bootlog");
            assert_eq!(records[0].facility, "com.apple.system.utmpx");
            assert_eq!(
                records[0].message,
                if first_id == 425 { "BOOT_TIME 1709652864 0" } else { "BOOT_TIME 1709654604 0" }
            );
            assert_eq!(records[0].extra[0], ("ut_id".into(), "0x00 0x00 0x00 0x00".into()));
            assert_eq!(records[1].sender, "loginwindow");
            assert_eq!(records[1].extra[0], ("ut_user".into(), "runner".into()));
        }
    }
}

#[test]
fn aslparser_empty_and_invalid_headers() {
    let error = expect_err(AslParser::new(Cursor::new(Vec::<u8>::new())));
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "ASL header is truncated");
    let error = expect_err(AslParser::new(Cursor::new(vec![0; 79])));
    assert_eq!(error.to_string(), "ASL header is truncated");

    let mut empty_database = vec![0; 80];
    empty_database[..12].copy_from_slice(b"ASL DB\0\0\0\0\0\0");
    empty_database[12..16].copy_from_slice(&2u32.to_be_bytes());
    assert!(
        parser(empty_database)
            .next_record()
            .unwrap()
            .is_none()
    );

    let mut bad = fixture_bytes();
    bad[0] = b'X';
    let error = expect_err(AslParser::new(Cursor::new(bad)));
    assert_eq!(error.to_string(), "invalid ASL database signature");

    let mut bad = fixture_bytes();
    bad[12..16].copy_from_slice(&3u32.to_be_bytes());
    let error = expect_err(AslParser::new(Cursor::new(bad)));
    assert_eq!(error.to_string(), "unsupported ASL database version");

    let mut bad = fixture_bytes();
    bad[16..24].copy_from_slice(&0u64.to_be_bytes());
    let error = expect_err(AslParser::new(Cursor::new(bad)));
    assert_eq!(error.to_string(), "invalid ASL first or last record offset");
}

#[test]
fn aslparser_rejects_malformed_records_and_cycles() {
    let mut bad = fixture_bytes();
    let offset = first_offset(&bad);
    bad[offset] = 1;
    assert_record_error(bad, "invalid ASL record tag");

    for length in [
        115u32,
        4 * 1024 * 1024 + 1,
    ] {
        let mut bad = fixture_bytes();
        let offset = first_offset(&bad);
        bad[offset + 2..offset + 6].copy_from_slice(&length.to_be_bytes());
        assert_record_error(bad, "invalid ASL record length");
    }

    let mut bad = fixture_bytes();
    let offset = first_offset(&bad);
    bad[37..45].copy_from_slice(&(offset as u64).to_be_bytes());
    bad[offset + 6..offset + 14].copy_from_slice(&0u64.to_be_bytes());
    bad.truncate(offset + 6 + 115);
    assert_record_error(bad, "invalid ASL record length");

    let mut bad = fixture_bytes();
    let offset = first_offset(&bad);
    bad[offset + 6 + 56..offset + 6 + 60].copy_from_slice(&3u32.to_be_bytes());
    assert_record_error(bad, "invalid ASL extra fields");

    let mut bad = fixture_bytes();
    let offset = first_offset(&bad);
    bad[offset + 6..offset + 14].copy_from_slice(&0u64.to_be_bytes());
    assert_record_error(bad, "invalid ASL next record offset");

    let mut bad = fixture_bytes();
    let offset = first_offset(&bad);
    bad[offset + 6..offset + 14].copy_from_slice(&(offset as u64).to_be_bytes());
    let mut cycle_parser = parser(bad);
    assert!(
        cycle_parser
            .next_record()
            .unwrap()
            .is_some()
    );
    let error = expect_err(cycle_parser.next_record());
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(
        error
            .to_string()
            .contains("cycle")
    );

    let mut bad = fixture_bytes();
    let offset = first_offset(&bad);
    let next = u64::from_be_bytes(
        bad[offset + 6..offset + 14]
            .try_into()
            .unwrap(),
    ) as usize;
    bad[next] = 1;
    let mut parser = parser(bad);
    assert!(
        parser
            .next_record()
            .unwrap()
            .is_some()
    );
    assert_eq!(expect_err(parser.next_record()).kind(), ErrorKind::InvalidData);
}

#[test]
fn aslparser_inline_strings_use_no_heap_storage() {
    let record = parser(database_with_string_reference([0; 8]))
        .next_record()
        .unwrap()
        .unwrap();
    for text in record_strings(&record) {
        assert!(text.is_empty());
        assert!(!text.is_heap_allocated());
    }

    for expected in [
        "", "a", "ab", "abc", "abcd", "abcde", "abcdef", "abcdefg", "é☃ab", "a\0b",
    ] {
        let mut reference = [0; 8];
        reference[0] = 0x80 | expected.len() as u8;
        reference[1..1 + expected.len()].copy_from_slice(expected.as_bytes());
        let record = parser(database_with_string_reference(reference))
            .next_record()
            .unwrap()
            .unwrap();
        for text in record_strings(&record) {
            assert_eq!(text, expected);
            assert!(!text.is_heap_allocated(), "{expected:?}");
        }
    }
}

#[test]
fn aslparser_referenced_strings_preserve_inline_and_heap_values() {
    let inline_capacity = std::mem::size_of::<String>();
    assert_eq!(std::mem::size_of::<CompactString>(), inline_capacity);
    for expected in [
        String::new(),
        "x".repeat(inline_capacity),
        "x".repeat(inline_capacity + 1),
        "é☃".repeat(1000),
        "before\0after".into(),
    ] {
        let offset = database_with_string_reference([0; 8]).len() as u64;
        let mut bytes = database_with_string_reference(offset.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());
        bytes.extend_from_slice(&((expected.len() + 1) as u32).to_be_bytes());
        bytes.extend_from_slice(expected.as_bytes());
        bytes.push(0);
        let record = parser(bytes)
            .next_record()
            .unwrap()
            .unwrap();
        for text in record_strings(&record) {
            assert_eq!(text.as_str(), expected);
            assert_eq!(text.is_heap_allocated(), expected.len() > inline_capacity);
        }
    }
}

#[test]
fn aslparser_referenced_strings_reject_invalid_utf8_and_missing_terminator() {
    for (payload, expected) in [
        (&[0xff, 0][..], "invalid ASL UTF-8"),
        (&[b'x'][..], "unterminated ASL string"),
    ] {
        let offset = database_with_string_reference([0; 8]).len() as u64;
        let mut bytes = database_with_string_reference(offset.to_be_bytes());
        bytes.extend_from_slice(&1u16.to_be_bytes());
        bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        bytes.extend_from_slice(payload);
        assert_record_error(bytes, expected);
    }
}

#[test]
fn aslparser_inline_and_invalid_string_references() {
    let mut bytes = fixture_bytes();
    let first = first_offset(&bytes);
    let sender_ref = first + 6 + 68;
    bytes[sender_ref..sender_ref + 8].copy_from_slice(&[
        0x83, b'A', b'B', b'C', 0, 0, 0, 0,
    ]);
    assert_eq!(
        parser(bytes)
            .next_record()
            .unwrap()
            .unwrap()
            .sender,
        "ABC"
    );

    let mut bad = fixture_bytes();
    let first = first_offset(&bad);
    let host_ref = first + 6 + 60;
    let beyond_end = bad.len() as u64 + 1;
    bad[host_ref..host_ref + 8].copy_from_slice(&beyond_end.to_be_bytes());
    assert_record_error(bad, "ASL string offset");

    for (reference, expected) in [
        (
            [
                0x88, 0, 0, 0, 0, 0, 0, 0,
            ],
            "invalid inline ASL string length",
        ),
        (
            [
                0x81, 0xff, 0, 0, 0, 0, 0, 0,
            ],
            "invalid inline ASL UTF-8",
        ),
    ] {
        let mut bad = fixture_bytes();
        let first = first_offset(&bad);
        bad[first + 6 + 68..first + 6 + 76].copy_from_slice(&reference);
        assert_record_error(bad, expected);
    }

    let mut bad = fixture_bytes();
    let first = first_offset(&bad);
    let string_ref = u64::from_be_bytes(
        bad[first + 6 + 60..first + 6 + 68]
            .try_into()
            .unwrap(),
    ) as usize;
    bad[string_ref] = 2;
    assert_record_error(bad, "invalid ASL string tag");

    let mut bad = fixture_bytes();
    let first = first_offset(&bad);
    let string_ref = u64::from_be_bytes(
        bad[first + 6 + 60..first + 6 + 68]
            .try_into()
            .unwrap(),
    ) as usize;
    bad[string_ref + 2..string_ref + 6].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_record_error(bad, "invalid ASL string length");
}
