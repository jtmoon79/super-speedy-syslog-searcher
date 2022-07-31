// printer_debug/helpers.rs
//

use crate::common::{
    FPath,
};

use crate::printer_debug::printers::{
    str_to_String_noraw,
};

#[allow(unused_imports)]  // XXX: clippy wrongly marks this as unused
use std::io::Write;  // for `NamedTempFile.write_all`

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate tempfile;
pub use tempfile::{
    NamedTempFile,
    tempdir,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// temporary file helper functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

//#[cfg(test)]
lazy_static! {
    static ref STRING_TEMPFILE_PREFIX: String = String::from("tmp-s4-test-");
    // there is no `String::default` so create this just once
    static ref STRING_TEMPFILE_SUFFIX: String = String::from("");
}

/// small helper for copying `NamedTempFile` path to a `FPath`
//#[cfg(test)]
// TODO: rename `NTF_FPath`
pub fn NTF_Path(ntf: &NamedTempFile) -> FPath {
    FPath::from(ntf.path().to_str().unwrap())
}

/// testing helper to write a `str` to a temporary file.
///
/// BUG: `NamedTempFile` created within `lazy_static` will fail to remove itself
///      https://github.com/Stebalien/tempfile/issues/183
///
//#[cfg(test)]
pub fn create_temp_file(data: &str) -> NamedTempFile {
    let mut ntf = match tempfile::Builder::new()
        // use known prefix for easier cleanup
        .prefix::<str>(&STRING_TEMPFILE_PREFIX)
        .tempfile()
    {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf.write_all(data.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf
}

/// testing helper to write a `str` to a specially-named temporary file.
/// `rand_len` is the sting length of a random character sequence
//#[cfg(test)]
pub fn create_temp_file_with_name_rlen(
    data: &[u8],
    prefix: Option<&String>,
    suffix: Option<&String>,
    rand_len: usize,
) -> NamedTempFile {
    let mut ntf = match tempfile::Builder::new()
        .prefix::<str>(prefix.unwrap_or(&STRING_TEMPFILE_PREFIX).as_ref())
        .suffix::<str>(suffix.unwrap_or(&STRING_TEMPFILE_SUFFIX).as_ref())
        .rand_bytes(rand_len)
        .tempfile()
    {
        Ok(val) => val,
        Err(err) => {
            panic!("tempfile::Builder::new()..tempfile() return Err {}", err);
        }
    };
    match ntf.write_all(data) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf
}

/// testing helper to write a `str` to a specially-named temporary file.
//#[cfg(test)]
pub fn create_temp_file_with_name(
    data: &str,
    prefix: Option<&String>,
    suffix: Option<&String>,
) -> NamedTempFile {
    // XXX: tempfile::NUM_RAND_CHARS is not pub
    create_temp_file_with_name_rlen(data.as_bytes(), prefix, suffix, 5)
}

/// testing helper to write a `str` to a temporary file with a specific suffix
//#[cfg(test)]
pub fn create_temp_file_with_suffix(
    data: &str,
    suffix: &String
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data.as_bytes(), None, Some(suffix), 5)
}

/// testing helper to write a `str` to a exactly-named temporary file.
//#[cfg(test)]
pub fn create_temp_file_with_name_exact(
    data: &str,
    name: &String
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data.as_bytes(), Some(name), None, 0)
}

/// testing helper to write a `[u8]` to a temporary file.
//#[cfg(test)]
pub fn create_temp_file_bytes(data: &[u8]) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, None, None, 5)
}

/// testing helper to write a `[u8]` to a temporary file.
//#[cfg(test)]
pub fn create_temp_file_bytes_with_suffix(data: &[u8], suffix: &String) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, None, Some(suffix), 5)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// testing helper to print the raw and noraw version of a file
///
/// only intended to help humans reading stderr output
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
