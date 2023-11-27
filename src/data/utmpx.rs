// src/data/utmp.rs

//! Implement [`Utmpx`] for [`utmpx`] C structs (sometimes referred to
//! as the older [`utmp`] C struct).
//!
//! [`Utmpx`]: self::Utmpx
//! [`utmpx`]: https://github.com/freebsd/freebsd-src/blob/release/12.4.0/include/utmpx.h#L43-L56
//! [`utmp`]: https://elixir.bootlin.com/glibc/glibc-2.37/source/bits/utmp.h#L57

#[doc(hidden)]
use crate::{de_err, de_wrn};
#[doc(hidden)]
use crate::common::FileOffset;
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    LocalResult,
    TimeZone,
};
use crate::readers::blockreader::{
    BlockOffset,
    BlockReader,
    BlockSz,
};
#[cfg(any(debug_assertions, test))]
use crate::debug::printers::byte_to_char_noraw;

use std::fmt;
use std::io::{Error, ErrorKind};

use ::lazy_static::lazy_static;

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
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// The following code is copied and modified from crate `uapi` https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
// which is under the MIT license https://github.com/mahkoh/uapi/blob/86d032c60bb33c4aa888085f9b50bf6e19f7ba24/LICENSE-MIT
// and "Copyright (c) The uapi developers"

/// Copied and modified from https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
#[allow(non_camel_case_types)]
pub mod linux_gnu {
    use crate::common::FileOffset;
    use std::mem::size_of;

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct __timeval {
        pub tv_sec: i32,
        pub tv_usec: i32,
    }

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct __exit_status {
        pub e_termination: i16,
        pub e_exit: i16,
    }

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct utmpx {
        pub ut_type: i16,
        pub ut_pid: i32,
        pub ut_line: [i8; 32],
        pub ut_id: [i8; 4],
        pub ut_user: [i8; 32],
        pub ut_host: [i8; 256],
        pub ut_exit: __exit_status,
        pub ut_session: i32,
        pub ut_tv: __timeval,
        pub ut_addr_v6: [i32; 4],
        /* private fields */
        pub __glibc_reserved: [i8; 20],
    }

    /// [`size_of::<utmpx>`]; 384 typically, 640 on Mac OS.
    ///
    /// [`size_of::<utmpx>`]: std::mem::size_of
    pub const UTMPX_SZ: usize = size_of::<utmpx>();

    /// [`UTMPX_SZ`] as a [`FileOffset`].
    ///
    /// [`UTMPX_SZ`]: UTMPX_SZ
    /// [`FileOffset`]: crate::common::FileOffset
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
}

#[allow(non_camel_case_types, unused)]
mod linux_other {
    use crate::common::FileOffset;
    use std::mem::size_of;

    pub type c_char = i8;
    pub type wchar_t = i32;
    pub type nlink_t = u64;
    pub type blksize_t = i64;
    pub type greg_t = i64;
    pub type suseconds_t = i64;
    pub type __u64 = c_ulonglong;
    pub type __s64 = c_longlong;

    pub type c_uchar = u8;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_float = f32;
    pub type c_double = f64;
    pub type c_longlong = i64;
    pub type c_ulonglong = u64;
    pub type intmax_t = i64;
    pub type uintmax_t = u64;

    pub type size_t = usize;
    pub type ptrdiff_t = isize;
    pub type intptr_t = isize;
    pub type uintptr_t = usize;
    pub type ssize_t = isize;

    pub type pid_t = i32;
    pub type in_addr_t = u32;
    pub type in_port_t = u16;
    pub type sighandler_t = size_t;
    pub type cc_t = c_uchar;

    pub const __UT_LINESIZE: usize = 32;
    pub const __UT_NAMESIZE: usize = 32;
    pub const __UT_HOSTSIZE: usize = 256;

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct __timeval {
        pub tv_sec: i32,
        pub tv_usec: i32,
    }

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct __exit_status {
        pub e_termination: i16,
        pub e_exit: i16,
    }

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    #[allow(non_camel_case_types)]
    pub struct utmpx {
        pub ut_type: c_short,
        pub ut_pid: pid_t,
        pub ut_line: [c_char; __UT_LINESIZE],
        pub ut_id: [c_char; 4],

        pub ut_user: [c_char; __UT_NAMESIZE],
        pub ut_host: [c_char; __UT_HOSTSIZE],
        pub ut_exit: __exit_status,

        #[cfg(any(target_arch = "aarch64",
                  target_arch = "s390x",
                  target_arch = "loongarch64",
                  all(target_pointer_width = "32",
                      not(target_arch = "x86_64"))))]
        pub ut_session: ::c_long,
        #[cfg(any(target_arch = "aarch64",
                  target_arch = "s390x",
                  target_arch = "loongarch64",
                  all(target_pointer_width = "32",
                      not(target_arch = "x86_64"))))]
        pub ut_tv: ::timeval,

        #[cfg(not(any(target_arch = "aarch64",
                      target_arch = "s390x",
                      target_arch = "loongarch64",
                      all(target_pointer_width = "32",
                          not(target_arch = "x86_64")))))]
        pub ut_session: i32,
        #[cfg(not(any(target_arch = "aarch64",
                      target_arch = "s390x",
                      target_arch = "loongarch64",
                      all(target_pointer_width = "32",
                          not(target_arch = "x86_64")))))]
        pub ut_tv: __timeval,

        pub ut_addr_v6: [i32; 4],
        __glibc_reserved: [c_char; 20],
    }
    /// [`size_of::<utmpx>`]; 384 typically, 640 on Mac OS.
    ///
    /// [`size_of::<utmpx>`]: std::mem::size_of
    pub const UTMPX_SZ: usize = size_of::<utmpx>();

    /// [`UTMPX_SZ`] as a [`FileOffset`].
    ///
    /// [`UTMPX_SZ`]: UTMPX_SZ
    /// [`FileOffset`]: crate::common::FileOffset
    pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;
}

// end copied code from crate `uapi` under MIT license

#[allow(non_camel_case_types)]
pub type tv_sec_type = i64;
#[allow(non_camel_case_types)]
pub type tv_usec_type = i64;
#[allow(non_camel_case_types)]
pub type nsecs_type = u32;
/// `Box` pointer to a `dyn`amically dispatched _trait object_ `UtmpxT`.
pub type UtmpxDynPtr = Box<dyn UtmpxT>;

/// Maximum size of underlying `utmp`/`utmpx` C struct for any system.
// TODO: use `std::cmp::max` on all the known UTMPX_SZ.
//       However `std::cmp::max` not yet stable as `const` in rust MSRV 1.67.0
//       (cannot find tracking Issue for this)
//       Currently just manually setting to known largest UTMPX_SZ value.
pub const UTMPX_SZ_MAX: usize = linux_gnu::UTMPX_SZ;

/// Map [`.ut_type`] value, implied in the index offset, to a `str`
/// representation.
///
/// See [`man utmp`].
///
/// [`.ut_type`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html#structfield.ut_type
/// [`man utmp`]: https://man7.org/linux/man-pages/man5/utmp.5.html
pub const UT_TYPE_VAL_TO_STR: &[&str] = &[
    "EMPTY",
    "RUN_LVL",
    "BOOT_TIME",
    "NEW_TIME",
    "OLD_TIME",
    "INIT_PROCESS",
    "LOGIN_PROCESS",
    "USER_PROCESS",
    "DEAD_PROCESS",
    "ACCOUNTING",
];

lazy_static! {
    /// fallback `DateTimeL` for failed conversions
    static ref DEFAULT_DT: DateTimeL =
        FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
}

/// A *C*oncrete utmp/utmpx data; a real utmpx data structure must `impl` this.
///
/// Because this is used a `dyn` trait then it must be an "Object safe trait".
/// Being "Object safe trait" enforces limitations on behavior, e.g. cannot
/// require trait `Sized` which implies cannot require trait `Clone`, among
/// other limitations.
///
/// See <https://doc.rust-lang.org/reference/items/traits.html#object-safety>
pub trait UtmpxT
where Self: Send, // required for sending from file processing thread to main thread
      Self: std::marker::Sync, // required for `lazy_static!`
{
    fn entry_type(&self) -> UtmpxType;
    fn size(&self) -> usize;
    fn ut_type(&self) -> i16;
    fn ut_pid(&self) -> i32;
    fn ut_line(&self) -> [i8; 32];
    fn ut_id(&self) -> [i8; 4];
    fn ut_user(&self) -> [i8; 32];
    fn ut_host(&self) -> [i8; 256];
    fn ut_exit_e_termination(&self) -> i16;
    fn ut_exit_e_exit(&self) -> i16;
    fn ut_session(&self) -> i32;
    fn ut_addr_v6(&self) -> [i32; 4];
    fn ut_tv_tv_sec(&self) -> tv_sec_type;
    fn ut_tv_tv_usec(&self) -> tv_usec_type;
}

impl UtmpxT for linux_gnu::utmpx {
    fn entry_type(&self) -> UtmpxType {
        UtmpxType::LinuxGnu
    }
    fn size(&self) -> usize {
        linux_gnu::UTMPX_SZ
    }
    fn ut_type(&self) -> i16 {
        self.ut_type
    }
    fn ut_pid(&self) -> i32 {
        self.ut_pid
    }
    fn ut_line(&self) -> [i8; 32] {
        self.ut_line
    }
    fn ut_id(&self) -> [i8; 4] {
        self.ut_id
    }
    fn ut_user(&self) -> [i8; 32] {
        self.ut_user
    }
    fn ut_host(&self) -> [i8; 256] {
        self.ut_host
    }
    fn ut_exit_e_termination(&self) -> i16 {
        self.ut_exit.e_termination
    }
    fn ut_exit_e_exit(&self) -> i16 {
        self.ut_exit.e_exit
    }
    fn ut_session(&self) -> i32 {
        self.ut_session
    }
    fn ut_addr_v6(&self) -> [i32; 4] {
        self.ut_addr_v6
    }
    fn ut_tv_tv_sec(&self) -> tv_sec_type {
        self.ut_tv.tv_sec as tv_sec_type
    }
    fn ut_tv_tv_usec(&self) -> tv_usec_type {
        self.ut_tv.tv_usec as tv_usec_type
    }
}

impl fmt::Debug for dyn UtmpxT {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("UtmpxT")
            .field("size", &self.size())
            .field("ut_type", &self.ut_type())
            .field("ut_pid", &self.ut_pid())
            .field("ut_line", &self.ut_line())
            .field("ut_id", &self.ut_id())
            .field("ut_user", &self.ut_user())
            .field("ut_host", &self.ut_host())
            .field("ut_exit_e_termination", &self.ut_exit_e_termination())
            .field("ut_exit_e_exit", &self.ut_exit_e_exit())
            .field("ut_session", &self.ut_session())
            .field("ut_tv_tv_sec", &self.ut_tv_tv_sec())
            .field("ut_tv_tv_usec", &self.ut_tv_tv_usec())
            .finish()
    }
}

/// A [`Utmpx`] managed a [C struct `utmpx`] and it's [`FileOffset`] and derived
/// [`DateTimeL`].
///
/// [`utmpx`] is a user accounting record.
///
/// [`Utmpx`]: self::Utmpx
/// [C struct `utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
/// [`FileOffset`]: crate::common::FileOffset
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
/// [`utmpx`]: https://en.wikipedia.org/w/index.php?title=Utmp&oldid=1143684808#utmpx,_wtmpx_and_btmpx
pub struct Utmpx
{
    /// The [`utmpx`] entry data.
    ///
    /// [`utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
    pub entry: UtmpxDynPtr,
    /// The byte offset into the file where the `entry` data begins.
    pub fileoffset: FileOffset,
    /// The derived DateTime instance using function
    /// [`convert_tvsec_utvcsec_datetime`].
    ///
    /// [`convert_tvsec_utvcsec_datetime`]: convert_tvsec_utvcsec_datetime
    dt: DateTimeL,
}

pub fn clone_entry(entry: &UtmpxDynPtr) -> UtmpxDynPtr {
    match entry.entry_type() {
        UtmpxType::LinuxGnu => {
            Box::new(
                linux_gnu::utmpx {
                    ut_type: entry.ut_type(),
                    ut_pid: entry.ut_pid(),
                    ut_line: entry.ut_line(),
                    ut_id: entry.ut_id(),
                    ut_user: entry.ut_user(),
                    ut_host: entry.ut_host(),
                    ut_exit: linux_gnu::__exit_status {
                        e_termination: entry.ut_exit_e_termination(),
                        e_exit: entry.ut_exit_e_exit(),
                    },
                    ut_session: entry.ut_session(),
                    ut_tv: linux_gnu::__timeval {
                        // TODO: handle overflow during conversion
                        tv_sec: entry.ut_tv_tv_sec() as i32,
                        // TODO: handle overflow during conversion
                        tv_usec: entry.ut_tv_tv_usec() as i32,
                    },
                    ut_addr_v6: entry.ut_addr_v6(),
                    __glibc_reserved: [0; 20],
                }
            )
        }
    }
}

impl Clone for Utmpx {
    /// Manually implemented `clone`.
    ///
    /// Cannot `#[derive(Clone)]` because that requires
    /// trait `Sized` to be `impl` for all fields of `Utmpx`.
    /// But the `UtmpxT` trait is used as a `dyn` trait
    /// object; it is dynically sized (it is sized at runtime).
    /// So a `UtmpxT` cannot `impl`ement `Sized` thus `Utmpx` cannot
    /// `impl`ement `Sized` thus must manually implement `clone`.
    fn clone(self: &Utmpx) -> Self {
        defñ!("Utmpx.clone()");
        let entry = clone_entry(&self.entry);
        Utmpx {
            entry,
            fileoffset: self.fileoffset,
            dt: self.dt.clone(),
        }
    }
}

impl fmt::Debug for Utmpx
{
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Utmpx")
            .field("size", &self.entry.size())
            .field("type", &self.entry.entry_type())
            .field("tv_sec", &self.entry.ut_tv_tv_sec())
            .field("fileoffset", &self.fileoffset)
            .field("dt", &self.dt)
            .finish()
    }
}

/// Index into a `[u8]` buffer. Used by [`as_bytes`].
///
/// [`as_bytes`]: self::Utmpx::as_bytes
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
/// [`as_bytes`]: self::Utmpx::as_bytes
pub enum InfoAsBytes {
    Ok(usize, BufIndex, BufIndex),
    Fail(usize),
}

/// Convert `utmpx.__timeval` types [`tv_sec`] and [`tv_usec`] to a
/// [`DateTimeL`] instance.
///
/// Allow lossy microsecond conversion.
/// Return `None` if second conversion fails.
///
/// [`tv_sec`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.__timeval.html#structfield.tv_sec
/// [`tv_usec`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.__timeval.html#structfield.tv_usec
// TODO: return `Err` upon failure
pub fn convert_tvsec_utvcsec_datetime(
    tv_sec: tv_sec_type,
    tv_usec: tv_usec_type,
    tz_offset: &FixedOffset,
) -> DateTimeLOpt {
    // Firstly, convert i64 to i32.
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

    defñ!("{:?}.timestamp({}, {})", tz_offset, tv_sec, nsec);
    match tz_offset.timestamp_opt(
        tv_sec, nsec
    ) {
        LocalResult::None => {
            // try again with zero nanoseconds
            match tz_offset.timestamp_opt(tv_sec, 0) {
                LocalResult::None => None,
                LocalResult::Single(dt) => Some(dt),
                LocalResult::Ambiguous(dt, _) => Some(dt),
            }
        }
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(dt, _) => Some(dt),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UtmpxType {
    LinuxGnu,
}

/// Given `buffer` determine the associated `UtmpxType`.
///
/// This is where bytes data is algorimically matched to the appropriate
/// `UtmpxType` type and then converted to the appropriate `UtmpxT` instance.
/// It is presumed the caller has no idea about the `UtmpxType` and is merely
/// passing a big buffer of bytes.
pub fn determine_type_utmpx(buffer: &[u8]) -> Option<UtmpxType>
{
    defn!("(buffer len {:?})", buffer.len());
    // TODO: do a better analysis of the `buffer`
    if buffer.len() >= linux_gnu::UTMPX_SZ {
        defx!("return UtmpxType::LinuxGnu");
        return Some(UtmpxType::LinuxGnu);
    }

    defx!("return None");
    None
}

/// Convert `[u8]` bytes to a [`UtmpxDynPtr`] instance.
/// If `is_type` is `Some` then only attempt to match the buffer to the
/// specified `UtmpxType`.
/// Else `is_type` is `None` then attempt to match the buffer using
/// [`determine_type_utmpx`] to determine the `UtmpxType`.
/// Returns `None` if the buffer is too small or other matching problems.
///
/// unsafe.
pub fn buffer_to_utmpx(buffer: &[u8], is_type: Option<UtmpxType>) -> Option<UtmpxDynPtr>
{
    defn!("(buffer len {:?}, is_type {:?})", buffer.len(), is_type);
    let utmpx_type = match is_type {
        Some(val) => val,
        None => {
            match determine_type_utmpx(buffer) {
                Some(val) => val,
                None => {
                    de_err!("Could not determine UtmpxType for buffer of len {}", buffer.len());
                    defx!("return None");
                    return None;
                }
            }
        }
    };
    match utmpx_type {
        UtmpxType::LinuxGnu => {
            const SZ: usize = linux_gnu::UTMPX_SZ;
            if buffer.len() < SZ {
                de_err!("Known type UtmpxType::LinuxGnu buffer too small; {}, require {}",
                        buffer.len(), SZ);
                defx!("return None");
                return None;
            }

            let b: UtmpxDynPtr;
            let slice_ = &buffer[..SZ];
            unsafe {
                b = Box::new(
                    std::ptr::read_unaligned(slice_.as_ptr().cast::<linux_gnu::utmpx>())
                );
            }

            defx!("return linux_gnu::utmpx (size {})", SZ);
            Some(b)
        }
    }
}

/// Helper to `Utmpx::as_bytes`.
/// Write a byte to the buffer at the given index.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_u8_buffer_at_or_err {
    ($buffer:ident, $at:ident, $c:expr) => (
        if $at >= $buffer.len() {
            return InfoAsBytes::Fail($at);
        }
        $buffer[$at] = $c;
        $at += 1;
    )
}

/// Helper to `Utmpx::as_bytes`.
/// Write a byte to the buffer at the given index.
/// If the index is out of bounds, return `InfoAsBytes::Fail`.
macro_rules! set_i8_buffer_at_or_err {
    ($buffer:ident, $at:ident, $c:expr) => (
        let c_: u8 = match ($c).try_into() {
            Ok(val) => val,
            Err(_) => 0,
        };
        set_u8_buffer_at_or_err!($buffer, $at, c_);
    )
}

impl Utmpx
{
    /// Create a new `Utmpx`.
    pub fn new(
        fileoffset: FileOffset,
        tz_offset: &FixedOffset,
        entry_buffer: &[u8],
        is_type: Option<UtmpxType>,
    ) -> Option<Utmpx>
    {
        defn!();
        let entry: UtmpxDynPtr = match buffer_to_utmpx(entry_buffer, is_type) {
            Some(val) => val,
            None => {
                de_err!("buffer_to_utmpx failed");
                defx!();
                return None;
            }
        };
        defo!("entry {:?}", entry);
        // Timeval can 32 bit or 64 bit depending on the platform.
        // For example, in FreeBSD 12.1 it can be either.
        // See https://github.com/freebsd/freebsd-src/blob/cae85a647ad76d191ac61c3bff67b49aabd716ba/sys/sys/_timeval.h#L50
        // But use 64 bit here.
        // Relates to Issue #108
        let tv_sec: tv_sec_type = match entry.ut_tv_tv_sec().try_into() {
            Ok(val) => val,
            Err(_err) => {
                de_err!("entry.ut_tv_tv_sec() failed to convert {:?}; {}", entry.ut_tv_tv_sec(), _err);
                defx!();
                return None;
            },
        };
        let tv_usec: tv_usec_type = match entry.ut_tv_tv_usec().try_into() {
            Ok(val) => val,
            Err(_err) => {
                de_err!("entry.ut_tv_tv_usec() failed to convert {:?}; {}", entry.ut_tv_tv_usec(), _err);
                0
            },
        };
        let dt = match convert_tvsec_utvcsec_datetime(tv_sec, tv_usec, tz_offset) {
            Some(dt) => dt,
            None => {
                // `convert_tvsec_utvcsec_datetime` should have already printed an error
                defx!();
                return None;
            }
        };
        defx!("tv_sec {}, tv_usec {}, dt {:?}", tv_sec, tv_usec, dt);
        Some (
            Utmpx {
                entry,
                fileoffset,
                dt,
            }
        )
    }

    pub fn from_entry(
        fileoffset: FileOffset,
        tz_offset: &FixedOffset,
        entry: UtmpxDynPtr,
    ) -> Result<Utmpx, Error>
    {
        let dt = match convert_tvsec_utvcsec_datetime(
                entry.ut_tv_tv_sec(), entry.ut_tv_tv_usec(), tz_offset
            ) {
            Some(dt) => dt,
            None => {
                // `convert_tvsec_utvcsec_datetime` should have debug-printed an error
                defx!();
                // TODO: isn't there a Result type or Error defined within this project
                //       that distinguishes Errors with messages that have the file path
                //       name versus those without file path name?
                return Result::Err(
                    Error::new(
                        ErrorKind::Other,
                        format!(
                            "convert_tvsec_utvcsec_datetime({}, {}, {:?}) failed",
                            entry.ut_tv_tv_sec(), entry.ut_tv_tv_usec(), tz_offset,
                        )
                    )
                );
            }
        };

        Result::Ok(
            Utmpx {
                entry,
                fileoffset,
                dt,
            }
        )
    }

    pub fn len(self: &Utmpx) -> usize
    {
        self.entry.size()
    }

    /// Clippy recommends `fn is_empty` since there is a `len()`.
    pub fn is_empty(self: &Utmpx) -> bool
    {
        self.len() == 0
    }

    /// [`FileOffset`] at beginning of the `Utmpx` (inclusive).
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    pub const fn fileoffset_begin(self: &Utmpx) -> FileOffset
    {
        self.fileoffset
    }

    /// [`FileOffset`] at one byte past ending of the `Utmpx` (exclusive).
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    pub fn fileoffset_end(self: &Utmpx) -> FileOffset
    {
        self.fileoffset + (self.len() as FileOffset)
    }

    /// First [`BlockOffset`] of underlying [`Block`s] for the given
    /// [`BlockSz`].
    ///
    /// [`BlockOffset`]: crate::readers::blockreader::BlockOffset
    /// [`Block`s]: crate::readers::blockreader::Block
    /// [`BlockSz`]: crate::readers::blockreader::BlockSz
    pub fn blockoffset_begin(&self, blocksz: BlockSz) -> BlockOffset {
        BlockReader::block_offset_at_file_offset(
            self.fileoffset_begin(),
            blocksz
        )
    }

    /// Last [`BlockOffset`] of underlying [`Block`s] for the given
    /// [`BlockSz`].
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

    /// Return a reference to [`self.dt`] (`DateTimeL`).
    ///
    /// [`self.dt`]: Utmpx::dt
    pub const fn dt(self: &Utmpx) -> &DateTimeL
    {
        &self.dt
    }

    /// Copy the `Utmpx` into the passed `buffer` as printable bytes.
    /// When successful, returns a [`InfoAsBytes::Ok`] variant with
    /// number of bytes copied, start index of datetime substring, and
    /// end index of datetime substring.
    ///
    /// If copying fails, returns a [`InfoAsBytes::Fail`] variant with
    /// number of bytes copied.
    ///
    /// Efficient function to copy the `Utmpx` into a single re-usable
    /// buffer for printing.
    pub fn as_bytes(self: &Utmpx, buffer: &mut [u8]) -> InfoAsBytes
    {
        let utmpx1: &UtmpxDynPtr = &self.entry;
        const ULEN: i16 = UT_TYPE_VAL_TO_STR.len() as i16;
        let buflen: usize = buffer.len();
        let mut at: usize = 0;

        /*
            https://docs.rs/uapi/0.2.10/uapi/c/index.html
            https://linux.die.net/man/5/utmpx

            #define EMPTY         0 /* Record does not contain valid info (formerly known as UT_UNKNOWN on Linux) */
            #define RUN_LVL       1 /* Change in system run-level (see init(8)) */
            #define BOOT_TIME     2 /* Time of system boot (in ut_tv) */
            #define NEW_TIME      3 /* Time after system clock change (in ut_tv) */
            #define OLD_TIME      4 /* Time before system clock change (in ut_tv) */
            #define INIT_PROCESS  5 /* Process spawned by init(8) */
            #define LOGIN_PROCESS 6 /* Session leader process for user login */
            #define USER_PROCESS  7 /* Normal process */
            #define DEAD_PROCESS  8 /* Terminated process */
            #define ACCOUNTING    9 /* Not implemented */

            pub struct utmpx {
                pub ut_type: i16,
                pub ut_pid: i32,
                pub ut_line: [i8; 32],
                pub ut_id: [i8; 4],
                pub ut_user: [i8; 32],
                pub ut_host: [i8; 256],
                pub ut_exit: __exit_status,
                pub ut_session: i32,
                pub ut_tv: __timeval,
                pub ut_addr_v6: [i32; 4],
                /* private fields */
            }

            pub struct __exit_status {
                pub e_termination: i16,
                pub e_exit: i16,
            }

            pub struct __timeval {
                pub tv_sec: i32,
                pub tv_usec: i32,
            }

        */

        // ut_type

        let ut_types: &str = match &utmpx1.ut_type() {
            n if &0 <= n && n < &ULEN => UT_TYPE_VAL_TO_STR[*n as usize],
            _ => "",
        };
        for b_ in ut_types.bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_pid

        for b_ in " ut_pid ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_pid().to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_line

        for b_ in " ut_line '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_line().iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_id

        for b_ in "' ut_id '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_id().iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_user

        for b_ in "' ut_user '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_user().iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_host

        for b_ in "' ut_host '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_host().iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        for b_ in "' ut_session ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_session().to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_addr_v6

        for b in format!(" ut_addr_v6 {0:X}:{1:X}:{2:X}:{3:X} ",
            &utmpx1.ut_addr_v6()[0],
            &utmpx1.ut_addr_v6()[1],
            &utmpx1.ut_addr_v6()[2],
            &utmpx1.ut_addr_v6()[3],
        ).as_str().as_bytes().iter() {
            let b_ = *b;
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_exit.e_termination

        for b_ in "e_termination ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_exit_e_termination().to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_exit.e_exit

        for b_ in " e_exit ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_exit_e_exit().to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        if at >= buflen {
            return InfoAsBytes::Fail(at);
        }
        buffer[at] = b' ';
        at += 1;

        // ut_tv.tv_sec

        let dt_beg: BufIndex = at;

        for b_ in utmpx1.ut_tv_tv_sec().to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        if at >= buflen {
            return InfoAsBytes::Fail(at);
        }
        buffer[at] = b'.';
        at += 1;

        // ut_tv.tv_usec

        for b_ in utmpx1.ut_tv_tv_usec().to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        if at >= buflen {
            return InfoAsBytes::Fail(at);
        }
        let dt_end: BufIndex = at;

        buffer[at] = b'\n';

        at += 1;
        if at >= buflen {
            return InfoAsBytes::Fail(at);
        }
        buffer[at] = b'\0';

        debug_assert_le!(dt_beg, dt_end);
        debug_assert_le!(dt_end, at);

        InfoAsBytes::Ok(at, dt_beg, dt_end)
    }

    /// Create `String` from known bytes referenced by `self.lineparts`.
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
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    fn impl_to_String_raw(
        self: &Utmpx,
        raw: bool,
    ) -> String
    {
        /*
            https://docs.rs/uapi/0.2.10/uapi/c/index.html
            https://linux.die.net/man/5/utmpx

            #define EMPTY         0 /* Record does not contain valid info (formerly known as UT_UNKNOWN on Linux) */
            #define RUN_LVL       1 /* Change in system run-level (see init(8)) */
            #define BOOT_TIME     2 /* Time of system boot (in ut_tv) */
            #define NEW_TIME      3 /* Time after system clock change (in ut_tv) */
            #define OLD_TIME      4 /* Time before system clock change (in ut_tv) */
            #define INIT_PROCESS  5 /* Process spawned by init(8) */
            #define LOGIN_PROCESS 6 /* Session leader process for user login */
            #define USER_PROCESS  7 /* Normal process */
            #define DEAD_PROCESS  8 /* Terminated process */
            #define ACCOUNTING    9 /* Not implemented */

            pub struct utmpx {
                pub ut_type: i16,
                pub ut_pid: i32,
                pub ut_line: [i8; 32],
                pub ut_id: [i8; 4],
                pub ut_user: [i8; 32],
                pub ut_host: [i8; 256],
                pub ut_exit: __exit_status,
                pub ut_session: i32,
                pub ut_tv: __timeval,
                pub ut_addr_v6: [i32; 4],
                /* private fields */
            }

            pub struct __exit_status {
                pub e_termination: i16,
                pub e_exit: i16,
            }

            pub struct __timeval {
                pub tv_sec: i32,
                pub tv_usec: i32,
            }

        */
        let mut buf: String = String::with_capacity(100);
        let utmpx1: &UtmpxDynPtr = &self.entry;

        const ULEN: i16 = UT_TYPE_VAL_TO_STR.len() as i16;
        let ut_types: &str = match &utmpx1.ut_type() {
            n if &0 <= n && n < &ULEN => UT_TYPE_VAL_TO_STR[*n as usize],
            _ => "",
        };
        buf.push_str(format!("ut_type {} ({}) ", utmpx1.ut_type(), ut_types).as_str());
        buf.push_str(format!("ut_pid {0:02} ", &utmpx1.ut_pid()).as_str());

        let mut at: usize = 0;
        const MARK: char = '|';

        buf.push_str("ut_line[…] '");
        for ut_line in utmpx1.ut_line().iter() {
            let c: char = match raw {
                true => *ut_line as u8 as char,
                false => byte_to_char_noraw(*ut_line as u8),
            };
            buf.push(c);
            at += 1;
            if ut_line == &0 {
                break;
            }
        }
        if at < utmpx1.ut_line().len() {
            let mut mark: bool = false;
            for ut_line in utmpx1.ut_line().iter().skip(at) {
                if ut_line == &0 {
                    continue;
                }
                if ! mark {
                    buf.push(MARK);
                    mark = true;
                }
                let c: char = match raw {
                    true => *ut_line as u8 as char,
                    false => byte_to_char_noraw(*ut_line as u8),
                };
                buf.push(c);
                at += 1;
            }
        }
        buf.push_str("' ");

        buf.push_str(" ut_id[…] '");
        at = 0;
        for ut_id in utmpx1.ut_id().iter() {
            let c: char = match raw {
                true => *ut_id as u8 as char,
                false => byte_to_char_noraw(*ut_id as u8),
            };
            buf.push(c);
            at += 1;
            if ut_id == &0 {
                break;
            }
        }
        if at < utmpx1.ut_id().len() {
            let mut mark: bool = false;
            for ut_id in utmpx1.ut_id().iter().skip(at) {
                if ut_id == &0 {
                    continue;
                }
                if ! mark {
                    buf.push(MARK);
                    mark = true;
                }
                let c: char = match raw {
                    true => *ut_id as u8 as char,
                    false => byte_to_char_noraw(*ut_id as u8),
                };
                buf.push(c);
                at += 1;
            }
        }
        buf.push_str("' ");

        buf.push_str("ut_user[…] '");
        at = 0;
        for ut_user in utmpx1.ut_user().iter() {
            let c: char = match raw {
                true => *ut_user as u8 as char,
                false => byte_to_char_noraw(*ut_user as u8),
            };
            buf.push(c);
            at += 1;
        }
        if at < utmpx1.ut_user().len() {
            let mut mark: bool = false;
            for ut_user in utmpx1.ut_user().iter().skip(at) {
                if ut_user == &0 {
                    continue;
                }
                if ! mark {
                    buf.push(MARK);
                    mark = true;
                }
                let c: char = match raw {
                    true => *ut_user as u8 as char,
                    false => byte_to_char_noraw(*ut_user as u8),
                };
                buf.push(c);
                at += 1;
            }
        }
        buf.push_str("' ");

        buf.push_str("ut_host[…] '");
        at = 0;
        for ut_host in utmpx1.ut_host().iter() {
            let c: char = match raw {
                true => *ut_host as u8 as char,
                false => byte_to_char_noraw(*ut_host as u8),
            };
            buf.push(c);
            at += 1;
            if ut_host == &0 {
                break;
            }
        }
        if at < utmpx1.ut_host().len() {
            let mut mark: bool = false;
            for ut_host in utmpx1.ut_host().iter().skip(at) {
                if ut_host == &0 {
                    continue;
                }
                if ! mark {
                    buf.push(MARK);
                    mark = true;
                }
                let c: char = match raw {
                    true => *ut_host as u8 as char,
                    false => byte_to_char_noraw(*ut_host as u8),
                };
                buf.push(c);
                at += 1;
            }
        }
        buf.push_str("' ");

        buf.push_str(format!("ut_session {} ", &utmpx1.ut_session()).as_str());
        buf.push_str(format!("addr_v6 {0:X}:{1:X}:{2:X}:{3:X} ",
            &utmpx1.ut_addr_v6()[0],
            &utmpx1.ut_addr_v6()[1],
            &utmpx1.ut_addr_v6()[2],
            &utmpx1.ut_addr_v6()[3],
        ).as_str());
        buf.push_str(format!("e_termination {} ", &utmpx1.ut_exit_e_termination()).as_str());
        buf.push_str(format!("e_exit {} ", &utmpx1.ut_exit_e_exit()).as_str());

        let _tv_sec = utmpx1.ut_tv_tv_sec();
        let _tv_usec = utmpx1.ut_tv_tv_usec();
        buf.push_str(format!("tv_sec {} ", &utmpx1.ut_tv_tv_sec()).as_str());
        buf.push_str(format!("tv_usec {} (", &utmpx1.ut_tv_tv_usec()).as_str());

        let dts: String = self.dt.to_rfc3339();
        buf.push_str(dts.as_str());
        buf.push(')');

        buf
    }

    // TODO fix non_snake_case

    /// `Utmpx` to `String`.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_raw(self: &Utmpx) -> String
    {
        self.impl_to_String_raw(true)
    }

    /// `Utmpx` to `String` but using printable chars for
    /// non-printable and/or formatting characters.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_noraw(self: &Utmpx) -> String
    {
        self.impl_to_String_raw(false)
    }
}
