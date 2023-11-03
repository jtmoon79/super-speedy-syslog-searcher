// src/libload/systemd_dlopen2.rs

//! Functions to dynamically load the `libsystemd` library using [`dlopen2`].
//!
//! [`dlopen2`]: https://docs.rs/dlopen2/0.4.1/dlopen2/index.html

use crate::bindings::sd_journal_h::{
    sd_id128_t,
    size_t,
    sd_journal,
};
use std::fmt;
use std::sync::{RwLock, Arc};

#[cfg(not(windows))]
use ::const_format::concatcp;
use ::dlopen2::wrapper::{Container, WrapperApi};
use ::lazy_static::lazy_static;
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// User-friendly name for the `libsystemd` library, used in error messages.
#[cfg(not(windows))]
pub const LIB_NAME_SYSTEMD: &str = "libsystemd.so";
// XXX: not sure that `libsystemd.dll` is even a thing (libsystemd on Windows!? maybe on cygwin?)
//      but we can try.
/// User-friendly name for the `libsystemd` library, used in error messages.
#[cfg(windows)]
pub const LIB_NAME_SYSTEMD: &str = "libsystemd.dll";
#[cfg(windows)]
const LIB_NAME_SYSTEMD_NAMES_LEN: usize = 1;
#[cfg(not(windows))]
const LIB_NAME_SYSTEMD_NAMES_LEN: usize = 7;
/// All possible names for the `libsystemd` library
/// used in [`load_library_systemd`].
///
/// In my search I found `libsystemd` library file paths:
///
/// - Alpine Linux 3.17 does not have a libsystemd package
/// - CentOS 7
/// ```text
/// /usr/lib64/libsystemd-daemon.so.0
/// /usr/lib64/libsystemd-daemon.so.0.0.12
/// /usr/lib64/libsystemd-id128.so.0
/// /usr/lib64/libsystemd-id128.so.0.0.28
/// /usr/lib64/libsystemd-journal.so.0
/// /usr/lib64/libsystemd-journal.so.0.11.5
/// /usr/lib64/libsystemd-login.so.0
/// /usr/lib64/libsystemd-login.so.0.9.3
/// /usr/lib64/libsystemd.so.0
/// /usr/lib64/libsystemd.so.0.6.0
/// ```
/// - CentOS 9
/// ```text
/// /usr/lib64/libsystemd.so.0
/// /usr/lib64/libsystemd.so.0.35.0
/// /usr/lib64/systemd/libsystemd-core-252.so
/// /usr/lib64/systemd/libsystemd-shared-252.so
/// ```
/// - Red Hat Enterprise Linux 9.1
/// ```text
/// /usr/lib64/libsystemd.so.0
/// /usr/lib64/libsystemd.so.0.33.0
/// /usr/lib/systemd/libsystemd-shared-250.so
/// /usr/lib/systemd/libsystemd-shared.abignore
/// ```
/// - OpenSUSE Tumbleweed
/// ```text
/// /usr/lib64/libsystemd.so.0
/// /usr/lib64/libsystemd.so.0.36.0
/// /usr/lib64/systemd/libsystemd-core-253.so
/// /usr/lib64/systemd/libsystemd-shared-253.so
/// ```
/// - Ubuntu 20.04
/// ```text
/// /usr/lib/systemd/libsystemd-shared-245.so
/// /usr/lib/x86_64-linux-gnu/libsystemd.so
/// /usr/lib/x86_64-linux-gnu/libsystemd.so.0
/// /usr/lib/x86_64-linux-gnu/libsystemd.so.0.28.0
/// ```
/// - Ubuntu 22.04
/// ```text
/// /usr/lib/x86_64-linux-gnu/libsystemd.so
/// /usr/lib/x86_64-linux-gnu/libsystemd.so.0
/// /usr/lib/x86_64-linux-gnu/libsystemd.so.0.32.0
/// ```
/// - FreeBSD and OpenBSD do not run systemd.
///
/// Using command:
/// ```text
/// (find / -xdev \( -type f -o -type l \) -name 'libsystemd*' 2>/dev/null || true) | sort
/// ```
pub const LIB_NAME_SYSTEMD_NAMES: [&str; LIB_NAME_SYSTEMD_NAMES_LEN] = [
    LIB_NAME_SYSTEMD,
    // on some Linux systems, there is only `libsystemd.so.0` (no `libsystemd.so` symlink)
    #[cfg(not(windows))]
    concatcp!(LIB_NAME_SYSTEMD, ".0"),
    #[cfg(not(windows))]
    concatcp!(LIB_NAME_SYSTEMD, ".0.28.0"),
    #[cfg(not(windows))]
    concatcp!(LIB_NAME_SYSTEMD, ".0.32.0"),
    #[cfg(not(windows))]
    concatcp!(LIB_NAME_SYSTEMD, ".0.36.0"),
    // on older Linux systems there might be `libsystemd-journal.so`
    #[cfg(not(windows))]
    "/usr/lib64/libsystemd-journal.so",
    #[cfg(not(windows))]
    "/usr/lib64/libsystemd-journal.so.0",
];

/// [`dlopen2`] API wrapper for `libsystemd.so`. Selected functions from
/// [`systemd/sd-journal.h`].
///
/// [`dlopen2`]: https://docs.rs/dlopen2/0.4.1/dlopen2/index.html
/// [`systemd/sd-journal.h`]: https://github.com/systemd/systemd/blob/v249/src/systemd/sd-journal.h
#[derive(WrapperApi)]
pub struct SdJournalHApi {
    /// Function signature for [`sd_journal_close`].
    ///
    /// [`sd_journal_close`]: https://www.man7.org/linux/man-pages/man3/sd-journal.3.html
    sd_journal_close: unsafe extern fn(
        j: *mut sd_journal,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_open_files`].
    ///
    /// [`sd_journal_open_files`]: https://man7.org/linux/man-pages/man3/sd_journal_open.3.html
    sd_journal_open_files: unsafe extern fn(
        j: *mut *mut sd_journal,
        paths: *mut *const ::std::os::raw::c_char,
        flags: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_seek_head`].
    ///
    /// [`sd_journal_seek_head`]: https://man7.org/linux/man-pages/man3/sd_journal_seek_head.3.html
    sd_journal_seek_head: unsafe extern fn(
        j: *mut sd_journal,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_next`].
    ///
    /// [`sd_journal_next`]: https://man7.org/linux/man-pages/man3/sd_journal_next.3.html
    sd_journal_next: unsafe extern fn(
        j: *mut sd_journal,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_seek_realtime_usec`].
    ///
    /// [`sd_journal_seek_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_seek_head.3.html
    sd_journal_seek_realtime_usec: unsafe extern fn(
        j: *mut sd_journal,
        usec: u64,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_get_realtime_usec`].
    ///
    /// [`sd_journal_get_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_get_realtime_usec.3.html
    sd_journal_get_realtime_usec: unsafe extern fn(
        j: *mut sd_journal,
        usec: *mut u64,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_get_monotonic_usec`].
    ///
    /// [`sd_journal_get_monotonic_usec`]: https://man7.org/linux/man-pages/man3/sd_journal_get_realtime_usec.3.html
    sd_journal_get_monotonic_usec: unsafe extern fn(
        j: *mut sd_journal,
        ret: *mut u64,
        ret_boot_id: *mut sd_id128_t,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_id128_get_boot`].
    ///
    /// [`sd_id128_get_boot`]: https://www.man7.org/linux/man-pages/man3/sd_id128_get_boot.3.html
    sd_id128_get_boot: unsafe extern fn(
        j: *mut sd_journal,
        ret: *mut u64,
        ret_boot_id: *mut sd_id128_t,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_get_cursor`].
    ///
    /// [`sd_journal_get_cursor`]: https://www.man7.org/linux/man-pages/man3/sd_journal_get_cursor.3.html
    sd_journal_get_cursor: unsafe extern fn(
        j: *mut sd_journal,
        cursor: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_restart_data`].
    ///
    /// [`sd_journal_restart_data`]: https://man7.org/linux/man-pages/man3/sd_journal_get_data.3.html
    sd_journal_restart_data: unsafe extern fn(
        j: *mut sd_journal,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_get_data`].
    ///
    /// [`sd_journal_get_data`]: https://www.man7.org/linux/man-pages/man3/sd_journal_get_data.3.html
    sd_journal_get_data: unsafe extern fn(
        j: *mut sd_journal,
        field: *const ::std::os::raw::c_char,
        data: *mut *const ::std::os::raw::c_void,
        l: *mut size_t,
    ) -> ::std::os::raw::c_int,

    /// Function signature for [`sd_journal_enumerate_available_data`].
    ///
    /// [`sd_journal_enumerate_available_data`]: https://man7.org/linux/man-pages/man3/sd_journal_get_data.3.html
    sd_journal_enumerate_available_data: unsafe extern fn(
        j: *mut sd_journal,
        data: *mut *const ::std::os::raw::c_void,
        l: *mut size_t,
    ) -> ::std::os::raw::c_int,
}

/// `dlopen2` container for the `libsystemd` interface.
pub type JournalApiContainer = Container::<SdJournalHApi>;

/// Multi-threaded pointer to a the `libsystemd` interface.
pub type JournalApiPtr = Arc<JournalApiContainer>;

lazy_static! {
    /// The interface for using shared library `libsystemd.so`.
    ///
    /// The `RwLock` is to allow setting the value in `load_library_systemd()`.
    /// It's an oddity of setting a `lazy_static` variable once and then again
    /// later on.
    /// The `Option` is to allow setting an initial value of `None` because
    /// there may be no need to load `libsystemd` (or loading `libsystemd`
    /// may fail so the program should then just skip `.journal` files).
    /// The `Arc` is to allow returning a reference to the `dlopen2::Container`
    /// that can be shared among threads.
    /// The `dlopen2::Container` is the how `dlopen2` wraps a shared library
    /// reference handle for the given API, `SdJournalHApi`.
    pub static ref SYSTEMD_JOURNAL_API: RwLock<Option<JournalApiPtr>> = {
        RwLock::new(None)
    };

    /// None means `load_library_systemd()` has not been called yet.
    /// Some(false) means `load_library_systemd()` was called but failed.
    /// Some(true) means `load_library_systemd()` was called and succeeded.
    pub static ref LOAD_LIBRARY_SYSTEMD_OK: RwLock<Option<bool>> = {
        RwLock::new(None)
    };
}

/// Helpful accessor for lazy_static [`SYSTEMD_JOURNAL_API`].
///
/// [`SYSTEMD_JOURNAL_API`]: static@SYSTEMD_JOURNAL_API
pub fn journal_api() -> JournalApiPtr {
    #[cfg(debug_assertions)]
    {
        if SYSTEMD_JOURNAL_API.read().is_err() {
            panic!("failed to read SYSTEMD_JOURNAL_API failed; did you call load_library_systemd() previously?");
        }
        if SYSTEMD_JOURNAL_API.read().unwrap().as_ref().is_none() {
            panic!("SYSTEMD_JOURNAL_API holds None; did you call load_library_systemd() previously?");
        }
    }
    SYSTEMD_JOURNAL_API.read().unwrap().as_ref().unwrap().clone()
}

/// Return values for [`load_library_systemd`].
pub enum LoadLibraryError {
    /// The library was successfully loaded.
    Ok,
    /// The library failed to load and this was the error.
    Err(::dlopen2::Error),
    /// A previous attempt to load the library failed (the previous attempt
    /// returned `Err`). No more attempts will be made to load the library.
    PrevErr,
}

impl PartialEq for LoadLibraryError {
    /// allow `Err` == `PrevErr`
    fn eq(&self, other: &LoadLibraryError) -> bool {
        match (self, other) {
            (&LoadLibraryError::Ok, &LoadLibraryError::Ok) |
            (&LoadLibraryError::Err(_), &LoadLibraryError::Err(_)) |
            (&LoadLibraryError::Err(_), &LoadLibraryError::PrevErr) |
            (&LoadLibraryError::PrevErr, &LoadLibraryError::Err(_)) |
            (&LoadLibraryError::PrevErr, &LoadLibraryError::PrevErr) => true,
            _ => false,
        }
    }
}
impl Eq for LoadLibraryError {}

impl fmt::Debug for LoadLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadLibraryError::Ok => {
                f.debug_struct("LoadLibraryError::Ok")
                .finish()
            }
            LoadLibraryError::Err(_) => {
                f.debug_struct("LoadLibraryError::Err")
                .finish()
            }
            LoadLibraryError::PrevErr => {
                f.debug_struct("LoadLibraryError::PrevErr")
                .finish()
            }
        }
    }
}

/// Wrapper to set the global static variables.
fn set_systemd_journal_api(container: JournalApiContainer) {
    defñ!();
    *SYSTEMD_JOURNAL_API.write().unwrap() = Some(Arc::new(container));
    *LOAD_LIBRARY_SYSTEMD_OK.write().unwrap() = Some(true);
}

/// Load the shared library `libsystemd`. Store in the global static
/// variable `SYSTEMD_JOURNAL_API` the [`dlopen2::Container`] object.
///
/// Only attempts to load the library once.
///
/// If the load library attempt fails the first time then that call will
/// return `LoadLibraryError::Err`. All subsequent calls to
/// `load_library_systemd` will return `LoadLibraryError::PrevErr`.
///
/// If the load library succeeds in the current call or in a previous call
/// then return `LoadLibraryError::Ok`.
///
/// [`dlopen2::Container`]: https://docs.rs/dlopen2/0.4.1/dlopen2/wrapper/struct.Container.html
pub fn load_library_systemd() -> LoadLibraryError {
    // only attempt to load the library once. if that fails don't try again.
    match *LOAD_LIBRARY_SYSTEMD_OK.read().unwrap() {
        Some(true) => return LoadLibraryError::Ok,
        Some(false) => return LoadLibraryError::PrevErr,
        None => {}
    }

    defn!();

    // load the library!
    for (index, libname) in LIB_NAME_SYSTEMD_NAMES.iter().enumerate() {
        defo!("Container::load({:?})", libname);
        match unsafe { JournalApiContainer::load(libname) }
        {
            Ok(container) => {
                defx!("loaded library {:?}", libname);
                set_systemd_journal_api(container);
                return LoadLibraryError::Ok;
            }
            Err(err) => {
                defo!("failed to load library: {}", err);
                if index == LIB_NAME_SYSTEMD_NAMES.len() - 1 {
                    *LOAD_LIBRARY_SYSTEMD_OK.write().unwrap() = Some(false);
                    defx!("return Err({:?})", err);
                    return LoadLibraryError::Err(err);
                }
            }
        }
    }
    // XXX: should never get here

    *LOAD_LIBRARY_SYSTEMD_OK.write().unwrap() = Some(false);

    LoadLibraryError::PrevErr
}
