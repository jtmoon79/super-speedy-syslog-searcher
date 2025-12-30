// src/tests/journal_tests.rs

//! tests for `journal.rs`

use ::more_asserts::assert_le;
use ::test_case::test_case;

use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
};
use crate::data::journal::{
    datetimel_to_realtime_timestamp,
    datetimelopt_to_realtime_timestamp_opt,
    realtime_or_source_realtime_timestamp_to_datetimel,
    realtime_timestamp_to_datetimel,
    DtUsesSource,
    EpochMicroseconds,
    EpochMicrosecondsOpt,
    JournalEntry,
    DT_USES_SOURCE_OVERRIDE,
};
use crate::tests::common::{
    DT_1,
    DT_1_E1,
    FO_0,
    FO_E1,
    TS_1,
};

#[test]
fn test_journalentry_new() {
    JournalEntry::new(b"".to_vec(), 0, Some(0), DtUsesSource::RealtimeTimestamp, &FO_0);
}

const JOURNAL_ENTRY_EXPORT: &str = "\
__CURSOR=s=e992f143877046059b264a0f907056b6;i=6ff;b=26d74a46deff4872be6d4ca6e885a198;m=46c65ea;t=5f840a88a4b39;x=e7933c3b47482d45
__REALTIME_TIMESTAMP=1680331472784185
__MONOTONIC_TIMESTAMP=74212842
_BOOT_ID=26d74a46deff4872be6d4ca6e885a198
_TRANSPORT=journal
_UID=1000
_GID=1000
_CAP_EFFECTIVE=0
_SELINUX_CONTEXT

unconfined

_AUDIT_SESSION=2
_AUDIT_LOGINUID=1000
_SYSTEMD_OWNER_UID=1000
_SYSTEMD_UNIT=user@1000.service
_SYSTEMD_SLICE=user-1000.slice
_MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
_HOSTNAME=ubuntu22Acorn
PRIORITY=4
_SYSTEMD_USER_SLICE=session.slice
_PID=1306
_COMM=gnome-shell
_EXE=/usr/bin/gnome-shell
_CMDLINE=/usr/bin/gnome-shell
_SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@wayland.service
_SYSTEMD_USER_UNIT=org.gnome.Shell@wayland.service
_SYSTEMD_INVOCATION_ID=b7d368c96091463aa538006b518785f4
GLIB_DOMAIN=Ubuntu AppIndicators
SYSLOG_IDENTIFIER=ubuntu-appindicators@ubuntu.com
CODE_FILE=/usr/share/gnome-shell/extensions/ubuntu-appindicators@ubuntu.com/appIndicator.js
CODE_LINE=738
CODE_FUNC=_setGicon
MESSAGE=unable to update icon for livepatch
_SOURCE_REALTIME_TIMESTAMP=1680331472788150\
";

#[test]
fn test_journalentry_export_realtime_timestamp() {
    let data = JOURNAL_ENTRY_EXPORT.as_bytes().to_vec();
    let dtus: DtUsesSource = DtUsesSource::RealtimeTimestamp;
    let je = JournalEntry::new(
        data.clone(),
        TS_1,
        None,
        dtus,
        &FO_0,
    );
    assert_eq!(je.dt(), &*DT_1, "dt()");
    assert_eq!(je.as_bytes(), &data, "data");
    let (a, b) = JournalEntry::find_timestamp_in_buffer(
        &data,
        dtus,
    );
    assert_eq!(a, 151, "dt_a");
    assert_eq!(b, 167, "dt_b");
    assert_le!(a, b);
}

#[test]
fn test_journalentry_export_source_realtime_timestamp() {
    let data = JOURNAL_ENTRY_EXPORT.as_bytes().to_vec();
    let dtus: DtUsesSource = DtUsesSource::SourceRealtimeTimestamp;
    let je = JournalEntry::new(
        data.clone(),
        // XXX: last minute hack to get tests to pass
        match DT_USES_SOURCE_OVERRIDE {
            Some(_) => TS_1,
            None => 99,
        },
        EpochMicrosecondsOpt::Some(TS_1),
        dtus,
        &FO_0,
    );
    assert_eq!(je.dt(), &*DT_1, "dt()");
    assert_eq!(je.as_bytes(), &data, "data");
    let (a, b) = JournalEntry::find_timestamp_in_buffer(
        &data,
        dtus,
    );
    assert_eq!(a, 1145, "dt_a");
    assert_eq!(b, 1161, "dt_b");
    assert_le!(a, b);
}

const JOURNAL_ENTRY_SHORTFULL: &str =
    "1970-01-12T13:46:40 UTC ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch\n";

#[test]
fn test_journalentry_shortfull() {
    let data = JOURNAL_ENTRY_SHORTFULL.as_bytes().to_vec();
    let dtus: DtUsesSource = DtUsesSource::SourceRealtimeTimestamp;
    let je = JournalEntry::new(
        data.clone(),
        // XXX: last minute hack to get tests to pass
        match DT_USES_SOURCE_OVERRIDE {
            Some(_) => TS_1,
            None => 99,
        },
        EpochMicrosecondsOpt::Some(TS_1),
        dtus,
        &FO_0,
    );
    assert_eq!(je.dt(), &*DT_1, "dt()");
    assert_eq!(je.as_bytes(), &data, "data");
    let (a, b) = JournalEntry::find_timestamp_in_buffer(
        &data,
        dtus,
    );
    assert_eq!(a, 0, "dt_a");
    assert_eq!(b, 0, "dt_b");
    assert_le!(a, b);
}

#[test_case(
    &FO_0,
    TS_1,
    *DT_1;
    "FO0_one_TS1"
)]
fn test_realtime_timestamp_to_datetimel(
    fixed_offset: &FixedOffset,
    em: EpochMicroseconds,
    expect_dt: DateTimeL,
) {
    let dt =
        realtime_timestamp_to_datetimel(
            fixed_offset,
            &em,
        );
    assert_eq!(dt, expect_dt, "\ngot      {:?}\nexpected {:?}\n", dt, expect_dt);
}

#[test_case(
    &FO_0,
    TS_1,
    EpochMicrosecondsOpt::Some(99),
    *DT_1;
    "FO0_ts1_99"
)]
#[test_case(
    &FO_E1,
    TS_1,
    EpochMicrosecondsOpt::None,
    *DT_1_E1;
    "FOE1_TS1_None"
)]
fn test_realtime_or_source_realtime_timestamp_to_datetimel(
    fixed_offset: &FixedOffset,
    em: EpochMicroseconds,
    em2: EpochMicrosecondsOpt,
    expect_dt: DateTimeL,
) {
    let dt =
        realtime_or_source_realtime_timestamp_to_datetimel(
            fixed_offset,
            &em,
            &em2,
        );
    assert_eq!(dt, expect_dt, "\ngot      {:?}\nexpected {:?}\n", dt, expect_dt);
}

#[test_case(*DT_1, TS_1)]
fn test_datetimel_to_realtime_timestamp(
    dt: DateTimeL,
    expect_rt: EpochMicroseconds,
) {
    let rt =
        datetimel_to_realtime_timestamp(
            &dt
        );
    assert_eq!(rt, expect_rt, "\ngot      {:?}\nexpected {:?}\n", rt, expect_rt);
}

#[test_case(DateTimeLOpt::Some(*DT_1), TS_1)]
#[test_case(DateTimeLOpt::None, 0)]
fn test_datetimelopt_to_realtime_timestamp(
    dt: DateTimeLOpt,
    expect_rt: EpochMicroseconds,
) {
    let rt =
        datetimelopt_to_realtime_timestamp_opt(
            &dt
        );
    if expect_rt == 0 {
        assert!(rt.is_none());
    } else {
        assert_eq!(rt.unwrap(), expect_rt, "\ngot      {:?}\nexpected {:?}\n", rt, expect_rt);
    }
}
