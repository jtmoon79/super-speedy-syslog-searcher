// src/debug/helpers.rs

//! Miscellaneous helper functions for debug printing and testing.

use crate::common::FPath;

#[allow(unused_imports)] // XXX: clippy wrongly marks this as unused
use std::io::Write; // for `NamedTempFile.write_all`

extern crate lazy_static;
use lazy_static::lazy_static;

extern crate tempfile;

#[doc(hidden)]
pub use tempfile::NamedTempFile;

#[doc(hidden)]
pub use tempfile::tempdir;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// temporary file helper functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// NamedTempFile instances default to this file name prefix.
///
/// Used by helper script `tools/rust-test.sh` to delete temporary files
/// remaining after testing.
/// See <https://github.com/Stebalien/tempfile/issues/183>.
pub const STR_TEMPFILE_PREFIX: &str = "tmp-s4-test-";

lazy_static! {
    pub static ref STRING_TEMPFILE_PREFIX: String = String::from(STR_TEMPFILE_PREFIX);
    // there is no `String::default` so create this just once
    static ref STRING_TEMPFILE_SUFFIX: String = String::from("");
}

/// Small helper function for copying `NamedTempFile` path to a `FPath`.
pub fn ntf_fpath(ntf: &NamedTempFile) -> FPath {
    FPath::from(ntf.path().to_str().unwrap())
}

/// Testing helper function to write a `str` to a temporary file.
///
/// BUG: `NamedTempFile` created within `lazy_static` will fail to remove itself
///      <https://github.com/Stebalien/tempfile/issues/183>.
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

/// Testing helper function to write a `[u8]` to a specially-named
/// temporary file.
///
/// `rand_len` is the string length of a random character sequence
pub fn create_temp_file_with_name_rlen(
    data: &[u8],
    prefix: Option<&String>,
    suffix: Option<&String>,
    rand_len: usize,
) -> NamedTempFile {
    let mut ntf = match tempfile::Builder::new()
        .prefix::<str>(
            prefix
                .unwrap_or(&STRING_TEMPFILE_PREFIX)
                .as_ref(),
        )
        .suffix::<str>(
            suffix
                .unwrap_or(&STRING_TEMPFILE_SUFFIX)
                .as_ref(),
        )
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

/// Testing helper function to write a `str` to a specially-named
/// temporary file.
pub fn create_temp_file_with_name(
    data: &str,
    prefix: Option<&String>,
    suffix: Option<&String>,
) -> NamedTempFile {
    // XXX: tempfile::NUM_RAND_CHARS is not pub
    create_temp_file_with_name_rlen(data.as_bytes(), prefix, suffix, 5)
}

/// Testing helper function to write a `str` to a temporary file with a specific
/// suffix
pub fn create_temp_file_with_suffix(
    data: &str,
    suffix: &String,
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data.as_bytes(), None, Some(suffix), 5)
}

/// Testing helper function to write a `str` to a exactly-named temporary file.
pub fn create_temp_file_with_name_exact(
    data: &str,
    name: &String,
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data.as_bytes(), Some(name), None, 0)
}

/// Testing helper function to write a `[u8]` to a temporary file.
pub fn create_temp_file_bytes(data: &[u8]) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, None, None, 5)
}

/// Testing helper function to write a `[u8]` to a temporary file.
pub fn create_temp_file_bytes_with_suffix(
    data: &[u8],
    suffix: &String,
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, None, Some(suffix), 5)
}
