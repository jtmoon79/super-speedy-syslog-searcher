// src/readers/filepreprocssor.rs

//! A collection of functions to search for potentially parseable files,
//! and prepare data needed to create a [`SyslogProcessor`] instance or other
//! "Reader" instance for file processing.
//!
//! [`SyslogProcessor`]: crate::readers::syslogprocessor::SyslogProcessor

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
#[cfg(test)]
use std::str::FromStr; // for `String::from_str`

#[allow(unused_imports)]
use ::si_trace_print::{
    defn,
    defo,
    defx,
    defñ,
    den,
    deo,
    dex,
    deñ,
};
use {
    ::jwalk,
    ::tar,
};

use crate::common::{
    FPath,
    FileSz,
    FileType,
    FileTypeArchive,
    FileTypeFixedStruct,
    FileTypeTextEncoding,
    OdlSubType,
    SUBPATH_SEP,
};
use crate::debug::printers::de_err;
#[cfg(any(debug_assertions, test))]
use crate::readers::helpers::fpath_to_path;
use crate::readers::helpers::path_to_fpath;

// ----------------
// FilePreProcessor

/// Initial path processing return type.
#[derive(Clone, Debug, Eq)]
pub enum ProcessPathResult {
    /// File can be processed by `s4`
    FileValid(FPath, FileType),
    FileErrEmpty(FPath, FileType),
    FileErrTooSmall(FPath, FileType, FileSz),
    // TODO: [2022/06] `FileErrNoPermissions` not currently checked until too late
    /// Filesystem permissions do not allow reading the file
    FileErrNoPermissions(FPath),
    /// File is a known or unknown type and is not supported
    /// The second optional field is descriptive error
    FileErrNotSupported(FPath, Option<String>),
    /// Path exists and is not a file
    FileErrNotAFile(FPath),
    /// Path does not exist
    FileErrNotExist(FPath),
    /// Error loading necessary shared libraries
    FileErrLoadingLibrary(FPath, &'static str, FileType),
    /// All other errors as described in the second parameter message
    FileErr(FPath, String),
}

impl PartialEq for ProcessPathResult {
    /// implemented for comparisons in tests
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ProcessPathResult::FileValid(fpath1, f1),
                ProcessPathResult::FileValid(fpath2, f2)
            ) => {
                fpath1 == fpath2 && f1 == f2
            }
            (
                ProcessPathResult::FileErrNoPermissions(fpath1),
                ProcessPathResult::FileErrNoPermissions(fpath2)
            ) => {
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrNotSupported(fpath1, _),
                ProcessPathResult::FileErrNotSupported(fpath2, _)
            ) => {
                // ignore the optional message field
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrNotAFile(fpath1),
                ProcessPathResult::FileErrNotAFile(fpath2)
            ) => {
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrNotExist(fpath1),
                ProcessPathResult::FileErrNotExist(fpath2)
            ) => {
                fpath1 == fpath2
            }
            (
                ProcessPathResult::FileErrLoadingLibrary(fpath1, lib1, f1),
                ProcessPathResult::FileErrLoadingLibrary(fpath2, lib2, f2)
            ) => {
                fpath1 == fpath2 && lib1 == lib2 && f1 == f2
            }
            (
                ProcessPathResult::FileErr(fpath1, msg1),
                ProcessPathResult::FileErr(fpath2, msg2)
            ) => {
                fpath1 == fpath2 && msg1 == msg2
            }
            _ => false
        }
    }
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
        ProcessPathResult::FileErrNotSupported(fpath, message) => {
            let fpath_c = canonicalize_fpath(&fpath);
            return ProcessPathResult::FileErrNotSupported(fpath_c, message);
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
    Archive(FileTypeArchiveMultiple, FileTypeArchive),
}

/// Determine the `FileType` of a file based on the `pathbuf` file name.
fn pathbuf_to_filetype_impl(
    pathbuf: &PathBuf,
    unparseable_are_text: bool,
    filetype_archive: Option<FileTypeArchive>,
) -> PathToFiletypeResult {
    defn!("({:?}, {:?}, {:?})", pathbuf, unparseable_are_text, filetype_archive);

    let fta: FileTypeArchive = match filetype_archive {
        Some(val) => val,
        None => FileTypeArchive::Normal,
    };

    #[allow(non_snake_case)]
    let RET_FALLBACK_TEXT: PathToFiletypeResult =
        PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: fta,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            });
    const RET_FALLBACK_UNPARSABLE: PathToFiletypeResult =
        PathToFiletypeResult::Filetype(FileType::Unparsable);

    defo!("pathbuf       {:?}", pathbuf);
    // trim trailing junk characters from suffix
    const JUNK_CHARS: &[char] = &['~', '-', ',', '?', ';'];
    let mut pathbuf_clean: &PathBuf = pathbuf;
    let mut file_name: &OsStr = pathbuf_clean
        .file_name()
        .unwrap_or_default();
    defo!("file_name     {:?}", file_name);
    let fname = file_name.to_str().unwrap_or_default();
    let mut pathbuf_ref: PathBuf;
    if fname.ends_with(JUNK_CHARS) {
        defo!("trailing junk characters {:?}", fname);
        let fname2 = fname.trim_end_matches(JUNK_CHARS);
        defo!("fname2        {:?}", fname2);
        if fname2.is_empty() {
            // the file name is all junk characters so return a fallback
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
        file_name = pathbuf_clean
            .file_name()
            .unwrap_or_default();
        defo!("file_name     {:?}", file_name);
    }

    // check for special case where symbolic directory, `.` or `..`, name was passed
    // this probably should never get here but handle it anyway
    let file_name_s = file_name
        .to_str()
        .unwrap_or_default();
    if !file_name_s.is_empty()
        && file_name_s
            .chars()
            .all(|c| c == '.')
    {
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

    // trim leading junk characters from file_name
    const JUNK_CHARS_LEAD: &[char] = &['~', '-', ',', '?', ';', '.'];
    let fname_ = file_name.to_str().unwrap_or_default();
    if fname_.starts_with(JUNK_CHARS_LEAD) {
        defo!("leading junk characters {:?}", fname_);
        let fname2 = fname_.trim_start_matches(JUNK_CHARS_LEAD);
        defo!("fname2        {:?}", fname2);
        if fname2.is_empty() {
            // the file name is all junk characters so return a fallback
            match unparseable_are_text {
                true => {
                    defx!("fname.starts_with(JUNK_CHARS) {:?}, return {:?}", fname_, RET_FALLBACK_TEXT);
                    return RET_FALLBACK_TEXT;
                }
                false => {
                    defx!("fname.starts_with(JUNK_CHARS) {:?}, return {:?}", fname_, RET_FALLBACK_UNPARSABLE);
                    return RET_FALLBACK_UNPARSABLE;
                }
            }
        }
        if fname2 != pathbuf_clean.extension().unwrap_or_default() {
            pathbuf_ref = pathbuf.with_file_name(fname2);
            pathbuf_clean = &pathbuf_ref;
            defo!("pathbuf_clean {:?}", pathbuf_clean);
            file_name = pathbuf_clean
                .file_name()
                .unwrap_or_default();
            defo!("file_name     {:?}", file_name);
        }
    }

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

    let file_suffix: String = pathbuf_clean
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    defo!("file_suffix   {:?}", file_suffix);

    if file_suffix.parse::<i32>().is_ok() {
        defo!("file_suffix   {:?} is a number; remove it", file_suffix);
        // suffix/extension is a number so remove it and call again
        let ret = pathbuf_to_filetype_impl(
            &pathbuf.clone().with_extension(""),
            unparseable_are_text,
            Some(fta),
        );
        defx!("return {:?}", ret);
        return ret;
    }

    match file_suffix.as_str() {
        "bz2" => {
            defo!("file_suffix {:?} is a bz2", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Bz2),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "etl" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Etl {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "evtx" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Evtx {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "journal" => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Journal {
                    archival_type: fta,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // `.loggz` is a OneDrive convention, though it typically wraps UTF-16 log files
        "loggz" => {
            defo!("file_suffix {:?} is a gzip but a OneDrive convention", file_suffix);
            let ret = PathToFiletypeResult::Filetype(
                FileType::Text {
                    archival_type: FileTypeArchive::Gz,
                    encoding_type: FileTypeTextEncoding::Utf16,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // `.gz` is very common
        // `.gzip` I have seen but I don't recall where
        "gz" | "gzip" => {
            defo!("file_suffix {:?} is a gzip", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Gz),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "lz4" => {
            defo!("file_suffix {:?} is lzma4", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Lz4),
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        odl_sub_type @ "aodl"
        | odl_sub_type @ "odl"
        | odl_sub_type @ "odlsent"
        | odl_sub_type @ "odlgz" => {
            let odl_sub_type_enum: OdlSubType = match odl_sub_type {
                "aodl" => OdlSubType::Aodl,
                "odl" => OdlSubType::Odl,
                "odlsent" => OdlSubType::Odlsent,
                "odlgz" => OdlSubType::Odlgz,
                _ => {
                    debug_panic!("unexpected odl_sub_type {:?}", odl_sub_type);
                    OdlSubType::Odl
                }
            };
            let ret = PathToFiletypeResult::Filetype(
                FileType::Odl {
                    archival_type: fta,
                    odl_sub_type: odl_sub_type_enum,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "tar" => {
            defo!("file_suffix {:?} is a tar filetype_archive.is_none()", file_suffix);
            let ret = PathToFiletypeResult::Archive(
                FileTypeArchiveMultiple::Tar,
                fta,
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        "xz" | "xzip" => {
            defo!("file_suffix {:?} is xz", file_suffix);
            let ret = pathbuf_to_filetype_impl(
                &pathbuf.clone().with_extension(""),
                unparseable_are_text,
                Some(FileTypeArchive::Xz),
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
                    archival_type: fta,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::AcctV3,
                }
            );
            defx!("matched file_suffix {:?}; return {:?}", file_suffix, ret);
            return ret;
        }
        // known unparseable file extensions/suffixes
        // covers some common extensions. not at all exhaustive but still helpful
        "7z"
        | "a"
        | "aac"
        | "aux"
        | "avi"
        | "bat"
        | "bin"
        | "bmp"
        | "bz"
        | "c"
        | "cat"
        | "class"
        | "cpp"
        | "cmd"
        | "diagpkg"
        | "dll"
        | "ear"
        | "exe"
        | "flac"
        | "flv"
        | "gif"
        | "h"
        | "hpp"
        | "htm"
        | "html"
        | "ico"
        | "jar"
        | "java"
        | "jpeg"
        | "jpg"
        | "lib"
        | "m4b"
        | "m4p"
        | "m4r"
        | "m4v"
        | "mkv"
        | "mov"
        | "mp3"
        | "mp4"
        | "msi"
        | "mui"
        | "o"
        | "ogg"
        | "opus"
        | "pl"
        | "png"
        | "ps1"
        | "psd1"
        | "py"
        | "rb"
        | "sh"
        | "so"
        | "svg"
        | "sys"
        | "tif"
        | "tiff"
        | "ttf"
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

    let file_name_sl: String = file_name
        .to_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_name_s: &str = file_name_sl.as_str();

    defo!("match file_name_s {:?}", file_name_s);
    match file_name_s {
        // OneDrive mapping files
        "general.keystore"
        | "ObfuscationStringMap.txt"
        => {
            match unparseable_are_text {
                true => {
                    defx!("file_name_s {:?} (OneDrive mapping), return {:?}", file_name_s, RET_FALLBACK_TEXT);
                    return RET_FALLBACK_TEXT;
                }
                false => {
                    defx!("file_name_s {:?} (OneDrive mapping), return {:?}", file_name_s, RET_FALLBACK_UNPARSABLE);
                    return RET_FALLBACK_UNPARSABLE;
                }
            }
        }
        _ => {}
    }

    defo!("match file_suffix.is_empty() {:?}", file_suffix.is_empty());

    if !file_suffix.is_empty() {
        defo!("file_suffix {:?} not empty; remove it and call again", file_suffix);
        // remove the file suffix/extension and call again
        let ret = pathbuf_to_filetype_impl(
            &pathbuf.clone().with_extension(""),
            unparseable_are_text,
            Some(fta),
        );
        defx!("return {:?}", ret);
        return ret;
    }

    // getting here means there is no suffix/extension in the file name

    defo!("match file_name_s.is_empty() {:?}", file_name_s.is_empty());

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

    defo!("match file_name_s {:?}", file_name_s);

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
                    archival_type: fta,
                    encoding_type: FileTypeTextEncoding::Utf8Ascii
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "journal"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::Journal {
                    archival_type: fta,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        "acct"
        => {
            let ret = PathToFiletypeResult::Filetype(
                FileType::FixedStruct {
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
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
                    archival_type: fta,
                    fixedstruct_type: FileTypeFixedStruct::Utmpx,
                }
            );
            defx!("matched file_name_s {:?}, return {:?}", file_name_s, ret);
            return ret;
        }
        _ => {
            defo!("file_name_s   {:?} not matched", file_name_s);
        }
    }

    // for example, `log_media`
    if file_name_s.starts_with("log_") {
        let ret = PathToFiletypeResult::Filetype(
            FileType::Text {
                archival_type: fta,
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
                archival_type: fta,
                encoding_type: FileTypeTextEncoding::Utf8Ascii
            }
        );
        defx!("ends_with('_log') {:?}; return {:?}", file_name_s, ret);
        return ret;
    }

    // not sure what the file is so try to process it as a plain text file

    let ret = PathToFiletypeResult::Filetype(
        FileType::Text {
            archival_type: fta,
            encoding_type: FileTypeTextEncoding::Utf8Ascii
        }
    );
    defx!("not sure what is {:?}; return {:?}", file_name_s, ret);

    ret
}

/// Determine the `FileType` of a file based on the `pathbuf` file name.
/// Attempt to determine the `FileType::archive_type` of the file.
///
/// Wrapper function for [`pathbuf_to_filetype_impl`]
pub fn pathbuf_to_filetype(
    pathbuf: &PathBuf,
    unparseable_are_text: bool,
) -> PathToFiletypeResult {
    pathbuf_to_filetype_impl(pathbuf, unparseable_are_text, None)
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
pub fn process_path_tar(
    path: &FPath,
    unparseable_are_text: bool,
    _filetypearchive: FileTypeArchive,
) -> Vec<ProcessPathResult> {
    defn!("({:?}, {:?}, {:?})", path, unparseable_are_text, _filetypearchive);

    // TODO [2024/04/29] handle `filetypearchive`; extract the file first and then process it

    // debug runtime checks
    #[cfg(all(debug_assertions, not(test)))]
    {
        defo!("debug_assertions");
        let std_path = fpath_to_path(path);
        if !std_path.exists() {
            panic!("path does not exist: {:?}", std_path);
        }
        if !std_path.is_file() {
            panic!("path is not a file: {:?}", std_path);
        }
    }

    let file: File = File::open(path).unwrap();
    defo!("tar::Archive::<File>::new({:?})", path);
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
    for (_i, entry_res) in entry_iter.enumerate() {
        let entry: tar::Entry<File> = match entry_res {
            Ok(val) => val,
            Err(err) => {
                defo!("entry Err {:?}; skip", err);
                let err_string = error_to_string(&err, path);
                results.push(ProcessPathResult::FileErr(path.clone(), err_string));
                continue;
            }
        };
        defo!("entry {:2}, size {:4}, {:?}", _i, entry.size(), entry.path().unwrap_or_default());
        let header: &tar::Header = entry.header();
        let etype: tar::EntryType = header.entry_type();
        // TODO: handle tar types `symlink` and `long_link`, currently they are ignored
        if !etype.is_file() {
            defo!("entry.header().entry_type() {:?}; skip", etype);
            continue;
        }
        defo!("entry.header().entry_type() {:?}", etype);
        let subpath: Cow<Path> = match entry.path() {
            Ok(val) => val,
            Err(err) => {
                defo!("entry.path() Err {:?}; skip", err);
                let err_string = error_to_string(&err, path);
                results.push(ProcessPathResult::FileErr(path.clone(), err_string));
                continue;
            }
        };
        if entry.size() == 0 {
            defo!("entry.size() is 0; skip");
            let subfpath: FPath = path.clone()
                + &String::from(SUBPATH_SEP)
                + subpath
                    .to_string_lossy()
                    .as_ref();
            results.push(ProcessPathResult::FileErrEmpty(subfpath, FileType::Unparsable));
            continue;
        }
        // first get the `FileType` of the subpath
        let subfpath: FPath = subpath
            .to_string_lossy()
            .to_string();
        let pathtofileresult = path_to_filetype(&subpath, unparseable_are_text);
        defo!("pathtofileresult {:?}", pathtofileresult);
        // path to a file within a .tar file looks like:
        //
        //     "path/file.tar|subpath/subfile"
        //
        // where `path/file.tar` are on the host filesystem, and `subpath/subfile` are within
        // the `.tar` file
        let mut fullpath: FPath = String::with_capacity(
            path.len()
            + SUBPATH_SEP.len_utf8()
            + subfpath.len()
            + 1
        );
        fullpath.push_str(path.as_str());
        fullpath.push(SUBPATH_SEP);
        fullpath.push_str(subfpath.as_str());
        defo!("fullpath {:?}", fullpath);
        let result: ProcessPathResult;
        match pathtofileresult {
            PathToFiletypeResult::Filetype(filetype) => {
                match filetype {
                    // Etl
                    FileType::Etl { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Etl { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Etl { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Etl{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Etl{ archival_type: at @ FileTypeArchive::Tar, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(String::from(
                                format!("cannot extract {} type from a tar archived file", at)
                            ))
                        );
                    }
                    FileType::Etl { archival_type: FileTypeArchive::Normal, .. }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Etl { archival_type: FileTypeArchive::Tar }
                        );
                    }
                    // Evtx
                    FileType::Evtx { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Evtx { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Evtx { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Evtx{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Evtx{ archival_type: at @ FileTypeArchive::Tar, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(String::from(
                                format!("cannot extract {} type from a tar archived file", at)
                            ))
                        );
                    }
                    FileType::Evtx { archival_type: FileTypeArchive::Normal, .. }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Evtx { archival_type: FileTypeArchive::Tar }
                        );
                    }
                    // FixedStruct
                    FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::FixedStruct{ archival_type: at @ FileTypeArchive::Tar, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(String::from(
                                format!("cannot extract {} type from a tar archived file", at)
                            ))
                        );
                    }
                    FileType::FixedStruct { archival_type: FileTypeArchive::Normal, fixedstruct_type: ft }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::FixedStruct {
                                archival_type: FileTypeArchive::Tar, fixedstruct_type: ft
                            }
                        );
                    }
                    // Odl
                    FileType::Odl { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Odl { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Odl { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Odl{ archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Odl{ archival_type: at @ FileTypeArchive::Tar, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(String::from(
                                format!("cannot extract {} type from a tar archived file", at)
                            ))
                        );
                    }
                    FileType::Odl { archival_type: FileTypeArchive::Normal, odl_sub_type }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Odl { archival_type: FileTypeArchive::Tar, odl_sub_type }
                        );
                    }
                    // Journal
                    FileType::Journal { archival_type: at @ FileTypeArchive::Bz2 }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Gz }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Lz4 }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Xz }
                    | FileType::Journal { archival_type: at @ FileTypeArchive::Tar }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(String::from(
                                format!("cannot extract {} type from a tar archived file", at)
                            ))
                        );
                    }
                    FileType::Journal { archival_type: FileTypeArchive::Normal }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Journal {
                                archival_type: FileTypeArchive::Tar,
                            }
                        );
                    }
                    // Text
                    FileType::Text { archival_type: at @ FileTypeArchive::Bz2, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Gz, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Lz4, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Xz, .. }
                    | FileType::Text { archival_type: at @ FileTypeArchive::Tar, .. }
                    => {
                        result = ProcessPathResult::FileErrNotSupported(
                            fullpath,
                            Some(String::from(
                                format!("cannot extract {} type from a tar archived file", at)
                            ))
                        );
                    }
                    FileType::Text { archival_type: FileTypeArchive::Normal, encoding_type: et }
                    => {
                        result = ProcessPathResult::FileValid(
                            fullpath, FileType::Text {
                                archival_type: FileTypeArchive::Tar, encoding_type: et
                            }
                        );
                    }
                    FileType::Unparsable => {
                        result = ProcessPathResult::FileErrNotSupported(fullpath, None);
                    }
                }
            }
            PathToFiletypeResult::Archive(..) => {
                // Issue #14 cannot support nested archives
                result = ProcessPathResult::FileErrNotSupported(
                    fullpath,
                    Some(String::from("nested archives are not supported")),
                );
            }
        }
        defo!("push {:?}", result);
        results.push(result);
    }
    #[cfg(any(debug_assertions, test))]
    {
        for (i, result) in results.iter().enumerate() {
            defo!("result {} {:?}", i, result);
        }
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
pub fn process_path(
    path: &FPath,
    unparseable_are_text: bool,
) -> Vec<ProcessPathResult> {
    defn!("({:?}, {:?})", path, unparseable_are_text);

    let mut std_path: PathBuf = PathBuf::from(path);

    deo!("std_path {:?}", std_path);

    std_path = match std_path.canonicalize() {
        Ok(val) => val,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                defx!("return FileErrNotExist({:?})", path);
                return vec![ProcessPathResult::FileErrNotExist(path.clone())];
            }
            std::io::ErrorKind::PermissionDenied => {
                defx!("return FileErrNoPermissions({:?})", path);
                return vec![ProcessPathResult::FileErrNoPermissions(path.clone())];
            }
            _ => {
                let err_string = error_to_string(&err, path);
                defx!("return FileErr({:?}, {:?})", path, err_string);
                return vec![ProcessPathResult::FileErr(path.clone(), err_string)];
            }
        },
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
                let paths: Vec<ProcessPathResult> = vec![ProcessPathResult::FileValid(path.clone(), filetype)];
                defx!("({:?}) {:?}", path, paths);
                return paths;
            }
            PathToFiletypeResult::Archive(archive, fta) => {
                match archive {
                    FileTypeArchiveMultiple::Tar => {
                        // getting here means `std_path` is a `.tar` file
                        defo!("std_path is a .tar file {:?}", std_path);
                        let results = process_path_tar(path, unparseable_are_text, fta);
                        defx!("return process_path_tar({:?}) returned {} results", path, results.len());
                        return results;
                    }
                }
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
            PathToFiletypeResult::Archive(archive, fta) => {
                let results: Vec<ProcessPathResult>;
                match archive {
                    FileTypeArchiveMultiple::Tar => {
                        // getting here means `std_path` is a `.tar` file
                        defo!("std_path_entry is a .tar file {:?}", std_path_entry);
                        results = process_path_tar(&path_to_fpath(std_path_entry), unparseable_are_text, fta);
                    }
                }
                for result in results.into_iter() {
                    paths.push(result);
                }
                continue;
            }
        };
        match filetype {
            FileType::Etl{ archival_type: FileTypeArchive::Normal }
            | FileType::Etl{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Etl{ archival_type: FileTypeArchive::Gz }
            | FileType::Etl{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Etl{ archival_type: FileTypeArchive::Tar }
            | FileType::Etl{ archival_type: FileTypeArchive::Xz }
            | FileType::Evtx{ archival_type: FileTypeArchive::Normal }
            | FileType::Evtx{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Evtx{ archival_type: FileTypeArchive::Gz }
            | FileType::Evtx{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Evtx{ archival_type: FileTypeArchive::Tar }
            | FileType::Evtx{ archival_type: FileTypeArchive::Xz }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Normal, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Bz2, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Gz, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Lz4, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Tar, fixedstruct_type: _ }
            | FileType::FixedStruct{ archival_type: FileTypeArchive::Xz, fixedstruct_type: _ }
            | FileType::Journal{ archival_type: FileTypeArchive::Normal }
            | FileType::Journal{ archival_type: FileTypeArchive::Bz2 }
            | FileType::Journal{ archival_type: FileTypeArchive::Gz }
            | FileType::Journal{ archival_type: FileTypeArchive::Lz4 }
            | FileType::Journal{ archival_type: FileTypeArchive::Tar }
            | FileType::Journal{ archival_type: FileTypeArchive::Xz }
            | FileType::Odl{ archival_type: FileTypeArchive::Normal, odl_sub_type: _ }
            | FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf8Ascii }
            | FileType::Text{ archival_type: FileTypeArchive::Bz2, encoding_type: FileTypeTextEncoding::Utf8Ascii }
            | FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf8Ascii }
            | FileType::Text{ archival_type: FileTypeArchive::Lz4, encoding_type: FileTypeTextEncoding::Utf8Ascii }
            | FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf8Ascii }
            | FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf8Ascii }
            => {
                deo!("paths.push(FileValid(({:?}, {:?})))", fpath_entry, filetype);
                paths.push(ProcessPathResult::FileValid(fpath_entry, filetype));
            }
            ft @ FileType::Odl{ archival_type: FileTypeArchive::Bz2, odl_sub_type: _ }
            | ft @ FileType::Odl{ archival_type: FileTypeArchive::Gz, odl_sub_type: _ }
            | ft @ FileType::Odl{ archival_type: FileTypeArchive::Lz4, odl_sub_type: _ }
            | ft @ FileType::Odl{ archival_type: FileTypeArchive::Tar, odl_sub_type: _ }
            | ft @ FileType::Odl{ archival_type: FileTypeArchive::Xz, odl_sub_type: _ }
           => {
                deo!("Odl archived is not supported {:?}", std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(
                    fpath_entry,
                    Some(format!("Compressed ODL {}", ft.archival_type())),
                ));
            }
            ft @ FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf16 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Normal, encoding_type: FileTypeTextEncoding::Utf32 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Bz2, encoding_type: FileTypeTextEncoding::Utf16 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Bz2, encoding_type: FileTypeTextEncoding::Utf32 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf16 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Gz, encoding_type: FileTypeTextEncoding::Utf32 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Lz4, encoding_type: FileTypeTextEncoding::Utf16 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Lz4, encoding_type: FileTypeTextEncoding::Utf32 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf16 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Tar, encoding_type: FileTypeTextEncoding::Utf32 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf16 }
            | ft @ FileType::Text{ archival_type: FileTypeArchive::Xz, encoding_type: FileTypeTextEncoding::Utf32 }
            => {
                let et: String = match ft.encoding_type() {
                    Some(e) => e.to_string(),
                    None => String::from(""),
                };
                deo!("Text encoding {} not supported {:?}", et, std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(
                    fpath_entry,
                    Some(format!("Encoding {}", et)),
                ));
            }
            FileType::Unparsable
            => {
                deo!("Path not a log file {:?}", std_path_entry);
                paths.push(ProcessPathResult::FileErrNotSupported(
                    fpath_entry,
                    None,
                ));
            }
        }
    }
    defx!("return {:?}", paths);

    paths
}
