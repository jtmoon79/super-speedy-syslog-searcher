//! Tests for rendering a parsed Apple System Log record.

use std::fs::File;
use std::io::ErrorKind;

use crate::data::common::PrintableEvent;
use crate::readers::aslparser::AslParser;
use crate::tests::common::{ASL_FIXTURES, FO_0, FO_W8};

#[test]
fn asl_render_timestamp_fields_and_control_characters() {
    let mut record = AslParser::new(File::open(ASL_FIXTURES[0].0).unwrap())
        .unwrap()
        .next_record()
        .unwrap()
        .unwrap();
    record.seconds = 0;
    record.nanoseconds = 123_456_789;
    record.level = 6;
    record.ref_pid = 42;
    record.host = "host\nname".into();
    record.message = "first\nsecond\tline".into();
    record.extra = vec![("field".into(), "value\rnext".into())];
    let asl = record.render(&FO_0).unwrap();
    let line = std::str::from_utf8(asl.as_bytes()).unwrap();
    assert!(line.starts_with("1970-01-01T00:00:00.123456789+00:00  id=208037  level=Info  "));
    assert!(line.contains("  ref_pid=42  "));
    assert!(line.contains("  host=host name  "));
    assert!(line.contains("  message='first second line'  field=value next\n"));
    assert_eq!(line.lines().count(), 1);
    assert_eq!(asl.dt_beg_end(), &Some((0, 35)));
    assert_eq!(asl.len(), asl.as_bytes().len());
    assert!(!asl.is_empty());
    assert_eq!(PrintableEvent::dt(&asl), asl.dt());
    assert_eq!(PrintableEvent::dt_beg_end(&asl), asl.dt_beg_end());
    assert_eq!(PrintableEvent::as_bytes(&asl), asl.as_bytes());

    let west = record.render(&FO_W8).unwrap();
    assert!(
        west.as_bytes()
            .starts_with(b"1969-12-31T16:00:00.123456789-08:00")
    );
}

#[test]
fn asl_render_preserves_complete_output_with_and_without_overflow() {
    let mut record = AslParser::new(File::open(ASL_FIXTURES[0].0).unwrap())
        .unwrap()
        .next_record()
        .unwrap()
        .unwrap();
    record.seconds = 0;
    record.nanoseconds = 123_456_789;
    record.id = 9;
    record.level = 8;
    record.pid = 1;
    record.uid = 2;
    record.gid = 3;
    record.read_uid = 4;
    record.read_gid = 5;
    record.ref_pid = 6;
    record.flags = 0xbeef;
    record.host = "h".into();
    record.ref_proc = "r".into();
    record.session = "s".into();
    record.facility = "f".into();
    record.message = "one\nétwo\t☃".into();
    record.extra = vec![("k\né".into(), "v\t☃".into())];

    for sender in [
        "é".to_owned(),
        "é".repeat(500),
    ] {
        record.sender = sender.into();
        let rendered = record.render(&FO_0).unwrap();
        let expected = format!(
            "1970-01-01T00:00:00.123456789+00:00  id=9  level=Other  pid=1  uid=2  gid=3  read_uid=4  read_gid=5  ref_pid=6  flags=0xbeef  host=h  RefProc=r  session=s  sender={}  facility=f  message='one étwo ☃'  k é=v ☃\n",
            record.sender
        );
        assert_eq!(rendered.as_bytes(), expected.as_bytes());
        assert_eq!(rendered.len(), expected.len());
        assert_eq!(rendered.dt_beg_end(), &Some((0, 35)));
    }
}

#[test]
fn asl_render_rejects_invalid_timestamps() {
    let mut record = AslParser::new(File::open(ASL_FIXTURES[0].0).unwrap())
        .unwrap()
        .next_record()
        .unwrap()
        .unwrap();
    record.seconds = u64::MAX;
    let error = record
        .render(&FO_0)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "ASL timestamp exceeds i64");

    record.seconds = 0;
    record.nanoseconds = 1_000_000_000;
    let error = record
        .render(&FO_0)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "unrepresentable ASL timestamp");
}
