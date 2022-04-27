// dbgpr/helpers.rs
//

#[cfg(test)]
use crate::common::{
    FPath,
};

#[cfg(test)]
use crate::dbgpr::printers::{
    str_to_String_noraw,
};

//#[allow(unused_imports)]  // XXX: clippy wrongly marks this as unused
#[cfg(test)]
use std::io::Write;  // for `NamedTempFile.write_all`

#[cfg(test)]
extern crate tempfile;
#[cfg(test)]
pub use tempfile::NamedTempFile;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// helper functions - misc.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// testing helper to write a `str` to a temporary file
/// The temporary file will be automatically deleted when returned `NamedTempFile` is dropped.
#[cfg(test)]
pub fn create_temp_file(content: &str) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(content.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf1
}

/// testing helper to write a `[u8]` to a temporary file
/// The temporary file will be automatically deleted when returned `NamedTempFile` is dropped.
#[cfg(test)]
pub fn create_temp_file_bytes(content: &[u8]) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(content) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf1
}

/// helper to print the raw and noraw version of a file
#[cfg(test)]
pub fn eprint_file(path: &FPath) {
    let contents_file: String = match std::fs::read_to_string(path) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Error reading file {:?}\n{:?}", path, err);
            return;
        },
    };
    let contents_file_count: usize = contents_file.lines().count();
    let contents_file_noraw: String = str_to_String_noraw(contents_file.as_str());
    eprintln!(
        "contents_file {:?} ({} lines):\n────────────────────────────────────────\n{}\n────────────────────────────────────────\n{}\n────────────────────────────────────────\n",
        path, contents_file_count,
        contents_file_noraw,
        contents_file,
    );
}
