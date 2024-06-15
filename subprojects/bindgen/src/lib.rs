// lib.rs
//
// Follows from https://rust-lang.github.io/rust-bindgen/tutorial-4.html

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!("./bindings.rs");

/// ripped from
/// https://github.com/systemd/systemd/blob/16a9ad557de7173c182e9587a9cc0ca146293ec8/src/libsystemd/sd-journal/journal-internal.h#L135-L142
fn JOURNAL_ERRNO_IS_UNAVAILABLE_FIELD(r: i32) -> bool {
    let e = nix::errno::Errno::from_i32(r.abs());
    match e {
        nix::errno::Errno::ENOBUFS            /* Field or decompressed field too large */
        | nix::errno::Errno::E2BIG            /* Field too large for pointer width */
        | nix::errno::Errno::EPROTONOSUPPORT  /* Unsupported compression */
        => true,
        _ => false,
    }
}

const KEY_SOURCE_REALTIME_TIMESTAMP: &str = "_SOURCE_REALTIME_TIMESTAMP";

#[cfg(test)]
mod tests {
    use super::*;
    use std::{mem, ffi::OsStr, os::unix::prelude::OsStrExt, collections::BTreeMap};

    #[test]
    fn test_journal() {
        unsafe {
            let mut j: sd_journal = sd_journal {
                _unused: [0; 0],
            };
            let mut pj: *mut sd_journal = &mut j;

            const PATH1S: &str = "../logs/programs/journal/user-1000.journal\0";
            let mut path1Os: &OsStr = &OsStr::new(PATH1S);
            let mut path1pc: *const ::std::os::raw::c_char = path1Os.as_bytes().as_ptr() as *const ::std::os::raw::c_char;

            let mut ppaths: [*const ::std::os::raw::c_char; 2] = [path1pc, ::std::ptr::null()];
            let mut pppaths: *mut *const ::std::os::raw::c_char = ppaths.as_mut_ptr();

            /*
                int sd_journal_open_files(sd_journal **ret, const char **paths, int flags)
            */
            let fp = sd_journal_open_files as unsafe extern "C" fn(_, _, _) -> _;
            eprintln!("• sd_journal_open_files @{:p}", fp);
            eprintln!("▶ sd_journal_open_files({:?}, {:?}, 0)", j, path1Os);
            let r: i32 = sd_journal_open_files(&mut pj, pppaths, 0);
            let e = nix::errno::Errno::from_i32(r);
            eprintln!("• sd_journal_open_files returned {}, {:?}", r, e);
            if r < 0 {
                return;
            }

            let mut entry_times: Vec<String> = Vec::new();

            eprintln!("▶ sd_journal_seek_head(@{:p})", pj);
            let r: i32 = sd_journal_seek_head(pj);
            let e = nix::errno::Errno::from_i32(r);
            eprintln!("• sd_journal_seek_head returned {}, {:?}", r, e);
            if r < 0 {
                return;
            }

            /*
            // Timestamp in milliseconds: 1680419520000
            // Date and time (GMT): Sunday, April 2, 2023 7:12:00 
            //
            // Timestamp in milliseconds: 1680422700000
            // Date and time (GMT): Sunday, April 2, 2023 8:05:00
            //
            // pass that time in microseconds
            let t: u64 = 1680419520000000;
            let t: u64 = 1680422700000000;
            eprintln!("▶ sd_journal_seek_realtime_usec(@{:p},  {})", pj, t);
            let r: i32 = sd_journal_seek_realtime_usec(pj, t);
            let e = nix::errno::Errno::from_i32(r);
            eprintln!("• sd_journal_seek_realtime_usec returned {}, {:?}", r, e);
            if r < 0 {
                return;
            }
            */


            let mut emerg_stop_next= 0;
            while emerg_stop_next < 1000 {
                /*
                    int sd_journal_next(sd_journal *j);
                */
                eprintln!("▶ sd_journal_next(…) {}", emerg_stop_next);
                let r: i32 = sd_journal_next(pj);
                let e = nix::errno::Errno::from_i32(r);
                eprintln!("• sd_journal_next returned {}, {:?}", r, e);
                if r <= 0 {
                    break;
                }
                emerg_stop_next += 1;

                /*
                    int sd_journal_restart_data(sd_journal *j);
                */
                // XXX: this is called by python-systemd
                //      but it doens't appear to have an effect here.
                //      Leaving it in anyway.
                //      https://github.com/systemd/python-systemd/blob/802c8dcaa3096719be0a1c121e747b2681ad31dc/systemd/_reader.c#L615
                //      https://github.com/systemd/systemd/blob/v249/src/systemd/sd-journal.h#L162-L163
                //eprintln!("▶ sd_journal_restart_data(...)");
                //sd_journal_restart_data(pj);

                let mut all_data_enumerate: Vec<String> = Vec::new();

                let mut emerg_stop_data_enumerate = 0;
                while emerg_stop_data_enumerate < 200 {
                    /*
                        int sd_journal_enumerate_data(sd_journal *j, const void **data, size_t *l);
                    */
                    eprintln!("▶ sd_journal_enumerate_available_data(…) {}", emerg_stop_data_enumerate);
                    let mut pdata: *const std::os::raw::c_void = mem::zeroed();
                    let mut ppdata: *mut *const std::os::raw::c_void = &mut pdata;
                    let mut i: std::os::raw::c_ulong = 0;
                    let mut pi: *mut std::os::raw::c_ulong = &mut i;    
                    let r = sd_journal_enumerate_available_data(pj, ppdata, pi);
                    let e = nix::errno::Errno::from_i32(r);
                    eprintln!("• sd_journal_enumerate_available_data returned {}, {:?}", r, e);
                    //if JOURNAL_ERRNO_IS_UNAVAILABLE_FIELD(r) {
                    //    eprintln!("• sd_journal_enumerate_available_data returned JOURNAL_ERRNO_IS_UNAVAILABLE_FIELD");
                    //    continue;
                    //}
                    if r <= 0 {
                        break;
                    }
                    emerg_stop_data_enumerate += 1;
                    // i is number of bytes
                    eprintln!("• sd_journal_enumerate_available_data returned i {}", i);
                    eprintln!("• sd_journal_enumerate_available_data returned pdata {:?}", pdata);
                    let data: &[u8] = std::slice::from_raw_parts(pdata as *const u8, i as usize);
                    let datas: &str = std::str::from_utf8(data).unwrap_or_default();
                    eprintln!("• sd_journal_enumerate_available_data returned data {:?}", datas);
                    all_data_enumerate.push(datas.to_string());
                    let mut field_x: &str = datas;
                    let mut value_x: &str = "";
                    if datas.contains('=') {
                        let mut parts: Vec<&str> = datas.splitn(2, '=').collect();
                        field_x = parts.remove(0);
                        value_x = parts.remove(0);
                    }
                    if field_x == KEY_SOURCE_REALTIME_TIMESTAMP {
                        entry_times.push(value_x.to_string());
                    }
                    eprintln!();
                }

                let mut rt: std::os::raw::c_ulonglong = 0;
                let prt: *mut std::os::raw::c_ulonglong = &mut rt;
                eprintln!("▶ sd_journal_get_realtime_usec(…)");
                let r = sd_journal_get_realtime_usec(pj, prt);
                let e = nix::errno::Errno::from_i32(r);
                eprintln!("• sd_journal_get_realtime_usec returned {}, {:?}", r, e);
                if r < 0 {
                    break;
                }
                eprintln!("• sd_journal_get_realtime_usec realtime {}", rt);
                // TODO: entry['__REALTIME_TIMESTAMP'] = self._get_realtime()

                let mut rt: std::os::raw::c_ulonglong = 0;
                let prt: *mut std::os::raw::c_ulonglong = &mut rt;
                let mut id128: sd_id128 = sd_id128 { bytes: [0; 16] };
                let pid128: *mut sd_id128 = &mut id128 as *mut _ as *mut sd_id128;
                eprintln!("▶ sd_journal_get_monotonic_usec(…)");
                let r = sd_journal_get_monotonic_usec(pj, prt, pid128);
                let e = nix::errno::Errno::from_i32(r);
                eprintln!("• sd_journal_get_monotonic_usec returned {}, {:?}", r, e);
                if r < 0 {
                    break;
                }
                eprintln!("• sd_journal_get_monotonic_usec realtime {}", rt);
                // TODO: entry['__MONOTONIC_TIMESTAMP'] = self._get_monotonic()

                let mut cursor: *mut std::os::raw::c_char = mem::zeroed();
                let mut pcursor: *mut *mut std::os::raw::c_char = &mut cursor;
                eprintln!("▶ sd_journal_get_cursor(…)");
                let r = sd_journal_get_cursor(pj, pcursor);
                let e = nix::errno::Errno::from_i32(r);
                eprintln!("• sd_journal_get_cursor returned {}, {:?}", r, e);
                if r < 0 {
                    break;
                }
                // TODO: entry['__CURSOR'] = self._get_cursor()

                /*
                ▶ PAGER= journalctl --lines=1 --all --utc --file=/tmp/libloader1/user-1000.journal
                Apr 02 08:05:50 ubuntu22Acorn pkexec[11738]: pam_unix(polkit-1:session): session opened for user root(uid=0) by (uid=1000)

                ▶ PAGER= journalctl --lines=1 --output=short-full --all --utc --file=/tmp/libloader1/user-1000.journal
                Sun 2023-04-02 08:05:50 UTC ubuntu22Acorn pkexec[11738]: pam_unix(polkit-1:session): session opened for user root(uid=0) by (uid=1000)

                ▶ PAGER= journalctl --lines=1 --output=verbose --all --utc --file=/tmp/libloader1/user-1000.journal
                Sun 2023-04-02 07:08:49.254986 UTC [s=e992f143877046059b264a0f907056b6;i=83f;b=26d74a46deff4872be6d4ca6e885a198;m=1479d979e7;t=5f8551d31a64a;x=76c5956b40e4fbcc]
                    _TRANSPORT=syslog
                    PRIORITY=6
                    SYSLOG_FACILITY=10
                    SYSLOG_IDENTIFIER=pkexec
                    SYSLOG_TIMESTAMP=Apr  2 00:08:49
                    MESSAGE=pam_unix(polkit-1:session): session opened for user root(uid=0) by (uid=1000)
                    _PID=5747
                    _UID=1000
                    _GID=1000
                    _COMM=pkexec
                    _EXE=/usr/bin/pkexec
                    _CMDLINE=pkexec /usr/lib/update-notifier/package-system-locked
                    _CAP_EFFECTIVE=1ffffffffff
                    _SELINUX_CONTEXT=unconfined
                    _AUDIT_SESSION=2
                    _AUDIT_LOGINUID=1000
                    _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-update\x2dnotifier-2811.scope
                    _SYSTEMD_OWNER_UID=1000
                    _SYSTEMD_UNIT=user@1000.service
                    _SYSTEMD_USER_UNIT=app-gnome-update\x2dnotifier-2811.scope
                    _SYSTEMD_SLICE=user-1000.slice
                    _SYSTEMD_USER_SLICE=app.slice
                    _SYSTEMD_INVOCATION_ID=6fbf34863d7445e18cdf0794fd12dd60
                    _SOURCE_REALTIME_TIMESTAMP=1680419329254986
                    _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
                    _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
                    _HOSTNAME=ubuntu22Acorn

                ▶ PAGER= journalctl --lines=1 --output=export --all --utc --file=/tmp/libloader1/user-1000.journal
                __CURSOR=s=e992f143877046059b264a0f907056b6;i=9de;b=26d74a46deff4872be6d4ca6e885a198;m=1545c30bd6;t=5f855e91b3838;x=8b1709436238c122
                __REALTIME_TIMESTAMP=1680422750337080
                __MONOTONIC_TIMESTAMP=91364723670
                _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
                _TRANSPORT=syslog
                PRIORITY=6
                SYSLOG_FACILITY=10
                SYSLOG_IDENTIFIER=pkexec
                MESSAGE=pam_unix(polkit-1:session): session opened for user root(uid=0) by (uid=1000)
                _UID=1000
                _GID=1000
                _COMM=pkexec
                _EXE=/usr/bin/pkexec
                _CMDLINE=pkexec /usr/lib/update-notifier/package-system-locked
                _CAP_EFFECTIVE=1ffffffffff
                _SELINUX_CONTEXT

                unconfined

                _AUDIT_SESSION=2
                _AUDIT_LOGINUID=1000
                _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-update\x2dnotifier-2811.scope
                _SYSTEMD_OWNER_UID=1000
                _SYSTEMD_UNIT=user@1000.service
                _SYSTEMD_USER_UNIT=app-gnome-update\x2dnotifier-2811.scope
                _SYSTEMD_SLICE=user-1000.slice
                _SYSTEMD_USER_SLICE=app.slice
                _SYSTEMD_INVOCATION_ID=6fbf34863d7445e18cdf0794fd12dd60
                _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
                _HOSTNAME=ubuntu22Acorn
                SYSLOG_TIMESTAMP=Apr  2 01:05:50
                _PID=11738
                _SOURCE_REALTIME_TIMESTAMP=1680422750337080
                 */

                eprintln!();
                eprintln!("• all_data_enumerate has {:?} items", all_data_enumerate.len());
                for data in all_data_enumerate {
                    eprintln!("• all_data_enumerate: {:?}", data);
                }
            }
            let entries: usize = entry_times.len();
            for time in entry_times {
                eprintln!("• entry_times: {:?}", time);
            }
            eprintln!("• journal_next called {:?} times", emerg_stop_next);
            assert!(entries > 1);
        }
    }
}
