// Readers/syslinereader_tests.rs
//

use crate::common::{
    FPath,
    FileType,
    ResultS4,
};

use crate::Readers::blockreader::{
    FileOffset,
    BlockSz,
};

use crate::Readers::filepreprocessor::{
    guess_filetype_from_fpath,
};

use crate::Readers::helpers::{
    randomize,
    fill,
};

use crate::Readers::syslinereader::{
    SyslineP,
    SyslineReader,
    ResultS4_SyslineFind,
};

use crate::Data::datetime::{
    FixedOffset,
    TimeZone,
    dt_pattern_has_tz,
    str_datetime,
    DateTimePattern,
    DateTimeL,
    DateTimeL_Opt,
    Result_Filter_DateTime2,
};

use crate::printer_debug::helpers::{
    NamedTempFile,
    create_temp_file,
    create_temp_file_bytes,
    NTF_Path,
    eprint_file,
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

use crate::printer::printers::{
    write_stderr,
};

use std::str;

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper to wrap the match and panic checks
#[cfg(test)]
fn new_SyslineReader(path: &FPath, blocksz: BlockSz, tzo: FixedOffset) -> SyslineReader {
    stack_offset_set(Some(2));
    let filetype: FileType = guess_filetype_from_fpath(path);
    match SyslineReader::new(path.clone(), filetype, blocksz, tzo) {
        Ok(val) => val,
        Err(err) => {
            panic!("ERROR: SyslineReader::new({:?}, {:?}, {:?}) failed {}", path, blocksz, tzo, err);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// basic test of `SyslineReader.find_datetime_in_line`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_find_datetime_in_line_by_block(blocksz: BlockSz) {
    eprintln!("{}_test_find_datetime_in_line_by_block()", sn());

    let ntf: NamedTempFile = create_temp_file(
        "\
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
"
    );
    let path = NTF_Path(&ntf);

    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(&path, blocksz, tzo);

    let mut fo1: FileOffset = 0;
    loop {
        let result = slr.find_sysline(fo1);
        let done = result.is_done() || result.is_eof();
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                eprintln!("{}test_find_datetime_in_line: slr.find_sysline({}) returned Found|Found_EOF({}, @{:p})", so(), fo1, fo, &*slp);
                eprintln!(
                    "{}test_find_datetime_in_line: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                //eprint_syslinep(&slp);
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

    eprintln!("{}_test_find_datetime_in_line_by_block()", sx());
}


#[test]
fn test_find_datetime_in_line_by_block2() {
    _test_find_datetime_in_line_by_block(2);
}

#[test]
fn test_find_datetime_in_line_by_block4() {
    _test_find_datetime_in_line_by_block(4);
}

#[test]
fn test_find_datetime_in_line_by_block8() {
    _test_find_datetime_in_line_by_block(8);
}

#[test]
fn test_find_datetime_in_line_by_block256() {
    _test_find_datetime_in_line_by_block(256);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type _test_find_sysline_at_datetime_filter_Checks<'a> = Vec<(FileOffset, &'a str, &'a str)>;

/// underlying test code for `SyslineReader.find_datetime_in_line`
/// called by other functions `test_find_sysline_at_datetime_filterX`
#[cfg(test)]
fn __test_find_sysline_at_datetime_filter(
    file_content: String,
    dt_pattern: DateTimePattern,
    blocksz: BlockSz,
    checks: _test_find_sysline_at_datetime_filter_Checks,
) {
    eprintln!("{}__test_find_sysline_at_datetime_filter(…, {:?}, {}, …)", sn(), dt_pattern, blocksz);

    let ntf: NamedTempFile = create_temp_file(file_content.as_str());
    let path = NTF_Path(&ntf);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    for (fo1, dts, sline_expect) in checks.iter() {
        // TODO: add `has_tz` to `checks`, remove this
        let has_tz = dt_pattern_has_tz(dt_pattern.as_str());
        eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", so(), str_to_String_noraw(dts), dt_pattern, has_tz, &tzo);
        let dt = match str_datetime(dts, dt_pattern.as_str(), has_tz, &tzo) {
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

    eprintln!("{}_test_find_sysline_at_datetime_filter(…)", sx());
}

// -------------------------------------------------------------------------------------------------

// TODO: [2022/03/16] create test cases with varying sets of Checks passed-in, current setup is always
//       clean, sequential series of checks from file_offset 0.
// TODO: BUG: [2022/03/15] why are these checks done in random order? The tests pass but run
//       in a confusing manner. Run `cargo test` to see.
/// basic test of `SyslineReader.find_datetime_in_line`
#[cfg(test)]
fn _test_find_sysline_at_datetime_filter(
    blocksz: BlockSz, checks: Option<_test_find_sysline_at_datetime_filter_Checks>,
) {
    stack_offset_set(None);
    eprintln!("{}_test_find_sysline_at_datetime_filter()", sn());
    let dt_fmt1: DateTimePattern = String::from("%Y-%m-%d %H:%M:%S");
    let file_content1 = String::from(
        "\
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
",
    );
    let checks0: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
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
        (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let _checksx: _test_find_sysline_at_datetime_filter_Checks = Vec::from([
        (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        (19, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        (40, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        (62, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        (85, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
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
        (755, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
        (799, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        (844, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
    ]);

    let checks_: _test_find_sysline_at_datetime_filter_Checks = checks.unwrap_or(checks0);
    __test_find_sysline_at_datetime_filter(file_content1, dt_fmt1, blocksz, checks_);
    eprintln!("{}_test_find_sysline_at_datetime_filter()", sx());
}

// XXX: are these different BlockSz tests necessary? are not these adequately tested by
//      other lower-level tests?

#[test]
fn test_find_sysline_at_datetime_filter_4() {
    _test_find_sysline_at_datetime_filter(4, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_8() {
    _test_find_sysline_at_datetime_filter(8, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_16() {
    _test_find_sysline_at_datetime_filter(16, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_32() {
    _test_find_sysline_at_datetime_filter(32, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_64() {
    _test_find_sysline_at_datetime_filter(64, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_128() {
    _test_find_sysline_at_datetime_filter(128, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_256() {
    _test_find_sysline_at_datetime_filter(256, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_512() {
    _test_find_sysline_at_datetime_filter(512, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_1024() {
    _test_find_sysline_at_datetime_filter(1024, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_2056() {
    _test_find_sysline_at_datetime_filter(2056, None);
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_() {
    _test_find_sysline_at_datetime_filter(64,Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:00",
            "2020-01-01 00:00:00\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_0_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            0,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_a() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            19,
            "2020-01-01 00:00:01",
            "2020-01-01 00:00:01a\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_b() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            40,
            "2020-01-01 00:00:02",
            "2020-01-01 00:00:02ab\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_c() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            62,
            "2020-01-01 00:00:03",
            "2020-01-01 00:00:03abc\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_d() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            85,
            "2020-01-01 00:00:04",
            "2020-01-01 00:00:04abcd\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_e() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            109,
            "2020-01-01 00:00:05",
            "2020-01-01 00:00:05abcde\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_f() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            134,
            "2020-01-01 00:00:06",
            "2020-01-01 00:00:06abcdef\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_g() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            160,
            "2020-01-01 00:00:07",
            "2020-01-01 00:00:07abcdefg\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_h() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            187,
            "2020-01-01 00:00:08",
            "2020-01-01 00:00:08abcdefgh\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_i() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            215,
            "2020-01-01 00:00:09",
            "2020-01-01 00:00:09abcdefghi\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_j() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            244,
            "2020-01-01 00:00:10",
            "2020-01-01 00:00:10abcdefghij\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_k() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            274,
            "2020-01-01 00:00:11",
            "2020-01-01 00:00:11abcdefghijk\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_l() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            305,
            "2020-01-01 00:00:12",
            "2020-01-01 00:00:12abcdefghijkl\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_m() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            337,
            "2020-01-01 00:00:13",
            "2020-01-01 00:00:13abcdefghijklm\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_n() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            370,
            "2020-01-01 00:00:14",
            "2020-01-01 00:00:14abcdefghijklmn\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_o() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            404,
            "2020-01-01 00:00:15",
            "2020-01-01 00:00:15abcdefghijklmno\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_p() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            439,
            "2020-01-01 00:00:16",
            "2020-01-01 00:00:16abcdefghijklmnop\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_q() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            475,
            "2020-01-01 00:00:17",
            "2020-01-01 00:00:17abcdefghijklmnopq\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_r() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            512,
            "2020-01-01 00:00:18",
            "2020-01-01 00:00:18abcdefghijklmnopqr\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_s() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            550,
            "2020-01-01 00:00:19",
            "2020-01-01 00:00:19abcdefghijklmnopqrs\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_t() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            589,
            "2020-01-01 00:00:20",
            "2020-01-01 00:00:20abcdefghijklmnopqrst\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_u() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            629,
            "2020-01-01 00:00:21",
            "2020-01-01 00:00:21abcdefghijklmnopqrstu\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_v() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            670,
            "2020-01-01 00:00:22",
            "2020-01-01 00:00:22abcdefghijklmnopqrstuv\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_w() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            712,
            "2020-01-01 00:00:23",
            "2020-01-01 00:00:23abcdefghijklmnopqrstuvw\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_x() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            755,
            "2020-01-01 00:00:24",
            "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_y() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            799,
            "2020-01-01 00:00:25",
            "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_x_z() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([(
            844,
            "2020-01-01 00:00:26",
            "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n",
        )])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_z_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_y_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_x_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_za() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ya() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_xa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:24", "2020-01-01 00:00:24abcdefghijklmnopqrstuvwx\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_2_ma() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3____() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ab() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__az() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__bd() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:04", "2020-01-01 00:00:04abcd\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__ml() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:12", "2020-01-01 00:00:12abcdefghijkl\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__my() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__mz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3__m_() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
            (0, "2020-01-01 00:00:13", "2020-01-01 00:00:13abcdefghijklm\n"),
            (0, "2020-01-01 00:00:00", "2020-01-01 00:00:00\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_abz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_aaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_byo() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:15", "2020-01-01 00:00:15abcdefghijklmno\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zbz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_zaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaa() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybc() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:03", "2020-01-01 00:00:03abc\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yba() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybn() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:14", "2020-01-01 00:00:14abcdefghijklmn\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yby() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_ybz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:02", "2020-01-01 00:00:02ab\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

#[test]
fn test_find_sysline_at_datetime_filter_checks_3_yaz() {
    _test_find_sysline_at_datetime_filter(
        64,
        Some(_test_find_sysline_at_datetime_filter_Checks::from([
            (0, "2020-01-01 00:00:25", "2020-01-01 00:00:25abcdefghijklmnopqrstuvwxy\n"),
            (0, "2020-01-01 00:00:01", "2020-01-01 00:00:01a\n"),
            (0, "2020-01-01 00:00:26", "2020-01-01 00:00:26abcdefghijklmnopqrstuvwxyz\n"),
        ])),
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: [2022/03/18] create one wrapper test test_find_sysline_at_datetime_checks_ that takes some
//        vec of test-input-output, and does all possible permutations.

/// testing helper
/// not efficient
/// XXX: does not handle multi-byte
#[cfg(test)]
fn eprint_syslinep(slp: &SyslineP) {
    let slices = (*slp).get_slices();
    for slice in slices.iter() {
        // XXX: this write is seen during all `cargo test`, I'd prefer it not be.
        //      Bit I'm not sure what's different about `stderr::write` versus `eprint!`
        write_stderr(slice);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
type _test_SyslineReader_check<'a> = (&'a str, FileOffset);

#[cfg(test)]
type _test_SyslineReader_checks<'a> = Vec<(&'a str, FileOffset)>;

/// basic linear test of `SyslineReader::find_sysline`
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_find_sysline(
    path: &FPath,
    blocksz: BlockSz,
    fileoffset: FileOffset,
    checks: &_test_SyslineReader_checks
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
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                eprint_syslinep(&slp);
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");
                fo1 = fo;

                if checks.is_empty() {
                    continue;
                }
                eprintln!("{}test_SyslineReader_find_sysline: find_sysline({}); check {} expect ({:?}, {:?})", so(), fo1, check_i, checks[check_i].1, checks[check_i].0);
                // check slp.String
                let check_String = checks[check_i].0.to_string();
                let actual_String = (*slp).to_String();
                assert_eq!(check_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}", check_String, fo1, actual_String);
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
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                eprint_syslinep(&slp);
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

    eprintln!("{}test_SyslineReader_find_sysline: Found {} Lines, {} Syslines", so(), slr.linereader.count_lines_processed(), slr.syslines.len());
    eprintln!("{}test_SyslineReader_find_sysline({:?}, {})", sx(), &path, blocksz);
}

#[allow(non_upper_case_globals)]
static test_data_file_A_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
static test_data_file_A_dt6_checks: [_test_SyslineReader_check; 6] = [
    ("2000-01-01 00:00:00\n", 20),
    ("2000-01-01 00:00:01a\n", 41),
    ("2000-01-01 00:00:02ab\n", 63),
    ("2000-01-01 00:00:03abc\n", 86),
    ("2000-01-01 00:00:04abcd\n", 110),
    ("2000-01-01 00:00:05abcde", 134),
];

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_A_ntf: NamedTempFile =
        create_temp_file(test_data_file_A_dt6);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_A_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_A_ntf);
}

#[test]
fn test_SyslineReader_A_dt6_128_0_()
{
    let checks = _test_SyslineReader_checks::from(test_data_file_A_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 0, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_1_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[1..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 40, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_2_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[2..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 62, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_3_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[3..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 85, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_4_()
{
    let checks = _test_SyslineReader_checks::from(&test_data_file_A_dt6_checks[4..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 86, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X_beforeend()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 132, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X_pastend()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 135, &checks);
}

#[test]
fn test_SyslineReader_A_dt6_128_X9999()
{
    let checks = _test_SyslineReader_checks::from([]);
    test_SyslineReader_find_sysline(&test_SyslineReader_A_ntf_path, 128, 9999, &checks);
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

#[cfg(test)]
type _test_SyslineReader_any_input_check<'a> = (FileOffset, ResultS4_SyslineFind_Test, &'a str);

#[cfg(test)]
type _test_SyslineReader_any_input_checks<'a> = Vec<(FileOffset, ResultS4_SyslineFind_Test, &'a str)>;

/// test of `SyslineReader::find_sysline` with test-specified fileoffset searches
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_A2_any_input_check(
    path: &FPath,
    blocksz: BlockSz,
    lru_cache_enable: bool,
    input_checks: &[_test_SyslineReader_any_input_check],
) {
    stack_offset_set(Some(2));
    eprintln!("{}test_SyslineReader_any_input_check({:?}, {})", sn(), path, blocksz);
    eprint_file(path);
    let tzo = FixedOffset::west(3600 * 8);
    let mut slr = new_SyslineReader(path, blocksz, tzo);
    if lru_cache_enable {
        slr.LRU_cache_disable();
    }

    let mut check_i: usize = 0;
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
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                //eprint_syslinep(&slp);
                assert!(!slr.is_sysline_last(&slp), "returned Found yet this Sysline is last! Should have returned Found_EOF or is this Sysline not last?");

                let actual_String = (*slp).to_String();
                let expect_String = String::from(*expect_val);
                eprintln!("{}test_SyslineReader: find_sysline({}); check {}", so(), input_fo, check_i);
                assert_eq!(expect_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}", expect_String, input_fo, actual_String);
            }
            ResultS4_SyslineFind::Found_EOF((_fo, slp)) => {
                eprintln!("{}test_SyslineReader: slr.find_sysline({}) returned Found_EOF({}, @{:p})", so(), input_fo, _fo, &*slp);
                eprintln!(
                    "{}test_SyslineReader: FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    _fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                //eprint_syslinep(&slp);
                assert!(slr.is_sysline_last(&slp), "returned Found_EOF yet this Sysline is oot last! Should have returned Found or is this Sysline last?");

                let actual_String = (*slp).to_String();
                let expect_String = String::from(*expect_val);
                eprintln!("{}test_SyslineReader: find_sysline({}); check {}", so(), input_fo, check_i);
                assert_eq!(expect_String, actual_String,"\nexpected string value     {:?}\nfind_sysline({:?}) returned {:?}", expect_String, input_fo, actual_String);
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
    }
    assert_eq!(input_checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", input_checks.len(), check_i);

    eprintln!("{}test_SyslineReader: Found {} Lines, {} Syslines", so(), slr.linereader.count_lines_processed(), slr.syslines.len());
    eprintln!("{}test_SyslineReader({:?}, {})", sx(), &path, blocksz);
}

#[allow(non_upper_case_globals)]
static test_data_any_file_A2_dt6: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

/// like `dt6_checks` but many more checks
#[allow(non_upper_case_globals)]
static test_data_any_file_A2_dt6_checks_many: [_test_SyslineReader_any_input_check; 50] = [
    (0, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (1, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (2, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (3, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (4, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (5, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (6, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (19, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:00\n"),
    (20, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (21, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (22, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (23, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (24, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (25, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (40, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:01a\n"),
    (41, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (42, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (43, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (44, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (45, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (46, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (47, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (61, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (62, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:02ab\n"),
    (63, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (64, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (65, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (66, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (67, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (84, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (85, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:03abc\n"),
    (86, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (87, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (88, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (89, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (90, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (108, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (109, ResultS4_SyslineFind_Test::Found(()), "2000-01-01 00:00:04abcd\n"),
    (110, ResultS4_SyslineFind_Test::Found_EOF(()), "2000-01-01 00:00:05abcde"),
    (111, ResultS4_SyslineFind_Test::Found_EOF(()), "2000-01-01 00:00:05abcde"),
    (112, ResultS4_SyslineFind_Test::Found_EOF(()), "2000-01-01 00:00:05abcde"),
    (113, ResultS4_SyslineFind_Test::Found_EOF(()), "2000-01-01 00:00:05abcde"),
    (114, ResultS4_SyslineFind_Test::Found_EOF(()), "2000-01-01 00:00:05abcde"),
    (134, ResultS4_SyslineFind_Test::Done, ""),
    (135, ResultS4_SyslineFind_Test::Done, ""),
    (136, ResultS4_SyslineFind_Test::Done, ""),
    (137, ResultS4_SyslineFind_Test::Done, ""),
    (138, ResultS4_SyslineFind_Test::Done, ""),
    (139, ResultS4_SyslineFind_Test::Done, ""),
    (140, ResultS4_SyslineFind_Test::Done, ""),
];

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_A2_any_ntf: NamedTempFile = 
        create_temp_file(test_data_any_file_A2_dt6);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_A2_any_ntf_path: FPath = 
        NTF_Path(&test_SyslineReader_A2_any_ntf);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn test_SyslineReader_A2_dt6_any_2()
{
    test_SyslineReader_A2_any_input_check(&test_SyslineReader_A2_any_ntf_path, 2, true, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_2_noLRUcache()
{
    test_SyslineReader_A2_any_input_check(&test_SyslineReader_A2_any_ntf_path, 2, false, &test_data_any_file_A2_dt6_checks_many);
}


#[test]
fn test_SyslineReader_A2_dt6_any_4()
{
    test_SyslineReader_A2_any_input_check(&test_SyslineReader_A2_any_ntf_path, 4, true, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_4_noLRUcache()
{
    test_SyslineReader_A2_any_input_check(&test_SyslineReader_A2_any_ntf_path, 4, false, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_0xFF()
{
    test_SyslineReader_A2_any_input_check(&test_SyslineReader_A2_any_ntf_path, 0xFF, true, &test_data_any_file_A2_dt6_checks_many);
}

#[test]
fn test_SyslineReader_A2_dt6_any_0xFF_noLRUcache()
{
    test_SyslineReader_A2_any_input_check(&test_SyslineReader_A2_any_ntf_path, 0xFF, false, &test_data_any_file_A2_dt6_checks_many);
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
static test_data_file_B_dt0: &str = "
foo
bar
";

#[allow(non_upper_case_globals)]
static test_data_file_B_dt0_checks: [_test_SyslineReader_check; 0] = [];

#[test]
fn test_SyslineReader_B_dt0_0()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = NTF_Path(&ntf);
    let checks = _test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader_find_sysline(&path, 128, 0, &checks);
}

#[test]
fn test_SyslineReader_B_dt0_3()
{
    let ntf = create_temp_file(test_data_file_B_dt0);
    let path = NTF_Path(&ntf);
    let checks = _test_SyslineReader_checks::from(test_data_file_B_dt0_checks);
    test_SyslineReader_find_sysline(&path, 128, 3, &checks);
}

// -------------------------------------------------------------------------------------------------

#[allow(non_upper_case_globals)]
static _test_data_file_C_dt6: &str = "\
[DEBUG] 2000-01-01 00:00:00
[DEBUG] 2000-01-01 00:00:01a
[DEBUG] 2000-01-01 00:00:02ab
[DEBUG] 2000-01-01 00:00:03abc
[DEBUG] 2000-01-01 00:00:04abcd
[DEBUG] 2000-01-01 00:00:05abcde";

#[allow(non_upper_case_globals)]
static _test_data_file_C_dt6_checks: [_test_SyslineReader_check; 6] = [
    ("[DEBUG] 2000-01-01 00:00:00\n", 28),
    ("[DEBUG] 2000-01-01 00:00:01a\n", 57),
    ("[DEBUG] 2000-01-01 00:00:02ab\n", 87),
    ("[DEBUG] 2000-01-01 00:00:03abc\n", 118),
    ("[DEBUG] 2000-01-01 00:00:04abcd\n", 150),
    ("[DEBUG] 2000-01-01 00:00:05abcde", 182),
];

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_C_ntf: NamedTempFile =
        create_temp_file(_test_data_file_C_dt6);
}

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref test_SyslineReader_C_ntf_path: FPath =
        NTF_Path(&test_SyslineReader_C_ntf);
}

#[test]
fn test_SyslineReader_C_dt6_0()
{
    let checks = _test_SyslineReader_checks::from(_test_data_file_C_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 0, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_3()
{
    let checks = _test_SyslineReader_checks::from(_test_data_file_C_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 3, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_27()
{
    let checks = _test_SyslineReader_checks::from(_test_data_file_C_dt6_checks);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 27, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_28_1__()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[1..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 28, &checks);
}

#[test]
fn test_SyslineReader_C_dt6_29_1__()
{
    let checks = _test_SyslineReader_checks::from(&_test_data_file_C_dt6_checks[1..]);
    test_SyslineReader_find_sysline(&test_SyslineReader_C_ntf_path, 128, 29, &checks);
}

#[test]
fn test_SyslineReader_D_invalid1()
{
    let data_invalid1: [u8; 1] = [ 0xFF ];
    let date_checks1: _test_SyslineReader_checks = _test_SyslineReader_checks::from([]);
    let ntf = create_temp_file_bytes(&data_invalid1);
    let path = NTF_Path(&ntf);
    test_SyslineReader_find_sysline(&path, 128, 0, &date_checks1);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/* already implemented above

#[cfg(test)]
type _test_SyslineReader_w_filtering_1_check<'a> = (
    FileOffset, ResultS4_SyslineFind_Test, DateTimeL_Opt,
);

/// basic test of SyslineReader sequential read with datetime filtering
#[allow(non_snake_case)]
#[cfg(test)]
fn test_SyslineReader_find_sysline_at_datetime_filter(
    ntf: &NamedTempFile,
    blocksz: BlockSz,
    tzo: FixedOffset,
    filter_dt_after_opt: &DateTimeL_Opt,
    checks: &[_test_SyslineReader_w_filtering_1_check],
) {
    eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?})",
        sn(),
        &ntf,
        blocksz,
        filter_dt_after_opt,
    );

    let path: FPath = FPath::from(ntf.path().to_str().unwrap());
    eprint_file(&path);

    let mut slr = new_SyslineReader(&path, blocksz, tzo);
    eprintln!("{}{:?}", so(), slr);

    let filesz = slr.filesz();
    let mut fo1: FileOffset = 0;
    let mut check_i: usize = 0;
    for (input_fo, expect_result, expect_datetime) in checks.iter() {
        let result = slr.find_sysline_at_datetime_filter(*input_fo, filter_dt_after_opt);
        assert_results4(input_fo, expect_result, &result);
        match result {
            ResultS4_SyslineFind::Found((fo, slp)) | ResultS4_SyslineFind::Found_EOF((fo, slp)) => {
                eprintln!(
                    "{}FileOffset {} Sysline @{:p}: line count {} sysline.len() {} {:?}",
                    so(),
                    fo,
                    &(*slp),
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                let dt_actual: &DateTimeL_Opt = &(*slp).dt;
                assert_eq!(expect_datetime, dt_actual, "Expected datetime    {:?}\nSysline actual datetime {:?}", expect_datetime, dt_actual);
            }
            ResultS4_SyslineFind::Done => {
                break;
            }
            ResultS4_SyslineFind::Err(err) => {
                panic!(
                    "ERROR: find_sysline_at_datetime_filter({}, {:?}) returned Err({})",
                    input_fo,
                    filter_dt_after_opt,
                    err,
                );
            }
        }
        check_i += 1;
    }

    assert_eq!(checks.len(), check_i, "expected {} Sysline checks but only {} Sysline checks were done", checks.len(), check_i);
    eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count_lines_processed(), slr.syslines.len());
    eprintln!(
        "{}test_SyslineReader_w_filtering_1({:?}, {}, {:?})",
        sx(),
        &path,
        blocksz,
        filter_dt_after_opt,
    );
}

#[allow(non_upper_case_globals)]
static _test_data_dtA: &str = "\
2000-01-01 00:00:00
2000-01-01 00:00:01a
2000-01-01 00:00:02ab
2000-01-01 00:00:03abc
2000-01-01 00:00:04abcd
2000-01-01 00:00:05abcde";

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref Test_data_file_dtA_ntf: NamedTempFile = {
        create_temp_file(_test_data_dtA)
    };
}

#[test]
fn test_SyslineReader_find_sysline_at_datetime_filter_0() {
    let tzo8 = FixedOffset::west(3600 * 8);
    let checks: [_test_SyslineReader_w_filtering_1_check; 0] = [];
    test_SyslineReader_find_sysline_at_datetime_filter(
        &Test_data_file_dtA_ntf,
        4,
        tzo8,
        &None,
        &checks,
    );
}

#[test]
fn test_SyslineReader_find_sysline_at_datetime_filter_1() {
    let tzo8 = FixedOffset::west(3600 * 8);
    let checks: [_test_SyslineReader_w_filtering_1_check; 1] = [
        (0, ResultS4_SyslineFind_Test::Found, DateTimeL_Opt::Some(DateTimeL::from())),
    ];
    test_SyslineReader_find_sysline_at_datetime_filter(
        &Test_data_file_dtA_ntf,
        4,
        tzo8,
        &None,
        &checks,
    );
}

*/

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// print the filtered syslines for a SyslineReader
/// quick debug helper for manual reviews
#[cfg(test)]
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
                slp.lines.len(),
                (*slp).len(),
                (*slp).to_String_noraw(),
            );
            fo1 = fo;
            eprint_syslinep(&slp);
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
                    slp.lines.len(),
                    (*slp).len(),
                    (*slp).to_String_noraw(),
                );
                eprintln!(
                    "{}sysline_pass_filters({:?}, {:?}, {:?})",
                    so(),
                    (*slp).dt,
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
                        eprint_syslinep(&slp);
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
#[cfg(test)]
fn _test_SyslineReader_process_file(
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
    let slr = new_SyslineReader(path, blocksz, tzo8);
    eprintln!("{}{:?}", so(), slr);
    eprintln!("{}process_file(…)", sx());

    Some(Box::new(slr))
}

/// basic test of SyslineReader things
#[allow(non_snake_case)]
#[cfg(test)]
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
    let slr_opt = _test_SyslineReader_process_file(path, blocksz, filter_dt_after_opt, filter_dt_before_opt);
    if slr_opt.is_some() {
        let slr = &slr_opt.unwrap();
        eprintln!("{}Found {} Lines, {} Syslines", so(), slr.linereader.count_lines_processed(), slr.syslines.len());
    }
    eprintln!("{}test_SyslineReader_w_filtering_2(…)", sx());
}

// TODO: add test cases for test_SyslineReader_w_filtering_2

// -------------------------------------------------------------------------------------------------

/// basic test of SyslineReader things
/// process multiple files
#[allow(non_snake_case)]
#[cfg(test)]
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
#[cfg(test)]
fn _test_SyslineReader_rand(path: &FPath, blocksz: BlockSz) {
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
    //slr.print_all(true);
    eprintln!("\n{}{:?}", so(), slr);
    eprintln!("{}test_SyslineReader_rand(…)", sx());
}

#[test]
fn test_SyslineReader_rand__zero__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/zero.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__2() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 2);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 4);
}

#[test]
fn test_SyslineReader_rand__test0_nlx1_Win__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx1_Win.log"), 8);
}

#[test]
fn test_SyslineReader_rand__test0_nlx2__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/test0-nlx2.log"), 4);
}

#[test]
fn test_SyslineReader_rand__basic_dt1__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/basic-dt1.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__4() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 4);
}

#[test]
fn test_SyslineReader_rand__dtf5_6c__8() {
    _test_SyslineReader_rand(&FPath::from("./logs/other/tests/dtf5-6c.log"), 8);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

type check_find_sysline_in_block = Vec::<(FileOffset, ResultS4_SyslineFind_Test, String)>;

/// test `SyslineReader::find_sysline_in_block`
#[allow(non_snake_case)]
#[cfg(test)]
fn _test_SyslineReader_find_sysline_in_block(
    path: FPath,
    checks: check_find_sysline_in_block,
    blocksz: BlockSz,
) {
    eprintln!("{}_test_SyslineReader_find_sysline_in_block({:?}, {})", sn(), path, blocksz);
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

    eprintln!("{}_test_SyslineReader_find_sysline_in_block(…)", sx());
}

#[test]
fn test_SyslineReader_find_sysline_in_block__empty0() {
    let data: &str = "";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![];

    _test_SyslineReader_find_sysline_in_block(path, checks, 2);
}

#[test]
fn test_SyslineReader_find_sysline_in_block__empty1() {
    let data: &str = "\n";
    let ntf = create_temp_file(data);
    let path = NTF_Path(&ntf);
    let checks: check_find_sysline_in_block = vec![
        (0, ResultS4_SyslineFind_Test::Done, String::from("")),
    ];

    _test_SyslineReader_find_sysline_in_block(path, checks, 2);
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

    _test_SyslineReader_find_sysline_in_block(path, checks, 4);
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

    _test_SyslineReader_find_sysline_in_block(path, checks, 4);
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

    _test_SyslineReader_find_sysline_in_block(path, checks, 4);
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

    _test_SyslineReader_find_sysline_in_block(path, checks, 0xFF);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_str_datetime() {
    let hour = 3600;
    let tzo8 = FixedOffset::west(3600 * 8);
    let tzo5 = FixedOffset::east(3600 * 5);

    // good without timezone
    let dts1 = "2000-01-01 00:01:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = str_datetime(dts1, p1, false, &tzo8).unwrap();
    let answer1 = tzo8.ymd(2000, 1, 1).and_hms(0, 1, 1);
    assert_eq!(dt1, answer1);

    // good without timezone
    let dts1 = "2000-01-01 00:02:01";
    let p1 = "%Y-%m-%d %H:%M:%S";
    let dt1 = str_datetime(dts1, p1, false, &tzo5).unwrap();
    let answer1 = tzo5.ymd(2000, 1, 1).and_hms(0, 2, 1);
    assert_eq!(dt1, answer1);

    // good with timezone
    let dts2 = "2000-01-01 00:00:02 -0100";
    let p2 = "%Y-%m-%d %H:%M:%S %z";
    let dt2 = str_datetime(dts2, p2, true, &tzo8).unwrap();
    let answer2 = FixedOffset::west(hour).ymd(2000, 1, 1).and_hms(0, 0, 2);
    assert_eq!(dt2, answer2);

    // bad with timezone
    let dts3 = "2000-01-01 00:00:03 BADD";
    let p3 = "%Y-%m-%d %H:%M:%S %z";
    let dt3 = str_datetime(dts3, p3, true, &tzo8);
    assert_eq!(dt3, None);

    // bad without timezone
    let dts4 = "2000-01-01 00:00:XX";
    let p4 = "%Y-%m-%d %H:%M:%S";
    let dt4 = str_datetime(dts4, p4, false, &tzo8);
    assert_eq!(dt4, None);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// given the vector of `DateTimeL`, return the vector index and value of the soonest
/// (minimum) value within a `Some`
/// If the vector is empty then return `None`
#[cfg(test)]
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

    let dt1_a = str_datetime("2001-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_a];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1a");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_a);

    let dt1_b = str_datetime("2001-01-01T12:00:00-0100", "%Y-%m-%dT%H:%M:%S%z", true, &tzo).unwrap();
    let vec1: Vec<DateTimeL> = vec![dt1_b];
    let (i_, dt_) = match datetime_soonest2(&vec1) {
        Some(val) => val,
        None => {
            panic!("datetime_soonest2 returned None1b");
        }
    };
    assert_eq!(i_, 0);
    assert_eq!(dt_, dt1_b);

    let dt2_a = str_datetime("2002-01-01T11:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
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

    let dt3 = str_datetime("2000-01-01T12:00:00", "%Y-%m-%dT%H:%M:%S", false, &tzo).unwrap();
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
