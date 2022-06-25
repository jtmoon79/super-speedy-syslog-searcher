// printer_debug/stack.rs
//
// functions to find current stack depth for indented trace prints
//

use std::collections::HashMap;
use std::thread;

extern crate debug_print;
use debug_print::debug_eprintln;

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
    debug_eprintln!("stack_offset_set({:?}): {:?}({}) stack_offset set to {}, stack_depth {}", correction, tid, thread_cur.name().unwrap_or(""), so, sd_);
}

/// return a string of spaces as long as `stack_offset()`
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn so() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => " ",
        1 => "     ",
        2 => "         ",
        3 => "             ",
        4 => "                 ",
        5 => "                     ",
        6 => "                         ",
        7 => "                             ",
        8 => "                                 ",
        9 => "                                     ",
        10 => "                                        ",
        11 => "                                            ",
        12 => "                                                ",
        13 => "                                                    ",
        14 => "                                                        ",
        15 => "                                                            ",
        16 => "                                                                ",
        17 => "                                                                    ",
        18 => "                                                                        ",
        19 => "                                                                            ",
        _ => "                                                                                ",
    }
}

/// return a string of spaces as long as `stack_offset()` when e*n*tering a function.
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn sn() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "→",
        1 => "    →",
        2 => "        →",
        3 => "            →",
        4 => "                →",
        5 => "                    →",
        6 => "                        →",
        7 => "                            →",
        8 => "                                →",
        9 => "                                    →",
        10 => "                                       →",
        11 => "                                           →",
        12 => "                                               →",
        13 => "                                                   →",
        14 => "                                                       →",
        15 => "                                                           →",
        16 => "                                                               →",
        17 => "                                                                   →",
        18 => "                                                                       →",
        19 => "                                                                           →",
        _ => "                                                                               →",
    }
}

/// return a string of spaces as long as `stack_offset()` when e*x*iting a function.
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn sx() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "←",
        1 => "    ←",
        2 => "        ←",
        3 => "            ←",
        4 => "                ←",
        5 => "                    ←",
        6 => "                        ←",
        7 => "                            ←",
        8 => "                                ←",
        9 => "                                    ←",
        10 => "                                        ←",
        11 => "                                            ←",
        12 => "                                                ←",
        13 => "                                                    ←",
        14 => "                                                        ←",
        15 => "                                                            ←",
        16 => "                                                                ←",
        17 => "                                                                    ←",
        18 => "                                                                        ←",
        19 => "                                                                            ←",
        _ => "                                                                                ←",
    }
}

/// return a string of spaces as long as `stack_offset()` when e*n*tering and e*x*iting a function.
///
/// for use in `print` calls
///
/// only intended for debug builds
#[allow(dead_code)]
pub fn snx() -> &'static str {
    let so_ = stack_offset();
    match so_ {
        0 => "↔",
        1 => "    ↔",
        2 => "        ↔",
        3 => "            ↔",
        4 => "                ↔",
        5 => "                    ↔",
        6 => "                        ↔",
        7 => "                            ↔",
        8 => "                                ↔",
        9 => "                                    ↔",
        10 => "                                        ↔",
        11 => "                                            ↔",
        12 => "                                                ↔",
        13 => "                                                    ↔",
        14 => "                                                        ↔",
        15 => "                                                            ↔",
        16 => "                                                                ↔",
        17 => "                                                                    ↔",
        18 => "                                                                        ↔",
        19 => "                                                                            ↔",
        _ => "                                                                                ↔",
    }
}

// TODO: [2021/09/22]
//       create new macro for current function name `fname`
//       macro function_name!() prints all parents `A::B::my_func`, just print `my_func`.
//       can be ripped from https://github.com/popzxc/stdext-rs/blob/2179f94475f925a2eacdc2f2408d7ab352d0052c/src/macros.rs#L44-L74
//       could possibly use `backtrace::trace` and return this as part of `so`, `sn`, `sx` ???
/*
fn fno() -> () {
    let bt = backtrace::Backtrace::new();
    let frames = bt.frames();
    dbg!(frames);
    for f in frames.iter() {
        dbg!(f);
        debug_eprintln!("\n");
        for s in f.symbols() {
            dbg!(s);
        }
        debug_eprintln!("\n\n\n");
    }
    frames[1].symbols()[0];
    debug_eprintln!("\n\n\n");
    panic!();
}
*/
