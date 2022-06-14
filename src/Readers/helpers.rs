// Readers/helpers.rs
//
// miscellaneous helper functions

use std;

#[cfg(test)]
use crate::common::{
    FileOffset,
};

use crate::common::{
    FPath,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// helper for a slightly annoying set of calls
pub fn path_to_fpath(path: &std::path::Path) -> FPath {
    // `PathBuf` to `String` https://stackoverflow.com/q/37388107/471376
    (*(path.to_string_lossy())).to_string()
}

/// helper for a slightly annoying set of calls
pub fn fpath_to_path(path: &FPath) -> &std::path::Path {
    std::path::Path::new(path)
}

/// testing helper
#[cfg(test)]
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
#[cfg(test)]
pub fn fill(v_: &mut Vec<FileOffset>) {
    let sz = v_.capacity();
    let mut i = 0;
    while i < sz {
        v_.push(i as FileOffset);
        i += 1;
    }
}
