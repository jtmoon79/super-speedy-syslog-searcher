// src/data/fixedstruct.rs

//! Implement [`FixedStruct`] to represent Unix record-keeping C structs.
//! This includes [`acct`], [`lastlog`], [`lastlogx`],
//! [`utmp`], and [`utmpx`] formats for various operating systems and
//! architectures.
//!
//! [`acct`]: https://www.man7.org/linux/man-pages/man5/acct.5.html
//! [`lastlog`]: https://man7.org/linux/man-pages/man8/lastlog.8.html
//! [`lastlogx`]: https://man.netbsd.org/lastlogx.5
//! [`utmp`]: https://www.man7.org/linux/man-pages/man5/utmp.5.html
//! [`utmpx`]: https://man.netbsd.org/utmpx.5

#![allow(clippy::tabs_in_doc_comments)]

#[doc(hidden)]
use crate::{de_err, de_wrn, debug_panic};
#[doc(hidden)]
use crate::common::{
    FileOffset,
    FileSz,
    FileTypeFixedStruct,
    max16,
    min16,
};
use crate::data::datetime::{
    DateTimeL,
    FixedOffset,
    LocalResult,
    TimeZone,
};
use crate::readers::blockreader::{
    BlockOffset,
    BlockReader,
    BlockSz,
};
use crate::readers::helpers::is_empty;
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::byte_to_char_noraw;

use core::panic;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::io::{Error, ErrorKind};

use ::const_format::assertcp;
#[allow(unused_imports)]
use ::more_asserts::{
    assert_ge,
    assert_gt,
    assert_le,
    assert_lt,
    debug_assert_ge,
    debug_assert_gt,
    debug_assert_le,
    debug_assert_lt,
};
use numtoa::NumToA;  // adds `numtoa` method to numbers
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};


/// size of the `[u8]` buffer used for `numtoa` conversions
/// good up to `i64::MAX` or `i64::MIN` plus a little "just in case" head room
pub const NUMTOA_BUF_SZ: usize = 22;

/// A scoring system for the quality of the data in a [`FixedStruct`]. A higher score
/// means the data is more likely to be the [`FixedStructType`] expected.
pub type Score = i32;

/// Helper to [`FixedStructType::tv_pair_from_buffer`].
macro_rules! buffer_to_timeval {
    ($timeval_type:ty, $timeval_sz:expr, $buffer:ident, $tv_sec:ident, $tv_usec:ident) => ({{
        // ut_tv
        let size: usize = $timeval_sz;
        defo!("size {:?}", size);
        debug_assert_eq!($buffer.len(), size);
        let time_val: $timeval_type = unsafe {
            *($buffer.as_ptr() as *const $timeval_type)
        };
        // XXX: copy to local variable to avoid alignment warning
        //      see #82523 <https://github.com/rust-lang/rust/issues/82523>
        let _tv_sec = time_val.tv_sec;
        let _tv_usec = time_val.tv_usec;
        defo!("time_val.tv_sec {:?} .tv_usec {:?}", _tv_sec, _tv_usec);
        // ut_tv.tv_sec
        $tv_sec = match time_val.tv_sec.try_into() {
            Ok(val) => val,
            Err(_) => {
                debug_panic!("tv_sec overflow: {:?}", _tv_sec);
                return None;
            }
        };
        defo!("tv_sec {:?}", $tv_sec);
        // ut_tv.tv_usec
        $tv_usec = match time_val.tv_usec.try_into() {
            Ok(val) => val,
            Err(_) => {
                debug_panic!("tv_usec overflow: {:?}", _tv_usec);

                0
            }
        };
        defo!("tv_usec {:?}", $tv_usec);
    }})
}

macro_rules! buffer_to_time_t {
    ($ll_time_t:ty, $ll_time_sz:expr, $buffer:ident, $tv_sec:ident) => ({{
        let size: usize = $ll_time_sz;
        defo!("size {:?}", size);
        debug_assert_eq!($buffer.len(), size);
        let ll_time: $ll_time_t = unsafe {
            *($buffer.as_ptr() as *const $ll_time_t)
        };
        defo!("ll_time {:?}", ll_time);
        $tv_sec = match ll_time.try_into() {
            Ok(val) => val,
            Err(_) => {
                debug_panic!("tv_sec overflow from ll_time {:?}", ll_time);
                return None;
            }
        };
        defo!("tv_sec {:?}", $tv_sec);
    }})
}

/// Helper to [`FixedStructType::tv_pair_from_buffer`].
/// Debug print the `DateTimeL` conversion of a `tv_pair`.
macro_rules! defo_tv_pair {
    ($tv_sec:ident, $tv_usec:ident) => ({{
        #[cfg(any(debug_assertions, test))]
        {
            let tz_offset: FixedOffset = FixedOffset::east_opt(0).unwrap();
            let tv_pair = tv_pair_type($tv_sec, $tv_usec);
            match convert_tvpair_to_datetime(tv_pair, &tz_offset) {
                Ok(_dt) => {
                    defo!("dt: {:?}", _dt);
                }
                Err(err) => {
                    de_err!("{:?}", err);
                    panic!(
                        "convert_tvpair_to_datetime({:?}) returned an error; {}",
                        tv_pair, err
                    );
                }
            }
        }
    }})
}

/// FixedStruct Implementation Type (name `FixedStructType` is taken).
///
/// The specific implementation of the FixedStruct. Each implementation of,
/// for example, a `utmp` struct, differs in fields and sizes among Operating
/// Systems FreeBsd, Linux, OpenBSD, and NetBSD. and also differ per CPU
/// architecture, e.g. x86_64 vs. i386 vs. ARM7
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FixedStructType {

    // FreeBSD x86_64 (amd64), x86_32 (i686)

    /// corresponds to [`freebsd_x8664::utmpx`]
    #[allow(non_camel_case_types)]
    Fs_Freebsd_x8664_Utmpx,

    // Linux

    // Linux ARM64 (aarch64)

    /// corresponds to [`linux_arm64aarch64::lastlog`]
    #[allow(non_camel_case_types)]
    Fs_Linux_Arm64Aarch64_Lastlog,

    /// corresponds to [`linux_arm64aarch64::utmpx`]
    #[allow(non_camel_case_types)]
    Fs_Linux_Arm64Aarch64_Utmpx,

    // Linux x86

    /// corresponds to [`linux_x86::acct`]
    #[allow(non_camel_case_types)]
    Fs_Linux_x86_Acct,

    /// corresponds to [`linux_x86::acct_v3`]
    #[allow(non_camel_case_types)]
    Fs_Linux_x86_Acct_v3,

    /// corresponds to [`linux_x86::lastlog`]
    #[allow(non_camel_case_types)]
    Fs_Linux_x86_Lastlog,

    /// corresponds to [`linux_x86::utmpx`]
    #[allow(non_camel_case_types)]
    Fs_Linux_x86_Utmpx,

    // NetBSD

    // NetBSD 9 x86_32 (i686)

    /// corresponds to [`netbsd_x8632::acct`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8632_Acct,

    /// corresponds to [`netbsd_x8632::lastlogx`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8632_Lastlogx,

    /// corresponds to [`netbsd_x8632::utmpx`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8632_Utmpx,

    // NetBSD 9 x86_64 (AMD64)

    /// corresponds to [`netbsd_x8664::lastlog`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8664_Lastlog,

    /// corresponds to [`netbsd_x8664::lastlogx`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8664_Lastlogx,

    /// corresponds to [`netbsd_x8664::utmp`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8664_Utmp,

    /// corresponds to [`netbsd_x8664::utmpx`]
    #[allow(non_camel_case_types)]
    Fs_Netbsd_x8664_Utmpx,

    // OpenBSD x86_32, x86_64

    /// corresponds to [`openbsd_x86::lastlog`]
    #[allow(non_camel_case_types)]
    Fs_Openbsd_x86_Lastlog,

    /// corresponds to [`openbsd_x86::utmp`]
    #[allow(non_camel_case_types)]
    Fs_Openbsd_x86_Utmp,
}

impl FixedStructType {
    /// return the associated `FixedStructType`'s `SZ` value (size in bytes).
    pub const fn size(&self) -> usize {
        match self {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => freebsd_x8664::UTMPX_SZ,
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => linux_arm64aarch64::LASTLOG_SZ,
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => linux_arm64aarch64::UTMPX_SZ,
            FixedStructType::Fs_Linux_x86_Acct => linux_x86::ACCT_SZ,
            FixedStructType::Fs_Linux_x86_Acct_v3 => linux_x86::ACCT_V3_SZ,
            FixedStructType::Fs_Linux_x86_Lastlog => linux_x86::LASTLOG_SZ,
            FixedStructType::Fs_Linux_x86_Utmpx => linux_x86::UTMPX_SZ,
            FixedStructType::Fs_Netbsd_x8632_Acct => netbsd_x8632::ACCT_SZ,
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => netbsd_x8632::LASTLOGX_SZ,
            FixedStructType::Fs_Netbsd_x8632_Utmpx => netbsd_x8632::UTMPX_SZ,
            FixedStructType::Fs_Netbsd_x8664_Lastlog => netbsd_x8664::LASTLOG_SZ,
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => netbsd_x8664::LASTLOGX_SZ,
            FixedStructType::Fs_Netbsd_x8664_Utmp => netbsd_x8664::UTMP_SZ,
            FixedStructType::Fs_Netbsd_x8664_Utmpx => netbsd_x8664::UTMPX_SZ,
            FixedStructType::Fs_Openbsd_x86_Lastlog => openbsd_x86::LASTLOG_SZ,
            FixedStructType::Fs_Openbsd_x86_Utmp => openbsd_x86::UTMP_SZ,
        }
    }

    /// return the associated `FixedStructType`'s offset to it's ***t***ime
    /// ***v***alue field
    pub const fn offset_tv(&self) -> usize {
        match self {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => freebsd_x8664::UTMPX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => linux_arm64aarch64::LASTLOG_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => linux_arm64aarch64::UTMPX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Linux_x86_Acct => linux_x86::ACCT_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Linux_x86_Acct_v3 => linux_x86::ACCT_V3_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Linux_x86_Lastlog => linux_x86::LASTLOG_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Linux_x86_Utmpx => linux_x86::UTMPX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8632_Acct => netbsd_x8632::ACCT_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => netbsd_x8632::LASTLOGX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8632_Utmpx => netbsd_x8632::UTMPX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8664_Lastlog => netbsd_x8664::LASTLOG_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => netbsd_x8664::LASTLOGX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8664_Utmp => netbsd_x8664::UTMP_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Netbsd_x8664_Utmpx => netbsd_x8664::UTMPX_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Openbsd_x86_Lastlog => openbsd_x86::LASTLOG_TIMEVALUE_OFFSET,
            FixedStructType::Fs_Openbsd_x86_Utmp => openbsd_x86::UTMP_TIMEVALUE_OFFSET,
        }
    }

    /// return the associated `FixedStructType`'s size of it's
    /// ***t***ime ***v***alue in bytes
    pub const fn size_tv(&self) -> usize {
        match self {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => freebsd_x8664::UTMPX_TIMEVALUE_SZ,
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => linux_arm64aarch64::LASTLOG_TIMEVALUE_SZ,
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => linux_arm64aarch64::UTMPX_TIMEVALUE_SZ,
            FixedStructType::Fs_Linux_x86_Acct => linux_x86::ACCT_TIMEVALUE_SZ,
            FixedStructType::Fs_Linux_x86_Acct_v3 => linux_x86::ACCT_V3_TIMEVALUE_SZ,
            FixedStructType::Fs_Linux_x86_Lastlog => linux_x86::LASTLOG_TIMEVALUE_SZ,
            FixedStructType::Fs_Linux_x86_Utmpx => linux_x86::UTMPX_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8632_Acct => netbsd_x8632::ACCT_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => netbsd_x8632::LASTLOGX_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8632_Utmpx => netbsd_x8632::UTMPX_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8664_Lastlog => netbsd_x8664::LASTLOG_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => netbsd_x8664::LASTLOGX_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8664_Utmp => netbsd_x8664::UTMP_TIMEVALUE_SZ,
            FixedStructType::Fs_Netbsd_x8664_Utmpx => netbsd_x8664::UTMPX_TIMEVALUE_SZ,
            FixedStructType::Fs_Openbsd_x86_Lastlog => openbsd_x86::LASTLOG_TIMEVALUE_SZ,
            FixedStructType::Fs_Openbsd_x86_Utmp => openbsd_x86::UTMP_TIMEVALUE_SZ,
        }
    }

    /// the datetime field as a pair of seconds and microseconds taken from
    /// raw bytes
    pub fn tv_pair_from_buffer(&self, buffer: &[u8]) -> Option<tv_pair_type> {
        defn!("type {:?}", self);
        let tv_sec: tv_sec_type;
        let tv_usec: tv_usec_type;
        let size = self.size_tv();
        match self {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => {
                buffer_to_timeval!(freebsd_x8664::timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
                buffer_to_time_t!(linux_arm64aarch64::ll_time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
                buffer_to_timeval!(linux_arm64aarch64::timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Linux_x86_Acct => {
                buffer_to_time_t!(linux_x86::b_time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Linux_x86_Acct_v3 => {
                buffer_to_time_t!(linux_x86::b_time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Linux_x86_Lastlog => {
                buffer_to_time_t!(linux_x86::ll_time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Linux_x86_Utmpx => {
                buffer_to_timeval!(linux_x86::__timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8632_Acct => {
                buffer_to_time_t!(netbsd_x8632::time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
                buffer_to_timeval!(netbsd_x8632::timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8632_Utmpx => {
                buffer_to_timeval!(netbsd_x8632::timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlog => {
                buffer_to_time_t!(netbsd_x8664::time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
                buffer_to_timeval!(netbsd_x8664::timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8664_Utmp => {
                buffer_to_time_t!(netbsd_x8664::time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8664_Utmpx => {
                buffer_to_timeval!(netbsd_x8664::timeval, size, buffer, tv_sec, tv_usec);
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Openbsd_x86_Lastlog => {
                buffer_to_time_t!(openbsd_x86::time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
            FixedStructType::Fs_Openbsd_x86_Utmp => {
                buffer_to_time_t!(openbsd_x86::time_t, size, buffer, tv_sec);
                tv_usec = 0;
                defo_tv_pair!(tv_sec, tv_usec);
            }
        }

        let tv_pair = tv_pair_type(tv_sec, tv_usec);
        defx!("return Some({:?})", tv_pair);

        Some(tv_pair)
    }
}

// listing of specific FixedStruct implementations
// all listings should be in alphabetical order

// BUG: [2024/03/10] the `&CStr` wrappers do not protect against no
//      null-termination in the underlying buffer.
//      e.g. `freebsd_x8664::utmpx::ut_user()` may include the following field
//      `freebsd_x8664::utmpx::ut_line`.
//      This only affects debug printing and tests.

// freebsd_x8664

/// FixedStruct definitions found on FreeBSD 14.0 amd64 and FreeBSD 13.1 amd64
/// (x86_64).
#[allow(non_camel_case_types, unused)]
pub mod freebsd_x8664 {
    use crate::common::FileOffset;
    use std::ffi::CStr;
    use std::mem::size_of;
    use ::memoffset::offset_of;
    use ::const_format::assertcp_eq;

    pub type subseconds_t = std::ffi::c_longlong;
    // XXX: use `i64` to satisfy various cross-compilation targets
    pub type time_t = i64;

    // utmpx

    /// From [`/usr/include/sys/_timeval.h`]
    /// 
    /// ```C
    /// struct timeval {
    ///     time_t          tv_sec;         /* seconds */
    ///     suseconds_t     tv_usec;        /* and microseconds */
    /// };
    /// ```
    ///
    /// [`/usr/include/sys/_timeval.h`]: https://github.com/freebsd/freebsd-src/blob/release/12.4.0/sys/sys/_timeval.h#L46-L52
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct timeval {
        pub tv_sec: time_t,
        pub tv_usec: subseconds_t,
    }

    pub const TIMEVAL_SZ: usize = size_of::<timeval>();
    assertcp_eq!(TIMEVAL_SZ, 16);
    assertcp_eq!(offset_of!(timeval, tv_sec), 0);
    assertcp_eq!(offset_of!(timeval, tv_usec), 8);

    pub const UT_IDSIZE: usize = 8;
    pub const UT_USERSIZE: usize = 32;
    pub const UT_LINESIZE: usize = 16;
    pub const UT_HOSTSIZE: usize = 128;

    pub type c_char = std::ffi::c_char;
    pub type c_short = std::ffi::c_short;
    pub type c_ushort = std::ffi::c_ushort;
    pub type pid_t = std::ffi::c_int;

    /// From [`/usr/src/include/utmpx.h`] on FreeBSD 14.0 amd64
    ///
    /// ```C
    /// struct utmpx {
    ///     short           ut_type;        /* Type of entry. */
    ///     struct timeval  ut_tv;          /* Time entry was made. */
    ///     char            ut_id[8];       /* Record identifier. */
    ///     pid_t           ut_pid;         /* Process ID. */
    ///     char            ut_user[32];    /* User login name. */
    ///     char            ut_line[16];    /* Device name. */
    /// #if __BSD_VISIBLE
    ///     char            ut_host[128];   /* Remote hostname. */
    /// #else
    ///     char            __ut_host[128];
    /// #endif
    ///     char            __ut_spare[64];
    /// };
    /// ```
    ///
    /// Also see [FreeBSD 12.4 `utmpx.h`].
    /// Also see [_Replacing utmp.h with utmpx.h_].
    ///
    /// ---
    ///
    ///
    /// ```text
    /// timeval               sizeof  16
    /// timeval.tv_sec   @  0 sizeof   8
    /// timeval.tv_usec  @  8 sizeof   8
    ///
    /// utmpx                   sizeof 280
    /// utmpx.ut_type      @  0 sizeof   2
    /// utmpx.ut_tv        @  8 sizeof  16
    /// utmpx.ut_tv.tv_sec @  8 sizeof   8
    /// utmpx.ut_tv.tv_usec@ 16 sizeof   8
    /// utmpx.ut_id        @ 24 sizeof   8
    /// utmpx.ut_pid       @ 32 sizeof   4
    /// utmpx.ut_user      @ 36 sizeof  32
    /// utmpx.ut_line      @ 68 sizeof  16
    /// utmpx.ut_host      @ 84 sizeof 128
    /// ```
    ///
    /// [`/usr/src/include/utmpx.h`]: https://cgit.freebsd.org/src/tree/include/utmpx.h?h=stable/14
    /// [FreeBSD 12.4 `utmpx.h`]: https://github.com/freebsd/freebsd-src/blob/release/12.4.0/include/utmpx.h#L43-L56
    /// [_Replacing utmp.h with utmpx.h_]: https://lists.freebsd.org/pipermail/freebsd-arch/2010-January/009816.html
    #[derive(Clone, Copy)]
    #[allow(non_camel_case_types)]
    #[repr(C, align(8))]
    pub struct utmpx {
        pub ut_type: c_short,
        pub __gap1: [u8; 6],
        pub ut_tv: timeval,
        pub ut_id: [c_char; UT_IDSIZE],
        pub ut_pid: pid_t,
        pub ut_user: [c_char; UT_USERSIZE],
        pub ut_line: [c_char; UT_LINESIZE],
        pub ut_host: [c_char; UT_HOSTSIZE],
        pub __ut_spare: [c_char; 64],
    }

    pub const UTMPX_SZ: usize = size_of::<utmpx>();
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
    pub const UTMPX_TIMEVALUE_OFFSET: usize = offset_of!(utmpx, ut_tv);
    pub const UTMPX_TIMEVALUE_SZ: usize = TIMEVAL_SZ;
    pub const UTMPX_TIMEVALUE_OFFSET_TV_SEC: usize = UTMPX_TIMEVALUE_OFFSET + offset_of!(timeval, tv_sec);
    pub const UTMPX_TIMEVALUE_OFFSET_TV_USEC: usize = UTMPX_TIMEVALUE_OFFSET + offset_of!(timeval, tv_usec);
    assertcp_eq!(UTMPX_SZ, 280);
    assertcp_eq!(offset_of!(utmpx, ut_type), 0);
    assertcp_eq!(offset_of!(utmpx, __gap1), 2);
    assertcp_eq!(offset_of!(utmpx, ut_tv), 8);
    assertcp_eq!(offset_of!(utmpx, ut_id), 24);
    assertcp_eq!(offset_of!(utmpx, ut_pid), 32);
    assertcp_eq!(offset_of!(utmpx, ut_user), 36);
    assertcp_eq!(offset_of!(utmpx, ut_line), 68);
    assertcp_eq!(offset_of!(utmpx, ut_host), 84);
    assertcp_eq!(offset_of!(utmpx, __ut_spare), 212);

    /// Helpers for use in `fmt::Debug` trait.
    impl utmpx {
        pub fn ut_id(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_id[..UT_IDSIZE].as_ptr()) }
        }
        pub fn ut_user(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_user[..UT_USERSIZE].as_ptr()) }
        }
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line[..UT_LINESIZE].as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host[..UT_HOSTSIZE].as_ptr()) }
        }
    }

    /// From [`include/utmpx.h`], FreeBSD 12.4
    /// ```C
    /// #define	EMPTY		0	/* No valid user accounting information. */
    /// #define	BOOT_TIME	1	/* Time of system boot. */
    /// #define	OLD_TIME	2	/* Time when system clock changed. */
    /// #define	NEW_TIME	3	/* Time after system clock changed. */
    /// #define	USER_PROCESS	4	/* A process. */
    /// #define	INIT_PROCESS	5	/* A process spawned by the init process. */
    /// #define	LOGIN_PROCESS	6	/* The session leader of a logged-in user. */
    /// #define	DEAD_PROCESS	7	/* A session leader who has exited. */
    /// #if __BSD_VISIBLE
    /// #define	SHUTDOWN_TIME	8	/* Time of system shutdown. */
    /// #endif
    /// 
    /// #if __BSD_VISIBLE
    /// #define	UTXDB_ACTIVE	0	/* Active login sessions. */
    /// #define	UTXDB_LASTLOGIN	1	/* Last login sessions. */
    /// #define	UTXDB_LOG	2	/* Log indexed by time. */
    /// #endif
    /// ```
    ///
    /// [`include/utmpx.h`]: https://github.com/freebsd/freebsd-src/blob/release/12.4.0/include/utmpx.h#L58C1-L74C7
    pub const UT_TYPES: [c_short; 9] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8
    ];
}

/// FixedStruct definitions found on Linux running on ARM64 (aarch64)
/// architecture.
#[allow(non_camel_case_types)]
pub mod linux_arm64aarch64 {
    use crate::common::FileOffset;
    use std::ffi::CStr;
    use std::mem::size_of;
    use ::const_format::assertcp_eq;
    use ::memoffset::offset_of;

    pub type c_char = std::ffi::c_char;
    pub type c_short = std::ffi::c_short;
    pub type ll_time_t = std::ffi::c_longlong;
    pub type pid_t = std::ffi::c_int;
    pub type suseconds_t = std::ffi::c_longlong;

    // timeval

    /// ```text
    /// timeval               sizeof  16
    /// timeval.tv_sec   @  0 sizeof   8
    /// timeval.tv_usec  @  8 sizeof   8
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct timeval {
        pub tv_sec: i64,
        pub tv_usec: i64,
    }

    pub const TIMEVAL_SZ: usize = size_of::<timeval>();
    assertcp_eq!(TIMEVAL_SZ, 16);
    assertcp_eq!(offset_of!(timeval, tv_sec), 0);
    assertcp_eq!(offset_of!(timeval, tv_usec), 8);

    pub const UT_LINESIZE: usize = 32;
    pub const UT_IDSIZE: usize = 4;
    pub const UT_NAMESIZE: usize = 32;
    pub const UT_USERSIZE: usize = 32;
    pub const UT_HOSTSIZE: usize = 256;

    // lastlog

    /// ```text
    /// lastlog               sizeof 296
    /// lastlog.ll_time  @  0 sizeof   8
    /// lastlog.ll_line  @  8 sizeof  32
    /// lastlog.ll_host  @ 40 sizeof 256
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct lastlog {
        pub ll_time: ll_time_t,
        pub ll_line: [c_char; UT_LINESIZE],
        pub ll_host: [c_char; UT_HOSTSIZE],
    }

    impl lastlog {
        pub fn ll_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_line.as_ptr()) }
        }
        pub fn ll_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_host.as_ptr()) }
        }
    }

    pub const LASTLOG_SZ: usize = size_of::<lastlog>();
    pub const LASTLOG_SZ_FO: FileOffset = LASTLOG_SZ as FileOffset;
    pub const LASTLOG_TIMEVALUE_OFFSET: usize = offset_of!(lastlog, ll_time);
    pub const LASTLOG_TIMEVALUE_SZ: usize = size_of::<ll_time_t>();
    assertcp_eq!(LASTLOG_SZ, 296);
    assertcp_eq!(offset_of!(lastlog, ll_time), 0);
    assertcp_eq!(offset_of!(lastlog, ll_line), 8);
    assertcp_eq!(offset_of!(lastlog, ll_host), 40);

    // utmp == utmpx

    /// From [`man utmpx`]
    ///
    /// ```C
    /// /* Values for ut_type field, below */
    ///
    /// #define EMPTY         0 /* Record does not contain valid info
    /// (formerly known as UT_UNKNOWN on Linux) */
    /// #define RUN_LVL       1 /* Change in system run-level (see
    /// init(8)) */
    /// #define BOOT_TIME     2 /* Time of system boot (in ut_tv) */
    /// #define NEW_TIME      3 /* Time after system clock change
    /// (in ut_tv) */
    /// #define OLD_TIME      4 /* Time before system clock change
    /// (in ut_tv) */
    /// #define INIT_PROCESS  5 /* Process spawned by init(8) */
    /// #define LOGIN_PROCESS 6 /* Session leader process for user login */
    /// #define USER_PROCESS  7 /* Normal process */
    /// #define DEAD_PROCESS  8 /* Terminated process */
    /// #define ACCOUNTING    9 /* Not implemented */
    ///
    /// #define UT_LINESIZE      32
    /// #define UT_NAMESIZE      32
    /// #define UT_HOSTSIZE     256
    ///
    /// struct exit_status {          /* Type for ut_exit, below */
    /// short int e_termination;      /* Process termination status */
    /// short int e_exit;             /* Process exit status */
    /// };
    ///
    /// struct utmp {
    /// short   ut_type;              /* Type of record */
    /// pid_t   ut_pid;               /* PID of login process */
    /// char    ut_line[UT_LINESIZE]; /* Device name of tty - "/dev/" */
    /// char    ut_id[4];             /* Terminal name suffix,
    ///                                  or inittab(5) ID */
    /// char    ut_user[UT_NAMESIZE]; /* Username */
    /// char    ut_host[UT_HOSTSIZE]; /* Hostname for remote login, or
    ///                                  kernel version for run-level
    ///                                  messages */
    /// struct  exit_status ut_exit;  /* Exit status of a process
    ///                                  marked as DEAD_PROCESS; not
    ///                                  used by Linux init(8) */
    /// /* The ut_session and ut_tv fields must be the same size when
    /// compiled 32- and 64-bit.  This allows data files and shared
    /// memory to be shared between 32- and 64-bit applications. */
    /// #if __WORDSIZE == 64 && defined __WORDSIZE_COMPAT32
    /// int32_t ut_session;           /* Session ID (getsid(2)),
    ///                                  used for windowing */
    /// struct {
    ///     int32_t tv_sec;           /* Seconds */
    ///     int32_t tv_usec;          /* Microseconds */
    /// } ut_tv;                      /* Time entry was made */
    /// #else
    /// long   ut_session;           /* Session ID */
    /// struct timeval ut_tv;        /* Time entry was made */
    /// #endif
    ///
    /// int32_t ut_addr_v6[4];        /* Internet address of remote
    ///                                  host; IPv4 address uses
    ///                                  just ut_addr_v6[0] */
    /// char __unused[20];            /* Reserved for future use */
    /// };
    ///
    /// /* Backward compatibility hacks */
    /// #define ut_name ut_user
    /// #ifndef _NO_UT_TIME
    /// #define ut_time ut_tv.tv_sec
    /// #endif
    /// #define ut_xtime ut_tv.tv_sec
    /// #define ut_addr ut_addr_v6[0]
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// utmpx                   sizeof 400
    /// utmpx.ut_type      @  0 sizeof   2
    /// utmpx.ut_pid       @  4 sizeof   4
    /// utmpx.ut_line      @  8 sizeof  32
    /// utmpx.ut_id        @ 40 sizeof   4
    /// utmpx.ut_user      @ 44 sizeof  32
    /// utmpx.ut_host      @ 76 sizeof 256
    /// utmpx.ut_exit      @332 sizeof   4
    /// utmpx.ut_session   @336 sizeof   8
    /// utmpx.ut_tv        @344 sizeof  16
    /// utmpx.ut_tv.tv_sec @344 sizeof   8
    /// utmpx.ut_tv.tv_usec@352 sizeof   8
    /// utmpx.ut_addr      @360 sizeof   4
    /// utmpx.ut_addr_v6   @360 sizeof  16
    /// ```
    ///
    /// [`man utmpx`]: https://linux.die.net/man/5/utmpx
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct utmpx {
        pub ut_type: c_short,
        pub ut_pid: pid_t,
        pub ut_line: [c_char; UT_LINESIZE],
        pub ut_id: [c_char; UT_IDSIZE],
        pub ut_user: [c_char; UT_NAMESIZE],
        pub ut_host: [c_char; UT_HOSTSIZE],
        pub ut_exit: i32,
        pub ut_session: i64,
        pub ut_tv: timeval,
        pub ut_addr_v6: [i32; 4],
        pub __glibc_reserved: [c_char; 20],
    }

    impl utmpx {
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line.as_ptr()) }
        }
        pub fn ut_id(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_id.as_ptr()) }
        }
        pub fn ut_user(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_user.as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host.as_ptr()) }
        }
    }

    pub const UTMPX_SZ: usize = size_of::<utmpx>();
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
    pub const UTMPX_TIMEVALUE_OFFSET: usize = offset_of!(utmpx, ut_tv);
    pub const UTMPX_TIMEVALUE_SZ: usize = TIMEVAL_SZ;
    assertcp_eq!(UTMPX_SZ, 400);
    assertcp_eq!(offset_of!(utmpx, ut_type), 0);
    assertcp_eq!(offset_of!(utmpx, ut_pid), 4);
    assertcp_eq!(offset_of!(utmpx, ut_line), 8);
    assertcp_eq!(offset_of!(utmpx, ut_id), 40);
    assertcp_eq!(offset_of!(utmpx, ut_user), 44);
    assertcp_eq!(offset_of!(utmpx, ut_host), 76);
    assertcp_eq!(offset_of!(utmpx, ut_exit), 332);
    assertcp_eq!(offset_of!(utmpx, ut_session), 336);
    assertcp_eq!(offset_of!(utmpx, ut_tv), 344);
    assertcp_eq!(offset_of!(utmpx, ut_addr_v6), 360);
    assertcp_eq!(offset_of!(utmpx, __glibc_reserved), 376);

    /// From [`utmpx.h`], Linux 6.1
    /// ```C
    /// #define EMPTY           0
    /// #define RUN_LVL         1
    /// #define BOOT_TIME       2
    /// #define NEW_TIME        3
    /// #define OLD_TIME        4
    /// #define INIT_PROCESS    5
    /// #define LOGIN_PROCESS   6
    /// #define USER_PROCESS    7
    /// #define DEAD_PROCESS    8
    /// ```
    ///
    /// [`utmpx.h`]: https://elixir.bootlin.com/musl/latest/source/include/utmpx.h
    pub const UT_TYPES: [c_short; 9] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8
    ];
}

/// FixedStruct definitions found in `lastlog.h`, `utmp.h`, `utmpx.h`
/// from GNU glibc for Linux, architectures amd64 (x86_64), i686 (x86_32),
/// ARM6 (aarch64), and RISC-V (riscv64).
///
/// Found on Ubuntu 22.04 amd64 (x86_64) Linux 5.15, in files:
/// * `/usr/include/lastlog.h`
/// * `/usr/include/utmp.h`
/// * `/usr/include/utmpx.h`
/// * `/usr/include/x86_64-linux-gnu/bits/utmp.h`
/// * `/usr/include/x86_64-linux-gnu/bits/utmpx.h`
///
/// Confirmed the sizes and offsets are the same on:
/// * CentOS 7 amd64 (x86_64) Linux 3.10
/// * Ubuntu 16 i686 (x86_32) Linux 4.4
/// * Debian 11 ARM6 (aarch64) Linux 6.1
/// * Debian 13 RISC-V (riscv64) Linux 6.1
///
/// However, `struct timeval` differs on in size on x86_64/32 and ARM6. Yet the
/// `utmpx.ut_tv`, which is sometimes defined in-place and sometimes typed as a
/// `struct timeval` or `struct __timeval`, is the same size on both.
/// From the [code comment in `utmpx.h`]:
/// ```C
/// /* The fields ut_session and ut_tv must be the same size when compiled
///    32- and 64-bit.  This allows files and shared memory to be shared
///    between 32- and 64-bit applications.  */
/// ```
///
/// The `utmp` struct is exactly the same as the `utmpx` struct except for a few
/// different names. This mod defines `utmpx` and not `utmp`.
///
/// [code comment in `utmpx.h`]: https://elixir.bootlin.com/glibc/latest/source/sysdeps/gnu/bits/utmpx.h
#[allow(non_camel_case_types)]
pub mod linux_x86 {
    use crate::common::FileOffset;
    use std::ffi::CStr;
    use std::mem::size_of;
    use ::memoffset::offset_of;
    use ::const_format::assertcp_eq;

    pub type b_time_t = std::ffi::c_uint;
    pub type c_char = std::ffi::c_char;
    pub type c_short = std::ffi::c_short;
    pub type comp_t = std::ffi::c_ushort;
    pub type ll_time_t = std::ffi::c_int;
    pub type pid_t = std::ffi::c_int;
    pub type suseconds_t = std::ffi::c_longlong;
    pub type uint16_t = std::ffi::c_ushort;

    pub const PATH_ACCT: &str = "/var/log/account/acct";
    pub const PATH_PACCT: &str = "/var/log/account/pacct";

    pub const ACCT_COMM: usize = 16;

    pub const AFORK: c_char = 0x01;
    pub const ASU: c_char = 0x02;
    pub const ACOMPAT: c_char = 0x04;
    pub const ACORE: c_char = 0x08;
    pub const AXSIG: c_char = 0x10;

    pub const AC_FLAGS_MASK: c_char = AFORK | ASU | ACOMPAT | ACORE | AXSIG;

    /// from [`/usr/include/uapi/linux/acct.h`] on Ubuntu 22.04
    ///
    /// [`/usr/include/uapi/linux/acct.h`]: https://github.com/torvalds/linux/blob/v5.15/include/uapi/linux/acct.h#L75-L101
    ///
    /// ```C
    /// struct acct
    /// {
    ///   char ac_flag;                 /* Flags.  */
    ///   uint16_t ac_uid;              /* Real user ID.  */
    ///   uint16_t ac_gid;              /* Real group ID.  */
    ///   uint16_t ac_tty;              /* Controlling terminal.  */
    ///   uint32_t ac_btime;            /* Beginning time.  */
    ///   comp_t ac_utime;              /* User time.  */
    ///   comp_t ac_stime;              /* System time.  */
    ///   comp_t ac_etime;              /* Elapsed time.  */
    ///   comp_t ac_mem;                /* Average memory usage.  */
    ///   comp_t ac_io;                 /* Chars transferred.  */
    ///   comp_t ac_rw;                 /* Blocks read or written.  */
    ///   comp_t ac_minflt;             /* Minor pagefaults.  */
    ///   comp_t ac_majflt;             /* Major pagefaults.  */
    ///   comp_t ac_swaps;              /* Number of swaps.  */
    ///   uint32_t ac_exitcode;         /* Process exitcode.  */
    ///   char ac_comm[ACCT_COMM+1];    /* Command name.  */
    ///   char ac_pad[10];              /* Padding bytes.  */
    /// }; 
    ///
    /// #define AFORK		0x01	/* ... executed fork, but did not exec */
    /// #define ASU		0x02	/* ... used super-user privileges */
    /// #define ACOMPAT		0x04	/* ... used compatibility mode (VAX only not used) */
    /// #define ACORE		0x08	/* ... dumped core */
    /// #define AXSIG		0x10	/* ... was killed by a signal */
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// acct                 sizeof  64
    /// acct.ac_flag    @  0 sizeof   1
    /// acct.ac_uid     @  2 sizeof   2
    /// acct.ac_gid     @  4 sizeof   2
    /// acct.ac_tty     @  6 sizeof   2
    /// acct.ac_btime   @  8 sizeof   4
    /// acct.ac_utime   @ 12 sizeof   2
    /// acct.ac_stime   @ 14 sizeof   2
    /// acct.ac_etime   @ 16 sizeof   2
    /// acct.ac_mem     @ 18 sizeof   2
    /// acct.ac_io      @ 20 sizeof   2
    /// acct.ac_rw      @ 22 sizeof   2
    /// acct.ac_exitcode@ 32 sizeof   4
    /// acct.ac_comm    @ 36 sizeof  17
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(2))]
    #[allow(non_camel_case_types)]
    pub struct acct {
        pub ac_flag: c_char,
        pub ac_uid: uint16_t,
        pub ac_gid: uint16_t,
        pub ac_tty: uint16_t,
        pub ac_btime: b_time_t,
        pub ac_utime: comp_t,
        pub ac_stime: comp_t,
        pub ac_etime: comp_t,
        pub ac_mem: comp_t,
        pub ac_io: comp_t,
        pub ac_rw: comp_t,
        pub ac_minflt: comp_t,
        pub ac_majflt: comp_t,
        pub ac_swaps: comp_t,
        pub ac_exitcode: u32,
        pub ac_comm: [c_char; ACCT_COMM + 1],
        pub ac_pad: [c_char; 10],
    }

    pub const ACCT_SZ: usize = size_of::<acct>();
    pub const ACCT_SZ_FO: FileOffset = ACCT_SZ as FileOffset;
    pub const ACCT_TIMEVALUE_OFFSET: usize = offset_of!(acct, ac_btime);
    pub const ACCT_TIMEVALUE_SZ: usize = size_of::<b_time_t>();
    assertcp_eq!(ACCT_SZ, 64);
    assertcp_eq!(offset_of!(acct, ac_flag), 0);
    assertcp_eq!(offset_of!(acct, ac_uid), 2);
    assertcp_eq!(offset_of!(acct, ac_gid), 4);
    assertcp_eq!(offset_of!(acct, ac_tty), 6);
    assertcp_eq!(offset_of!(acct, ac_btime), 8);
    assertcp_eq!(offset_of!(acct, ac_utime), 12);
    assertcp_eq!(offset_of!(acct, ac_stime), 14);
    assertcp_eq!(offset_of!(acct, ac_etime), 16);
    assertcp_eq!(offset_of!(acct, ac_mem), 18);
    assertcp_eq!(offset_of!(acct, ac_io), 20);
    assertcp_eq!(offset_of!(acct, ac_rw), 22);
    assertcp_eq!(offset_of!(acct, ac_minflt), 24);
    assertcp_eq!(offset_of!(acct, ac_majflt), 26);
    assertcp_eq!(offset_of!(acct, ac_swaps), 28);
    assertcp_eq!(offset_of!(acct, ac_exitcode), 32);
    assertcp_eq!(offset_of!(acct, ac_comm), 36);
    assertcp_eq!(offset_of!(acct, ac_pad), 53);

    impl acct {
        pub fn ac_comm(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ac_comm.as_ptr()) }
        }
    }

    /// From [`/usr/include/uapi/linux/acct.h`] on Ubuntu 22.04
    ///
    /// [`/usr/include/uapi/linux/acct.h`]: https://github.com/torvalds/linux/blob/v5.15/include/uapi/linux/acct.h#L75-L101
    ///
    /// ```C
    /// struct acct_v3
    /// {
    ///         char            ac_flag;                /* Flags */
    ///         char            ac_version;             /* Always set to ACCT_VERSION */
    ///         __u16           ac_tty;                 /* Control Terminal */
    ///         __u32           ac_exitcode;            /* Exitcode */
    ///         __u32           ac_uid;                 /* Real User ID */
    ///         __u32           ac_gid;                 /* Real Group ID */
    ///         __u32           ac_pid;                 /* Process ID */
    ///         __u32           ac_ppid;                /* Parent Process ID */
    ///         /* __u32 range means times from 1970 to 2106 */
    ///         __u32           ac_btime;               /* Process Creation Time */
    ///         float           ac_etime;               /* Elapsed Time */
    ///         comp_t          ac_utime;               /* User Time */
    ///         comp_t          ac_stime;               /* System Time */
    ///         comp_t          ac_mem;                 /* Average Memory Usage */
    ///         comp_t          ac_io;                  /* Chars Transferred */
    ///         comp_t          ac_rw;                  /* Blocks Read or Written */
    ///         comp_t          ac_minflt;              /* Minor Pagefaults */
    ///         comp_t          ac_majflt;              /* Major Pagefaults */
    ///         comp_t          ac_swaps;               /* Number of Swaps */
    ///         char            ac_comm[ACCT_COMM];     /* Command Name */
    /// };
    ///
    /// #define AFORK		0x01	/* ... executed fork, but did not exec */
    /// #define ASU		0x02	/* ... used super-user privileges */
    /// #define ACOMPAT		0x04	/* ... used compatibility mode (VAX only not used) */
    /// #define ACORE		0x08	/* ... dumped core */
    /// #define AXSIG		0x10	/* ... was killed by a signal */
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// acct_v3                  sizeof  64
    /// acct_v3.ac_flag     @  0 sizeof   1
    /// acct_v3.ac_version  @  1 sizeof   1
    /// acct_v3.ac_tty      @  2 sizeof   2
    /// acct_v3.ac_exitcode @  4 sizeof   4
    /// acct_v3.ac_uid      @  8 sizeof   4
    /// acct_v3.ac_gid      @ 12 sizeof   4
    /// acct_v3.ac_pid      @ 16 sizeof   4
    /// acct_v3.ac_ppid     @ 20 sizeof   4
    /// acct_v3.ac_btime    @ 24 sizeof   4
    /// acct_v3.ac_etime    @ 28 sizeof   4
    /// acct_v3.ac_utime    @ 32 sizeof   2
    /// acct_v3.ac_stime    @ 34 sizeof   2
    /// acct_v3.ac_mem      @ 36 sizeof   2
    /// acct_v3.ac_io       @ 38 sizeof   2
    /// acct_v3.ac_rw       @ 40 sizeof   2
    /// acct_v3.ac_minflt   @ 42 sizeof   2
    /// acct_v3.ac_majflt   @ 44 sizeof   2
    /// acct_v3.ac_swaps    @ 46 sizeof   2
    /// acct_v3.ac_comm     @ 48 sizeof  16
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct acct_v3 {
        pub ac_flag: c_char,
        pub ac_version: c_char,
        pub ac_tty: u16,
        pub ac_exitcode: u32,
        pub ac_uid: u32,
        pub ac_gid: u32,
        pub ac_pid: u32,
        pub ac_ppid: u32,
        pub ac_btime: b_time_t,
        pub ac_etime: f32,
        pub ac_utime: comp_t,
        pub ac_stime: comp_t,
        pub ac_mem: comp_t,
        pub ac_io: comp_t,
        pub ac_rw: comp_t,
        pub ac_minflt: comp_t,
        pub ac_majflt: comp_t,
        pub ac_swaps: comp_t,
        pub ac_comm: [c_char; ACCT_COMM],
    }
    // XXX: not handling ACCT_BYTEORDER which changes the byte order
    //      of the file

    pub const ACCT_V3_SZ: usize = size_of::<acct_v3>();
    pub const ACCT_V3_SZ_FO: FileOffset = ACCT_V3_SZ as FileOffset;
    pub const ACCT_V3_TIMEVALUE_OFFSET: usize = offset_of!(acct_v3, ac_btime);
    pub const ACCT_V3_TIMEVALUE_SZ: usize = size_of::<b_time_t>();
    assertcp_eq!(ACCT_V3_SZ, 64);
    assertcp_eq!(offset_of!(acct_v3, ac_flag), 0);
    assertcp_eq!(offset_of!(acct_v3, ac_version), 1);
    assertcp_eq!(offset_of!(acct_v3, ac_tty), 2);
    assertcp_eq!(offset_of!(acct_v3, ac_exitcode), 4);
    assertcp_eq!(offset_of!(acct_v3, ac_uid), 8);
    assertcp_eq!(offset_of!(acct_v3, ac_gid), 12);
    assertcp_eq!(offset_of!(acct_v3, ac_pid), 16);
    assertcp_eq!(offset_of!(acct_v3, ac_ppid), 20);
    assertcp_eq!(offset_of!(acct_v3, ac_btime), 24);
    assertcp_eq!(offset_of!(acct_v3, ac_etime), 28);
    assertcp_eq!(offset_of!(acct_v3, ac_utime), 32);
    assertcp_eq!(offset_of!(acct_v3, ac_stime), 34);
    assertcp_eq!(offset_of!(acct_v3, ac_mem), 36);
    assertcp_eq!(offset_of!(acct_v3, ac_io), 38);
    assertcp_eq!(offset_of!(acct_v3, ac_rw), 40);
    assertcp_eq!(offset_of!(acct_v3, ac_minflt), 42);
    assertcp_eq!(offset_of!(acct_v3, ac_majflt), 44);
    assertcp_eq!(offset_of!(acct_v3, ac_swaps), 46);
    assertcp_eq!(offset_of!(acct_v3, ac_comm), 48);

    impl acct_v3 {
        pub fn ac_comm(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ac_comm.as_ptr()) }
        }
    }

    // lastlog

    pub const UT_LINESIZE: usize = 32;
    pub const UT_IDSIZE: usize = 4;
    pub const UT_NAMESIZE: usize = 32;
    pub const UT_USERSIZE: usize = 32;
    pub const UT_HOSTSIZE: usize = 256;

    /// ```text
    /// lastlog               sizeof 292
    /// lastlog.ll_time  @  0 sizeof   4
    /// lastlog.ll_line  @  4 sizeof  32
    /// lastlog.ll_host  @ 36 sizeof 256
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct lastlog {
        pub ll_time: ll_time_t,
        pub ll_line: [c_char; UT_LINESIZE],
        pub ll_host: [c_char; UT_HOSTSIZE],
    }

    pub const LASTLOG_SZ: usize = size_of::<lastlog>();
    pub const LASTLOG_SZ_FO: FileOffset = LASTLOG_SZ as FileOffset;
    pub const LASTLOG_TIMEVALUE_OFFSET: usize = offset_of!(lastlog, ll_time);
    pub const LASTLOG_TIMEVALUE_SZ: usize = size_of::<ll_time_t>();
    assertcp_eq!(LASTLOG_SZ, 292);
    assertcp_eq!(offset_of!(lastlog, ll_time), 0);
    assertcp_eq!(offset_of!(lastlog, ll_line), 4);
    assertcp_eq!(offset_of!(lastlog, ll_host), 36);

    impl lastlog {
        pub fn ll_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_line.as_ptr()) }
        }
        pub fn ll_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_host.as_ptr()) }
        }
    }

    // utmp

    /// `utmp` == `utmpx` on this platform
    ///
    /// Strangely, the `timeval` on x86_64/32 is sizeof 16 and on ARM6 is
    /// sizeof 8. Yet the `tutmpx.ut_tv`, sometimes typed as a  `__timeval`,
    /// is sizeof 8 on both.
    /// So this `__timeval` is sized to 8 so the `utmp` is the correct size.
    ///
    /// ```text
    /// timeval               sizeof   8
    /// timeval.tv_sec   @  0 sizeof   4
    /// timeval.tv_usec  @  4 sizeof   4
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct __timeval {
        pub tv_sec: i32,
        pub tv_usec: i32,
    }
    
    pub const __TIMEVAL_SZ: usize = size_of::<__timeval>();
    assertcp_eq!(__TIMEVAL_SZ, 8);
    assertcp_eq!(offset_of!(__timeval, tv_sec), 0);
    assertcp_eq!(offset_of!(__timeval, tv_usec), 4);

    #[derive(Clone, Copy)]
    #[repr(C, align(2))]
    #[allow(non_camel_case_types)]
    pub struct __exit_status {
        pub e_termination: i16,
        pub e_exit: i16,
    }

    pub const __EXIT_STATUS: usize = size_of::<__exit_status>();
    assertcp_eq!(__EXIT_STATUS, 4);
    assertcp_eq!(offset_of!(__exit_status, e_termination), 0);
    assertcp_eq!(offset_of!(__exit_status, e_exit), 2);

    // TODO: [2024/02/16] add faillog struct.
    //       `faillog` is from older CentOS releases so low priority.
    //       https://unix.stackexchange.com/a/182107/21203

    /// The [`utmpx` struct].
    ///
    /// The `utmp` struct is the exact same.
    ///
    /// Taken from [`/usr/include/x86_64-linux-gnu/bits/utmpx.h`] on Ubuntu 22.04
    /// x86_64:
    /// 
    /// ```C
    /// /* The structure describing an entry in the user accounting database.  */
    /// struct utmpx
    /// {
    ///   short int ut_type;            /* Type of login.  */
    ///   __pid_t ut_pid;               /* Process ID of login process.  */
    ///   char ut_line[__UT_LINESIZE]
    ///     __attribute_nonstring__;    /* Devicename.  */
    ///   char ut_id[4]
    ///     __attribute_nonstring__;    /* Inittab ID.  */
    ///   char ut_user[__UT_NAMESIZE]
    ///     __attribute_nonstring__;    /* Username.  */
    ///   char ut_host[__UT_HOSTSIZE]
    ///     __attribute_nonstring__;    /* Hostname for remote login.  */
    ///   struct __exit_status ut_exit; /* Exit status of a process marked
    ///                                    as DEAD_PROCESS.  */
    /// /* The fields ut_session and ut_tv must be the same size when compiled
    ///    32- and 64-bit.  This allows files and shared memory to be shared
    ///    between 32- and 64-bit applications.  */
    /// #if __WORDSIZE_TIME64_COMPAT32
    ///   __int32_t ut_session;         /* Session ID, used for windowing.  */
    ///   struct
    ///   {
    ///     __int32_t tv_sec;           /* Seconds.  */
    ///     __int32_t tv_usec;          /* Microseconds.  */
    ///   } ut_tv;                      /* Time entry was made.  */
    /// #else
    ///   long int ut_session;          /* Session ID, used for windowing.  */
    ///   struct timeval ut_tv;         /* Time entry was made.  */
    /// #endif
    ///   __int32_t ut_addr_v6[4];      /* Internet address of remote host.  */
    ///   char __glibc_reserved[20];    /* Reserved for future use.  */
    /// };
    /// ```
    ///
    /// The prior code snippet is reprinted here by allowance of the
    /// GNU Lesser General Public License version 2.
    ///
    /// ---
    ///
    /// ```text
    /// utmpx                   sizeof 384
    /// utmpx.ut_type      @  0 sizeof   2
    /// utmpx.ut_pid       @  4 sizeof   4
    /// utmpx.ut_line      @  8 sizeof  32
    /// utmpx.ut_id        @ 40 sizeof   4
    /// utmpx.ut_user      @ 44 sizeof  32
    /// utmpx.ut_name      @ 44 sizeof  32
    /// utmpx.ut_host      @ 76 sizeof 256
    /// utmpx.ut_exit      @332 sizeof   4
    /// utmpx.ut_session   @336 sizeof   4
    /// utmpx.ut_time      @340 sizeof   4
    /// utmpx.ut_xtime     @340 sizeof   4
    /// utmpx.ut_tv        @340 sizeof   8
    /// utmpx.ut_tv.tv_sec @340 sizeof   4
    /// utmpx.ut_tv.tv_usec@344 sizeof   4
    /// utmpx.ut_addr      @348 sizeof   4
    /// utmpx.ut_addr_v6   @348 sizeof  16
    /// ```
    ///
    /// [`utmpx` struct]: https://linux.die.net/man/5/utmpx
    /// [`/usr/include/x86_64-linux-gnu/bits/utmpx.h`]: https://elixir.bootlin.com/glibc/latest/source/sysdeps/gnu/bits/utmpx.h
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct utmpx {
        pub ut_type: c_short,
        pub ut_pid: pid_t,
        pub ut_line: [c_char; UT_LINESIZE],
        pub ut_id: [c_char; UT_IDSIZE],
        pub ut_user: [c_char; UT_USERSIZE],
        pub ut_host: [c_char; UT_HOSTSIZE],
        pub ut_exit: __exit_status,
        pub ut_session: i32,
        pub ut_tv: __timeval,
        pub ut_addr_v6: [i32; 4],
        /* private fields */
        pub __glibc_reserved: [i8; 20],
    }

    pub const UTMPX_SZ: usize = size_of::<utmpx>();
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
    pub const UTMPX_TIMEVALUE_OFFSET: usize = offset_of!(utmpx, ut_tv);
    pub const UTMPX_TIMEVALUE_SZ: usize = __TIMEVAL_SZ;
    assertcp_eq!(UTMPX_SZ, 384);
    assertcp_eq!(offset_of!(utmpx, ut_type), 0);
    assertcp_eq!(offset_of!(utmpx, ut_pid), 4);
    assertcp_eq!(offset_of!(utmpx, ut_line), 8);
    assertcp_eq!(offset_of!(utmpx, ut_id), 40);
    assertcp_eq!(offset_of!(utmpx, ut_user), 44);
    assertcp_eq!(offset_of!(utmpx, ut_host), 76);
    assertcp_eq!(offset_of!(utmpx, ut_exit), 332);
    assertcp_eq!(offset_of!(utmpx, ut_session), 336);
    assertcp_eq!(offset_of!(utmpx, ut_tv), 340);
    assertcp_eq!(offset_of!(utmpx, ut_addr_v6), 348);
    assertcp_eq!(offset_of!(utmpx, __glibc_reserved), 364);

    /// helpers for `fmt::Debug` trait
    ///
    /// The slicing in each `CStr` function below is to due to
    /// `__attribute_nonstring__` which means the field may not end with a
    /// null byte.
    impl utmpx {
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line[..UT_LINESIZE].as_ptr()) }
        }
        pub fn ut_id(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_id[..UT_IDSIZE].as_ptr()) }
        }
        pub fn ut_user(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_user[..UT_USERSIZE].as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host[..UT_HOSTSIZE].as_ptr()) }
        }
    }

    /// From [`utmpx.h`], Linux 6.1
    /// ```C
    /// #define EMPTY           0
    /// #define RUN_LVL         1
    /// #define BOOT_TIME       2
    /// #define NEW_TIME        3
    /// #define OLD_TIME        4
    /// #define INIT_PROCESS    5
    /// #define LOGIN_PROCESS   6
    /// #define USER_PROCESS    7
    /// #define DEAD_PROCESS    8
    /// ```
    ///
    /// [`utmpx.h`]: https://elixir.bootlin.com/musl/latest/source/include/utmpx.h
    pub const UT_TYPES: [c_short; 9] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8
    ];
}

/// FixedStruct definitions found on NetBSD 9.3 i686 (x86_32).
/// These are slightly different than amd64 (x86_64).
#[allow(non_camel_case_types, unused)]
pub mod netbsd_x8632 {
    use crate::common::FileOffset;
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::mem::size_of;
    use ::memoffset::offset_of;
    use ::const_format::assertcp_eq;

    pub type comp_t = std::ffi::c_ushort;
    pub type c_char = std::ffi::c_char;
    pub type uid_t = std::ffi::c_uint;
    pub type gid_t = std::ffi::c_uint;
    pub type dev_t = std::ffi::c_longlong;
    pub type pid_t = std::ffi::c_int;
    pub type time_t = std::ffi::c_longlong;
    pub type uint8_t = std::ffi::c_uchar;
    pub type uint16_t = std::ffi::c_ushort;

    // acct

    pub const AFORK: u8 = 0x01;
    pub const ASU: u8 = 0x02;
    pub const ACOMPAT: u8 = 0x04;
    pub const ACORE: u8 = 0x08;
    pub const AXSIG: u8 = 0x10;

    pub const AC_FLAGS_MASK: u8 = AFORK | ASU | ACOMPAT | ACORE | AXSIG;

    pub const ACCT_COMM_SIZE: usize = 16;

    /// From [`/usr/include/sys/acct.h`] on NetBSD 9.3:
    ///
    /// ```C
    /// /*
    ///  * Accounting structures; these use a comp_t type which is a 3 bits base 8
    ///  * exponent, 13 bit fraction ``floating point'' number.  Units are 1/AHZ
    ///  * seconds.
    ///  */
    /// typedef uint16_t comp_t;
    /// 
    /// struct acct {
    ///         char      ac_comm[16];  /* command name */
    ///         comp_t    ac_utime;     /* user time */
    ///         comp_t    ac_stime;     /* system time */
    ///         comp_t    ac_etime;     /* elapsed time */
    ///         time_t    ac_btime;     /* starting time */
    ///         uid_t     ac_uid;       /* user id */
    ///         gid_t     ac_gid;       /* group id */
    ///         uint16_t  ac_mem;       /* average memory usage */
    ///         comp_t    ac_io;        /* count of IO blocks */
    ///         dev_t     ac_tty;       /* controlling tty */
    /// 
    /// #define AFORK   0x01            /* fork'd but not exec'd */
    /// #define ASU     0x02            /* used super-user permissions */
    /// #define ACOMPAT 0x04            /* used compatibility mode */
    /// #define ACORE   0x08            /* dumped core */
    /// #define AXSIG   0x10            /* killed by a signal */
    ///         uint8_t   ac_flag;      /* accounting flags */
    /// };
    ///
    /// /*
    ///  * 1/AHZ is the granularity of the data encoded in the comp_t fields.
    ///  * This is not necessarily equal to hz.
    ///  */
    /// #define AHZ     64
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// acct                 sizeof  56
    /// acct.ac_comm    @  0 sizeof  16
    /// acct.ac_utime   @ 16 sizeof   2
    /// acct.ac_stime   @ 18 sizeof   2
    /// acct.ac_etime   @ 20 sizeof   2
    /// acct.ac_btime   @ 24 sizeof   8
    /// acct.ac_uid     @ 32 sizeof   4
    /// acct.ac_gid     @ 36 sizeof   4
    /// acct.ac_mem     @ 40 sizeof   2
    /// acct.ac_io      @ 42 sizeof   2
    /// acct.ac_tty     @ 44 sizeof   8
    /// acct.ac_flag    @ 52 sizeof   1
    /// ```
    ///
    /// [`/usr/include/sys/acct.h`]: https://github.com/NetBSD/src/blob/1ba34bb0dc133c215a143601c18a24053c0e16e3/sys/sys/acct.h#L49
    #[derive(Clone, Copy)]
    #[repr(C, packed)]
    #[allow(non_camel_case_types)]
    pub struct acct {
        pub ac_comm: [c_char; ACCT_COMM_SIZE],
        pub ac_utime: comp_t,
        pub ac_stime: comp_t,
        pub ac_etime: comp_t,
        pub __gap1: [u8; 2],
        pub ac_btime: time_t,
        pub ac_uid: uid_t,
        pub ac_gid: gid_t,
        pub ac_mem: uint16_t,
        pub ac_io: comp_t,
        pub ac_tty: dev_t,
        pub ac_flag: uint8_t,
        pub __gap3: [u8; 3],
    }

    pub const ACCT_SZ: usize = size_of::<acct>();
    pub const ACCT_SZ_FO: FileOffset = ACCT_SZ as FileOffset;
    pub const ACCT_TIMEVALUE_OFFSET: usize = offset_of!(acct, ac_btime);
    pub const ACCT_TIMEVALUE_SZ: usize = size_of::<time_t>();
    assertcp_eq!(ACCT_SZ, 56);
    assertcp_eq!(offset_of!(acct, ac_comm), 0);
    assertcp_eq!(offset_of!(acct, ac_utime), 16);
    assertcp_eq!(offset_of!(acct, ac_stime), 18);
    assertcp_eq!(offset_of!(acct, ac_etime), 20);
    assertcp_eq!(offset_of!(acct, ac_btime), 24);
    assertcp_eq!(offset_of!(acct, ac_uid), 32);
    assertcp_eq!(offset_of!(acct, ac_gid), 36);
    assertcp_eq!(offset_of!(acct, ac_mem), 40);
    assertcp_eq!(offset_of!(acct, ac_io), 42);
    assertcp_eq!(offset_of!(acct, ac_tty), 44);
    assertcp_eq!(offset_of!(acct, ac_flag), 52);

    impl acct {
        pub fn ac_comm(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ac_comm.as_ptr()) }
        }
    }

    // timeval

    /// ```text
    /// timeval               sizeof  12
    /// timeval.tv_sec   @  0 sizeof   8
    /// timeval.tv_usec  @  8 sizeof   4
    /// ```
    #[derive(Clone, Copy)]
    // BUG: align(4) has no effect, must use `packed`
    //      see rust-lang/rust#48159 <https://github.com/rust-lang/rust/issues/48159>
    #[repr(C, packed)]
    #[allow(non_camel_case_types)]
    pub struct timeval {
        pub tv_sec: i64,
        pub tv_usec: i32,
    }

    pub const TIMEVAL_SZ: usize = size_of::<timeval>();
    assertcp_eq!(TIMEVAL_SZ, 12);
    assertcp_eq!(offset_of!(timeval, tv_sec), 0);
    assertcp_eq!(offset_of!(timeval, tv_usec), 8);

    // lastlog
    // same size as `linux_x86::lastlog`

    pub const UT_NAMESIZE: usize = 8;
    pub const UT_LINESIZE: usize = 8;
    pub const UT_HOSTSIZE: usize = 16;

    // lastlogx

    pub const UTX_LINESIZE: usize = 32;
    pub const UTX_HOSTSIZE: usize = 256;
    pub const UTX_SSSIZE: usize = 128;

    /// ```text
    /// lastlogx               sizeof 428
    /// lastlogx.ll_tv    @  0 sizeof  12
    /// lastlogx.ll_line  @ 12 sizeof  32
    /// lastlogx.ll_host  @ 44 sizeof 256
    /// lastlogx.ll_ss    @300 sizeof 128
    /// ```
    ///
    // TODO: [2024/03/10] This struct is 428 bytes whereas the scraped lastlogx
    //       file is 65536 bytes (not divisible by 428).
    //       see Issue #243 <https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/243>
    #[derive(Clone, Copy)]
    #[repr(C, packed)]
    #[allow(non_camel_case_types)]
    pub struct lastlogx {
        pub ll_tv: timeval,
        pub ll_line: [c_char; UTX_LINESIZE],
        pub ll_host: [c_char; UTX_HOSTSIZE],
        pub ll_ss: [u8; UTX_SSSIZE],
    }

    pub const LASTLOGX_SZ: usize = size_of::<lastlogx>();
    pub const LASTLOGX_SZ_FO: FileOffset = LASTLOGX_SZ as FileOffset;
    pub const LASTLOGX_TIMEVALUE_OFFSET: usize = offset_of!(lastlogx, ll_tv);
    pub const LASTLOGX_TIMEVALUE_SZ: usize = TIMEVAL_SZ;
    assertcp_eq!(LASTLOGX_SZ, 428);
    assertcp_eq!(offset_of!(lastlogx, ll_tv), 0);
    assertcp_eq!(offset_of!(lastlogx, ll_line), 12);
    assertcp_eq!(offset_of!(lastlogx, ll_host), 44);
    assertcp_eq!(offset_of!(lastlogx, ll_ss), 300);

    pub const PATH_LASTLOGX: &str = "/var/log/lastlogx";

    impl lastlogx {
        pub fn ll_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_line[..UTX_LINESIZE].as_ptr()) }
        }
        pub fn ll_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_host[..UTX_HOSTSIZE].as_ptr()) }
        }
    }

    // utmpx

    pub const UTX_USERSIZE: usize = 32;
    pub const UTX_IDSIZE: usize = 4;

    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct ut_exit {
        pub e_termination: uint16_t,
        pub e_exit: uint16_t,
    }

    assertcp_eq!(offset_of!(ut_exit, e_termination), 0);
    assertcp_eq!(offset_of!(ut_exit, e_exit), 2);

    /// ```text
    /// utmpx                   sizeof 516
    /// utmpx.ut_user      @  0 sizeof  32
    /// utmpx.ut_name      @  0 sizeof  32
    /// utmpx.ut_id        @ 32 sizeof   4
    /// utmpx.ut_line      @ 36 sizeof  32
    /// utmpx.ut_host      @ 68 sizeof 256
    /// utmpx.ut_session   @324 sizeof   2
    /// utmpx.ut_type      @326 sizeof   2
    /// utmpx.ut_pid       @328 sizeof   4
    /// utmpx.ut_exit      @332 sizeof   4
    /// utmpx.ut_ss        @336 sizeof 128
    /// utmpx.ut_xtime     @464 sizeof   8
    /// utmpx.ut_tv        @464 sizeof  12
    /// utmpx.ut_tv.tv_sec @464 sizeof   8
    /// utmpx.ut_tv.tv_usec@472 sizeof   4
    /// utmpx.ut_pad       @476 sizeof  40
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct utmpx {
        pub ut_name: [c_char; UTX_USERSIZE],
        pub ut_id: [c_char; UTX_IDSIZE],
        pub ut_line: [c_char; UTX_LINESIZE],
        pub ut_host: [c_char; UTX_HOSTSIZE],
        pub ut_session: uint16_t,
        pub ut_type: uint16_t,
        pub ut_pid: pid_t,
        pub ut_exit: ut_exit,
        pub ut_ss: [u8; UTX_SSSIZE],
        pub ut_tv: timeval,
        pub ut_pad: [c_char; 40],
    }

    pub const UTMPX_SZ: usize = size_of::<utmpx>();
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
    pub const UTMPX_TIMEVALUE_OFFSET: usize = offset_of!(utmpx, ut_tv);
    pub const UTMPX_TIMEVALUE_SZ: usize = TIMEVAL_SZ;
    assertcp_eq!(UTMPX_SZ, 516);
    assertcp_eq!(offset_of!(utmpx, ut_name), 0);
    assertcp_eq!(offset_of!(utmpx, ut_id), 32);
    assertcp_eq!(offset_of!(utmpx, ut_line), 36);
    assertcp_eq!(offset_of!(utmpx, ut_host), 68);
    assertcp_eq!(offset_of!(utmpx, ut_session), 324);
    assertcp_eq!(offset_of!(utmpx, ut_type), 326);
    assertcp_eq!(offset_of!(utmpx, ut_pid), 328);
    assertcp_eq!(offset_of!(utmpx, ut_exit), 332);
    assertcp_eq!(offset_of!(utmpx, ut_ss), 336);
    assertcp_eq!(offset_of!(utmpx, ut_tv), 464);
    assertcp_eq!(offset_of!(utmpx, ut_pad), 476);

    pub const PATH_UTMPX: &str = "/var/run/utmpx";
    pub const PATH_WTMPX: &str = "/var/log/wtmpx";

    /// Helpers for use in `fmt::Debug` trait.
    impl utmpx {
        pub fn ut_name(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_name[..UTX_USERSIZE].as_ptr()) }
        }
        pub fn ut_id(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_id[..UTX_IDSIZE].as_ptr()) }
        }
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line[..UTX_LINESIZE].as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host[..UTX_HOSTSIZE].as_ptr()) }
        }
    }

    /// From [`utmpx.h`] in NetBSD 9.3
    /// ```C
    /// #define EMPTY		0
    /// #define RUN_LVL		1
    /// #define BOOT_TIME	2
    /// #define OLD_TIME	3
    /// #define NEW_TIME	4
    /// #define INIT_PROCESS	5
    /// #define LOGIN_PROCESS	6
    /// #define USER_PROCESS	7
    /// #define DEAD_PROCESS	8
    ///
    /// #if defined(_NETBSD_SOURCE)
    /// #define ACCOUNTING	9
    /// #define SIGNATURE	10
    /// #define DOWN_TIME	11
    /// ```
    ///
    /// [`utmpx.h`]: https://github.com/NetBSD/src/blob/1ba34bb0dc133c215a143601c18a24053c0e16e3/include/utmpx.h
    pub const UT_TYPES: [uint16_t; 12] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 ,11
    ];
}

/// FixedStruct definitions found on NetBSD 9.3 amd64 (x86_64).
/// These are slightly different than i686 (x86_32).
#[allow(non_camel_case_types, unused)]
pub mod netbsd_x8664 {
    use crate::common::FileOffset;
    use std::ffi::CStr;
    use std::mem::size_of;
    use ::const_format::assertcp_eq;
    use ::memoffset::offset_of;

    pub type c_char = std::ffi::c_char;
    pub type pid_t = std::ffi::c_int;
    pub type time_t = std::ffi::c_longlong;
    pub type uint16_t = std::ffi::c_ushort;

    // timeval

    /// [from `time.h`]
    /// ```C
    /// struct timeval {
    /// 	time_t    	tv_sec;		/* seconds */
    /// 	suseconds_t	tv_usec;	/* and microseconds */
    /// };
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// timeval               sizeof  16
    /// timeval.tv_sec   @  0 sizeof   8
    /// timeval.tv_usec  @  8 sizeof   4
    /// ```
    ///
    /// [from `time.h`]: https://github.com/NetBSD/src/blob/1ba34bb0dc133c215a143601c18a24053c0e16e3/sys/sys/time.h
    #[derive(Clone, Copy)]
    #[repr(C, align(4))]
    #[allow(non_camel_case_types)]
    pub struct timeval {
        pub tv_sec: i64,
        pub tv_usec: i32,
        pub __pad: u32,
    }

    pub const TIMEVAL_SZ: usize = size_of::<timeval>();
    assertcp_eq!(TIMEVAL_SZ, 16);
    assertcp_eq!(offset_of!(timeval, tv_sec), 0);
    assertcp_eq!(offset_of!(timeval, tv_usec), 8);
    assertcp_eq!(offset_of!(timeval, __pad), 12);

    // lastlog

    pub const UT_NAMESIZE: usize = 8;
    pub const UT_LINESIZE: usize = 8;
    pub const UT_HOSTSIZE: usize = 16;

    /// [from `utmp.h`]
    /// ```C
    /// #define	UT_NAMESIZE	8
    /// #define	UT_LINESIZE	8
    /// #define	UT_HOSTSIZE	16
    ///
    /// struct lastlog {
    /// 	time_t	ll_time;
    /// 	char	ll_line[UT_LINESIZE];
    /// 	char	ll_host[UT_HOSTSIZE];
    /// };
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// lastlog               sizeof  32
    /// lastlog.ll_time  @  0 sizeof   8
    /// lastlog.ll_line  @  8 sizeof   8
    /// lastlog.ll_host  @ 16 sizeof  16
    /// ```
    ///
    /// Same struct and offsets were found on NetBSD 9.3 amd64 and i686.
    ///
    /// [from `utmp.h`]: https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmp.h
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct lastlog {
        pub ll_time: time_t,
        pub ll_line: [c_char; UT_LINESIZE],
        pub ll_host: [c_char; UT_HOSTSIZE],
    }

    pub const LASTLOG_SZ: usize = size_of::<lastlog>();
    pub const LASTLOG_SZ_FO: FileOffset = LASTLOG_SZ as FileOffset;
    pub const LASTLOG_TIMEVALUE_OFFSET: usize = offset_of!(lastlog, ll_time);
    pub const LASTLOG_TIMEVALUE_SZ: usize = size_of::<time_t>();
    assertcp_eq!(LASTLOG_SZ, 32);
    assertcp_eq!(offset_of!(lastlog, ll_time), 0);
    assertcp_eq!(offset_of!(lastlog, ll_line), 8);
    assertcp_eq!(offset_of!(lastlog, ll_host), 16);

    pub const PATH_LASTLOG: &str = "/var/log/lastlog";

    impl lastlog {
        pub fn ll_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_line[..UT_LINESIZE].as_ptr()) }
        }
        pub fn ll_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_host[..UT_HOSTSIZE].as_ptr()) }
        }
    }

    // lastlogx

    pub const UTX_LINESIZE: usize = 32;
    pub const UTX_HOSTSIZE: usize = 256;
    pub const UTX_SSSIZE: usize = 128;

    /// [from `utmpx.h`]
    /// ```C
    /// #define _UTX_USERSIZE	32
    /// #define _UTX_LINESIZE	32
    /// #define	_UTX_IDSIZE	4
    /// #define _UTX_HOSTSIZE	256
    ///
    /// #define ut_user ut_name
    /// #define ut_xtime ut_tv.tv_sec
    ///
    /// #if defined(_NETBSD_SOURCE)
    /// struct lastlogx {
    /// 	struct timeval ll_tv;		/* time entry was created */
    /// 	char ll_line[_UTX_LINESIZE];	/* tty name */
    /// 	char ll_host[_UTX_HOSTSIZE];	/* host name */
    /// 	struct sockaddr_storage ll_ss;	/* address where entry was made from */
    /// };
    /// #endif
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// lastlogx               sizeof 432
    /// lastlogx.ll_tv    @  0 sizeof  16
    /// lastlogx.ll_line  @ 16 sizeof  32
    /// lastlogx.ll_host  @ 48 sizeof 256
    /// lastlogx.ll_ss    @304 sizeof 128
    /// ```
    ///
    /// Also see [`man lastlogx`].
    ///
    /// [from `utmpx.h`]: https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmpx.h
    /// [`man lastlogx`]: https://man.netbsd.org/lastlogx.5
    // TODO: [2024/03/10] This struct is 432 bytes whereas lastlogx file is
    //       65536 bytes (not divisble by 432).
    //       see Issue #243 <https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/243>
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct lastlogx {
        pub ll_tv: timeval,
        pub ll_line: [c_char; UTX_LINESIZE],
        pub ll_host: [c_char; UTX_HOSTSIZE],
        pub ll_ss: [u8; UTX_SSSIZE],
    }

    pub const LASTLOGX_SZ: usize = size_of::<lastlogx>();
    pub const LASTLOGX_SZ_FO: FileOffset = LASTLOG_SZ as FileOffset;
    pub const LASTLOGX_TIMEVALUE_OFFSET: usize = offset_of!(lastlogx, ll_tv);
    pub const LASTLOGX_TIMEVALUE_SZ: usize = TIMEVAL_SZ;
    assertcp_eq!(LASTLOGX_SZ, 432);
    assertcp_eq!(offset_of!(lastlogx, ll_tv), 0);
    assertcp_eq!(offset_of!(lastlogx, ll_line), 16);
    assertcp_eq!(offset_of!(lastlogx, ll_host), 48);
    assertcp_eq!(offset_of!(lastlogx, ll_ss), 304);

    pub const PATH_LASTLOGX: &str = "/var/log/lastlogx";

    impl lastlogx {
        pub fn ll_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_line[..UTX_LINESIZE].as_ptr()) }
        }
        pub fn ll_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_host[..UTX_HOSTSIZE].as_ptr()) }
        }
    }

    // utmp

    /// [from `utmp.h`]
    /// ```C
    /// #define	UT_NAMESIZE	8
    /// #define	UT_LINESIZE	8
    /// #define	UT_HOSTSIZE	16
    ///
    /// struct utmp {
    /// 	char	ut_line[UT_LINESIZE];
    /// 	char	ut_name[UT_NAMESIZE];
    /// 	char	ut_host[UT_HOSTSIZE];
    /// 	time_t	ut_time;
    /// };
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// utmp                   sizeof  40
    /// utmp.ut_line      @  0 sizeof   8
    /// utmp.ut_name      @  8 sizeof   8
    /// utmp.ut_host      @ 16 sizeof  16
    /// utmp.ut_time      @ 32 sizeof   8
    /// ```
    ///
    /// Same struct and offsets were found on NetBSD 9.3 amd64 and i686.
    ///
    /// [from `utmp.h`]: https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmp.h
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct utmp {
        pub ut_line: [c_char; UT_LINESIZE],
        pub ut_name: [c_char; UT_NAMESIZE],
        pub ut_host: [c_char; UT_HOSTSIZE],
        pub ut_time: time_t,
    }

    pub const UTMP_SZ: usize = size_of::<utmp>();
    pub const UTMP_SZ_FO: FileOffset = UTMP_SZ as FileOffset;
    pub const UTMP_TIMEVALUE_OFFSET: usize = offset_of!(utmp, ut_time);
    pub const UTMP_TIMEVALUE_SZ: usize = size_of::<time_t>();
    assertcp_eq!(UTMP_SZ, 40);
    assertcp_eq!(offset_of!(utmp, ut_line), 0);
    assertcp_eq!(offset_of!(utmp, ut_name), 8);
    assertcp_eq!(offset_of!(utmp, ut_host), 16);
    assertcp_eq!(offset_of!(utmp, ut_time), 32);

    pub const PATH_UTMP: &str = "/var/run/utmp";
    pub const PATH_WTMP: &str = "/var/log/wtmp";

    impl utmp {
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line[..UT_LINESIZE].as_ptr()) }
        }
        pub fn ut_name(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_name[..UT_NAMESIZE].as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host[..UT_HOSTSIZE].as_ptr()) }
        }
    }

    // utmpx

    pub const UTX_USERSIZE: usize = 32;
    pub const UTX_IDSIZE: usize = 4;

    #[derive(Clone, Copy)]
    #[repr(C, align(2))]
    #[allow(non_camel_case_types)]
    pub struct ut_exit {
        pub e_termination: uint16_t,
        pub e_exit: uint16_t,
    }

    pub const UT_EXIT_SZ: usize = size_of::<ut_exit>();
    assertcp_eq!(UT_EXIT_SZ, 4);
    assertcp_eq!(offset_of!(ut_exit, e_termination), 0);
    assertcp_eq!(offset_of!(ut_exit, e_exit), 2);

    /// [from `utmpx.h`]
    /// ```C
    /// #define _UTX_USERSIZE	32
    /// #define _UTX_LINESIZE	32
    /// #define	_UTX_IDSIZE	4
    /// #define _UTX_HOSTSIZE	256
    ///
    /// #define ut_user ut_name
    /// #define ut_xtime ut_tv.tv_sec
    ///
    /// struct utmpx {
    /// 	char ut_name[_UTX_USERSIZE];	/* login name */
    /// 	char ut_id[_UTX_IDSIZE];	/* inittab id */
    /// 	char ut_line[_UTX_LINESIZE];	/* tty name */
    /// 	char ut_host[_UTX_HOSTSIZE];	/* host name */
    /// 	uint16_t ut_session;		/* session id used for windowing */
    /// 	uint16_t ut_type;		/* type of this entry */
    /// 	pid_t ut_pid;			/* process id creating the entry */
    /// 	struct {
    /// 		uint16_t e_termination;	/* process termination signal */
    /// 		uint16_t e_exit;	/* process exit status */
    /// 	} ut_exit;
    /// 	struct sockaddr_storage ut_ss;	/* address where entry was made from */
    /// 	struct timeval ut_tv;		/* time entry was created */
    /// 	uint8_t ut_pad[_UTX_PADSIZE];	/* reserved for future use */
    /// };
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// EMPTY 0
    /// RUN_LVL 1
    /// BOOT_TIME 2
    /// OLD_TIME 3
    /// NEW_TIME 4
    /// INIT_PROCESS 5
    /// LOGIN_PROCESS 6
    /// USER_PROCESS 7
    /// DEAD_PROCESS 8
    /// ACCOUNTING 9
    /// SIGNATURE 10
    /// DOWN_TIME 11
    ///
    /// UTX_USERSIZE 32
    /// UTX_LINESIZE 32
    /// UTX_IDSIZE 4
    /// UTX_HOSTSIZE 256
    ///
    /// utmpx                   sizeof 520
    /// utmpx.ut_user      @  0 sizeof  32
    /// utmpx.ut_name      @  0 sizeof  32
    /// utmpx.ut_id        @ 32 sizeof   4
    /// utmpx.ut_line      @ 36 sizeof  32
    /// utmpx.ut_host      @ 68 sizeof 256
    /// utmpx.ut_session   @324 sizeof   2
    /// utmpx.ut_type      @326 sizeof   2
    /// utmpx.ut_pid       @328 sizeof   4
    /// utmpx.ut_exit      @332 sizeof   4
    /// utmpx.ut_xtime     @464 sizeof   8
    /// utmpx.ut_tv        @464 sizeof  16
    /// utmpx.ut_tv.tv_sec @464 sizeof   8
    /// utmpx.ut_tv.tv_usec@472 sizeof   4
    /// utmpx.ut_pad       @480 sizeof  36
    /// ```
    ///
    /// [from `utmpx.h`]: https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmpx.h
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct utmpx {
        pub ut_user: [c_char; UTX_USERSIZE],
        pub ut_id: [c_char; UTX_IDSIZE],
        pub ut_line: [c_char; UTX_LINESIZE],
        pub ut_host: [c_char; UTX_HOSTSIZE],
        pub ut_session: uint16_t,
        pub ut_type: uint16_t,
        pub ut_pid: pid_t,
        pub ut_exit: ut_exit,
        pub __gap1: [c_char; 128],
        pub ut_tv: timeval,
        pub ut_pad: [c_char; 36],
    }

    pub const UTMPX_SZ: usize = size_of::<utmpx>();
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
    pub const UTMPX_TIMEVALUE_OFFSET: usize = offset_of!(utmpx, ut_tv);
    pub const UTMPX_TIMEVALUE_SZ: usize = TIMEVAL_SZ;
    assertcp_eq!(UTMPX_SZ, 520);
    assertcp_eq!(offset_of!(utmpx, ut_user), 0);
    assertcp_eq!(offset_of!(utmpx, ut_id), 32);
    assertcp_eq!(offset_of!(utmpx, ut_line), 36);
    assertcp_eq!(offset_of!(utmpx, ut_host), 68);
    assertcp_eq!(offset_of!(utmpx, ut_session), 324);
    assertcp_eq!(offset_of!(utmpx, ut_type), 326);
    assertcp_eq!(offset_of!(utmpx, ut_pid), 328);
    assertcp_eq!(offset_of!(utmpx, ut_exit), 332);
    assertcp_eq!(offset_of!(utmpx, ut_tv), 464);
    assertcp_eq!(offset_of!(utmpx, ut_pad), 480);

    pub const PATH_UTMPX: &str = "/var/run/utmpx";
    pub const PATH_WTMPX: &str = "/var/log/wtmpx";

    /// Helpers for use in `fmt::Debug` trait.
    impl utmpx {
        pub fn ut_user(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_user[..UTX_USERSIZE].as_ptr()) }
        }
        pub fn ut_id(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_id[..UTX_IDSIZE].as_ptr()) }
        }
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line[..UTX_LINESIZE].as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host[..UTX_HOSTSIZE].as_ptr()) }
        }
    }

    /// From [`utmpx.h`] in NetBSD 9.3
    /// ```C
    /// #define EMPTY		0
    /// #define RUN_LVL		1
    /// #define BOOT_TIME	2
    /// #define OLD_TIME	3
    /// #define NEW_TIME	4
    /// #define INIT_PROCESS	5
    /// #define LOGIN_PROCESS	6
    /// #define USER_PROCESS	7
    /// #define DEAD_PROCESS	8
    ///
    /// #if defined(_NETBSD_SOURCE)
    /// #define ACCOUNTING	9
    /// #define SIGNATURE	10
    /// #define DOWN_TIME	11
    /// ```
    ///
    /// [`utmpx.h`]: https://github.com/NetBSD/src/blob/1ba34bb0dc133c215a143601c18a24053c0e16e3/include/utmpx.h
    pub const UT_TYPES: [uint16_t; 12] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 ,11
    ];
}

/// FixedStruct definitions found on OpenBSD 7.2 i386 (x86_32) and amd64
/// (x86_64).
///
/// The same sizes were found for OpenBSD on i386 and amd64; see repository file
/// `logs/OpenBSD7.4/x86_64/utmp-offsets_amd64_OpenBSD_7.4_.out`.
///
/// See [the 7.2 OpenBSD code base].
/// See [`man utmp`].
/// See [`utmp.h` from the OpenBSD 7.2 release].
///
/// [the 7.2 OpenBSD code base]: https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/include/utmp.h
/// [`man utmp`]: https://web.archive.org/web/20230607124838/https://man.openbsd.org/utmp.5
/// [`utmp.h` from the OpenBSD 7.2 release]: https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/include/utmp.h#L56-L67
#[allow(non_camel_case_types, unused)]
pub mod openbsd_x86 {
    use crate::common::FileOffset;
    use std::ffi::CStr;
    use std::mem::size_of;
    use ::const_format::assertcp_eq;
    use ::memoffset::offset_of;

    pub type c_char = std::ffi::c_char;
    pub type time_t = std::ffi::c_longlong;

    pub const UT_NAMESIZE: usize = 32;
    pub const UT_LINESIZE: usize = 8;
    pub const UT_HOSTSIZE: usize = 256;

    // lastlog

    /// From <https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/include/utmp.h#L56C1-L61C1>
    ///
    /// ```C
    /// #define	UT_NAMESIZE	32
    /// #define	UT_LINESIZE	8
    /// #define	UT_HOSTSIZE	256
    ///
    /// struct lastlog {
    /// 	time_t	ll_time;
    /// 	char	ll_line[UT_LINESIZE];
    /// 	char	ll_host[UT_HOSTSIZE];
    /// };
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// lastlog               sizeof 272
    /// lastlog.ll_time  @  0 sizeof   8
    /// lastlog.ll_line  @  8 sizeof   8
    /// lastlog.ll_host  @ 16 sizeof 256
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct lastlog {
        pub ll_time: time_t,
        pub ll_line: [c_char; UT_LINESIZE],
        pub ll_host: [c_char; UT_HOSTSIZE],
    }

    pub const LASTLOG_SZ: usize = size_of::<lastlog>();
    pub const LASTLOG_SZ_FO: FileOffset = LASTLOG_SZ as FileOffset;
    pub const LASTLOG_TIMEVALUE_OFFSET: usize = offset_of!(lastlog, ll_time);
    pub const LASTLOG_TIMEVALUE_SZ: usize = size_of::<time_t>();
    assertcp_eq!(LASTLOG_SZ, 272);
    assertcp_eq!(offset_of!(lastlog, ll_time), 0);
    assertcp_eq!(offset_of!(lastlog, ll_line), 8);
    assertcp_eq!(offset_of!(lastlog, ll_host), 16);

    pub const PATH_LASTLOG: &str = "/var/log/lastlog";

    /// Helpers for use in `fmt::Debug` trait.
    ///
    /// The slicing in each `CStr` function below is to due to:
    /// ```C
    /// /*
    ///  * Note that these are *not* C strings and thus are not
    ///  * guaranteed to be NUL-terminated.
    /// */
    /// ```
    /// <https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/include/utmp.h#L51-L54>
    ///
    /// to avoid the `CStr` constructor from reading past the end of the given
    /// field into the next field.
    impl lastlog {
        pub fn ll_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_line[..UT_LINESIZE].as_ptr()) }
        }
        pub fn ll_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ll_host[..UT_HOSTSIZE].as_ptr()) }
        }
    }

    // utmp

    /// From <https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/include/utmp.h#L62-L67>
    ///
    /// ```C
    /// #define	UT_NAMESIZE	32
    /// #define	UT_LINESIZE	8
    /// #define	UT_HOSTSIZE	256
    ///
    /// struct utmp {
    /// 	char	ut_line[UT_LINESIZE];
    /// 	char	ut_name[UT_NAMESIZE];
    /// 	char	ut_host[UT_HOSTSIZE];
    /// 	time_t	ut_time;
    /// };
    /// ```
    ///
    /// ---
    ///
    /// ```text
    /// utmp                   sizeof 304
    /// utmp.ut_line      @  0 sizeof   8
    /// utmp.ut_name      @  8 sizeof  32
    /// utmp.ut_host      @ 40 sizeof 256
    /// utmp.ut_time      @296 sizeof   8
    /// ```
    #[derive(Clone, Copy)]
    #[repr(C, align(8))]
    #[allow(non_camel_case_types)]
    pub struct utmp {
        pub ut_line: [c_char; UT_LINESIZE],
        pub ut_name: [c_char; UT_NAMESIZE],
        pub ut_host: [c_char; UT_HOSTSIZE],
        pub ut_time: time_t,
    }

    pub const UTMP_SZ: usize = size_of::<utmp>();
    pub const UTMP_SZ_FO: FileOffset = UTMP_SZ as FileOffset;
    pub const UTMP_TIMEVALUE_OFFSET: usize = offset_of!(utmp, ut_time);
    pub const UTMP_TIMEVALUE_SZ: usize = size_of::<time_t>();
    assertcp_eq!(UTMP_SZ, 304);
    assertcp_eq!(offset_of!(utmp, ut_line), 0);
    assertcp_eq!(offset_of!(utmp, ut_name), 8);
    assertcp_eq!(offset_of!(utmp, ut_host), 40);
    assertcp_eq!(offset_of!(utmp, ut_time), 296);

    pub const PATH_UTMP: &str = "/var/run/utmp";
    pub const PATH_WTMP: &str = "/var/log/wtmp";

    /// Helpers for use in `fmt::Debug` trait.
    ///
    /// The slicing in each `CStr` function below is to due to:
    /// ```C
    /// /*
    ///  * Note that these are *not* C strings and thus are not
    ///  * guaranteed to be NUL-terminated.
    /// */
    /// ```
    /// <https://github.com/openbsd/src/blob/20248fc4cbb7c0efca41a8aafd40db7747023515/include/utmp.h#L51-L54>
    ///
    /// to avoid the `CStr` constructor from reading past the end of the given
    /// field into the next field.
    impl utmp {
        pub fn ut_line(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_line[..UT_LINESIZE].as_ptr()) }
        }
        pub fn ut_name(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_name[..UT_NAMESIZE].as_ptr()) }
        }
        pub fn ut_host(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.ut_host[..UT_HOSTSIZE].as_ptr()) }
        }
    }
}

/// Maximum size among all `acct`/`lastlog`/`utmp`/etc. C structs
pub const ENTRY_SZ_MAX: usize = max16(
    freebsd_x8664::UTMPX_SZ,
    linux_arm64aarch64::LASTLOG_SZ,
    linux_arm64aarch64::UTMPX_SZ,
    linux_x86::ACCT_SZ,
    linux_x86::ACCT_V3_SZ,
    linux_x86::LASTLOG_SZ,
    linux_x86::UTMPX_SZ,
    netbsd_x8632::ACCT_SZ,
    netbsd_x8632::LASTLOGX_SZ,
    netbsd_x8632::UTMPX_SZ,
    netbsd_x8664::LASTLOG_SZ,
    netbsd_x8664::LASTLOGX_SZ,
    netbsd_x8664::UTMP_SZ,
    netbsd_x8664::UTMPX_SZ,
    openbsd_x86::LASTLOG_SZ,
    openbsd_x86::UTMP_SZ,
);

/// Minimum size among all `acct`/`lastlog`/`utmp`/etc. C structs
pub const ENTRY_SZ_MIN: usize = min16(
    freebsd_x8664::UTMPX_SZ,
    linux_arm64aarch64::LASTLOG_SZ,
    linux_arm64aarch64::UTMPX_SZ,
    linux_x86::ACCT_SZ,
    linux_x86::ACCT_V3_SZ,
    linux_x86::LASTLOG_SZ,
    linux_x86::UTMPX_SZ,
    netbsd_x8632::ACCT_SZ,
    netbsd_x8632::LASTLOGX_SZ,
    netbsd_x8632::UTMPX_SZ,
    netbsd_x8664::LASTLOG_SZ,
    netbsd_x8664::LASTLOGX_SZ,
    netbsd_x8664::UTMP_SZ,
    netbsd_x8664::UTMPX_SZ,
    openbsd_x86::LASTLOG_SZ,
    openbsd_x86::UTMP_SZ,
);

/// Maximum size among all time values for all C structs
pub const TIMEVAL_SZ_MAX: usize = max16(
    freebsd_x8664::TIMEVAL_SZ,
    linux_arm64aarch64::LASTLOG_TIMEVALUE_SZ,
    linux_arm64aarch64::UTMPX_TIMEVALUE_SZ,
    linux_x86::ACCT_TIMEVALUE_SZ,
    linux_x86::ACCT_V3_TIMEVALUE_SZ,
    linux_x86::LASTLOG_TIMEVALUE_SZ,
    linux_x86::UTMPX_TIMEVALUE_SZ,
    netbsd_x8632::ACCT_TIMEVALUE_SZ,
    netbsd_x8632::LASTLOGX_TIMEVALUE_SZ,
    netbsd_x8632::UTMPX_TIMEVALUE_SZ,
    netbsd_x8664::LASTLOG_TIMEVALUE_SZ,
    netbsd_x8664::LASTLOGX_TIMEVALUE_SZ,
    netbsd_x8664::UTMP_TIMEVALUE_SZ,
    netbsd_x8664::UTMPX_TIMEVALUE_SZ,
    openbsd_x86::LASTLOG_TIMEVALUE_SZ,
    openbsd_x86::UTMP_TIMEVALUE_SZ,
);

/// Map [`utmp.ut_type`] value, implied in the index offset, to it's `str`
/// representation. These values and definitions appear consistent across all
/// platforms, except NetBSD appends three values.
///
/// See [NetBSD 9.3 `utmpx.h`].
///
/// [`utmp.ut_type`]: https://www.man7.org/linux/man-pages/man5/utmp.5.html
/// [NetBSD 9.3 `utmpx.h`]: https://github.com/NetBSD/src/blob/0d57c6f2979b7cf98608ef9ddbf6f739da0f8b42/include/utmpx.h
pub const UT_TYPE_VAL_TO_STR: &[&str] = &[
    "EMPTY", // 0
    "RUN_LVL", // 1
    "BOOT_TIME", // 2
    "NEW_TIME", // 3
    "OLD_TIME", // 4
    "INIT_PROCESS", // 5
    "LOGIN_PROCESS", // 6
    "USER_PROCESS", // 7
    "DEAD_PROCESS", // 8
    // NetBSD adds these values
    "ACCOUNTING", // 9
    "SIGNATURE", // 10
    "DOWN_TIME", // 11
];
/// Count of entries in [`UT_TYPE_VAL_TO_STR`].
pub const UT_TYPE_VAL_TO_STR_LEN: usize = UT_TYPE_VAL_TO_STR.len();
#[allow(non_upper_case_globals)]
pub const UT_TYPE_VAL_TO_STR_LEN_i16: i16 = UT_TYPE_VAL_TO_STR.len() as i16;
#[allow(non_upper_case_globals)]
pub const UT_TYPE_VAL_TO_STR_LEN_u16: u16 = UT_TYPE_VAL_TO_STR.len() as u16;

/// common denominator **t**ime **v**alue type representing
/// seconds since Unix epoch
#[allow(non_camel_case_types)]
pub type tv_sec_type = i64;

/// common denominator **t**ime **v**alue type representing additional
/// sub-second microseconds since Unix epoch
#[allow(non_camel_case_types)]
pub type tv_usec_type = i64;

/// common nanoseconds type used as intermediate representation during
/// conversion to [`DateTimeL`]
///
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
#[allow(non_camel_case_types)]
pub type nsecs_type = u32;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// **t**ime **v**alue pair type
pub struct tv_pair_type(pub tv_sec_type, pub tv_usec_type);

impl std::fmt::Debug for tv_pair_type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

/// `Box` pointer to a `dyn`amically dispatched _Trait object_
/// [`FixedStructTrait`].
pub type FixedStructDynPtr = Box<dyn FixedStructTrait>;

/// An abstraction for representing varying fixed-size C structs.
/// This includes record-keeping structs _acct_, _lastlog_, _utmp_ defined
/// in the namespaces [`freebsd_x8664`], [`linux_arm64aarch64`], etc.
/// Each specific definition defined in those namespaces must `impl`ement this
/// trait.
///
/// Because this uses `dyn` trait then it must be an "Object safe trait".
/// Being "Object safe trait" enforces limitations on behavior, e.g. cannot
/// require trait `Sized` which implies cannot require trait `Clone`, among
/// other limitations.
///
/// References to specific implementations of this Trait are stored as a Box
/// pointer to the trait object, `Box<dyn FixedStructTrait>`, which is aliased
/// as [`FixedStructDynPtr`]. This allows dynamic dispatching of the at runtime.
///
/// `Send` required for sending from file processing thread to main thread
///
/// `std::marker::Sync` required for `lazy_static!`
///
/// See <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
pub trait FixedStructTrait
where Self: Send,
      Self: std::marker::Sync,
{
    /// the type of the struct
    fn fixedstruct_type(&self) -> FixedStructType;
    /// the size of the struct in bytes
    fn size(&self) -> usize;

    // TODO: [2024/01/28] is there a more rustic way to combine these into one
    //       `as_fixedstruct_type` that is just real smart? so I don't have to repeat
    //       the same function names in each `impl` block, i.e. each impl block
    //       only explicitly defines a few of the functions, the rest panic.
    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx;
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog;
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx;
    fn as_linux_x86_acct(&self) -> &linux_x86::acct;
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3;
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog;
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx;
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct;
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx;
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx;
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog;
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx;
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp;
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx;
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog;
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp;
}

impl fmt::Debug for dyn FixedStructTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.fixedstruct_type() {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => {
                self.as_freebsd_x8664_utmpx().fmt(f)
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
                self.as_linux_arm64aarch64_lastlog().fmt(f)
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
                self.as_linux_arm64aarch64_utmpx().fmt(f)
            }
            FixedStructType::Fs_Linux_x86_Acct => {
                self.as_linux_x86_acct().fmt(f)
            }
            FixedStructType::Fs_Linux_x86_Acct_v3 => {
                self.as_linux_x86_acct_v3().fmt(f)
            }
            FixedStructType::Fs_Linux_x86_Lastlog => {
                self.as_linux_x86_lastlog().fmt(f)
            }
            FixedStructType::Fs_Linux_x86_Utmpx => {
                self.as_linux_x86_utmpx().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8632_Acct => {
                self.as_netbsd_x8632_acct().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
                self.as_netbsd_x8632_lastlogx().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8632_Utmpx => {
                self.as_netbsd_x8632_utmpx().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlog => {
                self.as_netbsd_x8664_lastlog().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
                self.as_netbsd_x8664_lastlogx().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8664_Utmp => {
                self.as_netbsd_x8664_utmp().fmt(f)
            }
            FixedStructType::Fs_Netbsd_x8664_Utmpx => {
                self.as_netbsd_x8664_utmpx().fmt(f)
            }
            FixedStructType::Fs_Openbsd_x86_Lastlog => {
                self.as_openbsd_x86_lastlog().fmt(f)
            }
            FixedStructType::Fs_Openbsd_x86_Utmp => {
                self.as_openbsd_x86_utmp().fmt(f)
            }
        }
    }
}

// freebsd_x8664::utmpx

impl FixedStructTrait for freebsd_x8664::utmpx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Freebsd_x8664_Utmpx
    }
    fn size(&self) -> usize {
        freebsd_x8664::UTMPX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        self
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on freebsd_x8664::utmpx");
    }
}

impl fmt::Debug for freebsd_x8664::utmpx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ut_pid_s = match self.ut_type {
            x if (0..UT_TYPE_VAL_TO_STR_LEN_i16).contains(&x) =>
                format!("{} ({})", self.ut_type, UT_TYPE_VAL_TO_STR[self.ut_type as usize]),
            _ => format!("{} (UNKNOWN)", self.ut_type),
        };
        f.debug_struct("freebsd_x8664::utmpx")
            .field("size", &self.size())
            .field("ut_type", &self.ut_type)
            .field("ut_pid", &format_args!("{}", ut_pid_s))
            .field("ut_line", &self.ut_line())
            .field("ut_id", &self.ut_id())
            .field("ut_user", &self.ut_user())
            .field("ut_host", &self.ut_host())
            .field("ut_tv.tv_sec", &self.ut_tv.tv_sec)
            .field("ut_tv.tv_usec", &self.ut_tv.tv_usec)
            .finish()
    }
}

// linux_arm64aarch64::lastlog

impl FixedStructTrait for linux_arm64aarch64::lastlog {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog
    }
    fn size(&self) -> usize {
        linux_arm64aarch64::LASTLOG_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        self
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on freebsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on freebsd_x8664::utmpx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on freebsd_x8664::utmpx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on freebsd_x8664::utmpx");
    }
}

impl fmt::Debug for linux_arm64aarch64::lastlog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("linux_arm64aarch64::lastlog")
            .field("size", &self.size())
            .field("ll_time", &self.ll_time)
            .field("ll_line", &self.ll_line())
            .field("ll_host", &self.ll_host())
            .finish()
    }
}

// linux_arm64aarch64::utmpx

impl FixedStructTrait for linux_arm64aarch64::utmpx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx
    }
    fn size(&self) -> usize {
        linux_arm64aarch64::UTMPX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on linux_arm64aarch64::utmpx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on linux_arm64aarch64::utmpx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        self
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on linux_arm64aarch64::utmpx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on linux_arm64aarch64::utmpx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on linux_arm64aarch64::utmpx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on linux_arm64aarch64::utmpx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on linux_arm64aarch64::utmpx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on linux_arm64aarch64::utmpx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on linux_arm64aarch64::utmpx");
    }
}

impl fmt::Debug for linux_arm64aarch64::utmpx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("linux_arm64aarch64::utmpx")
            .field("size", &self.size())
            .field("ut_type", &self.ut_type)
            .field("ut_pid", &self.ut_pid)
            .field("ut_line", &self.ut_line())
            .field("ut_id", &self.ut_id())
            .field("ut_user", &self.ut_user())
            .field("ut_host", &self.ut_host())
            .field("ut_tv.tv_sec", &self.ut_tv.tv_sec)
            .field("ut_tv.tv_usec", &self.ut_tv.tv_usec)
            .field("ut_addr_v6", &self.ut_addr_v6)
            .finish()
    }
}

// linux_x86::acct

impl FixedStructTrait for linux_x86::acct {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Linux_x86_Acct
    }
    fn size(&self) -> usize {
        linux_x86::ACCT_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on linux_x86::acct");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on linux_x86::acct");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on linux_x86::acct");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        self
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on linux_x86::acct");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on linux_x86::acct");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on linux_x86::acct");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on linux_x86::acct");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on linux_x86::acct");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on linux_x86::acct");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on linux_x86::acct");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on linux_x86::acct");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on linux_x86::acct");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on linux_x86::acct");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on linux_x86::acct");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on linux_x86::acct");
    }
}

impl fmt::Debug for linux_x86::acct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("linux_x86::acct")
            .field("size", &self.size())
            .field("ac_flag", &self.ac_flag)
            .field("ac_uid", &self.ac_uid)
            .field("ac_gid", &self.ac_gid)
            .field("ac_tty", &self.ac_tty)
            .field("ac_btime", &self.ac_btime)
            .field("ac_utime", &self.ac_utime)
            .field("ac_stime", &self.ac_stime)
            .field("ac_etime", &self.ac_etime)
            .field("ac_mem", &self.ac_mem)
            .field("ac_io", &self.ac_io)
            .field("ac_rw", &self.ac_rw)
            .field("ac_minflt", &self.ac_minflt)
            .field("ac_majflt", &self.ac_majflt)
            .field("ac_swaps", &self.ac_swaps)
            .field("ac_exitcode", &self.ac_exitcode)
            .field("ac_comm", &self.ac_comm())
            .finish()
    }
}

// linux_x86::acct_v3

impl FixedStructTrait for linux_x86::acct_v3 {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Linux_x86_Acct_v3
    }
    fn size(&self) -> usize {
        linux_x86::ACCT_V3_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on linux_x86::acct_v3");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on linux_x86::acct_v3");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on linux_x86::acct_v3");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on linux_x86::acct_v3");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        self
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on linux_x86::acct_v3");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on linux_x86::acct_v3");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on linux_x86::acct_v3");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on linux_x86::acct_v3");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on linux_x86::acct_v3");
    }
}

impl fmt::Debug for linux_x86::acct_v3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("linux_x86::acct_v3")
            .field("size", &self.size())
            .field("ac_flag", &self.ac_flag)
            .field("ac_tty", &self.ac_tty)
            .field("ac_exitcode", &self.ac_exitcode)
            .field("ac_uid", &self.ac_uid)
            .field("ac_gid", &self.ac_gid)
            .field("ac_pid", &self.ac_pid)
            .field("ac_ppid", &self.ac_ppid)
            .field("ac_btime", &self.ac_btime)
            .field("ac_etime", &self.ac_etime)
            .field("ac_utime", &self.ac_utime)
            .field("ac_stime", &self.ac_stime)
            .field("ac_mem", &self.ac_mem)
            .field("ac_io", &self.ac_io)
            .field("ac_rw", &self.ac_rw)
            .field("ac_minflt", &self.ac_minflt)
            .field("ac_majflt", &self.ac_majflt)
            .field("ac_swaps", &self.ac_swaps)
            .field("ac_comm", &self.ac_comm())
            .finish()
    }
}

// linux_x86::lastlog

impl FixedStructTrait for linux_x86::lastlog {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Linux_x86_Lastlog
    }
    fn size(&self) -> usize {
        linux_x86::LASTLOG_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on linux_x86::lastlog");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on linux_x86::lastlog");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on linux_x86::lastlog");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on linux_x86::lastlog");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on linux_x86::lastlog");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        self
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on linux_x86::lastlog");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on linux_x86::lastlog");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on linux_x86::lastlog");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on linux_x86::lastlog");
    }
}

impl fmt::Debug for linux_x86::lastlog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("linux_x86::lastlog")
            .field("size", &self.size())
            .field("ll_time", &self.ll_time)
            .field("ll_line", &self.ll_line())
            .field("ll_host", &self.ll_host())
            .finish()
    }
}

// linux_x86::utmpx

impl FixedStructTrait for linux_x86::utmpx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Linux_x86_Utmpx
    }
    fn size(&self) -> usize {
        linux_x86::UTMPX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on linux_x86::utmpx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on linux_x86::utmpx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on linux_x86::utmpx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on linux_x86::utmpx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on linux_x86::utmpx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on linux_x86::utmpx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        self
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on linux_x86::utmpx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on linux_x86::utmpx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on linux_x86::utmpx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on linux_x86::utmpx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on linux_x86::utmpx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on linux_x86::utmpx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on linux_x86::utmpx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on linux_x86::utmpx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on linux_x86::utmpx");
    }
}

impl fmt::Debug for linux_x86::utmpx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ut_pid_s = match self.ut_type {
            x if x < UT_TYPE_VAL_TO_STR_LEN_i16 && x >= 0 => format!("{} ({})", self.ut_type, UT_TYPE_VAL_TO_STR[self.ut_type as usize]),
            _ => format!("{} (UNKNOWN)", self.ut_type),
        };
        f.debug_struct("linux_x86::utmpx")
            .field("size", &self.size())
            .field("ut_type", &self.ut_type)
            .field("ut_pid", &format_args!("{}", ut_pid_s))
            .field("ut_line", &self.ut_line())
            .field("ut_id", &self.ut_id())
            .field("ut_user", &self.ut_user())
            .field("ut_host", &self.ut_host())
            .field("ut_exit_e_termination", &self.ut_exit.e_termination)
            .field("ut_exit_e_exit", &self.ut_exit.e_exit)
            .field("ut_session", &self.ut_session)
            .field("ut_tv.tv_sec", &self.ut_tv.tv_sec)
            .field("ut_tv.tv_usec", &self.ut_tv.tv_usec)
            .finish()
    }
}

// netbsd_x8632::acct

impl FixedStructTrait for netbsd_x8632::acct {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8632_Acct
    }
    fn size(&self) -> usize {
        netbsd_x8632::ACCT_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8632::lastlog");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8632::lastlog");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8632::lastlog");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8632::lastlog");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8632::lastlog");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8632::lastlog");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8632::lastlog");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        self
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on netbsd_x8632::lastlog");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on netbsd_x8632::lastlog");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on netbsd_x8632::lastlog");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on netbsd_x8632::lastlog");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on netbsd_x8632::lastlog");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on netbsd_x8632::lastlog");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8632::lastlog");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8632::lastlog");
    }
}

impl fmt::Debug for netbsd_x8632::acct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ac_utime = self.ac_utime;
        let ac_stime = self.ac_stime;
        let ac_etime = self.ac_etime;
        let ac_btime = self.ac_btime;
        let ac_uid = self.ac_uid;
        let ac_gid = self.ac_gid;
        let ac_mem = self.ac_mem;
        let ac_io = self.ac_io;
        let ac_tty = self.ac_tty;
        let ac_flag = self.ac_flag;
        f.debug_struct("netbsd_x8632::acct")
            .field("size", &self.size())
            .field("ac_comm", &self.ac_comm())
            .field("ac_utime", &ac_utime)
            .field("ac_stime", &ac_stime)
            .field("ac_etime", &ac_etime)
            .field("ac_btime", &ac_btime)
            .field("ac_uid", &ac_uid)
            .field("ac_gid", &ac_gid)
            .field("ac_mem", &ac_mem)
            .field("ac_io", &ac_io)
            .field("ac_tty", &ac_tty)
            .field("ac_flag", &ac_flag)
            .finish()
    }
}

// netbsd_x8632::lastlogx

impl FixedStructTrait for netbsd_x8632::lastlogx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8632_Lastlogx
    }
    fn size(&self) -> usize {
        netbsd_x8632::LASTLOGX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8632::lastlogx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8632::lastlogx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8632::lastlogx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8632::lastlogx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8632::lastlogx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8632::lastlogx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8632::lastlogx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on netbsd_x8632::lastlogx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        self
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on netbsd_x8632::lastlogx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on netbsd_x8632::lastlogx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on netbsd_x8632::lastlogx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on netbsd_x8632::lastlogx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on netbsd_x8632::lastlogx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8632::lastlogx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8632::lastlogx");
    }
}

impl fmt::Debug for netbsd_x8632::lastlogx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
        let tv_sec = self.ll_tv.tv_sec;
        // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
        let tv_usec = self.ll_tv.tv_usec;
        f.debug_struct("netbsd_x8632::lastlogx")
            .field("size", &self.size())
            .field("ll_tv.tv_sec", &tv_sec)
            .field("ll_tv.tv_usec", &tv_usec)
            .field("ll_line", &self.ll_line())
            .field("ll_host", &self.ll_host())
            .field("ll_ss", &self.ll_ss)
            .finish()
    }
}

// netbsd_x8632::utmpx

impl FixedStructTrait for netbsd_x8632::utmpx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8632_Utmpx
    }
    fn size(&self) -> usize {
        netbsd_x8632::UTMPX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8632::utmpx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8632::utmpx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8632::utmpx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8632::utmpx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8632::utmpx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8632::utmpx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8632::utmpx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on netbsd_x8632::utmpx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on netbsd_x8632::utmpx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        self
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on netbsd_x8632::utmpx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on netbsd_x8632::utmpx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on netbsd_x8632::utmpx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on netbsd_x8632::utmpx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8632::utmpx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8632::utmpx");
    }
}

impl fmt::Debug for netbsd_x8632::utmpx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
        let tv_sec = self.ut_tv.tv_sec;
        // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
        let tv_usec = self.ut_tv.tv_usec;
        f.debug_struct("netbsd_x8632::utmpx")
            .field("size", &self.size())
            .field("ut_name", &self.ut_name())
            .field("ut_id", &self.ut_id())
            .field("ut_line", &self.ut_line())
            .field("ut_host", &self.ut_host())
            .field("ut_session", &self.ut_session)
            .field("ut_type", &self.ut_type)
            .field("ut_pid", &self.ut_pid)
            .field("ut_exit.e_termination", &self.ut_exit.e_termination)
            .field("ut_exit.e_exit", &self.ut_exit.e_exit)
            .field("ut_ss", &self.ut_ss)
            .field("ut_tv.tv_sec", &tv_sec)
            .field("ut_tv.tv_usec", &tv_usec)
            .field("ut_pad", &self.ut_pad)
            .finish()
    }
}

// netbsd_x8664::lastlog

impl FixedStructTrait for netbsd_x8664::lastlog {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8664_Lastlog
    }
    fn size(&self) -> usize {
        netbsd_x8664::LASTLOG_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8664::lastlog");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8664::lastlog");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8664::lastlog");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8664::lastlog");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8664::lastlog");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8664::lastlog");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8664::lastlog");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on netbsd_x8664::lastlog");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on netbsd_x8664::lastlog");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on netbsd_x8664::lastlog");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        self
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on netbsd_x8664::lastlog");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on netbsd_x8664::lastlog");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on netbsd_x8664::lastlog");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8664::lastlog");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8664::lastlog");
    }
}

impl fmt::Debug for netbsd_x8664::lastlog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("netbsd_x8664::lastlog")
            .field("size", &self.size())
            .field("ll_time", &self.ll_time)
            .field("ll_line", &self.ll_line())
            .field("ll_host", &self.ll_host())
            .finish()
    }
}

// netbsd_x8664::lastlogx

impl FixedStructTrait for netbsd_x8664::lastlogx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8664_Lastlogx
    }
    fn size(&self) -> usize {
        netbsd_x8664::LASTLOGX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8664::lastlogx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8664::lastlogx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8664::lastlogx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8664::lastlogx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8664::lastlogx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8664::lastlogx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8664::lastlogx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on netbsd_x8664::lastlogx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on netbsd_x8664::lastlogx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on netbsd_x8664::lastlogx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on netbsd_x8664::lastlogx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        self
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on netbsd_x8664::lastlogx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on netbsd_x8664::lastlogx");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8664::lastlogx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8664::lastlogx");
    }
}

impl fmt::Debug for netbsd_x8664::lastlogx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("netbsd_x8664::lastlogx")
            .field("size", &self.size())
            .field("ll_tv.tv_sec", &self.ll_tv.tv_sec)
            .field("ll_tv.tv_usec", &self.ll_tv.tv_usec)
            .field("ll_line", &self.ll_line())
            .field("ll_host", &self.ll_host())
            .field("ll_ss", &self.ll_ss)
            .finish()
    }
}

// netbsd_x8664::utmp

impl FixedStructTrait for netbsd_x8664::utmp {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8664_Utmp
    }
    fn size(&self) -> usize {
        netbsd_x8664::UTMP_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8664::utmp");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8664::utmp");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8664::utmp");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8664::utmp");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8664::utmp");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8664::utmp");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8664::utmp");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on netbsd_x8664::utmp");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on netbsd_x8664::utmp");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on netbsd_x8664::utmp");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on netbsd_x8664::utmp");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on netbsd_x8664::utmp");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        self
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on netbsd_x8664::utmp");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8664::utmp");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8664::utmp");
    }
}

impl fmt::Debug for netbsd_x8664::utmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("netbsd_x8664::utmp")
            .field("size", &self.size())
            .field("ut_line", &self.ut_line())
            .field("ut_name", &self.ut_name())
            .field("ut_host", &self.ut_host())
            .field("ut_time", &self.ut_time)
            .finish()
    }
}

// netbsd_x8664::utmpx

impl FixedStructTrait for netbsd_x8664::utmpx {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Netbsd_x8664_Utmpx
    }
    fn size(&self) -> usize {
        netbsd_x8664::UTMPX_SZ
    }

    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on netbsd_x8664::utmpx");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on netbsd_x8664::utmpx");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on netbsd_x8664::utmpx");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on netbsd_x8664::utmpx");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on netbsd_x8664::utmpx");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on netbsd_x8664::utmpx");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on netbsd_x8664::utmpx");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        self
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on netbsd_x8664::utmpx");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on netbsd_x8664::utmpx");
    }
}

impl fmt::Debug for netbsd_x8664::utmpx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("netbsd_x8664::utmpx")
            .field("size", &self.size())
            .field("ut_user", &self.ut_user())
            .field("ut_id", &self.ut_id())
            .field("ut_line", &self.ut_line())
            .field("ut_host", &self.ut_host())
            .field("ut_session", &self.ut_session)
            .field("ut_type", &self.ut_type)
            .field("ut_pid", &self.ut_pid)
            .field("ut_exit.e_termination", &self.ut_exit.e_termination)
            .field("ut_exit.e_exit", &self.ut_exit.e_exit)
            .field("ut_tv.tv_sec", &self.ut_tv.tv_sec)
            .field("ut_tv.tv_usec", &self.ut_tv.tv_usec)
            .finish()
    }
}

// openbsd_x86::lastlog

impl FixedStructTrait for openbsd_x86::lastlog {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Openbsd_x86_Lastlog
    }
    fn size(&self) -> usize {
        openbsd_x86::LASTLOG_SZ
    }
    
    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on openbsd_x86::lastlog");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on openbsd_x86::lastlog");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on openbsd_x86::lastlog");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on openbsd_x86::lastlog");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on openbsd_x86::lastlog");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on openbsd_x86::lastlog");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on openbsd_x86::lastlog");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on openbsd_x86::lastlog");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        self
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        panic!("as_openbsd_x86_utmp() called on openbsd_x86::lastlog");
    }
}

impl fmt::Debug for openbsd_x86::lastlog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("openbsd_x86::lastlog")
            .field("size", &self.size())
            .field("ll_time", &self.ll_time)
            .field("ll_line", &self.ll_line())
            .field("ll_host", &self.ll_host())
            .finish()
    }
}

// openbsd_x86::utmp

impl FixedStructTrait for openbsd_x86::utmp {
    fn fixedstruct_type(&self) -> FixedStructType {
        FixedStructType::Fs_Openbsd_x86_Utmp
    }
    fn size(&self) -> usize {
        openbsd_x86::UTMP_SZ
    }
    
    fn as_freebsd_x8664_utmpx(&self) -> &freebsd_x8664::utmpx {
        panic!("as_freebsd_x8664_utmpx() called on openbsd_x86::utmp");
    }
    fn as_linux_arm64aarch64_lastlog(&self) -> &linux_arm64aarch64::lastlog {
        panic!("as_linux_arm64aarch64_lastlog() called on openbsd_x86::utmp");
    }
    fn as_linux_arm64aarch64_utmpx(&self) -> &linux_arm64aarch64::utmpx {
        panic!("as_linux_arm64aarch64_utmpx() called on openbsd_x86::utmp");
    }
    fn as_linux_x86_acct(&self) -> &linux_x86::acct {
        panic!("as_linux_x86_acct() called on openbsd_x86::utmp");
    }
    fn as_linux_x86_acct_v3(&self) -> &linux_x86::acct_v3 {
        panic!("as_linux_x86_acct_v3() called on openbsd_x86::utmp");
    }
    fn as_linux_x86_lastlog(&self) -> &linux_x86::lastlog {
        panic!("as_linux_x86_lastlog() called on openbsd_x86::utmp");
    }
    fn as_linux_x86_utmpx(&self) -> &linux_x86::utmpx {
        panic!("as_linux_x86_utmpx() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8632_acct(&self) -> &netbsd_x8632::acct {
        panic!("as_netbsd_x8632_acct() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8632_lastlogx(&self) -> &netbsd_x8632::lastlogx {
        panic!("as_netbsd_x8632_lastlogx() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8632_utmpx(&self) -> &netbsd_x8632::utmpx {
        panic!("as_netbsd_x8632_utmpx() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8664_lastlog(&self) -> &netbsd_x8664::lastlog {
        panic!("as_netbsd_x8664_lastlog() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8664_lastlogx(&self) -> &netbsd_x8664::lastlogx {
        panic!("as_netbsd_x8664_lastlogx() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8664_utmp(&self) -> &netbsd_x8664::utmp {
        panic!("as_netbsd_x8664_utmp() called on openbsd_x86::utmp");
    }
    fn as_netbsd_x8664_utmpx(&self) -> &netbsd_x8664::utmpx {
        panic!("as_netbsd_x8664_utmpx() called on openbsd_x86::utmp");
    }
    fn as_openbsd_x86_lastlog(&self) -> &openbsd_x86::lastlog {
        panic!("as_openbsd_x86_lastlog() called on openbsd_x86::utmp");
    }
    fn as_openbsd_x86_utmp(&self) -> &openbsd_x86::utmp {
        self
    }
}

impl fmt::Debug for openbsd_x86::utmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("openbsd_x86::utmp")
            .field("size", &self.size())
            .field("ut_line", &self.ut_line())
            .field("ut_name", &self.ut_name())
            .field("ut_host", &self.ut_host())
            .field("ut_time", &self.ut_time)
            .finish()
    }
}

pub (crate) type FixedStructTypeSet = HashMap<FixedStructType, Score>;

/// return all possible [`FixedStructType`] types based on the passed `filesz`.
/// Give a score bonus to matching `FileTypeFixedStruct` types.
pub(crate) fn filesz_to_types(filesz: FileSz, file_type_fixed_struct: &FileTypeFixedStruct)
    -> Option<FixedStructTypeSet>
{
    defn!("({:?}, {:?})", filesz, file_type_fixed_struct);

    if filesz == 0 {
        defx!("return None; filesz==0");
        return None;
    }

    let mut set = FixedStructTypeSet::new();

    const BONUS: Score = 15;

    // if the given `FileTypeFixedStruct` matches the size offset then
    // it gets a bonus score.
    match file_type_fixed_struct {
        FileTypeFixedStruct::Acct => {
            if filesz % linux_x86::ACCT_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Linux_x86_Acct, BONUS);
            }
            if filesz % netbsd_x8632::ACCT_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Netbsd_x8632_Acct, BONUS);
            }
        }
        FileTypeFixedStruct::AcctV3 => {
            if filesz % linux_x86::ACCT_V3_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Linux_x86_Acct_v3, BONUS);
            }
        }
        FileTypeFixedStruct::Lastlog => {
            if filesz % linux_arm64aarch64::LASTLOG_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog, BONUS);
            }
            if filesz % linux_x86::LASTLOG_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Linux_x86_Lastlog, BONUS);
            }
            if filesz % netbsd_x8664::LASTLOG_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Netbsd_x8664_Lastlog, BONUS);
            }
            if filesz % openbsd_x86::LASTLOG_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Openbsd_x86_Lastlog, BONUS);
            }
        }
        FileTypeFixedStruct::Lastlogx => {
            if filesz % netbsd_x8632::LASTLOGX_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Netbsd_x8632_Lastlogx, BONUS);
            }
        }
        FileTypeFixedStruct::Utmp => {
            if filesz % netbsd_x8664::UTMP_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Netbsd_x8664_Utmp, BONUS);
            }
            if filesz % openbsd_x86::UTMP_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Openbsd_x86_Utmp, BONUS);
            }
        }
        FileTypeFixedStruct::Utmpx => {
            if filesz % freebsd_x8664::UTMPX_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Freebsd_x8664_Utmpx, BONUS);
            }
            if filesz % linux_arm64aarch64::UTMPX_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx, BONUS);
            }
            if filesz % linux_x86::UTMPX_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Linux_x86_Utmpx, BONUS);
            }

            if filesz % netbsd_x8632::UTMPX_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Netbsd_x8632_Utmpx, BONUS);
            }
            if filesz % netbsd_x8664::UTMPX_SZ_FO == 0 {
                set.insert(FixedStructType::Fs_Netbsd_x8664_Utmpx, BONUS);
            }
        }
    }

    // try _all_ types anyway; the file naming varies widely and follows
    // general patterns but is not strict.

    if filesz % freebsd_x8664::UTMPX_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Freebsd_x8664_Utmpx).or_insert(0);
    }
    if filesz % linux_arm64aarch64::LASTLOG_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog).or_insert(0);
    }
    if filesz % linux_arm64aarch64::UTMPX_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx).or_insert(0);
    }
    if filesz % linux_x86::ACCT_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Linux_x86_Acct).or_insert(0);
    }
    if filesz % linux_x86::ACCT_V3_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Linux_x86_Acct_v3).or_insert(0);
    }
    if filesz % linux_x86::LASTLOG_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Linux_x86_Lastlog).or_insert(0);
    }
    if filesz % linux_x86::UTMPX_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Linux_x86_Utmpx).or_insert(0);
    }
    if filesz % netbsd_x8632::ACCT_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Netbsd_x8632_Acct).or_insert(0);
    }
    if filesz % netbsd_x8632::LASTLOGX_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Netbsd_x8632_Lastlogx).or_insert(0);
    }
    if filesz % netbsd_x8632::UTMPX_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Netbsd_x8632_Utmpx).or_insert(0);
    }
    if filesz % netbsd_x8664::LASTLOG_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Netbsd_x8664_Lastlog).or_insert(0);
    }
    if filesz % netbsd_x8664::UTMP_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Netbsd_x8664_Utmp).or_insert(0);
    }
    if filesz % netbsd_x8664::UTMPX_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Netbsd_x8664_Utmpx).or_insert(0);
    }
    if filesz % openbsd_x86::LASTLOG_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Openbsd_x86_Lastlog).or_insert(0);
    }
    if filesz % openbsd_x86::UTMP_SZ_FO == 0 {
        set.entry(FixedStructType::Fs_Openbsd_x86_Utmp).or_insert(0);
    }

    if set.is_empty() {
        defx!("return None; set.is_empty");
        return None;
    }

    defx!("return {} types: {:?}", set.len(), set);

    Some(set)
}

/// An entry for [pointing to a fixed-size C struct] with additional derived
/// information.
///
/// [pointing to a fixed-size C struct]: self::FixedStructTrait
pub struct FixedStruct
{
    /// A pointer to underlying fixed-sized structure entry data.
    pub fixedstructptr: FixedStructDynPtr,
    /// The `fixedstructtype` determines the runtime dispatching of various
    /// methods specific to the fixed-sized structure type.
    pub fixedstructtype: FixedStructType,
    /// `FileTypeFixedStruct` is library-wide type passed in from the caller
    /// It is used as a hint of the originating file.
    pub filetypefixedstruct: FileTypeFixedStruct,
    /// The byte offset into the file where the fixed-sized structure data
    /// begins.
    pub fileoffset: FileOffset,
    /// The derived `DateTimeL` instance using function
    /// [`convert_tvpair_to_datetime`].
    dt: DateTimeL,
    /// The derived tv_pair, equivalent to `dt` but as a `tv_sec` (seconds) and
    /// `tv_usec` (fractional microseconds) pair.
    /// Used early in the `FixedStruct` lifecycle for fast checking of
    /// an entry's datetime without the conversion.
    tv_pair: tv_pair_type,
}

pub fn copy_fixedstructptr(entry: &FixedStructDynPtr) -> FixedStructDynPtr {
    match entry.fixedstruct_type() {
        FixedStructType::Fs_Freebsd_x8664_Utmpx => {
            Box::new(*entry.as_freebsd_x8664_utmpx())
        }
        FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
            Box::new(*entry.as_linux_arm64aarch64_lastlog())
        }
        FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
            Box::new(*entry.as_linux_arm64aarch64_utmpx())
        }
        FixedStructType::Fs_Linux_x86_Acct => {
            Box::new(*entry.as_linux_x86_acct())
        }
        FixedStructType::Fs_Linux_x86_Acct_v3 => {
            Box::new(*entry.as_linux_x86_acct_v3())
        }
        FixedStructType::Fs_Linux_x86_Lastlog => {
            Box::new(*entry.as_linux_x86_lastlog())
        }
        FixedStructType::Fs_Linux_x86_Utmpx => {
            Box::new(*entry.as_linux_x86_utmpx())
        }
        FixedStructType::Fs_Netbsd_x8632_Acct => {
            Box::new(*entry.as_netbsd_x8632_acct())
        }
        FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
            Box::new(*entry.as_netbsd_x8632_lastlogx())
        }
        FixedStructType::Fs_Netbsd_x8632_Utmpx => {
            Box::new(*entry.as_netbsd_x8632_utmpx())
        }
        FixedStructType::Fs_Netbsd_x8664_Lastlog => {
            Box::new(*entry.as_netbsd_x8664_lastlog())
        }
        FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
            Box::new(*entry.as_netbsd_x8664_lastlogx())
        }
        FixedStructType::Fs_Netbsd_x8664_Utmp => {
            Box::new(*entry.as_netbsd_x8664_utmp())
        }
        FixedStructType::Fs_Netbsd_x8664_Utmpx => {
            Box::new(*entry.as_netbsd_x8664_utmpx())
        }
        FixedStructType::Fs_Openbsd_x86_Lastlog => {
            Box::new(*entry.as_openbsd_x86_lastlog())
        }
        FixedStructType::Fs_Openbsd_x86_Utmp => {
            Box::new(*entry.as_openbsd_x86_utmp())
        }
    }
}

impl Clone for FixedStruct {
    /// Manually implemented `clone`.
    ///
    /// Cannot `#[derive(Clone)]` because that requires
    /// trait `Sized` to be `impl` for all fields of `FixedStruct`.
    /// But the `FixedStructTrait` trait is used as a `dyn` trait
    /// object; it is dynamically sized (it is sized at runtime).
    /// So a `FixedStructTrait` cannot `impl`ement `Sized` thus `FixedStruct` cannot
    /// `impl`ement `Sized` thus must manually implement `clone`.
    fn clone(self: &FixedStruct) -> Self {
        defñ!("FixedStruct.clone()");
        let fixedstructptr = copy_fixedstructptr(&self.fixedstructptr);
        FixedStruct {
            fixedstructptr,
            fixedstructtype: self.fixedstructtype,
            filetypefixedstruct: self.filetypefixedstruct,
            fileoffset: self.fileoffset,
            dt: self.dt.clone(),
            tv_pair: self.tv_pair,
        }
    }
}

impl fmt::Debug for FixedStruct
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("FixedStruct")
            .field("size", &self.fixedstructptr.size())
            .field("type", &self.fixedstructtype)
            .field("filetype", &self.filetypefixedstruct)
            .field("fileoffset", &self.fileoffset)
            .field("dt", &self.dt)
            .field("tv_pair", &self.tv_pair)
            .finish()
    }
}

/// Index into a `[u8]` buffer. Used by [`as_bytes`].
///
/// [`as_bytes`]: self::FixedStruct::as_bytes
pub type BufIndex = usize;

/// Information returned from [`as_bytes`].
///
/// Variant `Ok` holds
/// - number of bytes copied
/// - start index of datetime substring
/// - end index of datetime substring
///
/// Variant `Fail` holds number of bytes copied
///
/// [`as_bytes`]: self::FixedStruct::as_bytes
pub enum InfoAsBytes {
    Ok(usize, BufIndex, BufIndex),
    Fail(usize),
}

/// Convert `timeval` types [`tv_sec_type`] and [`tv_usec_type`] to a
/// [`DateTimeL`] instance.
///
/// Allow lossy microsecond conversion.
/// Return `Error` if `tv_sec` conversion fails.
pub fn convert_tvpair_to_datetime(
    tv_pair: tv_pair_type,
    tz_offset: &FixedOffset,
) -> Result<DateTimeL, Error>{
    let tv_usec = tv_pair.1;
    // Firstly, convert i64 to i32 for passing to `timestamp_opt`.
    let mut nsec: nsecs_type = match tv_usec.try_into() {
        Ok(val) => val,
        Err(_err) => {
            de_wrn!("failed to convert tv_usec 0x{:X} to nsecs_type: {}", tv_usec, _err);
            // ignore overflow and continue; `tv_usec` merely supplements
            // the more coarse `tv_sec`
            0
        }
    };
    // Secondly, multiply by 1000 to get nanoseconds.
    nsec = match nsec.checked_mul(1000) {
        Some(val) => val,
        None => {
            de_wrn!("failed to multiply nsec 0x{:X} * 1000: overflow", nsec);
            // ignore overflow and continue; `tv_usec` merely supplements
            // the more coarse `tv_sec`
            0
        }
    };

    let tv_sec = tv_pair.0;
    defñ!("{:?}.timestamp({}, {})", tz_offset, tv_sec, nsec);
    match tz_offset.timestamp_opt(
        tv_sec, nsec
    ) {
        LocalResult::None => {
            // try again with zero nanoseconds
            match tz_offset.timestamp_opt(tv_sec, 0) {
                LocalResult::None => {
                    let err_s = format!(
                        "failed to convert tv_sec 0x{:08X} to DateTime from tz_offset {}",
                        tv_sec, tz_offset,
                    );
                    de_wrn!("{}", err_s);

                    Result::Err(
                        Error::new(
                            ErrorKind::InvalidData,
                            err_s,
                        )
                    )
                }
                LocalResult::Single(dt) => Ok(dt),
                LocalResult::Ambiguous(dt, _) => Ok(dt),
            }
        }
        LocalResult::Single(dt) => Ok(dt),
        // nothing can disambiguate this so just take the first datetime
        LocalResult::Ambiguous(dt, _) => Ok(dt),
    }
}

/// Convert [`DateTimeL`] instance to [`tv_sec_type`] and [`tv_usec_type`].
///
/// Return `Error` if `tv_sec` conversion fails.
pub fn convert_datetime_tvpair(
    dt: &DateTimeL,
) -> tv_pair_type {
    let tv_sec: tv_sec_type = dt.timestamp();
    let tv_usec: tv_usec_type = match dt.timestamp_subsec_micros().try_into() {
        Ok(val) => val,
        Err(_err) => {
            de_wrn!(
                "failed to convert subsec_micros {} to tv_usec_type: {}",
                dt.timestamp_subsec_micros(), _err,
            );
            // ignore overflow and continue; `tv_usec` merely supplements
            // the more coarse `tv_sec`
            0
        }
    };

    tv_pair_type(tv_sec, tv_usec)
}

#[cfg(any(debug_assertions, test))]
fn struct_field_to_string(buffer: &[u8], offset: usize, sz: usize) -> String {
    let mut s = String::with_capacity(sz);
    for i in offset..(offset + sz) {
        s.push(byte_to_char_noraw(buffer[i]));
    }

    s
}

/// Helper to [`buffer_to_fixedstructptr`]
/// [`deo!`] for the `$field`
#[cfg(any(debug_assertions, test))]
macro_rules! deo_field_dump {
    ($as_fixedstruct:expr, $field: ident, $buffer: ident) => ({{
        let fs = $as_fixedstruct;

        let base_ptr_offset: usize = std::ptr::addr_of!(*fs) as usize;
        let field_ptr = std::ptr::addr_of!(fs.$field);
        let field_offset = field_ptr as usize - base_ptr_offset;
        let field_sz = std::mem::size_of_val(&fs.$field);
        let field_offset_end = field_offset + field_sz;
        let field_val_string = struct_field_to_string($buffer, field_offset, field_sz);
        let field_name = stringify!($field);

        deo!("{:<20}: {:3}‥{:3} @{:p} | {}",
             field_name, field_offset, field_offset_end, field_ptr, field_val_string);
    }})
}

/// Helper to [`buffer_to_fixedstructptr`]
/// [`deo!`] for the `$field`
#[cfg(any(debug_assertions, test))]
macro_rules! deo_field_dump_num {
    ($as_fixedstruct:expr, $field: ident, $buffer: ident, $typ: ty) => ({{
        let fs = $as_fixedstruct;
        let field_sz = std::mem::size_of_val(&fs.$field);
        deo_field_dump_num_impl!(fs, $field, field_sz, $buffer, $typ);
    }})
}

/// Helper to [`buffer_to_fixedstructptr`]
/// [`deo!`] for the `$field`
///
/// Useful for `packed` structs that cannot be referenced in a macro as
/// `size_of_val!(struct.field)` (like in macro [`deo_field_dump_num`]).
#[cfg(any(debug_assertions, test))]
macro_rules! deo_field_dump_num_szof {
    ($as_fixedstruct:expr, $field: ident, $buffer: ident, $typ: ty) => ({{
        let fs = $as_fixedstruct;
        let field_sz = std::mem::size_of::<$typ>();
        deo_field_dump_num_impl!(fs, $field, field_sz, $buffer, $typ);
    }})
}

/// Helper to [`buffer_to_fixedstructptr`]
/// [`deo!`] for the `$field`
#[cfg(any(debug_assertions, test))]
macro_rules! deo_field_dump_num_impl {
    ($as_fixedstruct:expr, $field: ident, $field_sz: expr, $buffer: ident, $typ: ty) => ({{
        let fs = $as_fixedstruct;

        let base_ptr_offset: usize = std::ptr::addr_of!(*fs) as usize;
        let field_ptr = std::ptr::addr_of!(fs.$field);
        let field_offset = field_ptr as usize - base_ptr_offset;
        let field_offset_end = field_offset + $field_sz;
        let field_val_string = struct_field_to_string($buffer, field_offset, $field_sz);
        let field_name = stringify!($field);
        let value: $typ = fs.$field;

        deo!("{:<20}: {:3}‥{:3} @{:p} | {:<20} 0x{:08X}",
             field_name, field_offset, field_offset_end, field_ptr,
             field_val_string, value);
    }})
}

/// Convert `[u8]` bytes to a [`FixedStructDynPtr`] instance specified by
/// `fixedstructtype`.
///
/// A `buffer` of only null bytes (zero values) will return `None`.
/// A `buffer` of only 0xFF bytes will return `None`.
///
/// unsafe.
pub fn buffer_to_fixedstructptr(buffer: &[u8], fixedstructtype: FixedStructType) -> Option<FixedStructDynPtr>
{
    defn!("(buffer len {:?}, fixedstructtype {:?})", buffer.len(), fixedstructtype);

    let sz: usize = fixedstructtype.size();
    if buffer.len() < sz {
        if cfg!(debug_assertions) && ! cfg!(test)
        {
            debug_panic!(
                "buffer to small; {} bytes but fixedstruct type {:?} is {} bytes",
                buffer.len(), fixedstructtype, sz,
            );
        }
        defx!("return None");
        return None;
    }
    let slice_ = &buffer[..sz];
    // check for all zero bytes
    if slice_.iter().all(|&x| x == 0) {
        defx!("buffer[0‥{}] is all 0x00 bytes; return None", slice_.len());
        return None;
    }
    // check for all 0xFF bytes
    if slice_.iter().all(|&x| x == 0xFF) {
        defx!("buffer[0‥{}] is all 0xFF bytes; return None", slice_.len());
        return None;
    }
    #[cfg(debug_assertions)]
    {
        // print buffer bytes to the console
        const LEN: usize = 16;
        defo!("\nbuffer[‥{}] ({} bytes per line)\n[", sz, LEN);
        let _lock = si_trace_print::printers::GLOBAL_LOCK_PRINTER.lock().unwrap();
        for i in 0..sz {
            eprint!("{:?}, ", slice_[i] as char);
            if i != 0 && i % LEN == 0 {
                eprintln!();
            }
        }
        eprintln!("\n]");
        for i in 0..sz {
            eprint!("{}", byte_to_char_noraw(slice_[i]));
            if i != 0 && i % LEN == 0 {
                eprintln!();
            }
        }
        eprintln!();
    }
    let entry: FixedStructDynPtr = match fixedstructtype {
        FixedStructType::Fs_Freebsd_x8664_Utmpx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<freebsd_x8664::utmpx>())
                )
            }
        }
        FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_arm64aarch64::lastlog>())
                )
            }
        }
        FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_arm64aarch64::utmpx>())
                )
            }
        }
        FixedStructType::Fs_Linux_x86_Acct => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_x86::acct>())
                )
            }
        }
        FixedStructType::Fs_Linux_x86_Acct_v3 => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_x86::acct_v3>())
                )
            }
        }
        FixedStructType::Fs_Linux_x86_Lastlog => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_x86::lastlog>())
                )
            }
        }
        FixedStructType::Fs_Linux_x86_Utmpx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_x86::utmpx>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8632_Acct => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8632::acct>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8632::lastlogx>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8632_Utmpx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8632::utmpx>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8664_Lastlog => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8664::lastlog>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8664::lastlogx>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8664_Utmp => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8664::utmp>())
                )
            }
        }
        FixedStructType::Fs_Netbsd_x8664_Utmpx => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<netbsd_x8664::utmpx>())
                )
            }
        }
        FixedStructType::Fs_Openbsd_x86_Lastlog => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<openbsd_x86::lastlog>())
                )
            }
        }
        FixedStructType::Fs_Openbsd_x86_Utmp => {
            unsafe {
                Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<openbsd_x86::utmp>())
                )
            }
        }
    };

    #[cfg(any(debug_assertions, test))]
    match fixedstructtype {
        // print struct member offsets and bytes
        // `addr_of!` used here due to unaligned references being a warning:
        // credit https://stackoverflow.com/a/26271748/471376
        FixedStructType::Fs_Freebsd_x8664_Utmpx => {
            let utmpx: &freebsd_x8664::utmpx = entry.as_freebsd_x8664_utmpx();

            deo!("freebsd_x8664::utmpx offsets and bytes, size {}", utmpx.size());
            deo_field_dump_num!(utmpx, ut_type, buffer, freebsd_x8664::c_short);
            deo_field_dump!(utmpx, ut_tv, buffer);
            deo_field_dump!(utmpx, ut_id, buffer);
            deo_field_dump_num!(utmpx, ut_pid, buffer, freebsd_x8664::pid_t);
            deo_field_dump!(utmpx, ut_user, buffer);
            deo_field_dump!(utmpx, ut_line, buffer);
            deo_field_dump!(utmpx, ut_host, buffer);
            deo_field_dump!(utmpx, __ut_spare, buffer);
        }
        FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
            let lastlog: &linux_arm64aarch64::lastlog = entry.as_linux_arm64aarch64_lastlog();

            deo!("linux_arm64aarch64::lastlog offsets and bytes, size {}", lastlog.size());
            deo_field_dump_num!(lastlog, ll_time, buffer, linux_arm64aarch64::ll_time_t);
            deo_field_dump!(lastlog, ll_line, buffer);
            deo_field_dump!(lastlog, ll_host, buffer);
        }
        FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
            let utmpx: &linux_arm64aarch64::utmpx = entry.as_linux_arm64aarch64_utmpx();

            deo!("linux_arm64aarch64::utmpx offsets and bytes, size {}", utmpx.size());
            deo_field_dump_num!(utmpx, ut_type, buffer, linux_arm64aarch64::c_short);
            deo_field_dump_num!(utmpx, ut_pid, buffer, linux_arm64aarch64::pid_t);
            deo_field_dump!(utmpx, ut_line, buffer);
            deo_field_dump!(utmpx, ut_id, buffer);
            deo_field_dump!(utmpx, ut_user, buffer);
            deo_field_dump!(utmpx, ut_host, buffer);
            deo_field_dump_num!(utmpx, ut_exit, buffer, i32);
            deo_field_dump_num!(utmpx, ut_session, buffer, i64);
            deo_field_dump!(utmpx, ut_tv, buffer);
            deo_field_dump!(utmpx, ut_addr_v6, buffer);
            deo_field_dump!(utmpx, __glibc_reserved, buffer);
        }
        FixedStructType::Fs_Linux_x86_Acct => {
            let acct: &linux_x86::acct = entry.as_linux_x86_acct();

            deo!("linux_x86::acct offsets and bytes, size {}", acct.size());
            deo_field_dump_num!(acct, ac_flag, buffer, linux_x86::c_char);
            deo_field_dump_num!(acct, ac_uid, buffer, u16);
            deo_field_dump_num!(acct, ac_gid, buffer, u16);
            deo_field_dump_num!(acct, ac_tty, buffer, u16);
            deo_field_dump_num!(acct, ac_btime, buffer, linux_x86::b_time_t);
            deo_field_dump_num!(acct, ac_utime, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_stime, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_etime, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_mem, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_io, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_rw, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_minflt, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_majflt, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_swaps, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_exitcode, buffer, u32);
            deo_field_dump!(acct, ac_comm, buffer);
        }
        FixedStructType::Fs_Linux_x86_Acct_v3 => {
            let acct: &linux_x86::acct_v3 = entry.as_linux_x86_acct_v3();

            deo!("linux_x86::acct_v3 offsets and bytes, size {}", acct.size());
            deo_field_dump_num!(acct, ac_flag, buffer, linux_x86::c_char);
            deo_field_dump_num!(acct, ac_version, buffer, linux_x86::c_char);
            deo_field_dump_num!(acct, ac_tty, buffer, u16);
            deo_field_dump_num!(acct, ac_exitcode, buffer, u32);
            deo_field_dump_num!(acct, ac_uid, buffer, u32);
            deo_field_dump_num!(acct, ac_gid, buffer, u32);
            deo_field_dump_num!(acct, ac_pid, buffer, u32);
            deo_field_dump_num!(acct, ac_ppid, buffer, u32);
            deo_field_dump_num!(acct, ac_btime, buffer, linux_x86::b_time_t);
            deo_field_dump!(acct, ac_etime, buffer);
            deo_field_dump_num!(acct, ac_utime, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_stime, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_mem, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_io, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_rw, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_minflt, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_majflt, buffer, linux_x86::comp_t);
            deo_field_dump_num!(acct, ac_swaps, buffer, linux_x86::comp_t);
            deo_field_dump!(acct, ac_comm, buffer);
        }
        FixedStructType::Fs_Linux_x86_Lastlog => {
            let lastlog: &linux_x86::lastlog = entry.as_linux_x86_lastlog();

            deo!("linux_x86::lastlog offsets and bytes, size {}", lastlog.size());
            deo_field_dump_num!(lastlog, ll_time, buffer, linux_x86::ll_time_t);
            deo_field_dump!(lastlog, ll_line, buffer);
            deo_field_dump!(lastlog, ll_host, buffer);
        }
        FixedStructType::Fs_Linux_x86_Utmpx => {
            let utmpx: &linux_x86::utmpx = entry.as_linux_x86_utmpx();

            deo!("linux_x86::utmpx offsets and bytes, size {}", utmpx.size());
            deo_field_dump_num!(utmpx, ut_type, buffer, linux_x86::c_short);
            deo_field_dump_num!(utmpx, ut_pid, buffer, linux_x86::pid_t);
            deo_field_dump!(utmpx, ut_line, buffer);
            deo_field_dump!(utmpx, ut_id, buffer);
            deo_field_dump!(utmpx, ut_user, buffer);
            deo_field_dump!(utmpx, ut_host, buffer);
            deo_field_dump!(utmpx, ut_exit, buffer);
            deo_field_dump_num!(utmpx, ut_session, buffer, i32);
            deo_field_dump!(utmpx, ut_tv, buffer);
            deo_field_dump!(utmpx, ut_addr_v6, buffer);
            deo_field_dump!(utmpx, __glibc_reserved, buffer);
        }
        FixedStructType::Fs_Netbsd_x8632_Acct => {
            let acct: &netbsd_x8632::acct = entry.as_netbsd_x8632_acct();

            deo!("netbsd_x8664::acct offsets and bytes, size {}", acct.size());
            deo_field_dump!(acct, ac_comm, buffer);
            deo_field_dump_num_szof!(acct, ac_utime, buffer, netbsd_x8632::comp_t);
            deo_field_dump_num_szof!(acct, ac_stime, buffer, netbsd_x8632::comp_t);
            deo_field_dump_num_szof!(acct, ac_etime, buffer, netbsd_x8632::comp_t);
            deo_field_dump_num_szof!(acct, ac_btime, buffer, netbsd_x8632::time_t);
            deo_field_dump_num_szof!(acct, ac_uid, buffer, netbsd_x8632::uid_t);
            deo_field_dump_num_szof!(acct, ac_gid, buffer, netbsd_x8632::gid_t);
            deo_field_dump_num_szof!(acct, ac_mem, buffer, netbsd_x8632::comp_t);
            deo_field_dump_num_szof!(acct, ac_io, buffer, netbsd_x8632::comp_t);
            deo_field_dump_num_szof!(acct, ac_tty, buffer, netbsd_x8632::dev_t);
            deo_field_dump_num_szof!(acct, ac_flag, buffer, netbsd_x8632::uint8_t);
        }
        FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
            let lastlogx: &netbsd_x8632::lastlogx = entry.as_netbsd_x8632_lastlogx();

            deo!("netbsd_x8632::lastlogx offsets and bytes, size {}", lastlogx.size());
            deo_field_dump!(lastlogx, ll_tv, buffer);
            deo_field_dump!(lastlogx, ll_line, buffer);
            deo_field_dump!(lastlogx, ll_host, buffer);
            deo_field_dump!(lastlogx, ll_ss, buffer);
        }
        FixedStructType::Fs_Netbsd_x8632_Utmpx => {
            let utmpx: &netbsd_x8632::utmpx = entry.as_netbsd_x8632_utmpx();

            deo!("netbsd_x8632::utmpx offsets and bytes, size {}", utmpx.size());
            deo_field_dump!(utmpx, ut_name, buffer);
            deo_field_dump!(utmpx, ut_id, buffer);
            deo_field_dump!(utmpx, ut_line, buffer);
            deo_field_dump!(utmpx, ut_host, buffer);
            deo_field_dump_num!(utmpx, ut_session, buffer, netbsd_x8632::uint16_t);
            deo_field_dump_num!(utmpx, ut_type, buffer, netbsd_x8632::uint16_t);
            deo_field_dump_num!(utmpx, ut_pid, buffer, netbsd_x8632::pid_t);
            deo_field_dump!(utmpx, ut_exit, buffer);
            deo_field_dump!(utmpx, ut_ss, buffer);
            deo_field_dump!(utmpx, ut_tv, buffer);
            deo_field_dump!(utmpx, ut_pad, buffer);
        }
        FixedStructType::Fs_Netbsd_x8664_Lastlog => {
            let lastlog: &netbsd_x8664::lastlog = entry.as_netbsd_x8664_lastlog();

            deo!("netbsd_x8664::lastlog offsets and bytes, size {}", lastlog.size());
            deo_field_dump_num!(lastlog, ll_time, buffer, netbsd_x8664::time_t);
            deo_field_dump!(lastlog, ll_line, buffer);
            deo_field_dump!(lastlog, ll_host, buffer);
        }
        FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
            let lastlogx: &netbsd_x8664::lastlogx = entry.as_netbsd_x8664_lastlogx();

            deo!("netbsd_x8664::lastlogx offsets and bytes, size {}", lastlogx.size());
            deo_field_dump!(lastlogx, ll_tv, buffer);
            deo_field_dump!(lastlogx, ll_line, buffer);
            deo_field_dump!(lastlogx, ll_host, buffer);
            deo_field_dump!(lastlogx, ll_ss, buffer);
        }
        FixedStructType::Fs_Netbsd_x8664_Utmp => {
            let utmp: &netbsd_x8664::utmp = entry.as_netbsd_x8664_utmp();

            deo!("netbsd_x8664::utmp offsets and bytes, size {}", utmp.size());
            deo_field_dump!(utmp, ut_line, buffer);
            deo_field_dump!(utmp, ut_name, buffer);
            deo_field_dump!(utmp, ut_host, buffer);
            deo_field_dump_num!(utmp, ut_time, buffer, netbsd_x8664::time_t);
        }
        FixedStructType::Fs_Netbsd_x8664_Utmpx => {
            let utmpx: &netbsd_x8664::utmpx = entry.as_netbsd_x8664_utmpx();

            deo!("netbsd_x8664::utmpx offsets and bytes, size {}", utmpx.size());
            deo_field_dump!(utmpx, ut_user, buffer);
            deo_field_dump!(utmpx, ut_id, buffer);
            deo_field_dump!(utmpx, ut_line, buffer);
            deo_field_dump!(utmpx, ut_host, buffer);
            deo_field_dump_num!(utmpx, ut_session, buffer, netbsd_x8664::uint16_t);
            deo_field_dump_num!(utmpx, ut_type, buffer, netbsd_x8664::uint16_t);
            deo_field_dump_num!(utmpx, ut_pid, buffer, netbsd_x8664::pid_t);
            deo_field_dump!(utmpx, ut_exit, buffer);
            deo_field_dump!(utmpx, ut_tv, buffer);
            deo_field_dump!(utmpx, ut_pad, buffer);
        }
        FixedStructType::Fs_Openbsd_x86_Lastlog => {
            let lastlog: &openbsd_x86::lastlog = entry.as_openbsd_x86_lastlog();

            deo!("openbsd_x86::lastlog offsets and bytes, size {}", lastlog.size());
            deo_field_dump_num!(lastlog, ll_time, buffer, openbsd_x86::time_t);
            deo_field_dump!(lastlog, ll_line, buffer);
            deo_field_dump!(lastlog, ll_host, buffer);
        }
        FixedStructType::Fs_Openbsd_x86_Utmp => {
            let utmp: &openbsd_x86::utmp = entry.as_openbsd_x86_utmp();

            deo!("openbsd_x86::utmp offsets and bytes, size {}", utmp.size());
            deo_field_dump!(utmp, ut_line, buffer);
            deo_field_dump!(utmp, ut_name, buffer);
            deo_field_dump!(utmp, ut_host, buffer);
            deo_field_dump_num!(utmp, ut_time, buffer, openbsd_x86::time_t);
        }
    }
    defx!("return entry with fixedstruct_type {:?}",
          entry.fixedstruct_type());

    Some(entry)
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write a byte `b` that is `u8` to the `buffer` at index `at`
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_u8 {
    ($buffer:ident, $at:ident, $b:expr) => ({{
        if $at >= $buffer.len() {
            return InfoAsBytes::Fail($at);
        }
        $buffer[$at] = $b;
        $at += 1;
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write a byte `b` that is `i8` to the `buffer` at index `at`
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_i8 {
    ($buffer:ident, $at:ident, $c:expr) => ({{
        let c_: u8 = match ($c).try_into() {
            Ok(val) => val,
            Err(_) => 0,
        };
        set_buffer_at_or_err_u8!($buffer, $at, c_);
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write a byte `b` that is `&[u8]` to the `buffer` at index `at`
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_u8_array {
    ($buffer:ident, $at:ident, $b:expr) => ({{
        for b_ in $b.iter() {
            set_buffer_at_or_err_u8!($buffer, $at, *b_);
        }
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write a `str_` to the `buffer` starting `at` the given index.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_str {
    ($buffer:ident, $at:ident, $str_:expr) => ({{
        for b_ in $str_.bytes() {
            set_buffer_at_or_err_u8!($buffer, $at, b_);
        }
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write a `string_` to the `buffer` starting `at` the given index.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_string {
    ($buffer:ident, $at:ident, $string_:expr) => ({{
        for b_ in $string_.as_bytes() {
            set_buffer_at_or_err_u8!($buffer, $at, *b_);
        }
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write a `cstr` that may be missing ending '\0' to the `buffer` starting at
/// the given index `at` up to `cstr_len` bytes.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_cstrn {
    ($buffer:ident, $at:ident, $cstr:expr) => ({{
        let cstr_sz = std::mem::size_of_val(& $cstr);
        for (i, b_) in $cstr.iter().enumerate() {
            if b_ == &0 || i == cstr_sz {
                break;
            }
            set_buffer_at_or_err_i8!($buffer, $at, *b_);
        }
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `ut_type` that is `i16` to the `buffer` at index`at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_ut_type {
    ($buffer:ident, $at:ident, $ut_type:expr, $type_:ty, $len:ident) => ({{
        #[allow(non_camel_case_types)]
        {
            // sanity check passed number type is 2 bytes or less so `buffer_num` is enough
            assertcp!(std::mem::size_of::<$type_>() <= 2);
        }
        let mut buffer_num = [0u8; 8];
        match &$ut_type {
            n if &0 <= n && n < &$len => {
                set_buffer_at_or_err_str!($buffer, $at, UT_TYPE_VAL_TO_STR[*n as usize]);
            }
            n_ => {
                let num = n_.numtoa(10, &mut buffer_num);
                set_buffer_at_or_err_u8_array!($buffer, $at, num);
            },
        }
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `ut_type` that is `i16` to the `buffer` at index`at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_ut_type_i16 {
    ($buffer:ident, $at:ident, $ut_type:expr) => ({{
        set_buffer_at_or_err_ut_type!($buffer, $at, $ut_type, i16, UT_TYPE_VAL_TO_STR_LEN_i16)
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `ut_type` that is `u16` to the `buffer` at index `at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_ut_type_u16 {
    ($buffer:ident, $at:ident, $ut_type:expr) => ({{
        set_buffer_at_or_err_ut_type!($buffer, $at, $ut_type, u16, UT_TYPE_VAL_TO_STR_LEN_u16)
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `number` value of type `$type_` into `buffer` at index `at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_number {
    ($buffer:ident, $at:ident, $number:expr, $type_:ty) => ({{
        #[allow(non_camel_case_types)]
        {
            // sanity check passed number type is 8 bytes or less so `buffer_num` is enough
            assertcp!(std::mem::size_of::<$type_>() <= 8);
        }
        let mut buffer_num = [0u8; 22];
        // XXX: copy to local variable to avoid packed warning
        let num_val = $number;
        let num = num_val.numtoa(10, &mut buffer_num);
        set_buffer_at_or_err_u8_array!($buffer, $at, num);
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `number` value of type `i64` into `buffer` at index `at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_number_f32 {
    ($buffer:ident, $at:ident, $number:expr, $type_:ty) => ({{
        // XXX: copy to local variable to avoid packed warning
        let num = $number;
        let number_string = format!("{}", num);
        set_buffer_at_or_err_string!($buffer, $at, number_string);
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `number` value of type `i64` into `buffer` at index `at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_number_bin4 {
    ($buffer:ident, $at:ident, $number:expr, $type_:ty) => ({{
        let number_string = format!("0b{:04b}", $number);
        set_buffer_at_or_err_string!($buffer, $at, number_string);
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `number`that is an IPv4 address into `buffer` at index `at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_ipv4 {
    ($buffer:ident, $at:ident, $value:expr) => ({{
        // separate each octet of the IPv4 address
        let o0: u8 = (($value >> 24) & 0xFF) as u8;
        let o1: u8 = (($value >> 16) & 0xFF) as u8;
        let o2: u8 = (($value >> 8) & 0xFF) as u8;
        let o3: u8 = ($value & 0xFF) as u8;
        // write each octet to the buffer
        for (i, b_) in (&[o3, o2, o1, o0]).iter().enumerate() {
            set_buffer_at_or_err_number!($buffer, $at, *b_, u8);
            if i != 3 {
                set_buffer_at_or_err_u8!($buffer, $at, b'.');
            }
        }
    }})
}

/// Helper to [`FixedStruct::as_bytes`].
/// Write the `number`that is an IPv6 address into `buffer` at index `at`.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_buffer_at_or_err_ipv6 {
    ($buffer:ident, $at:ident, $value:expr) => ({{
        for b_ in format!("{:X}:{:X}:{:X}:{:X}",
            &$value[0],
            &$value[1],
            &$value[2],
            &$value[3],
        ).as_str().bytes() {
            set_buffer_at_or_err_u8!($buffer, $at, b_);
        }
    }})
}

/// Helper to [`FixedStruct::from_fixedstructptr`]
macro_rules! tv_or_err_tv_sec {
    ($fixedstructptr: ident, $tv_sec: expr) => ({{
        match $tv_sec.try_into() {
            Ok(val) => val,
            Err(err) => {
                let err_str = format!(
                    "{} failed to convert tv_sec {:?}; {}",
                    stringify!($tv_sec), $tv_sec, err
                );
                de_err!("{}", err_str);
                defx!("return Err {}", err);
                return Result::Err(
                    Error::new(ErrorKind::InvalidData, err_str)
                );
            }
        }
    }})
}

/// Helper to [`FixedStruct::from_fixedstructptr`]
macro_rules! tv_to_datetime_or_err {
    ($tv_sec: ident, $tv_usec: ident, $tz_offset: ident) => ({{
        match convert_tvpair_to_datetime(tv_pair_type($tv_sec, $tv_usec), $tz_offset) {
            Ok(dt) => dt,
            Err(err) => {
                // `convert_tvpair_to_datetime` should have already debug printed an error
                // so don't `de_err!` here
                defx!("return Err {}", err);
                return Result::Err(err);
            }
        }
    }})
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Increase score if the buffer appears to be a valid C string (has
/// typical ASCII characters found in such strings).
macro_rules! score_fixedstruct_cstr {
    ($score:ident, $cstr:expr) => {{
        let cstr_: &CStr = $cstr;
        if !is_empty(cstr_) {
            $score += 1;
            for c in cstr_.to_bytes().iter() {
                match c {
                    // all visible ASCII characters <128
                    &(b' '..=b'~') => $score += 2,
                    // 0xFF is very likely bad data
                    0xFF => $score -= 5,
                    _ => $score -= 3,
                }
            }
        }
        defo!(
            "score_fixedstruct_cstr ({}): score={}",
            stringify!($cstr), $score
        );
    }};
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Decrease score if bytes after the first null byte are not null.
/// Non-null bytes after the first null byte mean the field is not a C string.
macro_rules! score_fixedstruct_cstr_no_data_after_null {
    ($score:ident, $buffer:expr) => {{
        let mut found_null: bool = false;
        for b in $buffer.iter() {
            if *b == 0 {
                found_null = true;
                continue;
            }
            if !found_null {
                continue;
            }
            match b {
                // there is a non-null byte after the prior null bytes
                _ if *b != 0 => $score -= 5,
                _ => {}
            }
        }
        defo!(
            "score_fixedstruct_cstr_no_data_after_null ({}): score={}",
            stringify!($buffer), $score
        );
    }};
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Increase score if the last byte of the buffer is null.
/// Decrease score if the last byte of the buffer is not null.
macro_rules! score_fixedstruct_cstr_null_terminator {
    ($score:ident, $buffer:expr) => {{
        if let Some(b) = $buffer.iter().nth($buffer.len() - 1) {
            if *b == 0 {
                $score += 10;
            } else {
                $score -= 10;
            }
        }
        defo!(
            "score_fixedstruct_cstr_null_terminator ({}): score={}",
            stringify!($buffer), $score
        );
    }};
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Increase score if all bytes are null.
/// Decrease score if any bytes are not null.
macro_rules! score_fixedstruct_buffer_all_null {
    ($score:ident, $buffer:expr) => {{
        let mut all_zero: bool = true;
        if let Some(b) = $buffer.iter().nth($buffer.len() - 1) {
            if *b != 0 {
                $score -= 4;
                all_zero = false;
            }
        }
        if all_zero && $buffer.len() > 0 {
            $score += 10;
        }
        defo!(
            "score_fixedstruct_buffer_all_null ({}): score={}",
            stringify!($buffer), $score
        );
    }};
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Increase score if the value is not zero.
/// Decrease score if the value is zero.
macro_rules! score_fixedstruct_value_not_zero {
    ($score:ident, $value:expr) => {{
        if $value != 0 {
            $score += 10;
        } else {
            $score -= 10;
        }
        defo!(
            "score_fixedstruct_value_not_zero ({}): score={}",
            stringify!($value), $score
        );
    }};
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Increase score if the ut_type is a valid value.
/// Decrease score if the ut_type is a invalid value.
macro_rules! score_fixedstruct_ut_type {
    ($score:ident, $value:expr, $ut_types:expr) => {{
        for ut_type in $ut_types {
            if $value == ut_type {
                $score += 5;
                if $value != 0 {
                    $score += 10;
                }
                break;
            }
        }
        defo!(
            "score_fixedstruct_ut_type ({}): score={}",
            stringify!($value), $score
        );
    }};
}

/// Helper to [`FixedStruct::score_fixedstruct`]
/// Increase score if the bits are valid flag(s).
/// Decrease score if the bits are invalid flag(s).
macro_rules! score_fixedstruct_ac_flags {
    ($score:ident, $value:expr, $flags:expr) => {{
        if $value == 0 {
            $score += 2;
        } else if (!$flags) & $value != 0 {
            $score -= 30;
        } else {
            $score += 5;
        }
        defo!(
            "score_fixedstruct_ac_flags ({}): score={} (value=0b{:08b}, flags=0b{:08b})",
            stringify!($value), $score, $value, $flags
        );
    }};
}

/// Datetime 2000-01-01 00:00:00 is a reasonable past datetime to expect for
/// FixedStruct files.
/// Helper to [`score_fixedstruct_time_range`].
///
/// **TODO:** when were utmp/lastlog/etc. structs first introduced?
const EPOCH_SECOND_LOW: tv_sec_type = 946684800;

/// Datetime 2038-01-19 03:14:06 is a reasonable high datetime to expect for
/// FixedStruct files.
/// Helper to [`score_fixedstruct_time_range`].
const EPOCH_SECOND_HIGH: tv_sec_type = 2147483647;

/// Helper to [`FixedStruct::score_fixedstruct`].
/// Increase the score if the datetime (presumed to be Unix Epoch seconds)
/// is within a reasonable range. Decrease if it's outside of that range.
/// Decrease the score further if the value is zero.
macro_rules! score_fixedstruct_time_range {
    ($score:ident, $value:expr) => {{
        // XXX: copy to local to avoid warning about alignment
        let val = $value;
        let value_as: tv_sec_type = match val.try_into() {
            Ok(val_) => val_,
            Err(_err) => {
                de_err!("failed to convert {:?} to tv_sec_type; {}", val, _err);

                0
            }
        };
        if EPOCH_SECOND_LOW <= value_as && value_as <= EPOCH_SECOND_HIGH {
            $score += 20;
        } else {
            $score -= 30;
        }
        if value_as == 0 {
            $score -= 40;
        }
        defo!(
            "score_fixedstruct_time_range ({}): score={}",
            stringify!($value), $score
        );
    }};
}

impl FixedStruct
{
    /// Create a new `FixedStruct`.
    /// The `buffer` is passed to [`buffer_to_fixedstructptr`] which returns
    /// `None` if only null bytes.
    pub fn new(
        fileoffset: FileOffset,
        tz_offset: &FixedOffset,
        buffer: &[u8],
        fixedstruct_type: FixedStructType,
    ) -> Result<FixedStruct, Error>
    {
        defn!();
        let fs_ptr: FixedStructDynPtr = match buffer_to_fixedstructptr(buffer, fixedstruct_type) {
            Some(val) => val,
            None => {
                defx!("buffer_to_fixedstructptr returned None; return None");
                return Result::Err(
                    Error::new(
                        ErrorKind::InvalidData,
                        "buffer_to_fixedstructptr returned None",
                    )
                );
            }
        };
        defo!("fs_ptr {:?}", fs_ptr);

        FixedStruct::from_fixedstructptr(fileoffset, tz_offset, fs_ptr)
    }

    /// Create a new `FixedStruct` from a `FixedStructDynPtr`.
    pub fn from_fixedstructptr(
        fileoffset: FileOffset,
        tz_offset: &FixedOffset,
        fixedstructptr: FixedStructDynPtr,
    ) -> Result<FixedStruct, Error>
    {
        defn!("fixedstructptr {:?}", fixedstructptr);
        let dt: DateTimeL;
        let tv_sec: tv_sec_type;
        let tv_usec: tv_usec_type;
        let tv_pair: tv_pair_type;
        let fixedstructtype: FixedStructType = fixedstructptr.fixedstruct_type();
        let filetypefixedstruct: FileTypeFixedStruct;
        match fixedstructtype {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => {
                filetypefixedstruct = FileTypeFixedStruct::Utmpx;
                let fixedstructptr: &freebsd_x8664::utmpx = fixedstructptr.as_freebsd_x8664_utmpx();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_sec);
                tv_usec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_usec);
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
                filetypefixedstruct = FileTypeFixedStruct::Lastlog;
                let fixedstructptr: &linux_arm64aarch64::lastlog = fixedstructptr.as_linux_arm64aarch64_lastlog();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ll_time);
                tv_usec = 0;
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
                filetypefixedstruct = FileTypeFixedStruct::Utmpx;
                let fixedstructptr: &linux_arm64aarch64::utmpx = fixedstructptr.as_linux_arm64aarch64_utmpx();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_sec);
                tv_usec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_usec);
            }
            FixedStructType::Fs_Linux_x86_Acct => {
                filetypefixedstruct = FileTypeFixedStruct::Acct;
                let fixedstructptr: &linux_x86::acct = fixedstructptr.as_linux_x86_acct();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ac_btime);
                tv_usec = 0;
            }
            FixedStructType::Fs_Linux_x86_Acct_v3 => {
                filetypefixedstruct = FileTypeFixedStruct::AcctV3;
                let fixedstructptr: &linux_x86::acct_v3 = fixedstructptr.as_linux_x86_acct_v3();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ac_btime);
                tv_usec = 0;
            }
            FixedStructType::Fs_Linux_x86_Lastlog => {
                filetypefixedstruct = FileTypeFixedStruct::Lastlog;
                let fixedstructptr: &linux_x86::lastlog = fixedstructptr.as_linux_x86_lastlog();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ll_time);
                tv_usec = 0;
            }
            FixedStructType::Fs_Linux_x86_Utmpx => {
                filetypefixedstruct = FileTypeFixedStruct::Utmpx;
                let fixedstructptr: &linux_x86::utmpx = fixedstructptr.as_linux_x86_utmpx();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_sec);
                tv_usec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8632_Acct => {
                filetypefixedstruct = FileTypeFixedStruct::Acct;
                let fixedstructptr: &netbsd_x8632::acct = fixedstructptr.as_netbsd_x8632_acct();
                let ac_btime = fixedstructptr.ac_btime;
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, ac_btime);
                tv_usec = 0;
            }
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
                filetypefixedstruct = FileTypeFixedStruct::Lastlogx;
                let fixedstructptr: &netbsd_x8632::lastlogx = fixedstructptr.as_netbsd_x8632_lastlogx();
                // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
                let tv_sec_ = fixedstructptr.ll_tv.tv_sec;
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, tv_sec_);
                // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
                let tv_usec_ = fixedstructptr.ll_tv.tv_usec;
                tv_usec = tv_or_err_tv_sec!(fixedstructptr,  tv_usec_);
            }
            FixedStructType::Fs_Netbsd_x8632_Utmpx => {
                filetypefixedstruct = FileTypeFixedStruct::Utmpx;
                let fixedstructptr: &netbsd_x8632::utmpx = fixedstructptr.as_netbsd_x8632_utmpx();
                // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
                let tv_sec_ = fixedstructptr.ut_tv.tv_sec;
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, tv_sec_);
                // XXX: copy to local to avoid Issue #82523; #rust-lang/rust/82523
                let tv_usec_ = fixedstructptr.ut_tv.tv_usec;
                tv_usec = tv_or_err_tv_sec!(fixedstructptr, tv_usec_);
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlog => {
                filetypefixedstruct = FileTypeFixedStruct::Lastlog;
                let fixedstructptr: &netbsd_x8664::lastlog = fixedstructptr.as_netbsd_x8664_lastlog();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ll_time);
                tv_usec = 0;
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
                filetypefixedstruct = FileTypeFixedStruct::Lastlogx;
                let fixedstructptr: &netbsd_x8664::lastlogx = fixedstructptr.as_netbsd_x8664_lastlogx();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ll_tv.tv_sec);
                tv_usec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ll_tv.tv_usec);
            }
            FixedStructType::Fs_Netbsd_x8664_Utmp => {
                filetypefixedstruct = FileTypeFixedStruct::Utmp;
                let fixedstructptr: &netbsd_x8664::utmp = fixedstructptr.as_netbsd_x8664_utmp();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_time);
                tv_usec = 0;
            }
            FixedStructType::Fs_Netbsd_x8664_Utmpx => {
                filetypefixedstruct = FileTypeFixedStruct::Utmpx;
                let fixedstructptr: &netbsd_x8664::utmpx = fixedstructptr.as_netbsd_x8664_utmpx();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_sec);
                tv_usec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_tv.tv_usec);
            }
            FixedStructType::Fs_Openbsd_x86_Lastlog => {
                filetypefixedstruct = FileTypeFixedStruct::Lastlog;
                let fixedstructptr: &openbsd_x86::lastlog = fixedstructptr.as_openbsd_x86_lastlog();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ll_time);
                tv_usec = 0;
            }
            FixedStructType::Fs_Openbsd_x86_Utmp => {
                filetypefixedstruct = FileTypeFixedStruct::Utmp;
                let fixedstructptr: &openbsd_x86::utmp = fixedstructptr.as_openbsd_x86_utmp();
                tv_sec = tv_or_err_tv_sec!(fixedstructptr, fixedstructptr.ut_time);
                tv_usec = 0;
            }
        }
        dt = tv_to_datetime_or_err!(tv_sec, tv_usec, tz_offset);
        defo!("FixedStruct {{ dt {:?} }}", dt);
        tv_pair = tv_pair_type(tv_sec, tv_usec);
        defx!("FixedStruct {{ tv_pair {:?} }}", tv_pair);
        Result::Ok(
            FixedStruct {
                fixedstructptr,
                fixedstructtype,
                filetypefixedstruct,
                fileoffset,
                dt,
                tv_pair,
            }
        )
    }

    pub fn len(self: &FixedStruct) -> usize
    {
        self.fixedstructptr.size()
    }

    /// Clippy recommends `fn is_empty` since there is a `len()`.
    pub fn is_empty(self: &FixedStruct) -> bool
    {
        self.len() == 0
    }

    /// [`FileOffset`] at beginning of the `FixedStruct` (inclusive).
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    pub const fn fileoffset_begin(self: &FixedStruct) -> FileOffset
    {
        self.fileoffset
    }

    /// [`FileOffset`] at one byte past ending of the `FixedStruct` (exclusive).
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    pub fn fileoffset_end(self: &FixedStruct) -> FileOffset
    {
        self.fileoffset + (self.len() as FileOffset)
    }

    /// First [`BlockOffset`] of underlying [`Block`s] for the given
    /// [`BlockSz`].
    ///
    /// [`BlockOffset`]: crate::readers::blockreader::BlockOffset
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`BlockSz`]: crate::readers::blockreader::BlockSz
    pub const fn blockoffset_begin(&self, blocksz: BlockSz) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(
            self.fileoffset_begin(),
            blocksz
        )
    }

    /// Last [`BlockOffset`] of underlying [`Block`s] for the given
    /// [`BlockSz`] (inclusive).
    ///
    /// [`BlockOffset`s]: crate::readers::blockreader::BlockOffset
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`BlockSz`]: crate::readers::blockreader::BlockSz
    pub fn blockoffset_end(&self, blocksz: BlockSz) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(
            self.fileoffset_end(),
            blocksz
        )
    }

    /// Return a reference to [`self.dt`].
    ///
    /// [`self.dt`]: FixedStruct::dt
    pub const fn dt(self: &FixedStruct) -> &DateTimeL
    {
        &self.dt
    }

    /// Return a reference to [`self.tv_pair`].
    ///
    /// [`self.tv_pair`]: FixedStruct::tv_pair
    pub const fn tv_pair(self: &FixedStruct) -> &tv_pair_type
    {
        &self.tv_pair
    }

    /// Create a score for this FixedStruct entry
    ///
    /// The scoring system is a simple heuristic to determine the likelihood that
    /// the FixedStruct entry is a valid version of the given `FixedStructType`.
    /// The particulars of the score are just a guess; they seemed to work well in limited
    /// testing.
    ///
    /// Scoring is necessary because a file's exact fixedstruct type often
    /// cannot be determined from the name and file size alone.
    ///
    /// This function is used by [`FixedStructReader::score_file`].
    ///
    /// [`FixedStructReader::score_file`]: crate::readers::fixedstructreader::FixedStructReader::score_file
    // XXX: I considered also checking for matching platform. However, the platforms
    //      embedded in each namespace, e.g. `linux_x86`, may be the same for many
    //      platforms, e.g. `linux_x86` may be used for both 32-bit and 64-bit x86, and on various
    //      ARM and RISC architectures.
    pub fn score_fixedstruct(fixedstructptr: &FixedStructDynPtr, bonus: Score) -> Score {
        defn!();
        defo!("fixedstructptr.entry_type() = {:?}", fixedstructptr.fixedstruct_type());
        defo!("fixedstructptr.size() = {:?}", fixedstructptr.size());
        let mut score: Score = 0;
        defo!("score = {:?}", score);
        if bonus > 0 {
            score += bonus;
            defo!("score += bonus {:?}", score);
        }
        match fixedstructptr.fixedstruct_type() {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => {
                let utmpx: &freebsd_x8664::utmpx = fixedstructptr.as_freebsd_x8664_utmpx();
                score_fixedstruct_cstr!(score, utmpx.ut_user());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_user);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_user);

                score_fixedstruct_cstr!(score, utmpx.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_line);

                score_fixedstruct_cstr!(score, utmpx.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_host);

                score_fixedstruct_time_range!(score, utmpx.ut_tv.tv_sec);

                score_fixedstruct_buffer_all_null!(score, utmpx.__ut_spare);

                score_fixedstruct_ut_type!(score, utmpx.ut_type, freebsd_x8664::UT_TYPES);
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
                let lastlog: &linux_arm64aarch64::lastlog = fixedstructptr.as_linux_arm64aarch64_lastlog();

                score_fixedstruct_cstr!(score, lastlog.ll_line());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_line);

                score_fixedstruct_cstr!(score, lastlog.ll_host());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_host);
                score_fixedstruct_cstr_null_terminator!(score, lastlog.ll_host);

                score_fixedstruct_time_range!(score, lastlog.ll_time);
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
                let utmpx: &linux_arm64aarch64::utmpx = fixedstructptr.as_linux_arm64aarch64_utmpx();

                score_fixedstruct_cstr!(score, utmpx.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_line);

                score_fixedstruct_cstr!(score, utmpx.ut_user());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_user);

                score_fixedstruct_cstr!(score, utmpx.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_host);

                score_fixedstruct_time_range!(score, utmpx.ut_tv.tv_sec);

                score_fixedstruct_buffer_all_null!(score, utmpx.__glibc_reserved);

                score_fixedstruct_ut_type!(score, utmpx.ut_type, linux_arm64aarch64::UT_TYPES);
            }
            FixedStructType::Fs_Linux_x86_Acct => {
                let acct: &linux_x86::acct = fixedstructptr.as_linux_x86_acct();

                score_fixedstruct_cstr!(score, acct.ac_comm());
                score_fixedstruct_cstr_no_data_after_null!(score, acct.ac_comm);

                score_fixedstruct_time_range!(score, acct.ac_btime);

                score_fixedstruct_buffer_all_null!(score, acct.ac_pad);

                score_fixedstruct_ac_flags!(score, acct.ac_flag, linux_x86::AC_FLAGS_MASK);
            }
            FixedStructType::Fs_Linux_x86_Acct_v3 => {
                let acct: &linux_x86::acct_v3 = fixedstructptr.as_linux_x86_acct_v3();

                score_fixedstruct_cstr!(score, acct.ac_comm());
                score_fixedstruct_cstr_no_data_after_null!(score, acct.ac_comm);

                score_fixedstruct_time_range!(score, acct.ac_btime);

                score_fixedstruct_value_not_zero!(score, acct.ac_version);

                score_fixedstruct_ac_flags!(score, acct.ac_flag, linux_x86::AC_FLAGS_MASK);
            }
            FixedStructType::Fs_Linux_x86_Lastlog => {
                let lastlog: &linux_x86::lastlog = fixedstructptr.as_linux_x86_lastlog();

                score_fixedstruct_time_range!(score, lastlog.ll_time);

                score_fixedstruct_cstr!(score, lastlog.ll_line());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_line);

                score_fixedstruct_cstr!(score, lastlog.ll_host());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_host);
                score_fixedstruct_cstr_null_terminator!(score, lastlog.ll_host);
            }
            FixedStructType::Fs_Linux_x86_Utmpx => {
                let utmpx: &linux_x86::utmpx = fixedstructptr.as_linux_x86_utmpx();

                score_fixedstruct_cstr!(score, utmpx.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_line);

                score_fixedstruct_cstr!(score, utmpx.ut_id());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_id);

                score_fixedstruct_cstr!(score, utmpx.ut_user());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_user);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_user);

                score_fixedstruct_cstr!(score, utmpx.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_host);

                score_fixedstruct_time_range!(score, utmpx.ut_tv.tv_sec);

                score_fixedstruct_buffer_all_null!(score, utmpx.__glibc_reserved);

                score_fixedstruct_ut_type!(score, utmpx.ut_type, linux_x86::UT_TYPES);
            }
            FixedStructType::Fs_Netbsd_x8632_Acct => {
                let acct: &netbsd_x8632::acct = fixedstructptr.as_netbsd_x8632_acct();

                score_fixedstruct_cstr!(score, acct.ac_comm());
                score_fixedstruct_cstr_no_data_after_null!(score, acct.ac_comm);

                score_fixedstruct_time_range!(score, acct.ac_btime);

                score_fixedstruct_buffer_all_null!(score, acct.__gap1);
                score_fixedstruct_buffer_all_null!(score, acct.__gap3);

                score_fixedstruct_ac_flags!(score, acct.ac_flag, netbsd_x8632::AC_FLAGS_MASK);
            }
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
                let lastlogx: &netbsd_x8632::lastlogx = fixedstructptr.as_netbsd_x8632_lastlogx();

                score_fixedstruct_cstr!(score, lastlogx.ll_line());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlogx.ll_line);

                score_fixedstruct_cstr!(score, lastlogx.ll_host());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlogx.ll_host);
                score_fixedstruct_cstr_null_terminator!(score, lastlogx.ll_host);

                score_fixedstruct_time_range!(score, lastlogx.ll_tv.tv_sec);
            }
            FixedStructType::Fs_Netbsd_x8632_Utmpx => {
                let utmpx: &netbsd_x8632::utmpx = fixedstructptr.as_netbsd_x8632_utmpx();

                score_fixedstruct_cstr!(score, utmpx.ut_name());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_name);

                score_fixedstruct_cstr!(score, utmpx.ut_id());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_id);

                score_fixedstruct_cstr!(score, utmpx.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_line);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_line);

                score_fixedstruct_cstr!(score, utmpx.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_host);

                score_fixedstruct_time_range!(score, utmpx.ut_tv.tv_sec);

                score_fixedstruct_buffer_all_null!(score, utmpx.ut_pad);

                score_fixedstruct_ut_type!(score, utmpx.ut_type, netbsd_x8632::UT_TYPES);
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlog => {
                let lastlog: &netbsd_x8664::lastlog = fixedstructptr.as_netbsd_x8664_lastlog();

                score_fixedstruct_cstr!(score, lastlog.ll_line());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_line);

                score_fixedstruct_cstr!(score, lastlog.ll_host());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_host);

                score_fixedstruct_time_range!(score, lastlog.ll_time);
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
                let lastlogx: &netbsd_x8664::lastlogx = fixedstructptr.as_netbsd_x8664_lastlogx();

                score_fixedstruct_cstr!(score, lastlogx.ll_line());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlogx.ll_line);

                score_fixedstruct_cstr!(score, lastlogx.ll_host());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlogx.ll_host);
                score_fixedstruct_cstr_null_terminator!(score, lastlogx.ll_host);

                score_fixedstruct_time_range!(score, lastlogx.ll_tv.tv_sec);
            }
            FixedStructType::Fs_Netbsd_x8664_Utmp => {
                let utmp: &netbsd_x8664::utmp = fixedstructptr.as_netbsd_x8664_utmp();

                score_fixedstruct_cstr!(score, utmp.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmp.ut_line);

                score_fixedstruct_cstr!(score, utmp.ut_name());
                score_fixedstruct_cstr_no_data_after_null!(score, utmp.ut_name);

                score_fixedstruct_cstr!(score, utmp.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmp.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmp.ut_host);

                score_fixedstruct_time_range!(score, utmp.ut_time);
            }
            FixedStructType::Fs_Netbsd_x8664_Utmpx => {
                let utmpx: &netbsd_x8664::utmpx = fixedstructptr.as_netbsd_x8664_utmpx();

                score_fixedstruct_cstr!(score, utmpx.ut_user());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_user);

                score_fixedstruct_cstr!(score, utmpx.ut_id());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_id);

                score_fixedstruct_cstr!(score, utmpx.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_line);

                score_fixedstruct_cstr!(score, utmpx.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmpx.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmpx.ut_host);

                score_fixedstruct_time_range!(score, utmpx.ut_tv.tv_sec);

                score_fixedstruct_buffer_all_null!(score, utmpx.__gap1);
                score_fixedstruct_buffer_all_null!(score, utmpx.ut_pad);

                score_fixedstruct_ut_type!(score, utmpx.ut_type, netbsd_x8664::UT_TYPES);
            }
            FixedStructType::Fs_Openbsd_x86_Lastlog => {
                let lastlog: &openbsd_x86::lastlog = fixedstructptr.as_openbsd_x86_lastlog();

                score_fixedstruct_time_range!(score, lastlog.ll_time);

                score_fixedstruct_cstr!(score, lastlog.ll_line());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_line);

                score_fixedstruct_cstr!(score, lastlog.ll_host());
                score_fixedstruct_cstr_no_data_after_null!(score, lastlog.ll_host);
                score_fixedstruct_cstr_null_terminator!(score, lastlog.ll_host);
            }
            FixedStructType::Fs_Openbsd_x86_Utmp => {
                let utmp: &openbsd_x86::utmp = fixedstructptr.as_openbsd_x86_utmp();

                score_fixedstruct_cstr!(score, utmp.ut_line());
                score_fixedstruct_cstr_no_data_after_null!(score, utmp.ut_line);

                score_fixedstruct_cstr!(score, utmp.ut_name());
                score_fixedstruct_cstr_no_data_after_null!(score, utmp.ut_name);

                score_fixedstruct_cstr!(score, utmp.ut_host());
                score_fixedstruct_cstr_no_data_after_null!(score, utmp.ut_host);
                score_fixedstruct_cstr_null_terminator!(score, utmp.ut_host);

                score_fixedstruct_time_range!(score, utmp.ut_time);
            }
        }

        defx!("return score {}", score);

        score
    }

    /// Efficient function to copy the `FixedStruct` into a single re-usable
    /// buffer for printing.
    ///
    /// Copy the `FixedStruct` into the passed `buffer` as printable bytes.
    /// When successful, returns a [`InfoAsBytes::Ok`] variant with
    /// number of bytes copied, start index of datetime substring, and
    /// end index of datetime substring.
    ///
    /// If copying fails, returns a [`InfoAsBytes::Fail`] variant with
    /// number of bytes copied.
    pub fn as_bytes(self: &FixedStruct, buffer: &mut [u8]) -> InfoAsBytes
    {
        let entry: &FixedStructDynPtr = &self.fixedstructptr;
        let mut at: usize = 0;
        let dt_beg: BufIndex;
        let dt_end: BufIndex;

        match entry.fixedstruct_type() {
            FixedStructType::Fs_Freebsd_x8664_Utmpx => {
                let utmpx: &freebsd_x8664::utmpx = entry.as_freebsd_x8664_utmpx();

                // ut_type
                set_buffer_at_or_err_str!(buffer, at, "ut_type ");
                set_buffer_at_or_err_ut_type_i16!(buffer, at, utmpx.ut_type);
                // ut_tv
                set_buffer_at_or_err_str!(buffer, at,  " ut_tv ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_sec, freebsd_x8664::time_t);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_usec, freebsd_x8664::subseconds_t);
                dt_end = at;
                // ut_id
                set_buffer_at_or_err_str!(buffer, at, " ut_id '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_id);
                // ut_pid
                set_buffer_at_or_err_str!(buffer, at, "' ut_pid ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_pid, freebsd_x8664::pid_t);
                // ut_user
                set_buffer_at_or_err_str!(buffer, at, " ut_user '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_user);
                // ut_line
                set_buffer_at_or_err_str!(buffer, at, "' ut_line ");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_line);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_host);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Lastlog => {
                let lastlog: &linux_arm64aarch64::lastlog = entry.as_linux_arm64aarch64_lastlog();

                // ll_time
                set_buffer_at_or_err_str!(buffer, at, "ll_time ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, lastlog.ll_time, linux_arm64aarch64::ll_time_t);
                dt_end = at;
                // ll_line
                set_buffer_at_or_err_str!(buffer, at, " ll_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_line);
                // ll_host
                set_buffer_at_or_err_str!(buffer, at, "' ll_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_host);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Linux_Arm64Aarch64_Utmpx => {
                let utmpx: &linux_arm64aarch64::utmpx = entry.as_linux_arm64aarch64_utmpx();

                // ut_type
                set_buffer_at_or_err_str!(buffer, at, "ut_type ");
                set_buffer_at_or_err_ut_type_i16!(buffer, at, utmpx.ut_type);
                // ut_pid
                set_buffer_at_or_err_str!(buffer, at, " ut_pid ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_pid, linux_arm64aarch64::pid_t);
                // ut_line
                set_buffer_at_or_err_str!(buffer, at, " ut_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_line);
                // ut_id
                set_buffer_at_or_err_str!(buffer, at, "' ut_id '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_id);
                // ut_user
                set_buffer_at_or_err_str!(buffer, at, "' ut_user '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_user);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_host);
                // ut_exit
                set_buffer_at_or_err_str!(buffer, at, "' ut_exit ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit, i32);
                // ut_session
                set_buffer_at_or_err_str!(buffer, at, " ut_session '");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_session, i64);
                // ut_tv.tv_sec
                set_buffer_at_or_err_str!(buffer, at, "' ut_tv ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_sec, i64);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                // ut_tv.tv_usec
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_usec, i64);
                dt_end = at;
                // if ut_addr_v6 is all zeros then this is an IPv4 address
                if utmpx.ut_addr_v6[1..4].iter().all(|&x| x == 0) {
                    // ut_addr
                    set_buffer_at_or_err_str!(buffer, at, " ut_addr ");
                    set_buffer_at_or_err_ipv4!(buffer, at, utmpx.ut_addr_v6[0]);
                } else {
                    // ut_addr_v6
                    set_buffer_at_or_err_str!(buffer, at, " ut_addr_v6 ");
                    set_buffer_at_or_err_ipv6!(buffer, at, utmpx.ut_addr_v6);
                }
            }
            FixedStructType::Fs_Linux_x86_Acct => {
                let acct: &linux_x86::acct = entry.as_linux_x86_acct();

                // ac_flag
                set_buffer_at_or_err_str!(buffer, at, "ac_flag ");
                set_buffer_at_or_err_number_bin4!(buffer, at, acct.ac_flag, linux_x86::c_char);
                if acct.ac_flag != 0 {
                    set_buffer_at_or_err_str!(buffer, at, " (");
                    if acct.ac_flag & linux_x86::AFORK != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "AFORK|");
                    }
                    if acct.ac_flag & linux_x86::ASU != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ASU|");
                    }
                    if acct.ac_flag & linux_x86::ACOMPAT != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ACOMPAT|");
                    }
                    if acct.ac_flag & linux_x86::ACORE != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ACORE|");
                    }
                    if acct.ac_flag & linux_x86::AXSIG != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "AXSIG|");
                    }
                    // overwrite trailing '|' with ')'
                    if buffer[at - 1] == b'|' {
                        at -= 1;
                    }
                    set_buffer_at_or_err_str!(buffer, at, ")");
                }
                // ac_uid
                set_buffer_at_or_err_str!(buffer, at, " ac_uid ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_uid, linux_x86::uint16_t);
                // ac_gid
                set_buffer_at_or_err_str!(buffer, at, " ac_gid ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_gid, linux_x86::uint16_t);
                // ac_tty
                set_buffer_at_or_err_str!(buffer, at, " ac_tty ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_tty, linux_x86::uint16_t);
                // ac_btime
                set_buffer_at_or_err_str!(buffer, at, " ac_btime ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, acct.ac_btime, linux_x86::b_time_t);
                dt_end = at;
                // ac_utime
                set_buffer_at_or_err_str!(buffer, at, " ac_utime ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_utime, linux_x86::comp_t);
                // ac_stime
                set_buffer_at_or_err_str!(buffer, at, " ac_stime ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_stime, linux_x86::comp_t);
                // ac_etime
                set_buffer_at_or_err_str!(buffer, at, " ac_etime ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_etime, linux_x86::comp_t);
                // ac_mem
                set_buffer_at_or_err_str!(buffer, at, " ac_mem ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_mem, linux_x86::comp_t);
                // ac_io
                set_buffer_at_or_err_str!(buffer, at, " ac_io ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_io, linux_x86::comp_t);
                // ac_rw
                set_buffer_at_or_err_str!(buffer, at, " ac_rw ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_rw, linux_x86::comp_t);
                // ac_minflt
                set_buffer_at_or_err_str!(buffer, at, " ac_minflt ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_minflt, linux_x86::comp_t);
                // ac_majflt
                set_buffer_at_or_err_str!(buffer, at, " ac_majflt ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_majflt, linux_x86::comp_t);
                // ac_swaps
                set_buffer_at_or_err_str!(buffer, at, " ac_swaps ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_swaps, linux_x86::comp_t);
                // ac_exitcode
                set_buffer_at_or_err_str!(buffer, at, " ac_exitcode ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_exitcode, u32);
                // ac_comm
                set_buffer_at_or_err_str!(buffer, at, " ac_comm '");
                set_buffer_at_or_err_cstrn!(buffer, at, acct.ac_comm);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Linux_x86_Acct_v3 => {
                let acct_v3: &linux_x86::acct_v3 = entry.as_linux_x86_acct_v3();

                // ac_flag
                set_buffer_at_or_err_str!(buffer, at, "ac_flag ");
                set_buffer_at_or_err_number_bin4!(buffer, at, acct_v3.ac_flag, linux_x86::c_char);
                if acct_v3.ac_flag != 0 {
                    set_buffer_at_or_err_str!(buffer, at, " (");
                    if acct_v3.ac_flag & linux_x86::AFORK != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "AFORK|");
                    }
                    if acct_v3.ac_flag & linux_x86::ASU != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ASU|");
                    }
                    if acct_v3.ac_flag & linux_x86::ACOMPAT != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ACOMPAT|");
                    }
                    if acct_v3.ac_flag & linux_x86::ACORE != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ACORE|");
                    }
                    if acct_v3.ac_flag & linux_x86::AXSIG != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "AXSIG|");
                    }
                    // overwrite trailing '|' with ')'
                    if buffer[at - 1] == b'|' {
                        at -= 1;
                    }
                    set_buffer_at_or_err_str!(buffer, at, ")");
                }
                // ac_version
                set_buffer_at_or_err_str!(buffer, at, " ac_version ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_version, linux_x86::c_char);
                // ac_tty
                set_buffer_at_or_err_str!(buffer, at, " ac_tty ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_tty, u16);
                // ac_exitcode
                set_buffer_at_or_err_str!(buffer, at, " ac_exitcode ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_exitcode, u32);
                // ac_uid
                set_buffer_at_or_err_str!(buffer, at, " ac_uid ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_uid, u32);
                // ac_gid
                set_buffer_at_or_err_str!(buffer, at, " ac_gid ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_gid, u32);
                // ac_pid
                set_buffer_at_or_err_str!(buffer, at, " ac_pid ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_pid, u32);
                // ac_ppid
                set_buffer_at_or_err_str!(buffer, at, " ac_ppid ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_ppid, u32);
                // ac_btime
                set_buffer_at_or_err_str!(buffer, at, " ac_btime ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_btime, linux_x86::b_time_t);
                dt_end = at;
                // ac_etime
                set_buffer_at_or_err_str!(buffer, at, " ac_etime ");
                set_buffer_at_or_err_number_f32!(buffer, at, acct_v3.ac_etime, f32);
                // ac_utime
                set_buffer_at_or_err_str!(buffer, at, " ac_utime ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_utime, linux_x86::comp_t);
                // ac_stime
                set_buffer_at_or_err_str!(buffer, at, " ac_stime ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_stime, linux_x86::comp_t);
                // ac_mem
                set_buffer_at_or_err_str!(buffer, at, " ac_mem ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_mem, linux_x86::comp_t);
                // ac_io
                set_buffer_at_or_err_str!(buffer, at, " ac_io ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_io, linux_x86::comp_t);
                // ac_rw
                set_buffer_at_or_err_str!(buffer, at, " ac_rw ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_rw, linux_x86::comp_t);
                // ac_minflt
                set_buffer_at_or_err_str!(buffer, at, " ac_minflt ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_minflt, linux_x86::comp_t);
                // ac_majflt
                set_buffer_at_or_err_str!(buffer, at, " ac_majflt ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_majflt, linux_x86::comp_t);
                // ac_swaps
                set_buffer_at_or_err_str!(buffer, at, " ac_swaps ");
                set_buffer_at_or_err_number!(buffer, at, acct_v3.ac_swaps, linux_x86::comp_t);
                // ac_comm
                set_buffer_at_or_err_str!(buffer, at, " ac_comm '");
                set_buffer_at_or_err_cstrn!(buffer, at, acct_v3.ac_comm);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Linux_x86_Lastlog => {
                let lastlog: &linux_x86::lastlog = entry.as_linux_x86_lastlog();

                // ll_time
                set_buffer_at_or_err_str!(buffer, at, "ll_time ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, lastlog.ll_time, linux_x86::ll_time_t);
                dt_end = at;
                // ll_line
                set_buffer_at_or_err_str!(buffer, at, " ll_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_line);
                // ll_host
                set_buffer_at_or_err_str!(buffer, at, "' ll_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_host);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Linux_x86_Utmpx => {
                let utmpx: &linux_x86::utmpx = entry.as_linux_x86_utmpx();

                // ut_type
                set_buffer_at_or_err_str!(buffer, at, "ut_type ");
                set_buffer_at_or_err_ut_type_i16!(buffer, at, utmpx.ut_type);
                // ut_pid
                set_buffer_at_or_err_str!(buffer, at, " ut_pid ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_pid, linux_x86::pid_t);
                // ut_line
                set_buffer_at_or_err_str!(buffer, at, " ut_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_line);
                // ut_id
                set_buffer_at_or_err_str!(buffer, at, "' ut_id '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_id);
                // ut_user
                set_buffer_at_or_err_str!(buffer, at, "' ut_user '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_user);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_host);
                // ut_exit.e_termination
                set_buffer_at_or_err_str!(buffer, at, "' e_termination ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit.e_termination, i16);
                // ut_exit.e_exit
                set_buffer_at_or_err_str!(buffer, at, " e_exit ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit.e_exit, i16);
                // ut_session
                set_buffer_at_or_err_str!(buffer, at, " ut_session '");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_session, i32);
                // ut_tv.tv_sec
                set_buffer_at_or_err_str!(buffer, at, "' ut_xtime ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_sec, i32);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                // ut_tv.tv_usec
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_usec, i32);
                dt_end = at;
                // if ut_addr_v6 is all zeros then this is an IPv4 address
                if utmpx.ut_addr_v6[1..4].iter().all(|&x| x == 0) {
                    // ut_addr
                    set_buffer_at_or_err_str!(buffer, at, " ut_addr ");
                    set_buffer_at_or_err_ipv4!(buffer, at, utmpx.ut_addr_v6[0]);
                } else {
                    // ut_addr_v6
                    set_buffer_at_or_err_str!(buffer, at, " ut_addr_v6 ");
                    set_buffer_at_or_err_ipv6!(buffer, at, utmpx.ut_addr_v6);
                }
            }
            FixedStructType::Fs_Netbsd_x8632_Acct => {
                let acct: &netbsd_x8632::acct = entry.as_netbsd_x8632_acct();

                // ac_comm
                set_buffer_at_or_err_str!(buffer, at, "ac_comm '");
                set_buffer_at_or_err_cstrn!(buffer, at, acct.ac_comm);
                // ac_utime
                set_buffer_at_or_err_str!(buffer, at, "' ac_utime ");
                // TODO: need to handle `comp_t` values specially; from `acct.h`:
                //       /*
                //        * Accounting structures; these use a comp_t type which is a 3 bits base 8
                //        * exponent, 13 bit fraction ``floating point'' number.  Units are 1/AHZ
                //        * seconds.
                //        */
                set_buffer_at_or_err_number!(buffer, at, acct.ac_utime, netbsd_x8632::comp_t);
                // ac_stime
                set_buffer_at_or_err_str!(buffer, at, " ac_stime ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_stime, netbsd_x8632::comp_t);
                // ac_etime
                set_buffer_at_or_err_str!(buffer, at, " ac_etime ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_etime, netbsd_x8632::comp_t);
                // ac_btime
                set_buffer_at_or_err_str!(buffer, at, " ac_btime ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, acct.ac_btime, netbsd_x8632::time_t);
                dt_end = at;
                // ac_uid
                set_buffer_at_or_err_str!(buffer, at, " ac_uid ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_uid, netbsd_x8632::uid_t);
                // ac_gid
                set_buffer_at_or_err_str!(buffer, at, " ac_gid ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_gid, netbsd_x8632::gid_t);
                // ac_mem
                set_buffer_at_or_err_str!(buffer, at, " ac_mem ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_mem, netbsd_x8632::uint16_t);
                // ac_io
                set_buffer_at_or_err_str!(buffer, at, " ac_io ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_io, netbsd_x8632::comp_t);
                // ac_tty
                set_buffer_at_or_err_str!(buffer, at, " ac_tty ");
                set_buffer_at_or_err_number!(buffer, at, acct.ac_tty, netbsd_x8632::dev_t);
                // ac_flag is a bitfield
                // see https://man.netbsd.org/acct.5
                set_buffer_at_or_err_str!(buffer, at, " ac_flag ");
                set_buffer_at_or_err_number_bin4!(buffer, at, acct.ac_flag, netbsd_x8632::uint8_t);
                if acct.ac_flag != 0 {
                    set_buffer_at_or_err_str!(buffer, at, " (");
                    if acct.ac_flag & netbsd_x8632::AFORK != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "AFORK|");
                    }
                    if acct.ac_flag & netbsd_x8632::ASU != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ASU|");
                    }
                    if acct.ac_flag & netbsd_x8632::ACOMPAT != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ACOMPAT|");
                    }
                    if acct.ac_flag & netbsd_x8632::ACORE != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "ACORE|");
                    }
                    if acct.ac_flag & netbsd_x8632::AXSIG != 0 {
                        set_buffer_at_or_err_str!(buffer, at, "AXSIG|");
                    }
                    // overwrite trailing '|' with ')'
                    if buffer[at - 1] == b'|' {
                        at -= 1;
                    }
                    set_buffer_at_or_err_str!(buffer, at, ")");
                }
            }
            FixedStructType::Fs_Netbsd_x8632_Lastlogx => {
                let lastlogx: &netbsd_x8632::lastlogx = entry.as_netbsd_x8632_lastlogx();

                // ll_tv.tv_sec
                set_buffer_at_or_err_str!(buffer, at, "ll_tv ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, lastlogx.ll_tv.tv_sec, i64);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                // ll_tv.tv_usec
                set_buffer_at_or_err_number!(buffer, at, lastlogx.ll_tv.tv_usec, i32);
                dt_end = at;
                // ll_line
                set_buffer_at_or_err_str!(buffer, at, " ll_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlogx.ll_line);
                // ll_host
                set_buffer_at_or_err_str!(buffer, at, "' ll_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlogx.ll_host);
                // ll_ss
                set_buffer_at_or_err_str!(buffer, at, "' ll_ss ");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlogx.ll_ss);
            }
            FixedStructType::Fs_Netbsd_x8632_Utmpx => {
                let utmpx: &netbsd_x8632::utmpx = entry.as_netbsd_x8632_utmpx();

                // ut_name
                set_buffer_at_or_err_str!(buffer, at, "ut_name '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_name);
                // ut_id
                set_buffer_at_or_err_str!(buffer, at, "' ut_id '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_id);
                // ut_line
                set_buffer_at_or_err_str!(buffer, at, "' ut_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_line);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_host);
                // ut_session
                set_buffer_at_or_err_str!(buffer, at, "' ut_session '");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_session, netbsd_x8632::uint16_t);
                // ut_type
                set_buffer_at_or_err_str!(buffer, at, "' ut_type ");
                set_buffer_at_or_err_ut_type_u16!(buffer, at, utmpx.ut_type);
                // ut_pid
                set_buffer_at_or_err_str!(buffer, at, " ut_pid ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_pid, netbsd_x8632::pid_t);
                // ut_exit.e_termination
                set_buffer_at_or_err_str!(buffer, at, " e_termination ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit.e_termination, netbsd_x8632::uint16_t);
                // ut_exit.e_exit
                set_buffer_at_or_err_str!(buffer, at, " e_exit ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit.e_exit, netbsd_x8632::uint16_t);
                // ut_ss
                set_buffer_at_or_err_str!(buffer, at, " ut_ss '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_ss);
                // ut_tv.tv_sec
                set_buffer_at_or_err_str!(buffer, at, "' ut_tv ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_sec, i64);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                // ut_tv.tv_usec
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_usec, i32);
                dt_end = at;
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlog => {
                let lastlog: &netbsd_x8664::lastlog = entry.as_netbsd_x8664_lastlog();

                // ll_time
                set_buffer_at_or_err_str!(buffer, at, "ll_time ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, lastlog.ll_time, netbsd_x8664::time_t);
                dt_end = at;
                // ll_line
                set_buffer_at_or_err_str!(buffer, at, " ll_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_line);
                // ll_host
                set_buffer_at_or_err_str!(buffer, at, "' ll_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_host);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Netbsd_x8664_Lastlogx => {
                let lastlogx: &netbsd_x8664::lastlogx = entry.as_netbsd_x8664_lastlogx();

                // ll_tv.tv_sec
                set_buffer_at_or_err_str!(buffer, at, "ll_tv ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, lastlogx.ll_tv.tv_sec, i64);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                // ll_tv.tv_usec
                set_buffer_at_or_err_number!(buffer, at, lastlogx.ll_tv.tv_usec, i32);
                dt_end = at;
                // ll_line
                set_buffer_at_or_err_str!(buffer, at, " ll_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlogx.ll_line);
                // ll_host
                set_buffer_at_or_err_str!(buffer, at, "' ll_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlogx.ll_host);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
                // XXX: ll_ss is not printable
            }
            FixedStructType::Fs_Netbsd_x8664_Utmp => {
                let utmp: &netbsd_x8664::utmp = entry.as_netbsd_x8664_utmp();

                // ut_line
                set_buffer_at_or_err_str!(buffer, at, "ut_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmp.ut_line);
                // ut_name
                set_buffer_at_or_err_str!(buffer, at, "' ut_name '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmp.ut_name);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmp.ut_host);
                // ut_time
                set_buffer_at_or_err_str!(buffer, at, "' ut_time ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmp.ut_time, netbsd_x8664::time_t);
                dt_end = at;
            }
            FixedStructType::Fs_Netbsd_x8664_Utmpx => {
                let utmpx: &netbsd_x8664::utmpx = entry.as_netbsd_x8664_utmpx();

                // ut_user
                set_buffer_at_or_err_str!(buffer, at, "ut_user '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_user);
                // ut_id
                set_buffer_at_or_err_str!(buffer, at, "' ut_id '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_id);
                // ut_line
                set_buffer_at_or_err_str!(buffer, at, "' ut_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_line);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmpx.ut_host);
                // ut_session
                set_buffer_at_or_err_str!(buffer, at, "' ut_session '");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_session, netbsd_x8664::uint16_t);
                // ut_type
                set_buffer_at_or_err_str!(buffer, at, "' ut_type ");
                set_buffer_at_or_err_ut_type_u16!(buffer, at, utmpx.ut_type);
                
                // ut_pid
                set_buffer_at_or_err_str!(buffer, at, " ut_pid ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_pid, i32);
                // ut_exit.e_termination
                set_buffer_at_or_err_str!(buffer, at, " e_termination ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit.e_termination, netbsd_x8664::uint16_t);
                // ut_exit.e_exit
                set_buffer_at_or_err_str!(buffer, at, " e_exit ");
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_exit.e_exit, netbsd_x8664::uint16_t);
                // ut_tv.tv_sec
                set_buffer_at_or_err_str!(buffer, at, " ut_tv ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_sec, i64);
                set_buffer_at_or_err_u8!(buffer, at, b'.');
                set_buffer_at_or_err_number!(buffer, at, utmpx.ut_tv.tv_usec, i32);
                dt_end = at;
            }
            FixedStructType::Fs_Openbsd_x86_Lastlog => {
                let lastlog: &openbsd_x86::lastlog = entry.as_openbsd_x86_lastlog();

                // ll_time
                set_buffer_at_or_err_str!(buffer, at, "ll_time ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, lastlog.ll_time, openbsd_x86::time_t);
                dt_end = at;
                // ll_line
                set_buffer_at_or_err_str!(buffer, at, " ll_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_line);
                // ll_host
                set_buffer_at_or_err_str!(buffer, at, "' ll_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, lastlog.ll_host);
                set_buffer_at_or_err_u8!(buffer, at, b'\'');
            }
            FixedStructType::Fs_Openbsd_x86_Utmp => {
                let utmp: &openbsd_x86::utmp = entry.as_openbsd_x86_utmp();

                // ut_line
                set_buffer_at_or_err_str!(buffer, at, "ut_line '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmp.ut_line);
                // ut_name
                set_buffer_at_or_err_str!(buffer, at, "' ut_name '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmp.ut_name);
                // ut_host
                set_buffer_at_or_err_str!(buffer, at, "' ut_host '");
                set_buffer_at_or_err_cstrn!(buffer, at, utmp.ut_host);
                // ut_time
                set_buffer_at_or_err_str!(buffer, at, "' ut_time ");
                dt_beg = at;
                set_buffer_at_or_err_number!(buffer, at, utmp.ut_time, openbsd_x86::time_t);
                dt_end = at;
            }
        }
        // line end
        set_buffer_at_or_err_u8!(buffer, at, b'\n');
        // string end
        set_buffer_at_or_err_u8!(buffer, at, b'\0');

        debug_assert_le!(dt_beg, dt_end);
        debug_assert_le!(dt_end, at);

        InfoAsBytes::Ok(at, dt_beg, dt_end)
    }

    /// Create `String` from known bytes.
    ///
    /// `raw` is `true` means use byte characters as-is.
    /// `raw` is `false` means replace formatting characters or non-printable
    /// characters with pictoral representation (i.e. use
    /// [`byte_to_char_noraw`]).
    ///
    /// XXX: very inefficient and not always correct! *only* intended to help
    ///      humans visually inspect stderr output.
    ///
    /// [`byte_to_char_noraw`]: crate::debug::printers::byte_to_char_noraw
    // XXX: this function is a lot of tedious work and duplicates
    //      `fmt::Debug` and `as_bytes()`; consider removing it
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    fn impl_to_String_raw(
        self: &FixedStruct,
        _raw: bool,
    ) -> String
    {
        let mut buf: String = String::with_capacity(100);
        buf.push_str("(incomplete function impl_to_String_raw)");

        buf
    }

    // TODO fix non_snake_case

    /// `FixedStruct` to `String`.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_raw(self: &FixedStruct) -> String
    {
        self.impl_to_String_raw(true)
    }

    /// `FixedStruct` to `String` but using printable chars for
    /// non-printable and/or formatting characters.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_noraw(self: &FixedStruct) -> String
    {
        self.impl_to_String_raw(false)
    }
}
