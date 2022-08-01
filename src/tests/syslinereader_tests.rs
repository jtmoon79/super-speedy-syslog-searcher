// src/Readers/syslinereader_tests.rs
//

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::common::{
    FPath,
    ResultS4,
};

use crate::Data::sysline::{
    SyslineP,
};

use crate::Readers::blockreader::{
    FileOffset,
    BlockSz,
};

use crate::Readers::filepreprocessor::{
    fpath_to_filetype_mimeguess,
};

use crate::Readers::helpers::{
    randomize,
    fill,
};

use crate::Readers::syslinereader::{
    SyslineReader,
    ResultS4_SyslineFind,
};

use crate::Data::datetime::{
    // chrono imports
    TimeZone,
    FixedOffset,
    //
    DateTimeL,
    DateTimeL_Opt,
    DateTimePattern_str,
    Result_Filter_DateTime2,
    datetime_parse_from_str,
};

use crate::tests::datetime_tests::{
    dt_pattern_has_year,
    dt_pattern_has_tz,
};

use crate::tests::common::{
    eprint_file,
};

use crate::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_bytes,
    NTF_Path,
};

use crate::printer_debug::printers::{
    str_to_String_noraw,
};

use crate::printer_debug::stack::{
    stack_offset_set,
    sn,
    so,
    sx,
};

use std::str;

extern crate const_format;
use const_format::concatcp;

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper to wrap the match and panic checks
fn new_SyslineReader(path: &FPath, blocksz: BlockSz, tzo: FixedOffset) -> SyslineReader {
    stack_offset_set(Some(2));
    let (filetype, _mimeguess) = fpath_to_filetype_mimeguess(path);
    match SyslineReader::new(path.clone(), filetype, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const NTF5_DATA: &str = "\
[20200113-11:03:06] [DEBUG] Testing if xrdp can listen on 0.0.0.0 port 3389.
[20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
CLOSED!
[20200113-11:03:08] [INFO ] starting xrdp with pid 23198
[20200113-11:03:08] [INFO ] listening to port 3389 on 0.0.0.0
[20200113-11:13:59] [INFO ] Socket 12: AF_INET6 connection received from ::ffff:127.0.0.1 port 55426
[20200113-11:13:59] [DEBUG] Closed socket 12 (AF_INET6 ::ffff:127.0.0.1 port 3389)
[20200113-11:13:59] [DEBUG] Closed socket 11 (AF_INET6 :: port 3389)
[20200113-11:13:59] [INFO ] Using default X.509 certificate: /etc/xrdp/cert.pem
[20200113-11:13:59] [INFO ] Using default X.509 key file: /etc/xrdp/key.pem
[20200113-11:13:59] [DEBUG] read private key file /etc/xrdp/key.pem
[20200113-11:13:59] [DEBUG] Certification found
    FOUND CERTIFICATE!
[20200113-11:13:59] [DEBUG] Certification complete.
";

lazy_static! {
    static ref NTF5: NamedTempFile = {
        create_temp_file(&NTF5_DATA)
    };
    static ref NTF5_PATH: FPath = {
        NTF_Path(&NTF5)
    };
}

/// basic test of `SyslineReader.find_datetime_in_line`
#[allow(non_snake_case)]
fn test_find_datetime_in_line_by_block(blocksz: BlockSz) {
    eprintln!("{}test_find_datetime_in_line_by_block()", sn());
    let ntf: &NamedTempFile = &NTF5;
    let path = NTF_Path(ntf);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(&path, blocksz, tzo);

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp))
            | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Found|Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_find_datetime_in_line: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                fo1 = fo;
            }
            ResultS4_SyslineFind::Done => {
                //eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                panic!("ERROR: test_find_datetime_in_line: slr.find_sysline({}) returned Err({})", fo1, err);
            }
        }
        if done {
            break;
        }
    }

    eprintln!("{}test_find_datetime_in_line_by_block()", sx());
}

#[test]
fn test_find_datetime_in_line_by_block2() {
    test_find_datetime_in_line_by_block(2);
}

#[test]
fn test_find_datetime_in_line_by_block4() {
    test_find_datetime_in_line_by_block(4);
}

#[test]
fn test_find_datetime_in_line_by_block8() {
    test_find_datetime_in_line_by_block(8);
}

#[test]
fn test_find_datetime_in_line_by_block256() {
    test_find_datetime_in_line_by_block(256);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
fn test_find_sysline_at_datetime_filter2(
    ntf: &NamedTempFile,
    dt_pattern: &DateTimePattern_str,
    blocksz: BlockSz,
    checks: test_find_sysline_at_datetime_filter_Checks,
) {
    eprintln!("{}test_find_sysline_at_datetime_filter2(…, {:?}, {}, …)", sn(), dt_pattern, blocksz);

    let path = NTF_Path(&ntf);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    for (fo1, dts, sline_expect) in checks.iter() {
        // TODO: add `has_tz` to `checks`
        let has_tz = dt_pattern_has_tz(&dt_pattern);
        eprintln!("{}datetime_parse_from_str({:?}, {:?}, {:?}, {:?})", so(), str_to_String_noraw(dts), dt_pattern, has_tz, &tzo);
        let dt = match datetime_parse_from_str(dts, dt_pattern, has_tz, &tzo) {
            Some(val) => val,
            None => {
                panic!("ERROR: datetime_from_str({:?}, {:?}) returned None", dts, dt_pattern);
            }
        };
        let sline_expect_noraw = str_to_String_noraw(sline_expect);
        eprintln!("{}find_sysline_at_datetime_filter({}, {:?})", so(), fo1, dt);
        let result = slr.find_sysline_at_datetime_filter(*fo1, &Some(dt));
        match result {
            ResultS4_SyslineFind::Found(val) | ResultS4_SyslineFind::Found_EOF(val) => {
                let sline = val.1.to_String();
                let sline_noraw = str_to_String_noraw(sline.as_str());
                eprintln!("\nexpected: {:?}", sline_expect_noraw);
                eprintln!("returned: {:?}\n", sline_noraw);
                assert_eq!(
                    sline,
                    String::from(*sline_expect),
                    "Expected {:?} == {:?} but it is not!",
                    sline_noraw,
                    sline_expect_noraw
                );
                eprintln!(
                    "Check PASSED SyslineReader().find_sysline_at_datetime_filter({} {:?}) == {:?}",
                    fo1, dts, sline_noraw
                );
            }
            ResultS4_SyslineFind::Done => {
                panic!("During test unexpected result Done");
            }
            ResultS4_SyslineFind::Err(err) => {
                panic!("During test unexpected result Error {}", err);
            }
        }
    }

    eprintln!("{}test_find_sysline_at_datetime_filter1(…)", sx());
}

// -------------------------------------------------------------------------------------------------

type test_find_sysline_at_datetime_filter_Checks<'a> = Vec<(FileOffset, &'a str, &'a str)>;

const NTF26_DATA: &str = "\
2020-01-01 00:00:00
2020-01-01 00:00:01a
2020-01-01 00:00:02ab
2020-01-01 00:00:03abc
2020-01-01 00:00:04abcd
2020-01-01 00:00:05abcde
2020-01-01 00:00:06abcdef
2020-01-01 00:00:07abcdefg
2020-01-01 00:00:08abcdefgh
2020-01-01 00:00:09abcdefghi
2020-01-01 00:00:10abcdefghij
2020-01-01 00:00:11abcdefghijk
2020-01-01 00:00:12abcdefghijkl
2020-01-01 00:00:13abcdefghijklm
2020-01-01 00:00:14abcdefghijklmn
2020-01-01 00:00:15abcdefghijklmno
2020-01-01 00:00:16abcdefghijklmnop
2020-01-01 00:00:17abcdefghijklmnopq
2020-01-01 00:00:18abcdefghijklmnopqr
2020-01-01 00:00:19abcdefghijklmnopqrs
2020-01-01 00:00:20abcdefghijklmnopqrst
2020-01-01 00:00:21abcdefghijklmnopqrstu
2020-01-01 00:00:22abcdefghijklmnopqrstuv
2020-01-01 00:00:23abcdefghijklmnopqrstuvw
2020-01-01 00:00:24abcdefghijklmnopqrstuvwx
2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy
2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz
";

const NTF26_DATA_DT0: &str = "2020-01-01 00:00:00";
const NTF26_DATA_LINE0n: &str = "2020-01-01 00:00:00\n";
const NTF26_DATA_DT1: &str = "2020-01-01 00:00:01";
const NTF26_DATA_LINE1n: &str = "2020-01-01 00:00:01a\n";
const NTF26_DATA_DT2: &str = "2020-01-01 00:00:02";
const NTF26_DATA_LINE2n: &str = "2020-01-01 00:00:02ab\n";
const NTF26_DATA_DT3: &str = "2020-01-01 00:00:03";
const NTF26_DATA_LINE3n: &str = "2020-01-01 00:00:03abc\n";
const NTF26_DATA_DT4: &str = "2020-01-01 00:00:04";
const NTF26_DATA_LINE4n: &str = "2020-01-01 00:00:04abcd\n";
// blah, this is a lot of work...
const NTF26_DATA_DT24: &str = "2020-01-01 00:00:24";
const NTF26_DATA_LINE24n: &str = "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n";
const NTF26_DATA_DT25: &str = "2020-01-01 00:00:25";
const NTF26_DATA_LINE25n: &str = "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n";
const NTF26_DATA_DT26: &str = "2020-01-01 00:00:26";
const NTF26_DATA_LINE26n: &str = "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n";

lazy_static! {
    static ref NTF26: NamedTempFile = {
        create_temp_file(&NTF26_DATA)
    };
    static ref NTF26_PATH: FPath = {
        NTF_Path(&NTF26)
    };

    static ref NTF26_checks: test_find_sysline_at_datetime_filter_Checks<'static> = {
        Vec::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, NTF26_DATA_LINE3n),
            (0, NTF26_DATA_DT4, NTF26_DATA_LINE4n),
            (0, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
            (0, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
            (0, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
            (0, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
            (0, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
            (0, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
            (0, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
            (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
            (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
            (0, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
            (0, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
            (0, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
            (0, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
            (0, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
            (0, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
            (0, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
            (0, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
            (0, NTF26_DATA_DT24, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])
    };

    static ref NTF26_checksx: test_find_sysline_at_datetime_filter_Checks<'static> = {
        Vec::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (19, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (40, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (62, NTF26_DATA_DT3, NTF26_DATA_LINE3n),
            (85, NTF26_DATA_DT4, NTF26_DATA_LINE4n),
            (109, "2020-01-01 00:00:05", "2020-01-01 00:00:05abcde\n"),
            (134, "2020-01-01 00:00:06", "2020-01-01 00:00:06abcdef\n"),
            (162, "2020-01-01 00:00:07", "2020-01-01 00:00:07abcdefg\n"),
            (187, "2020-01-01 00:00:08", "2020-01-01 00:00:08abcdefgh\n"),
            (215, "2020-01-01 00:00:09", "2020-01-01 00:00:09abcdefghi\n"),
            (244, "2020-01-01 00:00:10", "2020-01-01 00:00:10abcdefghij\n"),
            (274, "2020-01-01 00:00:11", "2020-01-01 00:00:11abcdefghijk\n"),
            (305, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
            (337, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (370, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
            (404, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
            (439, "2020-01-01 00:00:16", "2020-01-01 00:00:16abcdefghijklmnop\n"),
            (475, "2020-01-01 00:00:17", "2020-01-01 00:00:17abcdefghijklmnopq\n"),
            (512, "2020-01-01 00:00:18", "2020-01-01 00:00:18abcdefghijklmnopqr\n"),
            (550, "2020-01-01 00:00:19", "2020-01-01 00:00:19abcdefghijklmnopqrs\n"),
            (589, "2020-01-01 00:00:20", "2020-01-01 00:00:20abcdefghijklmnopqrst\n"),
            (629, "2020-01-01 00:00:21", "2020-01-01 00:00:21abcdefghijklmnopqrstu\n"),
            (670, "2020-01-01 00:00:22", "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n"),
            (712, "2020-01-01 00:00:23", "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n"),
            (755, NTF26_DATA_DT24, NTF26_DATA_LINE24n),
            (799, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (844, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])
    };
}

// TODO: [2022/03/16] create test cases with varying sets of Checks passed-in, current setup is always
//       clean, sequential series of checks from file_offset 0.

/// basic test of `SyslineReader.find_datetime_in_line`
fn test_find_sysline_at_datetime_filter1(
    blocksz: BlockSz, checks: Option<test_find_sysline_at_datetime_filter_Checks>,
) {
    stack_offset_set(None);
    eprintln!("{}test_find_sysline_at_datetime_filter1()", sn());
    let dt_fmt1: &DateTimePattern_str = "%Y-%m-%d %H:%M:%S";

    let checks_: test_find_sysline_at_datetime_filter_Checks = checks.unwrap_or(NTF26_checks.clone());
    test_find_sysline_at_datetime_filter2(
        &NTF26,
        dt_fmt1,
        blocksz,
        checks_
    );
    eprintln!("{}test_find_sysline_at_datetime_filter1()", sx());
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test]
fn test_find_sysline_at_datetime_filter_4() {
    test_find_sysline_at_datetime_filter1(4, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_8() {
    test_find_sysline_at_datetime_filter1(8, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_16() {
    test_find_sysline_at_datetime_filter1(16, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_32() {
    test_find_sysline_at_datetime_filter1(32, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_64() {
    test_find_sysline_at_datetime_filter1(64, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_128() {
    test_find_sysline_at_datetime_filter1(128, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_256() {
    test_find_sysline_at_datetime_filter1(256, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_512() {
    test_find_sysline_at_datetime_filter1(512, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_1024() {
    test_find_sysline_at_datetime_filter1(1024, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_2056() {
    test_find_sysline_at_datetime_filter1(2056, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_() {
    test_find_sysline_at_datetime_filter1(64,Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT0,
            NTF26_DATA_LINE0n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_a() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT1,
            NTF26_DATA_LINE1n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_b() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT2,
            NTF26_DATA_LINE2n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_c() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT3,
            NTF26_DATA_LINE3n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_d() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT4,
            NTF26_DATA_LINE4n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_e() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_f() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_g() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_h() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_i() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_j() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_k() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_l() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_m() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_n() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_o() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_p() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_q() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_r() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_s() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_t() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_u() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_v() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_w() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_x() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT24,
            NTF26_DATA_LINE24n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_y() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT25,
            NTF26_DATA_LINE25n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_z() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            NTF26_DATA_DT26,
            NTF26_DATA_LINE26n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_a() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            19,
            NTF26_DATA_DT1,
            NTF26_DATA_LINE1n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_b() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            40,
            NTF26_DATA_DT2,
            NTF26_DATA_LINE2n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_c() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            62,
            NTF26_DATA_DT3,
            NTF26_DATA_LINE3n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_d() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            85,
            NTF26_DATA_DT4,
            NTF26_DATA_LINE4n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_e() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            109,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_f() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            134,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_g() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            160,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_h() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            187,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_i() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            215,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_j() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            244,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_k() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            274,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_l() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            305,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_m() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            337,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_n() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            370,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_o() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            404,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_p() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            439,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_q() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            475,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_r() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            512,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_s() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            550,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_t() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            589,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_u() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            629,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_v() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            670,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_w() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            712,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_x() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            755,
            NTF26_DATA_DT24,
            NTF26_DATA_LINE24n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_y() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            799,
            NTF26_DATA_DT25,
            NTF26_DATA_LINE25n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_z() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([(
            844,
            NTF26_DATA_DT26,
            NTF26_DATA_LINE26n,
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_z_() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_y_() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_x_() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT24, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_m_() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_za() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ya() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_xa() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT24, NTF26_DATA_LINE24n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ma() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3____() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ab() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__az() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__bd() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT4, NTF26_DATA_LINE4n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ml() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__my() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__mz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__m_() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, NTF26_DATA_DT0, NTF26_DATA_LINE0n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaa() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abc() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aba() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abn() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aby() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_byo() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaa() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbc() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zba() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbn() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zby() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaa() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybc() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT3, NTF26_DATA_LINE3n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yba() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybn() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yby() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT2, NTF26_DATA_LINE2n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz() {
    test_find_sysline_at_datetime_filter1(
        64,
        Some(test_find_sysline_at_datetime_filter_Checks::from([
            (0, NTF26_DATA_DT25, NTF26_DATA_LINE25n),
            (0, NTF26_DATA_DT1, NTF26_DATA_LINE1n),
            (0, NTF26_DATA_DT26, NTF26_DATA_LINE26n),
        ])),
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: [2022/03/18] create one wrapper test test_find_sysline_at_datetime_checks_ that takes some
//        vec of test-input-output, and does all possible permutations.

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type test_SyslineReader_check<'a> = (&'a str, FileOffset);
type test_SyslineReader_checks<'a> = Vec<(&'a str, FileOffset)>;

/// basic linear test of `SyslineReader::find_sysline`
#[allow(non_snake_case)]
fn test_SyslineReader_find_sysline(
    path: &FPath,
    blocksz: BlockSz,
    fileoffset: FileOffset,
    checks: &test_SyslineReader_checks
) {
    stack_offset_set(Some(2));
    eprintln!("{}test_SyslineReader_find_sysline({:?}, {})", sn(), path, blocksz);
    eprint_file(path);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo);

    let mut fo1: FileOffset = fileoffset;
    let mut check_i: usize = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) => {
                eprintln!("{}test_SyslineReader_find_sysline: slr.find_sysline({}) returned Found({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader_find_sysline: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");
                fo1 = fo;

                if checks.is_empty() {
                    continue;
                }
                eprintln!("{}test_SyslineReader_find_sysline: find_sysline({}); check {} expect ({:?}, {:?})", so(), fo1, check_i, checks[check_i].1, checks[check_i].0);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n", check_String, fo1, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                eprintln!("{}test_SyslineReader_find_sysline: slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader_find_sysline: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!");
                fo1 = fo;

                if checks.is_empty() {
                    continue;
                }
                eprintln!("{}test_SyslineReader_find_sysline: find_sysline({}); check {} expect ({:?}, {:?})", so(), fo1, check_i, checks[check_i].1, checks[check_i].0);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:2}) returned {:?}\n", check_String, fo1, actual_String);
                // check fileoffset
                let check_fo = checks[check_i].1;
                assert_eq!(check_fo, fo, "expected fileoffset {}, but find_sysline returned fileoffset {} for check {}", check_fo, fo, check_i);
            }
            ResultS4_SyslineFind::Done => {
                eprintln!("{}test_SyslineReader_find_sysline: slr.find_sysline({}) returned Done", so(), fo1);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}test_SyslineReader_find_sysline: slr.find_sysline({}) returned Err({})", so(), fo1, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        if done {
            break;
        }
    }
    assert_eq!(checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", checks.len(), check_i);

    eprintln!("{}test_SyslineReader_find_sysline: Found {} Lines, {} Syslines", so(), slr.count_lines_processed(), slr.count_syslines_stored());
    eprintln!("{}test_SyslineReader_find_sysline({:?}, {})", sx(), &path, blocksz);
}

const test_data_file_A1_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

const test_data_file_A1_dt6_checks: [test_SyslineReader_check; 6] = [
    ("2000-01-01 00:00:00\n", 20),
    ("2000-01-01 00:00:01a\n", 41),
    ("2000-01-01 00:00:02ab\n", 63),
    ("2000-01-01 00:00:03abc\n", 86),
    ("2000-01-01 00:00:04abcd\n", 110),
    ("2000-01-01 00:00:05abcde", 134),
];

lazy_static! {
    static ref NTF_A1: NamedTempFile = {
        create_temp_file(test_data_file_A1_dt6)
    };
    static ref NTF_A1_path: FPath = {
        NTF_Path(&NTF_A1)
    };
}

#[test]
fn test_SyslineReader_A1_dt6_4_0_()
{
    let checks = test_SyslineReader_checks::from(test_data_file_A1_dt6_checks);
    test_SyslineReader_find_sysline(&NTF_A1_path, 4, 0, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_0_()
{
    let checks = test_SyslineReader_checks::from(test_data_file_A1_dt6_checks);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 0, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_1_()
{
    let checks = test_SyslineReader_checks::from(&test_data_file_A1_dt6_checks[1..]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 40, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_2_()
{
    let checks = test_SyslineReader_checks::from(&test_data_file_A1_dt6_checks[2..]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 62, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_3_()
{
    let checks = test_SyslineReader_checks::from(&test_data_file_A1_dt6_checks[3..]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 85, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_4_()
{
    let checks = test_SyslineReader_checks::from(&test_data_file_A1_dt6_checks[4..]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 86, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_X_beforeend()
{
    let checks = test_SyslineReader_checks::from([]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 132, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_X_pastend()
{
    let checks = test_SyslineReader_checks::from([]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 135, &checks);
}

#[test]
fn test_SyslineReader_A1_dt6_128_X9999()
{
    let checks = test_SyslineReader_checks::from([]);
    test_SyslineReader_find_sysline(&NTF_A1_path, 128, 9999, &checks);
}

// -------------------------------------------------------------------------------------------------

/// helper to assert `find_sysline` return enum
fn assert_results4 (
    fo: &FileOffset,
    result_expect: &ResultS4_SyslineFind_Test,
    result_actual: &ResultS4_SyslineFind
) {
    let actual: String = format!("{}", result_actual);
    match result_expect {
        ResultS4_SyslineFind_Test::Found(()) => {
            assert!(matches!(result_actual, ResultS4_SyslineFind::Found(_)), "Expected Found, Actual {} for find_sysline({})", actual, fo);
        },
        ResultS4_SyslineFind_Test::Found_EOF(()) => {
            assert!(matches!(result_actual, ResultS4_SyslineFind::Found_EOF(_)), "Expected Found_EOF, Actual {} for find_sysline({})", actual, fo);
        },
        ResultS4_SyslineFind_Test::Done => {
            assert!(matches!(result_actual, ResultS4_SyslineFind::Done), "Expected Done, Actual {} for find_sysline({})", actual, fo);
        },
        _ => {
            panic!("Unexpected result_expect");
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// check the return type of `find_sysline` to this dummy approximation of `ResultS4_SyslineFind`
#[allow(non_camel_case_types)]
pub type ResultS4_SyslineFind_Test = ResultS4<(), std::io::Error>;

type test_SyslineReader_any_input_check<'a> = (FileOffset, ResultS4_SyslineFind_Test, &'a str);
type test_SyslineReader_any_input_checks<'a> = Vec<(FileOffset, ResultS4_SyslineFind_Test, &'a str)>;

/// test of `SyslineReader::find_sysline` with test-specified fileoffset searches
#[allow(non_snake_case)]
fn test_SyslineReader_any_input_check(
    path: &FPath,
    blocksz: BlockSz,
    lru_cache_enable: bool,
    input_checks: &[test_SyslineReader_any_input_check],
) {
    stack_offset_set(Some(2));
    eprintln!("{}test_SyslineReader_any_input_check({:?}, {})", sn(), path, blocksz);
    eprint_file(path);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if ! lru_cache_enable {
        slr.LRU_cache_disable();
    }

    let mut check_i: usize = 0;
    let mut first_loop = true;
    for (input_fo, expect_result, expect_val) in input_checks.iter() {
        let result = slr.find_sysline(*input_fo);
        assert_results4(input_fo, expect_result, &result);
        match result {
            ResultS4_SyslineFind::Found((_fo, slp)) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found({}, @{:p})", so(), input_fo, _fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    _fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");

                let actual_String = (*slp).to_String();
                let expect_String = String::from(*expect_val);
                eprintln!("{}test_SyslineReader: find_sysline({}); check {}", so(), input_fo, check_i);
                assert_eq!(expect_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n", expect_String, input_fo, actual_String);
            }
            ResultS4_SyslineFind::Found_EOF((_fo, slp)) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), input_fo, _fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    _fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is oot last! Should have returned Found or is this Sysline last?");

                let actual_String = (*slp).to_String();
                let expect_String = String::from(*expect_val);
                eprintln!("{}test_SyslineReader: find_sysline({}); check {}", so(), input_fo, check_i);
                assert_eq!(expect_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}\n", expect_String, input_fo, actual_String);
            }
            ResultS4_SyslineFind::Done => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Done", so(), input_fo);
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Err({})", so(), input_fo, err);
                panic!("ERROR: {}", err);
            }
        }
        check_i += 1;
        if first_loop {
            // force pattern analysis
            slr.dt_patterns_analysis();
            first_loop = false;
        }
    }
    assert_eq!(input_checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", input_checks.len(), check_i);

    eprintln!("{}test_SyslineReader: Found {} Lines, {} Syslines", so(), slr.count_lines_processed(), slr.count_syslines_stored());
    eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

const test_data_any_file_A2_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

const test_data_any_file_A2_dt6_sysline0: &str = "2000-01-01 00:00:00\n";
const test_data_any_file_A2_dt6_sysline1: &str = "2000-01-01 00:00:01a\n";
const test_data_any_file_A2_dt6_sysline2: &str = "2000-01-01 00:00:02ab\n";
const test_data_any_file_A2_dt6_sysline3: &str = "2000-01-01 00:00:03abc\n";
const test_data_any_file_A2_dt6_sysline4: &str = "2000-01-01 00:00:04abcd\n";
const test_data_any_file_A2_dt6_sysline5: &str = "2000-01-01 00:00:05abcde";

const test_data_any_file_A2_dt6_checks_many: [test_SyslineReader_any_input_check; 50] = [
    (0, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (1, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (2, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (3, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (4, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (5, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (6, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (19, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (20, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (21, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (22, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (23, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (24, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (25, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (40, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (41, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (42, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (43, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (44, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (45, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (46, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (47, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (61, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (62, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (63, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (64, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (65, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (66, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (67, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (84, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (85, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (86, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (87, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (88, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (89, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (90, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (108, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (109, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (110, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (111, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (112, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (113, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (114, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (134, ResultS4_SyslineFind_Test::Done, ""),
    (135, ResultS4_SyslineFind_Test::Done, ""),
    (136, ResultS4_SyslineFind_Test::Done, ""),
    (137, ResultS4_SyslineFind_Test::Done, ""),
    (138, ResultS4_SyslineFind_Test::Done, ""),
    (139, ResultS4_SyslineFind_Test::Done, ""),
    (140, ResultS4_SyslineFind_Test::Done, ""),
];

/// reverse order `test_data_any_file_A2_dt6_checks_many`
const test_data_any_file_A2_dt6_checks_many_rev: [test_SyslineReader_any_input_check; 50] = [
    (140, ResultS4_SyslineFind_Test::Done, ""),
    (139, ResultS4_SyslineFind_Test::Done, ""),
    (138, ResultS4_SyslineFind_Test::Done, ""),
    (137, ResultS4_SyslineFind_Test::Done, ""),
    (136, ResultS4_SyslineFind_Test::Done, ""),
    (135, ResultS4_SyslineFind_Test::Done, ""),
    (134, ResultS4_SyslineFind_Test::Done, ""),
    (114, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (113, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (112, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (111, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (110, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_any_file_A2_dt6_sysline5),
    (109, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (108, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (90, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (89, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (88, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (87, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (86, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline4),
    (85, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (84, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (67, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (66, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (65, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (64, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (63, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline3),
    (62, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (61, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (47, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (46, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (45, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (44, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (43, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (42, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (41, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline2),
    (40, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (25, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (24, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (23, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (22, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (21, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (20, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline1),
    (19, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (6, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (5, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (4, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (3, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (2, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (1, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
    (0, ResultS4_SyslineFind_Test::Found(()), test_data_any_file_A2_dt6_sysline0),
];

lazy_static! {
    static ref test_SyslineReader_A2_any_ntf: NamedTempFile = {
        create_temp_file(test_data_any_file_A2_dt6)
    };
    static ref test_SyslineReader_A2_any_ntf_path: FPath = {
        NTF_Path(&test_SyslineReader_A2_any_ntf)
    };
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_SyslineReader_A2_dt6_any_2()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 2, true, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_2_nocache()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 2, false, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_2_rev()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 2, true, &test_data_any_file_A2_dt6_checks_many_rev);
}

#[test]
fn test_SyslineReader_A2_dt6_any_2_rev_nocache()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 2, false, &test_data_any_file_A2_dt6_checks_many_rev);
}

#[test]
fn test_SyslineReader_A2_dt6_any_4()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 4, true, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_4_rev()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 4, true, &test_data_any_file_A2_dt6_checks_many_rev);
}

#[test]
fn test_SyslineReader_A2_dt6_any_4_nocache()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 4, false, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_4_rev_nocache()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 4, false, &test_data_any_file_A2_dt6_checks_many_rev);
}

#[test]
fn test_SyslineReader_A2_dt6_any_0xFF()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 0xFF, true, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_0xFF_rev()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 0xFF, true, &test_data_any_file_A2_dt6_checks_many_rev);
}

#[test]
fn test_SyslineReader_A2_dt6_any_0xFF_nocache()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 0xFF, false, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_0xFF_rev_nocache()
{
    test_SyslineReader_any_input_check(&test_SyslineReader_A2_any_ntf_path, 0xFF, false, &test_data_any_file_A2_dt6_checks_many_rev);
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const test_data_file_B_dt0: &str = "
foo
bar
";

#[allow(non_upper_case_globals)]
const test_data_file_B_dt0_checks: [test_SyslineReader_check; 0] = [];

#[test]
fn test_SyslineReader_B_dt0_0()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = NTF_Path(&ntf);
    let checks = test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader_find_sysline(&path, 128, 0, &checks);
}

#[test]
fn test_SyslineReader_B_dt0_3()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = NTF_Path(&ntf);
    let checks = test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader_find_sysline(&path, 128, 3, &checks);
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const test_data_file_C_dt6: &str = "\
[DEBUG] 2000-01-01 00:00:00
[DEBUG] 2000-01-01 00:00:01a
[DEBUG] 2000-01-01 00:00:02ab
[DEBUG] 2000-01-01 00:00:03abc
[DEBUG] 2000-01-01 00:00:04abcd
[DEBUG] 2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
const test_data_file_C_dt6_checks: [test_SyslineReader_check; 6] = [
    ("[DEBUG] 2000-01-01 00:00:00\n", 28),
    ("[DEBUG] 2000-01-01 00:00:01a\n", 57),
    ("[DEBUG] 2000-01-01 00:00:02ab\n", 87),
    ("[DEBUG] 2000-01-01 00:00:03abc\n", 118),
    ("[DEBUG] 2000-01-01 00:00:04abcd\n", 150),
    ("[DEBUG] 2000-01-01 00:00:05abcde", 182),
];

lazy_static! {
    static ref test_SyslineReader_C_ntf: NamedTempFile =
        create_temp_file(test_data_file_C_dt6);
    static ref test_SyslineReader_C_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_C_ntf);
}

#[test]
fn test_SyslineReader_C_dt6_0()
{
    let checks = test_SyslineReader_checks::from(test_data_file_C_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 0, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_3()
{
    let checks = test_SyslineReader_checks::from(test_data_file_C_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 3, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_27()
{
    let checks = test_SyslineReader_checks::from(test_data_file_C_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 27, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_28_1__()
{
    let checks = test_SyslineReader_checks::from(&test_data_file_C_dt6_checks[1..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 28, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_29_1__()
{
    let checks = test_SyslineReader_checks::from(&test_data_file_C_dt6_checks[1..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 29, &checks);
}

#[test]
fn test_SyslineReader_D_invalid1()
{
    let data_invalid1: [u8; 1] = [ 0xFF ];
    let date_checks1: test_SyslineReader_checks = test_SyslineReader_checks::from([]);
    let ntf = create_temp_file_bytes(&data_invalid1);
    let path = NTF_Path(&ntf);
    test_SyslineReader_find_sysline(&path, 128, 0, &date_checks1);
}

// -------------------------------------------------------------------------------------------------

/// notice the second line is an invalid date that will pass regex match
#[allow(non_upper_case_globals)]
const test_data_file_E_dt6: &str = "\
2001-01-01 00:00:00 _
2001-02-31 00:00:01 😩
2001-03-01 00:00:02 😀😁
2001-04-01 00:00:03 😀😁😂
2001-05-01 00:00:04 😀😁😂😃
2001-06-01 00:00:05 😀😁😂😃😄";

const test_data_file_E_dt6_sysline0: &str = "2001-01-01 00:00:00 _\n2001-02-31 00:00:01 😩\n";
const test_data_file_E_dt6_sysline1: &str = "2001-03-01 00:00:02 😀😁\n";
const test_data_file_E_dt6_sysline2: &str = "2001-04-01 00:00:03 😀😁😂\n";

lazy_static! {
    static ref test_SyslineReader_E_ntf: NamedTempFile =
        create_temp_file(test_data_file_E_dt6);
    static ref test_SyslineReader_E_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_E_ntf);
}

#[test]
fn test_SyslineReader_E_dt6_0()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (0, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_1()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (1, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_22()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (22, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_42()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (42, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_43()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_44()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (44, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_75()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (75, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_76()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (76, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline2),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_E_dt6_0______78()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (0, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (1, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (21, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (22, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (23, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (24, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (42, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (44, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (45, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (46, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline0),
            (47, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (48, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (49, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (70, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (71, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (72, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (73, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (74, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (75, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline1),
            (76, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline2),
            (77, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline2),
            (78, ResultS4_SyslineFind_Test::Found(()), test_data_file_E_dt6_sysline2),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_E_ntf_path,
        4,
        false,
        &checks
    );
}

// -------------------------------------------------------------------------------------------------

/// notice the fourth line is an invalid date that will pass regex match
#[allow(non_upper_case_globals)]
const _test_data_file_F_dt6: &str = "\
2001-01-01 00:00:00 _
2001-02-01 00:00:01 😀
2001-03-01 00:00:02 😀😁
2001-04-31 00:00:03 😫😫😫
2001-05-01 00:00:04 😀😁😂😃
2001-06-01 00:00:05 😀😁😂😃😄";

const test_data_file_F_dt6_sysline0: &str = "2001-01-01 00:00:00 _\n";
const test_data_file_F_dt6_sysline1: &str = "2001-02-01 00:00:01 😀\n";
const test_data_file_F_dt6_sysline2: &str = "2001-03-01 00:00:02 😀😁\n2001-04-31 00:00:03 😫😫😫\n";
const test_data_file_F_dt6_sysline3: &str = "2001-05-01 00:00:04 😀😁😂😃\n";

lazy_static! {
    static ref test_SyslineReader_F_ntf: NamedTempFile =
        create_temp_file(_test_data_file_F_dt6);
    static ref test_SyslineReader_F_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_F_ntf);
}

#[test]
fn test_SyslineReader_F_dt6_45()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (45, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline1),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_46()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (46, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline1),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_47()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (47, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_sysline2_sysline3_108_109()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (108, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
            (109, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline3),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_sysline2_sysline3_107_110()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (107, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
            (110, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline3),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_sysline2_sysline3_108_110()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (108, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
            (110, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline3),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_sysline3_sysline2_109_108()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (109, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline3),
            (108, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_sysline3_sysline2_110_107()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (110, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline3),
            (107, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_F_dt6_sysline3_sysline2_109_107()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (109, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline3),
            (107, ResultS4_SyslineFind_Test::Found(()), test_data_file_F_dt6_sysline2),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_F_ntf_path,
        4,
        false,
        &checks
    );
}

// -------------------------------------------------------------------------------------------------

/// notice the first line is an invalid date that will pass regex match
#[allow(non_upper_case_globals)]
const test_data_file_G_dt4: &str = "\
2001-02-31 00:00:01 a
2001-03-01 00:00:02 b
2001-04-01 00:00:03 c
2001-05-01 00:00:04 d";

const test_data_file_G_dt4_sysline0: &str = "2001-03-01 00:00:02 b\n";
const test_data_file_G_dt4_sysline1: &str = "2001-04-01 00:00:03 c\n";
const test_data_file_G_dt4_sysline2: &str = "2001-05-01 00:00:04 d";

lazy_static! {
    static ref test_SyslineReader_G_ntf: NamedTempFile =
        create_temp_file(test_data_file_G_dt4);
    static ref test_SyslineReader_G_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_G_ntf);
}

#[test]
fn test_SyslineReader_G_dt4_0()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (0, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_G_dt4_42()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (42, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_G_dt4_43()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_G_dt4_44()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (44, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline1),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_G_dt4_sysline0_sysline1_43_44()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
            (44, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline1),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_G_dt4_sysline1_sysline0_44_43()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (44, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline1),
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_G_dt4_sysline1_sysline0_44_45_42_43()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (44, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline1),
            (45, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline1),
            (42, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_G_dt4_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_G_ntf_path,
        4,
        false,
        &checks
    );
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
const test_data_file_H_dt4: &str = "\
2001-02-01 00:00:01 a
2001-03-01 00:00:02 b
2001-04-01 00:00:03 c
2001-05-01 00:00:04 d";

const test_data_file_H_dt4_sysline0: &str = "2001-02-01 00:00:01 a\n";
const test_data_file_H_dt4_sysline1: &str = "2001-03-01 00:00:02 b\n";
const test_data_file_H_dt4_sysline2: &str = "2001-04-01 00:00:03 c\n";
const test_data_file_H_dt4_sysline3: &str = "2001-05-01 00:00:04 d";

lazy_static! {
    static ref test_SyslineReader_H_ntf: NamedTempFile =
        create_temp_file(test_data_file_H_dt4);
    static ref test_SyslineReader_H_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_H_ntf);
}

#[test]
fn test_SyslineReader_H_dt4_sysline0()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (0, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (1, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (2, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (3, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (0, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (10, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (20, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
            (21, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline0),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_H_dt4_sysline1()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (22, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline1),
            (22, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline1),
            (22, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline1),
            (43, ResultS4_SyslineFind_Test::Found(()), test_data_file_H_dt4_sysline1),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_H_dt4_sysline3_Found_EOF_Done()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (86, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_file_H_dt4_sysline3),
            (85, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_file_H_dt4_sysline3),
            (87, ResultS4_SyslineFind_Test::Done, ""),
            (88, ResultS4_SyslineFind_Test::Done, ""),
            (87, ResultS4_SyslineFind_Test::Done, ""),
            (66, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_file_H_dt4_sysline3),
            (88, ResultS4_SyslineFind_Test::Done, ""),
            (86, ResultS4_SyslineFind_Test::Found_EOF(()), test_data_file_H_dt4_sysline3),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

#[test]
fn test_SyslineReader_H_dt4_Done()
{
    let checks = test_SyslineReader_any_input_checks::from(
        [
            (87, ResultS4_SyslineFind_Test::Done, ""),
            (88, ResultS4_SyslineFind_Test::Done, ""),
            (87, ResultS4_SyslineFind_Test::Done, ""),
        ]
    );
    test_SyslineReader_any_input_check(
        &test_SyslineReader_H_ntf_path,
        4,
        false,
        &checks
    );
}

// TODO: test `clear_syslines` and `remove_sysline`

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// print the filtered syslines for a SyslineReader
/// quick debug helper for manual reviews
fn process_SyslineReader(
    slr: &mut SyslineReader, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!("{}process_SyslineReader({:?}, {:?}, {:?})", sn(), slr, filter_dt_after_opt, filter_dt_before_opt,);
    let mut fo1: FileOffset = 0;
    let mut search_more = true;
    eprintln!("{}slr.find_sysline_at_datetime_filter({}, {:?})", so(), fo1, filter_dt_after_opt);
    let result = slr.find_sysline_at_datetime_filter(fo1, filter_dt_after_opt);
    match result {
        ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
            eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Found|Found_EOF({}, @{:p})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                fo,
                &*slp
            );
            eprintln!(
                "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                so(),
                fo,
                &(*slp),
                slp.count_lines(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
        }
        ResultS4_SyslineFind::Done => {
            eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Done",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt
            );
            search_more = false;
        }
        ResultS4_SyslineFind::Err(err) => {
            eprintln!(
                "{}slr.find_sysline_at_datetime_filter({}, {:?}, {:?}) returned Err({})",
                so(),
                fo1,
                filter_dt_after_opt,
                filter_dt_before_opt,
                err
            );
            search_more = false;
            panic!("ERROR: {}", err);
        }
    }
    if !search_more {
        eprintln!("{}! search_more", so());
        eprintln!("{}process_SyslineReader(…)", sx());
        return;
    }
    let mut fo2: FileOffset = fo1;
    loop {
        let result = slr.find_sysline(fo2);
        let eof = result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                if eof {
                    eprintln!("{}slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), fo2, fo, &*slp);
                } else {
                    eprintln!("{}slr.find_sysline({}) returned Found({}, @{:p})", so(), fo2, fo, &*slp);
                }
                fo2 = fo;
                eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.count_lines(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                eprintln!(
                    "{}sysline_pass_filters({:?}, {:?}, {:?})",
                    so(),
                    (*slp).dt(),
                    filter_dt_after_opt,
                    filter_dt_before_opt,
                );
                match SyslineReader::sysline_pass_filters(&slp, filter_dt_after_opt, filter_dt_before_opt) {
                    Result_Filter_DateTime2::BeforeRange | Result_Filter_DateTime2::AfterRange => {
                        eprintln!(
                            "{}sysline_pass_filters returned not Result_Filter_DateTime2::InRange; continue!",
                            so()
                        );
                        continue;
                    }
                    Result_Filter_DateTime2::InRange => {
                        if eof {
                            assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is not last!?");
                        } else {
                            assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last!? Should have returned Found_EOF or this Sysline is really not last.");
                        }
                    }
                }
            }
            ResultS4_SyslineFind::Done => {
                eprintln!("{}slr.find_sysline({}) returned Done", so(), fo2);
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo2, err);
                panic!("ERROR: {}", err);
                break;
            }
        }
    }
    eprintln!("{}process_SyslineReader({:?}, …)", sx(), slr.path());
}

/// quick debug helper
#[allow(non_snake_case)]
fn test_SyslineReader_process_file(
    path: &FPath,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeL_Opt,
    filter_dt_before_opt: &DateTimeL_Opt,
) -> Option<Box<SyslineReader>> {
    eprintln!(
        "{}process_file({:?}, {}, {:?}, {:?})",
        sn(),
        &path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let tzo8 = FixedOffset::west(3600 * 8);
    let slr: SyslineReader = new_SyslineReader(
        path,
        blocksz,
        tzo8,
    );
    eprintln!("{}{:?}", so(), slr);
    eprintln!("{}process_file(…)", sx());

    Some(Box::new(slr))
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
fn test_SyslineReader_w_filtering_2(
    path: &FPath, blocksz: BlockSz, filter_dt_after_opt: &DateTimeL_Opt, filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!(
        "{}test_SyslineReader_w_filtering_2({:?}, {}, {:?}, {:?})",
        sn(),
        path,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );
    let slr_opt = test_SyslineReader_process_file(path, blocksz, filter_dt_after_opt, filter_dt_before_opt);
    if slr_opt.is_some() {
        let slr = &slr_opt.unwrap();
        eprintln!("{}Found {} Lines, {} Syslines", so(), slr.count_lines_processed(), slr.count_syslines_stored());
    }
    eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
}

// TODO: add test cases for test_SyslineReader_w_filtering_2

// -------------------------------------------------------------------------------------------------

/// basic test of SyslineReader things
/// process multiple files
#[allow(non_snake_case)]
fn test_SyslineReader_w_filtering_3(
    paths: &Vec<String>,
    blocksz: BlockSz,
    filter_dt_after_opt: &DateTimeL_Opt,
    filter_dt_before_opt: &DateTimeL_Opt,
) {
    eprintln!(
        "{}test_SyslineReader_w_filtering_3({:?}, {}, {:?}, {:?})",
        sn(),
        &paths,
        blocksz,
        filter_dt_after_opt,
        filter_dt_before_opt,
    );

    let mut slrs = Vec::<SyslineReader>::with_capacity(paths.len());
    for path in paths.iter() {
        let tzo8 = FixedOffset::west(3600 * 8);
        eprintln!("{}SyslineReader::new({:?}, {}, {:?})", so(), path, blocksz, tzo8);
        let mut slr = new_SyslineReader(path, blocksz, tzo8);
        eprintln!("{}{:?}", so(), slr);
        slrs.push(slr)
    }
    for slr in slrs.iter_mut() {
        process_SyslineReader(slr, filter_dt_after_opt, filter_dt_before_opt);
        eprintln!();
    }
    eprintln!("{}test_SyslineReader_w_filtering_3(…)", sx());
}

// TODO: add test cases for `test_SyslineReader_w_filtering_3`

// -------------------------------------------------------------------------------------------------

/// basic test of `SyslineReader::find_sysline`
/// read all file offsets but randomly
///
/// TODO: [2021/09] this test was hastily designed for human review. Redesign it for automatic review.
#[allow(non_snake_case)]
fn test_SyslineReader_rand(path: &FPath, blocksz: BlockSz) {
    eprintln!("{}test_SyslineReader_rand({:?}, {})", sn(), path, blocksz);
    let tzo8 = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo8);
    eprintln!("{}SyslineReader {:?}", so(), slr);
    let mut offsets_rand = Vec::<FileOffset>::with_capacity(slr.filesz() as usize);
    fill(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);
    randomize(&mut offsets_rand);
    eprintln!("{}offsets_rand: {:?}", so(), offsets_rand);

    for fo1 in offsets_rand {
        let result = slr.find_sysline(fo1);
        #[allow(clippy::single_match)]
        match result {
            ResultS4_SyslineFind::Err(err) => {
                eprintln!("{}slr.find_sysline({}) returned Err({})", so(), fo1, err);
                panic!("slr.find_sysline({}) returned Err({})", fo1, err);
            }
            _ => {}
        }
    }
    // print the file as-is, it should not be affected by the previous random reads
    // TODO: [2022/03] this should capture printed output and do a direct comparison of files
    eprintln!("\n{}{:?}", so(), slr);
    eprintln!("{}test_SyslineReader_rand(…)", sx());
}

#[test]
fn test_SyslineReader_rand__zero__2() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/zero.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__2() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__4() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__8() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__2() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__4() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__8() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx2__4() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx2.log"), 4);
}

#[test]
fn test_SyslineReader_rand__basic_dt1__4() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/basic-dt1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__4() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__8() {
    test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 8);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type check_find_sysline_in_block = Vec::<(FileOffset, ResultS4_SyslineFind_Test, String)>;

/// test `SyslineReader::find_sysline_in_block`
#[allow(non_snake_case)]
fn test_SyslineReader_find_sysline_in_block(
    path: FPath,
    checks: check_find_sysline_in_block,
    blocksz: BlockSz,
) {
    eprintln!("{}test_SyslineReader_find_sysline_in_block({:?}, {})", sn(), path, blocksz);
    let tzo = FixedOffset::west(3600 * 2);
    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    eprintln!("{}SyslineReader {:?}", so(), slr);

    for (fo_input, result_expect, value_expect) in checks.iter() {
        let result = slr.find_sysline_in_block(*fo_input);
        assert_results4(fo_input, result_expect, &result);
        match result {
            ResultS4_SyslineFind::Found((_fo, slp)) => {
                let value_actual: String = (*slp).to_String();
                assert_eq!(
                    value_expect, &value_actual,
                    "find_sysline_in_block({})\nExpected {:?}\nActual {:?}",
                    fo_input, value_expect, value_actual,
                );
            },
            ResultS4_SyslineFind::Found_EOF((_fo, slp)) => {
                let value_actual: String = (*slp).to_String();
                assert_eq!(
                    value_expect, &value_actual,
                    "find_sysline_in_block({})\nExpected {:?}\nActual {:?}",
                    fo_input, value_expect, value_actual,
                );
            },
            ResultS4_SyslineFind::Done => {
                // self-check
                assert_eq!(
                    value_expect, &String::from(""), "bad test check value {:?}", value_expect,
                );
            },
            ResultS4_SyslineFind::Err(err) => {
                panic!("ERROR: find_sysline_in_block({}) returned Error {:?}", fo_input, err);
            },
        }
    }

    eprintln!("{}test_SyslineReader_find_sysline_in_block(…)", sx());
}

#[test]
fn test_SyslineReader_find_sysline_in_block__empty0() {
    let data: &str = "";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![];

    test_SyslineReader_find_sysline_in_block(path, checks, 2);
}

#[test]
fn test_SyslineReader_find_sysline_in_block__empty1() {
    let data: &str = "\n";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![
        (0, ResultS4_SyslineFind_Test::Done, String::from("")),
    ];

    test_SyslineReader_find_sysline_in_block(path, checks, 2);
}

#[test]
fn test_SyslineReader_find_sysline_in_block__empty2() {
    let data: &str = "\n\n";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![
        (0, ResultS4_SyslineFind_Test::Done, String::from("")),
        (1, ResultS4_SyslineFind_Test::Done, String::from("")),
    ];

    test_SyslineReader_find_sysline_in_block(path, checks, 4);
}

#[test]
fn test_SyslineReader_find_sysline_in_block__empty4() {
    let data: &str = "\n\n\n\n";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![
        (0, ResultS4_SyslineFind_Test::Done, String::from("")),
        (1, ResultS4_SyslineFind_Test::Done, String::from("")),
        (2, ResultS4_SyslineFind_Test::Done, String::from("")),
        (3, ResultS4_SyslineFind_Test::Done, String::from("")),
    ];

    test_SyslineReader_find_sysline_in_block(path, checks, 4);
}

#[test]
fn test_SyslineReader_find_sysline_in_block__1_4() {
    let data: &str = "2000-01-01 00:00:00\n";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![
        (0, ResultS4_SyslineFind_Test::Done, String::from("")),
        (1, ResultS4_SyslineFind_Test::Done, String::from("")),
        (2, ResultS4_SyslineFind_Test::Done, String::from("")),
        (3, ResultS4_SyslineFind_Test::Done, String::from("")),
    ];

    test_SyslineReader_find_sysline_in_block(path, checks, 4);
}

#[test]
fn test_SyslineReader_find_sysline_in_block__1_FF() {
    let data: &str = "2000-01-01 00:00:00\n";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![
        (0, ResultS4_SyslineFind_Test::Found_EOF(()), String::from("2000-01-01 00:00:00\n")),
        (1, ResultS4_SyslineFind_Test::Found_EOF(()), String::from("2000-01-01 00:00:00\n")),
        (2, ResultS4_SyslineFind_Test::Found_EOF(()), String::from("2000-01-01 00:00:00\n")),
        (3, ResultS4_SyslineFind_Test::Found_EOF(()), String::from("2000-01-01 00:00:00\n")),
        (19, ResultS4_SyslineFind_Test::Found_EOF(()), String::from("2000-01-01 00:00:00\n")),
        (20, ResultS4_SyslineFind_Test::Done, String::from("")),
    ];

    test_SyslineReader_find_sysline_in_block(path, checks, 0xFF);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const HOUR: i32 = 3600;
lazy_static!{
    static ref TZO8: FixedOffset = FixedOffset::west(3600 * 8);
    static ref TZO5: FixedOffset = FixedOffset::east(3600 * 5);
}

#[test]
fn test_datetime_parse_from_str_good_without_tz1() {
    // good without timezone
    let dts1 = "2000-01-01 00:01:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = datetime_parse_from_str(dts1, p1, false, &TZO8).unwrap();
    let answer1 = TZO8.ymd(2000, 1, 1).and_hms(0, 1, 1);
    assert_eq!(dt1, answer1);
}

#[test]
fn test_datetime_parse_from_str_2_good_without_tz() {
    // good without timezone
    let dts1 = "2000-01-01 00:02:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = datetime_parse_from_str(dts1, p1, false, &TZO5).unwrap();
    let answer1 = TZO5.ymd(2000, 1, 1).and_hms(0, 2, 1);
    assert_eq!(dt1, answer1);
}

#[test]
fn test_datetime_parse_from_str_3_good_with_tz() {
    // good with timezone
    let dts2 = "2000-01-01 00:00:02 -0100";
    let p2 = "%Y-%m-%d %H:%M:%S %z";
    let dt2 = datetime_parse_from_str(dts2, p2, true, &TZO8).unwrap();
    let answer2 = FixedOffset::west(HOUR).ymd(2000, 1, 1).and_hms(0, 0, 2);
    assert_eq!(dt2, answer2);
}

#[test]
fn test_datetime_parse_from_str_4_bad_with_tz() {
    // bad with timezone
    let dts3 = "2000-01-01 00:00:03 BADD";
    let p3 = "%Y-%m-%d %H:%M:%S %z";
    let dt3 = datetime_parse_from_str(dts3, p3, true, &TZO8);
    assert_eq!(dt3, None);
}

#[test]
fn test_datetime_parse_from_str_5_bad_without_tz() {
    // bad without timezone
    let dts4 = "2000-01-01 00:00:XX";
    let p4 = "%Y-%m-%d %H:%M:%S";
    let dt4 = datetime_parse_from_str(dts4, p4, false, &TZO8);
    assert_eq!(dt4, None);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// given the vector of `DateTimeL`, return the vector index and value of the soonest
/// (minimum) value within a `Some`
/// If the vector is empty then return `None`
#[allow(clippy::ptr_arg)]
fn datetime_soonest2(vec_dt: &Vec<DateTimeL>) -> Option<(usize, DateTimeL)> {
    if vec_dt.is_empty() {
        return None;
    }

    let mut index: usize = 0;
    for (index_, _) in vec_dt.iter().enumerate() {
        if vec_dt[index_] < vec_dt[index] {
            index = index_;
        }
    }

    Some((index, vec_dt[index]))
}

/// test function `datetime_soonest2`
#[test]
fn test_datetime_soonest2() {
    eprintln!("{}test_datetime_soonest2()", sn());
    let vec0 = Vec::<DateTimeL>::with_capacity(0);
    let val = datetime_soonest2(&vec0);
    assert!(val.is_none());
    let tzo = FixedOffset::west(3600 * 8);

    let dt1_a = datetime_parse_from_str("2001-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_a];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let dt1_b = datetime_parse_from_str("2001-01-01T12:00:00-0100", "%Y-%m-%dT%H:%M:%S%z", true, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_b];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1b");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_b);

    let dt2_a = datetime_parse_from_str("2002-01-01T11:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec2a: Vec<DateTimeL> = vec![dt1_a, dt2_a];
    let (i_, dt_) = match datetime_soonest2(&vec2a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let vec2b: Vec<DateTimeL> = vec![dt2_a, dt1_a];
    let (i_, dt_) = match datetime_soonest2(&vec2b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None2b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt1_a);

    let dt3 = datetime_parse_from_str("2000-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec3a: Vec<DateTimeL> = vec![dt1_a, dt2_a, dt3];
    let (i_, dt_) = match datetime_soonest2(&vec3a) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3a");
        }
    };
    assert_eq!(i_, 2);
    assert_eq!(dt_, dt3);

    let vec3b: Vec<DateTimeL> = vec![dt1_a, dt3, dt2_a];
    let (i_, dt_) = match datetime_soonest2(&vec3b) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3b");
        }
    };
    assert_eq!(i_, 1);
    assert_eq!(dt_, dt3);

    let vec3c: Vec<DateTimeL> = vec![dt3, dt1_a, dt2_a];
    let (i_, dt_) = match datetime_soonest2(&vec3c) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None3c");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt3);

    eprintln!("{}test_datetime_soonest2()", sx());
}
