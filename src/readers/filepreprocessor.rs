// src/readers/filepreprocssor.rs

//! A collection of functions to search for potentially parseable files,
//! and prepare data needed to create a [`SyslogProcessor`] instance or other
//! "Reader" instance for file processing.
//!
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor

use crate::common::{
    FileType,
    FileTypeArchive,
    FileTypeFixedStruct,
    FileTypeTextEncoding,
    FPath,
};
use crate::debug::printers::de_err;
use crate::readers::blockreader::SUBPATH_SEP;
use crate::readers::helpers::path_to_fpath;
#[cfg(test)]
use crate::readers::helpers::fpath_to_path;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
#[cfg(test)]
use std::str::FromStr; // for `String::from_str`

use ::jwalk;
#[allow(unused_imports)]
use ::si_trace_print::{defn, defo, defx, defñ, den, deo, dex, deñ};
use ::tar;


// ----------------
// FilePreProcessor

/// Initial path processing return type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessPathResult {
    /// File can be processed by `s4`
    FileValid(FPath, FileType),
    // TODO: [2022/06] `FileErrNoPermissions` not currently checked until too late
    /// Filesystem permissions do not allow reading the file
    FileErrNoPermissions(FPath),
    /// File is a known or unknown type and is not supported
    FileErrNotSupported(FPath),
    /// Path exists and is not a file
    FileErrNotAFile(FPath),
    /// Path does not exist
    FileErrNotExist(FPath),
    /// Error loading necessary shared libraries
    FileErrLoadingLibrary(FPath, &'static str, FileType),
    /// All other errors as described in the second parameter message
    FileErr(FPath, String),
}

/// a multi-file storage format
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileTypeArchiveMultiple {
    /// a `.tar` archive file
    Tar,
}

#[cfg(test)]
/// Helper to `copy_process_path_result_canonicalize_path`
fn canonicalize_fpath(fpath: &FPath) -> FPath {
    let path: &Path = fpath_to_path(fpath);
    match path.canonicalize() {
        Ok(pathbuf) => {
            let s = FPath::from_str(pathbuf.to_str().unwrap());
            return s.unwrap();
        }
        Err(_) => {
            // best effort: return the value passed-in
            return fpath.clone();
        }
    }
}

#[cfg(test)]
/// Test helper to canonicalize the path contained by `ProcessPathResult`
///
/// Some Windows hosts return the MS-DOS shortened form of a path.
/// Later, that will fail comparisons to the canonical full form of the same
/// path.
/// e.g. `"C:\\Users\\RUNNER~1\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
///      !=
///      `"C:\\Users\\runneradmin\\AppData\\Local\\Temp\\.tmp6TC2W5\\file1"`
/// This function should resolve the first string to the second string.
pub(crate) fn copy_process_path_result_canonicalize_path(ppr: ProcessPathResult) -> ProcessPathResult {
    match ppr {
        ProcessPathResult::FileValid(fpath, f) => {
            let fpath_c = canonicalize_fpath(&fpath);
            return ProcessPathResult::FileValid(fpath_c, f);
        }
        ProcessPathResult::FileErrNoPermissions(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);
            return ProcessPathResult::FileErrNoPermissions(fpath_c);
        }
        ProcessPathResult::FileErrNotSupported(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);
            return ProcessPathResult::FileErrNotSupported(fpath_c);
        }
        ProcessPathResult::FileErrNotAFile(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);
            return ProcessPathResult::FileErrNotAFile(fpath_c);
        }
        ProcessPathResult::FileErrNotExist(fpath) => {
            let fpath_c = canonicalize_fpath(&fpath);
            return ProcessPathResult::FileErrNotExist(fpath_c);
        }
        ret => {
            return ret;
        }
    }
}

pub type ProcessPathResults = Vec<ProcessPathResult>;

/// returned by `pathbuf_to_filetype`, if `filetype` is set then
/// it is the `FileType` of the file at `pathbuf`
/// else an multi-file `archive`` type was encountered and must be processed
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathToFiletypeResult {
    Filetype(FileType),
    Archive(FileTypeArchiveMultiple),
}

/// Determine the `FileType` of a file based on the `pathbuf` file name.
pub fn pathbuf_to_filetype(pathbuf: &PathBuf, unparseable_are_text: bool) -> PathToFiletypeResult {
    defn!("({:?}, {:?})", pathbuf, unparseable_are_text);

    const RET_FALLBACK_TEXT: PathToFiletypeResult =
        PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: FileTypeArchive::Normal,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            });
    const RET_FALLBACK_UNPARSABLE: PathToFiletypeResult =
        PathToFiletypeResult::Filetype(FileType::Unparsable);

    defo!("pathbuf {:?}", pathbuf);
    // trim trailing junk characters from suffix
    const JUNK_CHARS: &[char] = &['~', '-', ',', '?'];
    let mut pathbuf_clean: &PathBuf = pathbuf;
    let fname = pathbuf.file_name().unwrap_or_default().to_str().unwrap_or_default();
    let pathbuf_ref: PathBuf;
    if fname.ends_with(JUNK_CHARS) {
        let fname2 = fname.trim_end_matches(JUNK_CHARS);
        if fname2.is_empty() {
            match unparseable_are_text {
                true => {
                    defx!("fname.ends_with(JUNK_CHARS) {:?}, return {:?}", fname, RET_FALLBACK_TEXT);
                    return RET_FALLBACK_TEXT;
                }
                false => {
                    defx!("fname.ends_with(JUNK_CHARS) {:?}, return {:?}", fname, RET_FALLBACK_UNPARSABLE);
                    return RET_FALLBACK_UNPARSABLE;
                }
            }
        }
        pathbuf_ref = pathbuf.with_file_name(fname2);
        pathbuf_clean = &pathbuf_ref;
        defo!("pathbuf_clean {:?}", pathbuf_clean);
    }

    let file_name: &OsStr = pathbuf_clean
        .file_name()
        .unwrap_or_default();
    defo!("file_name {:?}", file_name);

    if file_name.is_empty() {
        match unparseable_are_text {
            true => {
                defx!("file_name.is_empty() {:?}, return {:?}", file_name, RET_FALLBACK_TEXT);
                return RET_FALLBACK_TEXT;
            }
            false => {
                defx!("file_name.is_empty() {:?}, return {:?}", file_name, RET_FALLBACK_UNPARSABLE);
                return RET_FALLBACK_UNPARSABLE;
            }
        }
    }

    let file_name_string: String = file_name
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_name_s: &str = file_name_string.as_str();
    defo!("file_name_s {:?}", file_name_s);

    if file_name_s.is_empty() {
        match unparseable_are_text {
            true => {
                defx!("file_name_s.is_empty() {:?}, return {:?}", file_name_s, RET_FALLBACK_TEXT);
                return RET_FALLBACK_TEXT;
            }
            false => {
                defx!("file_name_s.is_empty() {:?}, return {:?}", file_name_s, RET_FALLBACK_UNPARSABLE);
                return RET_FALLBACK_UNPARSABLE;
            }
        }
    }

    // check for special case where symbolic directory name was passed
    // this probably should never happen but handle it anyway
    if !file_name_s.is_empty() && file_name_s.chars().all(|c| c == '.') {
        de_err!("file_name_s {:?} is all '.'", file_name_s);
        match unparseable_are_text {
            true => {
                defx!("file_name_s {:?}, return {:?}", file_name_s, RET_FALLBACK_TEXT);
                return RET_FALLBACK_TEXT;
            }
            false => {
                defx!("file_name_s {:?}, return {:?}", file_name_s, RET_FALLBACK_UNPARSABLE);
                return RET_FALLBACK_UNPARSABLE;
            }
        }
    }

    match file_name_s {
        "dmesg"
        | "history"
        | "kernellog"
        | "kernelog"
        | "kernlog"
        | "log"
        | "messages"
        | "syslog"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: FileTypeArchive::Normal,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "acct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Acct,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "pacct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::AcctV3,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "lastlog"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Lastlog,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "lastlogx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Lastlogx,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "btmp"
        | "utmp"
        | "wtmp"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Utmp,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "btmpx"
        | "utmpx"
        | "wtmpx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Utmpx,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        _ => {
            defo!("file_name_s {:?} not matched", file_name_s);
        }
    }

    let file_suffix_s: &str = pathbuf_clean
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let file_suffix: String = file_suffix_s.to_ascii_lowercase();
    defo!("file_suffix {:?}", file_suffix);

    if file_suffix.parse::<i32>().is_ok() {
        defo!("file_suffix {:?} is a number", file_suffix);
        // remove the file suffix/extension and call again
        return pathbuf_to_filetype(&pathbuf_clean.with_extension(""), unparseable_are_text);
    }

    match file_suffix.as_str() {
        "evtx" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Evtx {
                    archival_type: FileTypeArchive::Normal,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "journal" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Journal {
                    archival_type: FileTypeArchive::Normal,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "gz" | "gzip" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: FileTypeArchive::Gz,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "tar" => {
            let ret = PathToFiletypeResult::Archive(FileTypeArchiveMultiple::Tar);
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "xz" | "xzip" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: FileTypeArchive::Xz,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "log"
        | "txt"
        | "text"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: FileTypeArchive::Normal,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "btmp"
        | "utmp"
        | "wtmp"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Utmp,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "btmpx"
        | "utmpx"
        | "wtmpx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Utmpx,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "lastlog"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Lastlog,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "lastlogx"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Lastlogx,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "acct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::Acct,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "pacct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: FileTypeArchive::Normal,
                    fixedstruct_type: FileTypeFixedStruct::AcctV3,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // known unparseable file extensions/suffixes
        // covers some common extensions. not exhaustive but still helpful
        "aac"
        | "avi"
        | "bat"
        | "bmp"
        | "bz"
        | "bz2"
        | "class"
        | "cmd"
        | "dll"
        | "ear"
        | "exe"
        | "flac"
        | "flv"
        | "gif"
        | "ico"
        | "jar"
        | "java"
        | "jpeg"
        | "jpg"
        | "lib"
        | "lzma"
        | "m4b"
        | "m4p"
        | "m4r"
        | "m4v"
        | "mkv"
        | "mov"
        | "mp3"
        | "mp4"
        | "msi"
        | "ogg"
        | "opus"
        | "pl"
        | "png"
        | "py"
        | "rb"
        | "sh"
        | "so"
        | "svg"
        | "tif"
        | "tiff"
        | "tgz"
        | "war"
        | "wav"
        | "webm"
        | "webp"
        | "wma"
        | "wmv"
        | "zip"
        => {
            match unparseable_are_text {
                true => {
                    defx!("matched file_suffix {:?}; return fallback FileType::Text", file_suffix);
                    return RET_FALLBACK_TEXT;
                }
                false => {
                    defx!("matched file_suffix {:?}; return FileType::Unparsable", file_suffix);
                    return RET_FALLBACK_UNPARSABLE;
                }
            }
        }
        _ => {
            defo!("file_suffix {:?} not matched", file_suffix);
        }
    }

    if !file_suffix.is_empty() {
        defo!("file_suffix {:?} not empty", file_suffix);
        // remove the file suffix/extension and call again
        let ret = pathbuf_to_filetype(
            &pathbuf_clean.with_extension(""),
            unparseable_are_text
        );
        defx!("return {:?}", ret);
        return ret;
    }

    // getting here means there is no suffix/extension in the file name

    // for example, `log_media`
    if file_name_s.starts_with("log_") {
        let ret = PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: FileTypeArchive::Normal,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            }
        );
        defx!("starts_with('log_') {:?}; return {:?}", file_name_s, ret);
        return ret;
    }
    // for example, `media_log`
    if file_name_s.ends_with("_log") {
        let ret = PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: FileTypeArchive::Normal,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            }
        );
        defx!("ends_with('_log') {:?}; return {:?}", file_name_s, ret);
        return ret;
    }

    // not sure what the file is so try to process it as a plain text file

    let ret = PathToFiletypeResult::Filetype(
        FileType::Text {
            archival_type: FileTypeArchive::Normal,
            encoding_type: FileTypeTextEncoding::Utf8Ascii
        }
    );
    defx!("return {:?}", ret);

    ret
}

/// Wrapper function for [`pathbuf_to_filetype`]
pub fn path_to_filetype(path: &Path, unparseable_are_text: bool) -> PathToFiletypeResult {
    pathbuf_to_filetype(&PathBuf::from(path), unparseable_are_text)
}

/// Wrapper function for [`path_to_filetype`]
#[cfg(test)]
pub fn fpath_to_filetype(path: &FPath, unparseable_are_text: bool) -> PathToFiletypeResult {
    path_to_filetype(fpath_to_path(path), unparseable_are_text)
}

/// Is the `FileType` processing implemented by `s4lib`?
///
/// There are plans for future support of differing files.
// TODO: [2024/04] should use this function, right?
pub fn processable_filetype(filetype: &FileType) -> bool {
    defñ!("({:?})", filetype);
    match filetype {
        FileType::Evtx{ archival_type: FileTypeArchive::Normal } => true,
        FileType::Evtx{ archival_type: FileTypeArchive::Gz } => false,
        FileType::Evtx{ archival_type: FileTypeArchive::Tar } => false,
        FileType::Evtx{ archival_type: FileTypeArchive::Xz } => false,
        FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ } => true,
        FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ } => false,
        FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ } => false,
        FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ } => false,
        FileType::Journal{ archival_type: FileTypeArchive::Normal } => true,
        FileType::Journal{ archival_type: FileTypeArchive::Gz } => false,
        FileType::Journal{ archival_type: FileTypeArchive::Tar } => false,
        FileType::Journal{ archival_type: FileTypeArchive::Xz } => false,
        FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: _ } => true,
        FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: _ } => false,
        FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: _ } => false,
        FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: _ } => false,
        FileType::Unparsable => false,
    }
}

/// Helper to `process_path_tar`
fn error_to_string(error: &std::io::Error, path: &FPath) -> String {
    String::from(
        format!(
            "{}: {} for file {:?}",
            error.kind(), error.to_string(), path,
        )
    )
}

/// Return a `ProcessPathResult` for each parseable file within
/// the `.tar` file at `path`.
pub fn process_path_tar(path: &FPath, unparseable_are_text: bool) -> Vec<ProcessPathResult> {
    defn!("({:?}, {:?})", path, unparseable_are_text);

    let file: File = File::open(path).unwrap();
    let mut archive: tar::Archive<File> = tar::Archive::<File>::new(file);
    let entry_iter: tar::Entries<File> = match archive.entries() {
        Ok(val) => val,
        Err(err) => {
            defx!("return FileErr; {:?}", err);
            let err_string = error_to_string(&err, path);
            return vec![ProcessPathResult::FileErr(path.clone(), err_string)];
        }
    };
    let mut results = Vec::<ProcessPathResult>::new();
    for entry_res in entry_iter {
        let entry: tar::Entry<File> = match entry_res {
            Ok(val) => val,
            Err(err) => {
                deo!("entry Err {:?}", err);
                let err_string = error_to_string(&err, path);
                results.push(ProcessPathResult::FileErr(path.clone(), err_string ));
                continue;
            }
        };
        let header: &tar::Header = entry.header();
        let etype: tar::EntryType = header.entry_type();
        // TODO: handle tar types `symlink` and `long_link`, currently they are ignored
        if !etype.is_file() {
            deo!("entry.header().entry_type() {:?} (IGNORED)", etype);
            continue;
        }
        deo!("entry.header().entry_type() {:?}", etype);
        let subpath: Cow<Path> = match entry.path() {
            Ok(val) => val,
            Err(err) => {
                deo!("entry.path() Err {:?}", err);
                let err_string = error_to_string(&err, path);
                results.push(ProcessPathResult::FileErr(path.clone(), err_string ));
                continue;
            }
        };
        // first get the `FileType` of the subpath
        let subfpath: FPath = subpath
            .to_string_lossy()
            .to_string();
        let _filetype_subpath = path_to_filetype(&subpath, unparseable_are_text);
        // path to a file within a .tar file looks like:
        //
        //     "path/file.tar|subpath/subfile"
        //
        // where `path/file.tar` are on the host filesystem, and `subpath/subfile` are within
        // the `.tar` file
        let mut fullpath: FPath =
            String::with_capacity(path.len() + SUBPATH_SEP.len_utf8() + subfpath.len() + 1);
        fullpath.push_str(path.as_str());
        fullpath.push(SUBPATH_SEP);
        fullpath.push_str(subfpath.as_str());
        // XXX: force filetype to be `Tar` (ignore `_filetype_subpath`). Later an attempt
        //      will be made to parse it.
        //      Chained reads are not supported. See Issue #14
        let filetype_ = FileType::Text { archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf8Ascii };
        deo!("push FileValid({:?}, {:?})", fullpath, filetype_);
        results.push(ProcessPathResult::FileValid(fullpath, filetype_));
    }
    defx!("process_path_tar({:?}) {} results", path, results.len());

    results
}

/// Return all parseable files in the `path`.
///
/// Given a directory, recurses the directory.<br/>
/// Given a plain file path, returns that path.<br/>
/// For each recursed file, checks if file is parseable (correct file type,
/// appropriate permissions).<br/>
/// For archive files, `.tar`, enumerates files within the archive.<br/>
/// For compressed files, `.gz` `.xz`, presumes they hold only one file
/// (Relates to Issue #11, Issue #8).
///
/// This behavior assumes a user-passed file path should attempt to be parsed.
pub fn process_path(path: &FPath, unparseable_are_text: bool) -> Vec<ProcessPathResult> {
    defn!("({:?}, {:?})", path, unparseable_are_text);

    let mut std_path: PathBuf = PathBuf::from(path);

    deo!("std_path {:?}", std_path);

    std_path = match std_path.canonicalize() {
        Ok(val) => val,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    defx!("return FileErrNotExist({:?})", path);
                    return vec![ProcessPathResult::FileErrNotExist(path.clone())];
                }
                _ => {
                    let err_string = error_to_string(&err, path);
                    defx!("return FileErr({:?}, {:?})", path, err_string);
                    return vec![ProcessPathResult::FileErr(path.clone(), err_string)];
                }
            }
        }
    };
    deo!("std_path {:?}", std_path);

    if std_path.is_file() {
        // if passed a path to a file (or a symlink to a plain file)
        // then assume the user wants to force an attempt to process
        // such a file (even if it's known to be unparsable, e.g. `picture.png`)
        // so pass `unparseable_are_text` as `true` so any unknown or unparseable
        // are treated as `FileType::Text`
        let result: PathToFiletypeResult = pathbuf_to_filetype(&std_path, true);
        match result {
            PathToFiletypeResult::Filetype(filetype) => {
                debug_assert!(!filetype.is_archived());
                let paths: Vec<ProcessPathResult> =
                    vec![ProcessPathResult::FileValid(path.clone(), filetype)];
                defx!("({:?}) {:?}", path, paths);
                return paths;
            }
            PathToFiletypeResult::Archive(_archive) => {
                // getting here means `std_path` is a `.tar` file
                let results = process_path_tar(path, unparseable_are_text);
                defx!("return process_path_tar({:?}) returned {} results", path, results.len());
                return results;
            }
        }
    }

    // getting here means `path` likely refers to a directory

    let mut paths: Vec<ProcessPathResult> = Vec::<ProcessPathResult>::new();

    deo!("jwalk::rayon::current_num_threads = {}", jwalk::rayon::current_num_threads());
    deo!("jwalk::rayon::max_num_threads = {}", jwalk::rayon::max_num_threads());

    deo!("WalkDir({:?})…", path);
    for entry in jwalk::WalkDir::new(path.as_str())
        .follow_links(true)
        .sort(true)
    {
        // XXX: what is type `T` in `Result<T, E>` returned by `WalkDir`?
        let path_entry = match entry {
            Ok(val) => {
                deo!("Ok({:?})", val);
                val
            }
            Err(_err) => {
                deo!("Err({:?})", _err);
                continue;
            }
        };

        deo!("analayzing {:?}", path_entry);
        let std_path_entry: &Path = &path_entry.path();
        let fpath_entry: FPath = path_to_fpath(std_path_entry);
        if !path_entry
            .file_type()
            .is_file()
        {
            if path_entry
                .file_type()
                .is_dir()
            {
                continue;
            }
            deo!("Path not a file {:?}", path_entry);
            paths.push(ProcessPathResult::FileErrNotAFile(fpath_entry));
            continue;
        }
        let result: PathToFiletypeResult = path_to_filetype(std_path_entry, false);
        defo!("path_to_filetype({:?}) returned {:?}", std_path_entry, result);
        let filetype: FileType = match result {
            PathToFiletypeResult::Filetype(filetype) => filetype,
            PathToFiletypeResult::Archive(_archive) => {
                // getting here means `std_path_entry` is a `.tar` file
                let results = process_path_tar(&fpath_entry, unparseable_are_text);
                for result in results.into_iter() {
                    paths.push(result);
                }
                continue;
            }
        };
        match filetype {
            FileType::Evtx{ archival_type: FileTypeArchive::Normal }
            | FileType::FixedStruct{ archival_type: _, fixedstruct_type: _ }
            | FileType::Journal{ archival_type: FileTypeArchive::Normal }
            | FileType::Text{ archival_type: _, encoding_type: _ }
            => {
                deo!("paths.push(FileValid(({:?}, {:?})))", fpath_entry, filetype);
                paths.push(ProcessPathResult::FileValid(fpath_entry, filetype));
            }
            FileType::Evtx{ archival_type: FileTypeArchive::Gz }
            | FileType::Evtx{ archival_type: FileTypeArchive::Tar }
            | FileType::Evtx{ archival_type: FileTypeArchive::Xz }
            | FileType::Journal{ archival_type: FileTypeArchive::Gz }
            | FileType::Journal{ archival_type: FileTypeArchive::Tar }
            | FileType::Journal{ archival_type: FileTypeArchive::Xz }
            | FileType::Unparsable => {
                deo!("Path not supported {:?}", std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(fpath_entry));
            }
        }
    }
    defx!("return {:?}", paths);

    paths
}
