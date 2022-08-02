// src/printer_debug/stack.rs
//
// functions to find current stack depth for indented trace prints
//

use std::collections::HashMap;
use std::thread;

extern crate const_format;
use const_format::concatcp;

extern crate lazy_static;
use lazy_static::lazy_static;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[allow(non_camel_case_types)]
type Map_ThreadId_SD<'a> = HashMap<thread::ThreadId, usize>;

// use `stack_offset_set` to set `_STACK_OFFSET_TABLE` once, use `stack_offset` to get
// XXX: no mutex to guard access; it's rarely written to 🤞
// XXX: a mutable static reference for "complex types" is not allowed in rust
//      use `lazy_static` and `mut_static` to create one
//      see https://github.com/tyleo/mut_static#quickstart
lazy_static! {
    static ref _STACK_OFFSET_TABLE: mut_static::MutStatic<Map_ThreadId_SD<'static>> =
        mut_static::MutStatic::new();
}

/// return current stack depth according to `backtrace::trace`, including this function
#[inline(always)]
pub fn stack_depth() -> usize {
    let mut sd: usize = 0;
    backtrace::trace(|_| {
        sd += 1;
        true
    });
    sd
}

/// return current stack offset compared to "original" stack depth. The "original" stack depth
/// should have been recorded at the beginning of the thread by calling `stack_offset_set`.
#[inline(always)]
pub fn stack_offset() -> usize {
    if ! (cfg!(debug_assertions) || cfg!(test)) {
        return 0;
    }
    let mut sd: usize = stack_depth() - 1;
    let sd2 = sd; // XXX: copy `sd` to avoid borrow error
    let tid = thread::current().id();
    // XXX: for tests, just set on first call
    if !_STACK_OFFSET_TABLE.is_set().unwrap() {
        #[allow(clippy::single_match)]
        match _STACK_OFFSET_TABLE.set(Map_ThreadId_SD::new()) {
            Err(err) => {
                eprintln!("ERROR: stack_offset: _STACK_OFFSET_TABLE.set failed {:?}", err);
            },
            _ => {},
        }
    }
    let so_table = _STACK_OFFSET_TABLE.read().unwrap();
    let so: &usize = so_table.get(&tid).unwrap_or(&sd2);
    if &sd < so {
        return 0;
    }
    sd -= so;
    sd
}

/// set once in each thread near the beginning of the thread.
/// a positive value `correction` will move the printed output to the right.
/// if the `correction` is too negative then it will print to the left-most column
/// of the terminal. Negative values are useful for when most of a program runs in
/// a function that is several calls deep.
/// Passing `None` will set `correction` to value `0`.
///
/// For example, the `main` function might
/// call an `intialize` function which might call a `run` function. The `run` function
/// might do the majority of work (and debug printing). In that case, from `main`,
/// pass a negative offset of 4 to `stack_offset_set`, i.e. `stack_offset_set(Some(-4))`
/// This way, debug printing from function `run` will start at the left-most column (and not
/// be indented to the right). This may improve readability.
pub fn stack_offset_set(correction: Option<isize>) {
    if ! (cfg!(debug_assertions) || cfg!(test)) {
        return;
    }
    let sd_ = stack_depth();
    let sdi: isize = (sd_ as isize) - correction.unwrap_or(0);
    let so = std::cmp::max(sdi, 0) as usize;
    let thread_cur = thread::current();
    let tid = thread_cur.id();
    if !_STACK_OFFSET_TABLE.is_set().unwrap() {
        // BUG: multiple simlutaneous calls to `_STACK_OFFSET_TABLE.is_set()` then
        //      `_STACK_OFFSET_TABLE.set(…)` may cause `.set(…)` to return an error.
        //      Seen in some calls to `cargo test` with filtering where many tests call
        //      `stack_offset_set`. Needs a mutex.
        #[allow(clippy::single_match)]
        match _STACK_OFFSET_TABLE.set(Map_ThreadId_SD::new()) {
            Err(err) => {
                eprintln!("ERROR: stack_offset_set: _STACK_OFFSET_TABLE.set failed {:?}", err);
            },
            _ => {},
        }
    }
    if _STACK_OFFSET_TABLE.read().unwrap().contains_key(&tid) {
        //eprintln!("WARNING: _STACK_OFFSET_TABLE has already been set for this thread {:?}; stack_offset_set() will be ignored", tid);
        return;
    }
    _STACK_OFFSET_TABLE.write().unwrap().insert(tid, so);
    #[cfg(debug_assertions)]
    eprintln!("stack_offset_set({:?}): {:?}({}) stack_offset set to {}, stack_depth {}", correction, tid, thread_cur.name().unwrap_or(""), so, sd_);
}

const S_0: &str = "";
const S_1: &str = "    ";
const S_2: &str = "        ";
const S_3: &str = "            ";
const S_4: &str = "                ";
const S_5: &str = "                    ";
const S_6: &str = "                        ";
const S_7: &str = "                            ";
const S_8: &str = "                                ";
const S_9: &str = "                                    ";
const S_10: &str = "                                        ";
const S_11: &str = "                                            ";
const S_12: &str = "                                                ";
const S_13: &str = "                                                    ";
const S_14: &str = "                                                        ";
const S_15: &str = "                                                            ";
const S_16: &str = "                                                                ";
const S_17: &str = "                                                                    ";
const S_18: &str = "                                                                        ";
const S_19: &str = "                                                                            ";
const S__: &str = "                                                                                ";

/// return a string of spaces as long as `stack_offset()`
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn so() -> &'static str {
    const LEAD: &str = " ";
    let so_ = stack_offset();
    match so_ {
        0 => concatcp!(S_0, LEAD),
        1 => concatcp!(S_1, LEAD),
        2 => concatcp!(S_2, LEAD),
        3 => concatcp!(S_3, LEAD),
        4 => concatcp!(S_4, LEAD),
        5 => concatcp!(S_5, LEAD),
        6 => concatcp!(S_6, LEAD),
        7 => concatcp!(S_7, LEAD),
        8 => concatcp!(S_8, LEAD),
        9 => concatcp!(S_9, LEAD),
        10 => concatcp!(S_10, LEAD),
        11 => concatcp!(S_11, LEAD),
        12 => concatcp!(S_12, LEAD),
        13 => concatcp!(S_13, LEAD),
        14 => concatcp!(S_14, LEAD),
        15 => concatcp!(S_15, LEAD),
        16 => concatcp!(S_16, LEAD),
        17 => concatcp!(S_17, LEAD),
        18 => concatcp!(S_18, LEAD),
        19 => concatcp!(S_19, LEAD),
        _ => concatcp!(S__, LEAD),
    }
}

/// return a string of spaces as long as `stack_offset()` when e*n*tering a function.
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn sn() -> &'static str {
    const LEAD: &str = "→";
    let so_ = stack_offset();
    match so_ {
        0 => concatcp!(S_0, LEAD),
        1 => concatcp!(S_1, LEAD),
        2 => concatcp!(S_2, LEAD),
        3 => concatcp!(S_3, LEAD),
        4 => concatcp!(S_4, LEAD),
        5 => concatcp!(S_5, LEAD),
        6 => concatcp!(S_6, LEAD),
        7 => concatcp!(S_7, LEAD),
        8 => concatcp!(S_8, LEAD),
        9 => concatcp!(S_9, LEAD),
        10 => concatcp!(S_10, LEAD),
        11 => concatcp!(S_11, LEAD),
        12 => concatcp!(S_12, LEAD),
        13 => concatcp!(S_13, LEAD),
        14 => concatcp!(S_14, LEAD),
        15 => concatcp!(S_15, LEAD),
        16 => concatcp!(S_16, LEAD),
        17 => concatcp!(S_17, LEAD),
        18 => concatcp!(S_18, LEAD),
        19 => concatcp!(S_19, LEAD),
        _ => concatcp!(S__, LEAD),
    }
}

/// return a string of spaces as long as `stack_offset()` when e*x*iting a function.
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn sx() -> &'static str {
    const LEAD: &str = "←";
    let so_ = stack_offset();
    match so_ {
        0 => concatcp!(S_0, LEAD),
        1 => concatcp!(S_1, LEAD),
        2 => concatcp!(S_2, LEAD),
        3 => concatcp!(S_3, LEAD),
        4 => concatcp!(S_4, LEAD),
        5 => concatcp!(S_5, LEAD),
        6 => concatcp!(S_6, LEAD),
        7 => concatcp!(S_7, LEAD),
        8 => concatcp!(S_8, LEAD),
        9 => concatcp!(S_9, LEAD),
        10 => concatcp!(S_10, LEAD),
        11 => concatcp!(S_11, LEAD),
        12 => concatcp!(S_12, LEAD),
        13 => concatcp!(S_13, LEAD),
        14 => concatcp!(S_14, LEAD),
        15 => concatcp!(S_15, LEAD),
        16 => concatcp!(S_16, LEAD),
        17 => concatcp!(S_17, LEAD),
        18 => concatcp!(S_18, LEAD),
        19 => concatcp!(S_19, LEAD),
        _ => concatcp!(S__, LEAD),
    }
}

/// return a string of spaces as long as `stack_offset()` when e*n*tering and e*x*iting a function.
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn snx() -> &'static str {
    const LEAD: &str = "↔";
    let so_ = stack_offset();
    match so_ {
        0 => concatcp!(S_0, LEAD),
        1 => concatcp!(S_1, LEAD),
        2 => concatcp!(S_2, LEAD),
        3 => concatcp!(S_3, LEAD),
        4 => concatcp!(S_4, LEAD),
        5 => concatcp!(S_5, LEAD),
        6 => concatcp!(S_6, LEAD),
        7 => concatcp!(S_7, LEAD),
        8 => concatcp!(S_8, LEAD),
        9 => concatcp!(S_9, LEAD),
        10 => concatcp!(S_10, LEAD),
        11 => concatcp!(S_11, LEAD),
        12 => concatcp!(S_12, LEAD),
        13 => concatcp!(S_13, LEAD),
        14 => concatcp!(S_14, LEAD),
        15 => concatcp!(S_15, LEAD),
        16 => concatcp!(S_16, LEAD),
        17 => concatcp!(S_17, LEAD),
        18 => concatcp!(S_18, LEAD),
        19 => concatcp!(S_19, LEAD),
        _ => concatcp!(S__, LEAD),
    }
}
