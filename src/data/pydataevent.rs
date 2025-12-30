// src/data/etl.rs

//! Data representation of `.etl` events.

use std::fmt;

use ::more_asserts::debug_assert_le;

#[allow(unused_imports)]
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
pub use crate::data::common::DtBegEndPairOpt;
use crate::data::datetime::DateTimeL;

pub type EventBytes = Bytes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EtlParserUsed {
    /// Use the Python project `dissect.etl` to parse the ETL file
    DissectEtl,
    /// Use the Python project `etl-parser` to parse the ETL file
    EtlParser,
}

/// Data representing a single `.etl` or `.odl` file log message.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PyDataEvent {
    /// Data extracted from the event.
    data: EventBytes,
    /// The derived `DateTime` instance.
    dt: DateTimeL,
    /// Indexes of start and end of datetime substring within `data`.
    dt_beg_end: DtBegEndPairOpt,
}

impl fmt::Debug for PyDataEvent {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("PyDataEvent")
            .field("bytes", &self.data.len())
            .field("dt", &self.dt)
            .field("dt_beg_end", &self.dt_beg_end)
            .finish()
    }
}

impl PyDataEvent {
    
    pub fn new(
        data: EventBytes,
        dt: DateTimeL,
        dt_beg_end: DtBegEndPairOpt,
    ) -> Self {
        def1ñ!();
        debug_assert_le!(
            &dt_beg_end.unwrap_or_default().0,
            &dt_beg_end.unwrap_or_default().1,
            "dt_beg_end is not in ascending order",
        );

        PyDataEvent {
            data,
            dt,
            dt_beg_end,
        }
    }

    // TODO: 2025/11 *almost* stable as const
    //       `Vec::<T, A>::len` is not yet stable as a const fn
    pub fn len(&self) -> usize {
        self.data.len()
    }

    // XXX: clippy recommends `fn is_empty` since there is a `len()`.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub const fn dt(&self) -> &DateTimeL {
        &self.dt
    }

    /// The ETL event as a `&[u8]`.
    // TODO: 2025/11 *almost* stable as const
    //       `Vec::<T, A>::as_slice` is not yet stable as a const fn
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// The substring byte offsets of the datetime stamp within the log message.
    pub const fn dt_beg_end(&self) -> &DtBegEndPairOpt {
        &self.dt_beg_end
    }

}
