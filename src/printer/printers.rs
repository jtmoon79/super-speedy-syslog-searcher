// src/printer/printers.rs

//! Specialized printer struct [`PrinterLogMessage`] and helper functions
//! for printing [`Sysline`s] and [`Utmpx`s].
//!
//! Byte-oriented printing (no `char`s).
//!
//! [`PrinterSysline`]: self::PrinterSysline
//! [`Sysline`s]: crate::data::sysline::Sysline
//! [`Utmpx`s]: crate::data::utmpx::Utmpx

use crate::common::NLu8;
use crate::data::evtx::Evtx;
use crate::data::datetime::{DateTimeL, FixedOffset};
use crate::data::line::{LineIndex, LineP};
use crate::data::sysline::SyslineP;
use crate::data::utmpx::{InfoAsBytes, Utmpx};
use crate::debug::printers::de_err;

use std::hint::black_box;
use std::io::{
    Error,
    ErrorKind,
    Result,
    StdoutLock,
    Write, // for `std::io::Stdout.flush`
};

use ::bstr::ByteSlice;
#[doc(hidden)]
pub use ::termcolor::{Color, ColorChoice, ColorSpec, WriteColor};
#[allow(unused_imports)]
use ::more_asserts::{
    debug_assert_le,
    debug_assert_lt,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// globals and constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

//pub const COLOR_DATETIME: Color = Color::Green;

/// [`Color`] for printing prepended data like datetime, file name, etc.
///
/// [`Color`]: https://docs.rs/termcolor/1.1.3/termcolor/enum.Color.html
pub const COLOR_DEFAULT: Color = Color::White;

/// [`Color`] for printing some user-facing error messages.
///
/// [`Color`]: https://docs.rs/termcolor/1.1.3/termcolor/enum.Color.html
pub const COLOR_ERROR: Color = Color::Red;

const CHARSZ: usize = 1;

/// A preselection of [`Color`s] for printing syslines.
/// Chosen for a dark background console.
///
/// A decent reference for RGB colors is
/// <https://www.rapidtables.com/web/color/RGB_Color.html>.
///
/// [`Color`s]: https://docs.rs/termcolor/1.1.3/termcolor/enum.Color.html
//
// TODO: It is presumptious to assume a dark background console. Would be good
//       to react to the console (is it light or dark?) and adjust at run-time.
//       Not sure if that is possible.
pub const COLORS_TEXT: [Color; 29] = [
    Color::Yellow,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    // XXX: colors with low pixel values are difficult to see on dark console
    //      backgrounds recommend at least one pixel value of 102 or greater
    Color::Rgb(102, 0, 0),
    Color::Rgb(102, 102, 0),
    Color::Rgb(127, 0, 0),
    Color::Rgb(0, 0, 127),
    Color::Rgb(127, 0, 127),
    Color::Rgb(153, 76, 0),
    Color::Rgb(153, 153, 0),
    Color::Rgb(0, 153, 153),
    Color::Rgb(127, 127, 127),
    Color::Rgb(127, 153, 153),
    Color::Rgb(127, 153, 127),
    Color::Rgb(127, 127, 230),
    Color::Rgb(127, 230, 127),
    Color::Rgb(230, 127, 127),
    Color::Rgb(127, 230, 230),
    Color::Rgb(230, 230, 127),
    Color::Rgb(230, 127, 230),
    Color::Rgb(230, 230, 230),
    Color::Rgb(153, 153, 255),
    Color::Rgb(153, 255, 153),
    Color::Rgb(255, 153, 153),
    Color::Rgb(153, 255, 255),
    Color::Rgb(255, 255, 153),
    Color::Rgb(255, 153, 255),
    Color::Rgb(255, 255, 255),
];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// "Cached" indexing value for `color_rand`.
///
/// XXX: not thread-aware
#[doc(hidden)]
#[allow(non_upper_case_globals)]
static mut _color_at: usize = 0;

/// Return a random color from [`COLORS_TEXT`].
pub fn color_rand() -> Color {
    let ci: usize;
    unsafe {
        _color_at += 1;
        if _color_at == COLORS_TEXT.len() {
            _color_at = 0;
        }
        ci = _color_at;
    }

    COLORS_TEXT[ci]
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PrinterLogMessage
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A printer specialized for [`Sysline`s], and [`Utmpx`s].
///
/// [`Sysline`s]: crate::data::sysline::Sysline
/// [`Utmpx`s]: crate::data::utmpx::Utmpx
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
    /// color of printed logmessage data
    // TODO: [2023/03/22] remove this
    _color_logmessage: Color,
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
    prepend_date_format: String,
    /// timezone offset of printed date
    prepend_date_offset: Option<FixedOffset>,
    /// last value passed to `self.stdout_color.set_color()`
    ///
    /// used by macro `setcolor_or_return`
    color_spec_last: ColorSpec,
}

/// Macro to write to given stdout. If there is an error then
/// `return PrinterLogMessageResult::Err`.
macro_rules! write_or_return {
    ($stdout:expr, $slice_:expr, $printed:expr) => {
        match $stdout.write_all($slice_) {
            Ok(_) => {
                $printed += $slice_.len();
            }
            Err(err) => {
                // XXX: this will print when this program stdout is truncated, like when piping
                //      to `head`, e.g. `s4 file.log | head`
                //          Broken pipe (os error 32)
                de_err!(
                    "{}.write({}@{:p}) (len {})) error {}",
                    stringify!($stdout),
                    stringify!($slice_),
                    $slice_,
                    $slice_.len(),
                    err
                );
                match $stdout.flush() {
                    Ok(_) => {}
                    Err(_) => {}
                }
                return PrinterLogMessageResult::Err(err);
            }
        }
    };
}

/// Macro that sets output color, only changed if needed.
///
/// Unnecessary changes to `set_color` may cause errant formatting bytes to
/// print to the terminal.
macro_rules! setcolor_or_return {
    ($stdout:expr, $color_spec:expr, $color_spec_last:expr) => {
        if $color_spec != $color_spec_last {
            if let Err(err) = $stdout.set_color(&$color_spec) {
                de_err!("{}.set_color({:?}) returned error {}", stringify!($stdout), $color_spec, err);
                return PrinterLogMessageResult::Err(err);
            };
            $color_spec_last = $color_spec.clone();
        }
    };
}

// XXX: this was a `fn -> PrinterLogMessageResult` but due to mutable and immutable error, it would not compile.
//      So a macro is a decent workaround.
/// Macro helper to print a single line in color
macro_rules! print_color_line {
    ($stdout_color:expr, $linep:expr, $printed:expr) => {{
        for linepart in (*$linep).lineparts.iter() {
            let slice: &[u8] = linepart.as_slice();
            write_or_return!($stdout_color, slice, $printed);
        }
    }};
}

// XXX: this marco was originally a `fn -> PrinterLogMessageResult` but due to mutable and immutable borrow
//      error, it would not compile. So this macro is a decent workaround.
//
/// Macro helper to print a single line in color and highlight the datetime
/// within the line
macro_rules! print_color_line_highlight_dt {
    ($self:expr, $linep:expr, $dt_beg:expr, $dt_end:expr, $printed:expr) => {{
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
                    setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_a, $printed);
                }
                // print line contents of the entire datetime
                if !slice_b_dt.is_empty() {
                    setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_b_dt, $printed);
                }
                // print line contents after the datetime
                if !slice_c.is_empty() {
                    setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_c, $printed);
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
                    setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_a, $printed);
                }
                // print line contents of the partial datetime
                if !slice_b_dt.is_empty() {
                    setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_b_dt, $printed);
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
                    setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_a_dt, $printed);
                }
                // print line contents after the datetime
                if !slice_b.is_empty() {
                    setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_b, $printed);
                }
            // datetime began in previous linepart, extends into next linepart
            } else if $dt_beg < at && at_end <= $dt_end {
                // print entire linepart which is the partial datetime
                setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                write_or_return!($self.stdout_color, slice, $printed);
            // datetime is not in this linepart
            } else {
                // print entire linepart
                setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                write_or_return!($self.stdout_color, slice, $printed);
            }
            at += slice.len() as LineIndex;
        }
    }};
}

/// Aliased [`Result`] returned by various [`PrinterLogMessage`] functions.
///
/// [`Result`]: std::io::Result
pub type PrinterLogMessageResult = Result<usize>;

impl PrinterLogMessage {
    /// Create a new `PrinterLogMessage`.
    pub fn new(
        color_choice: ColorChoice,
        color_logmessage: Color,
        prepend_file: Option<String>,
        prepend_date_format: Option<String>,
        prepend_date_offset: Option<FixedOffset>,
    ) -> PrinterLogMessage {
        // get a stdout handle once
        let stdout = std::io::stdout();
        let stdout_color = termcolor::StandardStream::stdout(color_choice);
        let do_color: bool = match color_choice {
            ColorChoice::Never => false,
            ColorChoice::Always | ColorChoice::AlwaysAnsi | ColorChoice::Auto => true,
        };
        let mut color_spec_default: ColorSpec = ColorSpec::new();
        color_spec_default.set_fg(Some(COLOR_DEFAULT));
        let mut color_spec_sysline: ColorSpec = ColorSpec::new();
        color_spec_sysline.set_fg(Some(color_logmessage));
        let mut color_spec_datetime: ColorSpec = ColorSpec::new();
        color_spec_datetime.set_fg(Some(color_logmessage));
        color_spec_datetime.set_underline(true);
        let color_spec_last = color_spec_default.clone();
        let do_prepend_date = prepend_date_offset.is_some();
        let prepend_date_format_: String = prepend_date_format.unwrap_or_default();
        if do_prepend_date {
            assert!(
                !prepend_date_format_.is_empty(),
                "passed a prepend_date_offset, must pass a prepend_date_format"
            );
        }

        PrinterLogMessage {
            stdout,
            stdout_color,
            do_color,
            _color_choice: color_choice,
            color_spec_default,
            _color_logmessage: color_logmessage,
            color_spec_sysline,
            color_spec_datetime,
            do_prepend_file: prepend_file.is_some(),
            prepend_file,
            do_prepend_date,
            prepend_date_format: prepend_date_format_,
            prepend_date_offset,
            color_spec_last,
        }
    }

    /// Prints the [`SyslineP`] based on `PrinterLogMessage` settings.
    ///
    /// Users should call this function.
    ///
    /// [`SyslineP`]: crate::data::sysline::SyslineP
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

    /// Prints the [`Utmpx`] based on `PrinterLogMessage` settings.
    ///
    /// Users should call this function.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    #[inline(always)]
    pub fn print_utmpx(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        // TODO: [2023/03/23] refactor `print_utmp*` similar to `print_evtx*`
        match (self.do_color, self.do_prepend_file, self.do_prepend_date) {
            (false, false, false) => self.print_utmpx_(utmpx, buffer),
            (false, true, false) => self.print_utmpx_prependfile(utmpx, buffer),
            (false, false, true) => self.print_utmpx_prependdate(utmpx, buffer),
            (false, true, true) => self.print_utmpx_prependfile_prependdate(utmpx, buffer),
            (true, false, false) => self.print_utmpx_color(utmpx, buffer),
            (true, true, false) => self.print_utmpx_prependfile_color(utmpx, buffer),
            (true, false, true) => self.print_utmpx_prependdate_color(utmpx, buffer),
            (true, true, true) => self.print_utmpx_prependfile_prependdate_color(utmpx, buffer),
        }
    }

    /// Prints the [`Evtx`] based on `PrinterLogMessage` settings.
    ///
    /// Users should call this function.
    ///
    /// [`Evtx`]: crate::data::evtx::Evtx
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

    /// Helper function to transform [`sysline.dt`] to a `String`.
    ///
    /// [`sysline.dt`]: crate::data::sysline::Sysline
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
            .dt
            .unwrap()
            .with_timezone(
                &self
                    .prepend_date_offset
                    .unwrap(),
            );
        let dt_delayedformat = dt_.format(
            self.prepend_date_format
                .as_str(),
        );

        dt_delayedformat.to_string()
    }

    /// Helper function to transform [`utmpx.dt`] to a `String`.
    ///
    /// [`utmpx.dt`]: crate::data::utmpx::utmpx#structfield.dt
    #[inline(always)]
    fn datetime_to_string_utmpx(
        &self,
        utmpx: &Utmpx,
    ) -> String {
        // write the `utmpx.dt` into a `String` once
        let dt_: DateTimeL = (*utmpx)
            .dt()
            .with_timezone(
                &self
                    .prepend_date_offset
                    .unwrap(),
            );
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
            .with_timezone(
                &self
                    .prepend_date_offset
                    .unwrap(),
            );
        let dt_delayedformat = dt_.format(
            self.prepend_date_format.as_str(),
        );

        dt_delayedformat.to_string()
    }

    // TODO: make this a macro and it could be used in all functions
    /// Helper function to print [`lineparts`].
    ///
    /// [`lineparts`]: crate::data::line::LineParts
    #[inline(always)]
    fn print_line(
        &self,
        linep: &LineP,
        stdout_lock: &mut StdoutLock,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        for linepart in (*linep).lineparts.iter() {
            let slice: &[u8] = linepart.as_slice();
            write_or_return!(stdout_lock, slice, printed);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Sysline`] without anything special.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            printed += self.print_line(linep, &mut stdout_lock)?;
        }
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Sysline`] with prepended datetime.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_prependdate(
        &self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        debug_assert!(
            self.prepend_date_offset
                .is_some(),
            "self.prepend_date_offset is {:?}",
            self.prepend_date_offset
        );

        let mut printed: usize = 0;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            write_or_return!(stdout_lock, dtb, printed);
            printed += self.print_line(linep, &mut stdout_lock)?;
        }
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// print a [`Sysline`] with prepended filename.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_prependfile(
        &self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);

        let mut printed: usize = 0;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            write_or_return!(stdout_lock, prepend_file, printed);
            printed += self.print_line(linep, &mut stdout_lock)?;
        }
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Sysline`] with prepended filename and datetime.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_prependfile_prependdate(
        &self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);
        debug_assert!(
            self.prepend_date_offset
                .is_some(),
            "self.prepend_date_offset is {:?}",
            self.prepend_date_offset
        );

        let mut printed: usize = 0;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            write_or_return!(stdout_lock, prepend_file, printed);
            write_or_return!(stdout_lock, dtb, printed);
            printed += self.print_line(linep, &mut stdout_lock)?;
        }
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Prints [`Sysline`] in color.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut line_first = true;
        let stdout_lock = self.stdout.lock();
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        for linep in (*syslinep).lines.iter() {
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, linep, dt_beg, dt_end, printed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep, printed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Sysline`] in color and prepended datetime.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_prependdate_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut line_first = true;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            write_or_return!(self.stdout_color, dtb, printed);
            setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, linep, dt_beg, dt_end, printed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep, printed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Prints [`Sysline`] in color and prepended filename.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_prependfile_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut line_first = true;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            write_or_return!(self.stdout_color, prepend_file, printed);
            setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, linep, dt_beg, dt_end, printed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep, printed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Sysline`] in color and prepended filename and datetime.
    ///
    /// [`Sysline`]: crate::data::sysline::Sysline
    fn print_sysline_prependfile_prependdate_color(
        &mut self,
        syslinep: &SyslineP,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut line_first = true;
        let dt_string: String = self.datetime_to_string_sysline(syslinep);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            write_or_return!(self.stdout_color, prepend_file, printed);
            write_or_return!(self.stdout_color, dtb, printed);
            setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
            if line_first {
                let dt_beg = (*syslinep).dt_beg;
                let dt_end = (*syslinep).dt_end;
                print_color_line_highlight_dt!(self, linep, dt_beg, dt_end, printed);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep, printed);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Utmpx`] without anything special.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_(
        &self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let at = match utmpx.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };
        let mut stdout_lock = self.stdout.lock();
        write_or_return!(stdout_lock, &buffer[..at], printed);
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Utmpx`] with prepended datetime.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_prependdate(
        &self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        debug_assert!(
            self.prepend_date_offset
                .is_some(),
            "self.prepend_date_offset is {:?}",
            self.prepend_date_offset
        );

        let mut printed: usize = 0;
        let dt_string: String = self.datetime_to_string_utmpx(utmpx);
        let dtb: &[u8] = dt_string.as_bytes();
        let at = match utmpx.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };

        let mut stdout_lock = self.stdout.lock();
        write_or_return!(stdout_lock, dtb, printed);
        write_or_return!(stdout_lock, &buffer[..at], printed);
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// prints [`Utmpx`] with prepended filename.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_prependfile(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);

        let mut printed: usize = 0;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let at = match utmpx.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };
        let mut stdout_lock = self.stdout.lock();
        write_or_return!(stdout_lock, prepend_file, printed);
        write_or_return!(stdout_lock, &buffer[..at], printed);
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Utmpx`] with prepended filename and datetime.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_prependfile_prependdate(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);
        debug_assert!(
            self.prepend_date_offset
                .is_some(),
            "self.prepend_date_offset is {:?}",
            self.prepend_date_offset
        );

        let mut printed: usize = 0;
        let dt_string: String = self.datetime_to_string_utmpx(utmpx);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let at = match utmpx.as_bytes(buffer) {
            InfoAsBytes::Ok(at, _, _) => at,
            InfoAsBytes::Fail(at) => at,
        };

        let mut stdout_lock = self.stdout.lock();
        write_or_return!(stdout_lock, dtb, printed);
        write_or_return!(stdout_lock, prepend_file, printed);
        write_or_return!(stdout_lock, &buffer[..at], printed);
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Prints [`Utmpx`] in color.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_color(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let (at, beg, end) = match utmpx.as_bytes(buffer) {
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
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[..beg], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_datetime, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[beg..end], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[end..at], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Utmpx`] in color and prepended datetime.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_prependdate_color(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let dt_string: String = self.datetime_to_string_utmpx(utmpx);
        let dtb: &[u8] = dt_string.as_bytes();
        let (at, beg, end) = match utmpx.as_bytes(buffer) {
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
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        write_or_return!(self.stdout_color, dtb, printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[..beg], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_datetime, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[beg..end], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[end..at], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Prints [`Utmpx`] in color and prepended filename.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_prependfile_color(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let (at, beg, end) = match utmpx.as_bytes(buffer) {
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
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        write_or_return!(self.stdout_color, prepend_file, printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[..beg], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_datetime, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[beg..end], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[end..at], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Utmpx`] in color and prepended filename and datetime.
    ///
    /// [`Utmpx`]: crate::data::utmpx::Utmpx
    fn print_utmpx_prependfile_prependdate_color(
        &mut self,
        utmpx: &Utmpx,
        buffer: &mut [u8],
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let dt_string: String = self.datetime_to_string_utmpx(utmpx);
        let dtb: &[u8] = dt_string.as_bytes();
        let prepend_file: &[u8] = self
            .prepend_file
            .as_ref()
            .unwrap()
            .as_bytes();
        let (at, beg, end) = match utmpx.as_bytes(buffer) {
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
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        write_or_return!(self.stdout_color, prepend_file, printed);
        write_or_return!(self.stdout_color, dtb, printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[..beg], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_datetime, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[beg..end], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &buffer[end..at], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Evtx`] without anything special. Optimized for this simple
    /// common case.
    ///
    /// [`Evtx`]: crate::data::evtx::Evtx
    fn print_evtx_(
        &self,
        evtx: &Evtx,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
        let mut stdout_lock = self.stdout.lock();
        write_or_return!(stdout_lock, evtx.as_bytes(), printed);
        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Evtx`] with prepended file and/or datetime.
    ///
    /// [`Evtx`]: crate::data::evtx::Evtx
    fn print_evtx_prepend(
        &self,
        evtx: &Evtx,
        do_prependfile: bool,
        do_prependdate: bool,
    ) -> PrinterLogMessageResult {
        debug_assert!(
            self.prepend_date_offset
                .is_some(),
            "self.prepend_date_offset is {:?}",
            self.prepend_date_offset
        );

        let mut printed: usize = 0;
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
        while let Some(b) = data[a..].find_byte(NLu8) {
            let line = &data[a..a + b + CHARSZ];
            a += b + CHARSZ;
            if line.is_empty() {
                continue;
            }
            if do_prependfile {
                write_or_return!(stdout_lock, prepend_file, printed);
            }
            if do_prependdate {
                write_or_return!(stdout_lock, prepend_date, printed);
            }
            write_or_return!(stdout_lock, line, printed);
        }

        if let Result::Err(err) = stdout_lock.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Prints [`Evtx`] in color. Optimized for this simple common case.
    ///
    /// [`Evtx`]: crate::data::evtx::Evtx
    fn print_evtx_color(
        &mut self,
        evtx: &Evtx,
    ) -> PrinterLogMessageResult {
        let (beg, end) = match evtx.dt_beg_end() {
            Some((beg, end)) => (*beg, *end),
            None => (0, 0)
        };
        debug_assert_lt!(beg, end, "beg: {}, end: {}", beg, end);
        let mut printed: usize = 0;
        let data = evtx.as_bytes();
        let stdout_lock = self.stdout.lock();
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &data[..beg], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_datetime, self.color_spec_last);
        write_or_return!(self.stdout_color, &data[beg..end], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        write_or_return!(self.stdout_color, &data[end..], printed);
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
        black_box(&stdout_lock);
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }

        PrinterLogMessageResult::Ok(printed)
    }

    /// Print a [`Evtx`] in color and prepended filename and/or datetime.
    ///
    /// [`Evtx`]: crate::data::evtx::Evtx
    fn print_evtx_prepend_color(
        &mut self,
        evtx: &Evtx,
        do_prependfile: bool,
        do_prependdate: bool,
    ) -> PrinterLogMessageResult {
        let mut printed: usize = 0;
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
            None => (0, 0)
        };
        debug_assert_le!(beg, end, "beg: {}, end: {}", beg, end);
        let data = evtx.as_bytes();
        let mut at: usize = 0;
        let mut a: usize = 0;
        let stdout_lock = self.stdout.lock();
        while let Some(b) = data[a..].find_byte(NLu8) {
            let line = &data[a..a + b + CHARSZ];
            a += b + CHARSZ;
            if line.is_empty() {
                continue;
            }
            let len = line.len();
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            if do_prependfile {
                write_or_return!(self.stdout_color, prepend_file, printed);
            }
            if do_prependdate {
                write_or_return!(self.stdout_color, prepend_date, printed);
            }
            match (at <= beg, end < at + len) {
                (true, true) => {
                    setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
                    write_or_return!(self.stdout_color, &line[..beg - at], printed);
                    setcolor_or_return!(self.stdout_color, self.color_spec_datetime, self.color_spec_last);
                    write_or_return!(self.stdout_color, &line[beg - at..end - at], printed);
                    setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
                    write_or_return!(self.stdout_color, &line[end - at..], printed);
                }
                _ => {
                    setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
                    write_or_return!(self.stdout_color, line, printed);
                }
            }
            at += line.len();
        }
        if let Result::Err(err) = self.stdout_color.flush() {
            return PrinterLogMessageResult::Err(err);
        }
        black_box(&stdout_lock);

        PrinterLogMessageResult::Ok(printed)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// other printer functions (no use of PrinterLogMessage)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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
    let _stderr_lock = std::io::stderr().lock();

    print_colored(color, value, &mut stderr)
}

/// Safely write the `buffer` to stdout with help of [`StdoutLock`].
///
/// [`StdoutLock`]: std::io::StdoutLock
pub fn write_stdout(buffer: &[u8]) {
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    let _stderr_lock = std::io::stderr().lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stdout_lock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), _err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stdout_lock.flush() error {}", _err);
        }
    }
}

/// Safely write the `buffer` to stderr with help of [`StderrLock`].
///
/// [`StderrLock`]: std::io::StderrLock
pub fn write_stderr(buffer: &[u8]) {
    let mut stderr_lock = std::io::stderr().lock();
    let mut stdout_lock = std::io::stdout().lock();
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
        Ok(_) => {}
        Err(_err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            de_err!("stderr flushing error {}", _err);
        }
    }
    if cfg!(debug_assertions) {
        #[allow(clippy::match_single_binding)]
        match stdout_lock.flush() {
            _ => {}
        }
    }
}
