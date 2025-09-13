// src/data/journal.rs

//! Data representation of `.journal` file log message entries.

use std::fmt;
use std::time::Duration as StdDuration;

use ::bstr::ByteSlice; // attaches `find` to `&[u8]`
use ::more_asserts::debug_assert_le;
use ::si_trace_print::{
    def1n,
    def1x,
    def1ñ,
    defñ,
};

#[doc(hidden)]
pub use crate::common::{
    Bytes,
    CharSz,
    Count,
    FPath,
    FileOffset,
    NLc,
    NLs,
    NLu8,
    ResultFind,
};
use crate::data::common::DtBegEndPairOpt;
use crate::data::datetime::{
    DateTime,
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    SystemTime,
    Utc,
};

/*

    $ PAGER= journalctl --lines=1 --output=export --all --utc --file=./user-1000.journal
    __CURSOR=s=e992f143877046059b264a0f907056b6;i=6ff;b=26d74a46deff4872be6d4ca6e885a198;m=46c65ea;t=5f840a88a4b39;x=e7933c3b47482d45
    __REALTIME_TIMESTAMP=1680331472784185
    __MONOTONIC_TIMESTAMP=74212842
    _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
    _TRANSPORT=journal
    _UID=1000
    _GID=1000
    _CAP_EFFECTIVE=0
    _SELINUX_CONTEXT

    unconfined

    _AUDIT_SESSION=2
    _AUDIT_LOGINUID=1000
    _SYSTEMD_OWNER_UID=1000
    _SYSTEMD_UNIT=user@1000.service
    _SYSTEMD_SLICE=user-1000.slice
    _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
    _HOSTNAME=ubuntu22Acorn
    PRIORITY=4
    _SYSTEMD_USER_SLICE=session.slice
    _PID=1306
    _COMM=gnome-shell
    _EXE=/usr/bin/gnome-shell
    _CMDLINE=/usr/bin/gnome-shell
    _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@wayland.service
    _SYSTEMD_USER_UNIT=org.gnome.Shell@wayland.service
    _SYSTEMD_INVOCATION_ID=b7d368c96091463aa538006b518785f4
    GLIB_DOMAIN=Ubuntu AppIndicators
    SYSLOG_IDENTIFIER=ubuntu-appindicators@ubuntu.com
    CODE_FILE=/usr/share/gnome-shell/extensions/ubuntu-appindicators@ubuntu.com/appIndicator.js
    CODE_LINE=738
    CODE_FUNC=_setGicon
    MESSAGE=unable to update icon for livepatch
    _SOURCE_REALTIME_TIMESTAMP=1680331472788150

*/

pub const FIELD_MID: &str = "=";
pub const FIELD_MID_C: char = '=';
pub const FIELD_MID_U8: u8 = FIELD_MID_C as u8;
pub const FIELD_END: &str = "\n";
pub const FIELD_END_C: char = '\n';
pub const FIELD_END_U8: u8 = FIELD_END_C as u8;
pub const ENTRY_END: &str = "\n";
pub const ENTRY_END_C: char = '\n';
pub const ENTRY_END_U8: u8 = ENTRY_END_C as u8;

pub const KEY__REALTIME_TIMESTAMP: &str = "__REALTIME_TIMESTAMP";
pub const KEY__REALTIME_TIMESTAMP_BYTES: &[u8] = KEY__REALTIME_TIMESTAMP.as_bytes();
pub const KEY_SOURCE_REALTIME_TIMESTAMP: &str = "_SOURCE_REALTIME_TIMESTAMP";
pub const KEY_SOURCE_REALTIME_TIMESTAMP_BYTES: &[u8] = KEY_SOURCE_REALTIME_TIMESTAMP.as_bytes();

/// Microseconds since the Unix epoch.
/// The DateTime signifier from Journal field [`__REALTIME_TIMESTAMP`]
/// and returned by API function [`sd_journal_get_realtime_usec`].
///
/// [`__REALTIME_TIMESTAMP`]: https://www.man7.org/linux/man-pages/man7/systemd.journal-fields.7.html
/// [`sd_journal_get_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_get_realtime_usec.3.html
pub type EpochMicroseconds = u64;
pub type EpochMicrosecondsOpt = Option<EpochMicroseconds>;

/// The boot time microseconds offset from Journal field
/// [`__MONOTONIC_TIMESTAMP`].
///
/// [`__MONOTONIC_TIMESTAMP`]: https://www.man7.org/linux/man-pages/man7/systemd.journal-fields.7.html
pub type MonotonicMicroseconds = u64;
pub type MonotonicMicrosecondsOpt = Option<MonotonicMicroseconds>;

/// `journalctl` prefers to base the datetime on `_SOURCE_REALTIME_TIMESTAMP`
/// if it is present, otherwise it uses `__REALTIME_TIMESTAMP`.
/// This notes which was used to create the `DateTimeL` returned by
/// [`JournalEntry::dt()`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DtUsesSource {
    /// field `_SOURCE_REALTIME_TIMESTAMP` was used
    SourceRealtimeTimestamp,
    /// field `__REALTIME_TIMESTAMP` was used
    RealtimeTimestamp,
}

/// Compile-time override of default preference of datetime source.
/// See [Issue #101].
///
/// [Issue #101]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/101
pub const DT_USES_SOURCE_OVERRIDE: Option<DtUsesSource> =
    Some(DtUsesSource::RealtimeTimestamp);

/// Convert a `journal` field [`__REALTIME_TIMESTAMP`] or
/// field `_SOURCE_REALTIME_TIMESTAMP` value
/// to a `s4` "datetime" (`DateTimeL`).
///
/// [`__REALTIME_TIMESTAMP`]: https://www.man7.org/linux/man-pages/man7/systemd.journal-fields.7.html
pub fn realtime_timestamp_to_datetimel(
    fixedoffset: &FixedOffset,
    realtime_timestamp: &EpochMicroseconds,
) -> DateTimeL {
    let duration: StdDuration = StdDuration::from_micros(*realtime_timestamp);
    let st: SystemTime = SystemTime::UNIX_EPOCH + duration;
    let dtu: DateTime<Utc> = DateTime::<Utc>::from(st);
    let dtu = dtu.with_timezone(fixedoffset);
    defñ!("converted {:?} to {:?}", realtime_timestamp, dtu);

    dtu
}

/// Convert a `journal` value from field [`__REALTIME_TIMESTAMP`] or
/// field `_SOURCE_REALTIME_TIMESTAMP` to a `s4` "datetime" (`DateTimeL`).
///
/// [`__REALTIME_TIMESTAMP`]: https://www.man7.org/linux/man-pages/man7/systemd.journal-fields.7.html
pub fn realtime_or_source_realtime_timestamp_to_datetimel(
    fixed_offset: &FixedOffset,
    realtime_timestamp: &EpochMicroseconds,
    source_realtime_timestamp: &EpochMicrosecondsOpt,
) -> DateTimeL {
    let actual_epoch_microseconds = match DT_USES_SOURCE_OVERRIDE {
        Some(dt_uses_sources) => {
            match dt_uses_sources {
                DtUsesSource::SourceRealtimeTimestamp => {
                    match source_realtime_timestamp {
                        Some(source_realtime_timestamp) => source_realtime_timestamp,
                        None => realtime_timestamp,
                    }
                }
                DtUsesSource::RealtimeTimestamp => {
                    realtime_timestamp
                }
            }
        }
        None => {
            match source_realtime_timestamp {
                Some(source_realtime_timestamp) => source_realtime_timestamp,
                None => realtime_timestamp,
            }
        }
    };

    realtime_timestamp_to_datetimel(
        fixed_offset,
        actual_epoch_microseconds
    )
}

/// Convert `datetime` to an `EpochMicroseconds`.
pub fn datetimel_to_realtime_timestamp(
    datetime: &DateTimeL,
) -> EpochMicroseconds {
    datetime.timestamp_micros() as EpochMicroseconds
}

/// Convert `datetime_opt` to an `Option<EpochMicroseconds>`.
pub fn datetimelopt_to_realtime_timestamp_opt(
    datetime_opt: &DateTimeLOpt,
) -> EpochMicrosecondsOpt {
    match datetime_opt {
        None => None,
        Some(datetime) => {
            Some(datetimel_to_realtime_timestamp(datetime))
        }
    }
}

/// Data representing a single `.journal` file log message.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalEntry {
    /// Taken from field `__REALTIME_TIMESTAMP`.
    realtime_timestamp: EpochMicroseconds,
    /// Taken from field `_SOURCE_REALTIME_TIMESTAMP`
    source_realtime_timestamp: EpochMicrosecondsOpt,
    /// The derived `DateTime` instance.
    dt: DateTimeL,
    /// The derived `DateTime` instance is from this field.
    dt_uses_source: DtUsesSource,
    /// The data for printing.
    data: Vec<u8>,
    /// Indexes of start and end of datetime in `data`.
    dt_beg_end: DtBegEndPairOpt,
}

impl fmt::Debug for JournalEntry {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Journal Entry")
            .field("realtime_timestamp", &self.realtime_timestamp)
            .field("source_realtime_timestamp", &self.source_realtime_timestamp)
            .field("dt", &self.dt)
            .finish()
    }
}

impl JournalEntry {
    /// Determines `dt_uses_source` from passed `source_realtime_timestamp` and
    /// `realtime_timestamp`.
    pub fn new(
        data: Vec::<u8>,
        realtime_timestamp: EpochMicroseconds,
        source_realtime_timestamp: EpochMicrosecondsOpt,
        dt_uses_source: DtUsesSource,
        fixed_offset: &FixedOffset,
    ) -> Self {
        def1ñ!();
        let dt: DateTimeL = realtime_or_source_realtime_timestamp_to_datetimel(
            fixed_offset,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        let (dt_a, dt_b) = Self::find_timestamp_in_buffer(data.as_slice(), dt_uses_source);
        JournalEntry {
            realtime_timestamp,
            source_realtime_timestamp,
            dt,
            dt_uses_source,
            data,
            dt_beg_end: DtBegEndPairOpt::Some((dt_a, dt_b)),
        }
    }

    /// Caller has all the nnecessary data to create a `JournalEntry`.
    pub fn new_with_date(
        data: Vec::<u8>,
        realtime_timestamp: EpochMicroseconds,
        source_realtime_timestamp: EpochMicrosecondsOpt,
        dt: DateTimeL,
        dt_uses_source: DtUsesSource,
        dt_a: usize,
        dt_b: usize,
    ) -> Self {
        def1ñ!();

        #[cfg(any(debug_assertions, test))]
        {
            if DT_USES_SOURCE_OVERRIDE.is_none() {
                match dt_uses_source {
                    DtUsesSource::RealtimeTimestamp => {
                        assert!(source_realtime_timestamp.is_none());
                    }
                    DtUsesSource::SourceRealtimeTimestamp => {
                        assert!(source_realtime_timestamp.is_some());
                    }
                }
            }
        }

        JournalEntry {
            realtime_timestamp,
            source_realtime_timestamp,
            dt,
            dt_uses_source,
            data,
            dt_beg_end: DtBegEndPairOpt::Some((dt_a, dt_b)),
        }
    }

    /// Caller has all the nnecessary data to create a `JournalEntry`.
    pub fn from_buffer(
        data: &[u8],
        realtime_timestamp: EpochMicroseconds,
        source_realtime_timestamp: EpochMicrosecondsOpt,
        dt: DateTimeL,
        dt_uses_source: DtUsesSource,
        dt_a: usize,
        dt_b: usize,
    ) -> Self {
        def1ñ!();
        let data = Vec::from(data);

        Self::from_vec(
            data,
            realtime_timestamp,
            source_realtime_timestamp,
            dt,
            dt_uses_source,
            dt_a,
            dt_b,
        )
    }

    /// Caller has all the nnecessary data to create a `JournalEntry`.
    pub fn from_vec(
        data: Vec<u8>,
        realtime_timestamp: EpochMicroseconds,
        source_realtime_timestamp: EpochMicrosecondsOpt,
        dt: DateTimeL,
        dt_uses_source: DtUsesSource,
        dt_a: usize,
        dt_b: usize,
    ) -> Self {
        def1ñ!();
        Self::new_with_date(
            data,
            realtime_timestamp,
            source_realtime_timestamp,
            dt,
            dt_uses_source,
            dt_a,
            dt_b,
        )
    }

    /// Create a `JournalEntry`, create a `DateTime` instance from passed data.
    pub fn from_vec_nodt(
        data: Vec<u8>,
        realtime_timestamp: EpochMicroseconds,
        source_realtime_timestamp: EpochMicrosecondsOpt,
        dt_uses_source: DtUsesSource,
        fixedoffset: &FixedOffset,
    ) -> Self {
        def1ñ!();
        let dt = realtime_or_source_realtime_timestamp_to_datetimel(
            fixedoffset,
            &realtime_timestamp,
            &source_realtime_timestamp
        );
        let (dt_a, dt_b) = Self::find_timestamp_in_buffer(data.as_slice(), dt_uses_source);
        Self::from_vec(
            data,
            realtime_timestamp,
            source_realtime_timestamp,
            dt,
            dt_uses_source,
            dt_a,
            dt_b,
        )
    }

    /// Find the datetime stamp substring by searching for relevant key
    /// patterns.
    pub(crate) fn find_timestamp_in_buffer(buffer: &[u8], dt_uses_source: DtUsesSource) -> (usize, usize) {
        def1n!();
        let dt_a: usize;
        let dt_b: usize;
        // determine the start index and end index of the datetime in the buffer.
        // search for key `__REALTIME_TIMESTAMP`
        match dt_uses_source {
            DtUsesSource::RealtimeTimestamp => {
                match buffer.find(KEY__REALTIME_TIMESTAMP_BYTES) {
                    Some(idx) => {
                        dt_a = idx + KEY__REALTIME_TIMESTAMP_BYTES.len() + FIELD_MID.len();
                        dt_b = match buffer[dt_a..].find_byte(FIELD_END_U8) {
                            None => dt_a,
                            Some(idx) => dt_a + idx,
                        };
                    }
                    None => {
                        // could not find key `__REALTIME_TIMESTAMP`
                        // search for key `_SOURCE_REALTIME_TIMESTAMP`
                        match buffer.find(KEY_SOURCE_REALTIME_TIMESTAMP_BYTES) {
                            None => {
                                dt_a = 0;
                                dt_b = 0;
                            }
                            Some(idx) => {
                                dt_a = idx + KEY_SOURCE_REALTIME_TIMESTAMP_BYTES.len() + FIELD_MID.len();
                                dt_b = match buffer[dt_a..].find_byte(FIELD_END_U8) {
                                    None => dt_a,
                                    Some(idx) => dt_a + idx,
                                };
                            }
                        }
                    }
                }
            }
            DtUsesSource::SourceRealtimeTimestamp => {
                match buffer.find(KEY_SOURCE_REALTIME_TIMESTAMP_BYTES) {
                    Some(idx) => {
                        dt_a = idx + KEY_SOURCE_REALTIME_TIMESTAMP_BYTES.len() + FIELD_MID.len();
                        dt_b = match buffer[dt_a..].find_byte(FIELD_END_U8) {
                            // value ends at end of buffer
                            None => buffer.len(),
                            // value ends at end of line
                            Some(idx) => dt_a + idx,
                        };
                    }
                    None => {
                        // could not find key `_SOURCE_REALTIME_TIMESTAMP`
                        // search for key `__REALTIME_TIMESTAMP`
                        match buffer.find(KEY__REALTIME_TIMESTAMP_BYTES) {
                            None => {
                                // nothing found
                                dt_a = 0;
                                dt_b = 0;
                            }
                            Some(idx) => {
                                dt_a = idx + KEY__REALTIME_TIMESTAMP_BYTES.len() + FIELD_MID.len();
                                dt_b = match buffer[dt_a..].find_byte(FIELD_END_U8) {
                                    // value ends at end of buffer
                                    None => buffer.len(),
                                    // value ends at end of line
                                    Some(idx) => dt_a + idx,
                                };
                            }
                        }
                    }
                }
            }
        }
        debug_assert_le!(dt_a, dt_b, "unexpected dt_a {}, dt_b {}", dt_a, dt_b);
        def1x!("return ({}, {})", dt_a, dt_b);

        (dt_a, dt_b)
    }

    pub fn dt(&self) -> &DateTimeL {
        &self.dt
    }

    /// The log message as a `&[u8]`. The log message is created by a
    /// `JournalReader`.
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }

    /// The substring byte offsets of the datetime stamp within the log message
    /// (in this `JournalEntry`'s `data` field).
    pub const fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }
}
