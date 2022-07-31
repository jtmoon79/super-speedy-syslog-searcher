// Readers/helpers.rs
//
// miscellaneous helper functions

use crate::common::{
    FileOffset,
    FPath,
};

use std;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

//// return basename of an `FPath`
pub fn basename(path: &FPath) -> FPath {
    let mut riter = path.rsplit(std::path::MAIN_SEPARATOR);

    FPath::from(riter.next().unwrap_or(""))
}

/// helper for a slightly annoying set of calls
pub fn path_to_fpath(path: &std::path::Path) -> FPath {
    // `PathBuf` to `String` https://stackoverflow.com/q/37388107/471376
    (*(path.to_string_lossy())).to_string()
}

/// helper for a slightly annoying set of calls
pub fn fpath_to_path(path: &FPath) -> &std::path::Path {
    std::path::Path::new(path)
}

/// count of `c` in `s`
pub fn count_chars_in_str(s: &str, c: char) -> usize {
    let mut count: usize = 0;
    for c_ in s.chars() {
        if c_ == '.' {
            count += 1;
        }
    }

    count
}

/// count number of file extensions in the file name, e.g. `'.'`
pub fn filename_count_extensions(path: &std::path::Path) -> usize {
    let file_name_osstr: &std::ffi::OsStr = match path.file_name() {
        Some(val) => val,
        None => {
            return 0;
        }
    };
    let file_name = file_name_osstr.to_string_lossy();

    count_chars_in_str(&file_name, '.')
}

/// remove the extension from a `Path`.
///
/// if no extension (no `'.'`) or other problems then return `None`
///
/// XXX: not efficient
pub fn remove_extension(path: &std::path::Path) -> Option<FPath> {
    let file_name: &std::ffi::OsStr = path.file_name()?;
    let file_name_str: &str = file_name.to_str()?;
    let index: usize = file_name_str.rfind('.')?;
    let name_new: &str = &file_name_str[..index];
    let name_new_path: &std::path::Path = std::path::Path::new(name_new);
    let dir_name: &std::path::Path = path.parent()?;
    let pathbuf2 = dir_name.join(name_new_path);
    let path2: &std::path::Path = pathbuf2.as_path();

    Some(path_to_fpath(path2))
}

/// testing helper
pub fn randomize(v_: &mut Vec<FileOffset>) {
    // XXX: can also use `rand::shuffle` https://docs.rs/rand/0.8.4/rand/seq/trait.SliceRandom.html#tymethod.shuffle
    let sz = v_.len();
    let mut i = 0;
    while i < sz {
        let r = rand::random::<usize>() % sz;
        v_.swap(r, i);
        i += 1;
    }
}

/// testing helper
pub fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}
