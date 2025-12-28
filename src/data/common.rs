// src/data/common.rs

//! Common types and constants for `readers`.

use crate::data::datetime::DateTimeL;
use crate::data::evtx::Evtx;
use crate::data::fixedstruct::FixedStruct;
use crate::data::journal::JournalEntry;
use crate::data::sysline::SyslineP;

/// The type of log message sent from file processing thread to the main
/// printing thread enclosing the specific message.
#[derive(Debug)]
pub enum LogMessage {
    Sysline(SyslineP),
    FixedStruct(FixedStruct),
    Evtx(Evtx),
    Journal(JournalEntry),
}
pub type LogMessageOpt = Option<LogMessage>;

impl LogMessage {
    /// Returns the datetime of the log message.
    pub fn dt(&self) -> &DateTimeL {
        match self {
            LogMessage::Sysline(sysline) => sysline.dt(),
            LogMessage::FixedStruct(fixedstruct) => fixedstruct.dt(),
            LogMessage::Evtx(evtx) => evtx.dt(),
            LogMessage::Journal(journal) => journal.dt(),
        }
    }
}

/// Bytes offsets of the beginning and end of the
/// datetime substring within a `String`.
// TODO: change to a typed `struct DtBegEndPair(usize, usize)`
pub type DtBegEndPair = (usize, usize);

/// [`Option`] of [`DtBegEndPair`].
pub type DtBegEndPairOpt = Option<DtBegEndPair>;
