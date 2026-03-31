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
use ::rustc_demangle::demangle;
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

    pub fn append_bytes(&mut self, bytes: &[u8]) -> core::fmt::Result {
        let remaining = self.buf.len().saturating_sub(self.len);
        if bytes.len() > remaining {
            return Err(core::fmt::Error);
        }
        let end: usize = self.len + bytes.len();
        self.buf[self.len..end].copy_from_slice(bytes);
        self.len = end;

        Ok(())
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


/// *(file index, function index, line, column, threadname index, threadid)* uniquely identifies an allocation call site.
pub type AllocatorDebugTrackingKey = (usize, usize, u32, u32, usize, u64);
/// *(allocator call count, total allocated bytes)* for a given call site.
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

/// Enable the guard (allow allocation debug activity)
#[inline(always)]
fn allocator_guard_enable() {
    // force ThreadId and ThreadName to get set once early on
    ALLOCATOR_TID.with(|_| {});
    ALLOCATOR_TNAME.with(|_| {});
    // set the guard
    ALLOCATOR_GUARD.with(|ap| match ap.write() {
        Ok(mut apw) => *apw = true,
        Err(_err) => {
            e_err!("ALLOCATOR_GUARD.write() failed");
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
            e_err!("ALLOCATOR_GUARD.write() failed");
            std::process::exit(EXIT_ERR);
        }
    });
}

// Global allocator statistics
static ALLOCATOR_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_ALLOCATED_TRACKED_FRAME: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_CURRENT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATOR_CALLS: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// Global map for tracking allocator call sites and their statistics. Only used if `ALLOCATOR_DO_TRACKING` is true.
    pub(super)static ref ALLOCATOR_TRACKING_MAP: RwLock<AllocatorDebugTrackingMap> =
        RwLock::new(AllocatorDebugTrackingMap::with_capacity(2056));
    /// Global table of all file names seen while resolving backtrace frames.
    pub(super) static ref ALLOCATOR_TRACKING_FILENAMES: RwLock<Vec<String>> =
        RwLock::new(Vec::with_capacity(100));
    /// Global table of all function names seen while resolving backtrace frames.
    pub(super) static ref ALLOCATOR_TRACKING_FUNCTIONS: RwLock<Vec<String>> =
        RwLock::new(Vec::with_capacity(100));
    /// Global table of all thread names seen while resolving backtrace frames.
    pub(super) static ref ALLOCATOR_TRACKING_THREADNAMES: RwLock<Vec<String>> =
        RwLock::new(Vec::with_capacity(50));

    pub(super) static ref PROJECT_ROOT: String = match ::project_root::get_project_root() {
        Ok(root) => root.to_string_lossy().to_string(),
        Err(_) => String::from(""),
    };
    static ref PROJECT_ROOT_SRC: String = {
        PROJECT_ROOT.clone() + "/src/"
    };
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
    drop(stderr_lock);
    std::process::exit(EXIT_ALLOC_ERR);
}

/// Try to get the demangled name, fallback to the raw name if demangling fails.
fn demangle_name(symbol_name: &SymbolName) -> String {
    let name_b = symbol_name.as_bytes();
    let name_s = str::from_utf8(name_b).unwrap_or("");
    let demangled = demangle(name_s);
    let demangled_name: String = format!("{}", demangled);
    if ! demangled_name.is_empty() {
        demangled_name
    }
    else {
        name_s.to_string()
    }
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
        // update stats
        ALLOCATOR_CALLS.fetch_add(1, Ordering::Relaxed);
        let ret = unsafe {
            // do the actual allocation
            System.alloc(layout)
        };
        // update stats again
        let sz: usize = layout.size();
        if !ret.is_null() {
            ALLOCATOR_ALLOCATED.fetch_add(sz, Ordering::Relaxed);
            ALLOCATOR_CURRENT.fetch_add(sz, Ordering::Relaxed);
        } else {
            let mut stderr_lock = std::io::stderr().lock();
            _ = stderr_lock.write(b"System.alloc() failed");
            _ = stderr_lock.flush();
            drop(stderr_lock);
            std::process::exit(EXIT_ALLOC_ERR);
        }

        let allocator_guard: bool = ALLOCATOR_GUARD.with(|ap|
            match ap.read() {
                Ok(ap) => *ap,
                Err(err) => alloc_exit("ALLOCATOR_GUARD.read() failed", Some(&err)),
            }
        );
        let allocator_do_print: bool = ALLOCATOR_DO_PRINT.with(|ap|
            match ap.read() {
                Ok(ap) => *ap,
                Err(err) => alloc_exit("ALLOCATOR_DO_PRINT.read() failed", Some(&err)),
            }
        );
        if allocator_guard {
            ALLOCATOR_GUARD.with(|ap|
                match ap.write() {
                    Ok(mut apw) => *apw = false,
                    Err(err) => alloc_exit("ALLOCATOR_GUARD.write() failed", Some(&err)),
                }
            );
            ALLOCATOR_FMT_BUF.with(|ap| {
                match ap.write() {
                    Ok(mut apw) => apw.clear(),
                    Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", Some(&err)),
                }
            });
            let allocator_do_track: bool = ALLOCATOR_DO_TRACKING.with(|ap|
                match ap.read() {
                    Ok(ap) => *ap,
                    Err(err) => alloc_exit("ALLOCATOR_DO_TRACKING.read() failed", Some(&err)),
                }
            );

            let mut frames_skipped: usize = 0;
            let mut frames_project: usize = 0;
            let mut frame_tracked: bool = false;
            ::backtrace::trace(|frame| {
                ::backtrace::resolve_frame(frame, |symbol| {
                    match symbol.filename_raw() {
                        Some(filename) => {
                            if ! filename.to_str_lossy().starts_with(PROJECT_ROOT_SRC.as_str()) {
                                // this frame is not project source code, skip it
                                frames_skipped += 1;
                                return;
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
                                    if let Some(name) = symbol.name() {
                                        _ = apw.append_bytes(b"         name=");
                                        let demangled_name = demangle_name(&name);
                                        _ = apw.append_bytes(demangled_name.as_bytes());
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
                        // project's source code into some other module's code.
                        frame_tracked = true;
                        ALLOCATOR_ALLOCATED_TRACKED_FRAME.fetch_add(sz, Ordering::Relaxed);
                        // track this allocation call site in the thread-local map
                        let mut apw = match ALLOCATOR_TRACKING_MAP.write() {
                            Ok(apw) => apw,
                            Err(err) => alloc_exit("ALLOCATOR_TRACKING_MAP.write() failed", Some(&err)),
                        };
                        let filename: String = symbol.filename_raw().map(|f| f.to_str_lossy().to_string()).unwrap_or_else(|| "<unknown>".to_string());
                        let function: String = match symbol.name() {
                            Some(name) => demangle_name(&name),
                            None => "<unknown>".to_string(),
                        };
                        let filename_idx: usize = tracking_filename_index(&filename);
                        let function_idx: usize = tracking_function_index(&function);
                        let lineno: u32 = symbol.lineno().unwrap_or(0);
                        let colno: u32 = symbol.colno().unwrap_or(0);
                        let tid = ALLOCATOR_TID.with(|ap| *ap);
                        let threadname = ALLOCATOR_TNAME.with(|ap| ap.clone());
                        let threadname_idx: usize = tracking_threadname_index(&threadname);
                        let key: AllocatorDebugTrackingKey = (filename_idx, function_idx, lineno, colno, threadname_idx, tid);
                        let mut filename_slice = filename.as_str();
                        // remove leading project root from file path to make it more readable
                        let pr_str = (*PROJECT_ROOT).as_str();
                        if let Some(i) = filename_slice.find(pr_str) {
                            filename_slice = &filename_slice[i + pr_str.len()..];
                        };
                        ALLOCATOR_FMT_BUF.with(|ap| {
                            match ap.write() {
                                Ok(mut apw) => {
                                    _ = apw.append_bytes(
                                        format!("           tracked allocation call site:\n             {}:{}:{}  [{}]\n             thread [{}]: {}\n",
                                            filename_slice, key.2, key.3, function, key.5, threadname
                                            ).as_bytes()
                                    );
                                }
                                Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", Some(&err)),
                            }
                        });
                        let value: &mut AllocatorDebugTrackingValue = apw.entry(key).or_insert((0, 0));
                        ALLOCATOR_FMT_BUF.with(|ap| {
                            match ap.write() {
                                Ok(mut apw) => {
                                    _ = apw.append_bytes(
                                        format!(
                                            "               {:>8} allocations, {:>10} bytes\n", value.0, value.1
                                        ).as_bytes()
                                    );
                                }
                                Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", Some(&err)),
                            }
                        });
                        value.0 += 1;
                        value.1 += sz;
                    }
                });

                // `true` means continue to next frame, else stop bracktrace traversal.
                allocator_do_print || (allocator_do_track && !frame_tracked)
            });

            ALLOCATOR_GUARD.with(|ap|
                match ap.write() {
                    Ok(mut ap) => *ap = allocator_guard,
                    Err(err) => alloc_exit("ALLOCATOR_GUARD.write() failed", Some(&err)),
                }
            );
        } // allocator_guard

        // print debug info
        if allocator_guard {
            if allocator_do_print {
                let mut buf = FmtBuf::<256>::new();
                let a = ALLOCATOR_ALLOCATED.load(Ordering::Relaxed);
                let tid = ALLOCATOR_TID.with(|ap| *ap);
                _ = core::fmt::write(
                    &mut buf,
                    format_args!(
                        "system_alloc_debug: (TID {}) @{:?} sz={:2} align={:2} total_allocated={}\n",
                        tid, ret, sz, layout.align(), a,
                    )
                );
                let mut stderr_lock = std::io::stderr().lock();
                _ = stderr_lock.write(&buf.as_bytes());
                ALLOCATOR_FMT_BUF.with(|ap| {
                    match ap.read() {
                        Ok(ap) => _ = stderr_lock.write(ap.as_bytes()),
                        Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.read() failed", Some(&err)),
                    }
                });
                _ = stderr_lock.flush();
                drop(stderr_lock);
                ALLOCATOR_FMT_BUF.with(|ap| {
                    match ap.write() {
                        Ok(mut apw) => apw.clear(),
                        Err(err) => alloc_exit("ALLOCATOR_FMT_BUF.write() failed", Some(&err)),
                    }
                });
            }
        }

        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(ptr, layout);
        }
        ALLOCATOR_DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        ALLOCATOR_CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}


/// Prints the contents of the `ALLOCATOR_TRACKING_MAP` in a user-friendly way.
/// This special print function sits outside of the normal `--summary` stuff.
/// It is presumed this will be called last before program exit.
pub fn print_tracking_map() {
    // disable the allocator debug guard to prevent infinite recursion
    // presumably the user does not need any more allocator debugging
    allocator_guard_disable();

    if !allocator_tracking() {
        return;
    }

    let project_root_ = (*PROJECT_ROOT).clone();
    let ap = match ALLOCATOR_TRACKING_MAP.write() {
        Ok(ap) => ap,
        Err(_err) => std::process::exit(11),
    };
    let mut entries: Vec<(&AllocatorDebugTrackingKey, &AllocatorDebugTrackingValue)>
        = ap.iter().collect();
    let entry_len = entries.len();
    entries.sort_by_key(|(key, value)| (value.0, value.1, key.5));
    entries.reverse();
    let filenames = match ALLOCATOR_TRACKING_FILENAMES.read() {
        Ok(filenames) => filenames,
        Err(_err) => std::process::exit(11),
    };
    let functions = match ALLOCATOR_TRACKING_FUNCTIONS.read() {
        Ok(functions) => functions,
        Err(_err) => std::process::exit(11),
    };
    let threadnames = match ALLOCATOR_TRACKING_THREADNAMES.read() {
        Ok(threadnames) => threadnames,
        Err(_err) => std::process::exit(11),
    };
    eprintln!(
        "{:<40} │ {:>5}:{:>3} │ {:>3}:{:<16} │ {:<100} │ {:>10} │ {:>10}",
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
        let bytes = &value.1;
        eprintln!(
            "{file_path:<40} │ {line_number:>5}:{column_number:>3} │ {thread_id:>3}:{thread_name:<16} │ {function_name:<100} │ {allocations:>11} │ {bytes:>10}"
        );
    }
    eprintln!();

    let allocator_allocated = ALLOCATOR_ALLOCATED.load(Ordering::Relaxed);
    let allocator_deallocated = ALLOCATOR_DEALLOCATED.load(Ordering::Relaxed);
    let allocator_allocated_tracked_frame = ALLOCATOR_ALLOCATED_TRACKED_FRAME.load(Ordering::Relaxed);
    let allocator_current = ALLOCATOR_CURRENT.load(Ordering::Relaxed);
    let allocator_calls = ALLOCATOR_CALLS.load(Ordering::Relaxed);
    eprintln!("call sites tracked: {} (rows in the table above)", entry_len);
    eprintln!("total allocated   : {:>14}", allocator_allocated.separate_with_commas());
    eprintln!("total deallocated : {:>14}", allocator_deallocated.separate_with_commas());
    eprintln!("total difference  : {:>14}",
        (allocator_allocated as isize - allocator_deallocated as isize).separate_with_commas());
    eprintln!("current allocated : {:>14}", allocator_current.separate_with_commas());
    eprintln!("total allocation calls: {}", allocator_calls.separate_with_commas());
    eprintln!("total allocated in tracked frames     : {:>14}",
        allocator_allocated_tracked_frame.separate_with_commas());
    eprintln!("total allocated outside tracked frames: {:>14}",
        (allocator_allocated - allocator_allocated_tracked_frame).separate_with_commas());

    let regex_captures = ::s4lib::data::datetime::REGEX_CAPTURES.load(Ordering::Relaxed);
    let regex_created = ::s4lib::data::datetime::REGEX_CREATED.load(Ordering::Relaxed);
    eprintln!();
    eprintln!("datetime regex captures: {:>3}", regex_captures);
    eprintln!("datetime regex created : {:>3}", regex_created);

    eprintln!();
    eprintln!(
        "cached filenames  : {:>3} unique (capacity {})",
        filenames.len(),
        filenames.capacity()
    );
    eprintln!(
        "cached functions  : {:>3} unique (capacity {})",
        functions.len(),
        functions.capacity()
    );
    eprintln!(
        "cached threadnames: {:>3} unique (capacity {})",
        threadnames.len(),
        threadnames.capacity()
    );
}
