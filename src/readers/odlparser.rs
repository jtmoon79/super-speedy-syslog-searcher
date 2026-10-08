//! Streaming OneDrive Log v2/v3 parser for any [`Read`] source.
//!
//! Internal gzip follows the file header; outer compression belongs to
//! [`OdlReader`](crate::readers::odlreader::OdlReader).
//! Layout references: <https://www.swiftforensics.com/2022/02/reading-onedrive-logs.html>
//! and <https://www.swiftforensics.com/2022/11/reading-onedrive-logs-part-2.html>.

use std::collections::HashMap;
use std::fmt;
use std::io::{
    self,
    BufRead,
    BufReader,
    Error,
    ErrorKind,
    Read,
};

use aes::{
    Aes128,
    Aes192,
    Aes256,
};
use base64::Engine;
use base64::engine::general_purpose::{
    STANDARD,
    URL_SAFE_NO_PAD,
};
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{
    BlockDecryptMut,
    KeyIvInit,
};
use flate2::bufread::MultiGzDecoder;
use zeroize::{
    Zeroize,
    Zeroizing,
};

use crate::data::odl::{
    OdlEvent,
    single_line,
};

/// Corpus maximum payload is 6412 bytes; allow over 160 times that size.
pub const ODL_RECORD_BYTES_MAX: usize = 1024 * 1024;
/// Applies to source/function strings and extracted string parameters.
pub const ODL_STRING_BYTES_MAX: usize = 64 * 1024;
/// v3 context length is a u16; the observed maximum is 164 bytes.
pub const ODL_CONTEXT_BYTES_MAX: usize = u16::MAX as usize;
pub const ODL_PARAMETERS_MAX: usize = 4096;
pub const ODL_DECODED_BYTES_MAX: usize = 4 * 1024 * 1024;
pub const ODL_COMPANION_BYTES_MAX: usize = 16 * 1024 * 1024;
const ODL_MAP_VALUES_MAX: usize = 100_000;
const ODL_KEYS_MAX: usize = 64;

fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidData, message.into())
}

fn read_bounded(reader: impl Read) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    reader
        .take((ODL_COMPANION_BYTES_MAX + 1) as u64)
        .read_to_end(&mut data)?;
    if data.len() > ODL_COMPANION_BYTES_MAX {
        return Err(invalid("ODL companion exceeds size limit"));
    }
    Ok(data)
}

fn decode_text(data: &[u8]) -> io::Result<String> {
    if data.starts_with(&[0xff, 0xfe]) || (data.len() >= 4 && data[1] == 0 && data[3] == 0) {
        let data = data
            .strip_prefix(&[0xff, 0xfe])
            .unwrap_or(data);
        if data.len() % 2 != 0 {
            return Err(invalid("odd-length UTF-16LE companion"));
        }
        let units: Vec<u16> = data
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        String::from_utf16(&units).map_err(|_| invalid("invalid UTF-16LE companion"))
    } else {
        let data = data
            .strip_prefix(&[0xef, 0xbb, 0xbf])
            .unwrap_or(data);
        std::str::from_utf8(data)
            .map(str::to_owned)
            .map_err(|_| invalid("invalid UTF-8 companion"))
    }
}

/// Optional per-source deobfuscation. Debug output never contains keys or maps.
#[derive(Default)]
pub struct OdlDecodingContext {
    map: HashMap<String, String>,
    keys: Vec<Zeroizing<Vec<u8>>>,
}

impl fmt::Debug for OdlDecodingContext {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("OdlDecodingContext")
            .field("map_entries", &self.map.len())
            .field("keys", &self.keys.len())
            .finish()
    }
}

impl Drop for OdlDecodingContext {
    fn drop(&mut self) {
        for (mut key, mut value) in self.map.drain() {
            key.zeroize();
            value.zeroize();
        }
    }
}

impl OdlDecodingContext {
    pub fn has_companions(&self) -> bool {
        !self.map.is_empty() || !self.keys.is_empty()
    }

    /// Load UTF-8/UTF-16LE legacy maps, retaining repeated-value ambiguity.
    pub fn load_map(
        &mut self,
        reader: impl Read,
    ) -> io::Result<()> {
        let data = Zeroizing::new(read_bounded(reader)?);
        let text = Zeroizing::new(decode_text(&data)?);
        let mut map: HashMap<String, String> = HashMap::new();
        let mut last_key = None;
        let mut entries = 0;
        for line in text.lines() {
            if let Some((key, value)) = line.split_once('\t') {
                if key.is_empty() {
                    return Err(invalid("empty ODL obfuscation-map key"));
                }
                entries += 1;
                if entries > ODL_MAP_VALUES_MAX {
                    return Err(invalid("ODL obfuscation-map entry limit exceeded"));
                }
                let entry = map
                    .entry(key.to_owned())
                    .or_default();
                if !entry.is_empty() {
                    entry.push('|');
                }
                entry.push_str(value);
                if entry.len() > ODL_STRING_BYTES_MAX || key.len() > ODL_STRING_BYTES_MAX {
                    return Err(invalid("ODL obfuscation-map string exceeds limit"));
                }
                last_key = Some(key.to_owned());
            } else if !line.is_empty() {
                let key = last_key
                    .as_ref()
                    .ok_or_else(|| invalid("ODL map continuation without a key"))?;
                let value = map
                    .get_mut(key)
                    .ok_or_else(|| invalid("ODL map missing key"))?;
                value.push('\n');
                value.push_str(line);
                if value.len() > ODL_STRING_BYTES_MAX {
                    return Err(invalid("ODL obfuscation-map string exceeds limit"));
                }
            }
        }
        self.map = map;
        Ok(())
    }

    /// Load version-1 JSON keystores. Key material is not included in errors.
    pub fn load_keystore(
        &mut self,
        reader: impl Read,
    ) -> io::Result<()> {
        let data = Zeroizing::new(read_bounded(reader)?);
        let text = Zeroizing::new(decode_text(&data)?);
        let mut value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| invalid("invalid ODL keystore JSON"))?;
        let result = (|| {
            let entries = value
                .as_array()
                .ok_or_else(|| invalid("ODL keystore must be an array"))?;
            if entries.is_empty() || entries.len() + self.keys.len() > ODL_KEYS_MAX {
                return Err(invalid("invalid ODL keystore key count"));
            }
            let mut keys = Vec::new();
            for entry in entries {
                if entry
                    .get("Version")
                    .and_then(serde_json::Value::as_u64)
                    != Some(1)
                {
                    return Err(invalid("unsupported ODL keystore version"));
                }
                let text = entry
                    .get("Key")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| invalid("ODL keystore missing Key"))?;
                let key = Zeroizing::new(
                    STANDARD
                        .decode(text.trim_end_matches('\0'))
                        .map_err(|_| invalid("invalid ODL keystore base64"))?,
                );
                if !matches!(key.len(), 16 | 24 | 32) {
                    return Err(invalid("unsupported ODL AES key length"));
                }
                keys.push(key);
            }
            self.keys.extend(keys);
            Ok(())
        })();
        if let Some(entries) = value.as_array_mut() {
            for entry in entries {
                if let Some(serde_json::Value::String(key)) = entry.get_mut("Key") {
                    key.zeroize();
                }
            }
        }
        result
    }

    fn decode_token(
        &self,
        token: &str,
    ) -> (String, bool) {
        if let Some(value) = self.map.get(token) {
            return (value.clone(), false);
        }
        if token.len() < 22 {
            return (token.to_owned(), false);
        }
        let mut bytes = match URL_SAFE_NO_PAD
            .decode(token.trim_end_matches('='))
            .or_else(|_| STANDARD.decode(token))
        {
            Ok(bytes) if !bytes.is_empty() && bytes.len() % 16 == 0 => Zeroizing::new(bytes),
            _ => return (token.to_owned(), false),
        };
        let iv = [0u8; 16];
        for key in &self.keys {
            let mut plain = Zeroizing::new(bytes.to_vec());
            let result = match key.len() {
                16 => cbc::Decryptor::<Aes128>::new_from_slices(key, &iv)
                    .map_err(|_| ())
                    .and_then(|c| {
                        c.decrypt_padded_mut::<Pkcs7>(&mut plain)
                            .map_err(|_| ())
                    }),
                24 => cbc::Decryptor::<Aes192>::new_from_slices(key, &iv)
                    .map_err(|_| ())
                    .and_then(|c| {
                        c.decrypt_padded_mut::<Pkcs7>(&mut plain)
                            .map_err(|_| ())
                    }),
                32 => cbc::Decryptor::<Aes256>::new_from_slices(key, &iv)
                    .map_err(|_| ())
                    .and_then(|c| {
                        c.decrypt_padded_mut::<Pkcs7>(&mut plain)
                            .map_err(|_| ())
                    }),
                _ => continue,
            };
            if let Ok(data) = result {
                if data.len() % 2 == 0 {
                    let units: Vec<u16> = data
                        .chunks_exact(2)
                        .map(|b| u16::from_le_bytes([b[0], b[1]]))
                        .collect();
                    if let Ok(text) = String::from_utf16(&units) {
                        if !text
                            .chars()
                            .any(|c| c == '\0')
                        {
                            return (text, false);
                        }
                    }
                }
            }
        }
        bytes.zeroize();
        (token.to_owned(), true)
    }

    fn decode_parameter(
        &self,
        text: &str,
    ) -> io::Result<(String, usize)> {
        let separators = r#":\.@%#&*|{}!?<>;~()/\"'"#;
        let mut result = String::new();
        let mut failures = 0;
        let mut start = 0;
        for (offset, character) in text.char_indices() {
            if separators.contains(character) {
                if start < offset {
                    let (value, failed) = self.decode_token(&text[start..offset]);
                    if result.len() + value.len() > ODL_STRING_BYTES_MAX {
                        return Err(invalid("decoded ODL parameter exceeds string limit"));
                    }
                    result.push_str(&value);
                    failures += usize::from(failed);
                }
                result.push(character);
                start = offset + character.len_utf8();
            }
        }
        if start < text.len() {
            let (value, failed) = self.decode_token(&text[start..]);
            if result.len() + value.len() > ODL_STRING_BYTES_MAX {
                return Err(invalid("decoded ODL parameter exceeds string limit"));
            }
            result.push_str(&value);
            failures += usize::from(failed);
        }
        if result.len() > ODL_STRING_BYTES_MAX {
            return Err(invalid("decoded ODL parameter exceeds string limit"));
        }
        Ok((single_line(&result), failures))
    }
}

#[derive(Clone, Debug)]
pub struct OdlHeader {
    pub version: u32,
    pub one_drive_version: String,
    pub platform_version: String,
    pub compressed: bool,
}

#[derive(Debug)]
pub enum OdlRecordError {
    Skipped { ordinal: u64, offset: u64, error: Error },
    Fatal { ordinal: u64, offset: u64, error: Error },
}

impl fmt::Display for OdlRecordError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Skipped { ordinal, offset, error } => {
                write!(f, "ODL record {ordinal} at logical offset {offset}: {error} (skipped)")
            }
            Self::Fatal { ordinal, offset, error } => {
                write!(f, "ODL record {ordinal} at logical offset {offset}: {error} (fatal)")
            }
        }
    }
}

impl std::error::Error for OdlRecordError {}

enum Body<R: Read> {
    Plain(BufReader<R>),
    Gzip(MultiGzDecoder<BufReader<R>>),
}

impl<R: Read> Read for Body<R> {
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> io::Result<usize> {
        match self {
            Self::Plain(reader) => reader.read(bytes),
            Self::Gzip(reader) => reader.read(bytes),
        }
    }
}

pub struct OdlParser<R: Read> {
    body: Body<R>,
    header: OdlHeader,
    decoding: OdlDecodingContext,
    offset: u64,
    ordinal: u64,
    done: bool,
    pub records_skipped: u64,
}

impl<R: Read> OdlParser<R> {
    pub fn new(
        mut reader: R,
        decoding: OdlDecodingContext,
    ) -> io::Result<Self> {
        let mut bytes = [0u8; 256];
        reader.read_exact(&mut bytes[..12])?;
        if &bytes[..8] != b"EBFGONED" {
            return Err(invalid("invalid ODL file signature"));
        }
        let version = u32::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11],
        ]);
        if !matches!(version, 2 | 3) {
            return Err(Error::new(ErrorKind::Unsupported, format!("unsupported ODL version {version}")));
        }
        reader.read_exact(&mut bytes[12..])?;
        let version_text = |data: &[u8]| {
            let end = data
                .iter()
                .position(|b| *b == 0)
                .unwrap_or(data.len());
            std::str::from_utf8(&data[..end])
                .map(str::to_owned)
                .map_err(|_| invalid("invalid ODL header version-string encoding"))
        };
        let mut reader = BufReader::new(reader);
        let compressed = loop {
            match reader.fill_buf() {
                Ok(prefix) => break prefix.first() == Some(&0x1f),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        };
        let header = OdlHeader {
            version,
            one_drive_version: version_text(&bytes[28..92])?,
            platform_version: version_text(&bytes[92..156])?,
            compressed,
        };
        let body = if compressed { Body::Gzip(MultiGzDecoder::new(reader)) } else { Body::Plain(reader) };
        Ok(Self {
            body,
            header,
            decoding,
            offset: 0,
            ordinal: 0,
            done: false,
            records_skipped: 0,
        })
    }

    pub const fn header(&self) -> &OdlHeader {
        &self.header
    }

    fn read_record(&mut self) -> io::Result<Option<OdlEvent>> {
        let size = if self.header.version == 2 { 56 } else { 32 };
        let mut header = [0u8; 56];
        loop {
            match self
                .body
                .read(&mut header[..1])
            {
                Ok(0) => return Ok(None),
                Ok(_) => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        self.body
            .read_exact(&mut header[1..size])?;
        if &header[..4] != b"\xcc\xdd\xee\xff" {
            return Err(invalid("invalid ODL record signature"));
        }
        let length_offset = size - 8;
        let length = u32::from_le_bytes([
            header[length_offset],
            header[length_offset + 1],
            header[length_offset + 2],
            header[length_offset + 3],
        ]) as usize;
        if length > ODL_RECORD_BYTES_MAX {
            return Err(invalid("ODL record exceeds size limit"));
        }
        let mut payload = vec![0; length];
        self.body
            .read_exact(&mut payload)?;
        self.ordinal += 1;
        let offset = self.offset;
        self.offset = self
            .offset
            .checked_add((size + length) as u64)
            .ok_or_else(|| invalid("ODL stream offset overflow"))?;
        let context_length = if self.header.version == 2 {
            0
        } else {
            match u16::from_le_bytes([header[4], header[5]]) {
                0 => 24,
                length => usize::from(length),
            }
        };
        let result = (|| {
            if context_length > ODL_CONTEXT_BYTES_MAX || context_length > payload.len() {
                return Err(invalid("ODL context exceeds record payload"));
            }
            let mut position = context_length;
            let source_file = read_string(&payload, &mut position)?;
            let flag_bytes = payload
                .get(position..position + 4)
                .ok_or_else(|| invalid("missing ODL flags"))?;
            let flags = u32::from_le_bytes([
                flag_bytes[0],
                flag_bytes[1],
                flag_bytes[2],
                flag_bytes[3],
            ]);
            position += 4;
            let function = read_string(&payload, &mut position)?;
            let parameter_bytes = payload[position..].to_vec();
            let (parameters, undecoded_bytes, decoding_failures) =
                extract_parameters(&parameter_bytes, &self.decoding)?;
            Ok(OdlEvent {
                timestamp_ms: u64::from_le_bytes([
                    header[8], header[9], header[10], header[11], header[12], header[13], header[14], header[15],
                ]),
                ordinal: self.ordinal,
                offset,
                source_file,
                function,
                flags,
                context: if self.header.version == 2 {
                    header[24..48].to_vec()
                } else {
                    payload[..context_length].to_vec()
                },
                parameter_bytes,
                parameters,
                undecoded_bytes,
                decoding_failures,
            })
        })();
        match result {
            Ok(event) => Ok(Some(event)),
            Err(error) => {
                self.records_skipped += 1;
                Err(Error::new(error.kind(), error))
            }
        }
    }

    pub fn next_event(&mut self) -> Option<Result<OdlEvent, OdlRecordError>> {
        if self.done {
            return None;
        }
        let ordinal = self.ordinal;
        let offset = self.offset;
        match self.read_record() {
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(error) if self.ordinal > ordinal => Some(Err(OdlRecordError::Skipped {
                ordinal: self.ordinal,
                offset,
                error,
            })),
            Err(error) => {
                self.done = true;
                Some(Err(OdlRecordError::Fatal {
                    ordinal: ordinal + 1,
                    offset,
                    error,
                }))
            }
        }
    }
}

fn read_string(
    payload: &[u8],
    position: &mut usize,
) -> io::Result<String> {
    let bytes = payload
        .get(*position..*position + 4)
        .ok_or_else(|| invalid("missing ODL string length"))?;
    let length = u32::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
    ]) as usize;
    if length > ODL_STRING_BYTES_MAX {
        return Err(invalid("ODL string exceeds size limit"));
    }
    *position += 4;
    let end = position
        .checked_add(length)
        .ok_or_else(|| invalid("ODL string length overflow"))?;
    let text = payload
        .get(*position..end)
        .ok_or_else(|| invalid("ODL string exceeds payload"))?;
    *position = end;
    std::str::from_utf8(text)
        .map(str::to_owned)
        .map_err(|_| invalid("invalid ODL string UTF-8"))
}

fn extract_parameters(
    bytes: &[u8],
    decoding: &OdlDecodingContext,
) -> io::Result<(Vec<String>, usize, usize)> {
    let mut parameters = Vec::new();
    let mut offset = 0;
    let mut unknown = 0;
    let mut failures = 0;
    let mut decoded_bytes = 0;
    while offset + 4 <= bytes.len() {
        let length = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        let end = (offset + 4).checked_add(length);
        let text = end
            .filter(|end| length > 0 && length <= ODL_STRING_BYTES_MAX && *end <= bytes.len())
            .and_then(|end| std::str::from_utf8(&bytes[offset + 4..end]).ok())
            .filter(|text| {
                !text
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t'))
            });
        if let Some(text) = text {
            if parameters.len() == ODL_PARAMETERS_MAX {
                return Err(invalid("ODL parameter count exceeds limit"));
            }
            let (parameter, failed) = decoding.decode_parameter(text)?;
            decoded_bytes += parameter.len() + 1;
            if decoded_bytes > ODL_DECODED_BYTES_MAX {
                return Err(invalid("decoded ODL parameters exceed event limit"));
            }
            parameters.push(parameter);
            failures += failed;
            offset += 4 + length;
        } else {
            unknown += 1;
            offset += 1;
        }
    }
    unknown += bytes.len() - offset;
    Ok((parameters, unknown, failures))
}
