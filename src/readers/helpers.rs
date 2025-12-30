// src/readers/helpers.rs

//! Miscellaneous helper functions for _Readers_.

use std;

#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
};
#[cfg(test)]
use rand;
#[cfg(test)]
use rand::seq::SliceRandom; // brings in `shuffle`

#[cfg(test)]
use crate::common::FileOffset;
use crate::common::{
    FPath,
    FileSz,
};

/// Return the basename of an `FPath`.
pub fn basename(path: &FPath) -> FPath {
    let mut riter = path.rsplit(std::path::MAIN_SEPARATOR);

    FPath::from(riter.next().unwrap_or(""))
}

/// Helper function for a slightly annoying set of calls.
pub fn path_to_fpath(path: &std::path::Path) -> FPath {
    // `PathBuf` to `String` https://stackoverflow.com/q/37388107/471376
    (*(path.to_string_lossy())).to_string()
}

/// Helper function for completeness.
pub fn fpath_to_path(path: &FPath) -> &std::path::Path {
    std::path::Path::new(path)
}

/// Helper function for a somewhat non-obvious expression.
pub fn path_clone(path: &std::path::Path) -> &std::path::Path {
    std::path::Path::new(path.as_os_str())
}

/// Return the size of the file.
pub fn path_filesz(path: &std::path::Path) -> Option<FileSz> {
    defn!("({:?})", path);
    let metadata = match std::fs::metadata(path) {
        Ok(val) => val,
        Err(_err) => {
            defx!("error {}, return None", _err);
            return None;
        }
    };
    let len: FileSz = metadata.len();
    defx!("return {}", len);

    Some(len)
}

/// wrapper for call to `path_filesz`
#[macro_export]
macro_rules! path_filesz_or_return_err {
    ($path: expr) => {{
        {
            match path_filesz($path) {
                Some(val) => val,
                None => {
                    defx!("path_filesz() returned None for {:?}", $path);
                    return Err(Error::new(ErrorKind::Other, format!("path_filesz() returned None for {:?}", $path)));
                }
            }
        }
    }};
}

/// Count instances of a particular `c` in `s`.
pub fn count_char_in_str(
    s: &str,
    c: char,
) -> usize {
    s.chars().filter(|x| *x == c).count()
}

/// Count number of file extensions in the file name, e.g. count `'.'`.
pub fn filename_count_extensions(path: &std::path::Path) -> usize {
    let file_name_osstr: &std::ffi::OsStr = match path.file_name() {
        Some(val) => val,
        None => {
            return 0;
        }
    };
    let file_name = file_name_osstr.to_string_lossy();

    count_char_in_str(&file_name, '.')
}

/// Testing helper.
#[doc(hidden)]
#[cfg(test)]
pub fn randomize(v_: &mut [FileOffset]) {
    let mut rng = rand::rng();
    v_.shuffle(&mut rng);
}

/// Testing helper.
#[doc(hidden)]
#[cfg(test)]
pub fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}
