// printer/printers.rs
//
// printing - printer functions and helpers
//

use std::io::Write;  // for `std::io::Stdout.flush`
use std::io::Result;

extern crate termcolor;
pub use termcolor::{
    Color,
    ColorSpec,
    WriteColor
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// globals and constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub static COLOR_DATETIME: Color = Color::Green;

static COLORS_TEXT: [Color; 29] = [
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

/// print colored output to terminal if possible choosing using passed stream
/// otherwise, print plain output
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
pub fn print_colored(color: Color, value: &[u8], std_: &mut termcolor::StandardStream) -> Result<()> {
    match std_.set_color(ColorSpec::new().set_fg(Some(color))) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std.set_color({:?}) returned error {}", color, err);
            return Err(err);
        }
    };
    //let mut stderr_lock:Option<io::StderrLock> = None;
    //if cfg!(debug_assertions) {
    //    stderr_lock = Some(io::stderr().lock());
    //}
    match std_.write(value) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std_.write(…) returned error {}", err);
            return Err(err);
        }
    }
    match std_.reset() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("ERROR: print_colored: std_.reset() returned error {}", err);
            return Err(err);
        }
    }
    std_.flush()?;
    Ok(())
}

/// print colored output to terminal on stdout
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
pub fn print_colored_stdout(
    color: Color,
    color_choice_opt: Option<termcolor::ColorChoice>,
    value: &[u8]
) -> std::io::Result<()> {
    let choice: termcolor::ColorChoice = match color_choice_opt {
        Some(choice_) => choice_,
        None => termcolor::ColorChoice::Auto,
    };
    let mut stdout = termcolor::StandardStream::stdout(choice);
    print_colored(color, value, &mut stdout)
}

/// print colored output to terminal on stderr
/// taken from https://docs.rs/termcolor/1.1.2/termcolor/#detecting-presence-of-a-terminal
pub fn print_colored_stderr(
    color: Color,
    color_choice_opt: Option<termcolor::ColorChoice>,
    value: &[u8]
) -> std::io::Result<()> {
    let choice: termcolor::ColorChoice = match color_choice_opt {
        Some(choice_) => choice_,
        None => termcolor::ColorChoice::Auto,
    };
    let mut stderr = termcolor::StandardStream::stderr(choice);
    print_colored(color, value, &mut stderr)
}

/// safely write the `buffer` to stdout with help of `StdoutLock`
pub fn write_stdout(buffer: &[u8]) {
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    match stdout_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: StdoutLock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), err);
        }
    }
    match stdout_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: stdout flushing error {}", err);
        }
    }
    if cfg!(debug_assertions) {
        #[allow(clippy::match_single_binding)]
        match std::io::stderr().flush() {
            _ => {},
        }
    }
}

/// safely write the `buffer` to stdout with help of `StderrLock`
pub fn write_stderr(buffer: &[u8]) {
    let stderr = std::io::stderr();
    let mut stderr_lock = stderr.lock();
    match stderr_lock.write(buffer) {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: StderrLock.write(buffer@{:p} (len {})) error {}", buffer, buffer.len(), err);
        }
    }
    match stderr_lock.flush() {
        Ok(_) => {}
        Err(err) => {
            // XXX: this will print when this program stdout is truncated, like to due to `program | head`
            //          Broken pipe (os error 32)
            //      Not sure if anything should be done about it
            eprintln!("ERROR: write: stderr flushing error {}", err);
        }
    }
    if cfg!(debug_assertions) {
        #[allow(clippy::match_single_binding)]
        match std::io::stdout().flush() {
            _ => {},
        }
    }
}
