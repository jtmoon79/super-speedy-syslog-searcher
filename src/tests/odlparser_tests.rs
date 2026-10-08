// src/tests/odlparser_tests.rs

//! tests for `odlparser.rs` and rendered `odl.rs` events

#![allow(non_snake_case)]

use std::fs::File;
use std::io::{
    Cursor,
    ErrorKind,
};
use std::path::Path;

use aes::Aes128;
use base64::Engine;
use base64::engine::general_purpose::{
    STANDARD,
    URL_SAFE_NO_PAD,
};
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{
    BlockEncryptMut,
    KeyIvInit,
};

use crate::data::common::PrintableEvent;
use crate::data::datetime::FixedOffset;
use crate::data::odl::{
    Odl,
    OdlEvent,
    single_line,
};
use crate::readers::odlparser::{
    ODL_COMPANION_BYTES_MAX,
    ODL_RECORD_BYTES_MAX,
    OdlDecodingContext,
    OdlParser,
    OdlRecordError,
};
use crate::tests::common::{
    FO_M7,
    NTF_LOG_EMPTY_FPATH,
    ODL_NUCLEUS_ODLGZ_DATA,
    ODL_NUCLEUS_ODLGZ_PATH,
    ODL_NUCLEUSLOCAL_AODL_DATA,
    ODL_NUCLEUSLOCAL_AODL_PATH,
    ODL_SYNCENGINE_ODL_DATA,
    ODL_SYNCENGINE_ODL_PATH,
    OdlExpectedLine,
};

fn decoding_for(path: &str) -> OdlDecodingContext {
    let keystore = Path::new(path)
        .parent()
        .unwrap()
        .join("general.keystore");
    let file = File::open(keystore).unwrap();
    let mut decoding = OdlDecodingContext::default();
    decoding
        .load_keystore(file)
        .unwrap();
    decoding
}

fn assert_rendered(
    actual: &Odl,
    expected: &OdlExpectedLine,
) {
    assert_eq!(actual.dt(), &expected.dt);
    assert_eq!(actual.dt_beg_end(), &Some((0, 29)));
    assert_eq!(
        actual
            .as_bytes()
            .strip_suffix(b"\n"),
        Some(expected.line)
    );
}

fn assert_parser_fixture(
    path: &str,
    compressed: bool,
    expected: &[OdlExpectedLine],
) {
    let file = File::open(path).unwrap();
    let mut parser = OdlParser::new(file, decoding_for(path)).unwrap();
    assert_eq!(parser.header().version, 3);
    assert_eq!(
        parser
            .header()
            .one_drive_version,
        "25.222.1112.0002"
    );
    assert_eq!(
        parser
            .header()
            .platform_version,
        "10.0.26100"
    );
    assert_eq!(parser.header().compressed, compressed);

    let mut actual: Vec<Odl> = Vec::new();
    while let Some(result) = parser.next_event() {
        let event = result.expect("ODL fixture should not contain record errors");
        assert_eq!(event.decoding_failures, 0);
        actual.push(
            event
                .render(&FO_M7)
                .expect("ODL fixture timestamp should be representable"),
        );
    }

    assert_eq!(actual.len(), expected.len(), "event count");
    for (actual, expected) in actual
        .iter()
        .zip(expected.iter())
    {
        assert_rendered(actual, expected);
    }
    assert_eq!(parser.records_skipped, 0);
}

#[test]
fn test_odlparser_nucleus_odlgz() {
    assert_parser_fixture(ODL_NUCLEUS_ODLGZ_PATH, true, &ODL_NUCLEUS_ODLGZ_DATA);
}

#[test]
fn test_odlparser_syncengine_odl() {
    assert_parser_fixture(ODL_SYNCENGINE_ODL_PATH, false, &ODL_SYNCENGINE_ODL_DATA);
}

#[test]
fn test_odlparser_nucleuslocal_aodl() {
    assert_parser_fixture(ODL_NUCLEUSLOCAL_AODL_PATH, false, &ODL_NUCLEUSLOCAL_AODL_DATA);
}

#[test]
fn test_odlparser_file_empty() {
    let file = File::open(&*NTF_LOG_EMPTY_FPATH).unwrap();
    let error = expect_err(OdlParser::new(file, OdlDecodingContext::default()));

    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
}

fn sample_event(
    timestamp_ms: u64,
    source_file: &str,
    function: &str,
    parameters: &[&str],
) -> OdlEvent {
    OdlEvent {
        timestamp_ms,
        ordinal: 1,
        offset: 0,
        source_file: source_file.to_owned(),
        function: function.to_owned(),
        flags: 0,
        context: Vec::new(),
        parameter_bytes: Vec::new(),
        parameters: parameters
            .iter()
            .map(|parameter| (*parameter).to_owned())
            .collect(),
        undecoded_bytes: 0,
        decoding_failures: 0,
    }
}

#[test]
fn test_single_line_replaces_controls() {
    assert_eq!(single_line("a\nb\tc"), "a b c");
    assert_eq!(single_line("plain"), "plain");
}

#[test]
fn test_odl_render_line_and_offset() {
    let rendered = sample_event(0, "a\nb", "c\td", &["param"])
        .render(&FO_M7)
        .unwrap();
    assert_eq!(rendered.as_bytes(), b"1969-12-31T17:00:00.000-07:00 a b:c d; param\n");
    assert_eq!(rendered.dt_beg_end(), &Some((0, 29)));
    assert_eq!(rendered.len(), rendered.as_bytes().len());
    assert!(!rendered.is_empty());
    assert_eq!(PrintableEvent::dt(&rendered), rendered.dt());
    assert_eq!(PrintableEvent::dt_beg_end(&rendered), rendered.dt_beg_end());
    assert_eq!(PrintableEvent::as_bytes(&rendered), rendered.as_bytes());

    let east = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
    let rendered = sample_event(0, "src.cpp", "Func", &[])
        .render(&east)
        .unwrap();
    assert_eq!(rendered.as_bytes(), b"1970-01-01T05:30:00.000+05:30 src.cpp:Func;\n");

    let rounded_up = FixedOffset::east_opt(30).unwrap();
    assert_eq!(
        sample_event(0, "s", "f", &[])
            .render(&rounded_up)
            .unwrap()
            .as_bytes(),
        b"1970-01-01T00:00:30.000+00:01 s:f;\n"
    );
    let rounded_down = FixedOffset::east_opt(29).unwrap();
    assert_eq!(
        sample_event(0, "s", "f", &[])
            .render(&rounded_down)
            .unwrap()
            .as_bytes(),
        b"1970-01-01T00:00:29.000+00:00 s:f;\n"
    );
}

#[test]
fn test_odl_render_rejects_unrepresentable_timestamp() {
    let exceeds = sample_event(u64::MAX, "s", "f", &[])
        .render(&FO_M7)
        .unwrap_err();
    assert_eq!(exceeds.kind(), ErrorKind::InvalidData);
    assert_eq!(exceeds.to_string(), "ODL timestamp exceeds i64");

    let unrepresentable = sample_event(i64::MAX as u64, "s", "f", &[])
        .render(&FO_M7)
        .unwrap_err();
    assert_eq!(unrepresentable.kind(), ErrorKind::InvalidData);
    assert_eq!(unrepresentable.to_string(), "unrepresentable ODL timestamp");
}

fn odl_file(
    version: u32,
    records: &[&[u8]],
) -> Vec<u8> {
    let mut data = vec![0u8; 256];
    data[..8].copy_from_slice(b"EBFGONED");
    data[8..12].copy_from_slice(&version.to_le_bytes());
    data[28..32].copy_from_slice(b"1.0\0");
    data[92..95].copy_from_slice(b"2.0");
    for record in records {
        data.extend_from_slice(record);
    }
    data
}

fn push_string(
    out: &mut Vec<u8>,
    text: &[u8],
) {
    let length = u32::try_from(text.len()).unwrap();
    out.extend_from_slice(&length.to_le_bytes());
    out.extend_from_slice(text);
}

fn v3_record(
    timestamp_ms: u64,
    source: &[u8],
    function: &[u8],
    parameters: &[&[u8]],
) -> Vec<u8> {
    let mut payload = vec![0u8; 24];
    push_string(&mut payload, source);
    payload.extend_from_slice(&0u32.to_le_bytes());
    push_string(&mut payload, function);
    for parameter in parameters {
        push_string(&mut payload, parameter);
    }
    let mut record = vec![0u8; 32];
    record[..4].copy_from_slice(&[
        0xcc, 0xdd, 0xee, 0xff,
    ]);
    record[8..16].copy_from_slice(&timestamp_ms.to_le_bytes());
    record[24..28].copy_from_slice(
        &u32::try_from(payload.len())
            .unwrap()
            .to_le_bytes(),
    );
    record.extend(payload);
    record
}

fn v2_record(
    timestamp_ms: u64,
    source: &[u8],
    function: &[u8],
) -> Vec<u8> {
    let mut payload = Vec::new();
    push_string(&mut payload, source);
    payload.extend_from_slice(&7u32.to_le_bytes());
    push_string(&mut payload, function);
    let mut record = vec![0u8; 56];
    record[..4].copy_from_slice(&[
        0xcc, 0xdd, 0xee, 0xff,
    ]);
    record[8..16].copy_from_slice(&timestamp_ms.to_le_bytes());
    record[24] = 0xab;
    record[48..52].copy_from_slice(
        &u32::try_from(payload.len())
            .unwrap()
            .to_le_bytes(),
    );
    record.extend(payload);
    record
}

fn expect_err<T>(result: std::io::Result<T>) -> std::io::Error {
    match result {
        Ok(_) => panic!("expected ODL parser error"),
        Err(error) => error,
    }
}

fn parser_of(data: Vec<u8>) -> OdlParser<Cursor<Vec<u8>>> {
    OdlParser::new(Cursor::new(data), OdlDecodingContext::default()).unwrap()
}

#[test]
fn test_odlparser_rejects_bad_signature_and_version() {
    let mut bad_magic = odl_file(3, &[]);
    bad_magic[..4].copy_from_slice(b"XXXX");
    let error = expect_err(OdlParser::new(Cursor::new(bad_magic), OdlDecodingContext::default()));
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "invalid ODL file signature");

    let error = expect_err(OdlParser::new(Cursor::new(odl_file(1, &[])), OdlDecodingContext::default()));
    assert_eq!(error.kind(), ErrorKind::Unsupported);
    assert_eq!(error.to_string(), "unsupported ODL version 1");
}

#[test]
fn test_odlparser_fatal_record_stops_stream() {
    let mut bad = vec![0u8; 32];
    bad[..4].copy_from_slice(b"NOPE");
    let mut odl = parser_of(odl_file(3, &[bad.as_slice()]));
    match odl.next_event() {
        Some(Err(OdlRecordError::Fatal { ordinal, offset, error })) => {
            assert_eq!(ordinal, 1);
            assert_eq!(offset, 0);
            assert_eq!(error.kind(), ErrorKind::InvalidData);
            assert_eq!(error.to_string(), "invalid ODL record signature: expected [204, 221, 238, 255], found [78, 79, 80, 69]");
        }
        other => panic!("expected fatal record error, got {other:?}"),
    }
    assert!(odl.next_event().is_none());

    let mut oversized = vec![0u8; 32];
    oversized[..4].copy_from_slice(&[
        0xcc, 0xdd, 0xee, 0xff,
    ]);
    let length = u32::try_from(ODL_RECORD_BYTES_MAX + 1).unwrap();
    oversized[24..28].copy_from_slice(&length.to_le_bytes());
    let mut odl = parser_of(odl_file(3, &[oversized.as_slice()]));
    match odl.next_event() {
        Some(Err(OdlRecordError::Fatal { error, .. })) => {
            assert_eq!(error.to_string(), "ODL record length 65537 exceeds size limit 65536");
        }
        other => panic!("expected fatal size error, got {other:?}"),
    }

    let mut odl = parser_of(odl_file(3, &[&[0xcc]]));
    match odl.next_event() {
        Some(Err(OdlRecordError::Fatal { error, .. })) => {
            assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
        }
        other => panic!("expected truncated fatal error, got {other:?}"),
    }
    assert!(odl.next_event().is_none());
}

#[test]
fn test_odlparser_skipped_record_does_not_stop_stream() {
    let skipped = v3_record(0, &[0xff], b"Func", &[]);
    let kept = v3_record(0, b"src.cpp", b"Func", &[b"ok"]);
    let mut odl = parser_of(odl_file(
        3,
        &[
            skipped.as_slice(),
            kept.as_slice(),
        ],
    ));

    match odl.next_event() {
        Some(Err(OdlRecordError::Skipped { ordinal, offset, error })) => {
            assert_eq!(ordinal, 1);
            assert_eq!(offset, 0);
            assert_eq!(error.kind(), ErrorKind::InvalidData);
        }
        other => panic!("expected skipped record, got {other:?}"),
    }
    let event = odl
        .next_event()
        .unwrap()
        .expect("record after a skipped record");
    assert_eq!(event.ordinal, 2);
    assert_eq!(event.source_file, "src.cpp");
    assert_eq!(event.function, "Func");
    assert_eq!(event.parameters, ["ok"]);
    assert!(odl.next_event().is_none());
    assert_eq!(odl.records_skipped, 1);
}

#[test]
fn test_odlparser_version2_record() {
    let record = v2_record(0, b"src.cpp", b"Func");
    let mut odl = parser_of(odl_file(2, &[record.as_slice()]));
    assert_eq!(odl.header().version, 2);
    assert!(!odl.header().compressed);
    assert_eq!(odl.header().one_drive_version, "1.0");
    assert_eq!(odl.header().platform_version, "2.0");

    let event = odl
        .next_event()
        .unwrap()
        .expect("version 2 record");
    assert_eq!(event.source_file, "src.cpp");
    assert_eq!(event.function, "Func");
    assert_eq!(event.flags, 7);
    assert_eq!(event.context.len(), 24);
    assert_eq!(event.context[0], 0xab);
    assert!(odl.next_event().is_none());
}

fn utf16_token(
    key: &[u8],
    text: &str,
) -> String {
    let mut plain: Vec<u8> = text
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();
    let length = plain.len();
    plain.resize(length + 16, 0);
    let encrypted = cbc::Encryptor::<Aes128>::new_from_slices(key, &[0u8; 16])
        .unwrap()
        .encrypt_padded_mut::<Pkcs7>(&mut plain, length)
        .unwrap();
    URL_SAFE_NO_PAD.encode(encrypted)
}

#[test]
fn test_odl_decoding_context_map_and_keystore() {
    let mut decoding = OdlDecodingContext::default();
    assert!(!decoding.has_companions());

    let map = b"k\tv1\nk\tv2\ncontinuation\n";
    decoding
        .load_map(Cursor::new(map.as_slice()))
        .unwrap();
    assert!(decoding.has_companions());

    let record = v3_record(0, b"s", b"f", &[b"k"]);
    let mut parser = OdlParser::new(Cursor::new(odl_file(3, &[record.as_slice()])), decoding).unwrap();
    let event = parser
        .next_event()
        .unwrap()
        .unwrap();
    assert_eq!(event.parameters, ["v1|v2 continuation"]);
    assert_eq!(event.decoding_failures, 0);

    let error = OdlDecodingContext::default()
        .load_map(Cursor::new(b"\tvalue\n".as_slice()))
        .unwrap_err();
    assert_eq!(error.to_string(), "empty ODL obfuscation-map key at line 0");

    let error = OdlDecodingContext::default()
        .load_map(Cursor::new(b"continuation\n".as_slice()))
        .unwrap_err();
    assert_eq!(error.to_string(), "ODL map continuation without a key");

    let mut utf16 = vec![0xff, 0xfe];
    for unit in "alpha\tbeta\n".encode_utf16() {
        utf16.extend_from_slice(&unit.to_le_bytes());
    }
    let mut decoding = OdlDecodingContext::default();
    decoding
        .load_map(Cursor::new(utf16))
        .unwrap();
    let record = v3_record(0, b"s", b"f", &[b"alpha"]);
    let event = OdlParser::new(Cursor::new(odl_file(3, &[record.as_slice()])), decoding)
        .unwrap()
        .next_event()
        .unwrap()
        .unwrap();
    assert_eq!(event.parameters, ["beta"]);

    let error = OdlDecodingContext::default()
        .load_map(Cursor::new(vec![b'a'; ODL_COMPANION_BYTES_MAX + 1]))
        .unwrap_err();
    assert_eq!(error.to_string(), "ODL companion size 16777217 exceeds size limit 16777216");

    let error = OdlDecodingContext::default()
        .load_keystore(Cursor::new(b"{}".as_slice()))
        .unwrap_err();
    assert_eq!(error.to_string(), "ODL keystore must be an array");

    let error = OdlDecodingContext::default()
        .load_keystore(Cursor::new(br#"[{"Version":2,"Key":"AAAA"}]"#.as_slice()))
        .unwrap_err();
    assert_eq!(error.to_string(), "unsupported ODL keystore version Some(2)");

    let short_key = STANDARD.encode([1u8; 15]);
    let error = OdlDecodingContext::default()
        .load_keystore(Cursor::new(format!(r#"[{{"Version":1,"Key":"{short_key}"}}]"#)))
        .unwrap_err();
    assert_eq!(error.to_string(), "unsupported ODL AES key length 15");

    let key = b"KEYMATERIAL12345";
    let mut decoding = OdlDecodingContext::default();
    let key_b64 = STANDARD.encode(key);
    decoding
        .load_keystore(Cursor::new(format!(r#"[{{"Version":1,"Key":"{key_b64}"}}]"#)))
        .unwrap();
    assert!(decoding.has_companions());
    let debug = format!("{decoding:?}");
    assert!(!debug.contains("KEYMATERIAL"));
    assert!(!debug.contains(&key_b64));
    assert!(debug.contains("keys"));

    let token = utf16_token(key, "opened");
    assert!(token.len() >= 22);
    let record = v3_record(0, b"s", b"f", &[token.as_bytes()]);
    let event = OdlParser::new(Cursor::new(odl_file(3, &[record.as_slice()])), decoding)
        .unwrap()
        .next_event()
        .unwrap()
        .unwrap();
    assert_eq!(event.parameters, ["opened"]);
    assert_eq!(event.decoding_failures, 0);

    let undecoded = URL_SAFE_NO_PAD.encode([0u8; 16]);
    let mut decoding = OdlDecodingContext::default();
    decoding
        .load_keystore(Cursor::new(format!(r#"[{{"Version":1,"Key":"{key_b64}"}}]"#)))
        .unwrap();
    let record = v3_record(0, b"s", b"f", &[undecoded.as_bytes()]);
    let event = OdlParser::new(Cursor::new(odl_file(3, &[record.as_slice()])), decoding)
        .unwrap()
        .next_event()
        .unwrap()
        .unwrap();
    assert_eq!(event.parameters, [undecoded.as_str()]);
    assert_eq!(event.decoding_failures, 1);
}
