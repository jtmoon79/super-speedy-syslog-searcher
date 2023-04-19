// src/libload/systemd_libloading.rs

//! Functions to dynamically load the `libsystemd` library using [`libloading`].
//!
//! [`libloading`]: https://docs.rs/libloading/0.7.4/libloading/index.html

use std::sync::RwLock;

use ::libloading::{Library, Symbol};
use ::lazy_static::lazy_static;
use ::si_trace_print::{
    defo,
    defx,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const LIB_NAME: &str = "libsystemd.so";

use crate::bindings::sd_journal_h::{
    size_t,
    sd_journal,
};

/// Function signature for [`sd_journal_open_files`].
///
/// ```c
/// int sd_journal_open_files(
///     sd_journal *j,
///     const char **paths,
///     int flags
/// );
/// ```
///
/// ```rust,ignore
/// pub fn sd_journal_open_files(
///     ret: *mut *mut sd_journal,
///     paths: *mut *const ::std::os::raw::c_char,
///     flags: ::std::os::raw::c_int,
/// ) -> ::std::os::raw::c_int;
/// ```
///
/// [`sd_journal_open_files`]: https://man7.org/linux/man-pages/man3/sd_journal_open.3.html
#[allow(non_camel_case_types)]
pub type func_sig_sd_journal_open_files = unsafe extern fn(
    ret: *mut *mut sd_journal,
    paths: *mut *const ::std::os::raw::c_char,
    flags: ::std::os::raw::c_int,
) -> ::std::os::raw::c_int;

const FUNC_NAME_JOURNAL_OPEN_FILES: &str = "sd_journal_open_files";

/// Function signature for [`sd_journal_seek_head`].
///
/// ```c
/// int sd_journal_seek_head(sd_journal *j);
/// ```
///
/// ```rust,ignore
/// pub fn sd_journal_seek_head(
///    j: *mut sd_journal
/// ) -> ::std::os::raw::c_int;
/// ```
///
/// [`sd_journal_seek_head`]: https://man7.org/linux/man-pages/man3/sd_journal_seek_head.3.html
#[allow(non_camel_case_types)]
pub type func_sig_sd_journal_seek_head = unsafe extern fn(
    ret: *mut sd_journal,
) -> ::std::os::raw::c_int;

const FUNC_NAME_JOURNAL_SEEK_HEAD: &str = "sd_journal_seek_head";

/// Function signature for [`sd_journal_seek_realtime_usec`].
///
/// ```c
/// int sd_journal_seek_realtime_usec(sd_journal *j, uint64_t usec);
/// ```
///
/// ```rust,ignore
/// pub fn sd_journal_seek_realtime_usec(
///     j: *mut sd_journal,
///     usec: u64
/// ) -> ::std::os::raw::c_int;
/// ```
///
/// [`sd_journal_seek_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_seek_head.3.html
#[allow(non_camel_case_types)]
pub type func_sig_sd_journal_seek_journal_realtime_usec = unsafe extern fn(
    ret: *mut sd_journal,
    usec: u64,
) -> ::std::os::raw::c_int;

const FUNC_NAME_JOURNAL_SEEK_REALTIME_USEC: &str = "sd_journal_seek_realtime_usec";

/// Function signature for [`sd_journal_next`].
///
/// ```c
/// int sd_journal_next(sd_journal *j);
/// ```
///
/// ```rust,ignore
/// pub fn sd_journal_next(
///    j: *mut sd_journal
/// ) -> ::std::os::raw::c_int;
/// ```
///
/// [`sd_journal_next`]: https://man7.org/linux/man-pages/man3/sd_journal_next.3.html
#[allow(non_camel_case_types)]
pub type func_sig_sd_journal_next = unsafe extern fn(
    ret: *mut sd_journal,
) -> ::std::os::raw::c_int;

const FUNC_NAME_JOURNAL_NEXT: &str = "sd_journal_next";

/// Function signature for [`sd_journal_get_realtime_usec`].
///
/// ```c
/// int sd_journal_get_realtime_usec(sd_journal *j, uint64_t *usec);
/// ```
///
/// ```rust,ignore
/// pub fn sd_journal_get_realtime_usec(
///     j: *mut sd_journal,
///     ret: *mut u64
/// ) -> ::std::os::raw::c_int;
/// ```
///
/// [`sd_journal_get_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_get_realtime_usec.3.html
#[allow(non_camel_case_types)]
pub type func_sig_sd_journal_get_realtime_usec = unsafe extern fn(
    ret: *mut sd_journal,
    usec: *mut u64,
) -> ::std::os::raw::c_int;

const FUNC_NAME_JOURNAL_GET_REALTIME_USEC: &str = "sd_journal_get_realtime_usec";

/// Function signature for [`sd_journal_enumerate_available_data`].
///
/// ```c
/// int sd_journal_enumerate_available_data(
///     sd_journal *j,
///     const void **data,
///     size_t *length
/// );
/// ```
///
/// ```rust,ignore
/// pub fn sd_journal_enumerate_available_data(
///     j: *mut sd_journal,
///     data: *mut *const ::std::os::raw::c_void,
///     l: *mut size_t,
/// ) -> ::std::os::raw::c_int;
/// ```
///
/// [`sd_journal_enumerate_available_data`]: https://man7.org/linux/man-pages/man3/sd_journal_get_data.3.html
#[allow(non_camel_case_types)]
pub type func_sig_sd_journal_enumerate_available_data = unsafe extern fn(
    ret: *mut sd_journal,
    data: *mut *const ::std::os::raw::c_void,
    l: *mut size_t,
) -> ::std::os::raw::c_int;

const FUNC_NAME_JOURNAL_ENUMERATE_AVAILABLE_DATA: &str = "sd_journal_enumerate_available_data";

lazy_static! {
    static ref LIB_SYSTEMD: RwLock<Option<Box<Library>>>
        = RwLock::new(None);

    /// Function pointer to [`sd_journal_open_files`] function.
    ///
    /// [`sd_journal_open_files`]: https://man7.org/linux/man-pages/man3/sd_journal_open.3.html
    static ref FUNC_SD_JOURNAL_OPEN_FILES: RwLock<Option<Symbol<'static, func_sig_sd_journal_open_files>>>
        = RwLock::new(None);

    /// Function pointer to [`sd_journal_seek_head`] function.
    ///
    /// [`sd_journal_seek_head`]: https://man7.org/linux/man-pages/man3/sd_journal_seek_head.3.html
    static ref FUNC_SD_JOURNAL_SEEK_HEAD: RwLock<Option<libloading::Symbol<'static, func_sig_sd_journal_seek_head>>>
        = RwLock::new(None);

    /// Function pointer to [`sd_journal_seek_realtime_usec`] function.
    ///
    /// [`sd_journal_seek_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_seek_head.3.html
    static ref FUNC_SD_JOURNAL_SEEK_REALTIME_USEC: RwLock<Option<libloading::Symbol<'static, func_sig_sd_journal_seek_journal_realtime_usec>>>
        = RwLock::new(None);

    /// Function pointer to [`sd_journal_next`] function.
    ///
    /// [`sd_journal_next`]: https://man7.org/linux/man-pages/man3/sd_journal_next.3.html
    static ref FUNC_SD_JOURNAL_NEXT: RwLock<Option<libloading::Symbol<'static, func_sig_sd_journal_next>>>
        = RwLock::new(None);

    /// Function pointer to [`sd_journal_get_realtime_usec`] function.
    ///
    /// [`sd_journal_get_realtime_usec`]: https://www.man7.org/linux/man-pages/man3/sd_journal_get_realtime_usec.3.html
    static ref FUNC_SD_JOURNAL_GET_REALTIME_USEC: RwLock<Option<libloading::Symbol<'static, func_sig_sd_journal_get_realtime_usec>>>
        = RwLock::new(None);

    /// Function pointer to [`sd_journal_enumerate_available_data`] function.
    ///
    /// [`sd_journal_enumerate_available_data`]: https://man7.org/linux/man-pages/man3/sd_journal_get_data.3.html
    static ref FUNC_SD_JOURNAL_ENUMERATE_AVAILABLE_DATA: RwLock<Option<libloading::Symbol<'static, func_sig_sd_journal_enumerate_available_data>>>
        = RwLock::new(None);

}

/// Load the shared library and get function pointers to
/// functions of interest.
pub fn load_library() -> Result<(), ::libloading::Error> {
    /*
    unsafe {
        defo!("libloading::Library::new({:?})", LIB_NAME);
        let lib: Box<Library> = Box::new(
            match Library::new(LIB_NAME) {
                Ok(lib) => lib,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            }
        );

        defo!("lib.get({:?})", FUNC_NAME_JOURNAL_OPEN_FILES);
        let func_sd_journal_open_files: Symbol<func_sig_sd_journal_open_files> =
            match (*lib).get(FUNC_NAME_JOURNAL_OPEN_FILES.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            };

        defo!("lib.get({:?})", FUNC_NAME_JOURNAL_SEEK_HEAD);
        let func_sd_journal_seek_head: Symbol<func_sig_sd_journal_seek_head> =
            match lib.get(FUNC_NAME_JOURNAL_SEEK_HEAD.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            };

        defo!("lib.get({:?})", FUNC_NAME_JOURNAL_SEEK_REALTIME_USEC);
        let func_sd_journal_seek_realtime_usec: Symbol<func_sig_sd_journal_seek_journal_realtime_usec> =
            match lib.get(FUNC_NAME_JOURNAL_SEEK_REALTIME_USEC.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            };

        defo!("lib.get({:?})", FUNC_NAME_JOURNAL_NEXT);
        let func_sd_journal_next: Symbol<func_sig_sd_journal_next> =
            match lib.get(FUNC_NAME_JOURNAL_NEXT.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            };

        defo!("lib.get({:?})", FUNC_NAME_JOURNAL_GET_REALTIME_USEC);
        let func_sd_journal_get_realtime_usec: Symbol<func_sig_sd_journal_get_realtime_usec> =
            match lib.get(FUNC_NAME_JOURNAL_GET_REALTIME_USEC.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            };

        defo!("lib.get({:?})", FUNC_NAME_JOURNAL_ENUMERATE_AVAILABLE_DATA);
        let func_sd_journal_enumerate_available_data: Symbol<func_sig_sd_journal_enumerate_available_data> =
            match lib.get(FUNC_NAME_JOURNAL_ENUMERATE_AVAILABLE_DATA.as_bytes()) {
                Ok(func) => func,
                Err(err) => {
                    defx!("Error {:?}", err);
                    return Err(err);
                }
            };

        *FUNC_SD_JOURNAL_OPEN_FILES.write().unwrap() = Some(func_sd_journal_open_files);
        *FUNC_SD_JOURNAL_SEEK_HEAD.write().unwrap() = Some(func_sd_journal_seek_head);
        *FUNC_SD_JOURNAL_SEEK_REALTIME_USEC.write().unwrap() = Some(func_sd_journal_seek_realtime_usec);
        *FUNC_SD_JOURNAL_NEXT.write().unwrap() = Some(func_sd_journal_next);
        *FUNC_SD_JOURNAL_GET_REALTIME_USEC.write().unwrap() = Some(func_sd_journal_get_realtime_usec);
        *FUNC_SD_JOURNAL_ENUMERATE_AVAILABLE_DATA.write().unwrap() = Some(func_sd_journal_enumerate_available_data);
        *LIB_SYSTEMD.write().unwrap() = Some(lib);
    } // unsafe
    */

    Ok(())
}
