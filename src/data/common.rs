// src/data/common.rs

//! Common types and constants for `readers`.

use crate::data::asl::Asl;
use crate::data::datetime::DateTimeL;
use crate::data::etl::Etl;
use crate::data::evtx::Evtx;
use crate::data::fixedstruct::FixedStruct;
use crate::data::journal::JournalEntry;
use crate::data::odl::Odl;
use crate::data::sysline::SyslineP;

/// The type of log message sent from file processing thread to the main
/// printing thread enclosing the specific message.
#[derive(Debug)]
pub enum LogMessage {
    // TODO: reorder in alphabetical order
    Sysline(SyslineP),
    FixedStruct(FixedStruct),
    Etl(Etl),
    Evtx(Evtx),
    Journal(JournalEntry),
    Odl(Odl),
    Asl(Asl),
}
pub type LogMessageOpt = Option<LogMessage>;

impl LogMessage {
    /// Returns the datetime of the log message.
    pub fn dt(&self) -> &DateTimeL {
        match self {
            // TODO: reorder in alphabetical order
            LogMessage::Sysline(sysline) => sysline.dt(),
            LogMessage::FixedStruct(fixedstruct) => fixedstruct.dt(),
            LogMessage::Etl(etl) => etl.dt(),
            LogMessage::Evtx(evtx) => evtx.dt(),
            LogMessage::Journal(journal) => journal.dt(),
            LogMessage::Odl(odl) => odl.dt(),
            LogMessage::Asl(asl) => asl.dt(),
        }
    }
}

/// Bytes offsets of the beginning and end of the
/// datetime substring within a `String`.
// TODO: change to a typed `struct DtBegEndPair(usize, usize)`
pub type DtBegEndPair = (usize, usize);

/// [`Option`] of [`DtBegEndPair`].
pub type DtBegEndPairOpt = Option<DtBegEndPair>;

/// Rendered native events that share the byte-oriented printing pipeline.
pub trait PrintableEvent {
    fn dt(&self) -> &DateTimeL;
    fn dt_beg_end(&self) -> &DtBegEndPairOpt;
    fn as_bytes(&self) -> &[u8];
}
