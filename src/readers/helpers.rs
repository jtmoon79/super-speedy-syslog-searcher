// src/readers/helpers.rs

//! Miscellaneous helper functions for _Readers_.

use std;
use std::io::{
    Read,
    Seek,
};

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
    err_from_err_path,
    FPath,
    FileSz,
    PathId,
};
use crate::readers::filehandlemanager::{
    FILE_HANDLE_MANAGER,
    FileHandleManaged,
    FileHandleRole,
    OpenOptionsManaged,
};
pub(crate) use ::zip::read::ZipArchive;
pub(crate) use ::zip::read::ZipFile;

pub(crate) type ZipHandle = ZipArchive<FileHandleManaged>;

pub(crate) fn gzip_header_metadata<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
) -> std::io::Result<(String, u32)> {
    defn!();

    let position: u64 = reader.stream_position()?;
    let result: std::io::Result<(String, u32)> = {
        let mut buffered = std::io::BufReader::new(&mut *reader);
        (|| {
            let mut fixed = [0; 10];
            buffered.read_exact(&mut fixed)?;
            let flags: u8 = fixed[3];
            let mtime: u32 = u32::from_le_bytes(fixed[4..8].try_into().unwrap());

            if flags & 0x04 != 0 {
                let mut extra_len = [0; 2];
                buffered.read_exact(&mut extra_len)?;
                buffered.seek(std::io::SeekFrom::Current(i64::from(u16::from_le_bytes(extra_len))))?;
            }

            let filename: Vec<u8> = if flags & 0x08 != 0 {
                read_gzip_zero_terminated(&mut buffered, true)?
            } else {
                Vec::new()
            };
            if flags & 0x10 != 0 {
                read_gzip_zero_terminated(&mut buffered, false)?;
            }
            if flags & 0x02 != 0 {
                let mut header_crc = [0; 2];
                buffered.read_exact(&mut header_crc)?;
            }

            Ok((String::from_utf8(filename).unwrap_or_default(), mtime))
        })()
    };
    reader.seek(std::io::SeekFrom::Start(position))?;
    defx!("return {:?}", result);

    result
}

fn read_gzip_zero_terminated<R: std::io::BufRead>(
    reader: &mut R,
    keep: bool,
) -> std::io::Result<Vec<u8>> {
    defn!();

    let mut value: Vec<u8> = Vec::new();
    loop {
        let buffer: &[u8] = reader.fill_buf()?;
        if buffer.is_empty() {
            defx!("return Err()");
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unterminated gzip header field",
            ));
        }
        match buffer.iter().position(|byte| *byte == 0) {
            Some(end) => {
                if keep {
                    value.extend_from_slice(&buffer[..end]);
                }
                reader.consume(end + 1);
                defx!("return Ok({:?} bytes)", value.len());
                return Ok(value);
            }
            None => {
                let count = buffer.len();
                if keep {
                    value.extend_from_slice(buffer);
                }
                reader.consume(count);
            }
        }
    }
}

pub(crate) fn zip_error(error: zip::result::ZipError, path: &str) -> std::io::Error {
    let error = std::io::Error::from(error);
    err_from_err_path(&error, &path.to_string(), Some("ZIP"))
}

pub(crate) fn open_zip(
    path_id: PathId,
    role: FileHandleRole,
    path: &std::path::Path,
) -> std::io::Result<ZipHandle> {
    let file = FILE_HANDLE_MANAGER.request_open_managed(
        path_id, role, path, OpenOptionsManaged::read_only(),
    )?;
    ZipArchive::new(file).map_err(|error| zip_error(error, &path_to_fpath(path)))
}

pub(crate) fn zip_member_index<R: std::io::Read + std::io::Seek>(
    archive: &ZipArchive<R>,
    member: &str,
    path: &str,
) -> std::io::Result<usize> {
    for index in 0..archive.len() {
        let entry = archive.by_index_data(index).map_err(|error| zip_error(error, path))?;
        if entry.name().map_err(|error| zip_error(error, path))?.as_ref() == member {
            if !entry.is_file() || entry.encrypted() || entry.name_raw().contains(&0) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!("ZIP member is not an unencrypted regular file with a valid name: {path:?}"),
                ));
            }
            return Ok(index);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound, format!("ZIP member not found: {path:?}"),
    ))
}

pub(crate) fn zip_member_mtime<R: std::io::Read>(
    entry: &ZipFile<'_, R>,
    fallback: std::time::SystemTime,
    path: &str,
) -> std::time::SystemTime {
    use chrono::{Local, NaiveDateTime, Utc};
    use zip::ExtraField;

    let ntfs = entry.extra_data_fields().find_map(|field| match field {
        ExtraField::Ntfs(time) => crate::data::etl::filetime_to_datetimel(
            time.mtime(), &chrono::FixedOffset::east_opt(0).unwrap(),
        ).map(std::time::SystemTime::from),
        _ => None,
    });
    if let Some(time) = ntfs {
        return time;
    }
    let unix = entry.extra_data_fields().find_map(|field| match field {
        ExtraField::ExtendedTimestamp(time) => time.mod_time()
            .and_then(|seconds| chrono::DateTime::<Utc>::from_timestamp(i64::from(seconds), 0))
            .map(std::time::SystemTime::from),
        _ => None,
    });
    if let Some(time) = unix {
        return time;
    }
    let dos = entry.last_modified()
        .and_then(|time| NaiveDateTime::try_from(time).ok())
        .and_then(|time| time.and_local_timezone(Local).single());
    if let Some(time) = dos {
        return time.into();
    }
    crate::e_wrn!("ZIP member {:?} has no usable modification time; using archive modification time", path);
    fallback
}

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
                    return Err(
                        Error::other(format!("path_filesz() returned None for {:?}", $path))
                    );
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
