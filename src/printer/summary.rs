// src/printer/summary.rs

//! CLI option `--summary` printing functions.
//! Only used by `s4.rs`.

#![allow(non_camel_case_types)]

use std::collections::{
    BTreeMap,
    HashMap,
};
use std::fmt;
use std::path::PathBuf;
use std::str;
use std::time::{
    Duration,
    Instant,
};

use ::chrono::{
    DateTime,
    Datelike,
    Local,
    TimeZone,
    Timelike,
};
use ::current_platform::CURRENT_PLATFORM;
use ::si_trace_print::defñ;

use crate::common::{
    debug_panic,
    AllocatorChosen,
    Count,
    FPath,
    FileType,
    FileTypeArchive,
    LogMessageType,
    PathId,
    SetPathId,
    FIXEDOFFSET0,
    SUBPATH_SEP,
};
use crate::data::common::LogMessage;
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    DateTimeParseDatasCompiledCount,
    DateTimeParseInstr,
    Utc,
    DATETIME_PARSE_DATAS,
    DATETIME_PARSE_DATAS_LEN,
};
use crate::data::evtx::Evtx;
use crate::data::fixedstruct::FixedStruct;
use crate::data::journal::JournalEntry;
use crate::data::pydataevent::PyDataEvent;
use crate::data::sysline::SyslineP;
use crate::debug::printers::{
    de_err,
    de_wrn,
    e_err,
};
use crate::printer::printers::{
    fpath_to_prependpath,
    print_colored_stderr,
    Color,
    ColorChoice,
    PrinterLogMessage,
    color_dimmed,
    COLOR_ERROR,
};
use crate::python::pyrunner::{
    ExitStatus,
    PythonPathsRan,
};
use crate::readers::blockreader::SummaryBlockReader;
use crate::readers::filepreprocessor::ProcessPathResult;
use crate::readers::fixedstructreader::SummaryFixedStructReader;
use crate::readers::linereader::SummaryLineReader;
use crate::readers::pyeventreader::PyEventType;
use crate::readers::summary::{
    Summary,
    SummaryOpt,
    SummaryReaderData,
};
use crate::readers::syslinereader::SummarySyslineReader;
use crate::readers::syslogprocessor::FileProcessingResultBlockZero;

pub type MapPathIdSummaryPrint = BTreeMap<PathId, SummaryPrinted>;
pub type MapPathIdSummary = HashMap<PathId, Summary>;
pub type MapPathIdToProcessPathResult = HashMap<PathId, ProcessPathResult>;
pub type MapPathIdToProcessPathResultOrdered = BTreeMap<PathId, ProcessPathResult>;
pub type MapPathIdToFPath = BTreeMap<PathId, FPath>;
pub type MapPathIdToColor = HashMap<PathId, Color>;
pub type MapPathIdToPrinterLogMessage = HashMap<PathId, PrinterLogMessage>;
pub type MapPathIdToModifiedTime = HashMap<PathId, DateTimeLOpt>;
pub type MapPathIdToFileProcessingResultBlockZero = HashMap<PathId, FileProcessingResultBlockZero>;
pub type MapPathIdToFileType = HashMap<PathId, FileType>;
pub type MapPathIdToLogMessageType = HashMap<PathId, LogMessageType>;

/// Print the various caching statistics.
const OPT_SUMMARY_PRINT_CACHE_STATS: bool = true;

/// Print the various drop statistics.
const OPT_SUMMARY_PRINT_DROP_STATS: bool = true;

/// For printing various levels of indentation.
const OPT_SUMMARY_PRINT_INDENT1: &str = "  ";
const OPT_SUMMARY_PRINT_INDENT2: &str = "      ";
const OPT_SUMMARY_PRINT_INDENT3: &str = "                   ";

/// datetime format for primary printing of datetime
const DATETIMEFMT: &str = "%Y-%m-%d %H:%M:%S %:z";
/// datetime format for secondary printing of datetime
const DATETIMEFMT_SEC: &str = "%Y-%m-%d %H:%M:%S.%6f %:z";

fn print_dimmed(
    s: &String,
    color_choice_opt: Option<ColorChoice>,
) {
    let color_dimmed_ = color_dimmed();
    print_colored_stderr(
        color_dimmed_,
        color_choice_opt,
        s.as_bytes()
    );
}

/// print the passed `DateTimeL` as UTC with dimmed color
fn print_datetime_utc_dimmed(
    dt: &DateTimeL,
    color_choice_opt: Option<ColorChoice>,
) {
    let dt_utc = dt.with_timezone(&FIXEDOFFSET0);
    let dt_utc_s = dt_utc.format(DATETIMEFMT_SEC);
    let s = format!("({})", dt_utc_s);
    print_dimmed(&s, color_choice_opt);
}

/// print the passed `DateTimeL` as-is and with UTC dimmed color
fn print_datetime_asis_utc_dimmed(
    dt: &DateTimeL,
    color_choice_opt: Option<ColorChoice>,
) {
    let dt_s = dt.format(DATETIMEFMT);
    eprint!("{} ", dt_s);
    let dt_utc = dt.with_timezone(&FIXEDOFFSET0);
    let dt_utc_s = dt_utc.format(DATETIMEFMT_SEC);
    let s = format!("({})", dt_utc_s);
    print_dimmed(&s, color_choice_opt);
}

/// Statistics about the main processing thread's printing activity.
/// Used with CLI option `--summary`.
#[derive(Copy, Clone, Default)]
pub struct SummaryPrinted {
    /// count of bytes printed
    pub bytes: Count,
    /// count of stdout.flush calls
    pub flushed: Count,
    /// underlying `LogMessageType`
    pub logmessagetype: LogMessageType,
    /// count of `Lines` printed
    pub lines: Count,
    /// count of `Syslines` printed
    pub syslines: Count,
    /// count of `FixedStruct` printed
    pub fixedstructentries: Count,
    /// count of `PyDataEvent` printed
    pub etlentries: Count,
    /// count of `Evtx` printed
    pub evtxentries: Count,
    /// count of `JournalEntry` printed
    pub journalentries: Count,
    /// count of `Odl` printed
    pub odlentries: Count,
    /// last datetime printed
    pub dt_first: DateTimeLOpt,
    pub dt_last: DateTimeLOpt,
}

pub type SummaryPrintedOpt = Option<SummaryPrinted>;

impl fmt::Debug for SummaryPrinted {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Printed:")
            .field("bytes", &self.bytes)
            .field("flushes", &self.flushed)
            .field("lines", &self.lines)
            .field("syslines", &self.syslines)
            .field(
                "dt_first",
                &format_args!(
                    "{}",
                    match self.dt_first {
                        Some(dt) => {
                            dt.to_string()
                        }
                        None => {
                            String::from("None")
                        }
                    }
                ),
            )
            .field(
                "dt_last",
                &format_args!(
                    "{}",
                    match self.dt_last {
                        Some(dt) => {
                            dt.to_string()
                        }
                        None => {
                            String::from("None")
                        }
                    }
                ),
            )
            .finish()
    }
}

impl SummaryPrinted {
    pub fn new(logmessagetype: LogMessageType) -> SummaryPrinted {
        SummaryPrinted {
            bytes: 0,
            flushed: 0,
            logmessagetype,
            lines: 0,
            syslines: 0,
            fixedstructentries: 0,
            etlentries: 0,
            evtxentries: 0,
            journalentries: 0,
            odlentries: 0,
            dt_first: None,
            dt_last: None,
        }
    }

    /// Print a `SummaryPrinted` with color on stderr for a file.
    ///
    /// Mimics debug print but with colorized zero values.
    /// Only colorize if associated `SummaryOpt` has corresponding
    /// non-zero values.
    pub fn print_colored_stderr(
        &self,
        color_choice_opt: Option<ColorChoice>,
        summary_opt: &SummaryOpt,
        indent1: &str,
        indent2: &str,
    ) {
        let summary: &Summary = match summary_opt {
            Some(s) => s,
            None => return,
        };
        let (
            summaryblockreader_opt,
            summarylinereader_opt,
            _summarysyslinereader_opt,
            _summarysyslogprocessor_opt,
            summaryfixedstructreader_opt,
            summarypyeventreader_opt,
            summaryevtxreader_opt,
            summaryjournalreader_opt,
        ) = match &summary.readerdata {
            // `Dummy` may occur for files without adequate read permissions
            SummaryReaderData::Dummy => return,
            SummaryReaderData::Syslog(
                (
                    summaryblockreader,
                    summarylinereader,
                    summarysyslinereader,
                    summarysyslogprocessor,
                )
            ) => {
                (
                    Some(summaryblockreader),
                    Some(summarylinereader),
                    Some(summarysyslinereader),
                    Some(summarysyslogprocessor),
                    None,
                    None,
                    None,
                    None,
                )
            }
            SummaryReaderData::FixedStruct(
                (
                    summaryblockreader,
                    summaryfixedstructreader,
                )
            ) => {
                (
                    Some(summaryblockreader),
                    None,
                    None,
                    None,
                    Some(summaryfixedstructreader),
                    None,
                    None,
                    None,
                )
            }
            SummaryReaderData::PyEvent(summarypyeventreader) => {
                (
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(summarypyeventreader),
                    None,
                    None,
                )
            }
            SummaryReaderData::Etvx(summaryevtxreader) => {
                (
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(summaryevtxreader),
                    None,
                )
            }
            SummaryReaderData::Journal(summaryjournalreader) => {
                (
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(summaryjournalreader),
                )
            }
        };
        eprintln!("{}Printed:", indent1);

        match summaryblockreader_opt {
            Some(summaryblockreader) => {
                eprint!("{}bytes         : ", indent2);
                let is_err: bool = self.bytes == 0 && summaryblockreader.blockreader_bytes != 0;
                eprintln_display_color_error(
                    &self.bytes,
                    |_| is_err,
                    &color_choice_opt.unwrap_or_default(),
                );

                eprintln!("{}flushes       : {}", indent2, self.flushed);

                if summarylinereader_opt.is_some() {
                    eprint!("{}lines         : ", indent2);
                    let is_err: bool = self.lines == 0 && summaryblockreader.blockreader_bytes != 0;
                    eprintln_display_color_error(
                        &self.lines,
                        |_| is_err,
                        &color_choice_opt.unwrap_or_default(),
                    );
                }
            }
            None => {}
        }

        match summaryfixedstructreader_opt {
            Some(summaryfixedstructreader) => {
                eprint!("{}entries       : ", indent2);
                let is_err: bool = self.fixedstructentries == 0 && summaryfixedstructreader.fixedstructreader_utmp_entries != 0;
                eprintln_display_color_error(
                    &self.fixedstructentries,
                    |_| is_err,
                    &color_choice_opt.unwrap_or_default(),
                );
            }
            None => {}
        }

        match summarylinereader_opt {
            // if lines were processed but no syslines were processed
            // then hint at an error with colored text
            Some(summarylinereader) => {
                eprint!("{}syslines      : ", indent2);
                let is_err: bool = self.syslines == 0 && summarylinereader.linereader_lines != 0;
                eprintln_display_color_error(
                    &self.syslines,
                    |_| is_err,
                    &color_choice_opt.unwrap_or_default(),
                );
            }
            None => {}
        }

        match summarylinereader_opt {
            Some(summarylinereader) => {
                if self.dt_first.is_none() && summarylinereader.linereader_lines != 0 {
                    // if no datetime_first was processed but lines were processed
                    // then hint at an error with colored text
                    eprint!("{}datetime first: ", indent2);
                    print_colored_stderr(
                        COLOR_ERROR,
                        color_choice_opt,
                        "None Found".as_bytes(),
                    );
                    eprintln!();
                } else {
                    match self.dt_first {
                        Some(dt) => {
                            eprint!("{}datetime first: ", indent2);
                            print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                            eprintln!();
                        }
                        None => {}
                    }
                }
                if self.dt_last.is_none() && summarylinereader.linereader_lines != 0 {
                    // if no datetime_last was processed but lines were processed
                    // then hint at an error with colored text
                    eprint!("{}datetime last : ", indent2);
                    print_colored_stderr(COLOR_ERROR, color_choice_opt, "None Found".as_bytes());
                    eprintln!();
                } else {
                    match self.dt_last {
                        Some(dt) => {
                            eprint!("{}datetime last : ", indent2);
                            print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                            eprintln!();
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }

        match summarypyeventreader_opt {
            Some(summarypyeventreader) => {
                eprintln!("{}bytes         : {}", indent2, self.bytes);
                eprintln!("{}flushes       : {}", indent2, self.flushed);
                eprintln!("{}events        : {}", indent2, summarypyeventreader.pyeventreader_events_accepted);
                match summarypyeventreader.pyeventreader_datetime_first_accepted {
                    Some(dt) => {
                        eprint!("{}datetime first: ", indent2);
                        print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
                match summarypyeventreader.pyeventreader_datetime_last_accepted {
                    Some(dt) => {
                        eprint!("{}datetime last : ", indent2);
                        print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                        debug_assert!(summarypyeventreader.pyeventreader_datetime_first_accepted.is_some());
                    }
                    None => {
                        debug_assert!(summarypyeventreader.pyeventreader_datetime_first_accepted.is_none());
                    }
                }
            }
            None => {}
        }

        match summaryevtxreader_opt {
            Some(summaryevtxreader) => {
                eprintln!("{}bytes         : {}", indent2, self.bytes);
                eprintln!("{}flushes       : {}", indent2, self.flushed);
                eprintln!("{}Events        : {}", indent2, self.evtxentries);
                match summaryevtxreader.evtxreader_datetime_first_accepted {
                    Some(dt) => {
                        eprint!("{}datetime first: ", indent2);
                        print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
                match summaryevtxreader.evtxreader_datetime_last_accepted {
                    Some(dt) => {
                        eprint!("{}datetime last : ", indent2);
                        print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
            }
            None => {}
        }

        match summaryjournalreader_opt {
            Some(summaryjournalreader) => {
                eprintln!("{}bytes         : {}", indent2, self.bytes);
                eprintln!("{}flushes       : {}", indent2, self.flushed);
                eprintln!("{}journal events: {}", indent2, self.journalentries);
                match summaryjournalreader.journalreader_datetime_first_accepted {
                    Some(dt) => {
                        eprint!("{}datetime first: ", indent2);
                        print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
                match summaryjournalreader.journalreader_datetime_last_accepted {
                    Some(dt) => {
                        eprint!("{}datetime last : ", indent2);
                        print_datetime_asis_utc_dimmed(&dt, color_choice_opt);
                        eprintln!();
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    fn summaryprint_update_dt(
        &mut self,
        dt: &DateTimeL,
    ) {
        defñ!();
        match self.dt_first {
            Some(dt_first) => {
                if dt < &dt_first {
                    self.dt_first = Some(*dt);
                };
            }
            None => {
                self.dt_first = Some(*dt);
            }
        };
        match self.dt_last {
            Some(dt_last) => {
                if dt > &dt_last {
                    self.dt_last = Some(*dt);
                };
            }
            None => {
                self.dt_last = Some(*dt);
            }
        };
    }

    /// Update a `SummaryPrinted` with information from a printed `Sysline`.
    pub fn summaryprint_update_sysline(
        &mut self,
        syslinep: &SyslineP,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        // TODO: can `All` be removed here?
        debug_assert!(
            matches!(self.logmessagetype, LogMessageType::Sysline | LogMessageType::All),
            "Unexpected LogMessageType {:?}", self.logmessagetype,
        );
        self.syslines += 1;
        self.lines += (*syslinep).count_lines();
        self.bytes += printed;
        self.flushed += flushed;
        self.summaryprint_update_dt((*syslinep).dt());
    }

    /// Update a `SummaryPrinted` with information from a printed `FixedStruct`.
    pub fn summaryprint_update_fixedstruct(
        &mut self,
        entry: &FixedStruct,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        debug_assert!(
            matches!(self.logmessagetype, LogMessageType::FixedStruct | LogMessageType::All),
            "Unexpected LogMessageType {:?}", self.logmessagetype,
        );
        self.fixedstructentries += 1;
        self.bytes += printed;
        self.flushed += flushed;
        self.summaryprint_update_dt(entry.dt());
    }

    pub fn summaryprint_update_pyevent(
        &mut self,
        pyevent: &PyDataEvent,
        pyevent_type: PyEventType,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        // TODO [2025/12/23]: can `LogMessageType::All` be removed here?
        //      I don't recall how this is ends up getting passed in.
        debug_assert!(
            matches!(self.logmessagetype, LogMessageType::PyEvent | LogMessageType::All),
            "Unexpected LogMessageType {:?}", self.logmessagetype,
        );
        match pyevent_type {
            PyEventType::Etl { .. } => {
                self.etlentries += 1;
            }
            PyEventType::Odl { .. } => {
                self.odlentries += 1;
            }
        }
        self.bytes += printed;
        self.flushed += flushed;
        self.summaryprint_update_dt(pyevent.dt());
    }

    /// Update a `SummaryPrinted` with information from a printed `Evtx`.
    pub fn summaryprint_update_evtx(
        &mut self,
        evtx: &Evtx,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        // TODO: can `All` be removed here?
        debug_assert!(
            matches!(self.logmessagetype, LogMessageType::Evtx | LogMessageType::All),
            "Unexpected LogMessageType {:?}", self.logmessagetype,
        );
        self.evtxentries += 1;
        self.bytes += printed;
        self.flushed += flushed;
        self.summaryprint_update_dt(evtx.dt());
    }

    pub fn summaryprint_update_journalentry(
        &mut self,
        journalentry: &JournalEntry,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        // TODO: can `All` be removed here?
        debug_assert!(
            matches!(self.logmessagetype, LogMessageType::Journal | LogMessageType::All),
            "Unexpected LogMessageType {:?}", self.logmessagetype,
        );
        self.journalentries += 1;
        self.bytes += printed;
        self.flushed += flushed;
        self.summaryprint_update_dt(journalentry.dt());
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `Sysline`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_sysline(
        syslinep: &SyslineP,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_sysline(syslinep, printed, flushed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Sysline);
                sp.summaryprint_update_sysline(syslinep, printed, flushed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `FixedStruct`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_fixedstruct(
        fixedstruct: &FixedStruct,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_fixedstruct(fixedstruct, printed, flushed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::FixedStruct);
                sp.summaryprint_update_fixedstruct(fixedstruct, printed, flushed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a PyEvent.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_pyevent(
        pyevent: &PyDataEvent,
        pathid: &PathId,
        pyevent_type: PyEventType,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_pyevent(pyevent, pyevent_type, printed, flushed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::PyEvent);
                sp.summaryprint_update_pyevent(pyevent, pyevent_type, printed, flushed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `FixedStruct`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_evtx(
        evtx: &Evtx,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_evtx(evtx, printed, flushed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Evtx);
                sp.summaryprint_update_evtx(evtx, printed, flushed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted` for a `JournalEntry`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn summaryprint_map_update_journalentry(
        journalentry: &JournalEntry,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        match map_.get_mut(pathid) {
            Some(sp) => {
                sp.summaryprint_update_journalentry(journalentry, printed, flushed);
            }
            None => {
                let mut sp = SummaryPrinted::new(LogMessageType::Journal);
                sp.summaryprint_update_journalentry(journalentry, printed, flushed);
                map_.insert(*pathid, sp);
            }
        };
    }

    /// Update a mapping of `PathId` to `SummaryPrinted`.
    ///
    /// Helper function to function `processing_loop`.
    pub fn _summaryprint_map_update(
        logmessage: &LogMessage,
        pathid: &PathId,
        map_: &mut MapPathIdSummaryPrint,
        printed: Count,
        flushed: Count,
    ) {
        defñ!();
        match logmessage {
            LogMessage::Evtx(evtx) => {
                Self::summaryprint_map_update_evtx(evtx, pathid, map_, printed, flushed)
            }
            LogMessage::FixedStruct(entry) => {
                Self::summaryprint_map_update_fixedstruct(
                    entry, pathid, map_, printed, flushed
                )
            }
            LogMessage::Journal(journalentry) => {
                Self::summaryprint_map_update_journalentry(
                    journalentry, pathid, map_, printed, flushed
                )
            }
            LogMessage::PyEvent(pyevent, pyevent_type) => {
                Self::summaryprint_map_update_pyevent(
                    pyevent, pathid, *pyevent_type, map_, printed, flushed
                )
            }
            LogMessage::Sysline(syslinep) => {
                Self::summaryprint_map_update_sysline(syslinep, pathid, map_, printed, flushed)
            }
        }
    }
}

/// Helper function to function `processing_loop`.
#[inline(always)]
pub fn summary_update(
    pathid: &PathId,
    summary: Summary,
    map_: &mut MapPathIdSummary,
) {
    if let Some(val) = map_.insert(*pathid, summary) {
        eprintln!(
            "Error: processing_loop: map_pathid_summary already contains key {:?} with {:?}, overwritten",
            pathid, val
        );
    };
}

/// Print the entire `--summary`.
/// This is the "entry point" for print the summary of all files.
pub fn print_summary(
    map_pathid_results: MapPathIdToProcessPathResult,
    map_pathid_results_invalid: MapPathIdToProcessPathResultOrdered,
    map_pathid_path: MapPathIdToFPath,
    map_pathid_modified_time: MapPathIdToModifiedTime,
    map_pathid_file_processing_result: MapPathIdToFileProcessingResultBlockZero,
    map_pathid_filetype: MapPathIdToFileType,
    map_pathid_logmessagetype: MapPathIdToLogMessageType,
    map_pathid_color: MapPathIdToColor,
    mut map_pathid_summary: MapPathIdSummary,
    mut map_pathid_sumpr: MapPathIdSummaryPrint,
    color_choice: ColorChoice,
    color_default: Color,
    paths_total: usize,
    paths_printed_logmessages: SetPathId,
    summaryprinted: SummaryPrinted,
    filter_dt_after_opt: &DateTimeLOpt,
    filter_dt_before_opt: &DateTimeLOpt,
    local_now: &DateTime<Local>,
    utc_now: &DateTime<Utc>,
    chan_recv_ok: Count,
    chan_recv_err: Count,
    start_time: Instant,
    named_temp_files_count: usize,
    thread_count: usize,
    thread_err_count: usize,
    allocator_chosen: AllocatorChosen,
) {
    let finish_time = Instant::now();
    // reset the text color to default
    print_colored_stderr(
        color_default,
        Some(color_choice),
        "".as_bytes()
    );

    // print details about all the valid files
    print_all_files_summaries(
        &map_pathid_path,
        &map_pathid_modified_time,
        &map_pathid_file_processing_result,
        &map_pathid_filetype,
        &map_pathid_logmessagetype,
        &map_pathid_color,
        &mut map_pathid_summary,
        &mut map_pathid_sumpr,
        &color_choice,
        &color_default,
    );
    // print a short note about the invalid files
    print_files_processpathresult(
        &map_pathid_results_invalid,
        &color_choice,
        &color_default,
        &COLOR_ERROR,
    );

    // here is the final printed summary of the all files
    eprintln!("Program Summary:\n");
    eprintln!("Paths considered       : {}", paths_total);
    eprintln!("Paths not processed    : {}", map_pathid_results_invalid.len());
    eprintln!("Files processed        : {}", map_pathid_results.len());
    eprintln!("Files printed          : {}", paths_printed_logmessages.len());
    eprintln!("Printed bytes          : {}", summaryprinted.bytes);
    eprintln!("Printed flushes        : {}", summaryprinted.flushed);
    eprintln!("Printed lines          : {}", summaryprinted.lines);
    eprintln!("Printed syslines       : {}", summaryprinted.syslines);
    eprintln!("Printed ETL events     : {}", summaryprinted.etlentries);
    eprintln!("Printed EVTX events    : {}", summaryprinted.evtxentries);
    // TODO: [2023/03/26] eprint count of EVTX files "out of order".
    eprintln!("Printed fixedstruct    : {}", summaryprinted.fixedstructentries);
    // TODO: [2024/02/25] eprint count of FixedStruct files "out of order".
    eprintln!("Printed journal events : {}", summaryprinted.journalentries);
    let count: isize = match DateTimeParseDatasCompiledCount.read() {
        Ok(count) => *count as isize,
        // XXX: hacky hint that the count is not available
        Err(_) => -1,
    };
    eprintln!("Printed ODL events     : {}", summaryprinted.odlentries);
    eprintln!("Regex patterns known   : {}", DATETIME_PARSE_DATAS_LEN);
    eprintln!("Regex patterns compiled: {}", count);

    eprint!("Datetime filter -a     :");
    match filter_dt_after_opt {
        Some(dt) => {
            let dt_s = dt.format(DATETIMEFMT);
            eprint!(" {} ", dt_s);
            print_datetime_utc_dimmed(dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    eprint!("Datetime printed first :");
    match summaryprinted.dt_first {
        Some(dt) => {
            let dt_s = dt.format(DATETIMEFMT);
            eprint!(" {} ", dt_s);
            print_datetime_utc_dimmed(&dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    eprint!("Datetime printed last  :");
    match summaryprinted.dt_last {
        Some(dt) => {
            let dt_s = dt.format(DATETIMEFMT);
            eprint!(" {} ", dt_s);
            print_datetime_utc_dimmed(&dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    eprint!("Datetime filter -b     :");
    match filter_dt_before_opt {
        Some(dt) => {
            let dt_s = dt.format(DATETIMEFMT);
            eprint!(" {} ", dt_s);
            print_datetime_utc_dimmed(&dt, Some(color_choice));
            eprintln!();
        }
        None => eprintln!(),
    }
    // print the time now as this program sees it
    let local_now_s = local_now.format(DATETIMEFMT);
    eprint!("Datetime Now           : {} ", local_now_s);
    // print UTC now without fractional, and with numeric offset `-00:00`
    // instead of `Z`
    let utc_now = Utc
        .with_ymd_and_hms(
            utc_now.year(),
            utc_now.month(),
            utc_now.day(),
            utc_now.hour(),
            utc_now.minute(),
            utc_now.second(),
        )
        .unwrap()
        .with_timezone(&FIXEDOFFSET0);
    print_datetime_utc_dimmed(&utc_now, Some(color_choice));
    eprintln!();
    // print the python executables that were run
    match PythonPathsRan.read() {
        Ok(python_exes_ran) => {
            for python_exe in python_exes_ran.iter() {
                eprint!("Python Interpreter     : {}", python_exe);
                // print the realpath if different than `python_exe`
                let path_: PathBuf = PathBuf::from(python_exe);
                match path_.canonicalize() {
                    Ok(pathbuf) => match pathbuf.to_str() {
                        Some(path_s) => {
                            if path_s != python_exe.as_str() {
                                print_dimmed(&format!(" ({})", path_s), Some(color_choice));
                            }
                        }
                        None => {}
                    },
                    Err(_err) => {
                        de_err!("canonicalize failed for {:?}; {}", path_, _err);
                    }
                }
                eprintln!();
            }
        }
        Err(_) => {}
    }
    // print basic stats about the channel
    eprintln!("Channel Receive ok     : {}", chan_recv_ok);
    eprintln!("Channel Receive err    : {}", chan_recv_err);
    eprintln!("Threads Spawned        : {}", thread_count);
    eprintln!("Thread Spawn errors    : {}", thread_err_count);
    let run_time = finish_time.checked_duration_since(start_time);
    let run_time_w_summary = Instant::now().checked_duration_since(start_time);
    match (run_time, run_time_w_summary) {
        (Some(rt), Some(rts)) => {
            eprintln!("Program Run Time       : {:.3} (seconds) (including this summary {:.3})",
                rt.as_secs_f64(), rts.as_secs_f64());
        }
        _ => {
            eprintln!("Program Run Time       : unknown");
        }
    }
    eprintln!("Temporary files created: {}", named_temp_files_count);
    eprintln!("Platform               : {}", CURRENT_PLATFORM);
    eprintln!("Allocator              : {}", allocator_chosen);
}

// TODO: [2023/04/05] move printing of `file size` from per-file "Processed:"
//       section to "About:" section. Having in the "Processed:" section is
//       confusing about what was actually read from storage (implies the
//       entire file was read, which is not true in most cases).

/// Print the file _About_ section (multiple lines).
fn print_file_about(
    path: &FPath,
    modified_time: &DateTimeLOpt,
    file_processing_result: Option<&FileProcessingResultBlockZero>,
    filetype: &FileType,
    logmessagetype: &LogMessageType,
    summary_opt: &SummaryOpt,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprint!("File: ");
    // print path
    print_colored_stderr(
        *color,
        Some(*color_choice),
        fpath_to_prependpath(path).as_bytes()
    );
    eprintln!("\n{}About:", OPT_SUMMARY_PRINT_INDENT1);
    // XXX: experimentation revealed std::fs::Metadata::is_symlink to be unreliable on WSL Ubuntu
    let mut path1: &str = path.as_str();
    let mut subpath: &str = "";
    if filetype.is_archived() && path.contains(SUBPATH_SEP) {
        // only canonicalize the first part of the path,
        // e.g. `"path/to/file.tar"` from `"path/to/file.tar|inner/file.txt"`
        (path1, subpath) = path1.split_once(SUBPATH_SEP).unwrap_or((path.as_str(), ""));
    }
    match std::fs::canonicalize(path1) {
        Ok(pathb) => match pathb.to_str() {
            Some(s) => {
                if s != path.as_str() {
                    eprint!("{}real path      : {}", OPT_SUMMARY_PRINT_INDENT2, s);
                    eprintln!();
                }
            }
            None => {}
        },
        Err(_err) => {
            de_err!("canonicalize failed for {:?}; {}", path1, _err);
        }
    }
    if !subpath.is_empty() {
        eprint!("{}archive subpath: {}", OPT_SUMMARY_PRINT_INDENT2, subpath);
        eprintln!();
    }
    match summary_opt {
        Some(summary) => {
            match &summary.path_ntf {
                Some(path_ntf) => {
                    eprint!("{}temporary path : {}", OPT_SUMMARY_PRINT_INDENT2, path_ntf);
                    eprintln!();
                }
                None => {}
            }
        }
        None => {}
    }
    // print other facts
    match modified_time {
        Some(dt) => {
            eprint!("{}modified time  : ", OPT_SUMMARY_PRINT_INDENT2);
            print_datetime_asis_utc_dimmed(dt, Some(*color_choice));
            eprintln!();
        }
        None => {}
    }
    // if `FileProcessingResultBlockZero::FileErrEmpty` then print the error
    // and be done printing the summary for this file
    if let Some(result) = file_processing_result {
        if matches!(result, FileProcessingResultBlockZero::FileErrEmpty) {
            eprint!("{}Processing Err : ", OPT_SUMMARY_PRINT_INDENT2);
            print_colored_stderr(
                COLOR_ERROR,
                Some(*color_choice),
                format!("{}", result).as_bytes(),
            );
            eprintln!();
            return;
        }
    }
    // print filetype leadin
    eprint!("{}filetype       : ", OPT_SUMMARY_PRINT_INDENT2);
    // print the filetype basic info
    match filetype {
        FileType::Text { encoding_type: et, .. } => {
            eprint!("{} {}", filetype, et);
        }
        | FileType::Odl { archival_type: _ , odl_sub_type: ot} => {
            eprint!("{}", ot);
        }
        FileType::FixedStruct { .. }
        | FileType::Etl { .. }
        | FileType::Evtx { .. }
        | FileType::Journal { .. }
        | FileType::Unparsable { .. }
        => {
            eprint!("{}", filetype);
        }
    }
    // print the filetype archival type if applicable
    match filetype {
        FileType::Etl { archival_type: at }
        | FileType::Evtx { archival_type: at }
        | FileType::FixedStruct { archival_type: at, .. }
        | FileType::Journal { archival_type: at }
        | FileType::Odl { archival_type: at , .. }
        | FileType::Text { archival_type: at, .. }
        => {
            match at {
                FileTypeArchive::Normal => {}
                fta => {
                    eprint!(" ({})", fta);
                }
            }
        }
        FileType::Unparsable => {
            debug_panic!("unexpected FileType::Unparsable");
        }
    }
    // print the descriptive pretty name
    eprintln!(" ({})", filetype.pretty_name());
    // print about FixedStruct types if applicable
    match filetype {
        FileType::FixedStruct { archival_type: _, fixedstruct_type: fst } => {
            eprintln!("{}fixedstructtype: {:?}", OPT_SUMMARY_PRINT_INDENT2, fst);
        }
        _ => {}
    }
    // print log message type
    eprintln!("{}logmessagetype : {}", OPT_SUMMARY_PRINT_INDENT2, logmessagetype);

    match summary_opt {
        Some(summary) => {
            match &summary.readerdata {
                SummaryReaderData::FixedStruct((_, summaryfixedstructreader)) => {
                    match summaryfixedstructreader.fixedstructreader_fixedstructtype_opt {
                        Some(fst) => {
                            eprintln!(
                                "{}fixedstructtype: {:?}",
                                OPT_SUMMARY_PRINT_INDENT2,
                                fst,
                            );
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }
        None => {}
    }
    // print `FileProcessingResult` if it was not okay
    if let Some(result) = file_processing_result {
        if !result.is_ok() {
            eprint!("{}Processing Err : ", OPT_SUMMARY_PRINT_INDENT2);
            let indent: String = OPT_SUMMARY_PRINT_INDENT2.to_string() + "                 ";
            // if the error message has newlines then indent it
            let s: String = format!("{}", result);
            let s: String = String::from(s.trim_end());
            let s: String = s.replace("\n", &format!("\n{}", indent));
            print_colored_stderr(
                COLOR_ERROR,
                Some(*color_choice),
                s.as_bytes(),
            );
            eprintln!();
        }
    }
}

/// print `value` normally if `predicate` returns `false`
/// else print `value` in color with the error color
fn eprintln_display_color_error<T, F>(value: &T, predicate: F, color_choice: &ColorChoice)
    where
        F: Fn(&T) -> bool,
        T: fmt::Display,
{
    if ! predicate(value) {
        eprintln!("{}", value);
    } else {
        let data = format!("{}", value);
        print_colored_stderr(
            COLOR_ERROR,
            Some(*color_choice),
            data.as_bytes(),
        );
        eprintln!();
    }
}

/// print `value` if `predicate` returns `false`
/// else print `value` in color with the error color
#[allow(dead_code)]
fn eprintln_debug_color_error<T, F>(value: &T, predicate: F, color_choice: &ColorChoice)
    where
        F: Fn(&T) -> bool,
        T: fmt::Debug,
{
    if ! predicate(value) {
        eprintln!("{:?}", value);
    } else {
        let data = format!("{:?}", value);
        print_colored_stderr(
            COLOR_ERROR,
            Some(*color_choice),
            data.as_bytes(),
        );
        eprintln!();
    }
}


/// Print the (optional) [`Summary`] (multiple lines) processed sections.
///
/// [`Summary`]: s4lib::readers::summary::Summary
fn print_summary_opt_processed(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    let summary = match summary_opt {
        Some(summary) => summary,
        None => {
            eprintln!("{}Processed: None", OPT_SUMMARY_PRINT_INDENT1);
            return;
        }
    };
    if summary.readerdata.is_dummy() {
        // `Dummy` may occur for files without adequate read permissions
        // there will be no interested information in gathered statistics
        return;
    }
    let indent1 = OPT_SUMMARY_PRINT_INDENT1;
    let indent2 = OPT_SUMMARY_PRINT_INDENT2;
    eprintln!("{}Processed:", indent1);
    print_summary_opt_processed_summaryblockreader(
        summary,
        indent2,
    );
    match &summary.readerdata {
        // `Dummy` may occur for files without adequate read permissions
        SummaryReaderData::Dummy => return,
        SummaryReaderData::Syslog(
            (
                _summaryblockreader,
                summarylinereader,
                summarysyslinereader,
                _summarysyslogprocessor,
            ),
        ) => {
            eprintln!("{}lines         : {}", indent2, summarylinereader.linereader_lines);
            eprintln!(
                "{}lines high    : {}",
                indent2, summarylinereader.linereader_lines_stored_highest
            );
            eprintln!("{}syslines      : {}", indent2, summarysyslinereader.syslinereader_syslines);
            eprintln!(
                "{}syslines high : {}",
                indent2, summarysyslinereader.syslinereader_syslines_stored_highest
            );
        }
        SummaryReaderData::FixedStruct((
            _summaryblockreader,
            summaryfixedstructreader,
        )) => {
            eprintln!("{}entries       : {}", indent2, summaryfixedstructreader.fixedstructreader_utmp_entries);
            eprintln!("{}entry size    : {} (bytes)",
                indent2, summaryfixedstructreader.fixedstructreader_fixedstruct_size
            );
            eprintln!("{}entry hi-score: {}",
                indent2, summaryfixedstructreader.fixedstructreader_high_score
            );
            eprint!("{}first entry   : ",
                indent2,
            );
            eprintln!("@{:?}", summaryfixedstructreader.fixedstructreader_first_entry_fileoffset);
            eprintln!(
                "{}entry high    : {}",
                indent2, summaryfixedstructreader.fixedstructreader_utmp_entries_max,
            );
            eprintln!(
                "{}peak map size : {}",
                indent2, summaryfixedstructreader.fixedstructreader_map_tvpair_fo_max_len
            );
            eprintln!(
                "{}out of order? : {}",
                indent2, summaryfixedstructreader.fixedstructreader_entries_out_of_order
            );
        }
        SummaryReaderData::PyEvent(summarypyeventreader) => {
            eprintln!(
                "{}file size          : {1} (0x{1:X}) (bytes)",
                indent2, summarypyeventreader.pyeventreader_filesz,
            );
            eprintln!("{}events processed   : {}", indent2, summarypyeventreader.pyeventreader_events_processed);
            eprintln!("{}events accepted    : {}", indent2, summarypyeventreader.pyeventreader_events_accepted);
            eprint!("{}events out of order: ", indent2);
            eprintln_display_color_error(
                &summarypyeventreader.pyeventreader_out_of_order,
                |x| *x != 0,
                color_choice,
            );
            eprintln!("{}events read max    : {}", indent2, summarypyeventreader.pyeventreader_events_read_max);
            eprintln!("{}events queue high  : {}", indent2, summarypyeventreader.pyeventreader_events_held_max);
            eprintln!("{}Python process polls       : {}", indent2, summarypyeventreader.pyeventreader_python_count_proc_polls);
            eprintln!("{}Python process reads stdout: {}", indent2, summarypyeventreader.pyeventreader_python_count_proc_reads_stdout);
            eprintln!("{}Python process reads stderr: {}", indent2, summarypyeventreader.pyeventreader_python_count_proc_reads_stderr);
            eprintln!("{}Python process writes stdin: {}", indent2, summarypyeventreader.pyeventreader_python_count_proc_writes);
            eprintln!("{}Python pipe recv stdout    : {}", indent2, summarypyeventreader.pyeventreader_python_count_pipe_recv_stdout);
            eprintln!("{}Python pipe recv stderr    : {}", indent2, summarypyeventreader.pyeventreader_python_count_pipe_recv_stderr);
            eprintln!("{}Python pipe size stdout    : {}", indent2, summarypyeventreader.pyeventreader_pipe_sz_stdout);
            eprintln!("{}Python pipe size stderr    : {}", indent2, summarypyeventreader.pyeventreader_pipe_sz_stderr);
            // drop sub-milliseconds data, milliseconds is precise enough
            let dur_ms_wait: Duration = Duration::from_millis(summarypyeventreader.pyeventreader_duration_proc_wait.as_millis() as u64);
            eprintln!("{}Python process waits       : {:?}", indent2, dur_ms_wait);
            let dur_ms_run: Duration = Duration::from_millis(summarypyeventreader.pyeventreader_duration_proc_run.as_millis() as u64);
            eprintln!("{}Python process runtime     : {:?}", indent2, dur_ms_run);
            eprint!("{}Python exit status         : ", indent2);
            let exit_status: &ExitStatus = &summarypyeventreader.pyeventreader_python_exit_status.unwrap_or_default();
            // remove "exit status: " from the ExitStatus display string
            let es_s: String = format!("{}", exit_status).replace("exit status: ", "");
            eprintln_display_color_error(
                &es_s,
                |_| { !exit_status.success() },
                color_choice,
            );
            let args: String = summarypyeventreader.pyeventreader_python_arguments.join(" ");
            eprintln!("{}Python script arguments    : {}", indent2, args);
        }
        SummaryReaderData::Etvx(summaryevtxreader) => {
            eprintln!(
                "{}file size          : {1} (0x{1:X}) (bytes)",
                indent2, summaryevtxreader.evtxreader_filesz,
            );
            // TODO: [2023/04/05] add `sourced` size. Requires additional
            //       tracking in `EvtxReader` (small `EvtxReader` refactor)
            //       and `SummaryEvtxReader`.
            eprintln!("{}Events processed   : {}", indent2, summaryevtxreader.evtxreader_events_processed);
            eprintln!("{}Events accepted    : {}", indent2, summaryevtxreader.evtxreader_events_accepted);
            // print out of order. If there are any, print in red.
            eprint!("{}Events out of order: ", indent2);
            eprintln_display_color_error(
                &summaryevtxreader.evtxreader_out_of_order,
                |x| *x != 0,
                color_choice,
            );
            match summaryevtxreader.evtxreader_datetime_first_processed {
                Some(dt) => {
                    eprint!("{}datetime first     : ", indent2);
                    print_datetime_asis_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            match summaryevtxreader.evtxreader_datetime_last_processed {
                Some(dt) => {
                    eprint!("{}datetime last      : ", indent2);
                    print_datetime_asis_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            // for evtx files, nothing left to print about it so return
            return;
        }
        SummaryReaderData::Journal(summaryjournalreader) => {
            eprintln!(
                "{}file size     : {1} (0x{1:X}) (bytes)",
                indent2, summaryjournalreader.journalreader_filesz,
            );
            eprintln!(
                "{}journal events: {}",
                indent2, summaryjournalreader.journalreader_events_processed,
            );
            // print out of order. If there are any, print in red.
            eprint!("{}out of order  : ", indent2);
            eprintln_display_color_error(
                &summaryjournalreader.journalreader_out_of_order,
                |n| *n != 0,
                color_choice,
            );
            eprintln!(
                "{}lib. API calls: {}",
                indent2, summaryjournalreader.journalreader_api_calls,
            );
            // print API call errors. If there are any, print in red.
            eprint!("{}API errors    : ", indent2);
            eprintln_display_color_error(
                &summaryjournalreader.journalreader_api_call_errors,
                |n| *n != 0,
                color_choice,
            );
            match summaryjournalreader.journalreader_datetime_first_processed {
                Some(dt) => {
                    eprint!("{}datetime first: ", indent2);
                    print_datetime_asis_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            match summaryjournalreader.journalreader_datetime_last_processed {
                Some(dt) => {
                    eprint!("{}datetime last : ", indent2);
                    print_datetime_asis_utc_dimmed(&dt, Some(*color_choice));
                    eprintln!();
                }
                None => {}
            }
            return;
        }
    }

    // print datetime first processed
    let dt_first = summary.datetime_first_processed();
    match dt_first {
        Some(dt_first) => {
            eprint!("{}datetime first     : ", indent2);
            print_datetime_asis_utc_dimmed(&dt_first, Some(*color_choice));
            eprintln!();
        }
        None => {}
    }
    // print datetime last processed
    let dt_last = summary.datetime_last_processed();
    match dt_last {
        Some(dt_last) => {
            eprint!("{}datetime last      : ", indent2);
            print_datetime_asis_utc_dimmed(&dt_last, Some(*color_choice));
            eprintln!();
        }
        None => {}
    }
    // assert that both are None or both are Some
    debug_assert!((dt_first.is_none() || dt_last.is_none()) || (dt_first.is_some() || dt_last.is_some()));

    // print datetime regex patterns
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            _summaryblockreader,
            _summarylinereader,
            summarysyslinereader,
            summarysyslogprocessor,
        )) => {
            if !summarysyslinereader.syslinereader_patterns.is_empty()
            {
                eprintln!("{}Parsers:", OPT_SUMMARY_PRINT_INDENT1);
            }
            for patt in summarysyslinereader
                .syslinereader_patterns
                .iter()
            {
                let dtpd: &DateTimeParseInstr = &DATETIME_PARSE_DATAS[*patt.0];
                eprintln!("{}@[{}] uses {} {:?}", indent2, patt.0, patt.1, dtpd);
            }
            match summarysyslogprocessor.syslogprocessor_missing_year {
                Some(year) => {
                    eprintln!(
                        "{}datetime format missing year; estimated year of last sysline {:?}",
                        OPT_SUMMARY_PRINT_INDENT3, year
                    );
                }
                None => {}
            }
        }
        _ => {}
    }
}

/// Helper to `print_summary_opt_processed`
fn print_summary_opt_processed_summaryblockreader(
    summary: &Summary,
    indent: &str,
) {
    if summary.readerdata.is_dummy() {
        return;
    }
    let summaryblockreader = match summary.blockreader() {
        Some(summaryblockreader) => summaryblockreader,
        None => {
            return;
        }
    };
    let filetype: FileType = match summary.filetype {
        Some(ft) => ft,
        None => {
            debug_panic!("summary.filetype is None");
            return;
        }
    };
    debug_assert!(!filetype.is_evtx());
    debug_assert!(!filetype.is_journal());
    match filetype {
        FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ }
        | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: _ }
        => {
            eprintln!(
                "{}file size     : {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
        }
        FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ }
        | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: _ }
        => {
            eprintln!(
                "{}file size archive   : {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
            eprintln!(
                "{}file size unarchived: {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz_actual
            );
        }
        FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, fixedstruct_type: _ }
        | FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ }
        | FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, fixedstruct_type: _ }
        | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ }
        | FileType::Text{ archival_type: FileTypeArchive::Bz2, encoding_type: _ }
        | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: _ }
        | FileType::Text{ archival_type: FileTypeArchive::Lz4, encoding_type: _ }
        | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: _ }
        => {
            eprintln!(
                "{}file size compressed  : {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz
            );
            eprintln!(
                "{}file size uncompressed: {1} (0x{1:X}) (bytes)",
                indent, summaryblockreader.blockreader_filesz_actual
            );
        }
        FileType::Etl{..}
        | FileType::Evtx{..}
        | FileType::Journal{..}
        | FileType::Odl { .. }
        | FileType::Unparsable
        => {
            debug_panic!("unexpected filetype {:?}", summary.filetype);
            eprintln!("{}unsupported filetype: {:?}", indent, summary.filetype);
            return;
        }
    }
    // TODO: [2023/04/05] add `sourced` count. Requires additional
    //       tracking in `BlockReader`.
    //       i.e. bytes read from storage.
    eprintln!("{}bytes         : {1} (0x{1:X})", indent, summaryblockreader.blockreader_bytes);
    // TODO: [2024/05/05] `bytes total` repeats `file size` printed above.
    //       Remove it entirely.
    eprintln!("{}bytes total   : {1} (0x{1:X})", indent, summaryblockreader.blockreader_bytes_total);
    eprintln!("{}block size    : {1} (0x{1:X})", indent, summaryblockreader.blockreader_blocksz);
    eprintln!("{}blocks        : {}", indent, summaryblockreader.blockreader_blocks);
    let bytes_total =
        summaryblockreader.blockreader_blocks_total * summaryblockreader.blockreader_blocksz;
    eprintln!(
        "{}blocks total  : {} ({} bytes)",
        indent, summaryblockreader.blockreader_blocks_total, bytes_total,
    );
    let bytes_high =
        (summaryblockreader.blockreader_blocks_highest as u64) * summaryblockreader.blockreader_blocksz;
    eprintln!(
        "{}blocks high   : {} ({} bytes)",
        indent, summaryblockreader.blockreader_blocks_highest, bytes_high,
    );
}

/// Print the (optional) `SummaryPrinted` section for one file.
pub fn print_summary_opt_printed(
    summary_print_opt: &SummaryPrintedOpt,
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_print_opt {
        Some(summary_print) => {
            summary_print.print_colored_stderr(
                Some(*color_choice),
                summary_opt,
                OPT_SUMMARY_PRINT_INDENT1,
                OPT_SUMMARY_PRINT_INDENT2,
            );
        }
        None => {
            SummaryPrinted::default().print_colored_stderr(
                Some(*color_choice),
                summary_opt,
                OPT_SUMMARY_PRINT_INDENT1,
                OPT_SUMMARY_PRINT_INDENT2,
            );
        }
    }
}

/// create percentage of `a` to `a + b`
fn percent64(
    a: &u64,
    b: &u64,
) -> f64 {
    let den = (*a as f64) + (*b as f64);
    if den == 0.0 {
        return 0.0;
    }
    ((*a as f64) / den) * 100.0
}

const WIDEP: usize = 4;

fn print_cache_stats_summaryblockreader(
    summaryblockreader: &SummaryBlockReader,
    color_choice: &ColorChoice,
    indent: &str,
    wide: usize,
) {
    // BlockReader::_read_blocks
    let mut percent = percent64(&summaryblockreader.blockreader_read_blocks_hit, &summaryblockreader.blockreader_read_blocks_miss);
    eprint!(
        "{}storage: BlockReader::read_block() blocks                    : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summaryblockreader.blockreader_read_blocks_hit,
        summaryblockreader.blockreader_read_blocks_miss,
        percent,
        summaryblockreader.blockreader_read_blocks_put,
        wide = wide,
        widep = WIDEP,
    );
    // append the rereads count, colorize if greater than 0
    let rereads = summaryblockreader.blockreader_read_blocks_reread_error;
    eprintln_display_color_error(
        &format!(" (rereads {})", rereads),
        |_| rereads > 0,
        color_choice,
    );
    // BlockReader::_read_blocks_cache
    percent = percent64(
        &summaryblockreader.blockreader_read_block_lru_cache_hit,
        &summaryblockreader.blockreader_read_block_lru_cache_miss,
    );
    eprintln!(
        "{}caching: BlockReader::read_block() LRU cache                 : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summaryblockreader.blockreader_read_block_lru_cache_hit,
        summaryblockreader.blockreader_read_block_lru_cache_miss,
        percent,
        summaryblockreader.blockreader_read_block_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
}

fn print_cache_stats_summarylinereader(
    summarylinereader: &SummaryLineReader,
    indent: &str,
    wide: usize,
) {
    // LineReader::_lines
    let mut percent = percent64(&summarylinereader.linereader_lines_hits, &summarylinereader.linereader_lines_miss);
    eprintln!(
        "{}storage: LineReader::find_line() lines                       : hit {:wide$}, miss {:wide$}, {:widep$.1}%",
        indent,
        summarylinereader.linereader_lines_hits,
        summarylinereader.linereader_lines_miss,
        percent,
        wide = wide,
        widep = WIDEP,
    );
    // LineReader::_find_line_lru_cache
    percent =
        percent64(&summarylinereader.linereader_find_line_lru_cache_hit, &summarylinereader.linereader_find_line_lru_cache_miss);
    eprintln!(
        "{}caching: LineReader::find_line() LRU cache                   : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarylinereader.linereader_find_line_lru_cache_hit,
        summarylinereader.linereader_find_line_lru_cache_miss,
        percent,
        summarylinereader.linereader_find_line_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
}

/// Print information about caching and optimizations for
/// `SummarySyslineReader`.
fn print_cache_stats_summarysyslinereader(
    summarysyslinereader: &SummarySyslineReader,
    indent: &str,
    wide: usize,
) {
    // SyslineReader
    // SyslineReader::get_boxptrs
    eprintln!(
        "{}copying: SyslineReader::get_boxptrs()                        : sgl {:wide$}, dbl  {:wide$}, mult {:wide$}",
        indent,
        summarysyslinereader.syslinereader_get_boxptrs_singleptr,
        summarysyslinereader.syslinereader_get_boxptrs_doubleptr,
        summarysyslinereader.syslinereader_get_boxptrs_multiptr,
        wide = wide,
    );
    // SyslineReader::syslines
    let mut percent = percent64(&summarysyslinereader.syslinereader_syslines_hit, &summarysyslinereader.syslinereader_syslines_miss);
    eprintln!(
        "{}storage: SyslineReader::find_sysline() syslines              : hit {:wide$}, miss {:wide$}, {:widep$.1}%",
        indent,
        summarysyslinereader.syslinereader_syslines_hit,
        summarysyslinereader.syslinereader_syslines_miss,
        percent,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::_syslines_by_range
    percent =
        percent64(&summarysyslinereader.syslinereader_syslines_by_range_hit, &summarysyslinereader.syslinereader_syslines_by_range_miss);
    eprintln!(
        "{}caching: SyslineReader::find_sysline() syslines_by_range_map : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_syslines_by_range_hit,
        summarysyslinereader.syslinereader_syslines_by_range_miss,
        percent,
        summarysyslinereader.syslinereader_syslines_by_range_put,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::_find_sysline_lru_cache
    percent = percent64(
        &summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
        &summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::find_sysline() LRU cache             : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_hit,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_miss,
        percent,
        summarysyslinereader.syslinereader_find_sysline_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::_parse_datetime_in_line_lru_cache
    percent = percent64(
        &summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
        &summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
    );
    eprintln!(
        "{}caching: SyslineReader::parse_datetime_in_line() LRU cache   : hit {:wide$}, miss {:wide$}, {:widep$.1}%, put {:wide$}",
        indent,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_hit,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_miss,
        percent,
        summarysyslinereader.syslinereader_parse_datetime_in_line_lru_cache_put,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::ezcheck12
    percent = percent64(
        &summarysyslinereader.syslinereader_ezcheck12_hit,
        &summarysyslinereader.syslinereader_ezcheck12_miss,
    );
    eprintln!(
        "{}optimize:SyslineReader::ezcheck12                            : hit {:wide$}, miss {:wide$}, {:widep$.1}%, largest skipped {}",
        indent,
        summarysyslinereader.syslinereader_ezcheck12_hit,
        summarysyslinereader.syslinereader_ezcheck12_miss,
        percent,
        summarysyslinereader.syslinereader_ezcheck12_hit_max,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::ezcheckd2
    percent = percent64(
        &summarysyslinereader.syslinereader_ezcheckd2_hit,
        &summarysyslinereader.syslinereader_ezcheckd2_miss,
    );
    eprintln!(
        "{}optimize:SyslineReader::ezcheckd2                            : hit {:wide$}, miss {:wide$}, {:widep$.1}%, largest skipped {}",
        indent,
        summarysyslinereader.syslinereader_ezcheckd2_hit,
        summarysyslinereader.syslinereader_ezcheckd2_miss,
        percent,
        summarysyslinereader.syslinereader_ezcheckd2_hit_max,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::ezcheck12d2
    percent = percent64(
        &summarysyslinereader.syslinereader_ezcheck12d2_hit,
        &summarysyslinereader.syslinereader_ezcheck12d2_miss,
    );
    eprintln!(
        "{}optimize:SyslineReader::ezcheck12d2                          : hit {:wide$}, miss {:wide$}, {:widep$.1}%, largest skipped {}",
        indent,
        summarysyslinereader.syslinereader_ezcheck12d2_hit,
        summarysyslinereader.syslinereader_ezcheck12d2_miss,
        percent,
        summarysyslinereader.syslinereader_ezcheck12d2_hit_max,
        wide = wide,
        widep = WIDEP,
    );
    // SyslineReader::regex_captures_attempted
    eprintln!(
        "{}process: regex captures attempted                            : {:?}",
        indent,
        summarysyslinereader.syslinereader_regex_captures_attempted,
    );
}

/// Print information about caching and optimizations for
/// `SummaryFixedStructReader`.
fn print_cache_stats_summaryfixedstructreader(
    summaryfixedstructreader: &SummaryFixedStructReader,
    indent: &str,
    wide: usize,
) {
    let percent = percent64(
        &summaryfixedstructreader.fixedstructreader_utmp_entries_hit,
        &summaryfixedstructreader.fixedstructreader_utmp_entries_miss,
    );
    eprintln!(
        "{}storage: FixedStructReader::find_entry()                     : hit {:wide$}, miss {:wide$}, {:widep$.1}%",
        indent,
        summaryfixedstructreader.fixedstructreader_utmp_entries_hit,
        summaryfixedstructreader.fixedstructreader_utmp_entries_miss,
        percent,
        wide = wide,
        widep = WIDEP,
    );
}

/// Print the various (optional) `Summary` caching and storage statistics
/// (multiple lines).
fn print_cache_stats(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    if summary_opt.is_none() {
        return;
    }
    let summary: &Summary = match summary_opt.as_ref() {
        Some(summary_) => summary_,
        None => {
            e_err!("unexpected None from match summary_opt");
            return;
        }
    };
    // `Dummy` may occur for files without adequate read permissions
    if summary.readerdata.is_dummy() {
        return;
    }
    let wide: usize = summary
        .max_hit_miss()
        .to_string()
        .len();
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            summaryblockreader,
            summarylinereader,
            summarysyslinereader,
            _summarysyslogprocessor,
        )) => {
            eprintln!("{}Processing Stores:", OPT_SUMMARY_PRINT_INDENT1);
            print_cache_stats_summaryblockreader(
                summaryblockreader,
                color_choice,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summarylinereader(
                summarylinereader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summarysyslinereader(
                summarysyslinereader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
        }
        SummaryReaderData::FixedStruct((
            summaryblockreader,
            summaryfixedstructreader,
        )) => {
            eprintln!("{}Processing Stores:", OPT_SUMMARY_PRINT_INDENT1);
            print_cache_stats_summaryfixedstructreader(
                summaryfixedstructreader,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
            print_cache_stats_summaryblockreader(
                summaryblockreader,
                color_choice,
                OPT_SUMMARY_PRINT_INDENT2,
                wide,
            );
        }
        SummaryReaderData::Etvx(_summaryevtxreader) => {}
        SummaryReaderData::Journal(_summaryjournalreader) => {}
        SummaryReaderData::PyEvent(_summarypyeventreader) => {}
        SummaryReaderData::Dummy => panic!("Unexpected SummaryReaderData::Dummy"),
    }
}

/// Print the (optional) various `Summary` drop error statistics
/// (multiple lines).
fn print_drop_stats(summary_opt: &SummaryOpt) {
    let summary: &Summary = match summary_opt {
        Some(summary) => summary,
        None => {
            de_wrn!("summary_opt is None");

            return;
        }
    };
    if summary.readerdata.is_dummy() {
        de_wrn!("summary.readerdata.is_dummy()");

        return;
    }
    // force early return for Evtx or Journal
    // the `EvtxReader` and `JournalReader` do not use `BlockReader`
    match summary.filetype {
        None => debug_panic!("unexpected None for summary.filetype"),
        Some(filetype_) => match filetype_ {
            FileType::Etl { .. }
            | FileType::Evtx { .. }
            | FileType::Journal { .. }
            | FileType::Odl { .. } => {
                return;
            }
            FileType::FixedStruct { .. }
            | FileType::Text { .. } => {}
            FileType::Unparsable => {
                debug_panic!("unexpected Unparsable for summary.filetype");
            }
        },
    }
    eprintln!("{}Processing Drops:", OPT_SUMMARY_PRINT_INDENT1);
    let wide: usize = summary
        .max_drop()
        .to_string()
        .len();
    match summary.blockreader() {
        Some(summaryblockreader) => {
            eprintln!(
                "{}streaming: BlockReader::drop_block()      : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summaryblockreader.blockreader_blocks_dropped_ok,
                summaryblockreader.blockreader_blocks_dropped_err,
                wide = wide,
            );
        }
        None => {}
    }
    match &summary.readerdata {
        SummaryReaderData::Syslog((
            _summaryblockreader,
            summarylinereader,
            summarysyslinereader,
            _summarysyslogreader,
        )) => {
            eprintln!(
                "{}streaming: LineReader::drop_line()        : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summarylinereader.linereader_drop_line_ok,
                summarylinereader.linereader_drop_line_errors,
                wide = wide,
            );
            eprintln!(
                "{}streaming: SyslineReader::drop_sysline()  : Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summarysyslinereader.syslinereader_drop_sysline_ok,
                summarysyslinereader.syslinereader_drop_sysline_errors,
                wide = wide,
            );
        }
        SummaryReaderData::FixedStruct(
            (
                _summaryblockreader,
                summaryfixedstructreader,
            )
        ) => {
            eprintln!(
                "{}streaming: FixedStructReader::drop_entry(): Ok {:wide$}, Err {:wide$}",
                OPT_SUMMARY_PRINT_INDENT2,
                summaryfixedstructreader.fixedstructreader_drop_entry_ok,
                summaryfixedstructreader.fixedstructreader_drop_entry_errors,
                wide = wide,
            );
        }
        SummaryReaderData::Etvx(..) => debug_panic!("Unexpected SummaryReaderData::Etvx"),
        SummaryReaderData::Journal(..) => debug_panic!("Unexpected SummaryReaderData::Journal"),
        SummaryReaderData::PyEvent(..) => debug_panic!("Unexpected SummaryReaderData::PyEvent"),
        SummaryReaderData::Dummy => debug_panic!("Unexpected SummaryReaderData::Dummy"),
    }
}

/// Print the [`Summary.error`], if any (one line).
///
/// [`Summary.error`]: s4lib::readers::summary::Summary
fn print_error_summary(
    summary_opt: &SummaryOpt,
    color_choice: &ColorChoice,
) {
    match summary_opt.as_ref() {
        Some(summary_) => match &summary_.error {
            Some(err_string) => {
                eprint!("{}Error:", OPT_SUMMARY_PRINT_INDENT1);
                if err_string.contains("\n") {
                    eprintln!();
                } else {
                    eprint!(" ");
                }
                #[allow(clippy::single_match)]
                print_colored_stderr(
                    COLOR_ERROR,
                    Some(*color_choice),
                    err_string.as_bytes(),
                );
                eprintln!();
            }
            None => {}
        },
        None => {}
    }
}

/// For one file, print the (optional) `Summary` and (optional) `SummaryPrinted`
/// (multiple lines).
#[allow(clippy::too_many_arguments)]
fn print_file_summary(
    path: &FPath,
    modified_time: &DateTimeLOpt,
    file_processing_result: Option<&FileProcessingResultBlockZero>,
    filetype: &FileType,
    logmessagetype: &LogMessageType,
    summary_opt: &SummaryOpt,
    summary_print_opt: &SummaryPrintedOpt,
    color: &Color,
    color_choice: &ColorChoice,
) {
    eprintln!();

    print_file_about(
        path,
        modified_time,
        file_processing_result,
        filetype,
        logmessagetype,
        summary_opt,
        color,
        color_choice,
    );
    // do not print any summary numbers for empty files (they should all be zero)
    if let Some(result) = file_processing_result {
        match result {
            FileProcessingResultBlockZero::FileErrEmpty => {
                return;
            }
            FileProcessingResultBlockZero::FileErrStub => {
                debug_panic!("result is FileErrStub");
                return;
            }
            _ => {}
        }
    }
    print_summary_opt_printed(summary_print_opt, summary_opt, color_choice);
    print_summary_opt_processed(summary_opt, color_choice);
    if OPT_SUMMARY_PRINT_CACHE_STATS {
        print_cache_stats(summary_opt, color_choice);
    }
    if OPT_SUMMARY_PRINT_DROP_STATS {
        print_drop_stats(summary_opt);
    }
    print_error_summary(summary_opt, color_choice);
}

/// Printing for CLI option `--summary`. Print each files'
/// [`Summary`] and [`SummaryPrinted`].
///
/// Helper function to function `processing_loop`.
#[allow(clippy::too_many_arguments)]
fn print_all_files_summaries(
    map_pathid_path: &MapPathIdToFPath,
    map_pathid_modified_time: &MapPathIdToModifiedTime,
    map_pathid_file_processing_result: &MapPathIdToFileProcessingResultBlockZero,
    map_pathid_filetype: &MapPathIdToFileType,
    map_pathid_logmessagetype: &MapPathIdToLogMessageType,
    map_pathid_color: &MapPathIdToColor,
    map_pathid_summary: &mut MapPathIdSummary,
    map_pathid_sumpr: &mut MapPathIdSummaryPrint,
    color_choice: &ColorChoice,
    color_default: &Color,
) {
    if map_pathid_path.is_empty() {
        return;
    }
    eprintln!("\nFiles:");
    for (pathid, path) in map_pathid_path.iter() {
        let color: &Color = map_pathid_color
            .get(pathid)
            .unwrap_or_else(
                || {
                    debug_panic!("color not found for PathID {:?} (path {:?})", pathid, path);

                   color_default
                }
            );
        let modified_time: &DateTimeLOpt = map_pathid_modified_time.get(pathid)
            .unwrap_or_else(
                || {
                    debug_panic!("modified_time not found for PathID {:?} (path {:?})", pathid, path);

                    &DateTimeLOpt::None
                }
            );
        let file_processing_result = map_pathid_file_processing_result.get(pathid);
        let filetype: &FileType = map_pathid_filetype
            .get(pathid)
            .unwrap_or_else(
                || {
                    debug_panic!("filetype not found for PathID {:?} (path {:?})", pathid, path);

                    &FileType::Unparsable
                }
            );
        let logmessagetype: &LogMessageType = map_pathid_logmessagetype
            .get(pathid)
            .unwrap_or_else(
                || {
                    debug_panic!("logmessagetype not found for PathID {:?} (path {:?})", pathid, path);

                    &LogMessageType::Sysline
                }
            );
        let summary_opt: SummaryOpt = map_pathid_summary.remove(pathid);
        if summary_opt.is_none() {
            debug_panic!("summary_opt is None for PathID {:?} (path {:?})", pathid, path);
        }
        let summary_print_opt: SummaryPrintedOpt = map_pathid_sumpr.remove(pathid);
        if summary_print_opt.is_none() {
            de_wrn!("summary_print_opt is None for PathID {:?} (path {:?})", pathid, path);
        }
        print_file_summary(
            path,
            modified_time,
            file_processing_result,
            filetype,
            logmessagetype,
            &summary_opt,
            &summary_print_opt,
            color,
            color_choice,
        );
    }
    eprintln!();
}

/// Printing for CLI option `--summary`; print an entry for invalid files
/// (one line).
///
/// Helper function to function `processing_loop`.
fn print_files_processpathresult(
    map_pathid_result: &MapPathIdToProcessPathResultOrdered,
    color_choice: &ColorChoice,
    color_default: &Color,
    color_error: &Color,
) {
    if map_pathid_result.is_empty() {
        return;
    }
    // local helper
    fn print_(
        buffer: String,
        color_choice: &ColorChoice,
        color: &Color,
    ) {
        print_colored_stderr(
            *color,
            Some(*color_choice),
            buffer.as_bytes(),
        );
    }

    for (_pathid, result) in map_pathid_result.iter() {
        match result {
            ProcessPathResult::FileValid(path, _filetype) => {
                // defo!("FileValid");
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
            }
            ProcessPathResult::FileErrEmpty(path, filetype) => {
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                print_("(empty file)".to_string(), color_choice, color_error);
                print_(format!(" {}", filetype), color_choice, color_default);
            }
            ProcessPathResult::FileErrTooSmall(path, filetype, len) => {
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                print_("(too small)".to_string(), color_choice, color_error);
                print_(format!(" ({} bytes) {}", len, filetype), color_choice, color_default);
            }
            ProcessPathResult::FileErrNoPermissions(path) => {
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                print_("(no permissions)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotSupported(path, message) => {
                // defo!("FileErrNotSupported 1");
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                // defo!("FileErrNotSupported 2");
                match message {
                    Some(m) => {
                        print_(format!("({})", m), color_choice, color_error);
                    }
                    None => {
                        print_("(not supported)".to_string(), color_choice, color_error);
                    }
                }
            }
            ProcessPathResult::FileErrNotAFile(path) => {
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                print_("(not a file)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrNotExist(path) => {
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                print_("(does not exist)".to_string(), color_choice, color_error);
            }
            ProcessPathResult::FileErrLoadingLibrary(path, libname, filetype) => {
                print_(format!("File: {} {:?} ", fpath_to_prependpath(path), filetype), color_choice, color_default);
                print_(format!("(failed to load shared library {:?})", libname), color_choice, color_error);
            }
            ProcessPathResult::FileErr(path, message) => {
                print_(format!("File: {} ", fpath_to_prependpath(path)), color_choice, color_default);
                print_(format!("({})", message), color_choice, color_error);
            }
        }
        eprintln!();
    }
    eprintln!();
}
