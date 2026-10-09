//! Version-2 Apple System Log database parser.
//! Eight-byte string references encode up to seven inline UTF-8 bytes or a file
//! offset. Resolved inline strings are stored without a heap allocation.

use std::collections::HashSet;
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom};

use compact_str::CompactString;

use crate::common::Bytes;
use crate::data::asl::AslRecord;
use crate::debug::printers::buffer_to_string_noraw;

const HEADER_LEN: u64 = 80;
const RECORD_HEADER_LEN: u64 = 6;
const RECORD_DATA_MIN: usize = 116;
const RECORD_BYTES_MAX: usize = 4 * 1024 * 1024;
const STRING_BYTES_MAX: usize = 4 * 1024 * 1024;
const SIGNATURE: &[u8; 12] = b"ASL DB\0\0\0\0\0\0";

fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidData, message.into())
}

fn be_u16(bytes: &[u8]) -> u16 {
    u16::from_be_bytes(
        bytes
            .try_into()
            .expect("two-byte slice"),
    )
}
fn be_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(
        bytes
            .try_into()
            .expect("four-byte slice"),
    )
}
fn be_u64(bytes: &[u8]) -> u64 {
    u64::from_be_bytes(
        bytes
            .try_into()
            .expect("eight-byte slice"),
    )
}

pub struct AslParser<R: Read + Seek> {
    reader: R,
    file_len: u64,
    next_offset: u64,
    last_offset: u64,
    ordinal: u64,
    seen: HashSet<u64>,
}

impl<R: Read + Seek> AslParser<R> {
    pub fn new(mut reader: R) -> io::Result<Self> {
        let file_len = reader.seek(SeekFrom::End(0))?;
        if file_len < HEADER_LEN {
            return Err(invalid("ASL header is truncated"));
        }
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0; HEADER_LEN as usize];
        reader.read_exact(&mut header)?;
        if &header[..12] != SIGNATURE {
            let s = buffer_to_string_noraw(&header[..12]);
            return Err(invalid(format!("invalid ASL database signature \"{}\"", s)));
        }
        const DB_VERSION: u32 = 2;
        if be_u32(&header[12..16]) != DB_VERSION {
            let v: u32 = be_u32(&header[12..16]);
            return Err(invalid(format!("unsupported ASL database version {v} (0x{v:08X}), expected {DB_VERSION}")));
        }
        let next_offset = be_u64(&header[16..24]);
        // Version-2 stores a one-byte mask before the last-record offset.
        let last_offset = be_u64(&header[37..45]);
        if (next_offset == 0) != (last_offset == 0)
            || (next_offset != 0 && (next_offset < HEADER_LEN || last_offset < HEADER_LEN))
        {
            return Err(invalid(
                format!("invalid ASL first record offset {next_offset} or last record offset {last_offset}")
            ));
        }

        Ok(Self {
            reader,
            file_len,
            next_offset,
            last_offset,
            ordinal: 0,
            seen: HashSet::new(),
        })
    }

    fn string_ref(
        &mut self,
        reference: u64,
    ) -> io::Result<CompactString> {
        if reference == 0 {
            return Ok(CompactString::default());
        }
        if reference & (1 << 63) != 0 {
            let raw = reference.to_be_bytes();
            let len = usize::from(raw[0] & 0x7f);
            if len > 7 {
                return Err(invalid(format!("invalid inline ASL string length {len}, must be [0, 7]")));
            }
            return std::str::from_utf8(&raw[1..1 + len])
                .map(CompactString::new)
                .map_err(|error| invalid(format!("invalid inline ASL UTF-8: {error}")));
        }
        if reference < HEADER_LEN
            || reference
                .checked_add(6)
                .is_none_or(|end| end > self.file_len)
        {
            return Err(invalid(format!("ASL string offset {reference} is outside the file")));
        }
        self.reader
            .seek(SeekFrom::Start(reference))?;
        let mut header = [0; 6];
        self.reader
            .read_exact(&mut header)?;
        if be_u16(&header[..2]) != 1 {
            return Err(invalid(format!("invalid ASL string tag at offset {reference}")));
        }
        let size = be_u32(&header[2..6]) as usize;
        if size == 0 || size > STRING_BYTES_MAX || reference + 6 + size as u64 > self.file_len {
            return Err(invalid(format!("invalid ASL string length at offset {reference}")));
        }
        let mut bytes: Bytes = vec![0; size];
        self.reader
            .read_exact(&mut bytes)?;
        if bytes.last() != Some(&0) {
            return Err(invalid(format!("unterminated ASL string at offset {reference}")));
        }
        bytes.pop();

        String::from_utf8(bytes)
            .map(CompactString::from)
            .map_err(|error| invalid(format!("invalid ASL UTF-8 at offset {reference}: {error}")))
    }

    pub fn next_record(&mut self) -> io::Result<Option<AslRecord>> {
        let offset = self.next_offset;
        if offset == 0 {
            return Ok(None);
        }
        if !self.seen.insert(offset) {
            return Err(invalid(format!("ASL record chain contains a cycle at offset {offset}")));
        }
        if offset < HEADER_LEN
            || offset
                .checked_add(RECORD_HEADER_LEN)
                .is_none_or(|end| end > self.file_len)
        {
            return Err(invalid(format!("ASL record offset {offset} is outside the file")));
        }
        self.reader
            .seek(SeekFrom::Start(offset))?;
        let mut header = [0; RECORD_HEADER_LEN as usize];
        self.reader
            .read_exact(&mut header)?;
        if be_u16(&header[..2]) != 0 {
            return Err(invalid(format!("invalid ASL record tag at offset {offset}")));
        }
        let size = be_u32(&header[2..6]) as usize;
        if !(RECORD_DATA_MIN..=RECORD_BYTES_MAX).contains(&size)
            || offset + RECORD_HEADER_LEN + size as u64 > self.file_len
        {
            return Err(invalid(format!("invalid ASL record length at offset {offset}")));
        }
        let mut data = vec![0; size];
        self.reader
            .read_exact(&mut data)?;
        let kv_count = be_u32(&data[56..60]) as usize;
        let extra_count = kv_count / 2;
        if kv_count % 2 != 0
            || extra_count
                .checked_mul(16)
                .and_then(|len| len.checked_add(RECORD_DATA_MIN))
                != Some(size)
        {
            return Err(invalid(format!("invalid ASL extra fields at offset {offset}")));
        }
        let next = be_u64(&data[..8]);
        if (offset == self.last_offset) != (next == 0) || (next != 0 && (next < HEADER_LEN || next >= self.file_len)) {
            return Err(invalid(format!("invalid ASL next record offset at {offset}")));
        }
        let mut extra = Vec::with_capacity(extra_count);
        for pair in data[108..size - 8].chunks_exact(16) {
            extra.push((self.string_ref(be_u64(&pair[..8]))?, self.string_ref(be_u64(&pair[8..]))?));
        }
        let record = AslRecord {
            offset,
            ordinal: self.ordinal,
            id: be_u64(&data[8..16]),
            seconds: be_u64(&data[16..24]),
            nanoseconds: be_u32(&data[24..28]),
            level: be_u16(&data[28..30]),
            flags: be_u16(&data[30..32]),
            pid: be_u32(&data[32..36]),
            uid: be_u32(&data[36..40]),
            gid: be_u32(&data[40..44]),
            read_uid: be_u32(&data[44..48]),
            read_gid: be_u32(&data[48..52]),
            ref_pid: be_u32(&data[52..56]),
            host: self.string_ref(be_u64(&data[60..68]))?,
            sender: self.string_ref(be_u64(&data[68..76]))?,
            facility: self.string_ref(be_u64(&data[76..84]))?,
            message: self.string_ref(be_u64(&data[84..92]))?,
            ref_proc: self.string_ref(be_u64(&data[92..100]))?,
            session: self.string_ref(be_u64(&data[100..108]))?,
            extra,
        };
        self.next_offset = next;
        self.ordinal += 1;

        Ok(Some(record))
    }
}
