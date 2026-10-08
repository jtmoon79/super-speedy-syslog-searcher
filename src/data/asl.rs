//! Parsed and printable Apple System Log events.

use std::fmt::{
    self,
    Write,
};
use std::io::{
    Error,
    ErrorKind,
    Result,
};

use compact_str::CompactString;

use crate::common::Bytes;
use crate::data::common::{
    DtBegEndPairOpt,
    PrintableEvent,
};
use crate::data::datetime::{
    DateTime,
    DateTimeL,
    FixedOffset,
    Utc,
};
use crate::data::odl::single_line_chars;

const RENDER_STACK_BYTES: usize = 1024;

struct RenderBuffer {
    stack: [u8; RENDER_STACK_BYTES],
    used: usize,
    overflow: Option<String>,
}

/// Helper structure for stack-allocated `render` buffer.
/// After writing, the content is transferred to a `String`.
impl RenderBuffer {
    fn new() -> Self {
        Self {
            stack: [0; RENDER_STACK_BYTES],
            used: 0,
            overflow: None,
        }
    }

    fn len(&self) -> usize {
        self.used
            + self
                .overflow
                .as_ref()
                .map_or(0, String::len)
    }

    fn into_string(self) -> String {
        let mut text = String::with_capacity(self.len());
        text.push_str(std::str::from_utf8(&self.stack[..self.used]).expect("formatted text is UTF-8"));
        if let Some(overflow) = self.overflow {
            text.push_str(&overflow);
        }

        text
    }
}

impl Write for RenderBuffer {
    fn write_str(
        &mut self,
        text: &str,
    ) -> fmt::Result {
        if let Some(overflow) = &mut self.overflow {
            overflow.push_str(text);
        } else if text.len() <= self.stack.len() - self.used {
            self.stack[self.used..self.used + text.len()].copy_from_slice(text.as_bytes());
            self.used += text.len();
        } else {
            self.overflow = Some(text.to_owned());
        }

        Ok(())
    }
}

struct SingleLine<'a>(&'a str);

impl fmt::Display for SingleLine<'_> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for character in single_line_chars(self.0) {
            formatter.write_char(character)?;
        }

        Ok(())
    }
}

pub const LEVELS: [&str; 8] = [
    "Emergency",
    "Alert",
    "Critical",
    "Error",
    "Warning",
    "Notice",
    "Info",
    "Debug",
];

/// A version-2 ASL database record. String references have been resolved.
/// Short strings are stored inline; longer strings retain heap-backed storage.
#[derive(Debug)]
pub struct AslRecord {
    pub offset: u64,
    pub ordinal: u64,
    pub id: u64,
    pub seconds: u64,
    pub nanoseconds: u32,
    pub level: u16,
    pub flags: u16,
    pub pid: u32,
    pub uid: u32,
    pub gid: u32,
    pub read_uid: u32,
    pub read_gid: u32,
    pub ref_pid: u32,
    pub host: CompactString,
    pub sender: CompactString,
    pub facility: CompactString,
    pub message: CompactString,
    pub ref_proc: CompactString,
    pub session: CompactString,
    pub extra: Vec<(CompactString, CompactString)>,
}

impl AslRecord {
    pub fn render(
        &self,
        fixed_offset: &FixedOffset,
    ) -> Result<Asl> {
        let seconds =
            i64::try_from(self.seconds).map_err(|_| Error::new(ErrorKind::InvalidData, "ASL timestamp exceeds i64"))?;
        let dt = DateTime::<Utc>::from_timestamp(seconds, self.nanoseconds)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "unrepresentable ASL timestamp"))?
            .with_timezone(fixed_offset);
        let mut text = RenderBuffer::new();
        write!(text, "{}", dt.format("%Y-%m-%dT%H:%M:%S%.9f%:z"))
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        let dt_end = text.len();

        write!(
            text,
            "  id={}  level={}  pid={}  uid={}  gid={}  read_uid={}  read_gid={}",
            self.id,
            LEVELS
                .get(usize::from(self.level))
                .copied()
                .unwrap_or("Other"),
            self.pid,
            self.uid,
            self.gid,
            self.read_uid,
            self.read_gid,
        )
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        if self.ref_pid != 0 {
            write!(text, "  ref_pid={}", self.ref_pid).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        }
        if self.flags != 0 {
            write!(text, "  flags=0x{:x}", self.flags).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        }
        if !self.host.is_empty() {
            write!(text, "  host={}", SingleLine(&self.host))
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        }
        if !self.ref_proc.is_empty() {
            write!(text, "  RefProc={}", SingleLine(&self.ref_proc))
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        }
        if !self.session.is_empty() {
            write!(text, "  session={}", SingleLine(&self.session))
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        }
        write!(
            text,
            "  sender={}  facility={}  message='{}'",
            SingleLine(&self.sender),
            SingleLine(&self.facility),
            SingleLine(&self.message),
        )
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        for (key, value) in &self.extra {
            write!(text, "  {}={}", SingleLine(key), SingleLine(value))
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        }
        writeln!(text).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        let text: String = text.into_string();

        Ok(Asl {
            dt,
            dt_beg_end: Some((0, dt_end)),
            data: text.into_bytes(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asl {
    dt: DateTimeL,
    dt_beg_end: DtBegEndPairOpt,
    data: Bytes,
}

impl Asl {
    pub const fn dt(&self) -> &DateTimeL {
        &self.dt
    }
    pub const fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    pub const fn len(&self) -> usize {
        self.data.len()
    }
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl PrintableEvent for Asl {
    fn dt(&self) -> &DateTimeL {
        self.dt()
    }
    fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        self.dt_beg_end()
    }
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
