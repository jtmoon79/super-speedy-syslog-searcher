//! Parsed and printable OneDrive Log events.

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

#[allow(unused_imports)]
use ::si_trace_print::{
    def2n,
    def2o,
    def2x,
    defn,
    defo,
    defx,
};

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

/// Unix timestamp in milliseconds stored in an ODL record.
pub type TimestampType = u64;

/// A decoded record. Unknown context and parameter bytes remain available.
/// Short text fields and parameters are stored inline; longer text remains heap-backed.
pub struct OdlEvent {
    pub timestamp_ms: TimestampType,
    pub ordinal: u64,
    pub offset: u64,
    pub source_file: CompactString,
    pub function: CompactString,
    pub flags: u32,
    pub context: Bytes,
    pub parameter_bytes: Bytes,
    pub parameters: Vec<CompactString>,
    /// summary statistic
    pub undecoded_bytes: usize,
    /// summary statistic
    pub decoding_failures: usize,
}

impl fmt::Debug for OdlEvent {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("OdlEvent")
            .field("timestamp_ms", &self.timestamp_ms)
            .field("ordinal", &self.ordinal)
            .field("offset", &self.offset)
            .field("context_bytes", &self.context.len())
            .field("parameter_bytes", &self.parameter_bytes.len())
            .finish()
    }
}

impl OdlEvent {
    fn render_capacity_estimate(&self) -> usize {
        // ISO 8601 with milliseconds and offset, allowing Chrono's extended years.
        32 + 4
            + self.source_file.len()
            + self.function.len()
            + self
                .parameters
                .iter()
                .map(CompactString::len)
                .sum::<usize>()
            + self.parameters.len()
    }

    pub fn render(
        &self,
        fixed_offset: &FixedOffset,
    ) -> Result<Odl> {
        def2n!("fixed_offset {:?}", fixed_offset);

        let timestamp = i64::try_from(self.timestamp_ms)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "ODL timestamp exceeds i64"))?;
        let dt = DateTime::<Utc>::from_timestamp_millis(timestamp)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "unrepresentable ODL timestamp"))?
            .with_timezone(fixed_offset);
        let rce: usize = self.render_capacity_estimate();
        def2o!("render_capacity_estimate={}", rce);
        let mut text = String::with_capacity(rce);
        dt.naive_local()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .write_to(&mut text)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        // RFC 3339 offsets have minute precision, rounded as in Chrono's formatter.
        let offset_seconds = fixed_offset.local_minus_utc();
        let offset_minutes = (offset_seconds.unsigned_abs() + 30) / 60;
        write!(
            text,
            "{}{:02}:{:02}",
            if offset_seconds < 0 { '-' } else { '+' },
            offset_minutes / 60,
            offset_minutes % 60,
        )
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        let dt_end = text.len();
        text.push(' ');
        text.extend(single_line_chars(&self.source_file));
        text.push(':');
        text.extend(single_line_chars(&self.function));
        text.push(';');
        for parameter in &self.parameters {
            text.push(' ');
            text.push_str(parameter);
        }
        text.push('\n');
        def2x!("dt={:?}, rendered text length={}", dt, text.len());

        Ok(Odl {
            dt,
            dt_beg_end: Some((0, dt_end)),
            data: text.into_bytes(),
        })
    }
}

/// Replace control characters with spaces without allocating.
pub(crate) fn single_line_chars(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
}

#[derive(Clone, PartialEq, Eq)]
pub struct Odl {
    dt: DateTimeL,
    dt_beg_end: DtBegEndPairOpt,
    data: Bytes,
}

impl fmt::Debug for Odl {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("Odl")
            .field("dt", &self.dt)
            .field("dt_beg_end", &self.dt_beg_end)
            .field("bytes", &self.data.len())
            .finish()
    }
}

impl Odl {
    pub const fn dt(&self) -> &DateTimeL {
        &self.dt
    }
    pub const fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl PrintableEvent for Odl {
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
