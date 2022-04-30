// Readers/datetime.rs

#[cfg(any(debug_assertions,test))]
use crate::dbgpr::printers::{
    str_to_String_noraw,
};

use crate::dbgpr::stack::{
    sn,
    snx,
    so,
    sx,
};

use crate::Readers::linereader::{
    LineIndex,
};

extern crate arrayref;
use arrayref::array_ref;

extern crate chrono;
pub(crate) use chrono::{
    DateTime,
    FixedOffset,
    Local,
    NaiveDateTime,
    TimeZone,
    Utc,
};

extern crate debug_print;
use debug_print::debug_eprintln;

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate more_asserts;
use more_asserts::{
    assert_le,
};

extern crate unroll;
use unroll::unroll_for_loops;

/// DateTime formatting pattern, passed to `chrono::datetime_from_str`
pub type DateTimePattern = String;
pub type DateTimePattern_str = str;
/// typical DateTime with TZ type
pub type DateTimeL = DateTime<FixedOffset>;
#[allow(non_camel_case_types)]
pub type DateTimeL_Opt = Option<DateTimeL>;

/// DateTimePattern for searching a line (not the results)
/// slice index begin, slice index end of entire datetime pattern
/// slice index begin just the datetime, slice index end just the datetime
/// TODO: instead of `LineIndex, LineIndex`, use `(RangeInclusive, Offset)` for the two pairs of LineIndex ranges
///       processing functions would attempt all values within `RangeInclusive` (plus the `Offset`).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct DateTime_Parse_Data {
    pub(crate) pattern: DateTimePattern,
    /// does the `pattern` have a year? ("%Y", "%y")
    pub(crate) year: bool,
    /// does the `pattern` have a timezone? ("%z", "%Z", etc.)
    pub(crate) tz: bool,
    /// slice index begin of entire pattern
    pub(crate) sib: LineIndex,
    /// slice index end of entire pattern
    pub(crate) sie: LineIndex,
    /// slice index begin of only datetime portion of pattern
    pub(crate) siba: LineIndex,
    /// slice index end of only datetime portion of pattern
    pub(crate) siea: LineIndex,
}

//type DateTime_Parse_Data = (DateTimePattern, bool, LineIndex, LineIndex, LineIndex, LineIndex);
/// Datetime Pattern, has year?, has timezone?, lineindex begin entire pattern, lineindex end, lineindex begin datetime portion, lineindex end
pub(crate) type DateTime_Parse_Data_str<'a> = (&'a DateTimePattern_str, bool, bool, LineIndex, LineIndex, LineIndex, LineIndex);
//type DateTime_Parse_Datas_ar<'a> = [DateTime_Parse_Data<'a>];
pub type DateTime_Parse_Datas_vec = Vec<DateTime_Parse_Data>;

/// describe the result of comparing one DateTime to one DateTime Filter
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime1 {
    Pass,
    OccursAtOrAfter,
    OccursBefore,
}

impl Result_Filter_DateTime1 {
    /// Returns `true` if the result is [`OccursAfter`].
    #[inline(always)]
    pub const fn is_after(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursAtOrAfter)
    }

    /// Returns `true` if the result is [`OccursBefore`].
    #[inline(always)]
    pub const fn is_before(&self) -> bool {
        matches!(*self, Result_Filter_DateTime1::OccursBefore)
    }
}

/// describe the result of comparing one DateTime to two DateTime Filters
/// `(after, before)`
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Result_Filter_DateTime2 {
    /// PASS
    InRange,
    /// FAIL
    BeforeRange,
    /// FAIL
    AfterRange,
}

impl Result_Filter_DateTime2 {
    #[inline(always)]
    pub const fn is_pass(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::InRange)
    }

    #[inline(always)]
    pub const fn is_fail(&self) -> bool {
        matches!(*self, Result_Filter_DateTime2::AfterRange | Result_Filter_DateTime2::BeforeRange)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// built-in Datetime formats
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const DATETIME_PARSE_DATAS_LEN: usize = 105;

/// built-in datetime parsing patterns, these are all known patterns attempted on processed files
/// first string is a chrono strftime pattern
/// https://docs.rs/chrono/latest/chrono/format/strftime/
/// first two numbers are total string slice offsets
/// last two numbers are string slice offsets constrained to *only* the datetime portion
/// offset values are [X, Y) (beginning offset is inclusive, ending offset is exclusive or "one past")
/// i.e. string `"[2000-01-01 00:00:00]"`, the pattern may begin at `"["`, the datetime begins at `"2"`
///      same rule for the endings.
/// TODO: use std::ops::RangeInclusive
pub(crate) const DATETIME_PARSE_DATAS: [DateTime_Parse_Data_str; DATETIME_PARSE_DATAS_LEN] = [
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/samba/log.10.7.190.134` (multi-line)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2020/03/05 12:17:59.631000,  3] ../source3/smbd/oplock.c:1340(init_oplocks)
    //        init_oplocks: initializing messages.
    //
    ("[%Y/%m/%d %H:%M:%S%.6f,", true, false, 0, 28, 1, 27),
    //
    // similar:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2000/01/01 00:00:04.123456] foo
    //
    ("[%Y/%m/%d %H:%M:%S%.6f]", true, false, 0, 28, 1, 27),
    //
    // ---------------------------------------------------------------------------------------------
    // prescripted datetime+tz
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 -0400 foo
    //     2000-01-01 00:00:05-0400 foo
    //
    ("%Y-%m-%d %H:%M:%S %z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%d %H:%M:%S%z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%dT%H:%M:%S %z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%dT%H:%M:%S%z ", true, true, 0, 25, 0, 24),
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 ACST foo
    //     2000-01-01 00:00:05ACST foo
    //
    ("%Y-%m-%d %H:%M:%S %Z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%d %H:%M:%S%Z ", true, true, 0, 24, 0, 23),
    ("%Y-%m-%dT%H:%M:%S %Z ", true, true, 0, 25, 0, 24),
    ("%Y-%m-%dT%H:%M:%S%Z ", true, true, 0, 24, 0, 23),
    //
    //               1         2
    //     012345678901234567890123456789
    //     2000-01-01 00:00:05 -04:00 foo
    //     2000-01-01 00:00:05-04:00 foo
    //
    ("%Y-%m-%d %H:%M:%S %:z ", true, true, 0, 27, 0, 26),
    ("%Y-%m-%d %H:%M:%S%:z ", true, true, 0, 26, 0, 25),
    ("%Y-%m-%dT%H:%M:%S %:z ", true, true, 0, 27, 0, 26),
    ("%Y-%m-%dT%H:%M:%S%:z ", true, true, 0, 26, 0, 25),
    //
    //               1         2         3
    //     0123456789012345678901234567890
    //     2000-01-01 00:00:01.234-0500 foo
    //     2000-01-01 00:00:01.234-05:00 foo
    //     2000-01-01 00:00:01.234 ACST foo
    //     2000-00-01T00:00:05.123-00:00 Five
    //
    ("%Y-%m-%d %H:%M:%S%.3f%z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%d %H:%M:%S%.3f%:z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%d %H:%M:%S%.3f %z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%d %H:%M:%S%.3f %:z ", true, true, 0, 31, 0, 30),
    ("%Y-%m-%d %H:%M:%S%.3f %Z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%dT%H:%M:%S%.3f%z ", true, true, 0, 29, 0, 28),
    ("%Y-%m-%dT%H:%M:%S%.3f%:z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f %z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f %:z ", true, true, 0, 31, 0, 30),
    ("%Y-%m-%dT%H:%M:%S%.3f %Z ", true, true, 0, 29, 0, 28),
    //
    //               1         2         3
    //     0123456789012345678901234567890123456789
    //     2000-01-01 00:00:01.234567-0800 foo
    //     2000-01-01 00:00:01.234567-08:00 foo
    //     2000-01-01 00:00:01.234567 ACST foo
    //
    ("%Y-%m-%d %H:%M:%S%.6f%z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%d %H:%M:%S%.6f %z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%d %H:%M:%S%.6f%:z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%d %H:%M:%S%.6f %:z ", true, true, 0, 34, 0, 33),
    ("%Y-%m-%d %H:%M:%S%.6f %Z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%dT%H:%M:%S%.6f%z ", true, true, 0, 32, 0, 31),
    ("%Y-%m-%dT%H:%M:%S%.6f %z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%dT%H:%M:%S%.6f%:z ", true, true, 0, 33, 0, 32),
    ("%Y-%m-%dT%H:%M:%S%.6f %:z ", true, true, 0, 34, 0, 33),
    ("%Y-%m-%dT%H:%M:%S%.6f %Z ", true, true, 0, 32, 0, 31),
    //
    //               1         2         3
    //     0123456789012345678901234567890123456789
    //     20000101T000001 -0800 foo
    //     20000101T000001 -08:00 foo
    //     20000101T000001 ACST foo
    //
    ("%Y%m%dT%H%M%S %z ", true, true, 0, 22, 0, 21),
    ("%Y%m%dT%H%M%S %:z ", true, true, 0, 23, 0, 22),
    ("%Y%m%dT%H%M%S %Z ", true, true, 0, 22, 0, 21),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware/hostd-62.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     2019-07-26T10:40:29.682-07:00 info hostd[03210] [Originator@6876 sub=Default] Current working directory: /usr/bin
    //
    ("%Y-%m-%dT%H:%M:%S%.3f%z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f%Z ", true, true, 0, 30, 0, 29),
    ("%Y-%m-%dT%H:%M:%S%.3f-", true, false, 0, 24, 0, 23),  // XXX: temporary stand-in
    ("%Y-%m-%d %H:%M:%S%.6f-", true, false, 0, 27, 0, 26),  // XXX: temporary stand-in
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/kernel.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     Mar  9 08:10:29 hostname1 kernel: [1013319.252568] device vethb356a02 entered promiscuous mode
    //
    // TODO: [2021/10/03] no support of inferring the year
    //("%b %e %H:%M:%S ", 0, 25, 0, 25),
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/synology/synobackup.log` (has horizontal alignment tabs)
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     info	2017/02/21 21:50:48	SYSTEM:	[Local][Backup Task LocalBackup1] Backup task started.
    //     err	2017/02/23 02:55:58	SYSTEM:	[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    // example escaped:
    //     info␉2017/02/21 21:50:48␉SYSTEM:␉[Local][Backup Task LocalBackup1] Backup task started.
    //     err␉2017/02/23 02:55:58␉SYSTEM:␉[Local][Backup Task LocalBackup1] Exception occured while backing up data. (Capacity at destination is insufficient.) [Path: /volume1/LocalBackup1.hbk]
    //
    // TODO: [2021/10/03] no support of variable offset datetime
    //       this could be done by trying range of offsets into something
    //       better is to search for a preceding regexp pattern
    //("\t%Y/%m/%d %H:%M:%S\t", 5, 24, 0, 24),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // iptables warning from kernel, from file `/var/log/messages` on OpenWRT
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     Mar 30 13:47:07 server1 kern.warn kernel: [57377.167342] DROP IN=eth0 OUT= MAC=ff:ff:ff:ff:ff:ff:01:cc:d0:a8:c8:32:08:00 SRC=68.161.226.20 DST=255.255.255.255 LEN=139 TOS=0x00 PREC=0x20 TTL=64 ID=0 DF PROTO=UDP SPT=33488 DPT=10002 LEN=119
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/xrdp.log`
    // example with offset:
    //
    //               1
    //     01234567890123456789
    //     [20200113-11:03:06] [DEBUG] Closed socket 7 (AF_INET6 :: port 3389)
    //
    ("[%Y%m%d-%H:%M:%S]", true, false, 0, 19, 1, 18),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/alternatives.log`
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890123456789
    //     update-alternatives 2020-02-03 13:56:07: run with --install /usr/bin/jjs jjs /usr/lib/jvm/java-11-openjdk-amd64/bin/jjs 1111
    //
    (" %Y-%m-%d %H:%M:%S: ", true, false, 19, 41, 20, 39),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/Ubuntu18/vmware-installer.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890123456789
    //     [2019-05-06 11:24:34,074] Successfully loaded GTK libraries.
    //
    ("[%Y-%m-%d %H:%M:%S,%3f] ", true, false, 0, 26, 1, 24),
    // repeat prior but no trailing space
    ("[%Y-%m-%d %H:%M:%S,%3f]", true, false, 0, 25, 1, 24),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/other/archives/proftpd/xferlog`
    // example with offset:
    //
    //               1         2
    //     0123456789012345678901234
    //     Sat Oct 03 11:26:12 2020 0 192.168.1.12 0 /var/log/proftpd/xferlog b _ o r root ftp 0 * c
    //
    ("%a %b %d %H:%M:%S %Y ", true, false, 0, 25, 0, 24),
    // repeat prior but no trailing space
    ("%a %b %d %H:%M:%S %Y", true, false, 0, 24, 0, 24),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/OpenSUSE15/zypper.log`
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2019-05-23 16:53:43 <1> trenker(24689) [zypper] main.cc(main):74 ===== Hi, me zypper 1.14.27
    //
    //("%Y-%m-%d %H:%M:%S ", 0, 20, 0, 19),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     2020-01-01 00:00:01.001 xyz
    //      2020-01-01 00:00:01.001 xyz
    //       2020-01-01 00:00:01.001 xyz
    //        2020-01-01 00:00:01.001 xyz
    //         2020-01-01 00:00:01.001 xyz
    //          2020-01-01 00:00:01.001 xyz
    //           2020-01-01 00:00:01.001 xyz
    //            2020-01-01 00:00:01.001 xyz
    //             2020-01-01 00:00:01.001 xyz
    //              2020-01-01 00:00:01.001 xyz
    //     2020-01-01 00:00:01 xyz
    //      2020-01-01 00:00:01 xyz
    //       2020-01-01 00:00:01 xyz
    //        2020-01-01 00:00:01 xyz
    //         2020-01-01 00:00:01 xyz
    //          2020-01-01 00:00:01 xyz
    //           2020-01-01 00:00:01 xyz
    //            2020-01-01 00:00:01 xyz
    //             2020-01-01 00:00:01 xyz
    //              2020-01-01 00:00:01 xyz
    //     2020-01-01 00:00:01xyz
    //      2020-01-01 00:00:01xyz
    //       2020-01-01 00:00:01xyz
    //        2020-01-01 00:00:01xyz
    //         2020-01-01 00:00:01xyz
    //          2020-01-01 00:00:01xyz
    //           2020-01-01 00:00:01xyz
    //            2020-01-01 00:00:01xyz
    //             2020-01-01 00:00:01xyz
    //              2020-01-01 00:00:01xyz
    //
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 0, 24, 0, 23),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 1, 25, 1, 24),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 2, 26, 2, 25),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 3, 27, 3, 26),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 4, 28, 4, 27),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 5, 29, 5, 28),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 6, 30, 6, 29),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 7, 31, 7, 30),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 8, 32, 8, 31),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 9, 33, 9, 32),
    ("%Y-%m-%d %H:%M:%S%.3f ", true, false, 10, 34, 10, 33),
    ("%Y-%m-%d %H:%M:%S ", true, false, 0, 20, 0, 19),
    ("%Y-%m-%d %H:%M:%S ", true, false, 1, 21, 1, 20),
    ("%Y-%m-%d %H:%M:%S ", true, false, 2, 22, 2, 21),
    ("%Y-%m-%d %H:%M:%S ", true, false, 3, 23, 3, 22),
    ("%Y-%m-%d %H:%M:%S ", true, false, 4, 24, 4, 23),
    ("%Y-%m-%d %H:%M:%S ", true, false, 5, 25, 5, 24),
    ("%Y-%m-%d %H:%M:%S ", true, false, 6, 26, 6, 25),
    ("%Y-%m-%d %H:%M:%S ", true, false, 7, 27, 7, 26),
    ("%Y-%m-%d %H:%M:%S ", true, false, 8, 28, 8, 27),
    ("%Y-%m-%d %H:%M:%S ", true, false, 9, 29, 9, 28),
    ("%Y-%m-%d %H:%M:%S ", true, false, 10, 30, 10, 29),
    ("%Y-%m-%d %H:%M:%S", true, false, 0, 19, 0, 19),
    ("%Y-%m-%d %H:%M:%S", true, false, 1, 20, 1, 20),
    ("%Y-%m-%d %H:%M:%S", true, false, 2, 21, 2, 21),
    ("%Y-%m-%d %H:%M:%S", true, false, 3, 22, 3, 22),
    ("%Y-%m-%d %H:%M:%S", true, false, 4, 23, 4, 23),
    ("%Y-%m-%d %H:%M:%S", true, false, 5, 24, 5, 24),
    ("%Y-%m-%d %H:%M:%S", true, false, 6, 25, 6, 25),
    ("%Y-%m-%d %H:%M:%S", true, false, 7, 26, 7, 26),
    ("%Y-%m-%d %H:%M:%S", true, false, 8, 27, 8, 27),
    ("%Y-%m-%d %H:%M:%S", true, false, 9, 28, 9, 28),
    ("%Y-%m-%d %H:%M:%S", true, false, 10, 29, 10, 29),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01 xyz
    //
    ("%Y-%m-%dT%H:%M:%S ", true, false, 0, 20, 0, 19),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2
    //     012345678901234567890
    //     2020-01-01T00:00:01xyz
    //
    ("%Y-%m-%dT%H:%M:%S", true, false, 0, 19, 0, 19),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101 000001 xyz
    //
    ("%Y%m%d %H%M%S ", true, false, 0, 16, 0, 15),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001 xyz
    //
    ("%Y%m%dT%H%M%S ", true, false, 0, 16, 0, 15),
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1
    //     012345678901234567
    //     20200101T000001xyz
    //
    ("%Y%m%dT%H%M%S", true, false, 0, 15, 0, 15),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // from file `./logs/debian9/apport.log.1`
    // example with offset:
    //
    //               1         2         3         4         5
    //     012345678901234567890123456789012345678901234567890
    //     ERROR: apport (pid 9) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 93) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 935) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //     ERROR: apport (pid 9359) Thu Feb 20 00:59:59 2020: called for pid 8581, signal 24, core limit 0, dump mode 1
    //
    (" %a %b %d %H:%M:%S %Y:", true, false, 22, 47, 22, 46),
    (" %a %b %d %H:%M:%S %Y:", true, false, 23, 48, 23, 47),
    (" %a %b %d %H:%M:%S %Y:", true, false, 24, 49, 24, 48),
    (" %a %b %d %H:%M:%S %Y:", true, false, 25, 50, 25, 49),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Thu Feb 20 00:59:59 2020 info
    //     ERROR: Thu Feb 20 00:59:59 2020 error
    //     DEBUG: Thu Feb 20 00:59:59 2020 debug
    //     VERBOSE: Thu Feb 20 00:59:59 2020 verbose
    //
    (" %a %b %d %H:%M:%S %Y ", true, false, 5, 31, 6, 30),
    (" %a %b %d %H:%M:%S %Y ", true, false, 6, 32, 7, 31),
    (" %a %b %d %H:%M:%S %Y ", true, false, 7, 33, 8, 32),
    (" %a %b %d %H:%M:%S %Y ", true, false, 8, 34, 9, 33),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     INFO: Sat Jan 01 2000 08:00:00 info
    //     WARN: Sat Jan 01 2000 08:00:00 warn
    //     ERROR: Sat Jan 01 2000 08:00:00 error
    //     DEBUG: Sat Jan 01 2000 08:00:00 debug
    //     VERBOSE: Sat Jan 01 2000 08:00:00 verbose
    //
    (" %a %b %d %Y %H:%M:%S ", true, false, 5, 31, 6, 30),
    (" %a %b %d %Y %H:%M:%S ", true, false, 6, 32, 7, 31),
    (" %a %b %d %Y %H:%M:%S ", true, false, 7, 33, 8, 32),
    (" %a %b %d %Y %H:%M:%S ", true, false, 8, 34, 9, 33),
    //
    // ---------------------------------------------------------------------------------------------
    //
    // example with offset:
    //
    //               1         2         3         4
    //     01234567890123456789012345678901234567890
    //     [ERROR] 2000-01-01T00:00:03 foo
    //     [WARN] 2000-01-01T00:00:03 foo
    //     [DEBUG] 2000-01-01T00:00:03 foo
    //     [INFO] 2000-01-01T00:00:03 foo
    //     [VERBOSE] 2000-01-01T00:00:03 foo
    //
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 5, 27, 7, 26),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 6, 28, 8, 27),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 7, 29, 9, 28),
    ("] %Y-%m-%dT%H:%M:%S ", true, false, 8, 30, 10, 29),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 5, 27, 7, 26),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 6, 28, 8, 27),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 7, 29, 9, 28),
    ("] %Y-%m-%d %H:%M:%S ", true, false, 8, 30, 10, 29),
    //
    // ---------------------------------------------------------------------------------------------
    // TODO: [2022/03/24] add timestamp formats seen at https://www.unixtimestamp.com/index.php
];

pub(crate)
fn DateTime_Parse_Data_str_to_DateTime_Parse_Data(dtpds: &DateTime_Parse_Data_str) -> DateTime_Parse_Data {
    DateTime_Parse_Data {
        pattern: dtpds.0.to_string(),
        year: dtpds.1,
        tz: dtpds.2,
        sib: dtpds.3,
        sie: dtpds.4,
        siba: dtpds.5,
        siea: dtpds.6,
    }
}

lazy_static! {
    pub(crate) static ref DATETIME_PARSE_DATAS_VEC: DateTime_Parse_Datas_vec =
        DATETIME_PARSE_DATAS.iter().map(
            |&x| DateTime_Parse_Data_str_to_DateTime_Parse_Data(&x)
        ).collect();
}

lazy_static! {
    static ref DATETIME_PARSE_DATAS_VEC_LONGEST: usize =
        DATETIME_PARSE_DATAS.iter().max_by(|x, y| x.0.len().cmp(&y.0.len())).unwrap().0.len();
}

/// does chrono datetime pattern have a timezone?
/// see https://docs.rs/chrono/latest/chrono/format/strftime/
#[inline(always)]
pub fn dt_pattern_has_tz(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Z") ||
    pattern.contains("%z") ||
    pattern.contains("%:z") ||
    pattern.contains("%#z")
}

/// does chrono datetime pattern have a year?
/// see https://docs.rs/chrono/latest/chrono/format/strftime/
#[inline(always)]
#[cfg(any(debug_assertions, test))]
pub fn dt_pattern_has_year(pattern: &DateTimePattern_str) -> bool {
    pattern.contains("%Y") ||
    pattern.contains("%y")
}

/// workaround for chrono Issue #660 https://github.com/chronotope/chrono/issues/660
/// match spaces at beginning and ending of inputs
/// TODO: handle all Unicode whitespace.
///       This fn is essentially counteracting an errant call to `std::string:trim`
///       within `Local.datetime_from_str`.
///       `trim` removes "Unicode Derived Core Property White_Space".
///       This implementation handles three whitespace chars. There are twenty-five whitespace
///       chars according to
///       https://en.wikipedia.org/wiki/Unicode_character_property#Whitespace
pub fn datetime_from_str_workaround_Issue660(value: &str, pattern: &DateTimePattern_str) -> bool {
    let spaces = " ";
    let tabs = "\t";
    let lineends = "\n\r";

    // match whitespace forwards from beginning
    let mut v_sc: u32 = 0;  // `value` spaces count
    let mut v_tc: u32 = 0;  // `value` tabs count
    let mut v_ec: u32 = 0;  // `value` line ends count
    let mut v_brk: bool = false;
    for v_ in value.chars() {
        if spaces.contains(v_) {
            v_sc += 1;
        } else if tabs.contains(v_) {
            v_tc += 1;
        } else if lineends.contains(v_) {
            v_ec += 1;
        } else {
            v_brk = true;
            break;
        }
    }
    let mut p_sc: u32 = 0;  // `pattern` space count
    let mut p_tc: u32 = 0;  // `pattern` tab count
    let mut p_ec: u32 = 0;  // `pattern` line ends count
    let mut p_brk: bool = false;
    for p_ in pattern.chars() {
        if spaces.contains(p_) {
            p_sc += 1;
        } else if tabs.contains(p_) {
            p_tc += 1;
        } else if lineends.contains(p_) {
            p_ec += 1;
        } else {
            p_brk = true;
            break;
        }
    }
    if v_sc != p_sc || v_tc != p_tc || v_ec != p_ec {
        return false;
    }

    // match whitespace backwards from ending
    v_sc = 0;
    v_tc = 0;
    v_ec = 0;
    if v_brk {
        for v_ in value.chars().rev() {
            if spaces.contains(v_) {
                v_sc += 1;
            } else if tabs.contains(v_) {
                v_tc += 1;
            } else if lineends.contains(v_) {
                v_ec += 1;
            } else {
                break;
            }
        }
    }
    p_sc = 0;
    p_tc = 0;
    p_ec = 0;
    if p_brk {
        for p_ in pattern.chars().rev() {
            if spaces.contains(p_) {
                p_sc += 1;
            } else if tabs.contains(p_) {
                p_tc += 1;
            } else if lineends.contains(p_) {
                p_ec += 1;
            } else {
                break;
            }
        }
    }
    if v_sc != p_sc || v_tc != p_tc || v_ec != p_ec {
        return false;
    }

    true
}

/// decoding `[u8]` bytes to a `str` takes a surprising amount of time, according to `tools/flamegraph.sh`.
/// first check `u8` slice with custom simplistic checker that, in case of complications,
/// falls back to using higher-resource and more-precise checker `encoding_rs::mem::utf8_latin1_up_to`.
/// this uses built-in unsafe `str::from_utf8_unchecked`.
/// See `benches/bench_decode_utf.rs` for comparison of bytes->str decode strategies
#[inline(always)]
pub fn u8_to_str(slice_: &[u8]) -> Option<&str> {
    let dts: &str;
    let mut fallback = false;
    // custom check for UTF8; fast but imperfect
    if ! slice_.is_ascii() {
        fallback = true;
    }
    if fallback {
        // found non-ASCII, fallback to checking with `utf8_latin1_up_to` which is a thorough check
        let va = encoding_rs::mem::utf8_latin1_up_to(slice_);
        if va != slice_.len() {
            return None;  // invalid UTF8
        }
    }
    unsafe {
        dts = std::str::from_utf8_unchecked(slice_);
    };
    Some(dts)
}

/// convert any `&str` to a chrono `Option<DateTime<FixedOffset>>` instance
#[inline(always)]
pub fn str_datetime(
    dts: &str,
    pattern: &DateTimePattern_str,
    patt_has_tz: bool,
    tz_offset: &FixedOffset
) -> DateTimeL_Opt {
    debug_eprintln!("{}str_datetime({:?}, {:?}, {:?}, {:?})", sn(), str_to_String_noraw(dts), pattern, patt_has_tz, tz_offset);
    // TODO: 2022/04/07
    //       if dt_pattern has TZ then create a `DateTime`
    //       if dt_pattern does not have TZ then create a `NaiveDateTime`
    //       then convert that to `DateTime` with aid of crate `chrono_tz`
    //       TZ::from_local_datetime();
    //       How to determine TZ to use? Should it just use Local?
    //       Defaulting to local TZ would be an adequate start.
    //       But pass around as `chrono::DateTime`, not `chrono::Local`.
    //       Replace use of `Local` with `DateTime. Change typecast `DateTimeL`
    //       type. Can leave the name in place for now.
    debug_assert_eq!(patt_has_tz, dt_pattern_has_tz(pattern), "wrong {} for pattern {}", patt_has_tz, pattern);
    if patt_has_tz {
        match DateTime::parse_from_str(dts, pattern) {
            Ok(val) => {
                debug_eprintln!(
                    "{}str_datetime: DateTime::parse_from_str({:?}, {:?}) extrapolated DateTime {:?}",
                    so(),
                    str_to_String_noraw(dts),
                    pattern,
                    val,
                );
                // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
                //       and `pattern`
                if !datetime_from_str_workaround_Issue660(dts, pattern) {
                    debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660", sx());
                    return None;
                }
                debug_eprintln!("{}str_datetime return {:?}", sx(), Some(val));
                return Some(val);
            }
            Err(err) => {
                debug_eprintln!("{}str_datetime: DateTime::parse_from_str({:?}, {:?}) failed ParseError {}", sx(), dts, pattern, err);
                return None;
            }
        };
    }

    // no timezone in `pattern` so first convert to a `NaiveDateTime` instance
    let dt_naive = match NaiveDateTime::parse_from_str(dts, pattern) {
        Ok(val) => {
            debug_eprintln!(
                "{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) extrapolated NaiveDateTime {:?}",
                so(),
                str_to_String_noraw(dts),
                pattern,
                val,
            );
            // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
            //       and `dtpd.pattern`
            if !datetime_from_str_workaround_Issue660(dts, pattern) {
                debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660", sx());
                return None;
            }
            val
        }
        Err(err) => {
            debug_eprintln!("{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) failed ParseError {}", sx(), dts, pattern, err);
            return None;
        }
    };
    // second convert the `NaiveDateTime` instance to `DateTime<FixedOffset>` instance
    match tz_offset.from_local_datetime(&dt_naive).earliest() {
        Some(val) => {
            debug_eprintln!(
                "{}str_datetime: tz_offset.from_local_datetime({:?}).earliest() extrapolated NaiveDateTime {:?}",
                so(),
                dt_naive,
                val,
            );
            // HACK: workaround chrono Issue #660 by checking for matching begin, end of `dts`
            //       and `dtpd.pattern`
            if !datetime_from_str_workaround_Issue660(dts, pattern) {
                debug_eprintln!("{}str_datetime: skip match due to chrono Issue #660, return None", sx());
                return None;
            }
            debug_eprintln!("{}str_datetime return {:?}", sx(), Some(val));
            Some(val)
        }
        None => {
            debug_eprintln!("{}str_datetime: NaiveDateTime.parse_from_str({:?}, {:?}) returned None, return None", sx(), dts, pattern);
            None
        }
    }
}

/// if `dt` is at or after `dt_filter` then return `OccursAtOrAfter`
/// if `dt` is before `dt_filter` then return `OccursBefore`
/// else return `Pass` (including if `dt_filter` is `None`)
pub fn dt_after_or_before(dt: &DateTimeL, dt_filter: &DateTimeL_Opt) -> Result_Filter_DateTime1 {
    if dt_filter.is_none() {
        debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::Pass; (no dt filters)", snx(),);
        return Result_Filter_DateTime1::Pass;
    }

    let dt_a = &dt_filter.unwrap();
    debug_eprintln!("{}dt_after_or_before comparing dt datetime {:?} to filter datetime {:?}", sn(), dt, dt_a);
    if dt < dt_a {
        debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursBefore; (dt {:?} is before dt_filter {:?})", sx(), dt, dt_a);
        return Result_Filter_DateTime1::OccursBefore;
    }
    debug_eprintln!("{}dt_after_or_before(…) return Result_Filter_DateTime1::OccursAtOrAfter; (dt {:?} is at or after dt_filter {:?})", sx(), dt, dt_a);

    Result_Filter_DateTime1::OccursAtOrAfter
}

/// If both filters are `Some` and `syslinep.dt` is "between" the filters then return `Pass`
/// comparison is "inclusive" i.e. `dt` == `dt_filter_after` will return `Pass`
/// If both filters are `None` then return `Pass`
/// TODO: finish this docstring
pub fn dt_pass_filters(
    dt: &DateTimeL, dt_filter_after: &DateTimeL_Opt, dt_filter_before: &DateTimeL_Opt,
) -> Result_Filter_DateTime2 {
    debug_eprintln!("{}dt_pass_filters({:?}, {:?}, {:?})", sn(), dt, dt_filter_after, dt_filter_before,);
    if dt_filter_after.is_none() && dt_filter_before.is_none() {
        debug_eprintln!(
            "{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange; (no dt filters)",
            sx(),
        );
        return Result_Filter_DateTime2::InRange;
    }
    if dt_filter_after.is_some() && dt_filter_before.is_some() {
        debug_eprintln!(
            "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt < {:?} dt_fiter_before ???",
            so(),
            &dt_filter_after.unwrap(),
            dt,
            &dt_filter_before.unwrap()
        );
        let da = &dt_filter_after.unwrap();
        let db = &dt_filter_before.unwrap();
        assert_le!(da, db, "Bad datetime range values filter_after {:?} {:?} filter_before", da, db);
        if dt < da {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::BeforeRange;", sx());
            return Result_Filter_DateTime2::BeforeRange;
        }
        if db < dt {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::AfterRange;", sx());
            return Result_Filter_DateTime2::AfterRange;
        }
        // assert da < dt && dt < db
        assert_le!(da, dt, "Unexpected range values da dt");
        assert_le!(dt, db, "Unexpected range values dt db");
        debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange;", sx());
        return Result_Filter_DateTime2::InRange;
    } else if dt_filter_after.is_some() {
        debug_eprintln!(
            "{}dt_pass_filters comparing datetime dt_filter_after {:?} < {:?} dt ???",
            so(),
            &dt_filter_after.unwrap(),
            dt
        );
        let da = &dt_filter_after.unwrap();
        if dt < da {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::BeforeRange;", sx());
            return Result_Filter_DateTime2::BeforeRange;
        }
        debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange;", sx());
        return Result_Filter_DateTime2::InRange;
    } else {
        debug_eprintln!(
            "{}dt_pass_filters comparing datetime dt {:?} < {:?} dt_filter_before ???",
            so(),
            dt,
            &dt_filter_before.unwrap()
        );
        let db = &dt_filter_before.unwrap();
        if db < dt {
            debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::AfterRange;", sx());
            return Result_Filter_DateTime2::AfterRange;
        }
        debug_eprintln!("{}dt_pass_filters(…) return Result_Filter_DateTime2::InRange;", sx());
        return Result_Filter_DateTime2::InRange;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - search a slice quickly (loop unroll version)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// TODO: [202/04/15] put performance tweaks into a mod
// pub mod fast_check {

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_6_2(slice_: &[u8; 6], search: &[u8; 2]) -> bool {
    for i in 0..5 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_7_2(slice_: &[u8; 7], search: &[u8; 2]) -> bool {
    for i in 0..6 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_8_2(slice_: &[u8; 8], search: &[u8; 2]) -> bool {
    for i in 0..7 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_9_2(slice_: &[u8; 9], search: &[u8; 2]) -> bool {
    for i in 0..8 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_10_2(slice_: &[u8; 10], search: &[u8; 2]) -> bool {
    for i in 0..9 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_11_2(slice_: &[u8; 11], search: &[u8; 2]) -> bool {
    for i in 0..10 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_12_2(slice_: &[u8; 12], search: &[u8; 2]) -> bool {
    for i in 0..11 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_13_2(slice_: &[u8; 13], search: &[u8; 2]) -> bool {
    for i in 0..12 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_14_2(slice_: &[u8; 14], search: &[u8; 2]) -> bool {
    for i in 0..13 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_15_2(slice_: &[u8; 15], search: &[u8; 2]) -> bool {
    for i in 0..14 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_16_2(slice_: &[u8; 16], search: &[u8; 2]) -> bool {
    for i in 0..15 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_17_2(slice_: &[u8; 17], search: &[u8; 2]) -> bool {
    for i in 0..16 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_18_2(slice_: &[u8; 18], search: &[u8; 2]) -> bool {
    for i in 0..17 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_19_2(slice_: &[u8; 19], search: &[u8; 2]) -> bool {
    for i in 0..18 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_20_2(slice_: &[u8; 20], search: &[u8; 2]) -> bool {
    for i in 0..19 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_21_2(slice_: &[u8; 21], search: &[u8; 2]) -> bool {
    for i in 0..20 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_22_2(slice_: &[u8; 22], search: &[u8; 2]) -> bool {
    for i in 0..21 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_23_2(slice_: &[u8; 23], search: &[u8; 2]) -> bool {
    for i in 0..22 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_24_2(slice_: &[u8; 24], search: &[u8; 2]) -> bool {
    for i in 0..23 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_25_2(slice_: &[u8; 25], search: &[u8; 2]) -> bool {
    for i in 0..24 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_26_2(slice_: &[u8; 26], search: &[u8; 2]) -> bool {
    for i in 0..25 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_27_2(slice_: &[u8; 27], search: &[u8; 2]) -> bool {
    for i in 0..26 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_28_2(slice_: &[u8; 28], search: &[u8; 2]) -> bool {
    for i in 0..27 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_29_2(slice_: &[u8; 29], search: &[u8; 2]) -> bool {
    for i in 0..28 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_30_2(slice_: &[u8; 30], search: &[u8; 2]) -> bool {
    for i in 0..29 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_31_2(slice_: &[u8; 31], search: &[u8; 2]) -> bool {
    for i in 0..30 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_32_2(slice_: &[u8; 32], search: &[u8; 2]) -> bool {
    for i in 0..31 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_33_2(slice_: &[u8; 33], search: &[u8; 2]) -> bool {
    for i in 0..32 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_34_2(slice_: &[u8; 34], search: &[u8; 2]) -> bool {
    for i in 0..33 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_35_2(slice_: &[u8; 35], search: &[u8; 2]) -> bool {
    for i in 0..34 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_36_2(slice_: &[u8; 36], search: &[u8; 2]) -> bool {
    for i in 0..35 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_37_2(slice_: &[u8; 37], search: &[u8; 2]) -> bool {
    for i in 0..36 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_38_2(slice_: &[u8; 38], search: &[u8; 2]) -> bool {
    for i in 0..37 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_39_2(slice_: &[u8; 39], search: &[u8; 2]) -> bool {
    for i in 0..38 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
fn slice_contains_40_2(slice_: &[u8; 40], search: &[u8; 2]) -> bool {
    for i in 0..39 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

/// loop unrolled implementation of `slice.contains` for a byte slice and a hardcorded array
/// benchmark `benches/bench_slice_contains.rs` demonstrates this is faster
#[inline(always)]
pub fn slice_contains_X_2(slice_: &[u8], search: &[u8; 2]) -> bool {
    match slice_.len() {
        6 => slice_contains_6_2(array_ref!(slice_, 0, 6), search),
        7 => slice_contains_7_2(array_ref!(slice_, 0, 7), search),
        8 => slice_contains_8_2(array_ref!(slice_, 0, 8), search),
        9 => slice_contains_9_2(array_ref!(slice_, 0, 9), search),
        10 => slice_contains_10_2(array_ref!(slice_, 0, 10), search),
        11 => slice_contains_11_2(array_ref!(slice_, 0, 11), search),
        12 => slice_contains_12_2(array_ref!(slice_, 0, 12), search),
        13 => slice_contains_13_2(array_ref!(slice_, 0, 13), search),
        14 => slice_contains_14_2(array_ref!(slice_, 0, 14), search),
        15 => slice_contains_15_2(array_ref!(slice_, 0, 15), search),
        16 => slice_contains_16_2(array_ref!(slice_, 0, 16), search),
        17 => slice_contains_17_2(array_ref!(slice_, 0, 17), search),
        18 => slice_contains_18_2(array_ref!(slice_, 0, 18), search),
        19 => slice_contains_19_2(array_ref!(slice_, 0, 19), search),
        20 => slice_contains_20_2(array_ref!(slice_, 0, 20), search),
        21 => slice_contains_21_2(array_ref!(slice_, 0, 21), search),
        22 => slice_contains_22_2(array_ref!(slice_, 0, 22), search),
        23 => slice_contains_23_2(array_ref!(slice_, 0, 23), search),
        24 => slice_contains_24_2(array_ref!(slice_, 0, 24), search),
        25 => slice_contains_25_2(array_ref!(slice_, 0, 25), search),
        26 => slice_contains_26_2(array_ref!(slice_, 0, 26), search),
        27 => slice_contains_27_2(array_ref!(slice_, 0, 27), search),
        28 => slice_contains_28_2(array_ref!(slice_, 0, 28), search),
        29 => slice_contains_29_2(array_ref!(slice_, 0, 29), search),
        30 => slice_contains_30_2(array_ref!(slice_, 0, 30), search),
        31 => slice_contains_31_2(array_ref!(slice_, 0, 31), search),
        32 => slice_contains_32_2(array_ref!(slice_, 0, 32), search),
        33 => slice_contains_33_2(array_ref!(slice_, 0, 33), search),
        34 => slice_contains_34_2(array_ref!(slice_, 0, 34), search),
        35 => slice_contains_35_2(array_ref!(slice_, 0, 35), search),
        36 => slice_contains_36_2(array_ref!(slice_, 0, 36), search),
        37 => slice_contains_37_2(array_ref!(slice_, 0, 37), search),
        38 => slice_contains_38_2(array_ref!(slice_, 0, 38), search),
        39 => slice_contains_39_2(array_ref!(slice_, 0, 39), search),
        40 => slice_contains_40_2(array_ref!(slice_, 0, 40), search),
        _ => {
            for c in slice_.iter() {
                if c == &search[0] || c == &search[1] {
                    return true;
                }
            }
            false
        }
    }
}
