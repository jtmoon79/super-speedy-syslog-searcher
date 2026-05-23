// src/data/evtx.rs

//! Implement [`Evtx`] for [`EvtxRecord`].
//!
//! [`EvtxRecord`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxRecord.html

use std::fmt;
use std::hash::Hash;
use std::io::{
    Error,
    ErrorKind,
};

pub(crate) use ::evtx::{
    err::EvtxError,
    SerializedEvtxRecord,
    Timestamp,
};
#[allow(unused_imports)]
use ::more_asserts::{
    assert_ge,
    assert_gt,
    assert_le,
    assert_lt,
    debug_assert_ge,
    debug_assert_gt,
    debug_assert_le,
    debug_assert_lt,
};
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
};

#[doc(hidden)]
use crate::common::{
    debug_panic,
    NLc,
};
use crate::de_err;
use crate::data::common::DtBegEndPairOpt;
use crate::data::datetime::{
    DateTime,
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    Utc,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::buffer_to_String_noraw;

pub type TimestampOpt = Option<Timestamp>;

/// From private `evtx::evtx_record::RecordId`.
///
/// See <https://github.com/omerbenamram/evtx/issues/234>
pub type RecordId = u64;

/// [`SerializedEvtxRecord`] with [`String`] as the data type.
///
/// [`SerializedEvtxRecord`]: https://docs.rs/evtx/0.8.1/evtx/struct.SerializedEvtxRecord.html
pub type EvtxRS = SerializedEvtxRecord<String>;
/// [`Result`] of [`EvtxRS`].
///
/// [`Result`]: std::result::Result
pub type ResultEvtxRS = std::result::Result<EvtxRS, EvtxError>;

const TIMECREATED_BEG_SUBSTR: &str = "<TimeCreated SystemTime=\"";
const TIMECREATED_END_SUBCHAR: char = '\"';

/// Convert a `evtx` "timestamp" (`jiff::Timestamp`)
/// to a `s4` "datetime" (`chrono::DateTime<Local>`).
pub fn timestamp_to_datetimelopt(
    timestamp: &Timestamp,
    fixed_offset: &FixedOffset,
) -> DateTimeLOpt {
    let ns128: i128 = timestamp.as_nanosecond();
    let ns64: i64 = match i64::try_from(ns128) {
        Ok(ns) => ns,
        Err(_err) => {
            debug_panic!("timestamp.as_nanosecond() value {:?} does not fit in i64: {}", ns128, _err);
            return DateTimeLOpt::None;
        }
    };
    let dt_utc: DateTime::<Utc> = DateTime::<Utc>::from_timestamp_nanos(ns64);
    let dt_fo: DateTime::<FixedOffset> = dt_utc.with_timezone(fixed_offset);

    DateTimeLOpt::Some(dt_fo)
}

/// Convert a `s4` "datetime" (`DateTimeL`)
/// to a `evtx` "timestamp" (`jiff::Timestamp`).
pub fn datetimelopt_to_timestampopt(
    datetimeopt: &DateTimeLOpt,
) -> TimestampOpt {
    let dt: &DateTimeL = match datetimeopt {
        DateTimeLOpt::None => return TimestampOpt::None,
        DateTimeLOpt::Some(dt) => dt,
    };
    let ns: i64 = match dt.timestamp_nanos_opt() {
        Some(ns) => ns,
        None => {
            de_err!("datetime.timestamp_nanos_opt() returned None for datetime {:?}", dt);
            return TimestampOpt::None;
        }
    };

    match Timestamp::from_nanosecond(ns as i128) {
        Ok(ts) => TimestampOpt::Some(ts),
        Err(_err) => {
            de_err!("Timestamp::from_nanosecond({:?}) returned Err {}", ns, _err);
            TimestampOpt::None
        }
    }
}

/// A `Evtx` holds information taken from an [`EvtxRecord`], a
/// [Windows Event Log] record.
///
/// Here is an example EVTX Event written by crate [`evtx`] as XML:
///
/// ```lang-xml
/// <?xml version="1.0" encoding="utf-8"?>
/// <Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
///   <System>
///     <Provider Name="OpenSSH" Guid="C4BB5D35-0136-5BC3-A262-37EF24EF9802">
///     </Provider>
///     <EventID>2</EventID>
///     <Version>0</Version>
///     <Level>2</Level>
///     <Task>0</Task>
///     <Opcode>0</Opcode>
///     <Keywords>0x8000000000000000</Keywords>
///     <TimeCreated SystemTime="2023-03-16T20:20:23.130640Z">
///     </TimeCreated>
///     <EventRecordID>3</EventRecordID>
///     <Correlation>
///     </Correlation>
///     <Execution ProcessID="25223" ThreadID="30126">
///     </Execution>
///     <Channel>OpenSSH</Channel>
///     <Computer>host1</Computer>
///     <Security UserID="S-1-2-20">
///     </Security>
///   </System>
///   <EventData>
///     <Data Name="process">sshd.exe</Data>
///     <Data Name="payload">error: kex_exchange_identification: Connection closed by remote host</Data>
///   </EventData>
/// </Event>
/// ```
///
/// [`EvtxRecord`]: https://docs.rs/evtx/0.8.1/evtx/struct.EvtxRecord.html
/// [`evtx`]: https://docs.rs/evtx/0.8.1/evtx/index.html
/// [Windows Event Log]: https://learn.microsoft.com/en-us/windows/win32/wes/windows-event-log
#[derive(Clone, PartialEq, Eq)]
pub struct Evtx {
    /// The derived `DateTime` instance.
    dt: DateTimeL,
    /// The byte offsets of the substring demarcating the embedded `DateTime`
    /// within `record.data`.
    dt_beg_end: DtBegEndPairOpt,
    /// The original `EvtxRS` record.
    record: EvtxRS,
}

impl PartialOrd for Evtx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dt.partial_cmp(&other.dt).or_else(|| self.id().partial_cmp(&other.id()))
    }
}

impl Hash for Evtx {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.record.event_record_id.hash(state);
        self.record.data.hash(state);
        self.dt.hash(state);
    }
}

impl fmt::Debug for Evtx {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Evtx Record")
            .field("ID", &self.id())
            .field("dt", &self.dt)
            .field("(beg, end)", &self.dt_beg_end)
            .finish()
    }
}

impl Evtx {
    /// Create a new `Evtx`.
    pub fn from_resultserializedrecord(
        record: ResultEvtxRS,
        fixed_offset: &FixedOffset,
    ) -> Result<Evtx, Error> {
        match record {
            Ok(record) => {
                Result::Ok(
                    Self::from_evtxrs(
                        DateTimeLOpt::None,
                        fixed_offset,
                        record,
                    )
                )
            }
            Err (err) => {
                Err(
                    Error::new(
                        ErrorKind::Other,
                        format!("EvtxError: {}", err)
                    )
                )
            }
        }
    }

    /// Create a new `Evtx`.
    pub fn from_evtxrs(
        datetime_opt: DateTimeLOpt,
        fixed_offset: &FixedOffset,
        record: EvtxRS,
    ) -> Evtx {
        let dt: DateTimeL = match datetime_opt {
            Some(dt) => dt,
            None => match timestamp_to_datetimelopt(
                        &record.timestamp,
                        fixed_offset,
                    ) {
                        DateTimeLOpt::None => {
                            debug_panic!("timestamp_to_datetimelopt returned None for timestamp {:?}", record.timestamp);
                            DateTime::<Utc>::from_timestamp_nanos(0).with_timezone(fixed_offset)
                        }
                        DateTimeLOpt::Some(dt) => dt,
                    }
        };
        let dt_beg_end = Self::get_dt_beg_end(&record.data);
        Evtx {
            dt,
            dt_beg_end,
            record,
        }
    }

    /// get byte offsets, beginning and end, of the substring demarcarting the
    /// embedded `DateTime`, e.g. given
    ///
    /// ```lang-text
    ///    <TimeCreated SystemTime="2023-03-16T20:20:23.130640Z">
    /// ```
    ///
    /// would return byte offset of the first `'2'` and the closing `'"'`.
    ///
    /// Returns `None` if the substring is not found.
    pub(crate) fn get_dt_beg_end(
        data: &str,
    ) -> DtBegEndPairOpt {
        let dt_beg: usize = match data.find(TIMECREATED_BEG_SUBSTR) {
            Some(dt_beg) => dt_beg + TIMECREATED_BEG_SUBSTR.len(),
            None => { return None; },
        };
        let dt_end: usize = match data[dt_beg..].find(TIMECREATED_END_SUBCHAR) {
            Some(dt_end) => dt_beg + dt_end,
            None => { return None; },
        };
        Some((dt_beg, dt_end))
    }

    /// Length of this `Evtx` in bytes.
    pub const fn len(self: &Evtx) -> usize {
        self.as_bytes().len()
    }

    /// Clippy recommends `fn is_empty` since there is a `len()`.
    pub const fn is_empty(self: &Evtx) -> bool {
        self.len() == 0
    }

    pub const fn id(self: &Evtx) -> RecordId {
        self.record.event_record_id
    }

    /// Return a reference to [`self.dt`] (`DateTimeL`).
    ///
    /// [`self.dt`]: Evtx::dt
    pub const fn dt(&self) -> &DateTimeL {
        &self.dt
    }

    pub const fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.record.data.as_bytes()
    }

    /// Does this `Evtx` end in a newline character?
    ///
    /// By default, "yes", but it's nice to provide this.
    pub fn ends_with_newline(self: &Evtx) -> bool {
        let byte_last = match self.record.data.as_bytes().last() {
            Some(byte_) => byte_,
            None => {
                return false;
            }
        };
        match char::try_from(*byte_last) {
            Ok(char_) => NLc == char_,
            Err(_err) => false,
        }
    }

    /// Create a `String` from `self.data` bytes.
    ///
    /// `raw` is `true` means use byte characters as-is.
    /// `raw` is `false` means replace formatting characters or non-printable
    /// characters with pictoral representation (i.e. use
    /// [`byte_to_char_noraw`]).
    ///
    /// XXX: very inefficient and not always correct! *only* intended to help
    ///      humans visually inspect stderr output.
    ///
    /// [`byte_to_char_noraw`]: crate::debug::printers::byte_to_char_noraw
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    fn impl_to_String_raw(
        &self,
        raw: bool,
    ) -> String {
        match raw {
            true => buffer_to_String_noraw(self.as_bytes()),
            false => self.record.data.clone(),
        }
    }

    /// `Evtx` to `String`.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_raw(&self) -> String {
        self.impl_to_String_raw(true)
    }

    /// `Evtx` to `String` but using printable chars for
    /// non-printable and/or formatting characters.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_noraw(&self) -> String {
        self.impl_to_String_raw(false)
    }

    /// for testing only
    #[cfg(test)]
    pub(crate) const fn new_forced(
        dt: DateTimeL,
        dt_beg_end: DtBegEndPairOpt,
        record: EvtxRS,
    ) -> Self {
        Self {
            dt,
            dt_beg_end,
            record,
        }
    }
}
