// src/printer/printers.rs

//! Specialized printer struct [`PrinterLogMessage`] and helper functions
//! for printing log messages.
//!
//! Byte-oriented printing (no reference to `char` or `str`).
//!
//! [`PrinterLogMessage`]: self::PrinterLogMessage

use std::hint::black_box;
use std::io::{
    Error,
    ErrorKind,
    Result,
    StdoutLock,
    Write, // for `std::io::Stdout.flush`
};
use std::sync::RwLock;

use ::bstr::ByteSlice;
#[allow(unused_imports)]
use ::more_asserts::{
    debug_assert_le,
    debug_assert_lt,
};
use ::lazy_static::lazy_static;
use ::si_trace_print::printers::debug_print_guard;
#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
};
#[doc(hidden)]
pub use ::termcolor::{
    Color,
    ColorChoice,
    ColorSpec,
    WriteColor,
};

use crate::common::{
    debug_panic,
    FPath,
    NLu8,
    SUBPATH_SEP,
    SUBPATH_SEP_DISPLAY_STR,
};
use crate::data::datetime::{
    DateTimeL,
    DateTimePattern_string,
    FixedOffset,
};
use crate::data::evtx::Evtx;
use crate::data::fixedstruct::{
    FixedStruct,
    InfoAsBytes,
};
use crate::data::journal::JournalEntry;
use crate::data::line::{
    LineIndex,
    LineP,
};
use crate::data::sysline::SyslineP;
use crate::debug::printers::de_err;
use crate::readers::helpers::basename;

// ---------------------
// globals and constants

const COLOR_THEME_COUNT: usize = 2;

/// `Color` for printing prepended data like datetime, file name, etc.
/// Dark Theme, Light Theme
pub const COLOR_DEFAULT: [Color; COLOR_THEME_COUNT] = [
    Color::White,
    Color::Black,
];

/// `Color` for printing prepended data like datetime, file name, etc.
/// Dark Theme, Light Theme
pub const COLOR_DIMMED: [Color; COLOR_THEME_COUNT] = [
    Color::Rgb(0xC0, 0xC0, 0xC0),
    Color::Rgb(0xC0, 0xC0, 0xC0),
];

/// `Color` for printing some user-facing error messages.
pub const COLOR_ERROR: Color = Color::Red;

// XXX: relates to Issue #16
const CHARSZ: usize = 1;

/// Default format string is `CLI_OPT_PREPEND_FMT` in `s4.rs` value
/// `%Y%m%dT%H%M%S%.3f%z`.
/// Example output `20210509T124318.616-0700` (24 chars).
/// Round up to 40 in case user passes long format string.
///
/// XXX: this should loosely follow changes to `CLI_OPT_PREPEND_FMT`
const CLI_OPT_PREPEND_FMT_CHARLEN: usize = 40;

const COLORS_TEXT_LEN: usize = 28;

/// A preselection of `Color`s for log messages.
/// Dark Theme: bright colors chosen for a dark background console.
///
/// A decent reference for RGB colors is
/// <https://www.rapidtables.com/web/color/RGB_Color.html>.
//
// TODO: It is presumptious to assume a dark background console. Would be good
//       to react to the console (is it light or dark?) and adjust at run-time.
//       Not sure if that is possible.
//       Issue #261
pub const COLORS_TEXT_DT: [Color; COLORS_TEXT_LEN] = [
    // XXX: colors with low pixel values are difficult to see on dark console
    //      backgrounds recommend at least one pixel value of 102 or greater
    // 102 + 230
    Color::Rgb(102, 102, 230),
    Color::Rgb(102, 230, 102),
    Color::Rgb(102, 230, 230),
    // 102 + 255
    Color::Rgb(102, 102, 255),
    Color::Rgb(102, 255, 102),
    Color::Rgb(102, 255, 255),
    // 127
    Color::Rgb(127, 127, 127),
    // 127 + 230
    Color::Rgb(127, 230, 127),
    Color::Rgb(127, 127, 230),
    Color::Rgb(127, 230, 230),
    // 127 + 255
    Color::Rgb(127, 255, 127),
    Color::Rgb(127, 127, 255),
    Color::Rgb(127, 255, 255),
    // 153
    Color::Rgb(153, 153, 153),
    // 153 + 255
    Color::Rgb(153, 153, 255),
    Color::Rgb(153, 255, 153),
    Color::Rgb(153, 255, 255),
    // 230 + 127
    Color::Rgb(230, 127, 127),
    Color::Rgb(230, 230, 127),
    Color::Rgb(230, 127, 230),
    // 230 + 153
    Color::Rgb(230, 153, 153),
    Color::Rgb(230, 230, 153),
    Color::Rgb(230, 153, 230),
    // 230
    Color::Rgb(230, 230, 230),
    // 230 + 255
    Color::Rgb(230, 255, 255),
    Color::Rgb(230, 230, 255),
    Color::Rgb(230, 255, 230),
    // 255
    Color::Rgb(255, 255, 255),
];

/// A preselection of `Color`s for printing log messages.
/// Light Theme: dimmer colors chosen for a light background console.
pub const COLORS_TEXT_LT: [Color; COLORS_TEXT_LEN] = [
    // 102 + 25
    Color::Rgb(102, 102, 25),
    Color::Rgb(102, 25, 102),
    Color::Rgb(102, 25, 25),
    // 102 + 0
    Color::Rgb(102, 102, 0),
    Color::Rgb(102, 0, 102),
    Color::Rgb(102, 0, 0),
    // 127
    Color::Rgb(127, 127, 127),
    // 127 + 25
    Color::Rgb(127, 25, 127),
    Color::Rgb(127, 127, 25),
    Color::Rgb(127, 25, 25),
    // 127 + 0
    Color::Rgb(127, 0, 127),
    Color::Rgb(127, 127, 0),
    Color::Rgb(127, 0, 0),
    // 102
    Color::Rgb(102, 102, 102),
    // 102 + 0
    Color::Rgb(102, 102, 0),
    Color::Rgb(102, 0, 102),
    Color::Rgb(102, 0, 0),
    // 25 + 127
    Color::Rgb(25, 127, 127),
    Color::Rgb(25, 25, 127),
    Color::Rgb(25, 127, 25),
    // 25 + 102
    Color::Rgb(25, 102, 102),
    Color::Rgb(25, 25, 102),
    Color::Rgb(25, 102, 25),
    // 25
    Color::Rgb(25, 25, 25),
    // 25 + 0
    Color::Rgb(25, 0, 0),
    Color::Rgb(25, 25, 0),
    Color::Rgb(25, 0, 25),
    // 0
    Color::Rgb(0, 0, 0),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorTheme {
    Dark = 0,
    Light,
}

pub const COLOR_THEME_DEFAULT: ColorTheme = ColorTheme::Dark;

// ----------------
// helper functions

lazy_static! {
    /// Global setting for the color theme.
    pub static ref ColorThemeGlobal: RwLock<ColorTheme> = {
        defñ!("lazy_static! ColorThemeGlobal");

        RwLock::new(COLOR_THEME_DEFAULT)
    };
}

/// "Cached" indexing value for `color_rand`.
///
/// XXX: not thread-aware, not recommended
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static mut _color_at: usize = 0;

/// "Cached" setting for `color_rand`.
///
/// XXX: not thread-aware, not recommended
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static mut _color_theme: Option<ColorTheme> = None;

/// Return a random color from either `COLORS_TEXT_*T`.
///
/// unsafe; not thread-aware; not recommended.
#[allow(static_mut_refs)]
pub fn color_rand() -> Color {
    let ci: usize;
    let colors_text: &[Color; COLORS_TEXT_LEN];

    unsafe {
        if _color_theme.is_none() {
            _color_theme = Some(*ColorThemeGlobal.read().unwrap());
        }
        colors_text = match _color_theme.unwrap() {
            ColorTheme::Dark => &COLORS_TEXT_DT,
            ColorTheme::Light => &COLORS_TEXT_LT,
        };

        _color_at += 1;
        if _color_at == colors_text.len() {
            _color_at = 0;
        }
        ci = _color_at;
    }

    colors_text[ci]
}

/// unsafe; not thread-aware; not recommended.
#[allow(static_mut_refs)]
fn color_theme_to_index() -> usize {
    let index: usize;
    unsafe {
        if _color_theme.is_none() {
            _color_theme = Some(*ColorThemeGlobal.read().unwrap());
        }
        match _color_theme.unwrap() {
            ColorTheme::Dark => index = 0,
            ColorTheme::Light => index = 1,
        }
    }

    index
}

pub fn color_dimmed() -> Color {
    let color_theme_index: usize = color_theme_to_index();

    COLOR_DIMMED[color_theme_index]
}

pub fn color_default() -> Color {
    let color_theme_index: usize = color_theme_to_index();

    COLOR_DEFAULT[color_theme_index]
}

/// helper to create string for CLI option `--prepend-filename`
pub fn fpath_to_prependname(path: &FPath) -> String {
    // `_` will be archive path
    // `b` will be path within archive or just the file path
    let path_ = match path.split_once(SUBPATH_SEP) {
        Some((_, b_)) => &FPath::from(b_),
        None => path,
    };

    basename(path_)
}

/// helper to create string for CLI option `--prepend-filepath`
pub fn fpath_to_prependpath(path: &FPath) -> FPath {
    path.replacen(
        SUBPATH_SEP,
        SUBPATH_SEP_DISPLAY_STR,
        1,
    )
}

//pub fn fpath_split_only_path(path: &FPath) -> FPath {
//    match path.rsplit_once(SUBPATH_SEP) {
//        Some((a, _)) => FPath::from(a),
//        None => path.clone(),
//    }
//}

// -----------------
// PrinterLogMessage

/// A printer for `s4lib` log messages that writes to Standard Out.
///
/// It can add text affects depending on the value of `color_choice`
/// and `color_logmessage` corresponding to end-user CLI options chosen.
/// The various `prepend_*` values also correspond to end-user CLI options
/// chosen.
///
/// It aims to be fast and efficient.
pub struct PrinterLogMessage {
    /// handle to stdout
    stdout: std::io::Stdout,
    /// termcolor handle to stdout
    stdout_color: termcolor::StandardStream,
    /// should printing be in color?
    do_color: bool,
    /// termcolor::ColorChoice
    _color_choice: ColorChoice,
    /// color settings for plain text (not sysline)
    color_spec_default: ColorSpec,
    /// color settings for sysline text
    // TODO: [2023/03/22] rename from `color_spec_sysline` to `color_spec_text`
    color_spec_sysline: ColorSpec,
    /// color settings for sysline dateline text
    color_spec_datetime: ColorSpec,
    /// should a file name or path be printed before each line?
    do_prepend_file: bool,
    /// the file name or path string.
    /// width spacing (CLI option --prepend-file-align) should already be
    /// embedded by the caller
    prepend_file: Option<String>,
    /// should a date be printed before each line?
    do_prepend_date: bool,
    /// format string for printed date
    prepend_date_format: DateTimePattern_string,
    /// timezone offset of printed date
    prepend_date_offset: FixedOffset,
    /// last value passed to `self.stdout_color.set_color()`
    ///
    /// used by macro `setcolor_or_return`
    color_spec_last: ColorSpec,
    /// buffer for writes to stdout
    buffer: Vec<u8>,
}

/// Aliased `Result` returned by various [`PrinterLogMessage`] functions.
pub type PrinterLogMessageResult = Result<(usize, usize)>;

const BUFFER_USE: bool = true;
const BUFFER_CAP: usize = 2056;

/// Flushes. Sets `error_ret` if there is an error.
macro_rules! buffer_flush_or_seterr {
    ($stdout:expr, $buffer:expr, $printed:expr, $flushed:expr, $error_ret:expr) => {{
        if !$buffer.is_empty() {
            match $stdout.write_all($buffer.as_slice()) {
                Ok(_) => {
                    $printed += $buffer.len();
                    $buffer.clear();
                    match $stdout.flush() {
                        Ok(_) => {}
                        Err(err) => {
                            $error_ret = Some(err);
                        }
                    }
                    $flushed += 1;
                }
                Err(err) => {
                    // XXX: this will print when this program stdout is truncated, like when piping
                    //      to `head`, e.g. `s4 file.log | head`
                    //          Broken pipe (os error 32)
                    de_err!(
                        "{}.write({}@{:p}) (len {})) error {}",
                        stringify!($stdout),
                        stringify!($buffer),
                        &$buffer,
                        $buffer.len(),
                        err
                    );
                    $error_ret = Some(err);
                    _ = $stdout.flush();
                    $flushed += 1;
                }
            }
        }
    }};
}

/// Flushes `PrinterLogMessage.buffer`. Returns upon Err.
macro_rules! buffer_flush_or_return {
    ($stdout:expr, $buffer:expr, $printed:expr, $flushed:expr) => {{
        let mut error_ret: Option<Error> = None;
        buffer_flush_or_seterr!($stdout, $buffer, $printed, $flushed, error_ret);
        match error_ret {
            Some(err) => return PrinterLogMessageResult::Err(err),
            None => {}
        }
    }};
}

/// Flushes `PrinterLogMessage.buffer`. No returns. No statistics updates.
/// Ignores errors. Only for some error-in-progress cases.
macro_rules! buffer_flush_nostats {
    ($stdout:expr, $buffer:expr) => {{
        let mut _error_ret: Option<Error> = None;
        let mut _flushed: usize = 0;
        let mut _printed: usize = 0;
        buffer_flush_or_seterr!($stdout, $buffer, _printed, _flushed, _error_ret);
    }};
}

/// Macro to write `$slice_` to `PrinterLogMessage.buffer`.
/// If there is an error then `return PrinterLogMessageResult::Err`.
/// May or may not flush `PrinterLogMessage.buffer`.
macro_rules! buffer_write_or_return {
    ($stdout:expr, $buffer:expr, $slice_:expr, $printed:expr, $flushed:expr) => {{
        let mut error_ret: Option<Error> = None;
        if !BUFFER_USE {
            match $stdout.write_all($slice_) {
                Ok(_) => {
                    $printed += $slice_.len();
                }
                Err(err) => {
                    de_err!(
                        "{}.write({}@{:p}) (len {})) error {}",
                        stringify!($stdout),
                        stringify!($slice_),
                        &$slice_,
                        $slice_.len(),
                        err
                    );
                    error_ret = Some(err);
                }
            }
            if let Err(err) = $stdout.flush() {
                if let None = error_ret {
                    error_ret = Some(err);
                }
            }
            $flushed += 1;
            match error_ret {
                Some(err) => return PrinterLogMessageResult::Err(err),
                None => {}
            }
        } else {
            let len: usize = $buffer.len();
            let slice_len: usize = $slice_.len();
            let cap: usize = $buffer.capacity();
            let remain: usize = cap - len;
            if slice_len <= remain {
                // remaining capacity in the buffer; only copy the slice
                $buffer.extend_from_slice($slice_);
            } else {
                // buffer is full, write it
                match $stdout.write_all($buffer.as_slice()) {
                    Ok(_) => {
                        $printed += $buffer.len();
                        $flushed += 1;
                        $buffer.clear();
                    }
                    Err(err) => {
                        // XXX: this will print when this program stdout is truncated, like when piping
                        //      to `head`, e.g. `s4 file.log | head`
                        //          Broken pipe (os error 32)
                        de_err!(
                            "{}.write({}@{:p}) (len {})) error {}",
                            stringify!($stdout),
                            stringify!($buffer),
                            &$buffer,
                            $buffer.len(),
                            err
                        );
                        $buffer.clear();
                        match $stdout.flush() {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                        $flushed += 1;
                        error_ret = Some(err);
                    }
                }
                match error_ret {
                    Some(_) => {
                        // there was an error; skip another chance to write
                        // but copy the slice even though it'll very likely
                        // never get written (because a printing error *should*
                        // cause the printing thread to return early)
                        $buffer.extend_from_slice($slice_);
                    }
                    None => {
                        // no error occurred from `write_all`.
                        // if slice is larger than buffer capacity then write the slice
                        // else copy the slice to buffer
                        if $slice_.len() > cap {
                            match $stdout.write_all($slice_) {
                                Ok(_) => {
                                    $printed += $slice_.len();
                                }
                                Err(err) => {
                                    de_err!(
                                        "{}.write({}@{:p}) (len {})) error {}",
                                        stringify!($stdout),
                                        stringify!($slice_),
                                        &$slice_,
                                        $slice_.len(),
                                        err
                                    );
                                    error_ret = Some(err);
                                }
                            }
                            if let Err(err) = $stdout.flush() {
                                if let None = error_ret {
                                    error_ret = Some(err);
                                }
                            }
                            $flushed += 1;
                        } else {
                            $buffer.extend_from_slice($slice_);
                        }
                    }
                }
                match error_ret {
                    Some(err) => return PrinterLogMessageResult::Err(err),
                    None => {}
                }
            }
        }
    }};
}

/// Macro that sets output color only if the color has changed since the last
/// call to this macro.
/// Flushes at end.
///
/// `$color_spec_last` tracks that last call to this macro. It helps avoid
/// unnecessary calls to `set_color`.
/// Unnecessary changes to `set_color` may cause errant formatting bytes to
/// print to the terminal. It is also a performance hit.
macro_rules! setcolor_or_return {
    ($stdout:expr, $buffer:expr, $color_spec:expr, $color_spec_last:expr, $printed:expr, $flushed:expr) => {{
        buffer_flush_or_return!($stdout, $buffer, $printed, $flushed);
        // set color if it has changed else skip the expensive call
        if $color_spec != $color_spec_last {
            // `set_color` eventually calls `write_all` which does not flush
            // (except on some platforms (Windows) termcolor may explicitly call flush)
            if let Err(err) = $stdout.set_color(&$color_spec) {
                de_err!("{}.set_color({:?}) returned error {}", stringify!($stdout), $color_spec, err);
                return PrinterLogMessageResult::Err(err);
            };
            // ... so call flush
            if let Result::Err(err) = $stdout.flush() {
                return PrinterLogMessageResult::Err(err);
            }
            $flushed += 1;
            $color_spec_last = $color_spec.clone();
        }
    }};
}

// XXX: this was a `fn -> PrinterLogMessageResult` but due to mutable and immutable error, it would not compile.
//      So a macro is a decent workaround.
/// Macro helper to print a single line in color. Uses `PrinterLogMessage.buffer`.
/// Flushes at end.
macro_rules! print_color_line {
    ($stdout_color:expr, $buffer:expr, $linep:expr, $printed:expr, $flushed:expr) => {{
        for linepart in (*$linep).lineparts.iter() {
            let slice: &[u8] = linepart.as_slice();
            buffer_write_or_return!($stdout_color, $buffer, slice, $printed, $flushed);
        }
        buffer_flush_or_return!($stdout_color, $buffer, $printed, $flushed);
    }};
}

// XXX: this macro was originally a `fn -> PrinterLogMessageResult` but due to mutable and immutable borrow
//      error, it would not compile. So this macro is a decent workaround.
//
/// Macro helper to print a single line in color and highlight the datetime
/// within the line. Uses `PrinterLogMessage.buffer`.
/// Flushes at end.
macro_rules! print_color_line_highlight_dt {
    ($self:expr, $buffer:expr, $linep:expr, $dt_beg:expr, $dt_end:expr, $printed:expr, $flushed:expr) => {{
        debug_assert_le!(
            $dt_beg,
            $dt_end,
            "passed bad datetime_beg {:?} datetime_end {:?}",
            $dt_beg,
            $dt_end
        );
        let mut at: LineIndex = 0;
        // this tedious indexing manual is faster than calling `line.get_boxptrs`
        // especially since `$dt_beg` `$dt_end` is a sub-slice(s) of the total `Line` slice(s)
        for linepart in (*$linep).lineparts.iter() {
            let slice: &[u8] = linepart.as_slice();
            debug_assert!(!slice.is_empty(), "linepart.as_slice() is empty!?");
            let at_end: usize = at + slice.len();
            // datetime is entirely within one linepart
            if at <= $dt_beg && $dt_end < at_end {
                debug_assert_le!(
                    ($dt_beg - at),
                    slice.len(),
                    "at {} dt_beg {} (dt_beg-at {} > {} slice.len()) dt_end {} A",
                    at,
                    $dt_beg,
                    $dt_beg - at,
                    slice.len(),
                    $dt_end
                );
                debug_assert_le!(
                    ($dt_end - at),
                    slice.len(),
                    "at {} dt_beg {} dt_end {} (dt_end-at {} > {} slice.len()) A",
                    at,
                    $dt_beg,
                    $dt_end,
                    $dt_end - at,
                    slice.len()
                );
                let slice_a = &slice[..($dt_beg - at)];
                let slice_b_dt = &slice[($dt_beg - at)..($dt_end - at)];
                let slice_c = &slice[($dt_end - at)..];
                // print line contents before the datetime
                if !slice_a.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_sysline, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_a, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
                // print line contents of the entire datetime
                if !slice_b_dt.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_datetime, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_b_dt, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
                // print line contents after the datetime
                if !slice_c.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_sysline, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_c, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
            // datetime begins in this linepart, extends into next linepart
            } else if at <= $dt_beg && $dt_beg < at_end && at_end <= $dt_end {
                debug_assert_le!(
                    ($dt_beg - at),
                    slice.len(),
                    "at {} dt_beg {} (dt_beg-at {} > {} slice.len()) dt_end {} at_end {} B",
                    at,
                    $dt_beg,
                    $dt_beg - at,
                    slice.len(),
                    $dt_end,
                    at_end
                );
                let slice_a = &slice[..($dt_beg - at)];
                let slice_b_dt = &slice[($dt_beg - at)..];
                // print line contents before the datetime
                if !slice_a.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_sysline, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_a, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
                // print line contents of the partial datetime
                if !slice_b_dt.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_datetime, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_b_dt, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
            // datetime began in previous linepart, extends into this linepart and ends within this linepart
            } else if $dt_beg < at && at <= $dt_end && $dt_end <= at_end {
                debug_assert_le!(
                    ($dt_end - at),
                    slice.len(),
                    "at {} dt_beg {} dt_end {} (dt_end-at {} > {} slice.len()) C",
                    at,
                    $dt_beg,
                    $dt_end,
                    $dt_end - at,
                    slice.len()
                );
                let slice_a_dt = &slice[..($dt_end - at)];
                let slice_b = &slice[($dt_end - at)..];
                // print line contents of the partial datetime
                if !slice_a_dt.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_datetime, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_a_dt, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
                // print line contents after the datetime
                if !slice_b.is_empty() {
                    setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_sysline, $self.color_spec_last, $printed, $flushed);
                    buffer_write_or_return!($self.stdout_color, $buffer, slice_b, $printed, $flushed);
                    buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
                }
            // datetime began in previous linepart, extends into next linepart
            } else if $dt_beg < at && at_end <= $dt_end {
                // print entire linepart which is the partial datetime
                setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_datetime, $self.color_spec_last, $printed, $flushed);
                buffer_write_or_return!($self.stdout_color, $buffer, slice, $printed, $flushed);
                buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
            // datetime is not in this linepart
            } else {
                // print entire linepart
                setcolor_or_return!($self.stdout_color, $buffer, $self.color_spec_sysline, $self.color_spec_last, $printed, $flushed);
                buffer_write_or_return!($self.stdout_color, $buffer, slice, $printed, $flushed);
                buffer_flush_or_return!($self.stdout_color, $buffer, $printed, $flushed);
            }
            at += slice.len() as LineIndex;
        }
    }};
}

impl PrinterLogMessage {
    /// Create a new `PrinterLogMessage`.
    pub fn new(
        color_choice: ColorChoice,
        color_logmessage: Color,
        prepend_file: Option<String>,
        prepend_date_format: Option<DateTimePattern_string>,
        prepend_date_offset: FixedOffset,
    ) -> PrinterLogMessage {
        defñ!(
            "color_choice {:?}, color_logmessage {:?}, prepend_file {:?}, prepend_date_format {:?}, prepend_date_offset {:?}",
            color_choice, color_logmessage, prepend_file, prepend_date_format, prepend_date_offset
        );
        // get a stdout handle once
        let stdout = std::io::stdout();
        let stdout_color = termcolor::StandardStream::stdout(color_choice);
        let do_color: bool = match color_choice {
            ColorChoice::Never => false,
            ColorChoice::Always | ColorChoice::AlwaysAnsi | ColorChoice::Auto => true,
        };
        let mut color_spec_default: ColorSpec = ColorSpec::new();
        color_spec_default.set_fg(Some(color_default()));
        let mut color_spec_sysline: ColorSpec = ColorSpec::new();
        color_spec_sysline.set_fg(Some(color_logmessage));
        let mut color_spec_datetime: ColorSpec = ColorSpec::new();
        color_spec_datetime.set_fg(Some(color_logmessage));
        color_spec_datetime.set_underline(true);
        // set `color_spec_last` to unset color so first print
        // forces a set (see `setcolor_or_return` macro)
        let color_spec_last = ColorSpec::new();
        debug_assert_ne!(color_spec_last, color_spec_default);
        let prepend_date_format_: DateTimePattern_string = prepend_date_format.unwrap_or_default();
        let do_prepend_date = !prepend_date_format_.is_empty();

        PrinterLogMessage {
            stdout,
            stdout_color,
            do_color,
            _color_choice: color_choice,
            color_spec_default,
            color_spec_sysline,
            color_spec_datetime,
            do_prepend_file: prepend_file.is_some(),
            prepend_file,
            do_prepend_date,
            prepend_date_format: prepend_date_format_,
            prepend_date_offset,
            color_spec_last,
            buffer: Vec::<u8>::with_capacity(if BUFFER_USE { BUFFER_CAP } else { 0 }),
        }
    }

    /// Print a `SyslineP` based on [`PrinterLogMessage`] settings.
    ///
    /// Users should call this function.
    #[inline(always)]
    pub fn print_sysline(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        // TODO: [2022/06/19] how to determine if "Auto" has become Always or Never?
        //       see https://docs.rs/termcolor/latest/termcolor/#detecting-presence-of-a-terminal
        // TODO: [2023/03/23] refactor `print_sysline*` similar to `print_evtx*`
        match (self.do_color, self.do_prepend_file, self.do_prepend_date) {
            (false, false, false) => self.print_sysline_(syslinep),
            (false, true, false) => self.print_sysline_prependfile(syslinep),
            (false, false, true) => self.print_sysline_prependdate(syslinep),
            (false, true, true) => self.print_sysline_prependfile_prependdate(syslinep),
            (true, false, false) => self.print_sysline_color(syslinep),
            (true, true, false) => self.print_sysline_prependfile_color(syslinep),
            (true, false, true) => self.print_sysline_prependdate_color(syslinep),
            (true, true, true) => self.print_sysline_prependfile_prependdate_color(syslinep),
        }
    }

    /// Print a `FixedStruct` based on [`PrinterLogMessage`] settings.
    ///
    /// Users should call this function.
    #[inline(always)]
    pub fn print_fixedstruct(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        // TODO: [2023/03/23] refactor `print_utmp*` similar to `print_evtx*`
        match (self.do_color, self.do_prepend_file, self.do_prepend_date) {
            (false, false, false) => self.print_fixedstruct_(fixedstruct, buffer),
            (false, true, false) => self.print_fixedstruct_prependfile(fixedstruct, buffer),
            (false, false, true) => self.print_fixedstruct_prependdate(fixedstruct, buffer),
            (false, true, true) => self.print_fixedstruct_prependfile_prependdate(fixedstruct, buffer),
            (true, false, false) => self.print_fixedstruct_color(fixedstruct, buffer),
            (true, true, false) => self.print_fixedstruct_prependfile_color(fixedstruct, buffer),
            (true, false, true) => self.print_fixedstruct_prependdate_color(fixedstruct, buffer),
            (true, true, true) => self.print_fixedstruct_prependfile_prependdate_color(fixedstruct, buffer),
        }
    }

    /// Print a `Evtx` based on [`PrinterLogMessage`] settings.
    ///
    /// Users should call this function.
    #[inline(always)]
    pub fn print_evtx(
        &mut self,
        evtx: &Evtx,
    ) -> PrinterLogMessageResult {
        match (self.do_color, self.do_prepend_file, self.do_prepend_date) {
            (false, false, false) => self.print_evtx_(evtx),
            (false, do_prepend_file, do_prepend_date) => {
                self.print_evtx_prepend(evtx, do_prepend_file, do_prepend_date)
            }
            (true, do_prepend_file, do_prepend_date) => {
                match (do_prepend_file, do_prepend_date) {
                    (false, false) => self.print_evtx_color(evtx),
                    (do_prepend_file, do_prepend_date) => self.print_evtx_prepend_color(evtx, do_prepend_file, do_prepend_date),
                }
            }
        }
    }

    /// Print a `JournalEntry` based on [`PrinterLogMessage`] settings.
    ///
    /// Users should call this function.
    #[inline(always)]
    pub fn print_journalentry(
        &mut self,
        journalentry: &JournalEntry,
    ) -> PrinterLogMessageResult {
        match (self.do_color, self.do_prepend_file, self.do_prepend_date) {
            (false, false, false) => self.print_journalentry_(journalentry),
            (false, do_prepend_file, do_prepend_date) => {
                self.print_journalentry_prepend(journalentry, do_prepend_file, do_prepend_date)
            }
            (true, do_prepend_file, do_prepend_date) => {
                match (do_prepend_file, do_prepend_date) {
                    (false, false) => self.print_journalentry_color(journalentry),
                    (do_prepend_file, do_prepend_date) => self.print_journalentry_prepend_color(journalentry, do_prepend_file, do_prepend_date),
                }
            }
        }
    }

    /// Helper function to transform [`sysline.dt`] to a `String`.
    ///
    /// [`sysline.dt`]: crate::data::sysline::Sysline#method.dt
    #[inline(always)]
    fn datetime_to_string_sysline(
        &self,
        syslinep: &SyslineP,
    ) -> String {
        // write the `syslinep.dt` into a `String` once
        //
        // XXX: would be cool if `chrono::DateTime` offered a format that returned
        //      `[u8; 100]` on the stack (where `100` is maximum possible length).
        //      That would be much faster than heap allocating a new `String`.
        //      instead, `format` returns a `DelayedFormat` object
        //      https://docs.rs/chrono/latest/chrono/format/struct.DelayedFormat.html
        //
        let dt_: DateTimeL = (*syslinep)
            .dt()
            .with_timezone(&self.prepend_date_offset);
        let dt_delayedformat = dt_.format(
            self.prepend_date_format.as_str(),
        );
        let mut dt_str = String::with_capacity(CLI_OPT_PREPEND_FMT_CHARLEN);
        match dt_delayedformat.write_to(&mut dt_str) {
            Ok(_) => {}
            Err(_err) => de_err!("{}", _err),
        }

        dt_str
    }

    /// Helper function to transform [`fixedstruct.dt`] to a `String`.
    ///
    /// [`fixedstruct.dt`]: crate::data::fixedstruct::fixedstruct#structfield.dt
    #[inline(always)]
    fn datetime_to_string_fixedstruct(
        &self,
        fixedstruct: &FixedStruct,
    ) -> String {
        // write the `fixedstruct.dt` into a `String` once
        let dt_: DateTimeL = (*fixedstruct)
            .dt()
            .with_timezone(&self.prepend_date_offset);
        let dt_delayedformat = dt_.format(
            self.prepend_date_format.as_str(),
        );

        dt_delayedformat.to_string()
    }

    /// Helper function to transform [`evtx.dt`] to a `String`.
    ///
    /// [`evtx.dt`]: crate::data::evtx::Evtx#structfield.dt
    #[inline(always)]
    fn datetime_to_string_evtx(
        &self,
        evtx: &Evtx,
    ) -> String {
        // write the `evtx.dt` into a `String` once
        let dt_: DateTimeL = evtx
            .dt()
            .with_timezone(&self.prepend_date_offset);
        let dt_delayedformat = dt_.format(
            self.prepend_date_format.as_str(),
        );

        dt_delayedformat.to_string()
    }

    /// Helper function to transform [`JournalEntry.dt`] to a `String`.
    ///
    /// [`JournalEntry.dt`]: crate::data::journal::JournalEntry#structfield.dt
    #[inline(always)]
    fn datetime_to_string_journalentry(
        &self,
        journalentry: &JournalEntry,
    ) -> String {
        // write the `journalentry.dt` into a `String` once
        let dt_: DateTimeL = journalentry
            .dt()
            .with_timezone(&self.prepend_date_offset);
        let dt_delayedformat = dt_.format(
            self.prepend_date_format.as_str(),
        );

        dt_delayedformat.to_string()
    }

    // TODO: make this a macro and it could be used in all functions
    /// Helper to `print_sysline_*` functions to print [`lineparts`].
    /// May or may not flush.
    ///
    /// [`lineparts`]: crate::data::line::LineParts
    #[inline(always)]
    fn print_line(
        &mut self,
        linep: &LineP,
        stdout_lock: &mut StdoutLock,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        for linepart in (*linep).lineparts.iter() {
            let slice: &[u8] = linepart.as_slice();
            buffer_write_or_return!(stdout_lock, self.buffer, slice, printed, flushed);
        }

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` without anything special.
    fn print_sysline_(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        // TODO: [2023/12/08] adding `stderr.lock()` after all `stdout.lock()`
        //       and before `_si_lock = debug_print_guard()` results in a deadlock.
        //           let _stderr_lock = self.stderr.lock();
        //       Why?
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            match self.print_line(linep, &mut stdout_lock) {
                PrinterLogMessageResult::Ok((p, f)) => {
                    printed += p;
                    flushed += f;
                }
                PrinterLogMessageResult::Err(err) => {
                    buffer_flush_nostats!(stdout_lock, self.buffer);
                    return PrinterLogMessageResult::Err(err);
                }
            }
        }
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` with prepended datetime.
    fn print_sysline_prependdate(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        debug_assert!(!self.prepend_date_format.is_empty());

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            buffer_write_or_return!(stdout_lock, self.buffer, dtb, printed, flushed);
            match self.print_line(linep, &mut stdout_lock) {
                PrinterLogMessageResult::Ok((p, f)) => {
                    printed += p;
                    flushed += f;
                }
                PrinterLogMessageResult::Err(err) => {
                    buffer_flush_nostats!(stdout_lock, self.buffer);
                    return PrinterLogMessageResult::Err(err);
                }
            }
        }
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` with prepended filename.
    fn print_sysline_prependfile(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            buffer_write_or_return!(stdout_lock, self.buffer, self.prepend_file.as_ref().unwrap().as_bytes(), printed, flushed);
            match self.print_line(linep, &mut stdout_lock) {
                PrinterLogMessageResult::Ok((p, f)) => {
                    printed += p;
                    flushed += f;
                }
                PrinterLogMessageResult::Err(err) => {
                    buffer_flush_nostats!(stdout_lock, self.buffer);
                    return PrinterLogMessageResult::Err(err);
                }
            }
        }
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` with prepended filename and datetime.
    fn print_sysline_prependfile_prependdate(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);
        debug_assert!(!self.prepend_date_format.is_empty());

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            buffer_write_or_return!(stdout_lock, self.buffer, self.prepend_file.as_ref().unwrap().as_bytes(), printed, flushed);
            buffer_write_or_return!(stdout_lock, self.buffer, dtb, printed, flushed);
            match self.print_line(linep, &mut stdout_lock) {
                PrinterLogMessageResult::Ok((p, f)) => {
                    printed += p;
                    flushed += f;
                }
                PrinterLogMessageResult::Err(err) => {
                    buffer_flush_nostats!(stdout_lock, self.buffer);
                    return PrinterLogMessageResult::Err(err);
                }
            }
        }
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` in color.
    fn print_sysline_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut line_first = true;
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        for linep in (*syslinep).lines.iter() {
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, self.buffer, linep, dt_beg, dt_end, printed, flushed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, self.buffer, linep, printed, flushed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` in color and prepended datetime.
    fn print_sysline_prependdate_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut line_first = true;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
            buffer_write_or_return!(self.stdout_color, self.buffer, dtb, printed, flushed);
            buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, self.buffer, linep, dt_beg, dt_end, printed, flushed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, self.buffer, linep, printed, flushed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Sysline` in color and prepended filename.
    fn print_sysline_prependfile_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut line_first = true;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
            buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
            buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, self.buffer, linep, dt_beg, dt_end, printed, flushed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, self.buffer, linep, printed, flushed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `SyslineP` in color and prepended filename and datetime.
    fn print_sysline_prependfile_prependdate_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut line_first = true;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
            buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
            buffer_write_or_return!(self.stdout_color, self.buffer, dtb, printed, flushed);
            buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, self.buffer, linep, dt_beg, dt_end, printed, flushed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, self.buffer, linep, printed, flushed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    // TODO: [2023/04/04] the series of function `print_fixedstruct_*`, `print_evtx_*`,
    //       and `print_journalentry_*` are nearly identical, and could be
    //       be turned into generic functions.

    /// Print a `FixedStruct` without anything special.
    fn print_fixedstruct_(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let at = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        buffer_write_or_return!(stdout_lock, self.buffer, &buffer[..at], printed, flushed);
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` with prepended datetime.
    fn print_fixedstruct_prependdate(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult
    {
        debug_assert!(!self.prepend_date_format.is_empty());

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let dt_string: String = self.datetime_to_string_fixedstruct(fixedstruct);
        let dtb: &[u8] = dt_string.as_bytes();
        let at = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };

        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        buffer_write_or_return!(stdout_lock, self.buffer, dtb, printed, flushed);
        buffer_write_or_return!(stdout_lock, self.buffer, &buffer[..at], printed, flushed);
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` with prepended filename.
    fn print_fixedstruct_prependfile(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let at = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        buffer_write_or_return!(stdout_lock, self.buffer, prepend_file, printed, flushed);
        buffer_write_or_return!(stdout_lock, self.buffer, &buffer[..at], printed, flushed);
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` with prepended filename and datetime.
    fn print_fixedstruct_prependfile_prependdate(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);
        debug_assert!(!self.prepend_date_format.is_empty());

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let dt_string: String = self.datetime_to_string_fixedstruct(fixedstruct);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let at = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };

        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        buffer_write_or_return!(stdout_lock, self.buffer, dtb, printed, flushed);
        buffer_write_or_return!(stdout_lock, self.buffer, prepend_file, printed, flushed);
        buffer_write_or_return!(stdout_lock, self.buffer, &buffer[..at], printed, flushed);
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` in color.
    fn print_fixedstruct_color(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let (at, beg, end) = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, beg, end) => (at, beg, end),
            InfoAsBytes::Fail(at) => {
                let err = Error::new(
                    ErrorKind::Other,
                    format!("buffer of len {} given too little data {}", buffer.len(), at),
                );
                return PrinterLogMessageResult::Err(err);
            }
        };
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[..beg], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[beg..end], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[end..at], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);

        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` in color and prepended datetime.
    fn print_fixedstruct_prependdate_color(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let dt_string: String = self.datetime_to_string_fixedstruct(fixedstruct);
        let dtb: &[u8] = dt_string.as_bytes();
        let (at, beg, end) = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, beg, end) => (at, beg, end),
            InfoAsBytes::Fail(at) => {
                let err = Error::new(
                    ErrorKind::Other,
                    format!("buffer of len {} given too little data {}", buffer.len(), at),
                );
                return PrinterLogMessageResult::Err(err);
            }
        };

        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, dtb, printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[..beg], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[beg..end], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[end..at], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);

        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` in color and prepended filename.
    fn print_fixedstruct_prependfile_color(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let (at, beg, end) = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, beg, end) => (at, beg, end),
            InfoAsBytes::Fail(at) => {
                let err = Error::new(
                    ErrorKind::Other,
                    format!("buffer of len {} given too little data {}", buffer.len(), at),
                );
                return PrinterLogMessageResult::Err(err);
            }
        };

        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[..beg], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[beg..end], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[end..at], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);

        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `FixedStruct` in color and prepended filename and datetime.
    fn print_fixedstruct_prependfile_prependdate_color(
        &mut self,
        fixedstruct: &FixedStruct,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let dt_string: String = self.datetime_to_string_fixedstruct(fixedstruct);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let (at, beg, end) = match fixedstruct.as_bytes(buffer) {
            InfoAsBytes::Ok(at, beg, end) => (at, beg, end),
            InfoAsBytes::Fail(at) => {
                let err = Error::new(
                    ErrorKind::Other,
                    format!("buffer of len {} given too little data {}", buffer.len(), at),
                );
                return PrinterLogMessageResult::Err(err);
            }
        };

        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, dtb, printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[..beg], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[beg..end], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &buffer[end..at], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Evtx` without anything special. Optimized for this simple
    /// common case.
    fn print_evtx_(
        &mut self,
        evtx: &Evtx,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        buffer_write_or_return!(stdout_lock, self.buffer, evtx.as_bytes(), printed, flushed);
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Evtx` with prepended file and/or datetime.
    fn print_evtx_prepend(
        &mut self,
        evtx: &Evtx,
        do_prependfile: bool,
        do_prependdate: bool,
    ) -> PrinterLogMessageResult {
        debug_assert!(!self.prepend_date_format.is_empty());

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let prepend_file: &[u8] = match do_prependfile {
            true => self
                    .prepend_file
                    .as_ref()
                    .unwrap()
                    .as_bytes(),
            false => &[],
        };
        let prepend_date_s: String;
        let prepend_date: &[u8] = match do_prependdate {
            true => {
                prepend_date_s = self.datetime_to_string_evtx(evtx);
                prepend_date_s.as_bytes()
            }
            false => &[],
        };
        let data = evtx.as_bytes();
        let mut a: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        while let Some(b) = data[a..].find_byte(NLu8) {
            let line = &data[a..a + b + CHARSZ];
            a += b + CHARSZ;
            if line.is_empty() {
                continue;
            }
            if do_prependfile {
                buffer_write_or_return!(stdout_lock, self.buffer, prepend_file, printed, flushed);
            }
            if do_prependdate {
                buffer_write_or_return!(stdout_lock, self.buffer, prepend_date, printed, flushed);
            }
            buffer_write_or_return!(stdout_lock, self.buffer, line, printed, flushed);
        }
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Evtx` in color. Optimized for this simple common case.
    fn print_evtx_color(
        &mut self,
        evtx: &Evtx,
    ) -> PrinterLogMessageResult {
        let (beg, end) = match evtx.dt_beg_end() {
            Some((beg, end)) => (*beg, *end),
            None => (0, 0),
        };
        debug_assert_le!(beg, end, "beg: {}, end: {}", beg, end);
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let data = evtx.as_bytes();
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &data[..beg], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &data[beg..end], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &data[end..], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `Evtx` in color and prepended filename and/or datetime.
    fn print_evtx_prepend_color(
        &mut self,
        evtx: &Evtx,
        do_prependfile: bool,
        do_prependdate: bool,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let prepend_file: &[u8] = match do_prependfile {
            true => self
                    .prepend_file
                    .as_ref()
                    .unwrap()
                    .as_bytes(),
            false => &[],
        };
        let prepend_date_s: String;
        let prepend_date: &[u8] = match do_prependdate {
            true => {
                prepend_date_s = self.datetime_to_string_evtx(evtx);
                prepend_date_s.as_bytes()
            }
            false => &[],
        };
        let (beg, end) = match evtx.dt_beg_end() {
            Some((beg, end)) => (*beg, *end),
            None => (0, 0),
        };
        debug_assert_le!(beg, end, "beg: {}, end: {}", beg, end);
        let data = evtx.as_bytes();
        let mut at: usize = 0;
        let mut a: usize = 0;
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        while let Some(b) = data[a..].find_byte(NLu8) {
            let line = &data[a..a + b + CHARSZ];
            a += b + CHARSZ;
            if line.is_empty() {
                continue;
            }
            let len = line.len();
            setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
            if do_prependfile {
                buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
            }
            if do_prependdate {
                buffer_write_or_return!(self.stdout_color, self.buffer, prepend_date, printed, flushed);
            }
            buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
            match (at <= beg, end < at + len) {
                (true, true) => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, &line[..beg - at], printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, &line[beg - at..end - at], printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, &line[end - at..], printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
                }
                _ => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, line, printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
                }
            }
            at += line.len();
        }
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `JournalEntry` without anything special. Optimized for this simple
    /// common case. May or may not flush.
    fn print_journalentry_(
        &mut self,
        journalentry: &JournalEntry,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        buffer_write_or_return!(stdout_lock, self.buffer, journalentry.as_bytes(), printed, flushed);
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `JournalEntry` with prepended file and/or datetime.
    fn print_journalentry_prepend(
        &mut self,
        journalentry: &JournalEntry,
        do_prependfile: bool,
        do_prependdate: bool,
    ) -> PrinterLogMessageResult {
        debug_assert!(!self.prepend_date_format.is_empty());

        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let prepend_file: &[u8] = match do_prependfile {
            true => self
                    .prepend_file
                    .as_ref()
                    .unwrap()
                    .as_bytes(),
            false => &[],
        };
        let prepend_date_s: String;
        let prepend_date: &[u8] = match do_prependdate {
            true => {
                prepend_date_s = self.datetime_to_string_journalentry(journalentry);
                prepend_date_s.as_bytes()
            }
            false => &[],
        };
        let data = journalentry.as_bytes();
        let mut a: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        while let Some(b) = data[a..].find_byte(NLu8) {
            let line = &data[a..a + b + CHARSZ];
            a += b + CHARSZ;
            if line.is_empty() {
                continue;
            }
            if do_prependfile {
                buffer_write_or_return!(stdout_lock, self.buffer, prepend_file, printed, flushed);
            }
            if do_prependdate {
                buffer_write_or_return!(stdout_lock, self.buffer, prepend_date, printed, flushed);
            }
            buffer_write_or_return!(stdout_lock, self.buffer, line, printed, flushed);
        }
        buffer_flush_or_return!(stdout_lock, self.buffer, printed, flushed);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `JournalEntry` in color. Optimized for this simple common case.
    fn print_journalentry_color(
        &mut self,
        journalentry: &JournalEntry,
    ) -> PrinterLogMessageResult {
        let (beg, end) = match journalentry.dt_beg_end() {
            Some((beg, end)) => (*beg, *end),
            None => (0, 0),
        };
        debug_assert_le!(beg, end, "beg: {}, end: {}", beg, end);
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let data = journalentry.as_bytes();
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &data[..beg], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &data[beg..end], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
        buffer_write_or_return!(self.stdout_color, self.buffer, &data[end..], printed, flushed);
        buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);

        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);

        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }

    /// Print a `JournalEntry` in color and prepended filename and/or datetime.
    fn print_journalentry_prepend_color(
        &mut self,
        journalentry: &JournalEntry,
        do_prependfile: bool,
        do_prependdate: bool,
    ) -> PrinterLogMessageResult {
        debug_assert!(
            do_prependfile || do_prependdate,
            "do_prependfile and do_prependdate are both false, expected at least one to be true"
        );
        let mut printed: usize = 0;
        let mut flushed: usize = 0;
        let prepend_file: &[u8] = match do_prependfile {
            true => self
                    .prepend_file
                    .as_ref()
                    .unwrap()
                    .as_bytes(),
            false => &[],
        };
        let prepend_date_s: String;
        let prepend_date: &[u8] = match do_prependdate {
            true => {
                prepend_date_s = self.datetime_to_string_journalentry(journalentry);
                prepend_date_s.as_bytes()
            }
            false => &[],
        };
        let (beg, end) = match journalentry.dt_beg_end() {
            Some((beg, end)) => (*beg, *end),
            None => (0, 0),
        };
        debug_assert_le!(beg, end, "beg: {}, end: {}", beg, end);
        let data = journalentry.as_bytes();
        let mut at: usize = 0;
        let mut a: usize = 0;
        let stdout_lock = self.stdout.lock();
        let _si_lock = debug_print_guard();
        while let Some(b) = data[a..].find_byte(NLu8) {
            let line = &data[a..a + b + CHARSZ];
            a += b + CHARSZ;
            if line.is_empty() {
                continue;
            }
            let len = line.len();
            match (do_prependfile, do_prependdate) {
                (true, true) => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, prepend_date, printed, flushed);
                }
                (true, false) => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, prepend_file, printed, flushed);
                }
                (false, true) => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, prepend_date, printed, flushed);
                }
                (false, false) => {
                    debug_panic!("do_prependfile and do_prependdate are both false, expected at least one to be true");
                }
            }
            buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
            match (at <= beg, end < at + len) {
                (true, true) => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, &line[..beg - at], printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_datetime, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, &line[beg - at..end - at], printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, &line[end - at..], printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
                }
                _ => {
                    setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_sysline, self.color_spec_last, printed, flushed);
                    buffer_write_or_return!(self.stdout_color, self.buffer, line, printed, flushed);
                    buffer_flush_or_return!(self.stdout_color, self.buffer, printed, flushed);
                }
            }
            at += line.len();
        }
        setcolor_or_return!(self.stdout_color, self.buffer, self.color_spec_default, self.color_spec_last, printed, flushed);

        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok((printed, flushed))
    }
}

// -----------------------------------------------------
// other printer functions, not part of "normal" file printing (no use of PrinterLogMessage)

/// Print colored output to terminal if possible using passed stream,
/// otherwise, print plain output.
///
/// Caller should take stream locks, e.g. `std::io::stdout().lock()`.
///
/// See an example <https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal>.
pub fn print_colored(
    color: Color,
    value: &[u8],
    out: &mut termcolor::StandardStream,
) -> std::io::Result<()> {
    match out.set_color(ColorSpec::new().set_fg(Some(color))) {
        Ok(_) => {}
        Err(err) => {
            de_err!("print_colored: std.set_color({:?}) returned error {}", color, err);
            return Err(err);
        }
    };
    match out.write(value) {
        Ok(_) => {}
        Err(err) => {
            de_err!("print_colored: out.write(…) returned error {}", err);
            return Err(err);
        }
    }
    match out.reset() {
        Ok(_) => {}
        Err(err) => {
            de_err!("print_colored: out.reset() returned error {}", err);
            return Err(err);
        }
    }
    out.flush()?;

    Ok(())
}

/// Print colored output to terminal on stdout.
///
/// See an example <https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal>.
#[doc(hidden)]
#[cfg(test)]
pub fn print_colored_stdout(
    color: Color,
    color_choice_opt: Option<ColorChoice>,
    value: &[u8],
) -> std::io::Result<()> {
    let choice: ColorChoice = match color_choice_opt {
        Some(choice_) => choice_,
        None => ColorChoice::Auto,
    };
    let mut stdout = termcolor::StandardStream::stdout(choice);
    let _stdout_lock = std::io::stdout().lock();
    let _stderr_lock = std::io::stderr().lock();
    let _si_lock = debug_print_guard();

    print_colored(color, value, &mut stdout)
}

/// Print colored output to terminal on stderr.
///
/// See an example <https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal>.
pub fn print_colored_stderr(
    color: Color,
    color_choice_opt: Option<ColorChoice>,
    value: &[u8],
) -> std::io::Result<()> {
    let choice: ColorChoice = match color_choice_opt {
        Some(choice_) => choice_,
        None => ColorChoice::Auto,
    };
    let mut stderr = termcolor::StandardStream::stderr(choice);
    let _stdout_lock = std::io::stdout().lock();
    //let _stderr_lock = std::io::stderr().lock();
    let _si_lock = debug_print_guard();

    print_colored(color, value, &mut stderr)
}

/// Safely write the `buffer` to stdout with help of [`StdoutLock`].
///
/// [`StdoutLock`]: std::io::StdoutLock
pub fn write_stdout(buffer: &[u8]) {
    // TODO: [2023/12/08] compare speed with and without these locks
    let mut stdout_lock = std::io::stdout().lock();
    //let mut stderr_lock = std::io::stderr().lock();
    let _si_lock = debug_print_guard();
    match stdout_lock.write_all(buffer) {
        Ok(_) => {}
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stdout_lock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), _err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {},
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stdout_lock.flush() error {}", _err);
        }
    }
    //_ = stderr_lock.flush();
}

/// Safely write the `buffer` to stderr with help of [`StderrLock`].
///
/// [`StderrLock`]: std::io::StderrLock
pub fn write_stderr(buffer: &[u8]) {
    //let mut stdout_lock = std::io::stdout().lock();
    let mut stderr_lock = std::io::stderr().lock();
    let _si_lock = debug_print_guard();
    // BUG: this print is shown during `cargo test` yet nearby `eprintln!` are not seen
    //      Would like this to only show when `--no-capture` is passed (this is how
    //      `eprintln!` behaves)
    match stderr_lock.write(buffer) {
        Ok(_) => {}
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stderr_lock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), _err);
        }
    }
    match stderr_lock.flush() {
        Ok(_) => {},
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stderr flushing error {}", _err);
        }
    }
    //_ = stdout_lock.flush();
}
