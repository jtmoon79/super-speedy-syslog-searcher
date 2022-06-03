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
pub use tempfile::{NamedTempFile, tempdir};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// temporary file helper functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// testing helper to write a `str` to a temporary file.
///
/// The temporary file will be automatically deleted when returned `NamedTempFile`
/// is dropped.
#[cfg(test)]
pub fn create_temp_file(data: &str) -> NamedTempFile {
    let mut ntf = match NamedTempFile::new() {
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
#[cfg(test)]
pub fn create_temp_file_with_name_rlen(
    data: &str,
    prefix: Option<String>,
    suffix: Option<String>,
    rand_len: usize,
) -> NamedTempFile {
    let mut ntf = match tempfile::Builder::new()
        .prefix::<str>(prefix.unwrap_or_else(|| String::from(".tmp")).as_ref())
        .suffix::<str>(suffix.unwrap_or_else(|| String::from("")).as_ref())
        .rand_bytes(rand_len)
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
#[cfg(test)]
pub fn create_temp_file_with_name(
    data: &str,
    prefix: Option<String>,
    suffix: Option<String>,
) -> NamedTempFile {
    // XXX: tempfile::NUM_RAND_CHARS is not pub
    create_temp_file_with_name_rlen(data, prefix, suffix, 5)
}

/// testing helper to write a `str` to a exactly-named temporary file.
#[cfg(test)]
pub fn create_temp_file_with_name_exact(
    data: &str,
    name: String
) -> NamedTempFile {
    create_temp_file_with_name_rlen(data, Some(name), None, 0)
}

/// wrapper for `create_temp_file`, unwraps the result to an `FPath`.
#[cfg(test)]
pub fn create_temp_file_path(data: &str) -> FPath {
    let ntf = create_temp_file(data);

    FPath::from(ntf.path().to_str().unwrap())
}

/// testing helper to write a `[u8]` to a temporary file.
///
/// The temporary file will be automatically deleted when returned `NamedTempFile`
/// is dropped.
#[cfg(test)]
pub fn create_temp_file_bytes(data: &[u8]) -> NamedTempFile {
    let mut ntf1 = match NamedTempFile::new() {
        Ok(val) => val,
        Err(err) => {
            panic!("NamedTempFile::new() return Err {}", err);
        }
    };
    match ntf1.write_all(data) {
        Ok(_) => {}
        Err(err) => {
            panic!("NamedTempFile::write_all() return Err {}", err);
        }
    }

    ntf1
}

/// small helper, wraps call to `create_temp_file_bytes`, unwraps the result to
/// an `FPath`.
#[cfg(test)]
pub fn create_temp_file_bytes_path(data: &[u8]) -> FPath {
    let ntf = create_temp_file_bytes(data);

    FPath::from(ntf.path().to_str().unwrap())
}

/// small helper for copying `NamedTempFile` path
#[cfg(test)]
pub fn NTF_Path(ntf: &NamedTempFile) -> FPath {
    FPath::from(ntf.path().to_str().unwrap())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// testing helper to print the raw and noraw version of a file
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
