// src/readers/etlparser.rs

//! Parse a Windows [Event Trace Log] (`.etl`) file from any [`Read`] source.
//!
//! An ETL file is a sequence of buffers. Each buffer begins with a
//! `WMI_BUFFER_HEADER` (0x48 bytes) followed by 8-byte aligned event records.
//! The first record of the first buffer is a kernel `EventTrace/Header` event
//! whose payload is a `TRACE_LOGFILE_HEADER`; it supplies the session start
//! time and clock type needed to convert every record's relative timestamp
//! to a `FILETIME`.
//!
//! Record payloads are decoded when possible:
//! - TraceLogging events carry their own schema (`EVENT_HEADER_EXT_TYPE_EVENT_SCHEMA_TL`)
//!   and are fully decoded.
//! - kernel `MSNT_SystemTrace` group `0` events are decoded by hard-coded
//!   layouts.
//! - everything else (manifest-based, WPP) is emitted as raw payload bytes.
//!
//! Format references: [Geoff Chappell], [`dissect.etl`], [`etl-parser`],
//! [`TraceLoggingProvider.h`].
//!
//! [Event Trace Log]: https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing
//! [`Read`]: std::io::Read
//! [Geoff Chappell]: https://www.geoffchappell.com/studies/windows/km/ntoskrnl/api/etw/tracelog/wmi_buffer_header.htm
//! [`dissect.etl`]: https://github.com/fox-it/dissect.etl
//! [`etl-parser`]: https://github.com/airbus-cert/etl-parser
//! [`TraceLoggingProvider.h`]: https://learn.microsoft.com/en-us/windows/win32/api/traceloggingprovider/

use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind, Read, Result};
use std::sync::Arc;

#[allow(unused_imports)]
use ::si_trace_print::{def1n, def1o, def1x, def1ñ, def2n, def2o, def2x, def2ñ, defn, defo, defx, defñ};

use crate::common::Count;
use crate::data::etl::{
    EtlDecoder, EtlEnvelope, EtlEvent, EtlFields, EtlHeaderKind, EtlName, EtlPayload, EtlValue, FileTime, Guid,
    SID_SUB_AUTHORITIES_MAX, SYSTEMTIME_SZ, Sid,
};
use crate::subprojects::rust_lzxpress;

// -------------------
// on-disk constants

/// `sizeof(WMI_BUFFER_HEADER)`
pub const BUFFER_HEADER_SZ: usize = 0x48;
/// `ETW_BUFFER_FLAG_COMPRESSED`
const BUFFER_FLAG_COMPRESSED: u16 = 0x0040;
/// sanity limit on a single buffer
const BUFFER_SZ_MAX: usize = 64 * 1024 * 1024;

/// high byte of a record marker for `TRACE_HEADER_FLAG` records
const MARKER_FLAG_HEADER: u8 = 0xC0;
/// high byte of a record marker for `TRACE_MESSAGE` (WPP) records
const MARKER_FLAG_MESSAGE: u8 = 0x90;
/// `TRACE_HEADER_FLAG`; set in the marker of every non-padding record
const MARKER_FLAG_TRACE_HEADER: u8 = 0x80;

const TRACE_HEADER_TYPE_SYSTEM32: u8 = 0x01;
const TRACE_HEADER_TYPE_SYSTEM64: u8 = 0x02;
const TRACE_HEADER_TYPE_COMPACT32: u8 = 0x03;
const TRACE_HEADER_TYPE_COMPACT64: u8 = 0x04;
const TRACE_HEADER_TYPE_FULL_HEADER32: u8 = 0x0A;
const TRACE_HEADER_TYPE_INSTANCE32: u8 = 0x0B;
const TRACE_HEADER_TYPE_ERROR: u8 = 0x0D;
const TRACE_HEADER_TYPE_PERFINFO32: u8 = 0x10;
const TRACE_HEADER_TYPE_PERFINFO64: u8 = 0x11;
const TRACE_HEADER_TYPE_EVENT_HEADER32: u8 = 0x12;
const TRACE_HEADER_TYPE_EVENT_HEADER64: u8 = 0x13;
const TRACE_HEADER_TYPE_FULL_HEADER64: u8 = 0x14;
const TRACE_HEADER_TYPE_INSTANCE64: u8 = 0x15;

const SYSTEM_HEADER_SZ: usize = 0x20;
const COMPACT_HEADER_SZ: usize = 0x18;
const PERFINFO_HEADER_SZ: usize = 0x10;
const EVENT_HEADER_SZ: usize = 0x50;
const FULL_HEADER_SZ: usize = 0x30;
const INSTANCE_HEADER_SZ: usize = 0x48;
const MESSAGE_HEADER_SZ: usize = 0x08;

/// `EVENT_HEADER_FLAG_EXTENDED_INFO`
const EVENT_HEADER_FLAG_EXTENDED_INFO: u16 = 0x0001;
/// `EVENT_HEADER_FLAG_32_BIT_HEADER`; the provider process is 32-bit
const EVENT_HEADER_FLAG_32_BIT_HEADER: u16 = 0x0020;
/// `EVENT_HEADER_FLAG_64_BIT_HEADER`; the provider process is 64-bit
const EVENT_HEADER_FLAG_64_BIT_HEADER: u16 = 0x0040;
/// `EVENT_HEADER_EXT_TYPE_RELATED_ACTIVITYID`
const EXT_TYPE_RELATED_ACTIVITYID: u16 = 0x0001;
/// `EVENT_HEADER_EXT_TYPE_EVENT_SCHEMA_TL`
const EXT_TYPE_EVENT_SCHEMA_TL: u16 = 0x000B;
/// `EVENT_HEADER_EXT_TYPE_PROV_TRAITS`
const EXT_TYPE_PROV_TRAITS: u16 = 0x000C;
/// sanity limit on extended data items per record
const EXT_ITEMS_MAX: usize = 16;

// `TRACE_MESSAGE_*` flags of a WPP message header
const TRACE_MESSAGE_SEQUENCE: u16 = 0x0001;
const TRACE_MESSAGE_GUID: u16 = 0x0002;
const TRACE_MESSAGE_COMPONENTID: u16 = 0x0004;
const TRACE_MESSAGE_TIMESTAMP: u16 = 0x0008;
const TRACE_MESSAGE_SYSTEMINFO: u16 = 0x0020;

/// `EventTraceGuid`; class GUID of kernel `EventTrace` events
pub const GUID_EVENT_TRACE: Guid = Guid::from_le_bytes([
    0x00, 0xd9, 0xfd, 0x68, 0x3e, 0x4a, 0xd1, 0x11, 0x84, 0xf4, 0x00, 0x00, 0xf8, 0x04, 0x64, 0xe3,
]);
/// provider name `tracerpt` shows for kernel events
const PROVIDER_NAME_KERNEL: &str = "MSNT_SystemTrace";

// ----------
// LogfileHeader

/// Clock used for record timestamps; `TRACE_LOGFILE_HEADER.ReservedFlags`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockType {
    /// `QueryPerformanceCounter` ticks scaled by `PerfFreq`
    Qpc,
    /// `FILETIME`
    SystemTime,
    /// CPU cycle counter scaled by `CpuSpeedInMHz`
    CpuCycle,
}

impl ClockType {
    fn from_reserved_flags(
        flags: u32,
        perf_freq: u64,
    ) -> ClockType {
        match flags {
            2 => ClockType::SystemTime,
            3 => ClockType::CpuCycle,
            1 => ClockType::Qpc,
            _ if perf_freq != 0 => ClockType::Qpc,
            _ => ClockType::SystemTime,
        }
    }
}

/// Fields of interest from the `TRACE_LOGFILE_HEADER`.
#[derive(Clone, Debug)]
pub struct LogfileHeader {
    pub buffer_size: u32,
    pub version: u32,
    pub provider_version: u32,
    pub number_of_processors: u32,
    pub end_time: FileTime,
    pub timer_resolution: u32,
    pub maximum_file_size: u32,
    pub log_file_mode: u32,
    pub buffers_written: u32,
    pub start_buffers: u32,
    pub pointer_size: u32,
    pub events_lost: u32,
    pub cpu_speed_mhz: u32,
    pub logger_name_ptr: u64,
    pub log_file_name_ptr: u64,
    pub boot_time: FileTime,
    pub perf_freq: u64,
    pub start_time: FileTime,
    pub reserved_flags: u32,
    pub buffers_lost: u32,
    pub session_name: String,
    pub log_file_name: String,
    pub clock_type: ClockType,
    /// `TimeDelta` of the record carrying this header; the origin of relative timestamps
    pub header_time_delta: u64,
    pub is_64bit: bool,
}

impl LogfileHeader {
    /// Convert a record `TimeDelta` to a `FILETIME`.
    pub fn time_delta_to_filetime(
        &self,
        time_delta: u64,
    ) -> FileTime {
        let delta: i128 = time_delta as i128 - self.header_time_delta as i128;
        let (num, den): (i128, i128) = match self.clock_type {
            ClockType::Qpc => (10_000_000, self.perf_freq.max(1) as i128),
            ClockType::SystemTime => (1, 1),
            ClockType::CpuCycle => (10, self.cpu_speed_mhz.max(1) as i128),
        };
        let ft: i128 = self.start_time as i128 + (delta * num) / den;

        ft.clamp(0, u64::MAX as i128) as FileTime
    }

    /// Render the header as payload fields, mirroring `tracerpt` naming.
    fn to_fields(&self) -> EtlFields {
        vec![
            ("BufferSize".into(), EtlValue::U64(self.buffer_size as u64)),
            ("Version".into(), EtlValue::U64(self.version as u64)),
            ("ProviderVersion".into(), EtlValue::U64(self.provider_version as u64)),
            ("NumberOfProcessors".into(), EtlValue::U64(self.number_of_processors as u64)),
            ("EndTime".into(), EtlValue::U64(self.end_time)),
            ("TimerResolution".into(), EtlValue::U64(self.timer_resolution as u64)),
            ("MaxFileSize".into(), EtlValue::U64(self.maximum_file_size as u64)),
            ("LogFileMode".into(), EtlValue::Hex32(self.log_file_mode)),
            ("BuffersWritten".into(), EtlValue::U64(self.buffers_written as u64)),
            ("StartBuffers".into(), EtlValue::U64(self.start_buffers as u64)),
            ("PointerSize".into(), EtlValue::U64(self.pointer_size as u64)),
            ("EventsLost".into(), EtlValue::U64(self.events_lost as u64)),
            ("CPUSpeed".into(), EtlValue::U64(self.cpu_speed_mhz as u64)),
            ("LoggerName".into(), EtlValue::Hex64(self.logger_name_ptr)),
            ("LogFileName".into(), EtlValue::Hex64(self.log_file_name_ptr)),
            ("BootTime".into(), EtlValue::U64(self.boot_time)),
            ("PerfFreq".into(), EtlValue::U64(self.perf_freq)),
            ("StartTime".into(), EtlValue::U64(self.start_time)),
            ("ReservedFlags".into(), EtlValue::Hex32(self.reserved_flags)),
            ("BuffersLost".into(), EtlValue::U64(self.buffers_lost as u64)),
            ("SessionNameString".into(), EtlValue::Str(self.session_name.clone())),
            ("LogFileNameString".into(), EtlValue::Str(self.log_file_name.clone())),
        ]
    }
}

// -------
// errors

/// A non-fatal or fatal problem found while iterating records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtlRecordError {
    /// A record could not be parsed; the remainder of its buffer is skipped.
    Skipped {
        buffer: usize,
        offset: usize,
        reason: String,
    },
    /// The file cannot be read further.
    Fatal(String),
}

impl fmt::Display for EtlRecordError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            EtlRecordError::Skipped { buffer, offset, reason } => {
                write!(f, "skipped remainder of buffer {} at offset 0x{:X}: {}", buffer, offset, reason)
            }
            EtlRecordError::Fatal(reason) => write!(f, "{}", reason),
        }
    }
}

// -------
// cursor

/// Little-endian bounds-checked byte reader.
struct Cur<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cur<'a> {
    fn new(data: &'a [u8]) -> Cur<'a> {
        Cur { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data
            .len()
            .saturating_sub(self.pos)
    }

    fn rest(&self) -> &'a [u8] {
        &self.data[self.pos.min(self.data.len())..]
    }

    fn take(
        &mut self,
        n: usize,
    ) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let s = self.data.get(self.pos..end)?;
        self.pos = end;

        Some(s)
    }

    fn skip(
        &mut self,
        n: usize,
    ) -> Option<()> {
        self.take(n).map(|_| ())
    }

    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|s| s[0])
    }

    fn u16(&mut self) -> Option<u16> {
        self.take(2)
            .map(|s| u16::from_le_bytes([s[0], s[1]]))
    }

    fn u32(&mut self) -> Option<u32> {
        self.take(4).map(|s| {
            u32::from_le_bytes([
                s[0], s[1], s[2], s[3],
            ])
        })
    }

    fn u64(&mut self) -> Option<u64> {
        self.take(8)
            .map(|s| u64::from_le_bytes(s.try_into().unwrap()))
    }

    fn guid(&mut self) -> Option<Guid> {
        self.take(16)
            .and_then(Guid::from_le_slice)
    }

    /// NUL-terminated 8-bit string; takes the remainder if unterminated.
    fn cstr(&mut self) -> Option<String> {
        let rest = self.rest();
        let end = rest
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(rest.len());
        let s = String::from_utf8_lossy(&rest[..end]).into_owned();
        self.pos += (end + 1).min(rest.len());

        Some(s)
    }

    /// NUL-terminated UTF-16LE string; takes the remainder if unterminated.
    fn wstr(&mut self) -> Option<String> {
        let rest = self.rest();
        let mut units: Vec<u16> = Vec::new();
        let mut i: usize = 0;
        while i + 1 < rest.len() {
            let u = u16::from_le_bytes([rest[i], rest[i + 1]]);
            i += 2;
            if u == 0 {
                break;
            }
            units.push(u);
        }
        self.pos += i;

        Some(String::from_utf16_lossy(&units))
    }
}

fn utf16le_to_string(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|u| *u != 0)
        .collect();

    String::from_utf16_lossy(&units)
}

fn read_u16_at(
    data: &[u8],
    at: usize,
) -> Option<u16> {
    data.get(at..at + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn read_u32_at(
    data: &[u8],
    at: usize,
) -> Option<u32> {
    data.get(at..at + 4).map(|s| {
        u32::from_le_bytes([
            s[0], s[1], s[2], s[3],
        ])
    })
}

fn read_u64_at(
    data: &[u8],
    at: usize,
) -> Option<u64> {
    data.get(at..at + 8)
        .map(|s| u64::from_le_bytes(s.try_into().unwrap()))
}

/// Read until `buf` is full or EOF; returns bytes read.
fn read_full<R: Read>(
    reader: &mut R,
    buf: &mut [u8],
) -> Result<usize> {
    let mut total: usize = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }

    Ok(total)
}

// ---------------
// kernel groups

/// Kernel `EVENT_TRACE_GROUP_*` to class GUID; index is `group`.
/// From `sechost.dll` via Geoff Chappell and `dissect.etl`.
const KERNEL_GROUP_GUIDS: [(u8, &str, [u8; 16]); 31] = [
    (
        0x00,
        "EventTrace",
        [
            0x00, 0xd9, 0xfd, 0x68, 0x3e, 0x4a, 0xd1, 0x11, 0x84, 0xf4, 0x00, 0x00, 0xf8, 0x04, 0x64, 0xe3,
        ],
    ),
    (
        0x01,
        "DiskIo",
        [
            0xd4, 0xa8, 0x6f, 0x3d, 0x05, 0xfe, 0xd0, 0x11, 0x9d, 0xda, 0x00, 0xc0, 0x4f, 0xd7, 0xba, 0x7c,
        ],
    ),
    (
        0x02,
        "PageFault",
        [
            0xd3, 0xa8, 0x6f, 0x3d, 0x05, 0xfe, 0xd0, 0x11, 0x9d, 0xda, 0x00, 0xc0, 0x4f, 0xd7, 0xba, 0x7c,
        ],
    ),
    (
        0x03,
        "Process",
        [
            0xd0, 0xa8, 0x6f, 0x3d, 0x05, 0xfe, 0xd0, 0x11, 0x9d, 0xda, 0x00, 0xc0, 0x4f, 0xd7, 0xba, 0x7c,
        ],
    ),
    (
        0x04,
        "FileIo",
        [
            0x39, 0xdc, 0xcb, 0x90, 0x3e, 0x4a, 0xd1, 0x11, 0x84, 0xf4, 0x00, 0x00, 0xf8, 0x04, 0x64, 0xe3,
        ],
    ),
    (
        0x05,
        "Thread",
        [
            0xd1, 0xa8, 0x6f, 0x3d, 0x05, 0xfe, 0xd0, 0x11, 0x9d, 0xda, 0x00, 0xc0, 0x4f, 0xd7, 0xba, 0x7c,
        ],
    ),
    (
        0x06,
        "TcpIp",
        [
            0xc0, 0x0a, 0x28, 0x9a, 0xe0, 0xc8, 0xd1, 0x11, 0x84, 0xe2, 0x00, 0xc0, 0x4f, 0xb9, 0x98, 0xa2,
        ],
    ),
    (
        0x07,
        "Job",
        [
            0x76, 0xfc, 0x82, 0x32, 0xed, 0xfe, 0x8e, 0x49, 0x8a, 0xa7, 0xe7, 0x0f, 0x45, 0x9d, 0x43, 0x0e,
        ],
    ),
    (
        0x08,
        "UdpIp",
        [
            0xc5, 0x50, 0x3a, 0xbf, 0xc9, 0xa9, 0x88, 0x49, 0xa0, 0x05, 0x2d, 0xf0, 0xb7, 0xc8, 0x0f, 0x80,
        ],
    ),
    (
        0x09,
        "Registry",
        [
            0x2e, 0x72, 0x53, 0xae, 0x63, 0xc8, 0xd2, 0x11, 0x86, 0x59, 0x00, 0xc0, 0x4f, 0xa3, 0x21, 0xa1,
        ],
    ),
    (
        0x0A,
        "DbgPrint",
        [
            0x09, 0x6d, 0x97, 0x13, 0x27, 0xa3, 0x8c, 0x43, 0x95, 0x0b, 0x7f, 0x03, 0x19, 0x28, 0x15, 0xc7,
        ],
    ),
    (
        0x0B,
        "EventTraceConfig",
        [
            0x65, 0x3a, 0x85, 0x01, 0x8f, 0x41, 0x36, 0x4f, 0xae, 0xfc, 0xdc, 0x0f, 0x1d, 0x2f, 0xd2, 0x35,
        ],
    ),
    (
        0x0C,
        "Spare1",
        [
            0x83, 0x43, 0x13, 0x99, 0x48, 0x52, 0xfc, 0x43, 0x83, 0x4b, 0x52, 0x94, 0x54, 0xe7, 0x5d, 0xf3,
        ],
    ),
    (
        0x0D,
        "Wnf",
        [
            0x62, 0x57, 0x69, 0x42, 0x50, 0xea, 0x7a, 0x49, 0x90, 0x68, 0x5c, 0xbb, 0xb3, 0x5e, 0x0b, 0x95,
        ],
    ),
    (
        0x0E,
        "Pool",
        [
            0xb6, 0xa8, 0x68, 0x02, 0xfd, 0x74, 0x02, 0x43, 0x9d, 0xd0, 0x6e, 0x8f, 0x17, 0x95, 0xc0, 0xcf,
        ],
    ),
    (
        0x0F,
        "PerfInfo",
        [
            0xb4, 0xbf, 0x1d, 0xce, 0x7e, 0x13, 0xa6, 0x4d, 0x87, 0xb0, 0x3f, 0x59, 0xaa, 0x10, 0x2c, 0xbc,
        ],
    ),
    (
        0x10,
        "Heap",
        [
            0xab, 0x62, 0x29, 0x22, 0x80, 0x61, 0x88, 0x4b, 0xa8, 0x25, 0x34, 0x6b, 0x75, 0xf2, 0xa2, 0x4a,
        ],
    ),
    (
        0x11,
        "Object",
        [
            0x50, 0x7f, 0x49, 0x89, 0xfe, 0xef, 0x40, 0x44, 0x8c, 0xf2, 0xce, 0x6b, 0x1c, 0xdc, 0xac, 0xa7,
        ],
    ),
    (
        0x12,
        "Power",
        [
            0xe0, 0x45, 0x34, 0xe4, 0x03, 0x09, 0xc3, 0x48, 0xb8, 0x78, 0xff, 0x0f, 0xcc, 0xeb, 0xdd, 0x04,
        ],
    ),
    (
        0x13,
        "ModBound",
        [
            0x00, 0x2f, 0x15, 0xa9, 0x58, 0x3f, 0xee, 0x4b, 0x92, 0xa1, 0x70, 0xc7, 0xd0, 0x79, 0xd5, 0xdd,
        ],
    ),
    (
        0x14,
        "Image",
        [
            0x1d, 0x5d, 0xb1, 0x2c, 0xc1, 0x5f, 0xd2, 0x11, 0xab, 0xe1, 0x00, 0xa0, 0xc9, 0x11, 0xf5, 0x18,
        ],
    ),
    (
        0x15,
        "Dpc",
        [
            0x72, 0x48, 0xd1, 0xb2, 0x5b, 0x7c, 0x3d, 0x46, 0x84, 0x19, 0xee, 0x9b, 0xf7, 0xd2, 0x3e, 0x04,
        ],
    ),
    (
        0x16,
        "Cc",
        [
            0x39, 0xa4, 0x87, 0x76, 0x52, 0xf7, 0xb8, 0x45, 0xb7, 0x41, 0x32, 0x1a, 0xec, 0x0f, 0x8d, 0xf9,
        ],
    ),
    (
        0x17,
        "CritSec",
        [
            0x36, 0x67, 0xc6, 0x3a, 0x59, 0xcc, 0xff, 0x4c, 0x81, 0x15, 0x8d, 0xf5, 0x0e, 0x39, 0x81, 0x6b,
        ],
    ),
    (
        0x18,
        "StackWalk",
        [
            0x46, 0xfe, 0xf2, 0xde, 0xd6, 0x7b, 0x80, 0x4b, 0xbd, 0x94, 0xf5, 0x7f, 0xe2, 0x0d, 0x0c, 0xe3,
        ],
    ),
    (
        0x19,
        "Ums",
        [
            0x4b, 0x97, 0xec, 0x9a, 0x8e, 0x5b, 0x18, 0x41, 0x9b, 0x92, 0x31, 0x86, 0xd8, 0x00, 0x2c, 0xe5,
        ],
    ),
    (
        0x1A,
        "Alpc",
        [
            0xcd, 0xcc, 0xd8, 0x45, 0x9f, 0x53, 0x72, 0x4b, 0xa8, 0xb7, 0x5c, 0x68, 0x31, 0x42, 0x60, 0x9a,
        ],
    ),
    (
        0x1B,
        "SplitIo",
        [
            0x92, 0xca, 0x37, 0xd8, 0xb9, 0x12, 0xa5, 0x44, 0xad, 0x6a, 0x3a, 0x65, 0xb3, 0x57, 0x8a, 0xd8,
        ],
    ),
    (
        0x1C,
        "ThreadPool",
        [
            0xe2, 0xd0, 0x61, 0xc8, 0xc1, 0xa2, 0x36, 0x4d, 0x9f, 0x9c, 0x97, 0x0b, 0xab, 0x94, 0x3a, 0x12,
        ],
    ),
    (
        0x1D,
        "Hypervisor",
        [
            0x5c, 0x40, 0x2a, 0x7f, 0xb5, 0x69, 0xf9, 0x4b, 0xa1, 0xf5, 0x30, 0xe8, 0xf1, 0xaf, 0xab, 0x5e,
        ],
    ),
    (
        0x1E,
        "HypervisorX",
        [
            0x49, 0xa1, 0xe9, 0x2c, 0xfe, 0xef, 0xf0, 0x42, 0xa6, 0x35, 0xa1, 0xd3, 0x9e, 0x26, 0xc8, 0xf2,
        ],
    ),
];

/// Map a kernel `HookId` group/opcode to `(class GUID, task name)`.
fn kernel_group_lookup(
    group: u8,
    opcode: u8,
) -> (Guid, Option<&'static str>) {
    // `Process` opcode 10 is really an `Image` event
    let group_: u8 = if group == 0x03 && opcode == 10 { 0x14 } else { group };
    match KERNEL_GROUP_GUIDS.get(group_ as usize) {
        Some((_, name, bytes)) => (Guid::from_le_bytes(*bytes), Some(name)),
        None => (Guid::NIL, None),
    }
}

/// Opcode names of kernel `EventTrace` (group 0) events.
fn kernel_group0_opcode_name(opcode: u8) -> Option<&'static str> {
    match opcode {
        0 => Some("Header"),
        5 => Some("Extension"),
        8 => Some("RDComplete"),
        32 => Some("EndExtension"),
        64 => Some("DbgIdRSDS"),
        66 => Some("BuildInfo"),
        67 => Some("ProviderBinaryPath"),
        80 => Some("PartitionInfoExtension"),
        82 => Some("LastDroppedTimes"),
        _ => None,
    }
}

// -------------
// TraceLogging

const TLG_IN_NULL: u8 = 0;
const TLG_IN_UNICODESTRING: u8 = 1;
const TLG_IN_ANSISTRING: u8 = 2;
const TLG_IN_INT8: u8 = 3;
const TLG_IN_UINT8: u8 = 4;
const TLG_IN_INT16: u8 = 5;
const TLG_IN_UINT16: u8 = 6;
const TLG_IN_INT32: u8 = 7;
const TLG_IN_UINT32: u8 = 8;
const TLG_IN_INT64: u8 = 9;
const TLG_IN_UINT64: u8 = 10;
const TLG_IN_FLOAT: u8 = 11;
const TLG_IN_DOUBLE: u8 = 12;
const TLG_IN_BOOL32: u8 = 13;
const TLG_IN_BINARY: u8 = 14;
const TLG_IN_GUID: u8 = 15;
const TLG_IN_POINTER: u8 = 16;
const TLG_IN_FILETIME: u8 = 17;
const TLG_IN_SYSTEMTIME: u8 = 18;
const TLG_IN_SID: u8 = 19;
const TLG_IN_HEXINT32: u8 = 20;
const TLG_IN_HEXINT64: u8 = 21;
const TLG_IN_COUNTEDSTRING: u8 = 22;
const TLG_IN_COUNTEDANSISTRING: u8 = 23;
const TLG_IN_STRUCT: u8 = 24;
const TLG_IN_COUNTEDBINARY: u8 = 25;

const TLG_IN_TYPE_MASK: u8 = 0x1F;
const TLG_IN_FLAG_CCOUNT: u8 = 0x20;
const TLG_IN_FLAG_VCOUNT: u8 = 0x40;
const TLG_IN_FLAG_CHAIN: u8 = 0x80;
const TLG_OUT_FLAG_CHAIN: u8 = 0x80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TlCount {
    Scalar,
    Fixed(u16),
    Var,
    /// custom serializer; not decodable
    Custom,
}

#[derive(Clone, Debug)]
struct TlField {
    name: EtlName,
    in_type: u8,
    out_type: u8,
    count: TlCount,
}

impl TlField {
    /// number of following metadata fields that belong to this struct
    fn struct_len(&self) -> usize {
        if self.in_type == TLG_IN_STRUCT {
            (self.out_type & 0x7F) as usize
        } else {
            0
        }
    }
}

/// A parsed TraceLogging event schema; shared across all events carrying the
/// same metadata blob.
#[derive(Debug)]
struct TlSchema {
    event_name: EtlName,
    fields: Vec<TlField>,
}

/// Parsed schemas keyed by the raw metadata blob; `None` marks an unparsable blob.
type TlSchemaCache = HashMap<Box<[u8]>, Option<Arc<TlSchema>>>;

/// Parse the TraceLogging event metadata blob into an event name and fields.
fn tl_parse_schema(schema: &[u8]) -> Option<TlSchema> {
    let mut cur = Cur::new(schema);
    let size: usize = cur.u16()? as usize;
    let schema: &[u8] = &schema[..size.min(schema.len())];
    let mut cur = Cur::new(schema);
    cur.skip(2)?;
    // extension bytes chain while high bit set
    loop {
        let b = cur.u8()?;
        if b & 0x80 == 0 {
            break;
        }
    }
    let event_name: EtlName = cur.cstr()?.into();
    let mut fields: Vec<TlField> = Vec::new();
    while cur.remaining() > 0 {
        let name: EtlName = cur.cstr()?.into();
        let in_type: u8 = cur.u8()?;
        let mut out_type: u8 = 0;
        if in_type & TLG_IN_FLAG_CHAIN != 0 {
            out_type = cur.u8()?;
            if out_type & TLG_OUT_FLAG_CHAIN != 0 {
                // field tags: chain of bytes while high bit set
                loop {
                    let b = cur.u8()?;
                    if b & 0x80 == 0 {
                        break;
                    }
                }
            }
        }
        let count: TlCount = match in_type & (TLG_IN_FLAG_CCOUNT | TLG_IN_FLAG_VCOUNT) {
            0 => TlCount::Scalar,
            TLG_IN_FLAG_CCOUNT => TlCount::Fixed(cur.u16()?),
            TLG_IN_FLAG_VCOUNT => TlCount::Var,
            _ => {
                // custom serializer: u16 size + type info blob
                let n = cur.u16()? as usize;
                cur.skip(n)?;
                TlCount::Custom
            }
        };
        fields.push(TlField {
            name,
            in_type: in_type & TLG_IN_TYPE_MASK,
            out_type: out_type & 0x7F,
            count,
        });
    }

    Some(TlSchema { event_name, fields })
}

/// Look up (or parse and insert) the schema for a metadata blob.
fn tl_schema_cached<'c>(
    cache: &'c mut TlSchemaCache,
    schema: &[u8],
) -> Option<&'c Arc<TlSchema>> {
    if !cache.contains_key(schema) {
        cache.insert(schema.into(), tl_parse_schema(schema).map(Arc::new));
    }
    cache
        .get(schema)
        .and_then(Option::as_ref)
}

/// number of metadata entries occupied by the field at `idx` (1 + nested struct members)
fn tl_field_extent(
    fields: &[TlField],
    idx: usize,
) -> usize {
    let mut n: usize = 1;
    let mut remaining: usize = fields
        .get(idx)
        .map_or(0, |f| f.struct_len());
    let mut i: usize = idx + 1;
    while remaining > 0 && i < fields.len() {
        let ext = tl_field_extent(fields, i);
        n += ext;
        i += ext;
        remaining -= 1;
    }

    n
}

fn tl_read_sid(cur: &mut Cur) -> Option<Sid> {
    let start: usize = cur.pos;
    let revision: u8 = cur.u8()?;
    let count: u8 = cur.u8()?;
    if count as usize > SID_SUB_AUTHORITIES_MAX {
        return None;
    }
    let identifier_authority: [u8; 6] = cur
        .take(6)?
        .try_into()
        .ok()?;
    let mut sub_authorities: [u32; SID_SUB_AUTHORITIES_MAX] = [0; SID_SUB_AUTHORITIES_MAX];
    for sub in sub_authorities
        .iter_mut()
        .take(count as usize)
    {
        *sub = cur.u32()?;
    }
    debug_assert_eq!(cur.pos - start, Sid::size_for(count));

    Some(Sid {
        revision,
        sub_authority_count: count,
        identifier_authority,
        sub_authorities,
    })
}

/// Decode one scalar value of `in_type` from the payload.
fn tl_read_scalar(
    cur: &mut Cur,
    in_type: u8,
    out_type: u8,
    pointer_size: usize,
) -> Option<EtlValue> {
    let v = match in_type {
        TLG_IN_NULL => EtlValue::Null,
        TLG_IN_UNICODESTRING => EtlValue::Str(cur.wstr()?),
        TLG_IN_ANSISTRING => EtlValue::Str(cur.cstr()?),
        TLG_IN_INT8 => EtlValue::I64(cur.u8()? as i8 as i64),
        TLG_IN_UINT8 => {
            let b = cur.u8()?;
            // TlgOutBOOLEAN
            if out_type == 3 {
                EtlValue::Bool(b != 0)
            } else {
                EtlValue::U64(b as u64)
            }
        }
        TLG_IN_INT16 => EtlValue::I64(cur.u16()? as i16 as i64),
        TLG_IN_UINT16 => EtlValue::U64(cur.u16()? as u64),
        TLG_IN_INT32 => EtlValue::I64(cur.u32()? as i32 as i64),
        TLG_IN_UINT32 => EtlValue::U64(cur.u32()? as u64),
        TLG_IN_INT64 => EtlValue::I64(cur.u64()? as i64),
        TLG_IN_UINT64 => EtlValue::U64(cur.u64()?),
        TLG_IN_FLOAT => EtlValue::F64(f32::from_le_bytes(
            cur.take(4)?
                .try_into()
                .unwrap(),
        ) as f64),
        TLG_IN_DOUBLE => EtlValue::F64(f64::from_le_bytes(
            cur.take(8)?
                .try_into()
                .unwrap(),
        )),
        TLG_IN_BOOL32 => EtlValue::Bool(cur.u32()? != 0),
        TLG_IN_BINARY | TLG_IN_COUNTEDBINARY => {
            let n = cur.u16()? as usize;
            EtlValue::Bytes(cur.take(n)?.to_vec())
        }
        TLG_IN_GUID => EtlValue::Guid(cur.guid()?),
        TLG_IN_POINTER => match pointer_size {
            4 => EtlValue::Hex64(cur.u32()? as u64),
            _ => EtlValue::Hex64(cur.u64()?),
        },
        TLG_IN_FILETIME => EtlValue::FileTime(cur.u64()?),
        TLG_IN_SYSTEMTIME => EtlValue::SystemTime(cur.take(SYSTEMTIME_SZ)?.try_into().ok()?),
        TLG_IN_SID => EtlValue::Sid(Box::new(tl_read_sid(cur)?)),
        TLG_IN_HEXINT32 => EtlValue::Hex32(cur.u32()?),
        TLG_IN_HEXINT64 => EtlValue::Hex64(cur.u64()?),
        TLG_IN_COUNTEDSTRING => {
            let n = cur.u16()? as usize;
            EtlValue::Str(utf16le_to_string(cur.take(n)?))
        }
        TLG_IN_COUNTEDANSISTRING => {
            let n = cur.u16()? as usize;
            EtlValue::Str(String::from_utf8_lossy(cur.take(n)?).into_owned())
        }
        _ => return None,
    };

    Some(v)
}

/// Decode `n` metadata fields starting at `idx` from the payload.
/// Returns `None` on the first field that cannot be decoded; `out` holds
/// what was decoded before the failure.
fn tl_decode_fields(
    fields: &[TlField],
    idx: &mut usize,
    n: usize,
    cur: &mut Cur,
    pointer_size: usize,
    out: &mut EtlFields,
) -> Option<()> {
    for _ in 0..n {
        let field: &TlField = fields.get(*idx)?;
        let extent: usize = tl_field_extent(fields, *idx);
        let sub_start: usize = *idx + 1;
        let sub_count: usize = field.struct_len();
        *idx += extent;

        let count: usize = match field.count {
            TlCount::Scalar => 1,
            TlCount::Fixed(c) => c as usize,
            TlCount::Var => cur.u16()? as usize,
            TlCount::Custom => return None,
        };
        let is_array: bool = !matches!(field.count, TlCount::Scalar);

        let read_one = |cur: &mut Cur| -> Option<EtlValue> {
            if field.in_type == TLG_IN_STRUCT {
                let mut sub: Vec<(EtlName, EtlValue)> = Vec::with_capacity(sub_count);
                let mut sub_idx: usize = sub_start;
                tl_decode_fields(fields, &mut sub_idx, sub_count, cur, pointer_size, &mut sub)?;
                Some(EtlValue::Struct(sub))
            } else {
                tl_read_scalar(cur, field.in_type, field.out_type, pointer_size)
            }
        };

        let value: EtlValue = if is_array {
            let mut items: Vec<EtlValue> = Vec::with_capacity(count.min(64));
            for _ in 0..count {
                items.push(read_one(cur)?);
            }
            EtlValue::Array(items)
        } else {
            read_one(cur)?
        };
        out.push((field.name.clone(), value));
    }

    Some(())
}

/// Decode a TraceLogging event payload against its parsed schema.
fn decode_tracelogging(
    schema: &TlSchema,
    user_data: &[u8],
    pointer_size: usize,
) -> EtlPayload {
    let fields: &[TlField] = &schema.fields;
    let mut cur = Cur::new(user_data);
    let mut out: EtlFields = Vec::with_capacity(fields.len());
    let mut idx: usize = 0;

    let mut top_level_fields: usize = 0;
    let mut top_level_idx: usize = 0;
    while top_level_idx < fields.len() {
        top_level_fields += 1;
        top_level_idx += tl_field_extent(fields, top_level_idx);
    }
    let complete: bool =
        tl_decode_fields(fields, &mut idx, top_level_fields, &mut cur, pointer_size, &mut out).is_some();

    match (complete, cur.remaining()) {
        (true, _) if out.is_empty() && user_data.is_empty() => EtlPayload::Empty,
        (true, 0) => EtlPayload::Fields(out),
        (_, _) => {
            let rest: Vec<u8> = cur.rest().to_vec();
            if out.is_empty() {
                EtlPayload::Raw(rest)
            } else {
                EtlPayload::Partial(out, rest)
            }
        }
    }
}

// -------------------
// kernel payloads

/// Decode kernel `EventTrace` group `0` payloads for well-known opcodes.
fn decode_kernel_group0(
    opcode: u8,
    user_data: &[u8],
    header: &LogfileHeader,
) -> Option<EtlPayload> {
    let mut cur = Cur::new(user_data);
    let fields: EtlFields = match opcode {
        0 => header.to_fields(),
        64 => vec![
            ("Guid".into(), EtlValue::Guid(cur.guid()?)),
            ("Age".into(), EtlValue::U64(cur.u32()? as u64)),
            ("PdbName".into(), EtlValue::Str(cur.cstr()?)),
        ],
        66 => vec![("BuildString".into(), EtlValue::Str(cur.cstr()?))],
        67 => {
            let count: u32 = cur.u32()?;
            let mut guids: Vec<EtlValue> = Vec::with_capacity(count.min(64) as usize);
            for _ in 0..count {
                guids.push(EtlValue::Guid(cur.guid()?));
            }
            vec![
                ("GuidCount".into(), EtlValue::U64(count as u64)),
                ("Guid".into(), EtlValue::Array(guids)),
                ("BinaryPath".into(), EtlValue::Str(cur.wstr()?)),
            ]
        }
        _ => return None,
    };

    Some(EtlPayload::Fields(fields))
}

// ----------
// EtlParser

/// One buffer read from the file: `(records, records_end, buffer_timestamp, was_compressed)`
type BufferData = (Vec<u8>, usize, u64, bool);

/// One record split from its buffer; header fields plus payload slices.
struct RawRecord<'a> {
    envelope: EtlEnvelope,
    time_delta: Option<u64>,
    tl_schema: Option<&'a [u8]>,
    /// provider name bytes from `PROV_TRAITS`, resolved against the cache later
    provider_name: Option<&'a [u8]>,
    /// pointer size of the logging process when the header says so
    pointer_size: Option<usize>,
    user_data: &'a [u8],
}

/// Parse an ETL file from a [`Read`] source.
///
/// Call [`EtlParser::new`] then repeatedly [`EtlParser::next_event`].
///
/// [`Read`]: std::io::Read
pub struct EtlParser<R: Read> {
    reader: R,
    header: LogfileHeader,
    /// records of the current buffer (decompressed if necessary)
    buf: Vec<u8>,
    /// offset within `buf` of the next record
    buf_pos: usize,
    /// end of records within `buf`
    buf_end: usize,
    /// `WMI_BUFFER_HEADER.TimeStamp` of the current buffer
    buf_timestamp: u64,
    /// index of the current buffer
    buf_index: usize,
    done: bool,
    /// provider names learned from `PROV_TRAITS` extended data
    provider_names: HashMap<Guid, EtlName>,
    /// parsed TraceLogging schemas
    schemas: TlSchemaCache,
    pub buffers_read: Count,
    pub buffers_compressed: Count,
    pub records_skipped: Count,
}

impl<R: Read> fmt::Debug for EtlParser<R> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("EtlParser")
            .field("session", &self.header.session_name)
            .field("clock", &self.header.clock_type)
            .field("buffers_read", &self.buffers_read)
            .finish()
    }
}

impl<R: Read> EtlParser<R> {
    /// Read the first buffer and the `TRACE_LOGFILE_HEADER` record.
    pub fn new(mut reader: R) -> Result<EtlParser<R>> {
        def1n!();
        let (buf, buf_end, buf_timestamp, compressed): (Vec<u8>, usize, u64, bool) =
            match Self::read_buffer(&mut reader, 0)? {
                Some(val) => val,
                None => {
                    def1x!("empty file");
                    return Err(Error::new(ErrorKind::UnexpectedEof, "ETL file has no buffers"));
                }
            };
        let records: &[u8] = &buf[..buf_end];
        let header: LogfileHeader = match Self::parse_logfile_header(records) {
            Some(val) => val,
            None => {
                def1x!("bad logfile header");
                return Err(Error::new(ErrorKind::InvalidData, "ETL file has no valid TRACE_LOGFILE_HEADER record"));
            }
        };
        def1o!("header {:?}", header);
        let mut provider_names: HashMap<Guid, EtlName> = HashMap::new();
        provider_names.insert(GUID_EVENT_TRACE, PROVIDER_NAME_KERNEL.into());
        def1x!();

        Ok(EtlParser {
            reader,
            header,
            buf,
            buf_pos: 0,
            buf_end,
            buf_timestamp,
            buf_index: 0,
            done: false,
            provider_names,
            schemas: TlSchemaCache::new(),
            buffers_read: 1,
            buffers_compressed: compressed as Count,
            records_skipped: 0,
        })
    }

    pub const fn header(&self) -> &LogfileHeader {
        &self.header
    }

    /// Read the next buffer; returns `(records, records_end, buffer_timestamp, was_compressed)`
    /// or `None` at end of file.
    fn read_buffer(
        reader: &mut R,
        index: usize,
    ) -> Result<Option<BufferData>> {
        def2n!("buffer {}", index);
        let mut hdr: [u8; BUFFER_HEADER_SZ] = [0; BUFFER_HEADER_SZ];
        let n: usize = read_full(reader, &mut hdr)?;
        if n == 0 {
            def2x!("EOF");
            return Ok(None);
        }
        if n < BUFFER_HEADER_SZ {
            def2x!("short buffer header {} bytes", n);
            return Ok(None);
        }
        let buffer_size: usize = read_u32_at(&hdr, 0x00).unwrap() as usize;
        let timestamp: u64 = read_u64_at(&hdr, 0x10).unwrap();
        let filled: usize = read_u32_at(&hdr, 0x30).unwrap() as usize;
        let flags: u16 = read_u16_at(&hdr, 0x34).unwrap();
        def2o!("buffer_size {} filled {} flags 0x{:04X} timestamp {}", buffer_size, filled, flags, timestamp);
        if buffer_size == 0 {
            // zero-filled tail
            def2x!("zero buffer size; treat as EOF");
            return Ok(None);
        }
        if !(BUFFER_HEADER_SZ..=BUFFER_SZ_MAX).contains(&buffer_size) {
            def2x!("bad buffer size");
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("ETL buffer {} has invalid BufferSize {}", index, buffer_size),
            ));
        }
        let mut data: Vec<u8> = vec![0; buffer_size - BUFFER_HEADER_SZ];
        let n: usize = read_full(reader, &mut data)?;
        if n < data.len() {
            def2o!("short buffer data {} of {} bytes", n, data.len());
            data.truncate(n);
        }
        let compressed: bool = flags & BUFFER_FLAG_COMPRESSED != 0;
        // for compressed buffers `BufferSize` is the on-disk compressed size while
        // `FilledBytes` is the uncompressed extent, so it bounds the decompressed data
        let records_end: usize = filled.saturating_sub(BUFFER_HEADER_SZ);
        if compressed {
            let _compressed_len: usize = data.len();
            data = match rust_lzxpress::decompress(&data) {
                Ok(d) => d,
                Err(err) => {
                    def2x!("decompress failed {:?}", err);
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("ETL buffer {} LZXPRESS decompression failed: {:?}", index, err),
                    ));
                }
            };
            def2o!("decompressed {} -> {} bytes", _compressed_len, data.len());
        }
        let records_end: usize = records_end.min(data.len());
        def2x!("records_end {}", records_end);

        Ok(Some((data, records_end, timestamp, compressed)))
    }

    /// Parse the `TRACE_LOGFILE_HEADER` from the first record of the first buffer.
    fn parse_logfile_header(records: &[u8]) -> Option<LogfileHeader> {
        let marker: u32 = read_u32_at(records, 0)?;
        let header_type: u8 = ((marker >> 16) & 0xFF) as u8;
        if (marker >> 24) as u8 != MARKER_FLAG_HEADER {
            return None;
        }
        let (fixed_sz, is_64bit): (usize, bool) = match header_type {
            TRACE_HEADER_TYPE_SYSTEM32 => (SYSTEM_HEADER_SZ, false),
            TRACE_HEADER_TYPE_SYSTEM64 => (SYSTEM_HEADER_SZ, true),
            TRACE_HEADER_TYPE_COMPACT32 => (COMPACT_HEADER_SZ, false),
            TRACE_HEADER_TYPE_COMPACT64 => (COMPACT_HEADER_SZ, true),
            _ => return None,
        };
        let version_word: u16 = read_u16_at(records, 0)?;
        let size: usize = read_u16_at(records, 4)? as usize;
        let hook_id: u16 = read_u16_at(records, 6)?;
        if hook_id != 0 {
            return None;
        }
        let time_delta: u64 = read_u64_at(records, 0x10)?;
        let header_sz: usize = fixed_sz + system_header_extra(version_word);
        let payload: &[u8] = records.get(header_sz..size.min(records.len()))?;
        let mut c = Cur::new(payload);
        let buffer_size = c.u32()?;
        let version = c.u32()?;
        let provider_version = c.u32()?;
        let number_of_processors = c.u32()?;
        let end_time = c.u64()?;
        let timer_resolution = c.u32()?;
        let maximum_file_size = c.u32()?;
        let log_file_mode = c.u32()?;
        let buffers_written = c.u32()?;
        let start_buffers = c.u32()?;
        let pointer_size = c.u32()?;
        let events_lost = c.u32()?;
        let cpu_speed_mhz = c.u32()?;
        let (logger_name_ptr, log_file_name_ptr): (u64, u64) =
            if is_64bit { (c.u64()?, c.u64()?) } else { (c.u32()? as u64, c.u32()? as u64) };
        // TIME_ZONE_INFORMATION (0xAC) + 4 bytes padding
        c.skip(0xAC + 4)?;
        let boot_time = c.u64()?;
        let perf_freq = c.u64()?;
        let start_time = c.u64()?;
        let reserved_flags = c.u32()?;
        let buffers_lost = c.u32()?;
        let session_name = c.wstr().unwrap_or_default();
        let log_file_name = c.wstr().unwrap_or_default();
        let clock_type = ClockType::from_reserved_flags(reserved_flags, perf_freq);

        Some(LogfileHeader {
            buffer_size,
            version,
            provider_version,
            number_of_processors,
            end_time,
            timer_resolution,
            maximum_file_size,
            log_file_mode,
            buffers_written,
            start_buffers,
            pointer_size,
            events_lost,
            cpu_speed_mhz,
            logger_name_ptr,
            log_file_name_ptr,
            boot_time,
            perf_freq,
            start_time,
            reserved_flags,
            buffers_lost,
            session_name,
            log_file_name,
            clock_type,
            header_time_delta: time_delta,
            is_64bit,
        })
    }

    /// Advance to the next buffer. Returns `Ok(false)` at end of file.
    fn next_buffer(&mut self) -> Result<bool> {
        match Self::read_buffer(&mut self.reader, self.buf_index + 1)? {
            Some((buf, buf_end, timestamp, compressed)) => {
                self.buf = buf;
                self.buf_pos = 0;
                self.buf_end = buf_end;
                self.buf_timestamp = timestamp;
                self.buf_index += 1;
                self.buffers_read += 1;
                self.buffers_compressed += compressed as Count;

                Ok(true)
            }
            None => {
                self.done = true;

                Ok(false)
            }
        }
    }

    /// Return the next event, a record-level error, or `None` at end of file.
    pub fn next_event(&mut self) -> Option<std::result::Result<EtlEvent, EtlRecordError>> {
        loop {
            if self.done {
                return None;
            }
            // need at least a marker
            if self.buf_pos + 4 > self.buf_end {
                match self.next_buffer() {
                    Ok(true) => continue,
                    Ok(false) => return None,
                    Err(err) => {
                        self.done = true;
                        return Some(Err(EtlRecordError::Fatal(err.to_string())));
                    }
                }
            }
            let records: &[u8] = &self.buf[..self.buf_end];
            let marker: u32 = read_u32_at(records, self.buf_pos).unwrap();
            if marker == 0xFFFF_FFFF || marker == 0 {
                // end-of-buffer padding
                self.buf_pos = self.buf_end;
                continue;
            }
            let offset: usize = self.buf_pos;
            let result: std::result::Result<(EtlEvent, usize), String> = Self::parse_record(
                &self.header,
                &mut self.provider_names,
                &mut self.schemas,
                self.buf_timestamp,
                records,
                offset,
            );
            match result {
                Ok((event, size)) => {
                    self.buf_pos = offset + ((size + 7) & !7);
                    return Some(Ok(event));
                }
                Err(reason) => {
                    self.records_skipped += 1;
                    // cannot resync within a buffer; skip its remainder
                    self.buf_pos = self.buf_end;
                    return Some(Err(EtlRecordError::Skipped {
                        buffer: self.buf_index,
                        offset,
                        reason,
                    }));
                }
            }
        }
    }

    /// Shared name for `guid`: the cached name when it matches the record's
    /// `PROV_TRAITS` bytes (or when the record carries none), otherwise a new
    /// shared name that replaces the cache entry.
    fn resolve_provider_name(
        provider_names: &mut HashMap<Guid, EtlName>,
        guid: Guid,
        raw_name: Option<&[u8]>,
    ) -> Option<EtlName> {
        let cached: Option<&EtlName> = provider_names.get(&guid);
        match (raw_name, cached) {
            (None, cached) => cached.cloned(),
            (Some(raw), Some(name)) if name.as_bytes() == raw => Some(name.clone()),
            (Some(raw), _) => {
                let name: EtlName = String::from_utf8_lossy(raw)
                    .into_owned()
                    .into();
                provider_names.insert(guid, name.clone());
                Some(name)
            }
        }
    }

    /// Parse one record at `offset`; returns the event and the record size.
    fn parse_record(
        header: &LogfileHeader,
        provider_names: &mut HashMap<Guid, EtlName>,
        schemas: &mut TlSchemaCache,
        buf_timestamp: u64,
        records: &[u8],
        offset: usize,
    ) -> std::result::Result<(EtlEvent, usize), String> {
        let data: &[u8] = &records[offset..];
        let marker: u32 = read_u32_at(data, 0).ok_or("truncated marker")?;
        let marker_flag: u8 = (marker >> 24) as u8;
        let header_type: u8 = ((marker >> 16) & 0xFF) as u8;

        let (raw, size): (RawRecord, usize) = match marker_flag {
            MARKER_FLAG_MESSAGE => Self::split_message(data)?,
            MARKER_FLAG_HEADER => match header_type {
                TRACE_HEADER_TYPE_SYSTEM32 | TRACE_HEADER_TYPE_SYSTEM64 => {
                    Self::split_system(data, SYSTEM_HEADER_SZ, EtlHeaderKind::System)?
                }
                TRACE_HEADER_TYPE_COMPACT32 | TRACE_HEADER_TYPE_COMPACT64 => {
                    Self::split_system(data, COMPACT_HEADER_SZ, EtlHeaderKind::Compact)?
                }
                TRACE_HEADER_TYPE_PERFINFO32 | TRACE_HEADER_TYPE_PERFINFO64 => {
                    Self::split_system(data, PERFINFO_HEADER_SZ, EtlHeaderKind::PerfInfo)?
                }
                TRACE_HEADER_TYPE_EVENT_HEADER32 | TRACE_HEADER_TYPE_EVENT_HEADER64 => Self::split_event(data)?,
                TRACE_HEADER_TYPE_FULL_HEADER32 | TRACE_HEADER_TYPE_FULL_HEADER64 => {
                    Self::split_full(data, FULL_HEADER_SZ, EtlHeaderKind::FullHeader)?
                }
                TRACE_HEADER_TYPE_INSTANCE32 | TRACE_HEADER_TYPE_INSTANCE64 => {
                    Self::split_full(data, INSTANCE_HEADER_SZ, EtlHeaderKind::Instance)?
                }
                TRACE_HEADER_TYPE_ERROR => Self::split_error(data)?,
                // TIMED (0x0C), WNODE (0x0E), MESSAGE (0x0F), and future types all
                // carry `Size` at offset 0; emit them opaque rather than lose the buffer
                _ => Self::split_error(data)?,
            },
            // any other `TRACE_HEADER_FLAG` marker also has `Size` at offset 0
            flag if flag & MARKER_FLAG_TRACE_HEADER != 0 => Self::split_error(data)?,
            _ => return Err(format!("unknown marker 0x{:08X}", marker)),
        };
        if size < 4 || size > data.len() {
            return Err(format!("record size {} exceeds buffer remainder {}", size, data.len()));
        }

        let mut envelope: EtlEnvelope = raw.envelope;
        envelope.filetime = header.time_delta_to_filetime(
            raw.time_delta
                .unwrap_or(buf_timestamp),
        );
        if envelope
            .provider_name
            .is_none()
        {
            envelope.provider_name = Self::resolve_provider_name(provider_names, envelope.provider_guid, raw.provider_name);
        }
        let pointer_size: usize = raw
            .pointer_size
            .unwrap_or(header.pointer_size as usize);
        let (payload, decoder): (EtlPayload, EtlDecoder) =
            Self::decode_payload(&mut envelope, raw.tl_schema, raw.user_data, pointer_size, header, schemas);

        Ok((
            EtlEvent {
                envelope,
                payload,
                decoder,
            },
            size,
        ))
    }

    /// Decode a payload by the first applicable tier: TraceLogging, kernel,
    /// (manifest: not implemented), raw.
    /*
    Work necessary for a more complete Event Trace Log parser.

    The decoding tiers below cover events whose schema is carried in the file
    (TraceLogging) plus a hard-coded subset of the kernel logger. Everything
    else falls through to `EtlPayload::Raw`. The remaining work, in decreasing
    order of value, is:

    1. Kernel `MSNT_SystemTrace` groups other than group 0.

       Kernel events (SYSTEM_TRACE_HEADER, COMPACT_TRACE_HEADER,
       PERFINFO_TRACE_HEADER) carry only a `HookId` = `(group << 8) | opcode`
       and a `Version` word; the payload layout is fixed by Windows and
       published as MOF classes (`%SystemRoot%\System32\wbem`, class
       `MSNT_SystemTrace` and its children such as `Process_TypeGroup1`,
       `Thread_TypeGroup1`, `Image_Load`, `FileIo_Name`, `DiskIo_TypeGroup1`,
       `Registry_TypeGroup1`, `PerfInfo_SampledProfile`, `StackWalk_Event`,
       `TcpIp_TypeGroup1`, `PageFault_*`). A layout is selected by the triple
       `(group, opcode, version)`; the same opcode changes layout across
       Windows releases and a single file may contain several versions
       (a WPR trace was observed with `Thread` v2, v3, and v5 events).
       Example, `Process` group 0x03 opcode 1 (`Start`):
         v0: ProcessId u32, ParentId u32, UserSID
         v1: PageDirectoryBase ptr, ProcessId, ParentId, SessionId u32,
             ExitStatus i32, UserSID, ImageFileName cstr
         v2: v1 + CommandLine wstr
         v3: UniqueProcessKey ptr, ProcessId, ParentId, SessionId,
             ExitStatus, DirectoryTableBase ptr, UserSID, ImageFileName,
             CommandLine
         v4: v3 + PackageFullName wstr, ApplicationId wstr
         v5: v4 with Flags u32 inserted before UserSID
       Field types needed: u8/u16/u32/u64/i32/i64, `ptr` (4 or 8 bytes per
       `LogfileHeader::pointer_size`), GUID, NUL-terminated ANSI and UTF-16LE
       strings, FILETIME, fixed-size byte arrays, counted arrays whose count
       is an earlier field (e.g. `StackWalk` addresses, `ProviderBinaryPath`
       GUIDs), and `UserSID` which is `2 * ptr` bytes of TOKEN_USER header
       followed by a variable-length SID (or 4 zero bytes when absent).
       The natural implementation is a `const` table
       `(group, opcode, version) -> &[(name, FieldType)]` interpreted by a
       generic walker over `Cur`, emitting `EtlValue`s, in a sibling module
       (`etlkernel.rs`) rather than growing `decode_kernel_group0`. The most
       complete machine-readable source of these layouts is the MOF-generated
       code in airbus-cert/etl-parser (directory `etl/parsers/kernel`, BSD);
       fox-it/dissect.etl (directory `dissect/etl/manifests`) has a smaller
       hand-checked subset. Full coverage is on the order of 2-3k lines of
       tables; the WPR/xperf subset (Process, Thread, Image, FileIo, DiskIo,
       Registry, PerfInfo, StackWalk, TcpIp/UdpIp, PageFault) is roughly 1k.
       tracerpt.exe decodes these from the system MOF and is a valid oracle
       for `tools/etl-compare-tracerpt.py`.

    2. Manifest-based providers (tier C; the seam is the `TODO` below).

       Events written with `EventWrite` carry only the `EVENT_DESCRIPTOR`
       (Id, Version, Channel, Level, Opcode, Task, Keywords) in the
       EVENT_HEADER. The template lives in a `WEVT_TEMPLATE` resource
       (`CRIM` container of `WEVT` provider blocks, each with `EVNT`, `TEMP`,
       `TTBL`, `MAPS`, `OPCO`, `LEVL`, `TASK`, `KEYW`, `CHAN` sub-blocks)
       compiled into the provider's DLL/EXE. Windows locates it via
       `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WINEVT\Publishers\
       {provider-guid}` (`ResourceFileName`, `MessageFileName`). The `TEMP`
       block is a BinXml template; its `<Data Name=".." inType=".."
       outType=".." length=".." count=".."/>` items give the field list and
       the `MAPS` block gives value->string maps. Field decoding is the same
       InType/OutType vocabulary already implemented for TraceLogging
       (`win:UInt32`, `win:UnicodeString`, `win:GUID`, `win:Pointer`,
       `win:Binary` with `length` referencing an earlier field, etc.).
       Portable design: accept a user-supplied directory of manifests
       (`.man` XML exported with `wevtutil gp <name> /ge /gm:true /f:xml`, or
       raw `WEVT_TEMPLATE` blobs) via a CLI option or environment variable,
       load them lazily into a `HashMap<Guid, ProviderManifest>` on first
       sight of a provider GUID, and look up `(event_id, version)`. On
       Windows an additional on-system path could read the registry and
       parse the resource directly, but that cannot help traces analyzed off
       the originating machine. `EtlDecoder` should gain a `Manifest`
       variant and `SummaryEtlReader` a corresponding counter.

    3. WPP software-tracing messages (MESSAGE_TRACE_HEADER, marker 0x90).

       The payload is a message GUID + message number + packed arguments;
       the format string and argument types were stripped at build time into
       `.tmf` files (normally embedded in the module's private `.pdb`).
       Without `.tmf` input the payload is inherently undecodable; a possible
       extension is `--etl-tmf <dir>` parsing TMF (`// PDB:` header, then
       `<guid> <name> // SRC=...` and `#typev` lines) and rendering the
       format string with the packed arguments. tracerpt's `-tmf` option
       behaves identically and prints "No Format Information found"
       otherwise, which is what the `Payload=0x..` fallback corresponds to.

    4. Remaining header and extended-data items.

       - EVENT_HEADER extended items not yet surfaced: SID (0x2),
         TS_ID (0x3), INSTANCE_INFO (0x4), STACK_TRACE32/64 (0x5/0x6, an
         array of ptr-sized return addresses preceded by a u64 MatchId),
         PROCESS_START_KEY (0xD), CONTAINER_ID (0x10), STACK_KEY (0x11/0x12).
         These could be appended as envelope fields (`Sid=`, `Stack=[...]`).
       - EVENT_HEADER `Flags` bit 0x0200 (`EVENT_HEADER_FLAG_NO_CPUTIME`)
         and `EventProperty` bits are read but not reported.
       - EVENT_INSTANCE_GUID_HEADER (0xB/0x15): `InstanceId`,
         `ParentInstanceId`, `ParentGuid` are skipped; they link related
         classic events and could populate `activity_id`/`related_activity_id`.
       - TRACE_HEADER_TYPE_ERROR (0xD), TIMED (0xC), WNODE (0xE), and any
         header type without a dedicated splitter are emitted as opaque
         `EtlHeaderKind::Error` records (size from offset 0, payload hex,
         timestamp from the buffer header); no header fields are decoded.
       - Kernel `PerfInfo` records carry the CPU number implicitly via the
         buffer's `ProcessorIndex`; a `Cpu=` envelope field would need the
         WMI_BUFFER_HEADER value threaded through `RawRecord`.

    5. File-level features.

       - Multi-file "relogged" sessions: `TRACE_LOGFILE_HEADER.BuffersWritten`
         may exceed the buffers present when a file was truncated; the parser
         stops at EOF, which is correct, but does not report the shortfall.
       - `LogFileMode` bits (real-time, circular, sequential, private,
         `EVENT_TRACE_COMPRESSED_MODE`) are decoded into the header event
         only; circular-mode files may have buffers whose `TimeStamp` is not
         monotonic and the reader relies on `EtlReader`'s BTreeMap sort.
       - Clock type 3 (CPU cycle counter) is converted with `CpuSpeedInMHz`,
         which is nominal; drift versus QPC is possible on long traces.
       - `EVENT_HEADER_FLAG_32_BIT_HEADER`/`64_BIT_HEADER` (0x0020/0x0040)
         select the pointer size for EVENT_HEADER records; other header
         kinds use `LogfileHeader::pointer_size` for the whole file even
         though their header type (`*32`/`*64`) also encodes bitness.
    */
    fn decode_payload(
        envelope: &mut EtlEnvelope,
        tl_schema: Option<&[u8]>,
        user_data: &[u8],
        pointer_size: usize,
        header: &LogfileHeader,
        schemas: &mut TlSchemaCache,
    ) -> (EtlPayload, EtlDecoder) {
        if let Some(schema_bytes) = tl_schema
            && let Some(schema) = tl_schema_cached(schemas, schema_bytes)
        {
            if !schema.event_name.is_empty() {
                envelope.event_name = Some(schema.event_name.clone());
            }
            let payload: EtlPayload = decode_tracelogging(schema, user_data, pointer_size);
            return (payload, EtlDecoder::TraceLogging);
        }
        if envelope.provider_guid == GUID_EVENT_TRACE {
            if let Some(payload) = decode_kernel_group0(envelope.opcode, user_data, header) {
                return (payload, EtlDecoder::Kernel);
            }
        }
        // TODO: tier C, manifest-based decoding keyed on (provider_guid, event_id, version)
        if user_data.is_empty() {
            return (EtlPayload::Empty, EtlDecoder::None);
        }

        (EtlPayload::Raw(user_data.to_vec()), EtlDecoder::None)
    }

    /// `SYSTEM_TRACE_HEADER`, `COMPACT_TRACE_HEADER`, `PERFINFO_TRACE_HEADER`
    fn split_system(
        data: &[u8],
        fixed_sz: usize,
        kind: EtlHeaderKind,
    ) -> std::result::Result<(RawRecord, usize), String> {
        let version_word: u16 = read_u16_at(data, 0).ok_or("truncated")?;
        let size: usize = read_u16_at(data, 4).ok_or("truncated")? as usize;
        let hook_id: u16 = read_u16_at(data, 6).ok_or("truncated")?;
        let opcode: u8 = (hook_id & 0xFF) as u8;
        let group: u8 = (hook_id >> 8) as u8;
        let header_sz: usize = fixed_sz + system_header_extra(version_word);
        if size < header_sz || size > data.len() {
            return Err(format!("{} record size {} invalid (header {})", kind.as_str(), size, header_sz));
        }
        let (tid, pid, time_delta): (Option<u32>, Option<u32>, u64) = match kind {
            EtlHeaderKind::PerfInfo => (None, None, read_u64_at(data, 8).unwrap()),
            _ => (
                Some(read_u32_at(data, 8).unwrap()),
                Some(read_u32_at(data, 0xC).unwrap()),
                read_u64_at(data, 0x10).unwrap(),
            ),
        };
        let (guid, group_name) = kernel_group_lookup(group, opcode);
        let event_name: Option<EtlName> = if group == 0 {
            kernel_group0_opcode_name(opcode).map(EtlName::from)
        } else {
            group_name.map(EtlName::from)
        };
        let envelope = EtlEnvelope {
            kind,
            provider_guid: guid,
            event_name,
            event_id: 0,
            version: (version_word & 0xFF) as u8,
            opcode,
            pid,
            tid,
            hook_id: Some(hook_id),
            ..EtlEnvelope::default()
        };

        Ok((
            RawRecord {
                envelope,
                time_delta: Some(time_delta),
                tl_schema: None,
                provider_name: None,
                pointer_size: None,
                user_data: &data[header_sz..size],
            },
            size,
        ))
    }

    /// `EVENT_HEADER` with optional `EVENT_HEADER_EXTENDED_DATA_ITEM`s
    fn split_event(data: &[u8]) -> std::result::Result<(RawRecord, usize), String> {
        let size: usize = read_u16_at(data, 0).ok_or("truncated")? as usize;
        if size < EVENT_HEADER_SZ || size > data.len() {
            return Err(format!("Event record size {} invalid", size));
        }
        let flags: u16 = read_u16_at(data, 4).unwrap();
        let tid: u32 = read_u32_at(data, 8).unwrap();
        let pid: u32 = read_u32_at(data, 0xC).unwrap();
        let time_delta: u64 = read_u64_at(data, 0x10).unwrap();
        let provider_guid: Guid = Guid::from_le_slice(&data[0x18..0x28]).unwrap();
        let event_id: u16 = read_u16_at(data, 0x28).unwrap();
        let version: u8 = data[0x2A];
        let level: u8 = data[0x2C];
        let opcode: u8 = data[0x2D];
        let task: u16 = read_u16_at(data, 0x2E).unwrap();
        let keywords: u64 = read_u64_at(data, 0x30).unwrap();
        let activity_id: Guid = Guid::from_le_slice(&data[0x40..0x50]).unwrap();

        let mut pos: usize = EVENT_HEADER_SZ;
        let mut tl_schema: Option<&[u8]> = None;
        let mut provider_name: Option<&[u8]> = None;
        let mut related_activity_id: Option<Guid> = None;
        if flags & EVENT_HEADER_FLAG_EXTENDED_INFO != 0 {
            let mut count: usize = 0;
            loop {
                if pos + 8 > size || count >= EXT_ITEMS_MAX {
                    break;
                }
                let ext_type: u16 = read_u16_at(data, pos + 2).unwrap();
                let linkage: u16 = read_u16_at(data, pos + 4).unwrap();
                let data_size: usize = read_u16_at(data, pos + 6).unwrap() as usize;
                let item_start: usize = pos + 8;
                let item_end: usize = item_start + data_size;
                if item_end > size {
                    return Err(format!("extended data item type 0x{:X} size {} overruns record", ext_type, data_size));
                }
                let item: &[u8] = &data[item_start..item_end];
                match ext_type {
                    EXT_TYPE_EVENT_SCHEMA_TL => tl_schema = Some(item),
                    EXT_TYPE_PROV_TRAITS => {
                        // u16 total size, then NUL-terminated provider name
                        if item.len() > 2 {
                            let name: &[u8] = &item[2..];
                            let end: usize = name
                                .iter()
                                .position(|b| *b == 0)
                                .unwrap_or(name.len());
                            provider_name = Some(&name[..end]);
                        }
                    }
                    EXT_TYPE_RELATED_ACTIVITYID => related_activity_id = Guid::from_le_slice(item),
                    _ => {}
                }
                pos = (item_end + 7) & !7;
                count += 1;
                if linkage & 1 == 0 {
                    break;
                }
            }
        }
        let user_start: usize = pos.min(size);
        let pointer_size: Option<usize> = if flags & EVENT_HEADER_FLAG_32_BIT_HEADER != 0 {
            Some(4)
        } else if flags & EVENT_HEADER_FLAG_64_BIT_HEADER != 0 {
            Some(8)
        } else {
            None
        };
        let envelope = EtlEnvelope {
            kind: EtlHeaderKind::Event,
            provider_guid,
            event_id,
            version,
            level,
            opcode,
            task,
            keywords,
            pid: Some(pid),
            tid: Some(tid),
            activity_id: Some(activity_id),
            related_activity_id,
            ..EtlEnvelope::default()
        };

        Ok((
            RawRecord {
                envelope,
                time_delta: Some(time_delta),
                tl_schema,
                provider_name,
                pointer_size,
                user_data: &data[user_start..size],
            },
            size,
        ))
    }

    /// `EVENT_TRACE_HEADER` and `EVENT_INSTANCE_GUID_HEADER`
    fn split_full(
        data: &[u8],
        header_sz: usize,
        kind: EtlHeaderKind,
    ) -> std::result::Result<(RawRecord, usize), String> {
        let size: usize = read_u16_at(data, 0).ok_or("truncated")? as usize;
        if size < header_sz || size > data.len() {
            return Err(format!("{} record size {} invalid", kind.as_str(), size));
        }
        let opcode: u8 = data[4];
        let level: u8 = data[5];
        let version: u16 = read_u16_at(data, 6).unwrap();
        let tid: u32 = read_u32_at(data, 8).unwrap();
        let pid: u32 = read_u32_at(data, 0xC).unwrap();
        let time_delta: u64 = read_u64_at(data, 0x10).unwrap();
        let provider_guid: Guid = Guid::from_le_slice(&data[0x18..0x28]).unwrap();
        let envelope = EtlEnvelope {
            kind,
            provider_guid,
            event_id: 0,
            version: version.min(u8::MAX as u16) as u8,
            level,
            opcode,
            pid: Some(pid),
            tid: Some(tid),
            ..EtlEnvelope::default()
        };

        Ok((
            RawRecord {
                envelope,
                time_delta: Some(time_delta),
                tl_schema: None,
                provider_name: None,
                pointer_size: None,
                user_data: &data[header_sz..size],
            },
            size,
        ))
    }

    /// `MESSAGE_TRACE_HEADER` (WPP) with `TRACE_MESSAGE_*` optional fields
    fn split_message(data: &[u8]) -> std::result::Result<(RawRecord, usize), String> {
        let size: usize = read_u16_at(data, 0).ok_or("truncated")? as usize;
        if size < MESSAGE_HEADER_SZ || size > data.len() {
            return Err(format!("Message record size {} invalid", size));
        }
        let message_number: u16 = read_u16_at(data, 4).unwrap();
        let flags: u16 = read_u16_at(data, 6).unwrap();
        let mut c = Cur::new(&data[MESSAGE_HEADER_SZ..size]);
        if flags & TRACE_MESSAGE_SEQUENCE != 0 {
            c.u32();
        }
        let mut provider_guid: Guid = Guid::NIL;
        if flags & TRACE_MESSAGE_GUID != 0 {
            provider_guid = c.guid().unwrap_or(Guid::NIL);
        } else if flags & TRACE_MESSAGE_COMPONENTID != 0 {
            c.u32();
        }
        let mut time_delta: Option<u64> = None;
        if flags & TRACE_MESSAGE_TIMESTAMP != 0 {
            time_delta = c.u64();
        }
        let (mut tid, mut pid): (Option<u32>, Option<u32>) = (None, None);
        if flags & TRACE_MESSAGE_SYSTEMINFO != 0 {
            tid = c.u32();
            pid = c.u32();
        }
        let user_start: usize = MESSAGE_HEADER_SZ + c.pos;
        let envelope = EtlEnvelope {
            kind: EtlHeaderKind::Message,
            provider_guid,
            event_id: message_number,
            pid,
            tid,
            message_number: Some(message_number),
            ..EtlEnvelope::default()
        };

        Ok((
            RawRecord {
                envelope,
                time_delta,
                tl_schema: None,
                provider_name: None,
                pointer_size: None,
                user_data: &data[user_start..size],
            },
            size,
        ))
    }

    /// `TRACE_HEADER_TYPE_ERROR` and any other header type not decoded here;
    /// `Size` is the `u16` at offset 0, contents are opaque.
    fn split_error(data: &[u8]) -> std::result::Result<(RawRecord, usize), String> {
        let size: usize = read_u16_at(data, 0).ok_or("truncated")? as usize;
        if size < 4 || size > data.len() {
            let marker: u32 = read_u32_at(data, 0).unwrap_or_default();
            return Err(format!("record size {} invalid for marker 0x{:08X}", size, marker));
        }

        Ok((
            RawRecord {
                envelope: EtlEnvelope {
                    kind: EtlHeaderKind::Error,
                    ..EtlEnvelope::default()
                },
                time_delta: None,
                tl_schema: None,
                provider_name: None,
                pointer_size: None,
                user_data: &data[4..size],
            },
            size,
        ))
    }
}

/// Extra header bytes signalled in the `Version` word of a system header.
fn system_header_extra(version_word: u16) -> usize {
    let mut extra: usize = 0;
    if version_word & 0x8000 != 0 {
        extra += 8;
    }
    extra += 8 * ((version_word & 0x0700) >> 8) as usize;

    extra
}
