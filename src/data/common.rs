// src/data/common.rs

//! Common types and constants for `readers`.

use crate::data::datetime::DateTimeL;
use crate::data::evtx::Evtx;
use crate::data::sysline::SyslineP;
use crate::data::utmpx::Utmpx;
use crate::data::journal::JournalEntry;

/// The type of log message sent from file processing thread to the main
/// printing thread enclosing the specific message.
#[derive(Debug)]
pub enum LogMessage {
    Sysline(SyslineP),
    Utmpx(Utmpx),
    Evtx(Evtx),
    Journal(JournalEntry),
}
pub type LogMessageOpt = Option<LogMessage>;

impl LogMessage {
    /// Returns the datetime of the log message.
    pub fn dt(&self) -> &DateTimeL {
        match self {
            LogMessage::Sysline(sysline) => sysline.dt(),
            LogMessage::Utmpx(utmpx) => utmpx.dt(),
            LogMessage::Evtx(evtx) => evtx.dt(),
            LogMessage::Journal(journal) => journal.dt(),
        }
    }
}

/// Type alias for bytes offsets of the beginning and end of the
/// datetime substring within a `String`.
pub type DtBegEndPair = (usize, usize);
/// Type alias for [`Option`] of [`DtBegEndPair`].
pub type DtBegEndPairOpt = Option<DtBegEndPair>;
