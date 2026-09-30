// src/data/etl.rs

//! Data representation of a Windows [Event Trace Log] (`.etl`) event.
//!
//! [`EtlEvent`] is the decoded form produced by the [`EtlParser`].
//! [`Etl`] is the rendered, printable form sent to the main printing thread.
//!
//! [Event Trace Log]: https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing
//! [`EtlParser`]: crate::readers::etlparser::EtlParser

use std::fmt::{self, Write as _};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

use ::chrono::{
    Datelike,
    Timelike,
};
use ::numtoa::NumToA;

use crate::common::{
    Bytes,
    NLc,
};
use crate::data::common::{
    DtBegEndPair,
    DtBegEndPairOpt,
};
use crate::data::datetime::{
    DateTime,
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    Utc,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::buffer_to_string_noraw;

// ---------
// FILETIME

/// Windows `FILETIME`; 100-nanosecond intervals since 1601-01-01T00:00:00Z.
pub type FileTime = u64;

/// `FILETIME` value of the Unix epoch 1970-01-01T00:00:00Z.
pub const FILETIME_UNIX_EPOCH: FileTime = 116_444_736_000_000_000;

/// Convert a `FILETIME` to a [`DateTimeL`] in the given `fixed_offset`.
pub fn filetime_to_datetimel(
    filetime: FileTime,
    fixed_offset: &FixedOffset,
) -> DateTimeLOpt {
    let ticks: i128 = filetime as i128 - FILETIME_UNIX_EPOCH as i128;
    let ns: i64 = match i64::try_from(ticks * 100) {
        Ok(ns) => ns,
        Err(_) => return None,
    };
    let dt_utc: DateTime<Utc> = DateTime::<Utc>::from_timestamp_nanos(ns);

    Some(dt_utc.with_timezone(fixed_offset))
}

/// Format a [`DateTimeL`] as ISO 8601 with seven fractional digits
/// (100 ns resolution, matching `FILETIME`), e.g.
/// `2026-09-24T23:02:44.3570123+00:00`.
///
/// chrono has no `%.7f` so this is done by hand.
pub fn format_datetime_etl(dt: &DateTimeL) -> String {
    let mut s: String = String::with_capacity(DATETIME_ETL_LEN);
    push_datetime_etl(&mut s, dt);
    s
}

/// Length of the string produced by [`format_datetime_etl`].
const DATETIME_ETL_LEN: usize = 33;

/// Size of a Windows `SYSTEMTIME` struct.
pub const SYSTEMTIME_SZ: usize = 16;

/// Length of the string produced by [`push_systemtime`].
const SYSTEMTIME_LEN: usize = 23;

/// stack buffer large enough for any `i64`/`u64` in base 10
const NUMTOA_BUF_SZ: usize = 22;

const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Append `dt` in the [`format_datetime_etl`] form without allocating.
fn push_datetime_etl(
    out: &mut String,
    dt: &DateTimeL,
) {
    let offset_secs: i32 = dt.offset().local_minus_utc();
    let (sign, offset_abs) = if offset_secs < 0 { ('-', -offset_secs) } else { ('+', offset_secs) };
    _ = write!(
        out,
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:07}{}{:02}:{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.nanosecond() / 100,
        sign,
        offset_abs / 3600,
        (offset_abs % 3600) / 60,
    );
}

/// Append a `SYSTEMTIME` as `YYYY-MM-DDTHH:MM:SS.mmm` without allocating.
/// `wDayOfWeek` (offset 4) is skipped.
fn push_systemtime(
    out: &mut String,
    b: &[u8; SYSTEMTIME_SZ],
) {
    let w = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    _ = write!(
        out,
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        w(0),
        w(2),
        w(6),
        w(8),
        w(10),
        w(12),
        w(14)
    );
}

/// Append a SID in SDDL form (`S-1-5-…`) without allocating.
fn push_sid(
    out: &mut String,
    sid: &Sid,
) {
    out.push_str("S-");
    push_u64(out, sid.revision as u64);
    out.push('-');
    push_u64(out, sid.authority());
    for sub in sid.sub_authorities() {
        out.push('-');
        push_u64(out, *sub as u64);
    }
}

/// Append `n` in decimal without allocating.
#[inline]
fn push_u64(
    out: &mut String,
    n: u64,
) {
    let mut buf = [0u8; NUMTOA_BUF_SZ];
    out.push_str(n.numtoa_str(10, &mut buf));
}

/// Append `n` in decimal without allocating.
#[inline]
fn push_i64(
    out: &mut String,
    n: i64,
) {
    let mut buf = [0u8; NUMTOA_BUF_SZ];
    out.push_str(n.numtoa_str(10, &mut buf));
}

// -----
// GUID

/// A Windows GUID stored in the on-disk (mixed-endian) layout.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Guid([u8; 16]);

impl Guid {
    pub const NIL: Guid = Guid([0; 16]);

    /// Create from the 16 bytes as stored in an ETL file.
    pub const fn from_le_bytes(bytes: [u8; 16]) -> Guid {
        Guid(bytes)
    }

    /// Create from a slice of at least 16 bytes.
    pub fn from_le_slice(bytes: &[u8]) -> Option<Guid> {
        let arr: [u8; 16] = bytes
            .get(..16)?
            .try_into()
            .ok()?;

        Some(Guid(arr))
    }

    pub const fn is_nil(&self) -> bool {
        let mut i = 0;
        while i < 16 {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }

        true
    }

    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for Guid {
    /// Standard `{8-4-4-4-12}` lowercase form.
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        let b = &self.0;
        write!(
            f,
            "{{{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
            b[3], b[2], b[1], b[0], b[5], b[4], b[7], b[6], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
        )
    }
}

impl fmt::Debug for Guid {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "Guid({})", self)
    }
}

// ----
// SID

/// `SID_MAX_SUB_AUTHORITIES`
pub const SID_SUB_AUTHORITIES_MAX: usize = 15;

/// A Windows security identifier in its on-disk form; fixed-size so no
/// allocation is needed to hold it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sid {
    pub revision: u8,
    pub sub_authority_count: u8,
    /// 48-bit big-endian `SID_IDENTIFIER_AUTHORITY`
    pub identifier_authority: [u8; 6],
    /// only the first `sub_authority_count` entries are meaningful
    pub sub_authorities: [u32; SID_SUB_AUTHORITIES_MAX],
}

impl Sid {
    /// On-disk size in bytes of a SID with `count` sub-authorities.
    pub const fn size_for(count: u8) -> usize {
        8 + 4 * count as usize
    }

    pub fn authority(&self) -> u64 {
        self.identifier_authority
            .iter()
            .fold(0u64, |acc, b| (acc << 8) | *b as u64)
    }

    pub fn sub_authorities(&self) -> &[u32] {
        &self.sub_authorities[..(self.sub_authority_count as usize).min(SID_SUB_AUTHORITIES_MAX)]
    }
}

impl fmt::Display for Sid {
    /// SDDL form, e.g. `S-1-5-18`.
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "S-{}-{}", self.revision, self.authority())?;
        for sub in self.sub_authorities() {
            write!(f, "-{}", sub)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Sid {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "Sid({})", self)
    }
}

// ------
// names

/// A field, event, or provider name. Either a compile-time constant or a
/// string shared (via `Arc`) with a per-file cache, so no allocation happens
/// per event.
#[derive(Clone, Debug)]
pub enum EtlName {
    Static(&'static str),
    Shared(Arc<str>),
}

impl Deref for EtlName {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            EtlName::Static(s) => s,
            EtlName::Shared(s) => s,
        }
    }
}

impl PartialEq for EtlName {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        **self == **other
    }
}

impl Eq for EtlName {}

impl fmt::Display for EtlName {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.write_str(self)
    }
}

impl From<&'static str> for EtlName {
    fn from(s: &'static str) -> Self {
        EtlName::Static(s)
    }
}

impl From<Arc<str>> for EtlName {
    fn from(s: Arc<str>) -> Self {
        EtlName::Shared(s)
    }
}

impl From<String> for EtlName {
    fn from(s: String) -> Self {
        EtlName::Shared(Arc::from(s))
    }
}

// -------------
// Event values

/// A decoded value of one event payload field.
#[derive(Clone, Debug, PartialEq)]
pub enum EtlValue {
    Null,
    Str(String),
    I64(i64),
    U64(u64),
    Hex32(u32),
    Hex64(u64),
    Bool(bool),
    F64(f64),
    Guid(Guid),
    FileTime(FileTime),
    /// raw `SYSTEMTIME` struct (eight little-endian `u16`s)
    SystemTime([u8; SYSTEMTIME_SZ]),
    Bytes(Vec<u8>),
    /// boxed: a `Sid` is 68 bytes and rare; keeps `EtlValue` small
    Sid(Box<Sid>),
    Array(Vec<EtlValue>),
    Struct(Vec<(EtlName, EtlValue)>),
}

// every field pair is moved several times per event; keep the value small
const _: () = assert!(std::mem::size_of::<EtlValue>() <= 32);

/// Named fields of a decoded event payload.
pub type EtlFields = Vec<(EtlName, EtlValue)>;

/// Which decoder produced the payload; for `--summary` statistics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EtlDecoder {
    /// self-describing TraceLogging metadata carried in the event
    TraceLogging,
    /// hard-coded kernel `MSNT_SystemTrace` decoders
    Kernel,
    /// no decoder available; payload is raw bytes
    None,
}

/// The decoded payload of an event.
#[derive(Clone, Debug, PartialEq)]
pub enum EtlPayload {
    Fields(EtlFields),
    /// decoded fields followed by bytes that could not be decoded
    Partial(EtlFields, Vec<u8>),
    Raw(Vec<u8>),
    Empty,
}

/// On-disk header type the Event record used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EtlHeaderKind {
    /// `SYSTEM_TRACE_HEADER`; kernel events
    System,
    /// `COMPACT_TRACE_HEADER`; kernel events
    Compact,
    /// `PERFINFO_TRACE_HEADER`; kernel events
    PerfInfo,
    /// `EVENT_HEADER`; modern manifest-based and TraceLogging events
    Event,
    /// `EVENT_TRACE_HEADER`; classic (MOF) provider events
    FullHeader,
    /// `EVENT_INSTANCE_GUID_HEADER`
    Instance,
    /// `MESSAGE_TRACE_HEADER`; WPP software tracing
    Message,
    /// unknown or error header
    Error,
}

impl EtlHeaderKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            EtlHeaderKind::System => "System",
            EtlHeaderKind::Compact => "Compact",
            EtlHeaderKind::PerfInfo => "PerfInfo",
            EtlHeaderKind::Event => "Event",
            EtlHeaderKind::FullHeader => "FullHeader",
            EtlHeaderKind::Instance => "Instance",
            EtlHeaderKind::Message => "Message",
            EtlHeaderKind::Error => "Error",
        }
    }
}

/// The fields common to every Event; taken from the record header.
#[derive(Clone, Debug, PartialEq)]
pub struct EtlEnvelope {
    pub kind: EtlHeaderKind,
    pub filetime: FileTime,
    pub provider_guid: Guid,
    pub provider_name: Option<EtlName>,
    pub event_name: Option<EtlName>,
    pub event_id: u16,
    pub version: u8,
    pub level: u8,
    pub opcode: u8,
    pub task: u16,
    pub keywords: u64,
    pub pid: Option<u32>,
    pub tid: Option<u32>,
    pub activity_id: Option<Guid>,
    pub related_activity_id: Option<Guid>,
    /// kernel events; `(group << 8) | opcode`
    pub hook_id: Option<u16>,
    /// WPP events
    pub message_number: Option<u16>,
}

impl Default for EtlEnvelope {
    fn default() -> Self {
        EtlEnvelope {
            kind: EtlHeaderKind::Error,
            filetime: 0,
            provider_guid: Guid::NIL,
            provider_name: None,
            event_name: None,
            event_id: 0,
            version: 0,
            level: 0,
            opcode: 0,
            task: 0,
            keywords: 0,
            pid: None,
            tid: None,
            activity_id: None,
            related_activity_id: None,
            hook_id: None,
            message_number: None,
        }
    }
}

/// A fully parsed event; produced by the `EtlParser`.
#[derive(Clone, Debug, PartialEq)]
pub struct EtlEvent {
    pub envelope: EtlEnvelope,
    pub payload: EtlPayload,
    pub decoder: EtlDecoder,
}

/// Append `s` to `out` as a double-quoted string with C-style escapes.
fn push_quoted(
    out: &mut String,
    s: &str,
) {
    const fn needs_escape(c: char) -> bool {
        matches!(c, '"' | '\\' | '\u{7f}') || (c as u32) < 0x20
    }

    out.push('"');
    let mut rest: &str = s;
    while let Some(i) = rest.find(needs_escape) {
        out.push_str(&rest[..i]);
        // `find` matched a char at `i` so `rest[i..]` is non-empty
        let c: char = rest[i..].chars().next().unwrap_or_default();
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => {
                _ = write!(out, "\\x{:02x}", c as u32);
            }
        }
        rest = &rest[i + c.len_utf8()..];
    }
    out.push_str(rest);
    out.push('"');
}

fn push_hex(
    out: &mut String,
    bytes: &[u8],
) {
    out.reserve(2 + bytes.len() * 2);
    out.push_str("0x");
    for b in bytes {
        out.push(HEX_LOWER[(b >> 4) as usize] as char);
        out.push(HEX_LOWER[(b & 0x0F) as usize] as char);
    }
}

fn push_value(
    out: &mut String,
    value: &EtlValue,
    fixed_offset: &FixedOffset,
) {
    match value {
        EtlValue::Null => out.push_str("null"),
        EtlValue::Str(s) => push_quoted(out, s),
        EtlValue::I64(v) => push_i64(out, *v),
        EtlValue::U64(v) => push_u64(out, *v),
        EtlValue::Hex32(v) => {
            _ = write!(out, "0x{:X}", v);
        }
        EtlValue::Hex64(v) => {
            _ = write!(out, "0x{:X}", v);
        }
        EtlValue::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
        EtlValue::F64(v) => {
            _ = write!(out, "{}", v);
        }
        EtlValue::Guid(g) => {
            _ = write!(out, "{}", g);
        }
        EtlValue::FileTime(ft) => match filetime_to_datetimel(*ft, fixed_offset) {
            Some(dt) => push_datetime_etl(out, &dt),
            None => push_u64(out, *ft),
        },
        EtlValue::SystemTime(b) => push_systemtime(out, b),
        EtlValue::Bytes(b) => push_hex(out, b),
        EtlValue::Sid(sid) => push_sid(out, sid),
        EtlValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i != 0 {
                    out.push_str(", ");
                }
                push_value(out, item, fixed_offset);
            }
            out.push(']');
        }
        EtlValue::Struct(fields) => {
            out.push('{');
            for (i, (name, item)) in fields.iter().enumerate() {
                if i != 0 {
                    out.push_str(", ");
                }
                out.push_str(name);
                out.push('=');
                push_value(out, item, fixed_offset);
            }
            out.push('}');
        }
    }
}

/// Stack-allocated chain of enclosing struct names for flattened field names.
struct Prefix<'a> {
    name: &'a str,
    parent: Option<&'a Prefix<'a>>,
}

/// Append `outer.inner.` for each ancestor in `prefix`, outermost first.
fn push_prefix(
    out: &mut String,
    prefix: Option<&Prefix>,
) {
    if let Some(p) = prefix {
        push_prefix(out, p.parent);
        out.push_str(p.name);
        out.push('.');
    }
}

/// Append ` name=value` pairs; top-level structs are flattened to
/// `name.subname=value`.
fn push_fields(
    out: &mut String,
    prefix: Option<&Prefix>,
    fields: &EtlFields,
    fixed_offset: &FixedOffset,
) {
    for (name, value) in fields.iter() {
        match value {
            EtlValue::Struct(sub) => {
                let prefix_ = Prefix { name, parent: prefix };
                push_fields(out, Some(&prefix_), sub, fixed_offset);
            }
            _ => {
                out.push(' ');
                push_prefix(out, prefix);
                out.push_str(name);
                out.push('=');
                push_value(out, value, fixed_offset);
            }
        }
    }
}

/// Rough upper bound of a field's rendered length, for `String::with_capacity`.
fn fields_capacity(fields: &EtlFields) -> usize {
    fields
        .iter()
        .map(|(name, value)| {
            name.len()
                + 2
                + match value {
                    EtlValue::Str(s) => s.len() + 2,
                    EtlValue::Sid(sid) => 8 + 11 * sid.sub_authority_count as usize,
                    EtlValue::SystemTime(_) => SYSTEMTIME_LEN,
                    EtlValue::Bytes(b) => 2 + b.len() * 2,
                    EtlValue::Guid(_) => 38,
                    EtlValue::FileTime(_) => DATETIME_ETL_LEN,
                    EtlValue::Struct(sub) => fields_capacity(sub),
                    EtlValue::Array(items) => items.len() * 12,
                    _ => 20,
                }
        })
        .sum()
}

impl EtlEvent {
    /// Estimated rendered length; avoids reallocation during [`EtlEvent::render`].
    fn render_capacity(&self) -> usize {
        let env: &EtlEnvelope = &self.envelope;
        // datetime, GUID, and the fixed `EventId=… Keywords=` group plus optional IDs
        let mut n: usize = DATETIME_ETL_LEN + 48 + 96 + 64;
        if let Some(s) = &env.provider_name {
            n += s.len() + 16;
        }
        if let Some(s) = &env.event_name {
            n += s.len() + 13;
        }
        n += match &self.payload {
            EtlPayload::Empty => 0,
            EtlPayload::Fields(fields) => fields_capacity(fields),
            EtlPayload::Partial(fields, rest) => fields_capacity(fields) + 12 + rest.len() * 2,
            EtlPayload::Raw(bytes) => 12 + bytes.len() * 2,
        };

        n
    }

    /// Render as a single line of text (with trailing newline) and return the
    /// byte offsets of the datetime substring.
    ///
    /// Returns `None` if the timestamp cannot be represented.
    pub fn render(
        &self,
        fixed_offset: &FixedOffset,
    ) -> Option<(DateTimeL, Bytes, DtBegEndPair)> {
        let dt: DateTimeL = filetime_to_datetimel(self.envelope.filetime, fixed_offset)?;
        let env: &EtlEnvelope = &self.envelope;
        let mut out: String = String::with_capacity(self.render_capacity());
        push_datetime_etl(&mut out, &dt);
        let dt_beg_end: DtBegEndPair = (0, out.len());

        _ = write!(out, " Provider={}", env.provider_guid);
        if let Some(name) = &env.provider_name {
            out.push_str(" ProviderName=");
            push_quoted(&mut out, name);
        }
        if let Some(name) = &env.event_name {
            out.push_str(" EventName=");
            push_quoted(&mut out, name);
        }
        _ = write!(
            out,
            " EventId={} Version={} Level={} Opcode={} Task={} Keywords=0x{:X}",
            env.event_id, env.version, env.level, env.opcode, env.task, env.keywords,
        );
        if let Some(pid) = env.pid {
            out.push_str(" PID=");
            push_u64(&mut out, pid as u64);
        }
        if let Some(tid) = env.tid {
            out.push_str(" TID=");
            push_u64(&mut out, tid as u64);
        }
        if let Some(aid) = &env.activity_id
            && !aid.is_nil()
        {
            _ = write!(out, " ActivityId={}", aid);
        }
        if let Some(rid) = &env.related_activity_id
            && !rid.is_nil()
        {
            _ = write!(out, " RelatedActivityId={}", rid);
        }
        if let Some(hook) = env.hook_id {
            _ = write!(out, " HookId=0x{:04X}", hook);
        }
        if let Some(msg) = env.message_number {
            out.push_str(" MessageId=");
            push_u64(&mut out, msg as u64);
        }
        match &self.payload {
            EtlPayload::Empty => {}
            EtlPayload::Fields(fields) => push_fields(&mut out, None, fields, fixed_offset),
            EtlPayload::Partial(fields, rest) => {
                push_fields(&mut out, None, fields, fixed_offset);
                out.push_str(" Payload=");
                if rest.is_empty() {
                    out.push_str("(empty)");
                } else {
                    push_hex(&mut out, rest);
                }
            }
            EtlPayload::Raw(bytes) => {
                out.push_str(" Payload=");
                if bytes.is_empty() {
                    out.push_str("(empty)");
                } else {
                    push_hex(&mut out, bytes);
                }
            }
        }
        out.push(NLc);

        Some((dt, out.into_bytes(), dt_beg_end))
    }
}

// ----
// Etl

/// A rendered ETL event ready for printing; the ETL counterpart of
/// [`Evtx`].
///
/// [`Evtx`]: crate::data::evtx::Evtx
#[derive(Clone, PartialEq, Eq)]
pub struct Etl {
    /// The derived `DateTime` instance.
    dt: DateTimeL,
    /// The byte offsets of the datetime substring within `data`.
    dt_beg_end: DtBegEndPairOpt,
    /// The rendered event text.
    data: Bytes,
}

impl PartialOrd for Etl {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(
            self.dt
                .cmp(&other.dt)
                .then_with(|| self.data.cmp(&other.data)),
        )
    }
}

impl Hash for Etl {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
        self.data.hash(state);
        self.dt.hash(state);
    }
}

impl fmt::Debug for Etl {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Etl")
            .field("dt", &self.dt)
            .field("(beg, end)", &self.dt_beg_end)
            .field("bytes", &self.data.len())
            .finish()
    }
}

impl Etl {
    pub const fn new(
        dt: DateTimeL,
        dt_beg_end: DtBegEndPairOpt,
        data: Bytes,
    ) -> Etl {
        Etl { dt, dt_beg_end, data }
    }

    /// Length of this `Etl` in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Clippy recommends `fn is_empty` since there is a `len()`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub const fn dt(&self) -> &DateTimeL {
        &self.dt
    }

    pub const fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// `Etl` to `String`.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_raw(&self) -> String {
        buffer_to_string_noraw(self.as_bytes())
    }

    /// `Etl` to `String` but using printable chars for
    /// non-printable and/or formatting characters.
    #[doc(hidden)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_string_noraw(&self) -> String {
        String::from_utf8_lossy(self.as_bytes()).into_owned()
    }
}
