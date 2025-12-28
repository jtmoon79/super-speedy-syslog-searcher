// src/readers/journalreader.rs

//! Implements a [`JournalReader`],
//! the driver of deriving [`JournalEntry`s] from a systemd [`.journal` file]
//! using a dynamically loaded [`libsystemd`] library.
//!
//! Sibling of [`SyslogProcessor`] and other "Readers".
//!
//! ### Writer, too
//!
//! However, very different in that `JournalReader` also "writes" the log
//! message to a bytes buffer which is then stored within a `JournalEntry`.
//! A `JournalReader` attempts to match the behavior of some of the
//! [`journalctl`] program `--output` options.
//!
//! ### Sorting
//!
//! ~~`JournalReader` does "pigeon-hole" sorting of log messages by datetime. This
//! differs from `journalctl` which does not attempt any log mesasage sorting
//! beyond what `libsytemd` does.~~ Currently overridden by hardcoded
//! [`DT_USES_SOURCE_OVERRIDE`].
//!
//! ~Implements [Issue #17].~
//!
//! ### `journalctl --output` examples
//!
//! For reference, example calls to `journalctl --output` for one entry:
//!
//! ```text
//! $ PAGER= journalctl --output=short --lines=1 --all --utc --file=./user-1000.journal
//! Apr 01 06:44:32 ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=short-precise --lines=1 --all --utc --file=./user-1000.journal
//! Apr 01 06:44:32.788150 ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=short-iso --lines=1 --all --utc --file=./user-1000.journal
//! 2023-04-01T06:44:32+0000 ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=short-iso-precise --lines=1 --all --utc --file=./user-1000.journal
//! 2023-04-01T06:44:32.788150+0000 ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=short-full --lines=1 --all --utc --file=./user-1000.journal
//! Sat 2023-04-01 06:44:32 UTC ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=short-monotonic --lines=1 --all --utc --file=./user-1000.journal
//! [   74.212842] ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=short-unix --lines=1 --all --utc --file=./user-1000.journal
//! 1680331472.788150 ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=cat --lines=1 --all --utc --file=./user-1000.journal
//! unable to update icon for livepatch
//!
//! $ PAGER= journalctl --output=with-unit --lines=1 --all --utc --file=./user-1000.journal
//! Sat 2023-04-01 06:44:32 UTC ubuntu22Acorn user@1000.service/org.gnome.Shell@wayland.service[1306]: unable to update icon for livepatch
//!
//! $ PAGER= journalctl --lines=1 --output=verbose --all --utc --file=./user-1000.journal
//! Sat 2023-04-01 06:44:32.788150 UTC [s=e992f143877046059b264a0f907056b6;i=6ff;b=26d74a46deff4872be6d4ca6e885a198;m=46c65ea;t=5f840a88a4b39;x=e7933c3b47482d45]
//!     _TRANSPORT=journal
//!     _UID=1000
//!     _GID=1000
//!     _CAP_EFFECTIVE=0
//!     _SELINUX_CONTEXT=unconfined
//!     _AUDIT_SESSION=2
//!     _AUDIT_LOGINUID=1000
//!     _SYSTEMD_OWNER_UID=1000
//!     _SYSTEMD_UNIT=user@1000.service
//!     _SYSTEMD_SLICE=user-1000.slice
//!     _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
//!     _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
//!     _HOSTNAME=ubuntu22Acorn
//!     PRIORITY=4
//!     _SYSTEMD_USER_SLICE=session.slice
//!     _PID=1306
//!     _COMM=gnome-shell
//!     _EXE=/usr/bin/gnome-shell
//!     _CMDLINE=/usr/bin/gnome-shell
//!     _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@wayland.service
//!     _SYSTEMD_USER_UNIT=org.gnome.Shell@wayland.service
//!     _SYSTEMD_INVOCATION_ID=b7d368c96091463aa538006b518785f4
//!     GLIB_DOMAIN=Ubuntu AppIndicators
//!     SYSLOG_IDENTIFIER=ubuntu-appindicators@ubuntu.com
//!     CODE_FILE=/usr/share/gnome-shell/extensions/ubuntu-appindicators@ubuntu.com/appIndicator.js
//!     CODE_LINE=738
//!     CODE_FUNC=_setGicon
//!     MESSAGE=unable to update icon for livepatch
//!     _SOURCE_REALTIME_TIMESTAMP=1680331472788150
//!
//! $ PAGER= journalctl --lines=1 --output=export --all --utc --file=./user-1000.journal
//! __CURSOR=s=e992f143877046059b264a0f907056b6;i=6ff;b=26d74a46deff4872be6d4ca6e885a198;m=46c65ea;t=5f840a88a4b39;x=e7933c3b47482d45
//! __REALTIME_TIMESTAMP=1680331472784185
//! __MONOTONIC_TIMESTAMP=74212842
//! _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
//! _TRANSPORT=journal
//! _UID=1000
//! _GID=1000
//! _CAP_EFFECTIVE=0
//! _SELINUX_CONTEXT
//!
//! unconfined
//!
//! _AUDIT_SESSION=2
//! _AUDIT_LOGINUID=1000
//! _SYSTEMD_OWNER_UID=1000
//! _SYSTEMD_UNIT=user@1000.service
//! _SYSTEMD_SLICE=user-1000.slice
//! _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
//! _HOSTNAME=ubuntu22Acorn
//! PRIORITY=4
//! _SYSTEMD_USER_SLICE=session.slice
//! _PID=1306
//! _COMM=gnome-shell
//! _EXE=/usr/bin/gnome-shell
//! _CMDLINE=/usr/bin/gnome-shell
//! _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@wayland.service
//! _SYSTEMD_USER_UNIT=org.gnome.Shell@wayland.service
//! _SYSTEMD_INVOCATION_ID=b7d368c96091463aa538006b518785f4
//! GLIB_DOMAIN=Ubuntu AppIndicators
//! SYSLOG_IDENTIFIER=ubuntu-appindicators@ubuntu.com
//! CODE_FILE=/usr/share/gnome-shell/extensions/ubuntu-appindicators@ubuntu.com/appIndicator.js
//! CODE_LINE=738
//! CODE_FUNC=_setGicon
//! MESSAGE=unable to update icon for livepatch
//! _SOURCE_REALTIME_TIMESTAMP=1680331472788150
//!
//! $ PAGER= journalctl --output=cat --lines=1 --all --utc --file=./user-1000.journal
//! unable to update icon for livepatch
//! ```
//!
//! [`JournalReader`]: self::JournalReader
//! [`JournalEntry`s]: crate::data::journal::JournalEntry
//! [`.journal` file]: https://systemd.io/JOURNAL_FILE_FORMAT/
//! [`libsystemd`]: https://github.com/systemd/systemd/blob/v249/src/libsystemd/libsystemd.sym
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor
//! [`journalctl`]: https://www.man7.org/linux/man-pages/man1/journalctl.1.html
//! [`JournalReader::fill_buffer`]: self::JournalReader#member.fill_buffer
//! [Issue #17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/17
//! [`DT_USES_SOURCE_OVERRIDE`]: DT_USES_SOURCE_OVERRIDE

use std::collections::{
    BTreeMap,
    HashMap,
};
use std::ffi::{
    CStr,
    CString,
};
use std::io::{
    Error,
    ErrorKind,
    Result,
};
#[cfg(test)]
use std::ops::Range;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use std::string::ToString;
use std::{
    fmt,
    mem,
};

use ::bstr::{
    ByteSlice,
    ByteVec,
};
use ::itertools::Itertools; // for `sorted`
use ::lazy_static::lazy_static;
#[allow(unused_imports)]
use ::more_asserts::{
    assert_le,
    debug_assert_ge,
    debug_assert_le,
    debug_assert_lt,
};
// TRACKING: rust-lang/rust#64797 https://github.com/rust-lang/rust/issues/64797
//           it would be better-suited to use rust feature `#[cfg(accessible(..))]`
//           which is unfortunately still unimplemented
//           Relates to Issue #100 jtmoon79/super-speedy-syslog-searcher#100
//           <https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/100>
#[cfg(not(target_os = "windows"))]
pub use ::nix::errno::Errno;
#[allow(unused_imports)]
use ::si_trace_print::{
    de,
    def1n,
    def1o,
    def1x,
    def1ñ,
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
    pfn,
    pfo,
    pfx,
};
use ::tempfile::NamedTempFile;

use crate::bindings::sd_journal_h::{
    sd_id128,
    sd_journal,
};
use crate::common::{
    Count,
    FPath,
    File,
    FileMetadata,
    FileOpenOptions,
    FileSz,
    FileType,
    FileTypeArchive,
    ResultFind,
    ResultFind4,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    Result_Filter_DateTime1,
    Result_Filter_DateTime2,
    SystemTime,
};
use crate::data::journal::{
    realtime_or_source_realtime_timestamp_to_datetimel,
    realtime_timestamp_to_datetimel,
    DtUsesSource,
    EpochMicroseconds,
    EpochMicrosecondsOpt,
    JournalEntry,
    MonotonicMicroseconds,
    DT_USES_SOURCE_OVERRIDE,
    ENTRY_END_U8,
    FIELD_END_U8,
    FIELD_MID_U8,
};
use crate::de_err;
use crate::libload::systemd_dlopen2::{
    journal_api,
    JournalApiPtr,
};
use crate::readers::filedecompressor::decompress_to_ntf;
use crate::readers::helpers::path_to_fpath;
use crate::readers::summary::Summary;

// XXX: ripped from `nix` crate
// stub `Errno` for to allow Windows to build
// minimal set of `Errno` values that were tested on Windows 11
// XXX: it would be better-suited to use rust feature `#[cfg(accessible(..))]`
//      which is unfortunately still unimplemented, see rust-lang/rust#64797
//      https://github.com/rust-lang/rust/issues/64797
//      Relates to Issue #100 jtmoon79/super-speedy-syslog-searcher#100
//      https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/100
#[cfg(target_os = "windows")]
mod errno {
    use ::libc;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(i32)]
    #[non_exhaustive]
    pub enum Errno {
        UnknownErrno = 0,
        EPERM = libc::EPERM,
        ENOENT = libc::ENOENT,
        ESRCH = libc::ESRCH,
        EINTR = libc::EINTR,
        EIO = libc::EIO,
        ENXIO = libc::ENXIO,
        E2BIG = libc::E2BIG,
        ENOEXEC = libc::ENOEXEC,
        EBADF = libc::EBADF,
        ECHILD = libc::ECHILD,
        EAGAIN = libc::EAGAIN,
        ENOMEM = libc::ENOMEM,
        EACCES = libc::EACCES,
        EFAULT = libc::EFAULT,
        EBUSY = libc::EBUSY,
        EEXIST = libc::EEXIST,
        EXDEV = libc::EXDEV,
        ENODEV = libc::ENODEV,
        ENOTDIR = libc::ENOTDIR,
        EISDIR = libc::EISDIR,
        EINVAL = libc::EINVAL,
        ENFILE = libc::ENFILE,
        EMFILE = libc::EMFILE,
        ENOTTY = libc::ENOTTY,
        ETXTBSY = libc::ETXTBSY,
        EFBIG = libc::EFBIG,
        ENOSPC = libc::ENOSPC,
        ESPIPE = libc::ESPIPE,
        EROFS = libc::EROFS,
        EMLINK = libc::EMLINK,
        EPIPE = libc::EPIPE,
        EDOM = libc::EDOM,
        ERANGE = libc::ERANGE,
        EDEADLK = libc::EDEADLK,
        ENAMETOOLONG = libc::ENAMETOOLONG,
        ENOLCK = libc::ENOLCK,
        ENOSYS = libc::ENOSYS,
        ENOTEMPTY = libc::ENOTEMPTY,
        ELOOP = libc::ELOOP,
        ENOMSG = libc::ENOMSG,
        EIDRM = libc::EIDRM,
        ENOSTR = libc::ENOSTR,
        ENODATA = libc::ENODATA,
        ETIME = libc::ETIME,
        ENOSR = libc::ENOSR,
        ENOLINK = libc::ENOLINK,
        EPROTO = libc::EPROTO,
        EBADMSG = libc::EBADMSG,
        EOVERFLOW = libc::EOVERFLOW,
        EILSEQ = libc::EILSEQ,
        ENOTSOCK = libc::ENOTSOCK,
        EDESTADDRREQ = libc::EDESTADDRREQ,
        EMSGSIZE = libc::EMSGSIZE,
        EPROTOTYPE = libc::EPROTOTYPE,
        ENOPROTOOPT = libc::ENOPROTOOPT,
        EPROTONOSUPPORT = libc::EPROTONOSUPPORT,
        EOPNOTSUPP = libc::EOPNOTSUPP,
        EAFNOSUPPORT = libc::EAFNOSUPPORT,
        EADDRINUSE = libc::EADDRINUSE,
        EADDRNOTAVAIL = libc::EADDRNOTAVAIL,
        ENETDOWN = libc::ENETDOWN,
        ENETUNREACH = libc::ENETUNREACH,
        ENETRESET = libc::ENETRESET,
        ECONNABORTED = libc::ECONNABORTED,
        ECONNRESET = libc::ECONNRESET,
        ENOBUFS = libc::ENOBUFS,
        EISCONN = libc::EISCONN,
        ENOTCONN = libc::ENOTCONN,
        ETIMEDOUT = libc::ETIMEDOUT,
        ECONNREFUSED = libc::ECONNREFUSED,
        EHOSTUNREACH = libc::EHOSTUNREACH,
        EALREADY = libc::EALREADY,
        EINPROGRESS = libc::EINPROGRESS,
        ECANCELED = libc::ECANCELED,
        EOWNERDEAD = libc::EOWNERDEAD,
        ENOTRECOVERABLE = libc::ENOTRECOVERABLE,
    }

    impl Errno {
        pub const EWOULDBLOCK: Errno = Errno::EAGAIN;
        pub const EDEADLOCK: Errno = Errno::EDEADLK;
        pub const ENOTSUP: Errno = Errno::EOPNOTSUPP;
    }

    pub const fn from_raw(e: i32) -> Errno {
        use self::Errno::*;

        match e {
            libc::EPERM => EPERM,
            libc::ENOENT => ENOENT,
            libc::ESRCH => ESRCH,
            libc::EINTR => EINTR,
            libc::EIO => EIO,
            libc::ENXIO => ENXIO,
            libc::E2BIG => E2BIG,
            libc::ENOEXEC => ENOEXEC,
            libc::EBADF => EBADF,
            libc::ECHILD => ECHILD,
            libc::EAGAIN => EAGAIN,
            libc::ENOMEM => ENOMEM,
            libc::EACCES => EACCES,
            libc::EFAULT => EFAULT,
            libc::EBUSY => EBUSY,
            libc::EEXIST => EEXIST,
            libc::EXDEV => EXDEV,
            libc::ENODEV => ENODEV,
            libc::ENOTDIR => ENOTDIR,
            libc::EISDIR => EISDIR,
            libc::EINVAL => EINVAL,
            libc::ENFILE => ENFILE,
            libc::EMFILE => EMFILE,
            libc::ENOTTY => ENOTTY,
            libc::ETXTBSY => ETXTBSY,
            libc::EFBIG => EFBIG,
            libc::ENOSPC => ENOSPC,
            libc::ESPIPE => ESPIPE,
            libc::EROFS => EROFS,
            libc::EMLINK => EMLINK,
            libc::EPIPE => EPIPE,
            libc::EDOM => EDOM,
            libc::ERANGE => ERANGE,
            libc::EDEADLK => EDEADLK,
            libc::ENAMETOOLONG => ENAMETOOLONG,
            libc::ENOLCK => ENOLCK,
            libc::ENOSYS => ENOSYS,
            libc::ENOTEMPTY => ENOTEMPTY,
            libc::ELOOP => ELOOP,
            libc::ENOMSG => ENOMSG,
            libc::EIDRM => EIDRM,
            libc::ENOSTR => ENOSTR,
            libc::ENODATA => ENODATA,
            libc::ETIME => ETIME,
            libc::ENOSR => ENOSR,
            libc::ENOLINK => ENOLINK,
            libc::EPROTO => EPROTO,
            libc::EBADMSG => EBADMSG,
            libc::EOVERFLOW => EOVERFLOW,
            libc::EILSEQ => EILSEQ,
            libc::ENOTSOCK => ENOTSOCK,
            libc::EDESTADDRREQ => EDESTADDRREQ,
            libc::EMSGSIZE => EMSGSIZE,
            libc::EPROTOTYPE => EPROTOTYPE,
            libc::ENOPROTOOPT => ENOPROTOOPT,
            libc::EPROTONOSUPPORT => EPROTONOSUPPORT,
            libc::EOPNOTSUPP => EOPNOTSUPP,
            libc::EAFNOSUPPORT => EAFNOSUPPORT,
            libc::EADDRINUSE => EADDRINUSE,
            libc::EADDRNOTAVAIL => EADDRNOTAVAIL,
            libc::ENETDOWN => ENETDOWN,
            libc::ENETUNREACH => ENETUNREACH,
            libc::ENETRESET => ENETRESET,
            libc::ECONNABORTED => ECONNABORTED,
            libc::ECONNRESET => ECONNRESET,
            libc::ENOBUFS => ENOBUFS,
            libc::EISCONN => EISCONN,
            libc::ENOTCONN => ENOTCONN,
            libc::ETIMEDOUT => ETIMEDOUT,
            libc::ECONNREFUSED => ECONNREFUSED,
            libc::EHOSTUNREACH => EHOSTUNREACH,
            libc::EALREADY => EALREADY,
            libc::EINPROGRESS => EINPROGRESS,
            libc::ECANCELED => ECANCELED,
            libc::EOWNERDEAD => EOWNERDEAD,
            libc::ENOTRECOVERABLE => ENOTRECOVERABLE,
            _ => UnknownErrno,
        }
    }

    impl Errno {
        pub const fn from_raw(err: i32) -> Errno {
            from_raw(err)
        }
    }
}

/// Unix ErrNo (Error Number) type.
#[cfg(target_os = "windows")]
pub type Errno = errno::Errno;

/// Return type for various `JournalReader::next` methods.
pub type ResultNext = ResultFind4<JournalEntry, Error>;
/// Return type for [`JournalReader::next_common`] method.
///
/// [`JournalReader::next_common`]: JournalReader::next_common
pub type ResultNextCommon = ResultFind4<(EpochMicroseconds, EpochMicrosecondsOpt, DtUsesSource), Error>;

#[cfg(test)]
pub type ForceErrorRange = Range<Count>;
#[cfg(test)]
pub type ForceErrorRangeOpt = Option<ForceErrorRange>;

/// Force an error when `journalreader.api_calls` is in the range.
/// Testing only.
macro_rules! testing_force_error {
    (
        $force_error_range_opt:expr,
        $func_name:expr,
        $api_calls:expr,
        $api_call_errors:expr,
        $err_type:expr,
        $path:expr
    ) => {
        #[cfg(test)]
        {
            match $force_error_range_opt {
                Some(range) => {
                    let r: i32 = -99;
                    let start = range.start;
                    let end = range.end;
                    if $api_calls >= start && $api_calls <= end {
                        $api_calls += 1;
                        $api_call_errors += 1;
                        let e = Errno::from_raw(r.abs());
                        def1o!("{}() FORCE_ERROR_RANGE {}; {:?}", $func_name, r, e);
                        let err = JournalReader::Error_from_Errno(r, &e, $func_name, $path);
                        return $err_type(err);
                    }
                }
                None => {}
            }
        }
    };
}

/// `journalctl --output=verbose` field prepend.
pub const FIELD_BEG_VERBOSE: &str = "    ";

#[allow(non_upper_case_globals)]
pub const KEY__CURSOR: &str = "__CURSOR";
pub const KEY_HOSTNAME: &str = "_HOSTNAME";
pub const KEY_HOSTNAME_BYTES: &[u8] = KEY_HOSTNAME.as_bytes();
pub const KEY_SYSLOG_FACILITY: &str = "SYSLOG_FACILITY";
pub const KEY_SYSLOG_FACILITY_BYTES: &[u8] = KEY_SYSLOG_FACILITY.as_bytes();
pub const KEY_SYSLOG_IDENTIFIER: &str = "SYSLOG_IDENTIFIER";
pub const KEY_SYSLOG_IDENTIFIER_BYTES: &[u8] = KEY_SYSLOG_IDENTIFIER.as_bytes();
pub const KEY_SYSLOG_PID: &str = "SYSLOG_PID";
pub const KEY_SYSLOG_PID_BYTES: &[u8] = KEY_SYSLOG_PID.as_bytes();
pub const KEY_COMM: &str = "_COMM";
pub const KEY_COMM_BYTES: &[u8] = KEY_COMM.as_bytes();
pub const KEY_SOURCE_REALTIME_TIMESTAMP: &str = "_SOURCE_REALTIME_TIMESTAMP";
pub const KEY_SOURCE_REALTIME_TIMESTAMP_BYTES: &[u8] = KEY_SOURCE_REALTIME_TIMESTAMP.as_bytes();
#[allow(non_upper_case_globals)]
pub const KEY__REALTIME_TIMESTAMP: &str = "__REALTIME_TIMESTAMP";
#[allow(non_upper_case_globals)]
pub const KEY__REALTIME_TIMESTAMP_BYTES: &[u8] = KEY__REALTIME_TIMESTAMP.as_bytes();
#[allow(non_upper_case_globals)]
pub const KEY__MONOTONIC_TIMESTAMP: &str = "__MONOTONIC_TIMESTAMP";
#[allow(non_upper_case_globals)]
pub const KEY__MONOTONIC_TIMESTAMP_BYTES: &[u8] = KEY__MONOTONIC_TIMESTAMP.as_bytes();
pub const KEY_MESSAGE_ID: &str = "MESSAGE_ID";
pub const KEY_MESSAGE_ID_BYTES: &[u8] = KEY_MESSAGE_ID.as_bytes();
pub const KEY_MESSAGE: &str = "MESSAGE";
pub const KEY_MESSAGE_BYTES: &[u8] = KEY_MESSAGE.as_bytes();
pub const KEY_PID: &str = "_PID";
pub const KEY_PID_BYTES: &[u8] = KEY_PID.as_bytes();
pub const KEY_SELINUX_CONTEXT: &str = "_SELINUX_CONTEXT";
pub const KEY_SELINUX_CONTEXT_BYTES: &[u8] = KEY_SELINUX_CONTEXT.as_bytes();

lazy_static! {
    pub static ref KEY_SOURCE_REALTIME_TIMESTAMP_CSTR: CString = CString::new(KEY_SOURCE_REALTIME_TIMESTAMP).unwrap();
    pub static ref KEY_MESSAGE_CSTR: CString = CString::new(KEY_MESSAGE).unwrap();
}

/// `journalctl` output formats.
///
/// Snippet from `journalctl --help` output from `systemd 249`:
/// ```text
///   -o --output=STRING   Change journal output mode (short, short-precise,
///                        short-iso, short-iso-precise, short-full,
///                        short-monotonic, short-unix, verbose, export,
///                        json, json-pretty, json-sse, json-seq, cat,
///                        with-unit)
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ::clap::ValueEnum)]
pub enum JournalOutput {
    #[default]
    Short,
    ShortPrecise,
    ShortIso,
    ShortIsoPrecise,
    ShortFull,
    ShortMonotonic,
    ShortUnix,
    Verbose,
    Export,
    Cat,
}

/// Should match options shown in `journalctl --help`
impl fmt::Display for JournalOutput {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            JournalOutput::Short => write!(f, "short"),
            JournalOutput::ShortPrecise => write!(f, "short-precise"),
            JournalOutput::ShortIso => write!(f, "short-iso"),
            JournalOutput::ShortIsoPrecise => write!(f, "short-iso-precise"),
            JournalOutput::ShortFull => write!(f, "short-full"),
            JournalOutput::ShortMonotonic => write!(f, "short-monotonic"),
            JournalOutput::ShortUnix => write!(f, "short-unix"),
            JournalOutput::Verbose => write!(f, "verbose"),
            JournalOutput::Export => write!(f, "export"),
            JournalOutput::Cat => write!(f, "cat"),
        }
    }
}

/// Should match options shown in `journalctl --help`
impl FromStr for JournalOutput {
    type Err = ();
    fn from_str(input: &str) -> std::result::Result<JournalOutput, Self::Err> {
        match input {
            "short" => Ok(JournalOutput::Short),
            "short-precise" => Ok(JournalOutput::ShortPrecise),
            "short-iso" => Ok(JournalOutput::ShortIso),
            "short-iso-precise" => Ok(JournalOutput::ShortIsoPrecise),
            "short-full" => Ok(JournalOutput::ShortFull),
            "short-monotonic" => Ok(JournalOutput::ShortMonotonic),
            "short-unix" => Ok(JournalOutput::ShortUnix),
            "verbose" => Ok(JournalOutput::Verbose),
            "export" => Ok(JournalOutput::Export),
            "cat" => Ok(JournalOutput::Cat),
            _ => Err(()),
        }
    }
}

impl JournalOutput {
    pub fn iterator() -> Iter<'static, JournalOutput> {
        static JOURNAL_OUTPUTS: [JournalOutput; (JournalOutput::Cat as usize) + 1] = [
            JournalOutput::Short,
            JournalOutput::ShortPrecise,
            JournalOutput::ShortIso,
            JournalOutput::ShortIsoPrecise,
            JournalOutput::ShortFull,
            JournalOutput::ShortMonotonic,
            JournalOutput::ShortUnix,
            JournalOutput::Verbose,
            JournalOutput::Export,
            JournalOutput::Cat,
        ];
        JOURNAL_OUTPUTS.iter()
    }
}

/// `journalctl --output=short` format
/// Apr 01 06:44:32
const DATETIME_FORMAT_SHORT: &str = "%b %d %H:%M:%S";
/// `journalctl --output=short-precise` format
/// Apr 01 06:44:32.123456
const DATETIME_FORMAT_SHORT_PRECISE: &str = "%b %d %H:%M:%S.%6f";
/// `journalctl --output=short-iso` format
/// 2021-04-01 06:44:32
const DATETIME_FORMAT_SHORT_ISO: &str = "%Y-%m-%d %H:%M:%S";
/// `journalctl --output=short-iso-precise` format
/// 2023-04-01T06:44:32.788150+0000
const DATETIME_FORMAT_SHORT_ISO_PRECISE: &str = "%Y-%m-%dT%H:%M:%S.%6f%z";
/// `journalctl --output=short-full` format
/// Sat 2023-04-01 06:44:32 UTC
const DATETIME_FORMAT_SHORT_FULL: &str = "%a %Y-%m-%d %H:%M:%S %Z";
/// `journalctl --output=short-unix` format
/// 1680331472.788150
const DATETIME_FORMAT_SHORT_UNIX: &str = "%s.%6f";
/// `journalctl --output=verbose` format
/// Sat 2023-04-01 06:44:32.788150 UTC
const DATETIME_FORMAT_VERBOSE: &str = "%a %Y-%m-%d %H:%M:%S.%6f %Z";
// NOTE:  `journalctl --output=short-monotonic` format is not a datetime but
//        an offset from the boot time surrounded by `[]`. e.g.
// [   74.212842]

/// Map a few of the most common `Errno` errors to a formal `ErrorKind`.
/// Give a little bit more information beyond catch-all `ErrorKind::Other`.
// TODO: [2023/04] submit PR to `nix` project with a hard-coded mapping
//       of `Errno` to `ErrorKind`
pub fn errno_to_errorkind(err: &Errno) -> ErrorKind {
    match *err {
        Errno::EACCES => ErrorKind::PermissionDenied,
        Errno::EADDRINUSE => ErrorKind::AddrInUse,
        Errno::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
        Errno::EAFNOSUPPORT => ErrorKind::AddrNotAvailable,
        Errno::EALREADY => ErrorKind::AlreadyExists,
        Errno::EBADF => ErrorKind::InvalidInput,
        Errno::EBADMSG => ErrorKind::InvalidData,
        Errno::EBUSY => ErrorKind::Other,
        Errno::ECANCELED => ErrorKind::Interrupted,
        _ => ErrorKind::Other,
    }
}

/// How does the passed `em` pass the optional `EpochMicrosecondsOpt`
/// filter instances, `em_filter_after` and `em_filter_before`?
/// Is `em` before ([`BeforeRange`]),
/// after ([`AfterRange`]), or in between ([`InRange`])?
///
/// Smilar to [`dt_pass_filters`] that takes a `EpochMicroseconds`
/// instead of a [`DateTimeL`].
///
/// [`dt_pass_filters`]: crate::data::datetime::dt_pass_filters
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
/// [`AfterRange`]: crate::data::datetime::Result_Filter_DateTime2::AfterRange
/// [`BeforeRange`]: crate::data::datetime::Result_Filter_DateTime2::BeforeRange
/// [`InRange`]: crate::data::datetime::Result_Filter_DateTime2::InRange
pub fn em_pass_filters(
    em: &EpochMicroseconds,
    em_filter_after: &EpochMicrosecondsOpt,
    em_filter_before: &EpochMicrosecondsOpt,
) -> Result_Filter_DateTime2 {
    defn!("({:?}, {:?}, {:?})", em, em_filter_after, em_filter_before);
    match (em_filter_after, em_filter_before) {
        (None, None) => {
            defx!("return InRange; (no dt filters)");

            Result_Filter_DateTime2::InRange
        }
        (Some(em_a), Some(em_b)) => {
            debug_assert_le!(em_a, em_b, "Bad datetime range values filter_after {:?} {:?} filter_before", em_a, em_b);
            if em < em_a {
                defx!("return BeforeRange");
                return Result_Filter_DateTime2::BeforeRange;
            }
            if em_b < em {
                defx!("return AfterRange");
                return Result_Filter_DateTime2::AfterRange;
            }
            // assert em_a < dt && em < em_b
            debug_assert_le!(em_a, em, "Unexpected range values em_a em");
            debug_assert_le!(em, em_b, "Unexpected range values em em_b");
            defx!("return InRange");

            Result_Filter_DateTime2::InRange
        }
        (Some(em_a), None) => {
            if em < em_a {
                defx!("return BeforeRange");
                return Result_Filter_DateTime2::BeforeRange;
            }
            defx!("return InRange");

            Result_Filter_DateTime2::InRange
        }
        (None, Some(em_b)) => {
            if em_b < em {
                defx!("return AfterRange");
                return Result_Filter_DateTime2::AfterRange;
            }
            defx!("return InRange");

            Result_Filter_DateTime2::InRange
        }
    }
}

/// Compare passed `em` to the passed filter `em_filter`.
///
/// Similar to [`dt_after_or_before`] that takes a `EpochMicroseconds`
/// instead of a [`DateTimeL`].
///
/// If `dt` is at or after `dt_filter` then return [`OccursAtOrAfter`]<br/>
/// If `dt` is before `dt_filter` then return [`OccursBefore`]<br/>
/// Else return [`Pass`] (including if `dt_filter` is `None`)
///
/// [`OccursAtOrAfter`]: crate::data::datetime::Result_Filter_DateTime1
/// [`OccursBefore`]: crate::data::datetime::Result_Filter_DateTime1
/// [`Pass`]: crate::data::datetime::Result_Filter_DateTime1
/// [`dt_after_or_before`]: crate::data::datetime::dt_after_or_before
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
pub fn em_after_or_before(
    em: &EpochMicroseconds,
    em_filter: &EpochMicrosecondsOpt,
) -> Result_Filter_DateTime1 {
    if em_filter.is_none() {
        defñ!("return Pass; (no dt filters)");
        return Result_Filter_DateTime1::Pass;
    }

    let em_a = &em_filter.unwrap();
    if em < em_a {
        defñ!("return OccursBefore; (em {:?} is before em_filter {:?})", em, em_a);
        return Result_Filter_DateTime1::OccursBefore;
    }
    defñ!("return OccursAtOrAfter; (em {:?} is at or after em_filter {:?})", em, em_a);

    Result_Filter_DateTime1::OccursAtOrAfter
}

// TODO: change to a typed `struct EntryBufferKey(...)`
type EntryBufferKey = (DateTimeL, usize);
type EntryBuffer = BTreeMap<EntryBufferKey, JournalEntry>;

/// A wrapper for parsing a systemd [`.journal` file] using [`libsystemd` API].
///
/// The `JournalReader` also writes the data to byte buffer which is later
/// passed to a printer. So this "Reader" does some amount of "writing".
/// This writing is done in the `next*` functions.
///
/// ### Sorting Entries
///
/// ~~`JournalReader` does "pigeon-hole" sorting of Journal Entries. The
/// developer found some `.journal` files had entries not in chronological
/// order (according to `journalctl` output). These out-of-order entries were
/// very often within tens log messages of each other,
/// and within tens of milliseconds of each other.
/// It appeared that a matter of an overloaded `systemd` service
/// will write incoming journal log entries in the happenstance order of
/// multi-threaded processing. The result is a clustering of out-of-order
/// entries within a small realtime window.~~
///
/// ~~So this `JournalReader` buffers a fixed amount of processed entries
/// ([`ENTRY_BUFFER_SZ`]), storing them in a `BTreeMap` sorted by `DateTimeL`
/// and order encountered.
/// The first call to `next_fill_buffer` will fill the `JournalReader`
/// buffer and then one entry from that buffer (the earlier entry). Subsequent
/// calls to `next_fill_buffer` will read another journal entry, store it in the
/// sorted buffer, and then pop the earliest journal entry from the buffer and
/// return that.~~
///
/// Overridden with [`DT_USES_SOURCE_OVERRIDE`].
///
/// ### Datetime source
///
/// The sorting of entries is based on the entry datetime. This can be taken
/// from either field `__REALTIME_TIMESTAMP` or
/// field `_SOURCE_REALTIME_TIMESTAMP`.
/// The `JournalReader` can follow the behavior of `journalctl` by preferring
/// the `_SOURCE_REALTIME_TIMESTAMP` field if it exists. The
/// `__REALTIME_TIMESTAMP` field always exists and is a reliable fallback. The
/// value is retrieved via the
/// `sd_journal_get_realtime_usec` API function in `libsystemd`.
///
/// The value retrieved is microseconds since the Unix Epoch. It is converted
/// to a [`DateTimeL`] by `JournalReader` or the [`JournalEntry`].
///
/// However...
///
/// #### Override
///
/// It was found that the `_SOURCE_REALTIME_TIMESTAMP` field is
/// not always available, differs from the `__REALTIME_TIMESTAMP` field, and
/// is not always in chronological order. This can lead to unexpected printing
/// of journal entries that do not appear in chronological order.
///
/// Reviewing the journal file
/// `./logs/programs/journal/RHE_91_system.journal` revealed that the
/// `__REALTIME_TIMESTAMP` field was always in chronological order _and_
/// (most importantly) appears to be a more accurate chronology of events.
///
/// For example, using `./tools/journal_print.py`, the following output was
/// observed:
///
/// ```text
/// $ ./tools/journal_print.py ./logs/programs/journal/RHE_91_system.journal | column -t -s '|'
/// index  __MONOTONIC_TIMESTAMP  _SOURCE_REALTIME_TIMESTAMP  __REALTIME_TIMESTAMP        MESSAGE_ID                        MESSAGE
/// 1      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    Linux version 5.14.0-162.6.1.el9_1.x86_64 (mockbuild@x86-vm-07.build.eng.bos.redhat.com) (gcc (GCC) 11.3.1 20220421 (Red Hat 11.3.1-2), GNU ld version 2.35.2-24.el9) #1 SMP PREEMPT_DYNAMIC Fri Sep 30 07:36:03 EDT 2022
/// 2      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    The list of certified hardware and cloud instances for Red Hat Enterprise Linux 9 can be viewed at the Red Hat Ecosystem Catalog, https://catalog.redhat.com.
/// 3      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    Command line: BOOT_IMAGE=(hd0,msdos1)/vmlinuz-5.14.0-162.6.1.el9_1.x86_64 root=/dev/mapper/rhel-root ro crashkernel=1G-4G:192M,4G-64G:256M,64G-:512M resume=/dev/mapper/rhel-swap rd.lvm.lv=rhel/root rd.lvm.lv=rhel/swap rhgb quiet
/// 4      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    [Firmware Bug]: TSC doesn't count with P0 frequency!
/// 5      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    x86/fpu: x87 FPU will use FXSAVE
/// 6      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    signal: max sigframe size: 1440
/// 7      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    BIOS-provided physical RAM map:
/// 8      0:00:04.554769                                     2023-04-10 13:56:30.138000                                    BIOS-e820: [mem 0x0000000000000000-0x000000000009fbff] usable
/// …
/// 428    0:00:04.554769                                     2023-04-10 13:56:30.138000                                    Finished Create List of Static Device Nodes.
/// 429    0:00:04.554769                                     2023-04-10 13:56:30.138000                                    Finished Create System Users.
/// 430    0:00:04.554769                                     2023-04-10 13:56:30.138000                                    Starting Create Static Device Nodes in /dev...
/// 431    0:00:04.554769                                     2023-04-10 13:56:30.138000  f77379a8490b408bbe5f6940505a777b  Journal started
/// 432    0:00:04.554769                                     2023-04-10 13:56:30.138000  ec387f577b844b8fa948f33cad9a75e6  Runtime Journal (/run/log/journal/b52998310a374c8b9c676f49cc62d044) is 4.4M, max 35.4M, 30.9M free.
/// 433    0:00:04.554769         2023-04-10 13:56:30.131199  2023-04-10 13:56:30.138000                                    Suggested group ID 65534 for nobody already used.
/// 434    0:00:04.554769         2023-04-10 13:56:30.131762  2023-04-10 13:56:30.138000                                    Creating group 'nobody' with GID 997.
/// 435    0:00:04.554769         2023-04-10 13:56:30.132320  2023-04-10 13:56:30.138000                                    Creating group 'users' with GID 100.
/// 436    0:00:04.554769         2023-04-10 13:56:30.132606  2023-04-10 13:56:30.138000                                    Creating group 'dbus' with GID 81.
/// 437    0:00:04.554769         2023-04-10 13:56:30.135833  2023-04-10 13:56:30.138000                                    Creating user 'dbus' (System Message Bus) with UID 81 and GID 81.
/// 438    0:00:04.560769         2023-04-10 13:56:30.144435  2023-04-10 13:56:30.144000  39f53479d3a045ac8e11786248231fbf  Finished Create Static Device Nodes in /dev.
/// …
/// ```
///
/// The index value is the enumerated order of the entries as they are returned
/// by `libsystemd` API call [`sd_journal_next`].
///
/// Entry 433 has a `_SOURCE_REALTIME_TIMESTAMP` that is earlier than the
/// `__REALTIME_TIMESTAMP` of all prior entries. But by reading through the
/// entry `MESSAGE` fields, it's obvious that it wouldn't make sense to have
/// entry 433 printed as the first entry emitted from this journal file.
/// Entry 1 is clearly the standard Linux boot start first message,
/// and the following entries are the typical boot startup messages.
/// So the `_SOURCE_REALTIME_TIMESTAMP` of 433 is inaccurate. The
/// `__REALTIME_TIMESTAMP` combined with the index is the best ordering of these
/// messages.
/// The same is true for some other entries shown.
///
/// Confusingly, the `journalctl` program will prefer to print the
/// `_SOURCE_REALTIME_TIMESTAMP` but will still use the enumerated order, the
/// same order printed above. The user will see entry 433 as
/// backwards in time, which is very confusing.
///
/// So this program just uses the enumerated order, and always uses
/// `__REALTIME_TIMESTAMP` to determine the datetime of the entry.
///
/// Though `JournalReader` can perform different sorting behavior, that is
/// currently overridden by hardcoded [`DT_USES_SOURCE_OVERRIDE`].
///
/// See [Issue #101].
///
/// ### Comparison to `journalctl`
///
/// The user can compare to sorted `journalctl` output against `s4` output
/// using commands:
/// ```sh
/// $ PAGER= journalctl --output=short-iso-precise --file=./file.journal > journalctl.txt
/// $ s4 --journal-output=short-iso-precise --color=never ./file.journal > s4.txt
/// $ diff -W $COLUMNS --side-by-side --suppress-common-lines journalctl.txt s4.txt
/// ```
///
/// [`ENTRY_BUFFER_SZ`]: Self::ENTRY_BUFFER_SZ
/// [`.journal` file]: https://systemd.io/JOURNAL_FILE_FORMAT/
/// [`DT_USES_SOURCE_OVERRIDE`]: crate::data::journal::DT_USES_SOURCE_OVERRIDE
/// [`JournalEntry`]: crate::data::journal::JournalEntry
/// [`libsystemd` API]: https://www.freedesktop.org/software/systemd/man/sd_journal.html#
/// [`sd_journal_next`]: https://www.man7.org/linux/man-pages/man3/SD_JOURNAL_FOREACH.3.html
/// [Issue #101]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/101
pub struct JournalReader {
    /// The [`sd_journal`] handle stored on the heap.
    _journal_handle: Box<sd_journal>,
    /// A pointer to the [`sd_journal`] handle.
    journal_handle_ptr: *mut sd_journal,
    /// The [`JournalApiPtr`] dynamic library interface.
    journal_api_ptr: JournalApiPtr,
    /// Buffer of [`JournalEntry`]s.
    /// Push new `JournalEntry`s. Pop in order of `DateTimeL`.
    fill_buffer: EntryBuffer,
    /// Internal helper to `next_fill_buffer`. A not-fancy way to maintain
    /// state within `next_fill_buffer`.
    next_fill_buffer_index: usize,
    /// Internal helper to `next_fill_buffer`. A not-fancy way to maintain
    /// state within `next_fill_buffer`.
    next_fill_buffer_loop: bool,
    /// The [`FPath`] of the file being read.
    ///
    /// [`FPath`]: crate::common::FPath
    path: FPath,
    /// If necessary, the extracted journal file as a temporary file.
    named_temp_file: Option<NamedTempFile>,
    /// The [`JournalOutput`] for the file being read.
    /// Derived from `journalctl --output` options.
    journal_output: JournalOutput,
    /// The `FixedOffset` used for `DateTime` creation.
    fixed_offset: FixedOffset,
    /// `Count` of [`JournalEntry`s] processed.
    ///
    /// [`JournalEntry`s]: crate::data::journal::JournalEntry
    //pub(super) events_processed: Box<Count>,
    pub(super) events_processed: Count,
    /// `Count` of [`JournalEntry`s] accepted by the datetime filters.
    ///
    /// [`JournalEntry`s]: crate::data::journal::JournalEntry
    pub(super) events_accepted: Count,
    /// First (soonest) accepted (printed) `EpochMicroseconds`.
    ///
    /// Intended for `--summary`.
    pub(super) ts_first_accepted: EpochMicrosecondsOpt,
    /// Last (latest) accepted (printed) `EpochMicroseconds`.
    ///
    /// Intended for `--summary`.
    pub(super) ts_last_accepted: EpochMicrosecondsOpt,
    /// First (soonest) processed `EpochMicroseconds`.
    ///
    /// Intended for `--summary`.
    pub(super) ts_first_processed: EpochMicrosecondsOpt,
    /// Last (latest) processed `EpochMicroseconds`.
    ///
    /// Intended for `--summary`.
    pub(super) ts_last_processed: EpochMicrosecondsOpt,
    /// File Size of the file being read in bytes.
    filesz: FileSz,
    /// file Last Modified time from file-system metadata
    mtime: SystemTime,
    /// Has `self.analyze()` been called?
    analyzed: bool,
    /// Number of systemd API calls (calls using `journal_api_ptr`).
    api_calls: Count,
    /// Number of systemd API calls that returned an unexpected error.
    api_call_errors: Count,
    /// Out of chronological order.
    out_of_order: Count,
    /// The last [`Error`], if any, as a `String`
    ///
    /// Annoyingly, cannot [Clone or Copy `Error`].
    ///
    /// [`Error`]: std::io::Error
    /// [Clone or Copy `Error`]: https://github.com/rust-lang/rust/issues/24135
    // TRACKING: https://github.com/rust-lang/rust/issues/24135
    error: Option<String>,
    #[cfg(test)]
    pub(crate) force_error_range_opt: ForceErrorRangeOpt,
}

impl fmt::Debug for JournalReader {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("JournalReader")
            .field("Path", &self.path)
            .field("Error?", &self.error)
            .finish()
    }
}

// TODO: [2023/04] remove redundant variable prefix name `journalreader_`
// TODO: [2023/05] instead of having 1:1 manual copying of `JournalReader`
//       fields to `SummaryJournalReader` fields, just store a
//       `SummaryJournalReader` in `JournalReader` and update directly.
#[allow(non_snake_case)]
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct SummaryJournalReader {
    /// Returned by `sd_journal_enumerate_data`.
    pub journalreader_events_processed: Count,
    /// Acceptable to datetime filters and sent to main thread for printing.
    pub journalreader_events_accepted: Count,
    /// datetime soonest accepted (printed)
    pub journalreader_datetime_first_accepted: DateTimeLOpt,
    /// datetime latest accepted (printed)
    pub journalreader_datetime_last_accepted: DateTimeLOpt,
    /// datetime soonest processed
    pub journalreader_datetime_first_processed: DateTimeLOpt,
    /// datetime latest processed
    pub journalreader_datetime_last_processed: DateTimeLOpt,
    pub journalreader_filesz: FileSz,
    pub journalreader_api_calls: Count,
    pub journalreader_api_call_errors: Count,
    pub journalreader_out_of_order: Count,
}

/// Implement the JournalReader.
impl<'a> JournalReader {
    /// This many `JournalEntry` will be held in memory before any are
    /// sent to the main printing thread (unless there are fewer available).
    pub const ENTRY_BUFFER_SZ: usize = 0x200 - 1;

    /// Create a new `JournalReader`.
    ///
    /// NOTE: should not attempt any file reads here,
    /// similar to other `*Readers::new()` unless the file is compressed
    /// or archived.
    pub fn new(
        path: FPath,
        journal_output: JournalOutput,
        fixed_offset: FixedOffset,
        file_type: FileType,
    ) -> Result<JournalReader> {
        def1n!("({:?}, {:?}, {:?})", path, fixed_offset, file_type);

        // get the file size according to the file metadata
        let path_std: &Path = Path::new(&path);
        let mut open_options = FileOpenOptions::new();
        let named_temp_file: Option<NamedTempFile>;
        let mtime_opt: Option<SystemTime>;

        (named_temp_file, mtime_opt) = match decompress_to_ntf(&path_std, &file_type) {
            Ok(ntf_mtime) => match ntf_mtime {
                Some((ntf, mtime_opt, _filesz)) => (Some(ntf), mtime_opt),
                None => (None, None),
            },
            Err(err) => {
                def1x!("decompress_to_ntf({:?}, {:?}) Error, return {:?}", path, file_type, err,);
                return Err(err);
            }
        };
        def1o!("named_temp_file {:?}", named_temp_file);
        def1o!("mtime_opt {:?}", mtime_opt);

        let path_actual: &Path = match named_temp_file {
            Some(ref ntf) => ntf.path(),
            None => path_std,
        };
        def1o!("path_actual {:?}", path_actual);
        def1o!("open_options.read(true).open({:?})", path_actual);
        let file: File = match open_options
            .read(true)
            .open(path_actual)
        {
            Result::Ok(val) => val,
            Result::Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let metadata: FileMetadata = match file.metadata() {
            Result::Ok(val) => val,
            Result::Err(err) => {
                def1x!("return {:?}", err);
                return Err(err);
            }
        };
        let filesz: FileSz = metadata.len() as FileSz;
        def1o!("filesz {:?}", filesz);

        let mtime: SystemTime = match mtime_opt {
            Some(val) => val,
            None => match metadata.modified() {
                Result::Ok(val) => val,
                Result::Err(_err) => {
                    de_err!("metadata.modified() failed {}", _err);
                    SystemTime::UNIX_EPOCH
                }
            },
        };
        def1o!("mtime {:?}", mtime);

        // create the `JournalFile` file descriptor handle
        let mut journal_handle: Box<sd_journal> = Box::new(sd_journal { _unused: [0; 0] });
        def1o!("journal_handle @{:p}", journal_handle.as_ref());
        let path_cs: CString = match named_temp_file {
            Some(ref ntf) => {
                let fpath: FPath = path_to_fpath(ntf.path());
                def1o!("fpath {:?}", fpath);
                CString::new(fpath.as_str()).unwrap()
            }
            None => CString::new(path_std.to_str().unwrap()).unwrap(),
        };
        def1o!("path_cs {:?}", path_cs);

        let mut journal_handle_ptr: *mut sd_journal = journal_handle.as_mut();
        def1o!("*journal_handle @{:?}", journal_handle_ptr);
        let journal_api_ptr = journal_api();
        unsafe {
            //
            // call sd_journal_open_files
            //
            let ppath: *const ::std::os::raw::c_char = path_cs.as_bytes().as_ptr() as *const ::std::os::raw::c_char;
            let mut ppaths: [*const ::std::os::raw::c_char; 2] = [ppath, ::std::ptr::null()];
            let pppaths: *mut *const ::std::os::raw::c_char = ppaths.as_mut_ptr();
            def1o!("sd_journal_open_files(@{:?}, {:?}, 0)", journal_handle_ptr, path_cs);
            let r: i32 = (*journal_api_ptr).sd_journal_open_files(&mut journal_handle_ptr, pppaths, 0);
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_open_files returned {}, {:?}", r, e);
            if r < 0 {
                let err = Error::new(
                    errno_to_errorkind(&e),
                    format!("sd_journal_open_files({:?}) returned {}; {:?}", path_cs, r, e),
                );
                def1x!("return {:?}", err);
                return Err(err);
            }
        }
        let fill_buffer: EntryBuffer = EntryBuffer::new();

        def1x!("return Ok(JournalReader)");

        Result::Ok(JournalReader {
            _journal_handle: journal_handle,
            journal_handle_ptr,
            journal_api_ptr,
            fill_buffer,
            next_fill_buffer_index: 0,
            next_fill_buffer_loop: true,
            path,
            named_temp_file,
            journal_output,
            fixed_offset,
            events_processed: 0,
            events_accepted: 0,
            ts_first_accepted: EpochMicrosecondsOpt::None,
            ts_last_accepted: EpochMicrosecondsOpt::None,
            ts_first_processed: EpochMicrosecondsOpt::None,
            ts_last_processed: EpochMicrosecondsOpt::None,
            filesz,
            mtime,
            analyzed: false,
            api_calls: 1,
            api_call_errors: 0,
            out_of_order: 0,
            error: None,
            #[cfg(test)]
            force_error_range_opt: None,
        })
    }

    pub const fn mtime(&self) -> SystemTime {
        self.mtime
    }

    /// Set the file cursor to the first record after the given
    /// `ts_filter_after` or the first record if `None`.
    ///
    /// This must be called once before calling `next`.
    pub fn analyze(
        &mut self,
        ts_filter_after: &EpochMicrosecondsOpt,
    ) -> Result<()> {
        def1n!("({:?})", ts_filter_after);

        match ts_filter_after {
            Some(ts) => {
                //
                // call sd_journal_seek_realtime_usec
                //
                unsafe {
                    def1o!("sd_journal_seek_realtime_usec(@{:?},  {})", self.journal_handle_ptr, ts);
                    let r: i32 = (*self.journal_api_ptr).sd_journal_seek_realtime_usec(
                        self.journal_handle_ptr,
                        *ts
                    );
                    let e = Errno::from_raw(r.abs());
                    def1o!("sd_journal_seek_realtime_usec returned {}, {:?}", r, e);
                    self.api_calls += 1;
                    if r < 0 {
                        let err = Error::new(
                            errno_to_errorkind(&e),
                            format!(
                                "sd_journal_seek_realtime_usec({:?}) returned {}; {:?} file {:?}",
                                ts, r, e, self.path
                            ),
                        );
                        def1x!("return {:?}", err);
                        return Err(err);
                    }
                }
            }
            None => {
                unsafe {
                    //
                    // call sd_journal_seek_head
                    //
                    def1o!("sd_journal_seek_head(@{:?})", self.journal_handle_ptr);
                    let r: i32 = (*self.journal_api_ptr).sd_journal_seek_head(self.journal_handle_ptr);
                    let e = Errno::from_raw(r.abs());
                    def1o!("sd_journal_seek_head returned {}, {:?}", r, e);
                    self.api_calls += 1;
                    if r < 0 {
                        self.api_call_errors += 1;
                        let err = Error::new(
                            errno_to_errorkind(&e),
                            format!("sd_journal_seek_head() returned {}; {:?} file {:?}", r, e, self.path),
                        );
                        def1x!("return {:?}", err);
                        return Err(err);
                    }
                }
            }
        }
        self.analyzed = true;
        def1x!();

        Result::Ok(())
    }

    const BUF_DEFAULT_LARGE_SZ: usize = 0x600;
    const BUF_DEFAULT_MEDIUM_SZ: usize = 0x500;
    const BUF_DEFAULT_SMALL_SZ: usize = 0x200;

    /// Helper to create an `Error` from an `Errno`.
    #[allow(non_snake_case)]
    fn Error_from_Errno(
        r: i32,
        e: &Errno,
        funcname: &str,
        path: &FPath,
    ) -> Error {
        Error::new(errno_to_errorkind(e), format!("{} returned {}; {:?} file {:?}", funcname, r, e, path))
    }

    // The following `fn call_*` functions have a somewhat awkward
    // implementation. `self` cannot be passed as a mutable reference
    // because multiple mutable borrows then occur. So callers must pass
    // individual references to members of `self`.

    /// Wrapper to call `sd_journal_next`.
    fn call_sd_journal_next(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
        path: &FPath,
    ) -> ResultFind<EpochMicroseconds, Error> {
        unsafe {
            def1n!("sd_journal_next(@{:?})", journal_handle_ptr);
            let r: i32 = (*journal_api_ptr).sd_journal_next(*journal_handle_ptr);
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_next returned {}, {:?}", r, e);
            *api_calls += 1;
            if r == 0 {
                def1x!("return Done");
                return ResultFind::Done;
            } else if r < 0 {
                *api_call_errors += 1;
                let err = Self::Error_from_Errno(r, &e, "sd_journal_next", path);
                def1x!("return {:?}", err);
                return ResultFind::Err(err);
            }
        }
        def1x!("Found");

        ResultFind::Found(0)
    }

    /// Wrapper to call `sd_journal_get_realtime_usec` and return the
    /// timestamp. Returns `None` if the call fails.
    ///
    /// This API call is the reliably available timestamp, usually stored in
    /// field `__REALTIME_TIMESTAMP`.
    ///
    /// Other fields like `_SOURCE_REALTIME_TIMESTAMP` are not always available.
    /// Field `SYSLOG_TIMESTAMP` is ad-hoc formatted and not needed for deriving
    /// a datetime.
    ///
    /// Know that `journalctl` prefers `_SOURCE_REALTIME_TIMESTAMP` over
    /// `__REALTIME_TIMESTAMP`.
    fn call_sd_journal_get_realtime_usec(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
        path: &FPath,
    ) -> Result<EpochMicroseconds> {
        let mut rt: std::os::raw::c_ulonglong = 0;
        unsafe {
            let prt: *mut std::os::raw::c_ulonglong = &mut rt;
            def1n!("sd_journal_get_realtime_usec(@{:p}, @{:p})", journal_handle_ptr, prt);
            let r: i32 = (*journal_api_ptr).sd_journal_get_realtime_usec(
                *journal_handle_ptr,
                prt
            );
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_get_realtime_usec returned {}, {:?}", r, e);
            *api_calls += 1;
            if r < 0 {
                *api_call_errors += 1;
                de_err!("sd_journal_get_realtime_usec() returned {}; {:?}", r, e);
                let err = Self::Error_from_Errno(r, &e, "sd_journal_get_realtime_usec", path);
                return Result::Err(err);
            } else {
                def1o!("sd_journal_get_realtime_usec realtime {}", rt);
            }
        }

        Result::Ok(rt as EpochMicroseconds)
    }

    /// Wrapper to call `sd_journal_get_data` and return the
    /// timestamp, then convert the timestamp.
    /// Returns `None` if the timestamp is not available.
    fn get_source_realtime_timestamp(&mut self) -> Option<EpochMicroseconds> {
        defn!();
        let data: &[u8] = match Self::call_sd_journal_get_data(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
            &KEY_SOURCE_REALTIME_TIMESTAMP_CSTR,
            &self.path,
            #[cfg(test)]
            &self.force_error_range_opt,
        ) {
            Result::Ok(data) => data,
            Result::Err(_e) => {
                // XXX: kludge: cover up the error from this particular API call
                self.api_call_errors -= 1;
                return None;
            }
        };
        let value: &[u8] = match data.find_byte(b'=') {
            Some(at) => {
                let b = std::cmp::min(at + 1,  data.len());

                &data[b..]
            },
            None => return None
        };
        let value_s: &str = match std::str::from_utf8(value) {
            Ok(s) => s,
            Err(_e) => return None
        };
        let source_realtime_timestamp: EpochMicroseconds =
            match EpochMicroseconds::from_str(value_s) {
                Ok(ts) => ts,
                Err(_e) => return None
            };

        defx!("source_realtime_timestamp={}", source_realtime_timestamp);
        Some(source_realtime_timestamp)
    }

    /// Wrapper to call `sd_journal_get_data`.
    /// Returns `Err` if the call fails.
    fn call_sd_journal_get_data(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
        field: &CString,
        path: &FPath,
        #[cfg(test)]
        force_error_range: &ForceErrorRangeOpt,
    ) -> Result<&'a [u8]> {
        testing_force_error!(
            &force_error_range,
            "call_sd_journal_get_data",
            (*api_calls),
            (*api_call_errors),
            Result::Err,
            path
        );
        def1n!("(@{:p}, {:?})", journal_handle_ptr, field);
        let data: &[u8];
        let pcfield = field.as_ptr();
        unsafe {
            //
            // call sd_journal_get_data
            //
            let mut length: std::os::raw::c_ulong = 0;
            let plength: *mut std::os::raw::c_ulong = &mut length;
            let mut pdata: *const std::os::raw::c_void = mem::zeroed();
            let ppdata: *mut *const std::os::raw::c_void = &mut pdata;
            def1o!("sd_journal_get_data(@{:p}, @{:p}, @{:p}, @{:p})", journal_handle_ptr, pcfield, ppdata, plength);
            let r: i32 = (*journal_api_ptr).sd_journal_get_data(*journal_handle_ptr, pcfield, ppdata, plength);
            *api_calls += 1;
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_get_data returned {}, {:?}", r, e);
            if r < 0 {
                *api_call_errors += 1;
                let err = Self::Error_from_Errno(r, &e, "sd_journal_get_data", path);
                def1x!("return Err");
                return Result::Err(err);
            }
            def1o!("sd_journal_get_data returned data length {}", length);
            def1o!("sd_journal_get_data returned pdata {:?}", pdata);
            data = std::slice::from_raw_parts(pdata as *const u8, length as usize);
        } // end unsafe
        def1x!();

        Result::Ok(data)
    }

    /// Wrapper to call `sd_journal_enumerate_available_data`.
    /// Returns `Err` if the call fails.
    /// Returns `Done` if there is no more data.
    fn call_sd_journal_enumerate_available_data(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
        path: &FPath,
        #[cfg(test)]
        force_error_range: &ForceErrorRangeOpt,
    ) -> ResultFind<&'a [u8], Error> {
        def1n!();
        testing_force_error!(&force_error_range, "call_sd_journal_enumerate_available_data", (*api_calls), (*api_call_errors), ResultFind::Err, path);
        let data: &[u8];
        unsafe {
            //
            // call sd_journal_enumerate_available_data
            //
            let mut pdata: *const std::os::raw::c_void = mem::zeroed();
            let ppdata: *mut *const std::os::raw::c_void = &mut pdata;
            let mut length: std::os::raw::c_ulong = 0;
            let plength: *mut std::os::raw::c_ulong = &mut length;
            def1o!("sd_journal_enumerate_available_data(@{:p}, @{:p}, @{:p})", journal_handle_ptr, ppdata, plength);
            let r: i32 = (*journal_api_ptr).sd_journal_enumerate_available_data(
                *journal_handle_ptr,
                ppdata,
                plength,
            );
            *api_calls += 1;
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_enumerate_available_data returned {}, {:?}", r, e);
            if r == 0 {
                def1x!("return Done");
                return ResultFind::Done;
            }
            if r < 0 {
                *api_call_errors += 1;
                de_err!("sd_journal_enumerate_available_data() returned {}; {:?}", r, e);
                let err = Self::Error_from_Errno(r, &e, "sd_journal_enumerate_available_data", path);
                def1x!("return Err");
                return ResultFind::Err(err);
            }
            // length is number of bytes
            def1o!("sd_journal_enumerate_available_data returned data length {}", length);
            def1o!("sd_journal_enumerate_available_data returned pdata {:?}", pdata);
            data = std::slice::from_raw_parts(pdata as *const u8, length as usize);
        } // end unsafe
        def1x!("return Found");

        ResultFind::Found(data)
    }

    /// Wrapper to call `sd_journal_get_monotonic_usec`.
    /// Returns the `Ok(value)` if the call succeeds.
    /// Returns `Err` if the call fails.
    fn call_sd_journal_get_monotonic_usec(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
        boot_id: Option<sd_id128>,
        path: &FPath,
    ) -> Result<MonotonicMicroseconds> {
        def1n!();
        let monotonic_usec: MonotonicMicroseconds;
        let mut boot_id_id128: sd_id128 = match boot_id {
            Some(b) => b,
            None => sd_id128 { bytes: [0; 16] },
        };
        unsafe {
            let mut rt: std::os::raw::c_ulonglong = 0;
            let prt: *mut std::os::raw::c_ulonglong = &mut rt;
            let pboot: *mut sd_id128 = &mut boot_id_id128 as *mut _ as *mut sd_id128;
            def1o!("sd_journal_get_monotonic_usec(@{:p}, @{:p}, @{:p})", journal_handle_ptr, prt, pboot);
            let r: i32 = (*journal_api_ptr).sd_journal_get_monotonic_usec(
                *journal_handle_ptr,
                prt,
                pboot
            );
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_get_monotonic_usec returned {}, {:?}", r, e);
            *api_calls += 1;
            if r < 0 {
                *api_call_errors += 1;
                de_err!("sd_journal_get_monotonic_usec() returned {}; {:?}", r, e);
                let err = Self::Error_from_Errno(r, &e, "sd_journal_get_monotonic_usec", path);
                def1x!("return Err");
                return Result::Err(err);
            } else {
                def1o!("sd_journal_get_monotonic_usec realtime {}", rt);
                monotonic_usec = rt;
            }
        } // end unsafe
        def1x!("return Ok");

        Result::Ok(monotonic_usec)
    }

    /// Wrapper to call `sd_id128_get_boot`.
    /// Returns the `Ok(value)` if the call succeeds.
    /// Returns `Err` if the call fails.
    fn call_sd_id128_get_boot(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
        path: &FPath,
    ) -> Result<sd_id128> {
        def1n!();
        let mut boot_id_id128: sd_id128 = sd_id128 { bytes: [0; 16] };
        unsafe {
            //
            // call sd_id128_get_boot
            //
            let mut rt: std::os::raw::c_ulonglong = 0;
            let prt: *mut std::os::raw::c_ulonglong = &mut rt;
            let pboot: *mut sd_id128 = &mut boot_id_id128 as *mut _ as *mut sd_id128;
            def1o!("sd_id128_get_boot(@{:p}, @{:p}, @{:p})", journal_handle_ptr, prt, pboot);
            let r: i32 = (*journal_api_ptr).sd_id128_get_boot(
                *journal_handle_ptr,
                prt,
                pboot
            );
            let e = Errno::from_raw(r.abs());
            def1o!("sd_journal_get_monotonic_usec returned {}, {:?}", r, e);
            *api_calls += 1;
            if r < 0 {
                *api_call_errors += 1;
                de_err!("sd_journal_get_monotonic_usec() returned {}; {:?}", r, e);
                let err = Self::Error_from_Errno(r, &e, "sd_journal_get_monotonic_usec", path);
                def1x!("return Err");
                return Result::Err(err);
            } else {
                def1o!("sd_journal_get_monotonic_usec realtime {}", rt);
            }
        } // end unsafe
        def1x!();

        Result::Ok(boot_id_id128)
    }

    /// Wrapper to call `sd_journal_get_cursor`.
    /// Returns the `Some(value)` if the call succeeds.
    /// Returns `None` if the call fails.
    fn call_sd_journal_get_cursor(
        journal_handle_ptr: &mut *mut sd_journal,
        journal_api_ptr: &mut JournalApiPtr,
        api_calls: &mut Count,
        api_call_errors: &mut Count,
    ) -> Option<&'a [u8]> {
        def1n!();
        let data: &[u8];
        unsafe {
            let mut cursor: *mut std::os::raw::c_char = mem::zeroed();
            let pcursor: *mut *mut std::os::raw::c_char = &mut cursor;
            def1o!("sd_journal_get_cursor(@{:p}, @{:p})", journal_handle_ptr, pcursor);
            let r: i32 = (*journal_api_ptr).sd_journal_get_cursor(
                *journal_handle_ptr,
                pcursor,
            );
            *api_calls += 1;
            let _e = Errno::from_raw(r.abs());
            def1o!("sd_journal_get_cursor returned {}, {:?}", r, _e);
            if r < 0 {
                *api_call_errors += 1;
                def1x!("return None");
                return None;
            }
            def1o!("sd_journal_get_cursor returned cursor {:?}", cursor);
            data = CStr::from_ptr(cursor).to_bytes();
        } // end unsafe

        Some(data)
    }

    /// wrap calls to `call_sd_id128_get_boot` and passes the optional `boot_id`
    /// to `call_sd_journal_get_monotonic_usec`.
    /// Returns the `Some(value)` if the call succeeds.
    /// Returns `None` if either call fails.
    fn get_monotonic_usec(
        &mut self,
    ) -> Option<MonotonicMicroseconds> {
        def1n!();
        let boot_id_opt = match Self::call_sd_id128_get_boot(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
            &self.path,
        ) {
            Result::Ok(bid) => Some(bid),
            Result::Err(_err) => {
                de_err!("call_sd_id128_get_boot() call failed; {:?}", _err);
                def1x!("return None");
                return None;
            }
        };
        match Self::call_sd_journal_get_monotonic_usec(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
            boot_id_opt,
            &self.path,
        ) {
            Result::Ok(mu) => {
                def1x!("return Some");
                Some(mu)
            }
            Result::Err(_err) => {
                de_err!("call_sd_journal_get_monotonic_usec() call failed; {:?}", _err);
                def1x!("return None");

                None
            }
        }
    }

    /// User-friendly function wrapper.
    /// Checks `DT_USES_SOURCE_OVERRIDE` and dispatches appropriately.
    #[inline(always)]
    pub fn next(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNext {
        def1ñ!("({:?})", rts_filter_before);
        match DT_USES_SOURCE_OVERRIDE {
            None => {
                self.next_fill_buffer(rts_filter_before)
            }
            Some(DtUsesSource::SourceRealtimeTimestamp) => {
                self.next_fill_buffer(rts_filter_before)
            }
            Some(DtUsesSource::RealtimeTimestamp) => {
                // bypass use of the `fill_buffer` entirely
                self.next_dispatch(rts_filter_before)
            }
        }
    }

    /// Call `self.next` until `self.fill_buffer` is full or `next` returns
    /// `Done`.
    ///
    /// Returns `Err` if `self.next_dispatch` returns `Err`.
    ///
    /// This is the only function the touches `self.fill_buffer`,
    /// `self.next_fill_buffer_index`, and `self.next_fill_buffer_loop`.
    fn next_fill_buffer(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNext {
        def1n!("({:?})", rts_filter_before);
        if self.next_fill_buffer_loop {
            loop {
                match self.next_dispatch(rts_filter_before) {
                    ResultNext::Found(je) => {
                        def1o!("next_dispatch returned Found (index {})", self.next_fill_buffer_index);
                        let key = (*je.dt(), self.next_fill_buffer_index);
                        self.fill_buffer.insert(key, je);
                        self.next_fill_buffer_index += 1;
                        if self.fill_buffer.len() >= Self::ENTRY_BUFFER_SZ - 1 {
                            let entry = self.fill_buffer.pop_first().unwrap();
                            def1x!("return Found (index {})", entry.0.1);
                            return ResultNext::Found(entry.1);
                        }
                    }
                    ResultNext::Done => {
                        def1o!("next_dispatch returned Done");
                        break;
                    }
                    ResultNext::ErrIgnore(_err) => {
                        de_err!("next_dispatch returned ErrIgnore; {:?}", _err);
                        def1o!("got ErrIgnore");
                        continue;
                    }
                    ResultNext::Err(err) => {
                        de_err!("next_dispatch returned Err; {:?}", err);
                        def1x!("return Err; {:?}", err);
                        return ResultNext::Err(err);
                    }
                }
            }
        }
        // getting here means the above call `next_dispatch` has returned
        // `Done`. Now just want to exhaust the remainder of `self.fill_buffer`.
        self.next_fill_buffer_loop = false;
        if !self.fill_buffer.is_empty() {
            let entry = self.fill_buffer.pop_first().unwrap();
            def1x!("return Found (index {})", entry.0.1);
            return ResultNext::Found(entry.1);
        }

        def1x!("return Done");
        ResultNext::Done
    }

    /// Call correct `next*` method depending on `JournalOutput` setting.
    #[inline(always)]
    fn next_dispatch(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNext {
        def1ñ!("({:?})", rts_filter_before);
        match self.journal_output {
            JournalOutput::Short => {
                self.next_short(rts_filter_before, DATETIME_FORMAT_SHORT, false)
            }
            JournalOutput::ShortPrecise => {
                self.next_short(rts_filter_before, DATETIME_FORMAT_SHORT_PRECISE, false)
            },
            JournalOutput::ShortIso => {
                self.next_short(rts_filter_before, DATETIME_FORMAT_SHORT_ISO, false)
            },
            JournalOutput::ShortIsoPrecise => {
                self.next_short(rts_filter_before, DATETIME_FORMAT_SHORT_ISO_PRECISE, false)
            },
            JournalOutput::ShortFull => {
                self.next_short(rts_filter_before, DATETIME_FORMAT_SHORT_FULL, false)
            },
            JournalOutput::ShortMonotonic => {
                self.next_short(rts_filter_before, &"", true)
            },
            JournalOutput::ShortUnix => {
                self.next_short(rts_filter_before, DATETIME_FORMAT_SHORT_UNIX, false)
            },
            JournalOutput::Verbose => {
                self.next_verbose(rts_filter_before)
            }
            JournalOutput::Export => {
                self.next_export(rts_filter_before)
            }
            JournalOutput::Cat => {
                self.next_cat(rts_filter_before)
            }
        }
    }

    /// Common calls for all `next*` functions.
    fn next_common(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNextCommon {
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next_common()`");
        def1n!("({:?})", rts_filter_before);

        // get the next entry

        match Self::call_sd_journal_next(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
            &self.path,
        ) {
            ResultFind::Found(_) => {}
            ResultFind::Done => {
                def1x!("return Done");
                return ResultNextCommon::Done;
            }
            ResultFind::Err(err) => {
                def1x!("return Err {:?}", err);
                return ResultNextCommon::Err(err);
            }
        }
        self.events_processed += 1;

        // get the realtime usec (epoch microseconds)

        let realtime_timestamp = match Self::call_sd_journal_get_realtime_usec(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
            &self.path,
        ) {
            Result::Ok(rt) => rt,
            Result::Err(err) => {
                de_err!("failed to get realtime_timestamp; {:?}", err);
                def1x!("return ErrIgnore({:?})", err);
                return ResultNextCommon::ErrIgnore(err);
            }
        };

        // get the optional field `_SOURCE_REALTIME_TIMESTAMP`
        let actual_epoch_usec: EpochMicroseconds;
        let source_realtime_timestamp = self.get_source_realtime_timestamp();
        // first attempt to use `DT_USES_SOURCE_OVERRIDE` if set
        let dt_uses_source: DtUsesSource = match DT_USES_SOURCE_OVERRIDE {
            Some(dt_uses_source) => {
                match dt_uses_source {
                    DtUsesSource::RealtimeTimestamp => {
                        actual_epoch_usec = realtime_timestamp;
                    }
                    DtUsesSource::SourceRealtimeTimestamp => {
                        // but don't panic if `_SOURCE_REALTIME_TIMESTAMP` is
                        // not available.
                        match source_realtime_timestamp {
                            Some(ts) => {
                                actual_epoch_usec = ts;
                            }
                            None => {
                                actual_epoch_usec = realtime_timestamp;
                            }
                        }
                    }
                }
                dt_uses_source
            }
            // if `DT_USES_SOURCE_OVERRIDE` is not set, then fallback to actually
            // analysing which fields were available.
            // prefer `_SOURCE_REALTIME_TIMESTAMP` as `journalctl` does.
            None => match source_realtime_timestamp {
                Some(ts) => {
                    actual_epoch_usec = ts;
                    DtUsesSource::SourceRealtimeTimestamp
                }
                None => {
                    actual_epoch_usec = realtime_timestamp;
                    DtUsesSource::RealtimeTimestamp
                }
            }
        };
        self.em_first_last_update_processed(&actual_epoch_usec);
        match em_after_or_before(&actual_epoch_usec, rts_filter_before) {
            Result_Filter_DateTime1::OccursAtOrAfter => {
                def1x!("OccursAtOrAfter: return Done");
                return ResultNextCommon::Done;
            }
            Result_Filter_DateTime1::Pass
            | Result_Filter_DateTime1::OccursBefore => {}
        }

        unsafe {
            // XXX: I thought `python-systemd` called `sd_journal_restart_data`
            //      but when I reviewed that code I couldn't find such a call.
            (*self.journal_api_ptr).sd_journal_restart_data(self.journal_handle_ptr);
        }
        self.api_calls += 1;
        def1x!("return Found({})", realtime_timestamp);

        ResultNextCommon::Found((realtime_timestamp, source_realtime_timestamp, dt_uses_source))
    }

    /// Journal entry output matching `--output=short` and `short*` variations.
    ///
    /// For example, the default `short` is:
    ///
    /// ```text
    /// $ PAGER= journalctl --output=short --lines=1 --all --utc --file=./user-1000.journal
    /// Apr 01 06:44:32 ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
    /// ```
    ///
    /// The special case of `short-monotonic` is:
    /// ```text
    /// $ PAGER= journalctl --output=short-monotonic --lines=1 --all --utc --file=./user-1000.journal
    /// [   74.212842] ubuntu22Acorn ubuntu-appindicators@ubuntu.com[1306]: unable to update icon for livepatch
    /// ```
    ///
    /// The log message format only varies by the `datetime_format`.
    ///
    /// In psuedo-code, the log message format comprises fields:
    ///     `${DATETIME} ${_HOSTNAME} (${SYSLOG_IDENTIFIER${SYSLOG_PID}?${_COMM}})[${_PID}]: ${MESSAGE}`
    fn next_short(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
        datetime_format: &str,
        is_monotonic: bool,
    ) -> ResultNext {
        def1n!("({:?})", rts_filter_before);
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next_order_enumerate_data()`");

        let mut buffer: Vec<u8> = Vec::with_capacity(Self::BUF_DEFAULT_MEDIUM_SZ);
        let realtime_timestamp: EpochMicroseconds;
        let source_realtime_timestamp: EpochMicrosecondsOpt;
        let dt_uses_source: DtUsesSource;

        match self.next_common(rts_filter_before) {
            ResultNextCommon::Found((rt, srt, dt)) => {
                realtime_timestamp = rt;
                source_realtime_timestamp = srt;
                dt_uses_source = dt;
            }
            ResultNextCommon::Done => {
                def1x!("return Done");
                return ResultNext::Done;
            }
            ResultNextCommon::Err(err) => {
                def1x!("return Err({:?})", err);
                return ResultNext::Err(err);
            }
            ResultNextCommon::ErrIgnore(err) => {
                def1x!("return ErrIgnore({:?})", err);
                return ResultNext::ErrIgnore(err);
            }
        }

        let mut data_hostname: Option<&[u8]> = None;
        let mut data_syslog_identifier: Option<&[u8]> = None;
        let mut data_syslog_pid: Option<&[u8]> = None;
        let mut data_comm: Option<&[u8]> = None;
        let mut data_pid: Option<&[u8]> = None;
        let mut data_message: Option<&[u8]> = None;
        // track which keys have been found, stop calling
        // `sd_journal_enumerate_available_data` when the necessary keys
        // are found
        // XXX: a local hashset or hashmap to track found keys would be
        //      nice-looking but it was found to use too much resources
        //      according to flamegraph;
        //      about 10% of this function was spent modifying the local hashset.
        //      A static hashset or hashmap was too tedious and also used a
        //      measureable amount of resources.
        let mut key_hostname_found: bool = false;
        let mut key_syslog_identifier_found: bool = false;
        let mut key_syslog_pid_found: bool = false;
        let mut key_comm_found: bool = false;
        let mut key_pid_found: bool = false;
        let mut key_message_found: bool = false;

        // instead of just a `loop` use a `while` loop with a emergency counter
        // TODO: [2023/05/07] enumerating over all the data takes much time according to
        //       flamegraph. Instead, just fetch exactly the data needed. There's only 5, maximum 6
        //       queries to do.
        let mut emerg_stop_data_enumerate = 0;
        while emerg_stop_data_enumerate < 200 {
            emerg_stop_data_enumerate += 1;
            let data = match Self::call_sd_journal_enumerate_available_data(
                &mut self.journal_handle_ptr,
                &mut self.journal_api_ptr,
                &mut self.api_calls,
                &mut self.api_call_errors,
                &self.path,
                #[cfg(test)]
                &self.force_error_range_opt,
            ) {
                ResultFind::Found(d) => d,
                ResultFind::Err(_err) => {
                    de_err!("failed to get data (continue); {:?}", _err);
                    continue;
                }
                ResultFind::Done => {
                    break;
                }
            };
            let keyn: usize;
            let key = match data.find_byte(FIELD_MID_U8) {
                Some(pos) => {
                    keyn = pos + 1;
                    &data[..keyn - 1]
                }
                None => {
                    de_err!("sd_journal_enumerate_available_data() call returned invalid data; no `=` found (continue)");
                    continue;
                }
            };
            def1o!("key {:?}", key.as_bstr());
            match key {
                // these match cases must be the same values as those
                // used to initialize `self.entries_next_short`
                KEY_HOSTNAME_BYTES => {
                    key_hostname_found = true;
                    data_hostname = Some(&data[keyn..]);
                }
                KEY_SYSLOG_IDENTIFIER_BYTES => {
                    key_syslog_identifier_found = true;
                    data_syslog_identifier = Some(&data[keyn..]);
                }
                KEY_SYSLOG_PID_BYTES => {
                    key_syslog_pid_found = true;
                    data_syslog_pid = Some(&data[keyn..]);
                }
                KEY_COMM_BYTES => {
                    key_comm_found = true;
                    data_comm = Some(&data[keyn..]);
                }
                KEY_PID_BYTES => {
                    key_pid_found = true;
                    data_pid = Some(&data[keyn..]);
                }
                KEY_MESSAGE_BYTES => {
                    key_message_found = true;
                    data_message = Some(&data[keyn..]);
                }
                _ => {}
            }
            match (
                key_hostname_found,
                key_syslog_identifier_found,
                key_syslog_pid_found,
                key_comm_found,
                key_pid_found,
                key_message_found,
            ) {
                (true, true, true, _, true, true) => {
                    // if all the keys are found then break out of the loop
                    // if all the keys except `_COMM` are found
                    // then break out of the loop (key `_COMM` is only used if
                    // `SYSLOG_IDENTIFIER` is not available)
                    def1o!("all needed keys were found; break");
                    break;
                }
                _ => {}
            }
        } // end while

        // field 1 Datetime or Monotonic time since boot
        let dt_a: usize;
        let dt_b: usize;
        let dt = realtime_or_source_realtime_timestamp_to_datetimel(
            &self.fixed_offset,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        if !is_monotonic {
            def1o!("write field 1 Datetime");
            debug_assert_ne!(datetime_format, "", "datetime_format must not be empty");
            let dts: String = dt.format(datetime_format).to_string();
            let dtsb: &[u8] = dts.as_str().as_bytes();
            buffer.push_str(dtsb);
            dt_a = 0;
            dt_b = dtsb.len();
        } else {
            def1o!("write field 1 monotonic_usec");
            debug_assert_eq!(datetime_format, "", "datetime_format must be empty for `short-monotonic`");
            match self.get_monotonic_usec() {
                Some(mu) => {
                    // [   74.212842]
                    buffer.push(b'[');
                    let mud = mu as f64 / 1000000.0;
                    buffer.push_str(format!("{:>12.6}", mud));
                    dt_a = match buffer.find_byteset(b"0123456789") {
                        Some(pos) => pos,
                        None => 1
                    };
                    dt_b = buffer.len();
                    buffer.push(b']');
                },
                None => {
                    de_err!("get_monotonic_usec() returned None");
                    // XXX: not sure if this is the best thing to do, maybe return an error?
                    buffer.push_str("[            ]");
                    dt_a = 0;
                    dt_b = 0;
                }
            }
        }
        debug_assert_le!(dt_a, dt_b, "bad datetime indexes");

        // field 2 `_HOSTNAME`
        if let Some(data) = data_hostname {
            def1o!("write field 2 _HOSTNAME");
            buffer.push(b' ');
            buffer.push_str(data);
        }

        // field 3 `SYSLOG_IDENTIFIER` or `_COMM`
        match data_syslog_identifier {
            Some(data) => {
                def1o!("write field 3 SYSLOG_IDENTIFIER");
                buffer.push(b' ');
                buffer.push_str(data);
            }
            None => {
                if let Some(data) = data_comm {
                    def1o!("write field 3 _COMM");
                    buffer.push(b' ');
                    buffer.push_str(data);
                }
            }
        }

        // field 4 prefer `_PID`
        match data_pid {
            Some(data) => {
                def1o!("write field 4 _PID");
                buffer.push(b'[');
                buffer.push_str(data);
                buffer.push(b']');
            }
            None => {
                // field 4 `SYSLOG_PID` if no `_PID`
                if let Some(data) = data_syslog_pid {
                    def1o!("write field 4 SYSLOG_PID");
                    buffer.push(b'[');
                    buffer.push_str(data);
                    buffer.push(b']');
                }
            }
        }

        // field 5 `MESSAGE`
        if let Some(data) = data_message {
            def1o!("write field 5 MESSAGE");
            buffer.push_str(": ");
            buffer.push_str(data);
        }

        // end of log line
        def1o!("write ENTRY_END_U8");
        buffer.push(ENTRY_END_U8);

        self.em_first_last_update_accepted_all(
            dt_uses_source,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        self.events_accepted += 1;
        def1x!();

        ResultNext::Found(
            JournalEntry::new_with_date(
                buffer,
                realtime_timestamp,
                source_realtime_timestamp,
                dt,
                dt_uses_source,
                dt_a,
                dt_b,
            )
        )
    }

    /// Journal entry output matching `--output=export`.
    ///
    /// Saves data directly to a `Vec<u8>` buffer; no intermediary or
    /// `str` conversions done at this rust layer.
    ///
    /// ```text
    /// $ PAGER= journalctl --lines=1 --output=export --all --utc --file=./user-1000.journal
    /// __CURSOR=s=e992f143877046059b264a0f907056b6;i=6ff;b=26d74a46deff4872be6d4ca6e885a198;m=46c65ea;t=5f840a88a4b39;x=e7933c3b47482d45
    /// __REALTIME_TIMESTAMP=1680331472784185
    /// __MONOTONIC_TIMESTAMP=74212842
    /// _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
    /// _TRANSPORT=journal
    /// _UID=1000
    /// _GID=1000
    /// _CAP_EFFECTIVE=0
    /// _SELINUX_CONTEXT
    /// 
    /// unconfined
    /// 
    /// _AUDIT_SESSION=2
    /// _AUDIT_LOGINUID=1000
    /// _SYSTEMD_OWNER_UID=1000
    /// _SYSTEMD_UNIT=user@1000.service
    /// _SYSTEMD_SLICE=user-1000.slice
    /// _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
    /// _HOSTNAME=ubuntu22Acorn
    /// PRIORITY=4
    /// _SYSTEMD_USER_SLICE=session.slice
    /// _PID=1306
    /// _COMM=gnome-shell
    /// _EXE=/usr/bin/gnome-shell
    /// _CMDLINE=/usr/bin/gnome-shell
    /// _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@wayland.service
    /// _SYSTEMD_USER_UNIT=org.gnome.Shell@wayland.service
    /// _SYSTEMD_INVOCATION_ID=b7d368c96091463aa538006b518785f4
    /// GLIB_DOMAIN=Ubuntu AppIndicators
    /// SYSLOG_IDENTIFIER=ubuntu-appindicators@ubuntu.com
    /// CODE_FILE=/usr/share/gnome-shell/extensions/ubuntu-appindicators@ubuntu.com/appIndicator.js
    /// CODE_LINE=738
    /// CODE_FUNC=_setGicon
    /// MESSAGE=unable to update icon for livepatch
    /// _SOURCE_REALTIME_TIMESTAMP=1680331472788150
    /// ```
    fn next_export(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNext {
        def1n!("({:?})", rts_filter_before);
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next_export()`");

        let mut buffer: Vec<u8> = Vec::with_capacity(Self::BUF_DEFAULT_LARGE_SZ);
        let realtime_timestamp: EpochMicroseconds;
        let source_realtime_timestamp: EpochMicrosecondsOpt;
        let dt_uses_source: DtUsesSource;

        match self.next_common(rts_filter_before) {
            ResultNextCommon::Found((rt, srt, dt)) => {
                realtime_timestamp = rt;
                source_realtime_timestamp = srt;
                dt_uses_source = dt;
            }
            ResultNextCommon::Done => {
                def1x!("return Done");
                return ResultNext::Done;
            }
            ResultNextCommon::Err(err) => {
                def1x!("return Err({:?})", err);
                return ResultNext::Err(err);
            }
            ResultNextCommon::ErrIgnore(err) => {
                def1x!("return ErrIgnore({:?})", err);
                return ResultNext::ErrIgnore(err);
            }
        }

        // line 1 `__CURSOR`

        match Self::call_sd_journal_get_cursor(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
        ) {
            Some(cursor) => {
                buffer.push_str(KEY__CURSOR);
                buffer.push(FIELD_MID_U8);
                buffer.push_str(cursor);
                buffer.push(FIELD_END_U8);
            }
            None => de_err!("failed to write cursor to buffer"),
        }

        // line 2 `__REALTIME_TIMESTAMP`

        buffer.push_str(KEY__REALTIME_TIMESTAMP);
        buffer.push(FIELD_MID_U8);
        // TODO: cost-savings: use crate `numtoa` to convert number to readable bytes
        buffer.push_str(&realtime_timestamp.to_string());
        buffer.push(FIELD_END_U8);

        // line 3 `__MONOTONIC_TIMESTAMP`

        match self.get_monotonic_usec() {
            Some(m) => {
                buffer.push_str(KEY__MONOTONIC_TIMESTAMP_BYTES);
                buffer.push(FIELD_MID_U8);
                // TODO: cost-savings: use crate `numtoa` to convert number to readable bytes
                buffer.push_str(&m.to_string());
                buffer.push(FIELD_END_U8);
            }
            None => {
                de_err!("get_monotonic_usec() failed; cannot write monotonic timestamp to buffer");
            }
        };

        // remaining lines

        // instead of just a `loop` use a `while` loop with a emergency counter
        let mut emerg_stop_data_enumerate = 0;
        while emerg_stop_data_enumerate < 200 {
            emerg_stop_data_enumerate += 1;
            let data = match Self::call_sd_journal_enumerate_available_data(
                &mut self.journal_handle_ptr,
                &mut self.journal_api_ptr,
                &mut self.api_calls,
                &mut self.api_call_errors,
                &self.path,
                #[cfg(test)]
                &self.force_error_range_opt,
            ) {
                ResultFind::Found(d) => d,
                ResultFind::Done => break,
                ResultFind::Err(_err) => {
                    de_err!("call_sd_journal_enumerate_available_data() failed (continue): {:?}", _err);
                    continue;
                }
            };
            buffer.push_str(data);
            buffer.push(FIELD_END_U8);
        } // end while

        buffer.push(ENTRY_END_U8);

        self.em_first_last_update_accepted_all(
            dt_uses_source,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        self.events_accepted += 1;
        def1x!();

        ResultNext::Found(
            JournalEntry::from_vec_nodt(
                buffer,
                realtime_timestamp,
                source_realtime_timestamp,
                dt_uses_source,
                &self.fixed_offset,
            )
        )
    }

    /// Field order as seen in `journalctl --ouput=verbose`; nearly that
    /// order. It's a rough approximation of the order most commonly seen.
    /// Noticeably, just taking the fields in order of
    /// `sd_journal_enumerate_data()` is not anywhere near what
    /// `journalctl` does. Yet, `journalctl` does not seem to
    /// have a perfectly consistent ordering itself. This ordering may not
    /// perfectly match `journalctl` but is good enough.
    ///
    /// To see all fields in `file.journal`, run Linux command:
    ///
    /// ```sh
    /// PAGER= journalctl --output=verbose --all --file=file.journal \
    ///     | grep -oEe '^    [[:alpha:]_]+\=' \
    ///     | tr -d '=' | sort | uniq | sed -E 's/^[ ]*//'
    /// ```
    const FIELD_ORDER_VERBOSE: [&'static [u8]; 102] = [
        b"_TRANSPORT",
        b"_UID",
        b"_GID",
        b"_FSUID",
        b"_CAP_EFFECTIVE",
        b"_SELINUX_CONTEXT",
        b"_AUDIT_FIELD_APPARMOR",
        b"_AUDIT_FIELD_ARCH",
        b"_AUDIT_FIELD_CAPABILITY",
        b"_AUDIT_FIELD_CAPNAME",
        b"_AUDIT_FIELD_CLASS",
        b"_AUDIT_FIELD_CODE",
        b"_AUDIT_FIELD_COMPAT",
        b"_AUDIT_FIELD_DENIED_MASK",
        b"_AUDIT_FIELD_INFO",
        b"_AUDIT_FIELD_IP",
        b"_AUDIT_FIELD_NAME",
        b"_AUDIT_FIELD_OPERATION",
        b"_AUDIT_FIELD_OUID",
        b"_AUDIT_FIELD_PROFILE",
        b"_AUDIT_FIELD_REQUESTED_MASK",
        b"_AUDIT_FIELD_SIG",
        b"_AUDIT_FIELD_SYSCALL",
        b"_AUDIT_ID",
        b"_AUDIT_LOGINUID",
        b"_AUDIT_SESSION",
        b"_AUDIT_TYPE",
        b"_AUDIT_TYPE_NAME",
        b"_BOOT_ID",
        b"_MACHINE_ID",
        b"_HOSTNAME",
        b"PRIORITY",
        b"_PID",
        b"TID",
        b"_COMM",
        b"_EXE",
        b"_CMDLINE",
        b"_SYSTEMD_CGROUP",
        b"_SYSTEMD_OWNER_UID",
        b"_SYSTEMD_UNIT",
        b"_SYSTEMD_USER_UNIT",
        b"_SYSTEMD_SLICE",
        b"_SYSTEMD_USER_SLICE",
        b"_SYSTEMD_INVOCATION_ID",
        b"_STREAM_ID",
        b"_KERNEL_SUBSYSTEM",
        b"_KERNEL_DEVICE",
        b"_UDEV_SYSNAME",
        b"GLIB_DOMAIN",
        b"GLIB_OLD_LOG_API",
        b"GNOME_SHELL_EXTENSION_NAME",
        b"GNOME_SHELL_EXTENSION_UUID",
        b"THREAD_ID",
        b"CODE_FILE",
        b"CODE_LINE",
        b"CODE_FUNC",
        b"INVOCATION_ID",
        b"SESSION_ID",
        b"USER_ID",
        b"LEADER",
        b"UNIT",
        b"UNIT_RESULT",
        b"JOB_ID",
        b"JOB_TYPE",
        b"JOB_RESULT",
        b"N_RESTARTS",
        b"PULSE_BACKTRACE",
        b"TIMESTAMP_MONOTONIC",
        b"TIMESTAMP_BOOTTIME",
        b"KERNEL_USEC",
        b"USERSPACE_USEC",
        b"CPU_USAGE_NSEC",
        b"MESSAGE_ID",
        b"SEAT_ID",
        b"MESSAGE",
        b"SHUTDOWN",
        b"JOURNAL_NAME",
        b"JOURNAL_PATH",
        b"CURRENT_USE",
        b"CURRENT_USE_PRETTY",
        b"MAX_USE",
        b"MAX_USE_PRETTY",
        b"DISK_KEEP_FREE",
        b"DISK_KEEP_FREE_PRETTY",
        b"DISK_AVAILABLE",
        b"DISK_AVAILABLE_PRETTY",
        b"LIMIT",
        b"LIMIT_PRETTY",
        b"AVAILABLE",
        b"AVAILABLE_PRETTY",
        b"EXIT_CODE",
        b"EXIT_STATUS",
        b"COMMAND",
        b"SYSLOG_FACILITY",
        b"SYSLOG_IDENTIFIER",
        b"SYSLOG_PID",
        b"SYSLOG_RAW",
        b"SYSLOG_TIMESTAMP",
        b"NM_DEVICE",
        b"NM_LOG_DOMAINS",
        b"NM_LOG_LEVEL",
        KEY__MONOTONIC_TIMESTAMP_BYTES,
    ];

    /// Journal entry output matching `--output=verbose`.
    ///
    /// Follows approximately the same field ordering as
    /// `journalctl --output=verbose` (`journalctl` field ordering is somewhat
    /// arbitrary and inconsistent).
    ///
    /// ```text
    /// $ PAGER= journalctl --lines=1 --output=verbose --all --utc --file=./user-1000.journal
    /// Sat 2023-04-01 06:44:32.788150 UTC [s=e992f143877046059b264a0f907056b6;i=6ff;b=26d74a46deff4872be6d4ca6e885a198;m=46c65ea;t=5f840a88a4b39;x=e7933c3b47482d45]
    ///     _TRANSPORT=journal
    ///     _UID=1000
    ///     _GID=1000
    ///     _CAP_EFFECTIVE=0
    ///     _SELINUX_CONTEXT=unconfined
    ///     _AUDIT_SESSION=2
    ///     _AUDIT_LOGINUID=1000
    ///     _SYSTEMD_OWNER_UID=1000
    ///     _SYSTEMD_UNIT=user@1000.service
    ///     _SYSTEMD_SLICE=user-1000.slice
    ///     _BOOT_ID=26d74a46deff4872be6d4ca6e885a198
    ///     _MACHINE_ID=9dd5669d37b84d03a7987b2a1a47ccbb
    ///     _HOSTNAME=ubuntu22Acorn
    ///     PRIORITY=4
    ///     _SYSTEMD_USER_SLICE=session.slice
    ///     _PID=1306
    ///     _COMM=gnome-shell
    ///     _EXE=/usr/bin/gnome-shell
    ///     _CMDLINE=/usr/bin/gnome-shell
    ///     _SYSTEMD_CGROUP=/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@wayland.service
    ///     _SYSTEMD_USER_UNIT=org.gnome.Shell@wayland.service
    ///     _SYSTEMD_INVOCATION_ID=b7d368c96091463aa538006b518785f4
    ///     GLIB_DOMAIN=Ubuntu AppIndicators
    ///     SYSLOG_IDENTIFIER=ubuntu-appindicators@ubuntu.com
    ///     CODE_FILE=/usr/share/gnome-shell/extensions/ubuntu-appindicators@ubuntu.com/appIndicator.js
    ///     CODE_LINE=738
    ///     CODE_FUNC=_setGicon
    ///     MESSAGE=unable to update icon for livepatch
    ///     _SOURCE_REALTIME_TIMESTAMP=1680331472788150
    /// ```
    fn next_verbose(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNext {
        def1n!("({:?})", rts_filter_before);
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next_verbose()`");

        let mut fields: HashMap<&[u8], &[u8]> = HashMap::new();
        let realtime_timestamp: EpochMicroseconds;
        let source_realtime_timestamp: EpochMicrosecondsOpt;
        let dt_uses_source: DtUsesSource;

        match self.next_common(rts_filter_before) {
            ResultNextCommon::Found((rt, srt, dt)) => {
                realtime_timestamp = rt;
                source_realtime_timestamp = srt;
                dt_uses_source = dt;
            }
            ResultNextCommon::Done => {
                def1x!("return Done");
                return ResultNext::Done;
            }
            ResultNextCommon::Err(err) => {
                def1x!("return Err({:?})", err);
                return ResultNext::Err(err);
            }
            ResultNextCommon::ErrIgnore(err) => {
                def1x!("return ErrIgnore({:?})", err);
                return ResultNext::ErrIgnore(err);
            }
        }

        // instead of just a `loop` use a `while` loop with a emergency counter
        let mut emerg_stop_data_enumerate = 0;
        while emerg_stop_data_enumerate < 200 {
            emerg_stop_data_enumerate += 1;
            let data = match Self::call_sd_journal_enumerate_available_data(
                &mut self.journal_handle_ptr,
                &mut self.journal_api_ptr,
                &mut self.api_calls,
                &mut self.api_call_errors,
                &self.path,
                #[cfg(test)]
                &self.force_error_range_opt,
            ) {
                ResultFind::Found(d) => d,
                ResultFind::Err(err) => {
                    def1x!("return {:?}", err);
                    return ResultNext::Err(err);
                }
                ResultFind::Done => {
                    def1o!("returned Done");
                    break;
                }
            };
            let mid: usize = data.find_byte(FIELD_MID_U8).unwrap_or(data.len());
            let key = &data[..mid];
            let valb = std::cmp::min(mid + 1, data.len());
            let mut value = &data[valb..];
            // trim end of `_SELINUX_CONTEXT`; it often has trailing cruft bytes.
            // The cruft bytes are not printed by `journalctl --output=verbose`
            // but are printed in `journalctl --output=export`.
            if key == KEY_SELINUX_CONTEXT_BYTES {
                while value.ends_with(b"\0")
                    || value.ends_with(b"\r")
                    || value.ends_with(b"\n")
                    || value.ends_with(b" ")
                {
                    value = &value[..value.len() - 1];
                }
            }
            fields.insert(key, value);
        } // end while

        // `python-systemd` stores value `get_monotonic_usec` in field "__MONOTONIC_TIMESTAMP"
        // https://github.com/systemd/python-systemd/blob/v235/systemd/journal.py#L257
        // if that field is not already then add it.
        // XXX: declare `monotonic_usec_str` outside this `if` block so that it lives
        //      long enough for use in `fields`
        let monotonic_usec_str: String;
        if !fields.contains_key(&*KEY__MONOTONIC_TIMESTAMP_BYTES) {
            match self.get_monotonic_usec() {
                Some(m) => {
                    monotonic_usec_str = m.to_string();
                    fields.insert(&*KEY__MONOTONIC_TIMESTAMP_BYTES, monotonic_usec_str.as_bytes());
                },
                None => de_err!("get_monotonic_usec() failed"),
            };
        }
        self.em_first_last_update_accepted_all(
            dt_uses_source,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        self.events_accepted += 1;

        // write it all to one buffer
        let mut buffer: Vec<u8> = Vec::with_capacity(Self::BUF_DEFAULT_LARGE_SZ);

        // line 1 Datetime and Cursor

        // field 1 Datetime
        let dt = realtime_or_source_realtime_timestamp_to_datetimel(
            &self.fixed_offset,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        let dts: String = dt.format(DATETIME_FORMAT_VERBOSE).to_string();
        let dtsb: &[u8] = dts.as_str().as_bytes();
        def1o!("field 1 datetime");
        buffer.push_str(dtsb);
        let dt_a: usize = 0;
        let dt_b: usize = dtsb.len();
        debug_assert_le!(dt_a, dt_b, "bad datetime indexes");
        buffer.push(b' ');

        // field 2 Cursor
        match Self::call_sd_journal_get_cursor(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
        ) {
            Some(cursor) => {
                def1o!("field 2 Cursor");
                buffer.push(b'[');
                buffer.push_str(cursor);
                buffer.push(b']');
            }
            None => {
                de_err!("failed to write cursor to buffer");
            }
        }
        buffer.push(FIELD_END_U8);

        // set aside `_SOURCE_REALTIME_TIMESTAMP`
        let source_realtime_timestamp_field: Option<&[u8]> = match fields.remove(&*KEY_SOURCE_REALTIME_TIMESTAMP_BYTES) {
            Some(s) => {
                Some(s)
            }
            None => {
                None
            }
        };

        // write the remaining lines in order of `FIELD_ORDER_VERBOSE`
        for field in &Self::FIELD_ORDER_VERBOSE {
            match fields.remove(field) {
                Some(value) => {
                    def1o!("field from fields {:?}", field.as_bstr());
                    buffer.push_str(FIELD_BEG_VERBOSE);
                    buffer.push_str(field);
                    buffer.push(FIELD_MID_U8);
                    buffer.push_str(value);
                    buffer.push(FIELD_END_U8);
                }
                None => {}
            }
        }

        // write any remaining lines in a sorted order
        // (do not use random order of `HashMap.into_iter()`)
        for (field, value) in fields.into_iter().sorted() {
            buffer.push_str(FIELD_BEG_VERBOSE);
            def1o!("field remaining {:?}", field.as_bstr());
            buffer.push_str(field);
            buffer.push(FIELD_MID_U8);
            buffer.push_str(value);
            buffer.push(FIELD_END_U8);
        }

        // write `_SOURCE_REALTIME_TIMESTAMP` last
        match source_realtime_timestamp_field {
            Some(s) => {
                def1o!("field (second-to-last) {:?}", KEY_SOURCE_REALTIME_TIMESTAMP);
                buffer.push_str(FIELD_BEG_VERBOSE);
                buffer.push_str(KEY_SOURCE_REALTIME_TIMESTAMP);
                buffer.push(FIELD_MID_U8);
                buffer.push_str(s);
                buffer.push(FIELD_END_U8);
            }
            None => {
                de_err!("failed to write _SOURCE_REALTIME_TIMESTAMP to buffer");
            }
        }
        def1x!();

        ResultNext::Found(
            JournalEntry::from_vec(
                buffer,
                realtime_timestamp,
                source_realtime_timestamp,
                dt,
                dt_uses_source,
                dt_a,
                dt_b,
            )
        )
    }

    /// Journal entry output matching `--output=cat`.
    ///
    /// For example:
    ///
    /// ```text
    /// $ PAGER= journalctl --output=cat --lines=1 --all --utc --file=./user-1000.journal
    /// unable to update icon for livepatch
    /// ```
    fn next_cat(
        &mut self,
        rts_filter_before: &EpochMicrosecondsOpt,
    ) -> ResultNext {
        def1n!("({:?})", rts_filter_before);
        debug_assert!(self.analyzed, "must call `analyze()` before calling `next_cat()`");

        let mut buffer: Vec<u8> = Vec::with_capacity(Self::BUF_DEFAULT_SMALL_SZ);
        let realtime_timestamp: EpochMicroseconds;
        let source_realtime_timestamp: EpochMicrosecondsOpt;
        let dt_uses_source: DtUsesSource;

        match self.next_common(rts_filter_before) {
            ResultNextCommon::Found((rt, srt, dt)) => {
                realtime_timestamp = rt;
                source_realtime_timestamp = srt;
                dt_uses_source = dt;
            }
            ResultNextCommon::Done => {
                def1x!("return Done");
                return ResultNext::Done;
            }
            ResultNextCommon::Err(err) => {
                def1x!("return Err({:?})", err);
                return ResultNext::Err(err);
            }
            ResultNextCommon::ErrIgnore(err) => {
                def1x!("return ErrIgnore({:?})", err);
                return ResultNext::ErrIgnore(err);
            }
        }

        let dt = realtime_or_source_realtime_timestamp_to_datetimel(
            &self.fixed_offset,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );

        let data = match Self::call_sd_journal_get_data(
            &mut self.journal_handle_ptr,
            &mut self.journal_api_ptr,
            &mut self.api_calls,
            &mut self.api_call_errors,
            &*KEY_MESSAGE_CSTR,
            &self.path,
            #[cfg(test)]
            &self.force_error_range_opt,
        ) {
            Result::Ok(d) => d,
            Result::Err(err) => {
                de_err!("sd_journal_get_data() call failed; {:?}", err);
                def1x!("return {:?}", err);
                // skip this journal entry but instruct caller to continue
                return ResultNext::ErrIgnore(err);
            }
        };
        let mut value: &[u8] = data;
        match data.find_byte(FIELD_MID_U8) {
            Some(pos) => value = &data[pos + 1..],
            None => {
                de_err!("sd_journal_get_data() call returned invalid data; no `=` found");
                // continue and copy `value` to `buffer` anyway
            }
        }

        // field 1 `MESSAGE`
        buffer.push_str(value);

        // end of log line
        buffer.push(ENTRY_END_U8);

        self.em_first_last_update_accepted_all(
            dt_uses_source,
            &realtime_timestamp,
            &source_realtime_timestamp,
        );
        self.events_accepted += 1;
        def1x!();

        // `journalctl` format `cat` does not have a datetime substring
        // so pass `0, 0` for `dt_a, dt_b`.
        ResultNext::Found(
            JournalEntry::new_with_date(
                buffer,
                realtime_timestamp,
                source_realtime_timestamp,
                dt,
                dt_uses_source,
                0,
                0,
            )
        )
    }

    /// `Count` of `JournalEntry`s processed (or attempted to) by
    /// this `JournalReader` instance.
    #[inline(always)]
    pub fn count_events_processed(&self) -> Count {
        self.events_processed
    }

    /// `Count` of `JournalEntry`s processed and accepted.
    #[inline(always)]
    pub fn count_events_accepted(&self) -> Count {
        self.events_accepted
    }

    #[inline(always)]
    pub const fn path(&self) -> &FPath {
        &self.path
    }

    #[inline(always)]
    pub const fn filetype(&self) -> FileType {
        FileType::Journal { archival_type: FileTypeArchive::Normal }
    }

    /// File size in bytes
    #[inline(always)]
    pub const fn filesz(&self) -> FileSz {
        self.filesz
    }

    /// Update the two statistic `EpochMicroseconds` of
    /// `self.ts_first_accepted` and `self.ts_last_accepted`.
    ///
    /// Similar to [`dt_first_last_update`].
    ///
    /// [`dt_first_last_update`]: crate::readers::syslinereader::SyslineReader#method.dt_first_last_update
    fn em_first_last_update_accepted(
        &mut self,
        em: &EpochMicroseconds,
    ) {
        defñ!("({:?})", em);
        // TODO: the `ts_first` and `ts_last` are only for `--summary`,
        //       no need to always copy datetimes.
        //       Would be good to only run this when `if self.do_summary {...}`
        match self.ts_first_accepted {
            Some(first) => {
                if &first > em {
                    self.ts_first_accepted = Some(*em);
                }
            }
            None => {
                self.ts_first_accepted = Some(*em);
            }
        }
        match self.ts_last_accepted {
            Some(last) => {
                if &last < em {
                    defo!("ts_last_accepted set {:?}", em);
                    self.ts_last_accepted = Some(*em);
                }
            }
            None => {
                defo!("ts_last_accepted set {:?}", em);
                self.ts_last_accepted = Some(*em);
            }
        }
    }

    fn em_first_last_update_accepted_all(
        &mut self,
        mut dt_uses_source: DtUsesSource,
        realtime_timestamp: &EpochMicroseconds,
        source_realtime_timestamp: &EpochMicrosecondsOpt,
    ) {
        if let Some(dt) = DT_USES_SOURCE_OVERRIDE {
            dt_uses_source = dt;
        }
        match dt_uses_source {
            DtUsesSource::RealtimeTimestamp => {
                self.em_first_last_update_accepted(&realtime_timestamp);
            }
            DtUsesSource::SourceRealtimeTimestamp => match source_realtime_timestamp {
                Some(em) => {
                    self.em_first_last_update_accepted(em);
                }
                None => {
                    self.em_first_last_update_accepted(&realtime_timestamp);
                }
            },
        }
    }

    /// Update the two statistic `EpochMicroseconds` of
    /// `self.ts_first_processed` and `self.ts_last_processed`.
    ///
    /// Similar to [`dt_first_last_update`].
    ///
    /// [`dt_first_last_update`]: crate::readers::syslinereader::SyslineReader#method.dt_first_last_update
    // TODO: [2024/04] track log messages not in chronological order, similar to
    //       tracking done in `FixedStructReader::entries_out_of_order` and
    //       `EvtxReader::out_of_order`
    //       Print the result in the `--summary` output.
    fn em_first_last_update_processed(
        &mut self,
        em: &EpochMicroseconds,
    ) {
        defñ!("({:?})", em);
        // TODO: the `em_first` and `em_last` are only for `--summary`,
        //       no need to always copy datetimes.
        //       Would be good to only run this when `if self.do_summary {...}`
        match self.ts_first_processed {
            Some(first) => {
                if &first > em {
                    self.out_of_order += 1;
                    self.ts_first_processed = Some(*em);
                }
            }
            None => {
                self.ts_first_processed = Some(*em);
            }
        }
        match self.ts_last_processed {
            Some(last) => {
                if &last < em {
                    self.ts_last_processed = Some(*em);
                } else if &last > em {
                    self.out_of_order += 1;
                }
            }
            None => {
                self.ts_last_processed = Some(*em);
            }
        }
    }

    /// Return the `DateTimeLOpt` of the first [Journal entry] accepted by the
    /// datetime filters.
    ///
    /// [Journal entry]: crate::data::journal::JournalEntry
    pub fn dt_first_accepted(&self) -> DateTimeLOpt {
        match self.ts_first_accepted {
            EpochMicrosecondsOpt::None => DateTimeLOpt::None,
            EpochMicrosecondsOpt::Some(ts) => {
                DateTimeLOpt::Some(realtime_timestamp_to_datetimel(&self.fixed_offset, &ts))
            }
        }
    }

    /// Return the `DateTimeLOpt` of the last [Journal entry] accepted by the
    /// datetime filters.
    ///
    /// [Journal entry]: crate::data::journal::JournalEntry
    pub fn dt_last_accepted(&self) -> DateTimeLOpt {
        match self.ts_last_accepted {
            EpochMicrosecondsOpt::None => DateTimeLOpt::None,
            EpochMicrosecondsOpt::Some(ts) => {
                DateTimeLOpt::Some(realtime_timestamp_to_datetimel(&self.fixed_offset, &ts))
            }
        }
    }

    /// Return the `DateTimeLOpt` of the first [Journal Entry] processed.
    ///
    /// [Journal entry]: crate::data::journal::JournalEntry
    pub fn dt_first_processed(&self) -> DateTimeLOpt {
        match self.ts_first_processed {
            EpochMicrosecondsOpt::None => DateTimeLOpt::None,
            EpochMicrosecondsOpt::Some(ts) => {
                DateTimeLOpt::Some(realtime_timestamp_to_datetimel(&self.fixed_offset, &ts))
            }
        }
    }

    /// Return the `DateTimeLOpt` of the last [Journal Entry] processed.
    ///
    /// [Journal entry]: crate::data::journal::JournalEntry
    pub fn dt_last_processed(&self) -> DateTimeLOpt {
        match self.ts_last_processed {
            EpochMicrosecondsOpt::None => DateTimeLOpt::None,
            EpochMicrosecondsOpt::Some(ts) => {
                DateTimeLOpt::Some(realtime_timestamp_to_datetimel(&self.fixed_offset, &ts))
            }
        }
    }

    /// Return an up-to-date `SummaryJournalReader` instance for this
    /// `JournalReader`.
    #[allow(non_snake_case)]
    pub fn summary(&self) -> SummaryJournalReader {
        let journalreader_events_processed: Count = self.count_events_processed();
        let journalreader_events_accepted: Count = self.count_events_accepted();
        let journalreader_datetime_first_accepted = self.dt_first_accepted();
        let journalreader_datetime_last_accepted = self.dt_last_accepted();
        let journalreader_datetime_first_processed = self.dt_first_processed();
        let journalreader_datetime_last_processed = self.dt_last_processed();
        let journalreader_filesz = self.filesz();
        let journalreader_api_calls: Count = self.api_calls;
        let journalreader_api_call_errors: Count = self.api_call_errors;
        let journalreader_out_of_order: Count = self.out_of_order;

        SummaryJournalReader {
            journalreader_events_processed,
            journalreader_events_accepted,
            journalreader_datetime_first_accepted,
            journalreader_datetime_last_accepted,
            journalreader_datetime_first_processed,
            journalreader_datetime_last_processed,
            journalreader_filesz,
            journalreader_api_calls,
            journalreader_api_call_errors,
            journalreader_out_of_order,
        }
    }

    /// Return an up-to-date [`Summary`] instance for this `JournalReader`.
    ///
    /// [`Summary`]: crate::readers::summary::Summary
    pub fn summary_complete(&self) -> Summary {
        let path = self.path().clone();
        let path_ntf: Option<FPath> = match &self.named_temp_file {
            Some(ntf) => Some(path_to_fpath(ntf.path())),
            None => None,
        };
        let filetype = self.filetype();
        let logmessagetype = filetype.to_logmessagetype();
        let summaryjournalreader = self.summary();
        let error: Option<String> = self.error.clone();

        Summary::new(
            path,
            path_ntf,
            filetype,
            logmessagetype,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(summaryjournalreader),
            error,
        )
    }
}
