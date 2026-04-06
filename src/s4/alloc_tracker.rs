// src/s4/alloc_tracker.rs

/// A custom global allocator that wraps the system allocator and tracks call
/// sites of allocations.
/// Provides functions to print a summary at program exit.
/// 
/// Users can control it with environment variables:
/// - `S4_ALLOC_TRACKER_PRINT` to print a backtrace of each allocation and
/// - `S4_ALLOC_TRACKER_TRACKING` to track allocator statistics.

use std::alloc::{
    GlobalAlloc,
    Layout,
    System,
};
use std::collections::HashMap;
use std::io::Write as StdWrite;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use std::sync::RwLock;

use ::backtrace::SymbolName;
use ::lazy_static::lazy_static;
use ::rustc_demangle::{
    Demangle,
    demangle,
};
use ::thousands::Separable;

use ::s4lib::common::threadid_to_u64;
use ::s4lib::e_err;

use crate::s4::{
    EXIT_ERR,
};

/// alloc error exit value (ENOMEM)
const EXIT_ALLOC_ERR: i32 = 12;

/// A simple fixed-size buffer that most importantly can be written to by
/// `core::fmt::write`.
/// Safe for using within `alloc`.
/// Use the provided API so `.len` is correctly updated.
struct FmtBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FmtBuf<N> {
    pub fn new() -> Self {
        Self {
            buf: [0u8; N],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Appends as much of `bytes` as fits in the buffer.
    /// Returns `Err(core::fmt::Error)` if not all bytes were copied.
    pub fn append_bytes(&mut self, bytes: &[u8]) -> core::fmt::Result {
        let remaining = self.buf.len().saturating_sub(self.len);
        let copy_len = bytes.len().min(remaining);
        let end: usize = self.len + copy_len;
        self.buf[self.len..end].copy_from_slice(&bytes[..copy_len]);
        self.len = end;

        if copy_len < bytes.len() {
            return Err(core::fmt::Error);
        }

        Ok(())
    }

    pub fn append_byte(&mut self, byte: u8) -> core::fmt::Result {
        self.append_bytes(&[byte])
    }

    pub fn append_byte_or_wide_string(&mut self, b: &backtrace::BytesOrWideString) -> core::fmt::Result {
        match b {
            backtrace::BytesOrWideString::Bytes(bytes) => self.append_bytes(bytes),
            backtrace::BytesOrWideString::Wide(wide) => self.append_wide_string(wide),
        }
    }

    /// copies as much from the UTF-16 string as possible until the buffer is full.
    pub fn append_wide_string(&mut self, wide: &[u16]) -> core::fmt::Result {
        // XXX: jenky copy of UTF16
        for &c in wide.iter() {
            if c <= 0xFF {
                match self.append_byte(c as u8) {
                    Ok(()) => {},
                    Err(err) => return Err(err),
                }
            }
            // else non-ASCII char, skip it
        }
        Ok(())
    }

    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.as_bytes().starts_with(prefix)
    }

    pub fn clear(&mut self) {
        self.buf.fill(0);
        self.len = 0;
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<const N: usize> core::fmt::Write for FmtBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.append_bytes(s.as_bytes())
    }
}

/// call `core::fmt::write` with a provided buffer (a `FmtBuf` most likely).
/// Write the buffer to stderr.
/// No allocations are performed.
///
/// Returns the locked stderr for further writing if needed.
macro_rules! alloc_stderr_write_fmt {
    ($buf:expr, $($arg:tt)*) => {{
        let buf = $buf;
        _ = core::fmt::write(buf, format_args!($($arg)*));
        let mut stderr_lock = std::io::stderr().lock();
        _ = stderr_lock.write(buf.as_bytes());
        _ = stderr_lock.flush();

        stderr_lock
    }};
}

/// `(file index, function index, line, column, threadname index, threadid)` uniquely identifies an allocation call site.
/// `file index` is for `ALLOCATOR_TRACKING_FILENAMES`
/// `function index` is for `ALLOCATOR_TRACKING_FUNCTIONS`
/// `threadname index` is for `ALLOCATOR_TRACKING_THREADNAMES`
pub type AllocatorDebugTrackingKey = (usize, usize, u32, u32, usize, u64);
/// `(allocator call count, total allocated bytes)` for a given call site.
pub type AllocatorDebugTrackingValue = (usize, usize);
pub type AllocatorDebugTrackingMap = HashMap<AllocatorDebugTrackingKey, AllocatorDebugTrackingValue>;

std::thread_local! {
    /// Guards against infinite loops within `fn alloc` that are caused by allocations
    /// within `fn alloc` (e.g. by `backtrace::resolve_frame` or by formatting the debug info).
    static ALLOCATOR_GUARD: RwLock<bool> = RwLock::new(false);
    /// Thread global buffer for printing debug info about the allocator backtrace.
    static ALLOCATOR_FMT_BUF: RwLock<FmtBuf<10_000>> = RwLock::new(FmtBuf::<10_000>::new());
    /// Print allocator backtraces?
    static ALLOCATOR_DO_PRINT: RwLock<bool> = RwLock::new(false);
    /// Track allocator statistics?
    static ALLOCATOR_DO_TRACKING: RwLock<bool> = RwLock::new(false);
    /// Cache the thread ID to avoid allocs.
    static ALLOCATOR_TID: u64 = threadid_to_u64(std::thread::current().id());
    /// Cache the thread name to avoid allocs.
    static ALLOCATOR_TNAME: String = std::thread::current().name().unwrap_or("<unnamed>").to_string();
}
/// User-set environment variable to enable printing of allocator backtraces.
const ENV_ALLOCATOR_PRINT: &str = "S4_ALLOC_TRACKER_PRINT";
/// User-set environment variable to enable tracking of allocator call sites.
const ENV_ALLOCATOR_TRACKING: &str = "S4_ALLOC_TRACKER_TRACKING";

// Global allocator statistics
static ALLOCATOR_ALLOCATED_TRACKING_OFF: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_ALLOCATED_TRACKING_ON: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_CALLS_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_ALLOCATED_CURRENT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_ALLOCATED_TRACKED_FRAME: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_CALLS_TRACKING_OFF: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_CALLS_TRACKING_ON: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// Global map for tracking allocator call sites and their statistics.
    pub(super)static ref ALLOCATOR_TRACKING_MAP: RwLock<AllocatorDebugTrackingMap> =
        RwLock::new(AllocatorDebugTrackingMap::with_capacity(2056));
    /// Global table of all file names seen while resolving backtrace frames.
    /// Indexes to this table are saved in `ALLOCATOR_TRACKING_MAP`
    pub(super) static ref ALLOCATOR_TRACKING_FILENAMES: RwLock<Vec<String>> =
        RwLock::new(Vec::with_capacity(100));
    /// Global table of all function names seen while resolving backtrace frames.
    /// Indexes to this table are saved in `ALLOCATOR_TRACKING_MAP`
    pub(super) static ref ALLOCATOR_TRACKING_FUNCTIONS: RwLock<Vec<String>> =
        RwLock::new(Vec::with_capacity(300));
    /// Global table of all thread names seen while resolving backtrace frames.
    /// Indexes to this table are saved in `ALLOCATOR_TRACKING_MAP`
    pub(super) static ref ALLOCATOR_TRACKING_THREADNAMES: RwLock<Vec<String>> =
        RwLock::new(Vec::with_capacity(50));

    pub(super) static ref PROJECT_ROOT: String = match ::project_root::get_project_root() {
        Ok(root) => root.to_string_lossy().to_string(),
        Err(_) => String::from(""),
    };
    pub(super) static ref PROJECT_ROOT_BYTES: Vec<u8> = PROJECT_ROOT.as_bytes().to_vec();
}

/// Checks environment variable `S4_ALLOC_TRACKER_PRINT` to see if we should print allocator backtraces.
fn allocator_print() -> bool {
    std::env::var(ENV_ALLOCATOR_PRINT).map(|val| !val.is_empty()).unwrap_or(false)
}

/// Checks environment variable `S4_ALLOC_TRACKER_TRACKING` to determine if
/// tracking allocator call sites and their statistics should be enabled.
fn allocator_tracking() -> bool {
    std::env::var(ENV_ALLOCATOR_TRACKING).map(|val| !val.is_empty()).unwrap_or(false)
}

/// Tracking and printing will begin after this is called.
/// Must call this for each thread to track.
pub fn allocator_tracker_enable() {
    // force ThreadId and ThreadName to get set once early on
    ALLOCATOR_TID.with(|_| {});
    ALLOCATOR_TNAME.with(|_| {});
    // secret environment variable option to print the stack strace of each alloc_tracker
    if allocator_print() {
        ALLOCATOR_DO_PRINT.with(|adep| match adep.write() {
            Ok(mut adepw) => *adepw = true,
            Err(err) => {
                e_err!("ALLOCATOR_DO_PRINT.write() failed: {:?}", err);
                std::process::exit(EXIT_ERR);
            }
        });
    }
    if allocator_tracking() {
        ALLOCATOR_DO_TRACKING.with(|adet| match adet.write() {
            Ok(mut adetw) => *adetw = true,
            Err(err) => {
                e_err!("ALLOCATOR_DO_TRACKING.write() failed: {:?}", err);
                std::process::exit(EXIT_ERR);
            }
        });
    }
    // start alloc_tracker debug logging
    allocator_guard_enable();
}

/// get the value of the guard
#[inline(always)]
fn allocator_guard() -> bool {
    ALLOCATOR_GUARD.with(|ap| match ap.read() {
        Ok(apr) => *apr,
        Err(_err) => {
            e_err!("ALLOCATOR_GUARD.read() failed in allocator_guard");
            std::process::exit(EXIT_ERR);
        }
    })
}

/// Enable the guard (allow allocation debug activity)
#[inline(always)]
fn allocator_guard_enable() {
    // set the guard
    ALLOCATOR_GUARD.with(|ap| match ap.write() {
        Ok(mut apw) => *apw = true,
        Err(_err) => {
            e_err!("ALLOCATOR_GUARD.write() failed in allocator_guard_enable");
            std::process::exit(EXIT_ERR);
        }
    });
}

/// Disable the guard (disallow allocation debug activity)
#[inline(always)]
fn allocator_guard_disable() {
    ALLOCATOR_GUARD.with(|ap| match ap.write() {
        Ok(mut apw) => *apw = false,
        Err(_err) => {
            e_err!("ALLOCATOR_GUARD.write() failed in allocator_guard_disable");
            std::process::exit(EXIT_ERR);
        }
    });
}

/// Restore the guard to the value
#[inline(always)]
fn allocator_guard_restore(allocator_guard: bool) {
    ALLOCATOR_GUARD.with(|ap| match ap.write() {
        Ok(mut apw) => *apw = allocator_guard,
        Err(_err) => {
            e_err!("ALLOCATOR_GUARD.write() failed in allocator_guard_restore");
            std::process::exit(EXIT_ERR);
        }
    });
}

/// Exit with code, print the `mesg` and optional `err` without allocating.
/// Intended for errors within `alloc`.
fn alloc_exit(mesg: &str, err: Option<&dyn std::error::Error>) -> ! {
    let mut stderr_lock = std::io::stderr().lock();
    _ = stderr_lock.write(mesg.as_bytes());
    if let Some(e) = err {
        _ = stderr_lock.write(b": ");
        // prefer `description` instead of `Debug` or `Display` to avoid potential call to `alloc`
        #[allow(deprecated)]
        let d = e.description();
        _ = stderr_lock.write(d.as_bytes());
    }
    _ = stderr_lock.write(b"\n");
    _ = stderr_lock.flush();
    std::process::exit(EXIT_ALLOC_ERR);
}

/// Try to get the demangled name, fallback to the raw name if demangling fails.
/// Write to `out` to avoid allocations.
fn demangle_name<const N: usize>(
    symbol_name: &SymbolName,
    out: &mut FmtBuf<N>,
) -> core::fmt::Result {
    let name_b: &[u8] = symbol_name.as_bytes();
    let name_s: &str = match str::from_utf8(name_b) {
        Ok(s) => s,
        Err(_) => return out.append_bytes(name_b),
    };
    let demangled: Demangle = demangle(name_s);
    core::fmt::write(out, format_args!("{}", demangled))
}

/// Intern a string into a global table and return its index.
fn tracking_intern_index(table: &RwLock<Vec<String>>, value: &str) -> usize {
    let mut guard = match table.write() {
        Ok(guard) => guard,
        Err(err) => alloc_exit("tracking_intern_index.table.write() failed", Some(&err)),
    };
    if let Some(idx) = guard.iter().position(|entry| entry == value) {
        return idx;
    }
    guard.push(value.to_string());
    guard.len() - 1
}

#[inline(always)]
fn tracking_filename_index(filename: &str) -> usize {
    tracking_intern_index(&ALLOCATOR_TRACKING_FILENAMES, filename)
}

#[inline(always)]
fn tracking_function_index(function: &str) -> usize {
    tracking_intern_index(&ALLOCATOR_TRACKING_FUNCTIONS, function)
}

#[inline(always)]
fn tracking_threadname_index(threadname: &str) -> usize {
    tracking_intern_index(&ALLOCATOR_TRACKING_THREADNAMES, threadname)
}

pub struct AllocTrackerImpl;

unsafe impl GlobalAlloc for AllocTrackerImpl {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe {
            // do the actual allocation
            System.alloc(layout)
        };
        if ret.is_null() {
            // https://doc.rust-lang.org/1.94.1/std/alloc/trait.GlobalAlloc.html#tymethod.alloc
            //     Implementations are encouraged to return null on memory exhaustion rather than aborting
            return ret;
        }
        let sz: usize = layout.size();
        ALLOCATOR_ALLOCATED_CURRENT.fetch_add(sz, Ordering::Relaxed);

        let allocator_guard_: bool = allocator_guard();
        if ! allocator_guard_ {
            ALLOCATOR_ALLOCATED_TRACKING_OFF.fetch_add(sz, Ordering::Relaxed);
            ALLOCATOR_CALLS_TRACKING_OFF.fetch_add(1, Ordering::Relaxed);
            return ret;
        }
        allocator_guard_disable();
        ALLOCATOR_ALLOCATED_TRACKING_ON.fetch_add(sz, Ordering::Relaxed);
        ALLOCATOR_CALLS_TRACKING_ON.fetch_add(1, Ordering::Relaxed);

        ALLOCATOR_FMT_BUF.with(|ap| {
            match ap.write() {
                Ok(mut apw) => apw.clear(),
                Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", Some(&err)),
            }
        });
        let allocator_do_print: bool = ALLOCATOR_DO_PRINT.with(|ap|
            match ap.read() {
                Ok(ap) => *ap,
                Err(err) => alloc_exit("ALLOCATOR_DO_PRINT.read() failed", Some(&err)),
            }
        );
        let allocator_do_track: bool = ALLOCATOR_DO_TRACKING.with(|ap|
            match ap.read() {
                Ok(ap) => *ap,
                Err(err) => alloc_exit("ALLOCATOR_DO_TRACKING.read() failed", Some(&err)),
            }
        );

        const FILENAME_LEN_MAX: usize = 512;
        const FUNCTION_LEN_MAX: usize = 256;

        let mut frames_skipped: usize = 0;
        let mut frames_project: usize = 0;
        let mut frame_tracked: bool = false;

        // Traverse the backtrace frames for this allocation
        // looking for the first frame that refers to this project's source code
        // and tracking it as the call site of this allocation.
        // Also print info about each frame if `allocator_do_print` is true.
        // `backtrace::resolve_frame` allocates, hence the need for `allocator_guard` to prevent infinite loops.
        ::backtrace::trace(|frame| {
            ::backtrace::resolve_frame(frame, |symbol| {
                match symbol.filename_raw() {
                    // only consider tracking frames referring to source code from project files
                    Some(filename) => {
                        match filename {
                            ::backtrace::BytesOrWideString::Bytes(b) => {
                                if ! b.starts_with(&PROJECT_ROOT_BYTES) {
                                    // this frame is not project source code, skip it
                                    frames_skipped += 1;
                                    return;
                                }
                            }
                            ::backtrace::BytesOrWideString::Wide(w) => {
                                // XXX: jenky copy of UTF16
                                let mut filename_bytes: FmtBuf<FILENAME_LEN_MAX> = FmtBuf::new();
                                match filename_bytes.append_wide_string(&w) {
                                    Ok(()) => {},
                                    Err(_err) => alloc_exit("filename_bytes.append_wide_string() failed", None),
                                };
                                if ! filename_bytes.starts_with(&PROJECT_ROOT_BYTES) {
                                    // this frame is not project source code, skip it
                                    frames_skipped += 1;
                                    return;
                                }
                            }
                        }
                    }
                    None => return,
                }
                frames_project += 1;
                if frames_project < 3 {
                    // skip first two frames which always refer to this allocator code and the backtrace code
                    return;
                }
                if allocator_do_print {
                    // accumulate info to print about this frame in the thread-local buffer
                    ALLOCATOR_FMT_BUF.with(|ap| {
                        match ap.write() {
                            Ok(mut apw) => {
                                _ = apw.append_bytes(b"  frame: ");
                                let ip = frame.ip();
                                _ = apw.append_bytes(format!("ip={:?}", ip).as_bytes());
                                let sp = frame.sp();
                                _ = apw.append_bytes(format!(" sp={:?}", sp).as_bytes());
                                let symbol_address = frame.symbol_address();
                                _ = apw.append_bytes(format!(" symbol_address=@{:?}", symbol_address).as_bytes());
                                let module_base_address = frame.module_base_address();
                                if module_base_address.is_some() {
                                    _ = apw.append_bytes(format!(" module_base_address=@{:?}", module_base_address).as_bytes());
                                }
                                _ = apw.append_bytes(b"\n");
                                if let Some(symbol_name) = symbol.name() {
                                    _ = apw.append_bytes(b"         name=");
                                    _ = demangle_name(&symbol_name, &mut apw);
                                    _ = apw.append_bytes(b"\n");
                                }
                                let mut nl = false;
                                if let Some(filename) = symbol.filename_raw() {
                                    _ = apw.append_bytes(b"         filename=");
                                    _ = apw.append_bytes(filename.to_str_lossy().as_bytes());
                                    nl = true;
                                }
                                if let Some(lineno) = symbol.lineno() {
                                    _ = apw.append_bytes(format!("  lineno={}", lineno).as_bytes());
                                    nl = true;
                                }
                                if let Some(colno) = symbol.colno() {
                                    _ = apw.append_bytes(format!("  colno={}", colno).as_bytes());
                                    nl = true;
                                }
                                if nl {
                                    _ = apw.append_bytes(b"\n");
                                }
                            }
                            Err(_err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", None),
                        }
                    });
                }
                if allocator_do_track && !frame_tracked {
                    // only track this frame, which should be the last frame of control from this
                    // project's source code before it calls into some other module's code.
                    frame_tracked = true;
                    ALLOCATOR_ALLOCATED_TRACKED_FRAME.fetch_add(sz, Ordering::Relaxed);
                    // track this allocation call site in the thread-local map
                    let mut apw = match ALLOCATOR_TRACKING_MAP.write() {
                        Ok(apw) => apw,
                        Err(err) => alloc_exit("ALLOCATOR_TRACKING_MAP.write() failed", Some(&err)),
                    };

                    // get filename
                    const FILENAME_LEN_MAX: usize = 512;
                    let mut filename_buf = FmtBuf::<FILENAME_LEN_MAX>::new();
                    match symbol.filename_raw() {
                        Some(symbol_filename) => {
                            filename_buf.append_byte_or_wide_string(&symbol_filename).unwrap_or_else(
                                |_| alloc_exit("filename_buf.append_byte_or_wide_string() failed", None)
                            );
                        }
                        None => {
                            _ = filename_buf.append_bytes(b"<unknown file (symbol.filename_raw() was None)>");
                        }
                    }
                    let filename_s = unsafe { str::from_utf8_unchecked(filename_buf.as_bytes()) };

                    // get function name
                    let mut function_buf = FmtBuf::<FUNCTION_LEN_MAX>::new();
                    match symbol.name() {
                        Some(name) => {
                            _ = demangle_name(&name, &mut function_buf);
                        }
                        None => {
                            _ = function_buf.append_bytes(b"<unknown function (symbol.name() was None)>");
                        }
                    };
                    let function_s = unsafe { str::from_utf8_unchecked(function_buf.as_bytes()) };

                    let filename_idx: usize = tracking_filename_index(filename_s);
                    let function_idx: usize = tracking_function_index(function_s);
                    let lineno: u32 = symbol.lineno().unwrap_or(0);
                    let colno: u32 = symbol.colno().unwrap_or(0);
                    let tid = ALLOCATOR_TID.with(|ap| *ap);
                    let threadname = ALLOCATOR_TNAME.with(|ap| ap.clone());
                    let threadname_idx: usize = tracking_threadname_index(&threadname);
                    let key: AllocatorDebugTrackingKey = (filename_idx, function_idx, lineno, colno, threadname_idx, tid);
                    let mut filename_slice: &[u8] = filename_buf.as_bytes();
                    // remove leading project root from file path to make it more readable
                    let pr_bytes: &[u8] = &PROJECT_ROOT_BYTES;
                    if filename_buf.starts_with(pr_bytes) {
                        filename_slice = &filename_slice[pr_bytes.len()..];
                    };
                    ALLOCATOR_FMT_BUF.with(|ap| {
                        match ap.write() {
                            Ok(mut apw) => {
                                let s = unsafe { str::from_utf8_unchecked(filename_slice) };
                                core::fmt::write(
                                    &mut *apw,
                                    format_args!(
                                        "\
           tracked allocation call site:
             {s}:{lineno}:{colno}  [{function_s}]
             thread [{tid}]: {threadname}\n",
                                    )
                                ).unwrap_or_else(
                                    |err| alloc_exit("core::fmt::write() failed in ALLOCATOR_FMT_BUF.write() A", Some(&err))
                                );
                            }
                            Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed A", Some(&err)),
                        }
                    });
                    let value: &mut AllocatorDebugTrackingValue = apw.entry(key).or_insert((0, 0));
                    ALLOCATOR_FMT_BUF.with(|ap| {
                        match ap.write() {
                            Ok(mut apw) => {
                                core::fmt::write(
                                    &mut *apw,
                                    format_args!(
                                        "\
               {:>8} allocations, {:>10} bytes\n", value.0, value.1
                                    )
                                ).unwrap_or_else(
                                    |err| alloc_exit("core::fmt::write() failed in ALLOCATOR_FMT_BUF.write() B", Some(&err))
                                );
                            }
                            Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed B", Some(&err)),
                        }
                    });
                    value.0 += 1;
                    value.1 += sz;
                }
            });

            // `true` means continue to next frame, else stop bracktrace traversal.
            allocator_do_print || (allocator_do_track && !frame_tracked)
        });
        if allocator_do_print {
            let mut buf = FmtBuf::<256>::new();
            let aa = ALLOCATOR_ALLOCATED_TRACKING_ON.load(Ordering::Relaxed);
            let tid = ALLOCATOR_TID.with(|ap| *ap);
            // write this message
            let align = layout.align();
            let mut stderr_lock = alloc_stderr_write_fmt!(
                &mut buf,
                "system_alloc_debug: (TID {tid}) @{ret:?} sz={sz:2} align={align:2} total_allocated={aa}\n",
            );
            // then write the built-up message in `ALLOCATOR_FMT_BUF` during the backtrace traversal
            ALLOCATOR_FMT_BUF.with(|ap| {
                match ap.read() {
                    Ok(ap) => _ = stderr_lock.write(ap.as_bytes()),
                    Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.read() failed", Some(&err)),
                }
            });
            _ = stderr_lock.flush();
            drop(stderr_lock);
            // clear `ALLOCATOR_FMT_BUF` for next allocation debug info
            ALLOCATOR_FMT_BUF.with(|ap| {
                match ap.write() {
                    Ok(mut apw) => apw.clear(),
                    Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", Some(&err)),
                }
            });
        }

        // restore the guard
        allocator_guard_restore(allocator_guard_);

        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(ptr, layout);
        }
        let sz: usize = layout.size();
        ALLOCATOR_DEALLOCATED.fetch_add(sz, Ordering::Relaxed);
        ALLOCATOR_CALLS_DEALLOCATED.fetch_add(1, Ordering::Relaxed);
        ALLOCATOR_ALLOCATED_CURRENT.fetch_sub(sz, Ordering::Relaxed);
    }
}


/// Prints the contents of the `ALLOCATOR_TRACKING_MAP` in a user-friendly way.
/// This special print function sits outside of the normal `--summary` stuff.
/// It is presumed this will be called last before program exit.
/// Avoids allocations.
pub fn print_tracking_map() {
    // must turn off to avoid stack overflow
    allocator_guard_disable();

    if !allocator_tracking() {
        return;
    }

    let project_root_ = (*PROJECT_ROOT).clone();
    let ap = match ALLOCATOR_TRACKING_MAP.write() {
        Ok(ap) => ap,
        Err(err) => alloc_exit("ALLOCATOR_TRACKING_MAP.write() failed in print_tracking_map", Some(&err)),
    };
    let mut entries: Vec<(&AllocatorDebugTrackingKey, &AllocatorDebugTrackingValue)>
        = ap.iter().collect();
    let entry_len = entries.len();
    entries.sort_by_key(|(key, value)| (value.0, value.1, key.5));
    entries.reverse();
    let filenames = match ALLOCATOR_TRACKING_FILENAMES.read() {
        Ok(filenames) => filenames,
        Err(err) => alloc_exit("ALLOCATOR_TRACKING_FILENAMES.read() failed in print_tracking_map", Some(&err)),
    };
    let functions = match ALLOCATOR_TRACKING_FUNCTIONS.read() {
        Ok(functions) => functions,
        Err(err) => alloc_exit("ALLOCATOR_TRACKING_FUNCTIONS.read() failed in print_tracking_map", Some(&err)),
    };
    let threadnames = match ALLOCATOR_TRACKING_THREADNAMES.read() {
        Ok(threadnames) => threadnames,
        Err(err) => alloc_exit("ALLOCATOR_TRACKING_THREADNAMES.read() failed in print_tracking_map", Some(&err)),
    };

    // print a rudimentary table of the tracked allocator call sites and their stats
    // sorted by allocated bytes descending

    let mut buf = FmtBuf::<2056>::new();
    buf.clear();
    _ = alloc_stderr_write_fmt!(
        &mut buf,
        "{:<40} | {:>5}:{:>3} | {:>3}:{:<16} | {:<100} | {:>10} | {:>13}\n",
        "File", "Line", "Col", "ID", "Name (thread)", "Function", "Allocations", "Bytes"
    );
    for (key, value) in entries.into_iter() {
        let file_name = match filenames.get(key.0) {
            Some(name) => name.as_str(),
            None => "<unknown>",
        };
        let function_name = match functions.get(key.1) {
            Some(name) => name.as_str(),
            None => "<unknown>",
        };
        let thread_name = match threadnames.get(key.4) {
            Some(name) => name.as_str(),
            None => "<unknown>",
        };
        // remove leading project root from file path to make it more readable
        let file_path: &str = if file_name.starts_with(&project_root_) {
            &file_name[project_root_.len() + 1..]
        } else {
            file_name
        };
        // attempt to print some user-friendly columns
        let line_number = &key.2;
        let column_number = &key.3;
        let thread_id: &u64 = &key.5;
        let allocations = &value.0;
        let allocations_s = allocations.separate_with_commas();
        let bytes = &value.1;
        let bytes_s = bytes.separate_with_commas();
        buf.clear();
        _ = alloc_stderr_write_fmt!(
            &mut buf,
            "{file_path:<40} | {line_number:>5}:{column_number:>3} | {thread_id:>3}:{thread_name:<16} | {function_name:<100} | {allocations_s:>11} | {bytes_s:>13}\n"
        );
    }
    buf.clear();
    _ = alloc_stderr_write_fmt!(&mut buf, "\n");

    // print the summary of the allocator tracking stats

    let a_t_on_bytes = ALLOCATOR_ALLOCATED_TRACKING_ON.load(Ordering::Relaxed);
    let a_t_on_bytes_s = a_t_on_bytes.separate_with_commas();
    let a_t_on_calls = ALLOCATOR_CALLS_TRACKING_ON.load(Ordering::Relaxed);
    let a_t_on_calls_s = a_t_on_calls.separate_with_commas();
    let d = ALLOCATOR_DEALLOCATED.load(Ordering::Relaxed);
    let d_s = d.separate_with_commas();
    let d_calls = ALLOCATOR_CALLS_DEALLOCATED.load(Ordering::Relaxed);
    let d_calls_s = d_calls.separate_with_commas();
    let a_t_off_bytes = ALLOCATOR_ALLOCATED_TRACKING_OFF.load(Ordering::Relaxed);
    let a_t_off_bytes_s = a_t_off_bytes.separate_with_commas();
    let a_t_off_calls = ALLOCATOR_CALLS_TRACKING_OFF.load(Ordering::Relaxed);
    let a_t_off_calls_s = a_t_off_calls.separate_with_commas();

    // ratio tracking on vs off
    // bytes
    let ratio_on_off: f64 = if a_t_off_bytes > 0 {
        (a_t_on_bytes as f64 / a_t_off_bytes as f64) * 100.0
    } else {
        0.0
    };
    let ratio_on_off_int: u32 = ratio_on_off.round() as u32;
    // calls
    let ratio_on_off_calls: f64 = if a_t_off_calls > 0 {
        (a_t_on_calls as f64 / a_t_off_calls as f64) * 100.0
    } else {
        0.0
    };
    let ratio_on_off_calls_int: u32 = ratio_on_off_calls.round() as u32;

    let filenames_len = filenames.len();
    let functions_len = functions.len();
    let threadnames_len = threadnames.len();

    // last thing to get is current allocated so it's nearest to final value at program exit
    let a_t_current = ALLOCATOR_ALLOCATED_CURRENT.load(Ordering::Relaxed);
    let a_t_current_s = a_t_current.separate_with_commas();

    const W_B: usize = 14; // width bytes
    const W_C: usize = 12; // width calls
    const W_CC: usize = 3; // width caches
    _ = alloc_stderr_write_fmt!(
        &mut buf,
        "allocator tracking summary:
  call sites   : {entry_len:>W_B$} (rows in the table above)
  allocations  : {a_t_on_bytes_s:>W_B$} bytes in {a_t_on_calls_s:>W_C$} calls
  deallocations: {d_s:>W_B$} bytes in {d_calls_s:>W_C$} calls (includes while tracking)
  from tracking: {a_t_off_bytes_s:>W_B$} bytes in {a_t_off_calls_s:>W_C$} calls
  current      : {a_t_current_s:>W_B$} bytes
  ratio tracking to normal: 100 to {ratio_on_off_int} bytes (100 to {ratio_on_off_calls_int} calls)
  cached file names    : {filenames_len:>W_CC$}
  cached function names: {functions_len:>W_CC$}
  cached thread names  : {threadnames_len:>W_CC$}
");
}
