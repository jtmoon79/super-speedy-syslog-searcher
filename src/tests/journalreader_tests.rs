// src/tests/journalreader_tests.rs

//! tests for `journalreader.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::io::ErrorKind;
use std::path::Path;
use std::ops::Range;

use crate::common::{
    Count,
    FileSz,
    FPath,
    FileType,
    LogMessageType,
};
use crate::data::datetime::{
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    DateTimeLOpt,
};
use crate::data::journal::{
    EpochMicroseconds,
    EpochMicrosecondsOpt,
};
use crate::libload::systemd_dlopen2::{
    LoadLibraryError,
    load_library_systemd,
};
use crate::readers::helpers::path_to_fpath;
use crate::readers::summary::SummaryReaderData;
use crate::readers::journalreader::{
    JournalOutput,
    JournalReader,
    em_after_or_before,
    em_pass_filters,
    errno_to_errorkind,
    Errno,
    ResultNext,
    ForceErrorRangeOpt,
};
use crate::tests::common::{
    FO_0,
    TS_1,
    NTF_JOURNAL_EMPTY_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JOURNAL_FILE_RHE_91_SYSTEM_FPATH,
    JOURNAL_FILE_RHE_91_SYSTEM_EVENT_COUNT,
    JOURNAL_FILE_RHE_91_SYSTEM_EVENT_FILESZ,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY_FIRST_DT,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY_LAST_DT,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORT,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTPRECISE,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTISO,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTISOPRECISE,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTFULL,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTMONOTONIC,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTUNIX,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_VERBOSE,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_EXPORT,
    JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_CAT,
    JOURNAL_FILE_UBUNTU_22_SYSTEM_PATH,
    JOURNAL_FILE_UBUNTU_22_SYSTEM_FPATH,
    JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_FILESZ,
    JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT,
    JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT,
};

use bstr::ByteSlice;
use ::criterion::black_box;
use ::test_case::test_case;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 - 1),
    Result_Filter_DateTime1::OccursAtOrAfter;
    "TS_1 OccursAtOrAfter"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 + 1),
    Result_Filter_DateTime1::OccursBefore;
    "TS_1 OccursBefore"
)]
fn test_em_after_or_before(
    em: EpochMicroseconds,
    em_filter: EpochMicrosecondsOpt,
    expect_result: Result_Filter_DateTime1,
) {
    let result = em_after_or_before(&em, &em_filter);
    assert_eq!(
        result, expect_result,
        "result {:?}, expect_result {:?}",
        result, expect_result
    );
}

#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 - 1),
    EpochMicrosecondsOpt::Some(*TS_1 + 1),
    Result_Filter_DateTime2::InRange;
    "Some Some InRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 + 1),
    EpochMicrosecondsOpt::Some(*TS_1 + 2),
    Result_Filter_DateTime2::BeforeRange;
    "Some Some BeforeRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 - 2),
    EpochMicrosecondsOpt::Some(*TS_1 - 1),
    Result_Filter_DateTime2::AfterRange;
    "Some Some AfterRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 - 1),
    EpochMicrosecondsOpt::None,
    Result_Filter_DateTime2::InRange;
    "Some None InRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::Some(*TS_1 + 1),
    EpochMicrosecondsOpt::None,
    Result_Filter_DateTime2::BeforeRange;
    "Some None BeforeRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::None,
    EpochMicrosecondsOpt::Some(*TS_1 - 1),
    Result_Filter_DateTime2::AfterRange;
    "None Some AfterRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::None,
    EpochMicrosecondsOpt::Some(*TS_1 + 1),
    Result_Filter_DateTime2::InRange;
    "None Some InRange"
)]
#[test_case(
    *TS_1,
    EpochMicrosecondsOpt::None,
    EpochMicrosecondsOpt::None,
    Result_Filter_DateTime2::InRange;
    "None None InRange"
)]
fn test_em_pass_filters(
    em: EpochMicroseconds,
    em_filter_after: EpochMicrosecondsOpt,
    em_filter_before: EpochMicrosecondsOpt,
    expect_result: Result_Filter_DateTime2,
) {
    let result = em_pass_filters(
        &em,
        &em_filter_after,
        &em_filter_before,
    );
    assert_eq!(
        result, expect_result,
        "result {:?}, expect_result {:?}",
        result, expect_result
    );
}

#[test]
fn test_errno_to_errorkind() {
    let e = Errno::EACCES;
    let ek = errno_to_errorkind(&e);
    assert_eq!(ErrorKind::PermissionDenied, ek);
}

/// test creating a new `JournalReader`
#[test_case(&NTF_JOURNAL_EMPTY_FPATH, false)]
#[test_case(&FPath::from("BAD PATH"), false)]
#[test_case(&*JOURNAL_FILE_RHE_91_SYSTEM_FPATH, true)]
#[test_case(&*JOURNAL_FILE_UBUNTU_22_SYSTEM_FPATH, true)]
fn test_JournalReader_new_(path: &FPath, ok: bool) {
    assert!(matches!(load_library_systemd(), LoadLibraryError::Ok));
    match JournalReader::new(
        path.clone(),
        JournalOutput::Short,
        *FO_0,
    ) {
        Ok(_) => {
            assert!(ok, "JournalReader::new({:?}) should have failed", path);
        }
        Err(_err) => {
            assert!(!ok, "JournalReader::new({:?}) should have succeeded", path);
        }
    }
}

/// test the output of the first entry returned by `JournalReader::next()`
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::Short,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORT;
    "RHE91 short"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::ShortPrecise,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTPRECISE;
    "RHE91 shortprecise"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::ShortIso,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTISO;
    "RHE91 shortiso"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::ShortIsoPrecise,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTISOPRECISE;
    "RHE91 shortisoprecise"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::ShortFull,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTFULL;
    "RHE91 shortfull"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::ShortMonotonic,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTMONOTONIC;
    "RHE91 shortmonotonic"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::ShortUnix,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_SHORTUNIX;
    "RHE91 shortunix"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::Verbose,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_VERBOSE;
    "RHE91 verbose"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::Export,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_EXPORT;
    "RHE91 export"
)]
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    JournalOutput::Cat,
    &*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY1_CAT;
    "RHE91 cat"
)]
fn test_JournalReader_entry1_output(
    path: &Path,
    journal_output: JournalOutput,
    expect_data: &str,
) {
    assert!(matches!(load_library_systemd(), LoadLibraryError::Ok));
    let fpath = path_to_fpath(path);
    let mut journalreader = JournalReader::new(
        fpath,
        journal_output,
        *FO_0,
    ).unwrap();
    match journalreader.analyze(&None) {
        Ok(_) => {}
        Err(err) => {
            panic!("journalreader.analyze() failed: {}", err);
        }
    }
    let result = journalreader.next(&None);
    let je = match result {
        ResultNext::Found(je) => {
            je
        }
        ResultNext::Done => {
            panic!("journalreader.next() failed: Done");
        }
        ResultNext::Err(err) => {
            panic!("journalreader.next() failed: {}", err);
        }
        ResultNext::ErrIgnore(err) => {
            panic!("journalreader.next() failed (ErrIgnore): {}", err);
        }
    };
    assert_eq!(je.as_bytes(), expect_data.as_bytes(),
        "\nje.as_bytes():\n{:?}\nexpect_data:\n{:?}\n",
        je.as_bytes().to_str(), expect_data
    );

}

/// test the summary statistics after processing the entire file
#[test_case(
    &*JOURNAL_FILE_RHE_91_SYSTEM_PATH,
    *JOURNAL_FILE_RHE_91_SYSTEM_EVENT_FILESZ,
    *JOURNAL_FILE_RHE_91_SYSTEM_EVENT_COUNT,
    *JOURNAL_FILE_RHE_91_SYSTEM_EVENT_COUNT,
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY_LAST_DT),
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_RHE_91_SYSTEM_ENTRY_LAST_DT),
    55126,
    0,
    ForceErrorRangeOpt::None;
    "RHE91"
)]
#[test_case(
    &*JOURNAL_FILE_UBUNTU_22_SYSTEM_PATH,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_FILESZ,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT),
    113,
    0,
    ForceErrorRangeOpt::None;
    "UBUNTU22"
)]
#[test_case(
    &*JOURNAL_FILE_UBUNTU_22_SYSTEM_PATH,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_FILESZ,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT),
    115,
    2,
    ForceErrorRangeOpt::Some(Range { start: 45, end: 46 });
    "UBUNTU22 errors 45 46"
)]
#[test_case(
    &*JOURNAL_FILE_UBUNTU_22_SYSTEM_PATH,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_FILESZ,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    *JOURNAL_FILE_UBUNTU_22_SYSTEM_EVENT_COUNT,
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_FIRST_DT),
    Some(*JOURNAL_FILE_UBUNTU_22_SYSTEM_ENTRY_LAST_DT),
    118,
    5,
    ForceErrorRangeOpt::Some(Range { start: 110, end: 114 });
    "UBUNTU22 errors 110 114"
)]
fn test_JournalReader_next_summary(
    path: &Path,
    filesz: FileSz,
    events_processed: Count,
    events_accepted: Count,
    datetime_first_accepted: DateTimeLOpt,
    datetime_last_accepted: DateTimeLOpt,
    datetime_first_processed: DateTimeLOpt,
    datetime_last_processed: DateTimeLOpt,
    api_calls: Count,
    api_call_errors: Count,
    range_error_opt: ForceErrorRangeOpt,
) {
    assert!(matches!(load_library_systemd(), LoadLibraryError::Ok));
    let fpath = path_to_fpath(path);
    let fpath2 = fpath.clone();
    let mut journalreader = JournalReader::new(
        fpath,
        JournalOutput::Short,
        *FO_0,
    ).unwrap();
    match journalreader.analyze(&None) {
        Ok(_) => {}
        Err(err) => {
            panic!("journalreader.analyze() failed: {}", err);
        }
    }
    journalreader.force_error_range_opt = range_error_opt;
    loop {
        let result = journalreader.next(&None);
        match result {
            ResultNext::Found(je) => {
                black_box(je);
            }
            ResultNext::Done => {
                break;
            }
            ResultNext::Err(err) => {
                panic!("journalreader.next() failed: {}", err);
            }
            ResultNext::ErrIgnore(err) => {
                panic!("journalreader.next() failed (ErrIgnore): {}", err);
            }
        }
    }

    // assert JournalReader
    assert_eq!(journalreader.path(), &fpath2, "fpath");
    assert_eq!(journalreader.filesz(), filesz, "filesz");
    assert_eq!(journalreader.dt_first_processed(), datetime_first_processed,
        "dt_first_processed");
    assert_eq!(journalreader.dt_last_processed(), datetime_last_processed,
        "dt_last_processed");
    assert_eq!(journalreader.dt_first_accepted(), datetime_first_accepted,
        "dt_first_accepted");
    assert_eq!(journalreader.dt_last_accepted(), datetime_last_accepted,
        "dt_last_accepted");

    // assert SummaryJournalReader
    let summary = journalreader.summary();
    assert_eq!(summary.journalreader_events_processed, events_processed,
        "summary.count_events_processed");
    assert_eq!(summary.journalreader_events_accepted, events_accepted,
        "summary.count_events_accepted");
    assert_eq!(summary.journalreader_filesz, filesz, "summary.filesz");
    assert_eq!(summary.journalreader_datetime_first_accepted, datetime_first_accepted,
        "summary.datetime_first_accepted");
    assert_eq!(summary.journalreader_datetime_last_accepted, datetime_last_accepted,
        "summary.datetime_last_accepted");
    assert_eq!(summary.journalreader_datetime_first_processed, datetime_first_processed,
        "summary.datetime_first_processed");
    assert_eq!(summary.journalreader_datetime_last_processed, datetime_last_processed,
        "summary.datetime_last_processed");
    assert_eq!(summary.journalreader_api_calls, api_calls,
        "summary.api_calls");
    assert_eq!(summary.journalreader_api_call_errors, api_call_errors,
        "summary.api_call_errors");

    // assert Summary
    let summary_c = journalreader.summary_complete();
    assert_eq!(summary_c.filetype, FileType::Journal, "summary_c.filetype");
    assert_eq!(summary_c.logmessagetype, LogMessageType::Journal,
        "summary_c.logmessagetype");
    assert!(summary_c.blockreader().is_none());
    assert_eq!(summary_c.datetime_first(), &datetime_first_accepted);
    assert_eq!(summary_c.datetime_last(), &datetime_last_accepted);
    assert_eq!(summary_c.max_drop(), 0);
    assert_eq!(summary_c.max_hit_miss(), events_accepted);
    match summary_c.readerdata {
        SummaryReaderData::Journal(_summary_journal) => {}
        _ => {
            panic!("summary_c.readerdata() should be SummaryReaderData::Journal");
        }
    }
}
