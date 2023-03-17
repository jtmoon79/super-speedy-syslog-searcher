// src/data/utmp.rs

//! Implement [`Utmpx`] for [`utmpx`] C structs (sometimes referred to
//! as the older [`utmp`] C struct).
//!
//! [`Utmpx`]: self::Utmpx
//! [`utmpx`]: https://github.com/freebsd/freebsd-src/blob/release/12.4.0/include/utmpx.h#L43-L56
//! [`utmp`]: https://elixir.bootlin.com/glibc/glibc-2.37/source/bits/utmp.h#L57

#[doc(hidden)]
pub use crate::common::{Bytes, CharSz, Count, FPath, FileOffset, NLu8, ResultS3};
use crate::data::datetime::{
    DateTimeL,
    DateTimeLOpt,
    FixedOffset,
    LocalResult,
    NaiveDateTime,
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
use std::mem::size_of;

use ::lazy_static::lazy_static;

/// target_os that support `uapi` crate.
#[doc(hidden)]
macro_rules! cfg_supports_uapi {
    { $($item:item)* } => {
        $(
            #[cfg(not(
                any(
                    target_os = "android",
                    target_os = "freebsd",
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "vxworks",
                )
            ))]
            $item
        )*
    }
}

/// target_os that do not support `uapi` crate.
///
/// Must match prior `cfg_supports_uapi!` macro.
#[doc(hidden)]
macro_rules! cfg_not_supports_uapi {
    { $($item:item)* } => {
        $(
            #[cfg(
                any(
                    target_os = "android",
                    target_os = "freebsd",
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "vxworks",
                )
            )]
            $item
        )*
    }
}

cfg_supports_uapi!{
    pub use ::uapi::c::{
        utmpx,
        __timeval,
        __exit_status,
    };
}
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

// If `target_os` is "windows" or other non-supporting OS then manually define
// a `utmpx` struct.
//
// The following code is copied and modified from crate `uapi` https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
// which is under the MIT license https://github.com/mahkoh/uapi/blob/86d032c60bb33c4aa888085f9b50bf6e19f7ba24/LICENSE-MIT
// and "Copyright (c) The uapi developers"

cfg_not_supports_uapi!{
    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct __timeval {
        pub tv_sec: i32,
        pub tv_usec: i32,
    }

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct __exit_status {
        pub e_termination: i16,
        pub e_exit: i16,
    }

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    #[repr(C)]
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
}

// end copied code from crate `uapi` under MIT license

/// [`size_of::<utmpx>`] (384).
///
/// [`size_of::<utmpx>`]: std::mem::size_of
pub const UTMPX_SZ: usize = size_of::<utmpx>();

/// [`UTMPX_SZ`] as a [`FileOffset`].
///
/// [`UTMPX_SZ`]: UTMPX_SZ
/// [`FileOffset`]: crate::common::FileOffset
pub const UTMPX_SZ_FO: FileOffset = UTMPX_SZ as FileOffset;

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
    static ref DEFAULT_DT: DateTimeL = {
        DateTimeL::from_utc(
            NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
            FixedOffset::east_opt(0).unwrap(),
        )
    };
}

/// A [`Utmpx`] holds a [C struct `utmpx`] and it's [`FileOffset`] and derived
/// [`DateTimeL`].
///
/// [`Utmpx`]: self::Utmpx
/// [C struct `utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
/// [`FileOffset`]: crate::common::FileOffset
/// [`DateTimeL`]: crate::data::datetime::DateTimeL
#[derive(Clone, Copy)]
pub struct Utmpx {
    /// The [`utmpx`] entry data.
    ///
    /// [`utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
    pub entry: utmpx,
    /// The byte offset into the file where this `Utmpx` begins.
    pub fileoffset: FileOffset,
    /// The derived DateTime instance using function
    /// [`convert_tvsec_utvcsec_datetime`].
    ///
    /// [`convert_tvsec_utvcsec_datetime`]: super::convert_tvsec_utvcsec_datetime
    dt: DateTimeL,
}

impl fmt::Debug for Utmpx {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("Utmpx")
            .field("fileoffset", &self.fileoffset)
            .field("dt", &self.dt)
            // TODO: complete this
            .finish()
    }
}

/// [`__timeval.tv_sec` second type] from `utmpx.h`
///
/// [`__timeval.tv_sec` second type]: https://docs.rs/uapi/0.2.10/uapi/c/struct.__timeval.html
#[allow(non_camel_case_types)]
pub type tv_sec_type = i32;

/// [`__timeval.tv_usec` microsecond type] from `utmpx.h`
///
/// [`__timeval.tv_usec` microsecond type]: https://docs.rs/uapi/0.2.10/uapi/c/struct.__timeval.html
#[allow(non_camel_case_types)]
pub type tv_usec_type = i32;

/// nanosecond type
#[allow(non_camel_case_types)]
pub type nsecs_type = u32;

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
pub fn convert_tvsec_utvcsec_datetime(
    tv_sec: tv_sec_type,
    tv_usec: tv_usec_type,
    tz_offset: &FixedOffset,
) -> DateTimeLOpt {
    // Firstly, convert microseconds to nanoseconds.
    // If conversion fails then use zero.
    let mut nsec: nsecs_type = match tv_usec.try_into() {
        Ok(val) => val,
        Err(_) => 0,
    };
    nsec = match nsec.checked_mul(1000) {
        Some(val) => val,
        None => 0,
    };
    // Secondly, convert seconds.
    // If conversion fails then return None.
    let tv_sec_i64: i64 = match tv_sec.try_into() {
        Ok(val) => val,
        Err(_) => {
            return None;
        }
    };

    defñ!("{:?}.timestamp({}, {})", tz_offset, tv_sec_i64, nsec);
    match tz_offset.timestamp_opt(
        tv_sec_i64, nsec
    ) {
        LocalResult::None => {
            // try again with zero nanoseconds
            match tz_offset.timestamp_opt(tv_sec_i64, 0) {
                LocalResult::None => None,
                LocalResult::Single(dt) => Some(dt),
                LocalResult::Ambiguous(dt, _) => Some(dt),
            }
        }
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(dt, _) => Some(dt),
    }
}

/// Convert `[u8]` bytes to a [`utmpx`] instance.
/// Returns `None` if the buffer is too small.
///
/// unsafe.
///
/// [`utmpx`]: https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
pub fn buffer_to_utmpx(buffer: &[u8]) -> Option<utmpx> {
    if buffer.len() < UTMPX_SZ {
        return None;
    }
    unsafe {
        Some(std::ptr::read(buffer.as_ptr() as *const _))
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

impl Utmpx {
    /// Create a new `Utmpx`.
    pub fn new(
        fileoffset: FileOffset,
        tz_offset: &FixedOffset,
        entry: utmpx,
    ) -> Utmpx {
        // See
        //  https://docs.rs/uapi/0.2.10/uapi/c/struct.utmpx.html
        //  https://docs.rs/uapi/0.2.10/uapi/c/struct.__timeval.html
        let tv_sec: tv_sec_type = entry.ut_tv.tv_sec;
        let tv_usec: tv_usec_type = entry.ut_tv.tv_usec;
        // XXX: still not sure if it's better to panic or somehow alert caller
        //      that the time conversion failed. A failed time conversion means
        //      the data is most likely invalid/wrong format.
        let dt = match convert_tvsec_utvcsec_datetime(tv_sec, tv_usec, tz_offset) {
            Some(dt) => dt,
            None => {
                defñ!("tv_sec {}, tv_usec {}", tv_sec, tv_usec);
                *DEFAULT_DT
            }
        };
        defñ!("tv_sec {}, tv_usec {}, dt {:?}", tv_sec, tv_usec, dt);
        Utmpx {
            fileoffset,
            entry,
            dt,
        }
    }

    /// Length of a `Utmpx` in bytes.
    pub const fn len(self: &Utmpx) -> usize {
        UTMPX_SZ
    }

    /// Clippy recommends `fn is_empty` since there is a `len()`.
    pub const fn is_empty(self: &Utmpx) -> bool {
        self.len() == 0
    }

    /// [`FileOffset`] at beginning of the `Utmpx` (inclusive).
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    pub const fn fileoffset_begin(self: &Utmpx) -> FileOffset {
        self.fileoffset
    }

    /// [`FileOffset`] at one byte past ending of the `Utmpx` (exclusive).
    ///
    /// [`FileOffset`]: crate::common::FileOffset
    pub const fn fileoffset_end(self: &Utmpx) -> FileOffset {
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
    pub const fn dt(self: &Utmpx) -> &DateTimeL {
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
    pub fn as_bytes(self: &Utmpx, buffer: &mut [u8])
        -> InfoAsBytes
    {
        let utmpx1: &utmpx = &self.entry;
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

        let ut_types: &str = match &utmpx1.ut_type {
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

        for b_ in utmpx1.ut_pid.to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_line

        for b_ in " ut_line '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_line.iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_id

        for b_ in "' ut_id '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_id.iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_user

        for b_ in "' ut_user '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_user.iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_host

        for b_ in "' ut_host '".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_host.iter() {
            if b_ == &0 {
                break;
            }
            set_i8_buffer_at_or_err!(buffer, at, *b_);
        }

        // ut_session

        for b_ in "' ut_session ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_session.to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_addr_v6

        for b in format!(" ut_addr_v6 {0:X}:{1:X}:{2:X}:{3:X} ",
            &utmpx1.ut_addr_v6[0],
            &utmpx1.ut_addr_v6[1],
            &utmpx1.ut_addr_v6[2],
            &utmpx1.ut_addr_v6[3],
        ).as_str().as_bytes().iter() {
            let b_ = *b;
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_exit.e_termination

        for b_ in "e_termination ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_exit.e_termination.to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        // ut_exit.e_exit

        for b_ in " e_exit ".bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        for b_ in utmpx1.ut_exit.e_exit.to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        if at >= buflen {
            return InfoAsBytes::Fail(at);
        }
        buffer[at] = b' ';
        at += 1;

        // ut_tv.tv_sec

        let dt_beg: BufIndex = at;

        for b_ in utmpx1.ut_tv.tv_sec.to_string().as_str().bytes() {
            set_u8_buffer_at_or_err!(buffer, at, b_);
        }

        if at >= buflen {
            return InfoAsBytes::Fail(at);
        }
        buffer[at] = b'.';
        at += 1;

        // ut_tv.tv_usec

        for b_ in utmpx1.ut_tv.tv_usec.to_string().as_str().bytes() {
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
    ) -> String {
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
        let utmpx1: &utmpx = &self.entry;

        const ULEN: i16 = UT_TYPE_VAL_TO_STR.len() as i16;
        let ut_types: &str = match &utmpx1.ut_type {
            n if &0 <= n && n < &ULEN => UT_TYPE_VAL_TO_STR[*n as usize],
            _ => "",
        };
        buf.push_str(format!("ut_type {} ({}) ", utmpx1.ut_type, ut_types).as_str());
        buf.push_str(format!("ut_pid {0:02} ", &utmpx1.ut_pid).as_str());

        let mut at: usize = 0;
        const MARK: char = '|';

        buf.push_str("ut_line[…] '");
        for ut_line in utmpx1.ut_line.iter() {
            let c: char = match raw {
                true => *ut_line as u8 as char,
                false => byte_to_char_noraw(*ut_line as u8),
            };
            //if c == '\0' {
            //    break;
            //}
            buf.push(c);
            at += 1;
            if ut_line == &0 {
                break;
            }
        }
        if at < utmpx1.ut_line.len() {
            let mut mark: bool = false;
            for ut_line in utmpx1.ut_line.iter().skip(at) {
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
        for ut_id in utmpx1.ut_id.iter() {
            let c: char = match raw {
                true => *ut_id as u8 as char,
                false => byte_to_char_noraw(*ut_id as u8),
            };
            //if c == '\0' {
            //    break;
            //}
            buf.push(c);
            at += 1;
            if ut_id == &0 {
                break;
            }
        }
        if at < utmpx1.ut_id.len() {
            let mut mark: bool = false;
            for ut_id in utmpx1.ut_id.iter().skip(at) {
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
        for ut_user in utmpx1.ut_user.iter() {
            let c: char = match raw {
                true => *ut_user as u8 as char,
                false => byte_to_char_noraw(*ut_user as u8),
            };
            buf.push(c);
            at += 1;
        }
        if at < utmpx1.ut_user.len() {
            let mut mark: bool = false;
            for ut_user in utmpx1.ut_user.iter().skip(at) {
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
        for ut_host in utmpx1.ut_host.iter() {
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
        if at < utmpx1.ut_host.len() {
            let mut mark: bool = false;
            for ut_host in utmpx1.ut_host.iter().skip(at) {
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

        buf.push_str(format!("ut_session {} ", &utmpx1.ut_session).as_str());
        buf.push_str(format!("addr_v6 {0:X}:{1:X}:{2:X}:{3:X} ",
            &utmpx1.ut_addr_v6[0],
            &utmpx1.ut_addr_v6[1],
            &utmpx1.ut_addr_v6[2],
            &utmpx1.ut_addr_v6[3],
        ).as_str());
        //for (i, ut_addr_v6) in utmpx1.ut_addr_v6.iter().enumerate() {
        //    println!("utmpx.ut_addr_v6[{1:2}]: {0:02} (0x{0:2x}) (0b{0:08b})", &ut_addr_v6, i);
        //}
        buf.push_str(format!("e_termination {} ", &utmpx1.ut_exit.e_termination).as_str());
        buf.push_str(format!("e_exit {} ", &utmpx1.ut_exit.e_exit).as_str());

        let _tv_sec = utmpx1.ut_tv.tv_sec;
        let _tv_usec = utmpx1.ut_tv.tv_usec;
        buf.push_str(format!("tv_sec {} ", &utmpx1.ut_tv.tv_sec).as_str());
        buf.push_str(format!("tv_usec {} (", &utmpx1.ut_tv.tv_usec).as_str());

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
    pub fn to_String_raw(self: &Utmpx) -> String {
        self.impl_to_String_raw(true)
    }

    /// `Utmpx` to `String` but using printable chars for
    /// non-printable and/or formatting characters.
    #[doc(hidden)]
    #[allow(non_snake_case)]
    #[cfg(any(debug_assertions, test))]
    pub fn to_String_noraw(self: &Utmpx) -> String {
        self.impl_to_String_raw(false)
    }
}
