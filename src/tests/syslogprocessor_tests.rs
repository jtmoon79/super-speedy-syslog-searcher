// src/tests/syslogprocessor_tests.rs

//! tests for `syslogprocessor.rs`

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::common::{Count, FPath, FileOffset};
use crate::data::datetime::{
    datetime_parse_from_str,
    DateTimeL,
    DateTimeLOpt,
    DateTimePattern_str,
    DateTimeParseDatasCompiledCount,
    FixedOffset,
    SystemTime,
    seconds_to_systemtime,
    ymdhms0,
};
use crate::data::sysline::SyslineP;
use crate::debug::helpers::{
    create_temp_file,
    create_temp_file_data,
    create_temp_file_no_permissions,
    ntf_fpath,
    NamedTempFile,
};
use crate::readers::blockreader::{
    BlockSz,
    SummaryBlockReader,
};
use crate::readers::helpers::path_to_fpath;
use crate::readers::filepreprocessor::{
    fpath_to_filetype,
    PathToFiletypeResult,
};
use crate::readers::linereader::SummaryLineReader;
use crate::readers::summary::SummaryReaderData;
use crate::readers::syslinereader::{
    DateTimePatternCounts,
    SummarySyslineReader,
    ResultS3SyslineFind,
};
use crate::readers::syslogprocessor::{
    FileProcessingResultBlockZero,
    SummarySyslogProcessor,
    SyslogProcessor,
    SYSLOG_SZ_MAX_BSZ,
};
use crate::tests::common::{
    eprint_file,
    eprint_file_blocks,
    FILETYPE_UTF8,
    FO_0,
    NTF_GZ_EMPTY_FPATH,
    NTF_LOG_EMPTY_FPATH,
    NTF_GZ_8BYTE_FPATH,
};

use ::const_format::concatcp;
use ::filetime;
use ::lazy_static::lazy_static;
use ::more_asserts::{assert_ge, assert_gt};
#[allow(unused_imports)]
use ::si_trace_print::printers::{defn, defo, defx, defñ};
use ::test_case::test_case;


const SZ: BlockSz = SyslogProcessor::BLOCKSZ_MIN;

//
// NTF1S_A
//

// a one line file with a short datetime pattern
// 2019-03-01 16:56

const NTF1S_A_DATA_LINE0: &str = "jan 1 12:34:56";

const NTF1S_A_DATA: &str = concatcp!(
    NTF1S_A_DATA_LINE0,
);

/// Unix epoch time for time `NTF1S_A_DATA_LINE0` year 2001 at UTC
const NTF1S_A_MTIME_UNIXEPOCH: i64 = 978381296;

//
// NTF1S_B
//

// a one line file with a short datetime pattern
// 2019-03-01 16:56

const NTF1S_B_DATA_LINE0: &str = "jan 1 12:34:56\n";

const NTF1S_B_DATA: &str = concatcp!(
    NTF1S_B_DATA_LINE0,
);

/// Unix epoch time for time `NTF1S_B_DATA_LINE0` year 2001 at UTC
const NTF1S_B_MTIME_UNIXEPOCH: i64 = 978381296;

//
// NTF1S_C - deliberately short data
//

const NTF1S_C_DATA_LINE0: &str = "ABC";

const NTF1S_C_DATA: &str = concatcp!(
    NTF1S_C_DATA_LINE0,
);

/// Unix epoch time for time `NTF1S_C_DATA_LINE0` year 2001 at UTC
const NTF1S_C_MTIME_UNIXEPOCH: i64 = 978381296;

//
// NTF2S_A
//

// a one line file with a short datetime pattern
// 2019-03-01 16:56

const NTF2S_A_DATA_LINE0: &str = "jan 1 12:34:56\n";
const NTF2S_A_DATA_LINE1: &str = "jan 2 23:45:60\n";

const NTF2S_A_DATA: &str = concatcp!(
    NTF2S_A_DATA_LINE0,
    NTF2S_A_DATA_LINE1,
);

/// Unix epoch time for time `NTF1S_B_DATA_LINE0` year 2001 at UTC
const NTF2S_A_MTIME_UNIXEPOCH: i64 = 978381296;

//
// NTF5
//

// the five lines of data that makes up file `NTF5`
const NTF5_DATA_LINE0: &str = "Jan 1 01:00:11 5a\n";
const NTF5_DATA_LINE1: &str = "Feb 29 02:00:22 5b\n";
const NTF5_DATA_LINE2: &str = "Mar 3 03:00:33 5c\n";
const NTF5_DATA_LINE3: &str = "Apr 4 04:00:44 5d\n";
const NTF5_DATA_LINE4: &str = "May 5 05:00:55 5e\n";

/// Unix epoch time for time `NTF5_DATA_LINE4` at UTC
const NTF5_MTIME_UNIXEPOCH: i64 = 957502855;

const NTF5_DATA: &str = concatcp!(
    NTF5_DATA_LINE0,
    NTF5_DATA_LINE1,
    NTF5_DATA_LINE2,
    NTF5_DATA_LINE3,
    NTF5_DATA_LINE4,
);

const NTF5_LINE2_DATETIME_STR: &str = "Mar 3 03:00:00 +0000";
const NTF5_LINE2_DATETIME_PATTERN: &DateTimePattern_str = "%b %e %H:%M:%S %z";

//
// NTF5X4
//

// the five lines of data that makes up file `NTF5X4`
// the first line is different datetime format from the others, see Issue #74
const NTF5X4_DATA_LINE0: &str = "Jan 1 01:00:11 5X4a\n";
const NTF5X4_DATA_LINE1: &str = "2000-02-12T02:00:22 5X4b\n";
const NTF5X4_DATA_LINE2: &str = "2000-03-13T03:00:33 5X4c\n";
const NTF5X4_DATA_LINE3: &str = "2000-04-14T04:00:44 5X4d\n";
const NTF5X4_DATA_LINE4: &str = "2000-05-15T05:00:55 5X4e\n";

/// Unix epoch time for time `NTF5X4_DATA_LINE4` at UTC
const NTF5X4_MTIME_UNIXEPOCH: i64 = 958392055;

const NTF5X4_DATA: &str = concatcp!(
    NTF5X4_DATA_LINE0,
    NTF5X4_DATA_LINE1,
    NTF5X4_DATA_LINE2,
    NTF5X4_DATA_LINE3,
    NTF5X4_DATA_LINE4,
);

//
// NTF3
//

const NTF3_DATA_LINE0: &str = "Jan 1 01:00:00 2000 A3\n";
const NTF3_DATA_LINE1: &str = "Feb 2 02:00:00 2000 B3\n";
const NTF3_DATA_LINE2: &str = "Mar 3 03:00:00 2000 C3\n";

const NTF3_DATA: &str = concatcp!(NTF3_DATA_LINE0, NTF3_DATA_LINE1, NTF3_DATA_LINE2,);

const NTF3_LINE1_DATETIME_STR: &str = "Feb 2 02:00:00 2000 +0000";
const NTF3_LINE1_DATETIME_PATTERN: &DateTimePattern_str = "%b %e %H:%M:%S %Y %z";

const NTF3_DATA_SYSLINES: [&str; 3] = [
    NTF3_DATA_LINE0,
    NTF3_DATA_LINE1,
    NTF3_DATA_LINE2,
];

//
// NTF9
//

// the nine lines of data that makes up file `NTF9`
const NTF9_DATA_LINE0: &str = "Jan 11 01:31:21 2000 9à\n";
const NTF9_DATA_LINE1: &str = "Feb 29 02:32:22 2000 9bb\n";
const NTF9_DATA_LINE2: &str = "Mar 13 03:33:23 2000 9ccc\n";
const NTF9_DATA_LINE3: &str = "Apr 14 04:34:24 2000 9dddd\n";
const NTF9_DATA_LINE4: &str = "May 15 05:35:25 2000 9èèèèè\n";
const NTF9_DATA_LINE5: &str = "Jun 16 05:36:26 2000 9ffffff\n";
const NTF9_DATA_LINE6: &str = "Jul 17 05:37:27 2000 9ggggggg\n";
const NTF9_DATA_LINE7: &str = "Aug 18 05:38:28 2000 9hhhhhhhh\n";
const NTF9_DATA_LINE8: &str = "Sep 19 05:39:29 2000 9ììììììììì\n";
const NTF9_DATA_LINE9: &str = "Oct 20 05:39:30 2000 10λλλλλλλλλλ\n";
const NTF9_DATA_LINE10: &str = "Nov 21 05:39:31 2000 11ΜΜΜΜΜΜΜΜΜΜΜ\n";

const NTF9_DATA: &str = concatcp!(
    NTF9_DATA_LINE0,
    NTF9_DATA_LINE1,
    NTF9_DATA_LINE2,
    NTF9_DATA_LINE3,
    NTF9_DATA_LINE4,
    NTF9_DATA_LINE5,
    NTF9_DATA_LINE6,
    NTF9_DATA_LINE7,
    NTF9_DATA_LINE8,
    NTF9_DATA_LINE9,
    NTF9_DATA_LINE10,
);

const NTF9_DATA_LINE0_OFFSET: usize = 0;
const NTF9_DATA_LINE1_OFFSET: usize = NTF9_DATA_LINE0_OFFSET
    + NTF9_DATA_LINE0
        .as_bytes()
        .len();
const NTF9_DATA_LINE2_OFFSET: usize = NTF9_DATA_LINE1_OFFSET
    + NTF9_DATA_LINE1
        .as_bytes()
        .len();
#[allow(dead_code)]
const NTF9_DATA_LINE3_OFFSET: usize = NTF9_DATA_LINE2_OFFSET
    + NTF9_DATA_LINE2
        .as_bytes()
        .len();
const NTF9_BLOCKSZ_MIN: BlockSz = (NTF9_DATA_LINE2_OFFSET + NTF9_DATA_LINE2_OFFSET % 2) as BlockSz;

//
// NTF7_2
//

// seven syslines with two lines each that makes up file `NTF7_2`
const NTF7_2_DATA_LINE0: &str = "Jan 11 01:31:21 2000 6-3 à\n";
const NTF7_2_DATA_LINE1: &str = "ββββββββββββββββββββββββββ\n";
const NTF7_2_DATA_LINE2: &str = "Mar 13 03:33:23 2000 6-3 ccc\n";
const NTF7_2_DATA_LINE3: &str = "ΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔΔ\n";
const NTF7_2_DATA_LINE4: &str = "May 15 05:35:25 2000 6-3 èèèèè\n";
const NTF7_2_DATA_LINE5: &str = "ζζζζζζζζζζζζζζζζζζζζζζζζζζζζζζ\n";
const NTF7_2_DATA_LINE6: &str = "Jul 17 07:37:27 2000 6-3 ggggggg\n";
const NTF7_2_DATA_LINE7: &str = "ΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗΗ\n";
const NTF7_2_DATA_LINE8: &str = "Sep 19 09:39:29 2000 6-3 ììììììììì\n";
const NTF7_2_DATA_LINE9: &str = "ιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιι\n";
const NTF7_2_DATA_LINE10: &str = "Nov 21 11:41:41 2000 6-3 ììììììììì\n";
const NTF7_2_DATA_LINE11: &str = "ιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιιι\n";
const NTF7_2_DATA_LINE12: &str = "Jan 31 01:02:03 2001 6-3 KKKKKKKKKK\n";
const NTF7_2_DATA_LINE13: &str = "ΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛΛ\n";
const NTF7_2_DATA_LINE14: &str = "Feb 2 02:03:04 2001 6-3 mmmmmmmmmmmm\n";
const NTF7_2_DATA_LINE15: &str = "ηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηηη\n";

const NTF7_2_DATA: &str = concatcp!(
    NTF7_2_DATA_LINE0,
    NTF7_2_DATA_LINE1,
    NTF7_2_DATA_LINE2,
    NTF7_2_DATA_LINE3,
    NTF7_2_DATA_LINE4,
    NTF7_2_DATA_LINE5,
    NTF7_2_DATA_LINE6,
    NTF7_2_DATA_LINE7,
    NTF7_2_DATA_LINE8,
    NTF7_2_DATA_LINE9,
    NTF7_2_DATA_LINE10,
    NTF7_2_DATA_LINE11,
    NTF7_2_DATA_LINE12,
    NTF7_2_DATA_LINE13,
    NTF7_2_DATA_LINE14,
    NTF7_2_DATA_LINE15,
);

const NTF7_2_DATA_LINE0_OFFSET: usize = 0;
const NTF7_2_DATA_LINE1_OFFSET: usize = NTF7_2_DATA_LINE0_OFFSET
    + NTF7_2_DATA_LINE0
        .as_bytes()
        .len();
const NTF7_2_DATA_LINE2_OFFSET: usize = NTF7_2_DATA_LINE1_OFFSET
    + NTF7_2_DATA_LINE1
        .as_bytes()
        .len();
const NTF7_2_DATA_LINE3_OFFSET: usize = NTF7_2_DATA_LINE2_OFFSET
    + NTF7_2_DATA_LINE2
        .as_bytes()
        .len();
const NTF7_2_DATA_LINE4_OFFSET: usize = NTF7_2_DATA_LINE3_OFFSET
    + NTF7_2_DATA_LINE3
        .as_bytes()
        .len();
#[allow(dead_code)]
const NTF7_2_DATA_LINE5_OFFSET: usize = NTF7_2_DATA_LINE4_OFFSET
    + NTF7_2_DATA_LINE4
        .as_bytes()
        .len();
const NTF7_2_BLOCKSZ_MIN: BlockSz = (NTF7_2_DATA_LINE2_OFFSET + NTF7_2_DATA_LINE2_OFFSET % 2) as BlockSz;

#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE0: &str = concatcp!(NTF7_2_DATA_LINE0, NTF7_2_DATA_LINE1);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE1: &str = concatcp!(NTF7_2_DATA_LINE2, NTF7_2_DATA_LINE3);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE2: &str = concatcp!(NTF7_2_DATA_LINE4, NTF7_2_DATA_LINE5);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE3: &str = concatcp!(NTF7_2_DATA_LINE6, NTF7_2_DATA_LINE7);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE4: &str = concatcp!(NTF7_2_DATA_LINE8, NTF7_2_DATA_LINE9);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE5: &str = concatcp!(NTF7_2_DATA_LINE10, NTF7_2_DATA_LINE11);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE6: &str = concatcp!(NTF7_2_DATA_LINE12, NTF7_2_DATA_LINE13);
#[allow(dead_code)]
const NTF7_2_DATA_SYSLINE7: &str = concatcp!(NTF7_2_DATA_LINE14, NTF7_2_DATA_LINE15);

#[allow(dead_code)]
const NTF7_2_DATA_SYSLINES: [&str; 8] = [
    NTF7_2_DATA_SYSLINE0,
    NTF7_2_DATA_SYSLINE1,
    NTF7_2_DATA_SYSLINE2,
    NTF7_2_DATA_SYSLINE3,
    NTF7_2_DATA_SYSLINE4,
    NTF7_2_DATA_SYSLINE5,
    NTF7_2_DATA_SYSLINE6,
    NTF7_2_DATA_SYSLINE7,
];

//
// NTF0X12000
//

const NTF0X12000_DATA: &[u8; 12000] = &[0; 12000];

lazy_static! {
    static ref TIMEZONE_0: FixedOffset = FixedOffset::west_opt(0).unwrap();

    //
    // NTF1S_A
    //

    static ref NTF1S_A: NamedTempFile = {
        let ntf = create_temp_file(NTF1S_A_DATA);
        // set the file's modified time to `NTF1S_A_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF1S_A_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF1S_A_PATH: FPath = {
        ntf_fpath(&NTF1S_A)
    };

    // a `DateTimeL` instance three minutes after `NTF1S_A_DATA_LINE0`
    static ref NTF1S_A_DATA_LINE0_AFTER: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2001-01-01T12:37:56-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    //
    // NTF1S_B
    //

    static ref NTF1S_B: NamedTempFile = {
        let ntf = create_temp_file(NTF1S_B_DATA);
        // set the file's modified time to `NTF1S_B_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF1S_B_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF1S_B_PATH: FPath = {
        ntf_fpath(&NTF1S_B)
    };

    // a `DateTimeL` instance three minutes after `NTF1S_B_DATA_LINE0`
    static ref NTF1S_B_DATA_LINE0_AFTER: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2001-01-01T12:37:56-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    //
    // NTF1S_C
    //

    static ref NTF1S_C: NamedTempFile = {
        let ntf = create_temp_file(NTF1S_C_DATA);
        // set the file's modified time to `NTF1S_C_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF1S_C_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF1S_C_PATH: FPath = {
        ntf_fpath(&NTF1S_C)
    };

    //
    // NTF2S_A
    //

    static ref NTF2S_A: NamedTempFile = {
        let ntf = create_temp_file(NTF2S_A_DATA);
        // set the file's modified time to `NTF2S_A_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF2S_A_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF2S_A_PATH: FPath = {
        ntf_fpath(&NTF2S_A)
    };

    // a `DateTimeL` instance at `NTF2S_A_DATA_LINE1`
    static ref NTF2S_A_DATA_LINE1_AFTER: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2001-01-02T23:45:60-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    //
    // NTF5
    //

    // a `DateTimeL` instance a few hours before `NTF5_DATA_LINE2` and after
    // `NTF5_DATA_LINE1`
    static ref NTF5_DATA_LINE2_BEFORE: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2000-03-01T12:00:00-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    // a `DateTimeL` instance some hours after `NTF5_DATA_LINE4`
    static ref NTF5_DATA_LINE4_AFTER: DateTimeLOpt = {
        match DateTimeL::parse_from_rfc3339("2000-05-05T23:00:00-00:00") {
            Ok(dt) => Some(dt),
            Err(err) => panic!("Error parse_from_rfc3339 failed {:?}", err),
        }
    };

    static ref NTF5_LINE2_DATETIME: DateTimeL = {
        match datetime_parse_from_str(
            NTF5_LINE2_DATETIME_STR, NTF5_LINE2_DATETIME_PATTERN, true, &TIMEZONE_0
        ) {
            Some(dt) => dt,
            None => {
                panic!("bad parameters to datetime_parse_from_str for NTF5_LINE2_DATETIME_STR");
            }
        }
    };

    static ref NTF5: NamedTempFile = {
        let ntf = create_temp_file(NTF5_DATA);
        // set the file's modified time to `NTF5_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF5_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF5_PATH: FPath = {
        ntf_fpath(&NTF5)
    };

    //
    // NTF5X4
    //

    static ref NTF5X4: NamedTempFile = {
        let ntf = create_temp_file(NTF5X4_DATA);
        // set the file's modified time to `NTF5X4_MTIME_UNIXEPOCH`
        let mtime = filetime::FileTime::from_unix_time(NTF5X4_MTIME_UNIXEPOCH, 0);
        match filetime::set_file_mtime(ntf.path(), mtime) {
            Ok(_) => {},
            Err(err) => panic!("Error failed to set_file_mtime({:?}, {:?}) {:?}", ntf.path(), mtime, err),
        }

        ntf
    };

    static ref NTF5X4_PATH: FPath = {
        ntf_fpath(&NTF5X4)
    };

    //
    // NTF3
    //

    static ref NTF3_LINE1_DATETIME: DateTimeL = {
        match datetime_parse_from_str(
            NTF3_LINE1_DATETIME_STR, NTF3_LINE1_DATETIME_PATTERN, true, &TIMEZONE_0
        ) {
            Some(dt) => dt,
            None => {
                panic!("bad parameters to datetime_parse_from_str for NTF3_LINE1_DATETIME_STR");
            }
        }
    };

    static ref NTF3: NamedTempFile = {
        create_temp_file(NTF3_DATA)
    };

    static ref NTF3_PATH: FPath = {
        ntf_fpath(&NTF3)
    };

    //
    // NTF9
    //

    static ref NTF9: NamedTempFile = {
        create_temp_file(NTF9_DATA)
    };

    static ref NTF9_PATH: FPath = {
        ntf_fpath(&NTF9)
    };

    //
    // NTF7_2
    //

    static ref NTF7_2: NamedTempFile = {
        create_temp_file(NTF7_2_DATA)
    };

    static ref NTF7_2_PATH: FPath = {
        ntf_fpath(&NTF7_2)
    };

    //
    // NTF0X12000
    // zero-byte x12,000 times
    //

    static ref NTF0X12000: NamedTempFile = {
        create_temp_file_data(NTF0X12000_DATA)
    };

    static ref NTF0X12000_PATH: FPath = {
        ntf_fpath(&NTF0X12000)
    };

    //

    // 76208400
    // Thursday, June 1, 1972 1:00:00 AM GMT+00:00
    // Wednesday, May 31, 1972 6:00:00 PM GMT-07:00
    static ref SYSTEMTIME_1972_06_01: SystemTime = {
        seconds_to_systemtime(&76208400)
    };

    // 107744400
    // Friday, June 1, 1973 1:00:00 AM GMT+00:00
    // Thursday, May 31, 1973 6:00:00 PM GMT-07:00
    static ref SYSTEMTIME_1973_06_01: SystemTime = {
        seconds_to_systemtime(&107744400)
    };
}

/// Helper to wrap the match and panic checks
fn new_SyslogProcessor(
    path: &FPath,
    blocksz: BlockSz,
) -> SyslogProcessor {
    let tzo: FixedOffset = *FO_0;
    defñ!("SyslogProcessor::new({:?}, {:?}, {:?})", path, blocksz, tzo);
    let result = fpath_to_filetype(path, true);
    let filetype = match result {
        PathToFiletypeResult::Filetype(ft) => ft,
        PathToFiletypeResult::Archive(..) => {
            panic!("ERROR: fpath_to_filetype({:?}) returned an PathToFiletypeResult::Archive", path);
        }
    };
    match SyslogProcessor::new(path.clone(), filetype, blocksz, tzo, None, None) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslogProcessor::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// test `SyslogProcessor::new`
#[test]
fn test_SyslogProcessor_new_empty() {
    new_SyslogProcessor(&NTF_LOG_EMPTY_FPATH, SZ);
}

#[test]
#[should_panic]
fn test_SyslogProcessor_new_bad_path_panics() {
    new_SyslogProcessor(&FPath::from("/THIS/PATH/DOES///EXIST!!!!!"), SZ);
}

#[cfg(target_family = "unix")]
#[test]
fn test_new_SyslogProcessor_no_file_permissions() {
    let ntf = create_temp_file_no_permissions(".log");
    let path = ntf.path();
    let fpath = path_to_fpath(path);
    match SyslogProcessor::new(
        fpath.clone(),
        FILETYPE_UTF8,
        1024,
        *FO_0,
        None,
        None,
    ) {
        Ok(_) => {
            panic!("no permissions to read {:?}", path);
        }
        Err(err) => {
            defo!("no permissions to read {:?}", path);
            defo!("error (expected): {}", err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_process_missing_year_1972() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);
    slp.process_missing_year(*SYSTEMTIME_1972_06_01, &None);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_find_sysline() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);
    let mut fo: FileOffset = 0;
    loop {
        let result = slp.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
    }
}

#[test]
fn test_find_sysline_between_datetime_filters_Found() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);

    let result = slp.find_sysline_between_datetime_filters(0);
    match result {
        ResultS3SyslineFind::Found((_fo, _syslinep)) => {}
        ResultS3SyslineFind::Done => {
            panic!("Unexpected Done");
        }
        ResultS3SyslineFind::Err(err) => {
            panic!("Error {}", err);
        }
    }
}

#[test]
fn test_find_sysline_between_datetime_filters_Done() {
    let mut slp = new_SyslogProcessor(&NTF5_PATH, SZ);
    let fo: FileOffset = NTF5_DATA.len() as FileOffset;

    let result = slp.find_sysline_between_datetime_filters(fo);
    match result {
        ResultS3SyslineFind::Found((_fo, _syslinep)) => {
            panic!("Unexpected Found");
        }
        ResultS3SyslineFind::Done => {}
        ResultS3SyslineFind::Err(err) => {
            panic!("Error {}", err);
        }
    }
}

// -------------------------------------------------------------------------------------------------

const FILEOK: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileOk;
const FILEEMPTY: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrEmpty;
const FILENOSYSLINESFOUND: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrNoSyslinesFound;
const FILENULLBYTES: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrNullBytes;
const FILETOOSMALL: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrTooSmall;
#[allow(dead_code)]
const FILENOSYSLINESINRANGE: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrNoSyslinesInDtRange;
#[allow(dead_code)]
const FILEWRONGTYPE: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrWrongType;
#[allow(dead_code)]
const FILEDECOMPRESS: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrDecompress;
#[allow(dead_code)]
const FILESTUB: FileProcessingResultBlockZero = FileProcessingResultBlockZero::FileErrStub;

#[test_case(&NTF_LOG_EMPTY_FPATH, SYSLOG_SZ_MAX_BSZ, FILEEMPTY)]
#[test_case(&NTF_GZ_EMPTY_FPATH, SYSLOG_SZ_MAX_BSZ, FILEEMPTY)]
#[test_case(&NTF3_PATH, SYSLOG_SZ_MAX_BSZ, FILEOK)]
fn test_process_stage0(
    path: &FPath,
    blocksz: BlockSz,
    fprbz_expect: FileProcessingResultBlockZero,
) {
    eprint_file_blocks(path, blocksz);
    let mut slp = new_SyslogProcessor(path, blocksz);

    let fprbz_actual = slp.process_stage0_valid_file_check();
    assert_eq!(fprbz_actual, fprbz_expect,
        "process_stage0_valid_file_check\n  expected {:?}, actual {:?}",
        fprbz_expect, fprbz_actual,
    );
}

#[test_case(
    &*NTF7_2_PATH,
    (NTF7_2_DATA_LINE1_OFFSET + (NTF7_2_DATA_LINE1_OFFSET % 2)) as BlockSz,
    FILEOK;
    "NTF7_2_PATH NTF7_2_DATA_LINE1_OFFSET"
)]
#[test_case(
    &*NTF7_2_PATH,
    (NTF7_2_DATA_LINE2_OFFSET + (NTF7_2_DATA_LINE2_OFFSET % 2)) as BlockSz,
    FILEOK;
    "NTF7_2_PATH NTF7_2_DATA_LINE2_OFFSET"
)]
#[test_case(
    &*NTF7_2_PATH,
    (NTF7_2_DATA_LINE3_OFFSET + (NTF7_2_DATA_LINE3_OFFSET % 2)) as BlockSz,
    FILEOK;
    "NTF7_2_PATH NTF7_2_DATA_LINE3_OFFSET"
)]
#[test_case(
    &*NTF7_2_PATH,
    (NTF7_2_DATA_LINE4_OFFSET + (NTF7_2_DATA_LINE4_OFFSET % 2)) as BlockSz,
    FILEOK;
    "NTF7_2_PATH NTF7_2_DATA_LINE4_OFFSET"
)]
#[test_case(&*NTF7_2_PATH, 0x10, FILENOSYSLINESFOUND)]
#[test_case(&*NTF7_2_PATH, 0x100, FILEOK)]
#[test_case(&*NTF7_2_PATH, 0x200, FILEOK)]
#[test_case(&*NTF7_2_PATH, 0x1000, FILEOK)]
#[test_case(&*NTF3_PATH, 0x10, FILENOSYSLINESFOUND)]
#[test_case(&*NTF3_PATH, 0x14, FILENOSYSLINESFOUND)]
#[test_case(&*NTF3_PATH, 0x16, FILENOSYSLINESFOUND)]
#[test_case(&*NTF3_PATH, 0x18, FILEOK)]
#[test_case(&*NTF3_PATH, 0x1A, FILEOK)]
#[test_case(&*NTF3_PATH, 0x20, FILEOK)]
#[test_case(&*NTF3_PATH, 0x40, FILEOK)]
#[test_case(&*NTF3_PATH, 0x1000, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x2, FILENOSYSLINESFOUND)]
#[test_case(&*NTF5X4_PATH, 0x10, FILENOSYSLINESFOUND)]
#[test_case(&*NTF5X4_PATH, 0x20, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x30, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x40, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x50, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x60, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x70, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x80, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x100, FILEOK)]
#[test_case(&*NTF5X4_PATH, 0x200, FILEOK)]
#[test_case(&*NTF0X12000_PATH, 0x10, FILENULLBYTES)]
#[test_case(&*NTF0X12000_PATH, SYSLOG_SZ_MAX_BSZ * 2, FILENULLBYTES)]
#[test_case(&*NTF1S_A_PATH, 0x2, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_A_PATH, 0x4, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_A_PATH, 0xA, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_A_PATH, 0xB, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_A_PATH, 0xC, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_A_PATH, 0xD, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_A_PATH, 0xE, FILEOK)]
#[test_case(&*NTF1S_A_PATH, 0xF, FILEOK)]
#[test_case(&*NTF1S_A_PATH, 0x10, FILEOK)]
#[test_case(&*NTF1S_A_PATH, 0x100, FILEOK)]
#[test_case(&*NTF1S_B_PATH, 0x2, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0x4, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0xA, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0xB, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0xC, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0xD, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0xE, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_B_PATH, 0xF, FILEOK)]
#[test_case(&*NTF1S_B_PATH, 0x10, FILEOK)]
#[test_case(&*NTF1S_B_PATH, 0x100, FILEOK)]
#[test_case(&*NTF1S_C_PATH, 0x2, FILENOSYSLINESFOUND)]
#[test_case(&*NTF1S_C_PATH, 0x4, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0xA, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0xB, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0xC, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0xD, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0xE, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0xF, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0x10, FILETOOSMALL)]
#[test_case(&*NTF1S_C_PATH, 0x100, FILETOOSMALL)]
#[test_case(&*NTF2S_A_PATH, 0x2, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0x4, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0xA, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0xB, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0xC, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0xD, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0xE, FILENOSYSLINESFOUND)]
#[test_case(&*NTF2S_A_PATH, 0xF, FILEOK)]
#[test_case(&*NTF2S_A_PATH, 0x10, FILEOK)]
#[test_case(&*NTF2S_A_PATH, 0x100, FILEOK)]
fn test_process_stage1_blockzero_analysis_varying(
    path: &FPath,
    blocksz: BlockSz,
    fprbz_expect: FileProcessingResultBlockZero,
) {
    eprint_file_blocks(path, blocksz);
    let mut slp = new_SyslogProcessor(path, blocksz);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage0 {:?}; expected FileOk, blocksz {} 0x{:02X}", result, blocksz, blocksz);
        }
    }

    let fprbz_actual = slp.process_stage1_blockzero_analysis();
    assert_eq!(fprbz_actual, fprbz_expect,
        "process_stage1_blockzero_analysis\n  expected {:?}, actual {:?}",
        fprbz_expect, fprbz_actual,
    );
}

#[test_case(&*NTF3_PATH, 0x200, FILEOK, &NTF3_DATA_SYSLINES)]
fn test_process_stages_0to5(
    path: &FPath,
    blocksz: BlockSz,
    fprbz_expect: FileProcessingResultBlockZero,
    syslines_expect: &[&str],
) {
    let mut slp = new_SyslogProcessor(path, blocksz);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect,
                "process_stage0_valid_file_check\n  expected {:?}, actual {:?}",
                fprbz_expect, result,
            );
            return;
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect,
                "process_stage1_blockzero_analysis\n  expected {:?}, actual {:?}",
                fprbz_expect, result,
            );
            return;
        }
    }

    match slp.process_stage2_find_dt(&None) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect,
                "process_stage2_find_dt\n  expected {:?}, actual {:?}",
                fprbz_expect, result,
            );
        }
    }

    match slp.process_stage3_stream_syslines() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage3 {:?}", result);
        }
    }

    let checks = syslines_expect;
    let mut check_counter: usize = 0;
    let mut fo: FileOffset = 0;
    loop {
        defo!("check_counter {}, slp.find_sysline({})", check_counter, fo);
        let result = slp.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, syslinep)) => {
                fo = fo_;
                assert_eq!(
                    checks[check_counter],
                    syslinep.to_String().as_str(),
                    "failed check {}",
                    check_counter,
                );
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
        check_counter += 1;
    }
    assert_eq!(
        checks.len(),
        check_counter,
        "only counted {} syslines, expected to count {} syslines",
        check_counter,
        checks.len()
    );

    let _summary = slp.process_stage4_summary();
}

// test files without a year and a `dt_filter_after_opt` do not process
// the entire file, only back to `dt_filter_after_opt`
#[test_case(&*NTF5_PATH, &None, 5)]
#[test_case(&*NTF5_PATH, &NTF5_DATA_LINE2_BEFORE, 4)]
#[test_case(&*NTF5_PATH, &NTF5_DATA_LINE4_AFTER, 1)]
#[test_case(&*NTF1S_A_PATH, &NTF1S_A_DATA_LINE0_AFTER, 1)]
fn test_process_stage2_find_dt_and_missing_year(
    path: &FPath,
    filter_dt_after_opt: &DateTimeLOpt,
    count_syslines_expect: Count,
) {
    eprint_file(path);
    let mut slp = new_SyslogProcessor(path, 0xFFFF);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage0 {:?}", result);
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage1 {:?}", result);
        }
    }

    match slp.process_stage2_find_dt(filter_dt_after_opt) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            panic!("Unexpected result stage2 {:?}", result);
        }
    }

    assert_eq!(slp.count_syslines_stored(), count_syslines_expect);
}

// -------------------------------------------------------------------------------------------------

#[test_case(&NTF9_PATH, NTF9_BLOCKSZ_MIN, FILEOK)]
#[test_case(&NTF7_2_PATH, NTF7_2_BLOCKSZ_MIN, FILENOSYSLINESFOUND)]
#[test_case(&NTF7_2_PATH, NTF7_2_BLOCKSZ_MIN * 2, FILEOK)]
fn test_process_stage0to3_drop_data(
    path: &FPath,
    blocksz: BlockSz,
    fprbz_expect: FileProcessingResultBlockZero,
) {
    eprint_file(path);
    let mut slp = new_SyslogProcessor(path, blocksz);

    match slp.process_stage0_valid_file_check() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect, "Unexpected result stage0");
            return;
        }
    }

    match slp.process_stage1_blockzero_analysis() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect, "Unexpected result stage1");
            return;
        }
    }

    match slp.process_stage2_find_dt(&None) {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect, "Unexpected result stage2");
            return;
        }
    }

    match slp.find_sysline_between_datetime_filters(0) {
        ResultS3SyslineFind::Found(_) => {}
        ResultS3SyslineFind::Done => {
            panic!("Unexpected Done");
        }
        ResultS3SyslineFind::Err(err) => {
            panic!(
                "ERROR: SyslogProcessor.find_sysline_between_datetime_filters(0) Path {:?} Error {}",
                path, err
            );
        }
    }

    match slp.process_stage3_stream_syslines() {
        FileProcessingResultBlockZero::FileOk => {}
        result => {
            assert_eq!(result, fprbz_expect, "Unexpected result stage3");
            return;
        }
    }

    let mut fo: FileOffset = 0;
    let mut syslinep_last_opt: Option<SyslineP> = None;
    loop {
        match slp.find_sysline(fo) {
            ResultS3SyslineFind::Found((fo_, syslinep)) => {
                fo = fo_;
                if let Some(syslinep_) = syslinep_last_opt {
                    slp.drop_data_try(&syslinep_);
                }
                syslinep_last_opt = Some(syslinep);
            }
            ResultS3SyslineFind::Done => break,
            ResultS3SyslineFind::Err(err) => {
                panic!(
                    "ERROR: SyslogProcessor.find_sysline({}) Path {:?} Error {}",
                    fo, path, err
                );
            }
        }
    }

    let dropped_syslines = slp.dropped_syslines();
    assert_gt!(dropped_syslines.len(), 0, "Expected *some* dropped Syslines but zero were dropped, blocksz {:?}, filesz {:?}", blocksz, slp.filesz());
    let dropped_lines = slp.dropped_lines();
    assert_gt!(dropped_lines.len(), 0, "Expected *some* dropped Lines but zero were dropped, blocksz {:?}, filesz {:?}", blocksz, slp.filesz());
    let dropped_blocks = slp.dropped_blocks();
    assert_gt!(dropped_blocks.len(), 0, "Expected *some* dropped Blocks but zero were dropped, blocksz {:?}, filesz {:?}", blocksz, slp.filesz());
}

/// It is difficult to test `DateTimeParseDatasCompiledCount` because it is a
/// global singleton. And since the ordering of tests is not predictable
/// then it's value cannot be predicted except to be greater than some number after
/// exercising a `SyslogProcessor` (or `SyslineReader`).
/// Setting and resetting the global singletons `DATETIME_PARSE_DATAS_REGEX_VEC`
/// and `DateTimeParseDatasCompiledCount` requires an associated global mutex.
/// If there was a global mutex then _every_ test function that effects those
/// global singletons would need to use that global mutex lock.
/// This is too much boilerplate to require for so many tests.
/// So just test `DateTimeParseDatasCompiledCount` is greater than some number
/// after processing the file with a `SyslogProcessor` instance.
#[test_case(&NTF1S_A_PATH, 34)]
#[test_case(&NTF1S_B_PATH, 34)]
#[test_case(&NTF1S_C_PATH, 0)]
#[test_case(&NTF5_PATH, 34)]
#[test_case(&NTF5X4_PATH, 79)]
#[test_case(&NTF3_PATH, 32)]
#[test_case(&NTF9_PATH, 32)]
#[test_case(&NTF7_2_PATH, 32)]
fn test_datetime_parse_datas_compiled_count(
    path: &FPath,
    count_ge: usize,
) {
    let mut syslogprocessor = new_SyslogProcessor(path, SZ);

    // find all the entries
    let mut fo: FileOffset = 0;
    loop {
        match syslogprocessor.find_sysline(fo) {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
    }

    let count = match DateTimeParseDatasCompiledCount.read() {
        Ok(count) => *count,
        Err(err) => {
            panic!("Error DateTimeParseDatasCompiledCount.read() failed {}", err);
        }
    };
    eprintln!("DateTimeParseDatasCompiledCount {}", count);
    assert_ge!(
        count, count_ge,
        "DateTimeParseDatasCompiledCount expected >={}, actual {}", count_ge, count
    );
}

/// test `SyslogProcessor::summary` and `SyslogProcessor::summary_complete`
/// before doing any processing
#[test_case(&NTF_LOG_EMPTY_FPATH, 0x100)]
#[test_case(&NTF_GZ_8BYTE_FPATH, 0x100)]
fn test_SyslogProcessor_summary_empty(
    path: &FPath,
    blocksz: BlockSz,
) {
    let syslogprocessor = new_SyslogProcessor(
        path,
        blocksz,
    );
    _ = syslogprocessor.summary();
    _ = syslogprocessor.summary_complete();
}

#[test_case(
    &*NTF5X4_PATH,
    0x64,
    SummaryBlockReader {
        blockreader_bytes: 120,
        blockreader_bytes_total: 120,
        blockreader_blocks: 2,
        blockreader_blocks_total: 2,
        blockreader_blocksz: 100,
        blockreader_filesz: 120,
        blockreader_filesz_actual: 120,
        blockreader_read_block_lru_cache_hit: 4,
        blockreader_read_block_lru_cache_miss: 2,
        blockreader_read_block_lru_cache_put: 2,
        blockreader_read_blocks_hit: 0,
        blockreader_read_blocks_miss: 2,
        blockreader_read_blocks_put: 2,
        blockreader_read_blocks_reread_error: 0,
        blockreader_blocks_highest: 2,
        blockreader_blocks_dropped_ok: 0,
        blockreader_blocks_dropped_err: 0,
    },
    SummaryLineReader {
        linereader_lines: 5,
        linereader_lines_stored_highest: 5,
        linereader_lines_hits: 0,
        linereader_lines_miss: 9,
        linereader_find_line_lru_cache_hit: 4,
        linereader_find_line_lru_cache_miss: 7,
        linereader_find_line_lru_cache_put: 5,
        linereader_drop_line_ok: 0,
        linereader_drop_line_errors: 0,
    },
    SummarySyslineReader {
        syslinereader_syslines: 5,
        syslinereader_syslines_stored_highest: 5,
        syslinereader_syslines_hit: 0,
        syslinereader_syslines_miss: 6,
        syslinereader_syslines_by_range_hit: 0,
        syslinereader_syslines_by_range_miss: 6,
        syslinereader_syslines_by_range_put: 5,
        syslinereader_patterns: DateTimePatternCounts::from([(33, 1), (78, 4)]),
        syslinereader_datetime_first: Some(ymdhms0(1972, 1, 1, 1, 0, 11)),
        syslinereader_datetime_last: Some(ymdhms0(2000, 5, 15, 5, 0, 55)),
        syslinereader_find_sysline_lru_cache_hit: 0,
        syslinereader_find_sysline_lru_cache_miss: 6,
        syslinereader_find_sysline_lru_cache_put: 6,
        syslinereader_parse_datetime_in_line_lru_cache_hit: 4,
        syslinereader_parse_datetime_in_line_lru_cache_miss: 5,
        syslinereader_parse_datetime_in_line_lru_cache_put: 0,
        syslinereader_get_boxptrs_singleptr: 116,
        syslinereader_get_boxptrs_doubleptr: 1,
        syslinereader_get_boxptrs_multiptr: 0,
        syslinereader_regex_captures_attempted: 117,
        syslinereader_drop_sysline_ok: 0,
        syslinereader_drop_sysline_errors: 0,
        syslinereader_ezcheck12_hit: 0,
        syslinereader_ezcheck12_miss: 0,
        syslinereader_ezcheck12_hit_max: 0,
        syslinereader_ezcheckd2_hit: 0,
        syslinereader_ezcheckd2_miss: 9,
        syslinereader_ezcheckd2_hit_max: 0,
        syslinereader_ezcheck12d2_hit: 0,
        syslinereader_ezcheck12d2_miss: 108,
        syslinereader_ezcheck12d2_hit_max: 0,
    },
    SummarySyslogProcessor {
        syslogprocessor_missing_year: None,
    }; "NTF5X4_PATH 0x64"
)]
fn test_Reader_summary(
    path: &FPath,
    blocksz: BlockSz,
    expect_summaryblockreader: SummaryBlockReader,
    expect_summarylinereader: SummaryLineReader,
    expect_summarysyslinereader: SummarySyslineReader,
    expect_summarysyslogprocessor: SummarySyslogProcessor,
) {
    let mut syslogprocessor = new_SyslogProcessor(
        path,
        blocksz,
    );

    // find all the entries
    let mut fo: FileOffset = 0;
    loop {
        let result = syslogprocessor.find_sysline(fo);
        match result {
            ResultS3SyslineFind::Found((fo_, _syslinep)) => {
                fo = fo_;
            }
            ResultS3SyslineFind::Done => {
                break;
            }
            ResultS3SyslineFind::Err(err) => {
                panic!("Error {}", err);
            }
        }
    }

    // get the summaries
    let summary = syslogprocessor.summary_complete();
    let (
        summaryblockreader,
        summarylinereader,
        summarysyslinereader,
        summarysyslogreader,
    ) = match summary.readerdata {
        SummaryReaderData::Syslog((
            summaryblockreader,
            summarylinereader,
            summarysyslinereader,
            summarysyslogreader,
        )) => {
            (
                summaryblockreader,
                summarylinereader,
                summarysyslinereader,
                summarysyslogreader,
            )
        }
        _ => panic!(),
    };
    eprintln!("\nsummaryblockreader:\n{:?}\n", summaryblockreader);
    eprintln!("\nsummarylinereader:\n{:?}\n", summarylinereader);
    eprintln!("\nsummarysyslinereader:\n{:?}\n", summarysyslinereader);
    eprintln!("\nsummarysyslogreader:\n{:?}\n", summarysyslogreader);
    assert_eq!(
        summaryblockreader,
        expect_summaryblockreader,
    );
    assert_eq!(
        summarylinereader,
        expect_summarylinereader,
    );
    assert_eq!(
        summarysyslinereader,
        expect_summarysyslinereader,
    );
    assert_eq!(
        summarysyslogreader,
        expect_summarysyslogprocessor,
    );

    let summarysyslogreader2 = syslogprocessor.summary();
    assert_eq!(
        summarysyslogreader2,
        expect_summarysyslogprocessor,
    );
}
