// printer/printers.rs
//
// printing - printer functions and helpers
//

use std::io::Write;  // for `std::io::Stdout.flush`
use std::io::Result;

extern crate debug_print;
use debug_print::{
    debug_eprintln,
};

extern crate termcolor;
pub use termcolor::{
    Color,
    ColorChoice,
    ColorSpec,
    WriteColor,
};

use crate::Data::line::{
    LineP,
    LineIndex,
};

use crate::Data::sysline::{
    SyslineP,
};

use crate::Data::datetime::{
    DateTimeL,
    FixedOffset,
};

extern crate more_asserts;
use more_asserts::{
    assert_le,
    debug_assert_le,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// globals and constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub const COLOR_DATETIME: Color = Color::Green;

pub const COLOR_ERROR: Color = Color::Red;

/// A preselection of colors for printing syslines and file names.
/// Chosen for a dark background console.
const COLORS_TEXT: [Color; 29] = [
    Color::Yellow,
    Color::Cyan,
    Color::Red,
    Color::Magenta,
    // decent reference https://www.rapidtables.com/web/color/RGB_Color.html
    // XXX: colors with low pixel values are difficult to see on dark console backgrounds
    //      recommend at least one pixel value of 102 or greater
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

/// "cached" indexing value for `color_rand`
///
/// XXX: not thread-aware
#[allow(non_upper_case_globals)]
static mut _color_at: usize = 0;

/// return a random color from `COLORS`
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
// Printer_Sysline
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// a printer specialized for `Sysline`s
pub struct Printer_Sysline {
    /// handle to stdout
    /// TODO: make this a single global lazy_static
    stdout: std::io::Stdout,
    /// termcolor handle to stdout
    /// TODO: make this a single global lazy_static
    stdout_color: termcolor::StandardStream,
    /// should printing be in color?
    do_color: bool,
    /// termcolor::ColorChoice
    color_choice: ColorChoice,
    /// color settings for plain text (not sysline)
    color_spec_default: ColorSpec,
    /// color of printed sysline data
    color_sysline: Color,
    /// color settings for sysline text
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
    /// managed by macro `setcolor_or_return`
    color_spec_last: ColorSpec,
}

macro_rules! write_or_return {
    ($stdout:expr, $var_a:expr) => {
        match $stdout.write($var_a) {
            Ok(_) => {}
            Err(err) => {
                // XXX: this will print when this program stdout is truncated, like when piping
                //      to `head`, e.g. `s4 file.log | head`
                //          Broken pipe (os error 32)
                eprintln!("ERROR: {}.write({}@{:p}) (len {})) error {}", stringify!($stdout), stringify!($var_a), $var_a, $var_a.len(), err);
                match $stdout.flush() {
                    Ok(_) => {},
                    Err(_) => {},
                }
                return Err(err);
            }
        }
    };
}

macro_rules! setcolor_or_return {
    ($stdout:expr, $color_spec:expr, $color_spec_last:expr) => {
        if  $color_spec != $color_spec_last {
            if let Err(err) = $stdout.set_color(&$color_spec) {
                eprintln!("ERROR: {}.set_color({:?}) returned error {}", stringify!($stdout), $color_spec, err);
                return Err(err);
            };
            $color_spec_last = $color_spec.clone();
        }
    };
}

// XXX: this was a `fn -> Result<()>` but due to mutable and immutable error, it would not compile.
//      So a macro is a decent workaround.
/// helper to print a single line in color
macro_rules! print_color_line {
    ($stdout_color:expr, $linep:expr) => {
        {
            for linepart in (*$linep).lineparts.iter() {
                let slice: &[u8] = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
                write_or_return!($stdout_color, slice);
            }
        }
    }
}

// XXX: this was a `fn -> Result<()>` but due to mutable and immutable borrow error, it would not compile.
//      So this macro is a decent workaround.
/// helper to print a single line in color and highlight the datetime within the line
macro_rules! print_color_line_highlight_dt {
    ($self:expr, $linep:expr, $dt_beg:expr, $dt_end:expr) => {
        {
            debug_assert_le!($dt_beg, $dt_end, "passed bad datetime_beg {:?} datetime_end {:?}", $dt_beg, $dt_end);
            let mut at: LineIndex = 0;
            // this tedious indexing manual is faster than calling `line.get_boxptrs`
            // especially since `$dt_beg` `$dt_end` is a sub-slice(s) of the total `Line` slice(s)
            for linepart in (*$linep).lineparts.iter() {
                let slice: &[u8] = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
                let at_end: usize = at + slice.len();
                // datetime is entirely within one linepart
                if at <= $dt_beg && $dt_end < at_end {
                    assert_le!(($dt_beg-at), slice.len(), "at {} dt_beg {} (dt_beg-at {} > {} slice.len()) dt_end {} A", at, $dt_beg, $dt_beg-at, slice.len(), $dt_end);
                    assert_le!(($dt_end-at), slice.len(), "at {} dt_beg {} dt_end {} (dt_end-at {} > {} slice.len()) A", at, $dt_beg, $dt_end, $dt_end-at, slice.len());
                    let slice_a = &slice[..($dt_beg-at)];
                    let slice_b_dt = &slice[($dt_beg-at)..($dt_end-at)];
                    let slice_c = &slice[($dt_end-at)..];
                    // print line contents before the datetime
                    if !slice_a.is_empty() {
                        setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                        write_or_return!($self.stdout_color, slice_a);
                    }
                    // print line contents of the entire datetime
                    setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice_b_dt);
                    // print line contents after the datetime
                    if !slice_c.is_empty() {
                        setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                        write_or_return!($self.stdout_color, slice_c);
                    }
                // datetime begins in this linepart, extends into next linepart
                } else if at <= $dt_beg && $dt_beg < at_end && at_end <= $dt_end {
                    assert_le!(($dt_beg-at), slice.len(), "at {} dt_beg {} (dt_beg-at {} > {} slice.len()) dt_end {} at_end {} B", at, $dt_beg, $dt_beg-at, slice.len(), $dt_end, at_end);
                    let slice_a = &slice[..($dt_beg-at)];
                    let slice_b_dt = &slice[($dt_beg-at)..];
                    // print line contents before the datetime
                    if !slice_a.is_empty() {
                        setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                        write_or_return!($self.stdout_color, slice_a);
                    }
                    // print line contents of the partial datetime
                    if !slice_b_dt.is_empty() {
                        setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                        write_or_return!($self.stdout_color, slice_b_dt);
                    }
                // datetime began in previous linepart, extends into this linepart and ends within this linepart
                } else if $dt_beg < at && at <= $dt_end && $dt_end <= at_end {
                    assert_le!(($dt_end-at), slice.len(), "at {} dt_beg {} dt_end {} (dt_end-at {} > {} slice.len()) C", at, $dt_beg, $dt_end, $dt_end-at, slice.len());
                    let slice_a_dt = &slice[..($dt_end-at)];
                    let slice_b = &slice[($dt_end-at)..];
                    // print line contents of the partial datetime
                    if !slice_a_dt.is_empty() {
                        setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                        write_or_return!($self.stdout_color, slice_a_dt);
                    }
                    // print line contents after the datetime
                    if !slice_b.is_empty() {
                        setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                        write_or_return!($self.stdout_color, slice_b);
                    }
                // datetime began in previous linepart, extends into next linepart
                } else if $dt_beg < at && at_end <= $dt_end {
                    // print entire linepart which is the partial datetime
                    setcolor_or_return!($self.stdout_color, $self.color_spec_datetime, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice);
                // datetime is not in this linepart
                } else {
                    // print entire linepart
                    setcolor_or_return!($self.stdout_color, $self.color_spec_sysline, $self.color_spec_last);
                    write_or_return!($self.stdout_color, slice);
                }
                at += slice.len() as LineIndex;
            };
        }
    }
}

impl Printer_Sysline {
    pub fn new(
        color_choice: ColorChoice,
        color_sysline: Color,
        prepend_file: Option<String>,
        prepend_date_format: Option<String>,
        prepend_date_offset: Option<FixedOffset>,
    ) -> Printer_Sysline {
        // get a stdout handle once
        let stdout = std::io::stdout();
        let stdout_color = termcolor::StandardStream::stdout(color_choice);
        let do_color: bool = match color_choice {
            ColorChoice::Never => false,
            ColorChoice::Always | ColorChoice::AlwaysAnsi | ColorChoice::Auto => true,
        };
        let color_spec_default: ColorSpec = ColorSpec::new();
        let mut color_spec_sysline: ColorSpec = ColorSpec::new();
        color_spec_sysline.set_fg(Some(color_sysline));
        let mut color_spec_datetime: ColorSpec = ColorSpec::new();
        color_spec_datetime.set_fg(Some(color_sysline));
        color_spec_datetime.set_underline(true);
        let color_spec_last = color_spec_default.clone();
        let do_prepend_date = prepend_date_offset.is_some();
        let prepend_date_format_: String = prepend_date_format.unwrap_or_default();
        if do_prepend_date {
            assert!(!prepend_date_format_.is_empty(), "passed a prepend_date_offset, must pass a prepend_date_format");
        }

        Printer_Sysline {
            stdout,
            stdout_color,
            do_color,
            color_choice,
            color_spec_default,
            color_sysline,
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

    /// prints the `SyslineP` based on `Printer_Sysline` settings
    ///
    /// users should call this function
    #[inline(always)]
    pub fn print_sysline(&mut self, syslinep: &SyslineP) -> Result<()> {
        // TODO: [2022/06/19] how to determine if "Auto" has become Always or Never?
        // see https://docs.rs/termcolor/latest/termcolor/#detecting-presence-of-a-terminal
        match (self.do_color, self.do_prepend_file, self.do_prepend_date) {
            (false, false, false) => self.print_sysline_(syslinep),
            (false, true, false) => self.print_sysline_prependfile(syslinep),
            (false, false, true) => self.print_sysline_prependdate(syslinep),
            (false, true, true) => self.print_sysline_prependfile_prependdate(syslinep),
            (true, false, false) => self.print_color_sysline(syslinep),
            (true, true, false) => self.print_color_sysline_prependfile(syslinep),
            (true, false, true) => self.print_color_sysline_prependdate(syslinep),
            (true, true, true) => self.print_color_sysline_prependfile_prependdate(syslinep),
        }
    }

    /// helper to transform `syslinep.dt` to a `String`
    #[inline(always)]
    fn datetime_to_string(&mut self, syslinep: &SyslineP) -> String {
        // write the `syslinep.dt` into a `String` once
        //
        // XXX: would be cool if `chrono::DateTime` offered a format that returned
        //      `[u8; 100]` on the stack (where `100` is maximum possible length).
        //      That would be much faster than heap allocating a new `String`.
        //      instead, `format` returns a `DelayedFormat` object
        //      https://docs.rs/chrono/latest/chrono/format/struct.DelayedFormat.html
        //
        let dt_: DateTimeL = (*syslinep).dt.unwrap().with_timezone(&self.prepend_date_offset.unwrap());
        let dt_delayedformat = dt_.format(self.prepend_date_format.as_str());

        dt_delayedformat.to_string()
    }

    // TODO: make this a macro and it could be used in all functions
    // TODO: handle special common case of Line slice residing on one line, do it faster
    /// helper to print lineparts
    #[inline(always)]
    fn print_line(&self, linep: &LineP, stdout_lock: &mut std::io::StdoutLock) -> Result<()> {
        for linepart in (*linep).lineparts.iter() {
            let slice: &[u8] = &linepart.blockp[linepart.blocki_beg..linepart.blocki_end];
            write_or_return!(stdout_lock, slice);
        }

        Ok(())
    }

    // TODO: 2020/06/20 handle common case where Sysline resides entirely
    //       on one block (one `&[u8]` sequence), print entire slice in one write call.
    //       more efficient for common case
    /// print a `Sysline` without anything special
    ///
    fn print_sysline_(&mut self, syslinep: &SyslineP) -> Result<()> {
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            self.print_line(linep, &mut stdout_lock)?;
        }

        stdout_lock.flush()
    }

    fn print_sysline_prependdate(&mut self, syslinep: &SyslineP) -> Result<()> {
        debug_assert!(self.prepend_date_offset.is_some(), "self.prepend_date_offset is {:?}", self.prepend_date_offset);

        let dt_string: String = self.datetime_to_string(syslinep);
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            write_or_return!(stdout_lock, dt_string.as_bytes());
            self.print_line(linep, &mut stdout_lock)?;
        }

        stdout_lock.flush()
    }

    fn print_sysline_prependfile(&mut self, syslinep: &SyslineP) -> Result<()> {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);

        let prepend_file: &[u8] = self.prepend_file.as_ref().unwrap().as_bytes();
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            write_or_return!(stdout_lock, prepend_file);
            self.print_line(linep, &mut stdout_lock)?;
        }

        stdout_lock.flush()
    }

    fn print_sysline_prependfile_prependdate(&mut self, syslinep: &SyslineP) -> Result<()> {
        debug_assert!(self.prepend_file.is_some(), "self.prepend_file is {:?}", self.prepend_file);
        debug_assert!(self.prepend_date_offset.is_some(), "self.prepend_date_offset is {:?}", self.prepend_date_offset);

        let dt_string: String = self.datetime_to_string(syslinep);
        let prepend_file: &[u8] = self.prepend_file.as_ref().unwrap().as_bytes();
        let mut stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            write_or_return!(stdout_lock, prepend_file);
            write_or_return!(stdout_lock, dt_string.as_bytes());
            self.print_line(linep, &mut stdout_lock)?;
        }

        stdout_lock.flush()
    }

    /// prints `Sysline` with color
    fn print_color_sysline(&mut self, syslinep: &SyslineP) -> Result<()> {
        let mut line_first = true;
        let _stdout_lock = self.stdout.lock();
        setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
        for linep in (*syslinep).lines.iter() {
            if line_first {
                print_color_line_highlight_dt!(self, linep, (*syslinep).dt_beg, (*syslinep).dt_end);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);

        self.stdout_color.flush()
    }

    fn print_color_sysline_prependdate(&mut self, syslinep: &SyslineP) -> Result<()> {
        let mut line_first = true;
        let dt_string: String = self.datetime_to_string(syslinep);
        let _stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            write_or_return!(self.stdout_color, dt_string.as_bytes());
            setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
            if line_first {
                print_color_line_highlight_dt!(self, linep, (*syslinep).dt_beg, (*syslinep).dt_end);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);

        self.stdout_color.flush()
    }

    fn print_color_sysline_prependfile(&mut self, syslinep: &SyslineP) -> Result<()> {
        let mut line_first = true;
        let prepend_file: &[u8] = self.prepend_file.as_ref().unwrap().as_bytes();
        let _stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            write_or_return!(self.stdout_color, prepend_file);
            setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
            if line_first {
                print_color_line_highlight_dt!(self, linep, (*syslinep).dt_beg, (*syslinep).dt_end);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);

        self.stdout_color.flush()
    }

    fn print_color_sysline_prependfile_prependdate(&mut self, syslinep: &SyslineP) -> Result<()> {
        let mut line_first = true;
        let dt_string: String = self.datetime_to_string(syslinep);
        let prepend_file: &[u8] = self.prepend_file.as_ref().unwrap().as_bytes();
        let _stdout_lock = self.stdout.lock();
        for linep in (*syslinep).lines.iter() {
            setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);
            write_or_return!(self.stdout_color, prepend_file);
            write_or_return!(self.stdout_color, dt_string.as_bytes());
            setcolor_or_return!(self.stdout_color, self.color_spec_sysline, self.color_spec_last);
            if line_first {
                print_color_line_highlight_dt!(self, linep, (*syslinep).dt_beg, (*syslinep).dt_end);
                line_first = false;
            } else {
                print_color_line!(self.stdout_color, linep);
            }
        }
        setcolor_or_return!(self.stdout_color, self.color_spec_default, self.color_spec_last);

        self.stdout_color.flush()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// other printer functions (no use of Printer_Sysline)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// print colored output to terminal if possible choosing using passed stream,
/// otherwise, print plain output.
///
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
pub fn print_colored(color: Color, value: &[u8], out: &mut termcolor::StandardStream) -> std::io::Result<()> {
    match out.set_color(ColorSpec::new().set_fg(Some(color))) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std.set_color({:?}) returned error {}", color, err);
            return Err(err);
        }
    };
    match out.write(value) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: out.write(…) returned error {}", err);
            return Err(err);
        }
    }
    match out.reset() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: out.reset() returned error {}", err);
            return Err(err);
        }
    }
    out.flush()?;

    Ok(())
}

/// print colored output to terminal on stdout.
///
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
#[cfg(test)]
pub fn print_colored_stdout(
    color: Color,
    color_choice_opt: Option<ColorChoice>,
    value: &[u8]
) -> std::io::Result<()> {
    let choice: ColorChoice = match color_choice_opt {
        Some(choice_) => choice_,
        None => ColorChoice::Auto,
    };
    let mut stdout = termcolor::StandardStream::stdout(choice);
    let stdout_lock = std::io::stdout().lock();
    let stderr_lock = std::io::stderr().lock();

    print_colored(color, value, &mut stdout)
}

/// print colored output to terminal on stderr.
///
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
pub fn print_colored_stderr(
    color: Color,
    color_choice_opt: Option<ColorChoice>,
    value: &[u8]
) -> std::io::Result<()> {
    let choice: ColorChoice = match color_choice_opt {
        Some(choice_) => choice_,
        None => ColorChoice::Auto,
    };
    let mut stderr = termcolor::StandardStream::stderr(choice);
    let stdout_lock = std::io::stdout().lock();
    let stderr_lock = std::io::stderr().lock();

    print_colored(color, value, &mut stderr)
}

/// safely write the `buffer` to stdout with help of `StdoutLock`
pub fn write_stdout(buffer: &[u8]) {
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    let stderr_lock = std::io::stderr().lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: StdoutLock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: stdout flushing error {}", err);
        }
    }
}

/// safely write the `buffer` to stdout with help of `StderrLock`
#[cfg(test)]
pub fn write_stderr(buffer: &[u8]) {
    let mut stderr_lock = std::io::stderr().lock();
    let mut stdout_lock = std::io::stdout().lock();
    // BUG: this print is shown during `cargo test` yet nearby `eprintln!` are not seen
    //      Would like this to only show when `--no-capture` is passed (this is how
    //      `eprintln!` behaves)
    match stderr_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: stderr_lock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), err);
        }
    }
    match stderr_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: stderr flushing error {}", err);
        }
    }
    if cfg!(debug_assertions) {
        #[allow(clippy::match_single_binding)]
        match stdout_lock.flush() {
            _ => {},
        }
    }
}
