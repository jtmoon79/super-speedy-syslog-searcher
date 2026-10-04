// src/tests/etlparser_tests.rs

//! tests for `etlparser.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

use std::fs::File;
use std::io::{
    Cursor,
    ErrorKind,
};

use crate::data::datetime::FixedOffset;
use crate::data::etl::{
    Etl,
    EtlName,
    EtlPayload,
    EtlValue,
    Guid,
};
use crate::readers::etlparser::{
    ClockType,
    EtlParser,
    LogfileHeader,
    TlCount,
    TlSchema,
    decode_kernel_group0,
    decode_tracelogging,
    kernel_group_lookup,
    read_u16_at,
    read_u32_at,
    read_u64_at,
    read_full,
    tl_parse_schema,
    tl_schema_cached,
    tl_read_sid,
    utf16le_to_string,
    TLG_IN_FLAG_CCOUNT,
    TLG_IN_FLAG_CHAIN,
    TLG_IN_NULL,
    TLG_IN_STRUCT,
    TLG_IN_UINT8,
    TL_ARRAY_LEN_MAX,
    TL_EVENT_VALUES_MAX,
};
use crate::tests::common::{
    NTF_LOG_EMPTY_FPATH,
    ETL_FILE1_PATH,
    ETL_FILE1_DATA,
};

#[allow(non_upper_case_globals)]
const FO_m7: FixedOffset = FixedOffset::east_opt(-7 * 3600).unwrap();

/// create a `EtlParser` for the file at `ETL_FILE1_PATH`
/// process the Events, compare them against expected results
#[test]
fn test_etlparser_file1() {
    let file = File::open(ETL_FILE1_PATH).unwrap();
    let mut parser = EtlParser::new(file).unwrap();
    let mut actual: Vec<Etl> = Vec::new();

    while let Some(result) = parser.next_event() {
        let event = result.expect("ETL fixture should not contain record errors");
        let (dt, mut data, dt_beg_end) = event
            .render(&FO_m7)
            .expect("ETL event timestamp should be representable");
        assert_eq!(data.pop(), Some(b'\n'), "rendered event line terminator");
        actual.push(Etl::new(dt, Some(dt_beg_end), data));
    }

    let mut expected: Vec<Etl> = ETL_FILE1_DATA.clone();
    actual.sort_by(|a, b| a.partial_cmp(b).unwrap());
    expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(actual.len(), expected.len(), "event count");
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual.dt(), expected.dt());
        assert_eq!(actual.dt_beg_end(), expected.dt_beg_end());
        assert_eq!(actual.as_bytes(), expected.as_bytes());
    }
}

/// Have `EtlParser` handle an empty file gracefully
#[test]
fn test_etlparser_file_empty() {
    let file = File::open(&*NTF_LOG_EMPTY_FPATH).unwrap();
    let error = EtlParser::new(file).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
}

fn logfileheader1() -> LogfileHeader {
    LogfileHeader {
        buffer_size: 4096,
        version: 1,
        provider_version: 1,
        number_of_processors: 1,
        end_time: 0,
        timer_resolution: 0,
        maximum_file_size: 0,
        log_file_mode: 0,
        buffers_written: 0,
        start_buffers: 0,
        pointer_size: 8,
        events_lost: 0,
        cpu_speed_mhz: 0,
        logger_name_ptr: 0,
        log_file_name_ptr: 0,
        boot_time: 0,
        perf_freq: 10_000_000,
        start_time: crate::data::etl::FILETIME_UNIX_EPOCH,
        reserved_flags: 2,
        buffers_lost: 0,
        session_name: String::from("test-session"),
        log_file_name: String::from("test.etl"),
        clock_type: ClockType::SystemTime,
        header_time_delta: 0,
        is_64bit: true,
    }
}

#[test]
fn test_utf16le_to_string() {
    assert_eq!(utf16le_to_string(&[b'H', 0, b'i', 0, 0, 0, b'!']), "Hi");
    assert_eq!(utf16le_to_string(&[0x00, 0xD8, 0, 0]), "\u{FFFD}");
}

#[test]
fn test_read_u16_at() {
    let data = [0x34, 0x12, 0x78, 0x56];

    assert_eq!(read_u16_at(&data, 0), Some(0x1234));
    assert_eq!(read_u16_at(&data, 2), Some(0x5678));
    assert_eq!(read_u16_at(&data, 3), None);
}

#[test]
fn test_read_u32_at() {
    let data = [0x78, 0x56, 0x34, 0x12, 0xAA];

    assert_eq!(read_u32_at(&data, 0), Some(0x1234_5678));
    assert_eq!(read_u32_at(&data, 2), None);
}

#[test]
fn test_read_u64_at() {
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9];

    assert_eq!(read_u64_at(&data, 0), Some(0x0807_0605_0403_0201));
    assert_eq!(read_u64_at(&data, 2), None);
}

#[test]
fn test_read_full() {
    let mut reader = Cursor::new(b"complete".as_slice());
    let mut buffer = [0; 8];
    assert_eq!(read_full(&mut reader, &mut buffer).unwrap(), buffer.len());
    assert_eq!(&buffer, b"complete");

    let mut reader = Cursor::new(b"short".as_slice());
    let mut buffer = [0; 8];
    assert_eq!(read_full(&mut reader, &mut buffer).unwrap(), 5);
    assert_eq!(&buffer[..5], b"short");
}

#[test]
fn test_tl_parse_schema() {
    let mut schema = vec![
        0, 0, // schema size, filled below
        0, // extension chain terminator
        b'E', b'v', b't', 0,
        b'L', b'e', b'v', b'e', b'l', 0, TLG_IN_UINT8,
        b'T', b'a', b'g', 0, TLG_IN_UINT8 | TLG_IN_FLAG_CCOUNT, 3, 0,
    ];
    let size = schema.len() as u16;
    schema[..2].copy_from_slice(&size.to_le_bytes());

    let parsed = tl_parse_schema(&schema).expect("valid TraceLogging schema");
    assert_eq!(&*parsed.event_name, "Evt");
    assert_eq!(parsed.fields.len(), 2);
    assert_eq!(&*parsed.fields[0].name, "Level");
    assert_eq!(parsed.fields[0].in_type, TLG_IN_UINT8);
    assert_eq!(parsed.fields[0].count, TlCount::Scalar);
    assert_eq!(&*parsed.fields[1].name, "Tag");
    assert_eq!(parsed.fields[1].in_type, TLG_IN_UINT8);
    assert_eq!(parsed.fields[1].count, TlCount::Fixed(3));
    assert!(tl_parse_schema(&[0, 0]).is_none());
}

fn push_cstr(
    out: &mut Vec<u8>,
    s: &str,
) {
    out.extend_from_slice(s.as_bytes());
    out.push(0);
}

fn push_fixed(
    out: &mut Vec<u8>,
    name: &str,
    in_type: u8,
    count: u16,
) {
    push_cstr(out, name);
    out.push(in_type | TLG_IN_FLAG_CCOUNT);
    out.extend_from_slice(&count.to_le_bytes());
}

fn push_struct_fixed(
    out: &mut Vec<u8>,
    name: &str,
    members: u8,
    count: u16,
) {
    push_cstr(out, name);
    out.push(TLG_IN_STRUCT | TLG_IN_FLAG_CHAIN | TLG_IN_FLAG_CCOUNT);
    out.push(members);
    out.extend_from_slice(&count.to_le_bytes());
}

fn schema_from_fields(field_bytes: &[u8]) -> TlSchema {
    let mut schema: Vec<u8> = vec![0, 0, 0];
    push_cstr(&mut schema, "Evt");
    schema.extend_from_slice(field_bytes);
    let size: u16 = schema.len() as u16;
    schema[..2].copy_from_slice(&size.to_le_bytes());

    tl_parse_schema(&schema).expect("valid TraceLogging schema")
}

fn assert_decode_failed(payload: &EtlPayload) {
    assert!(
        matches!(payload, EtlPayload::Partial(_, _) | EtlPayload::Raw(_)),
        "over-budget decode must fail closed, got {:?}",
        payload,
    );
}

/// In-budget arrays decode; over-limit and nested over-budget arrays fail closed.
#[test]
fn test_decode_tracelogging_array_budget() {
    let mut fields: Vec<u8> = Vec::new();
    push_fixed(&mut fields, "Tag", TLG_IN_UINT8, 3);
    let schema: TlSchema = schema_from_fields(&fields);
    assert_eq!(
        decode_tracelogging(&schema, &[1, 2, 3], 8),
        EtlPayload::Fields(vec![(
            EtlName::from("Tag"),
            EtlValue::Array(vec![
                EtlValue::U64(1),
                EtlValue::U64(2),
                EtlValue::U64(3),
            ]),
        )])
    );

    let mut fields: Vec<u8> = Vec::new();
    push_struct_fixed(&mut fields, "S", 1, 2);
    push_fixed(&mut fields, "N", TLG_IN_NULL, 2);
    let schema: TlSchema = schema_from_fields(&fields);
    let nulls: EtlValue = EtlValue::Array(vec![EtlValue::Null, EtlValue::Null]);
    assert_eq!(
        decode_tracelogging(&schema, &[], 8),
        EtlPayload::Fields(vec![(
            EtlName::from("S"),
            EtlValue::Array(vec![
                EtlValue::Struct(vec![(EtlName::from("N"), nulls.clone())]),
                EtlValue::Struct(vec![(EtlName::from("N"), nulls)]),
            ]),
        )])
    );

    // 4097 is within the event budget, so only the per-array cap rejects it.
    let mut fields: Vec<u8> = Vec::new();
    push_fixed(&mut fields, "N", TLG_IN_NULL, (TL_ARRAY_LEN_MAX as u16) + 1);
    assert_decode_failed(&decode_tracelogging(&schema_from_fields(&fields), &[], 8));

    let mut fields: Vec<u8> = Vec::new();
    push_fixed(&mut fields, "N", TLG_IN_NULL, u16::MAX);
    assert_decode_failed(&decode_tracelogging(&schema_from_fields(&fields), &[], 8));

    // Each array is within the per-array cap; the product exceeds the event budget.
    // Without the shared budget this allocates TL_ARRAY_LEN_MAX squared Nulls.
    assert!(TL_ARRAY_LEN_MAX * TL_ARRAY_LEN_MAX > TL_EVENT_VALUES_MAX);
    let mut fields: Vec<u8> = Vec::new();
    push_struct_fixed(&mut fields, "S", 1, TL_ARRAY_LEN_MAX as u16);
    push_fixed(&mut fields, "N", TLG_IN_NULL, TL_ARRAY_LEN_MAX as u16);
    assert_decode_failed(&decode_tracelogging(&schema_from_fields(&fields), &[], 8));
}

#[test]
fn test_kernel_group_lookup() {
    let (disk_io_guid, disk_io_name) = kernel_group_lookup(0x01, 0);
    assert_eq!(disk_io_name, Some("DiskIo"));
    assert_eq!(
        disk_io_guid,
        Guid::from_le_bytes([
            0xD4, 0xA8, 0x6F, 0x3D, 0x05, 0xFE, 0xD0, 0x11, 0x9D, 0xDA, 0x00, 0xC0, 0x4F, 0xD7, 0xBA, 0x7C,
        ])
    );

    let (_, image_name) = kernel_group_lookup(0x03, 10);
    assert_eq!(image_name, Some("Image"));

    let (unknown_guid, unknown_name) = kernel_group_lookup(u8::MAX, 0);
    assert_eq!(unknown_guid, Guid::NIL);
    assert_eq!(unknown_name, None);
}

/// Test supported and unsupported kernel group 0 payloads.
#[test]
fn test_decode_kernel_group0() {
    let header = logfileheader1();
    let decoded = decode_kernel_group0(66, b"build string\0", &header);
    assert_eq!(
        decoded,
        Some(EtlPayload::Fields(vec![(
            EtlName::from("BuildString"),
            EtlValue::Str(String::from("build string")),
        )]))
    );
    assert!(matches!(
        decode_kernel_group0(0, &[], &header),
        Some(EtlPayload::Fields(_))
    ));
    assert_eq!(decode_kernel_group0(1, &[], &header), None);
}
