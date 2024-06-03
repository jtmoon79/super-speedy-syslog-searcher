// src/readers/filedecompressor.rs

//! The `filedecompressor` module is for decompressing files to temporary files.

use std::borrow::Cow;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Result, Write};
use std::path::Path;
use std::time::SystemTime;

use crate::{debug_panic, e_wrn, path_filesz_or_return_err};
use crate::common::{
    err_from_err_path,
    err_from_err_path_result,
    File,
    FileOpenOptions,
    FileSz,
    FileType,
    FileTypeArchive,
    FPath,
    FileMetadata,
};
use crate::readers::blockreader::{
    BlockReader,
    SUBPATH_SEP,
    TarChecksum,
    TarHandle,
    TarMTime,
};
use crate::readers::helpers::{
    fpath_to_path,
    path_to_fpath,
    path_filesz,
};

use ::bzip2_rs::DecoderReader as Bz2DecoderReader;
// `flate2` is for gzip files.
use ::flate2::read::GzDecoder;
use ::flate2::GzHeader;
// `lz4_flex` is for lz4 files.
use ::lz4_flex;
// `lzma_rs` is for xz files.
use ::lzma_rs;
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
use ::tempfile::{Builder, NamedTempFile};


type BufReaderLz4 = BufReader<File>;
type Lz4FrameReader = lz4_flex::frame::FrameDecoder<BufReaderLz4>;

const SUFFIX_EVTX: &str = ".evtx";
const SUFFIX_FIXEDSTRUCT: &str = ".wtmp";
const SUFFIX_JOURNAL: &str = ".journal";
const SUFFIX_TEXT: &str = ".log";

/// optional tuple value returned by `decompress_to_ntf()`:
/// - `NamedTempFile` is the temporary file handle
/// - `Option<SystemTime>` is the file modification time if found
/// - `FileSz` is the size of the temporary file
type DecompressToNtfValue = Option<(NamedTempFile, Option<SystemTime>, FileSz)>;
/// `Result` wrapper for `DecompressToNtfValue`
type DecompressToNtfResult = Result<DecompressToNtfValue>;

/// wrapper for call to `err_from_err_path_result::<DecompressToNtfValue>`
macro_rules! err_from_err_path_result_dtn {
    ($error: expr, $fpath: expr, $mesg: expr) => ({{
        err_from_err_path_result::<DecompressToNtfValue>($error, $fpath, $mesg)
    }})
}

/// helper function to decompress a `path_std` to a temporary file.
/// Returns a `Result` containing a [`DecompressToNtfValue`].
/// Return value `None` means no file was decompressed because it was not needed
/// as determined by the passed `file_type`.
pub fn decompress_to_ntf(path_std: &Path, file_type: &FileType)
    -> DecompressToNtfResult
{
    defn!("({:?}, file_type={:?})", path_std, file_type);
    const BUF_SZ: usize = 65536;
    let mut buf: [u8; BUF_SZ] = [0; BUF_SZ];
    let mut mtime_opt: Option<SystemTime> = None;
    let suffix: &str;
    let file_type_archive: &FileTypeArchive = match file_type {
        FileType::Evtx { archival_type } => {
            suffix = SUFFIX_EVTX;

            archival_type
        }
        FileType::FixedStruct { archival_type, .. } => {
            debug_panic!("Unexpected {:?}", file_type);
            suffix = SUFFIX_FIXEDSTRUCT;

            archival_type
        }
        FileType::Journal { archival_type } => {
            suffix = SUFFIX_JOURNAL;

            archival_type
        }
        FileType::Text { archival_type, .. } => {
            debug_panic!("Unexpected {:?}", file_type);
            suffix = SUFFIX_TEXT;

            archival_type
        }
        FileType::Unparsable => {
            debug_panic!("Unexpected {:?}", file_type);

            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("Unexpected FileType::Unparsable for {:?}", path_std)
                )
            );
        }
    };
    match file_type_archive {
        FileTypeArchive::Normal => {
            defx!("FileTypeArchive::Normal; return Ok(None)");
            return Ok(None);
        },
        FileTypeArchive::Bz2
        | FileTypeArchive::Gz
        | FileTypeArchive::Lz4
        | FileTypeArchive::Tar
        | FileTypeArchive::Xz
        => {}
    }

    let fpath: FPath = path_to_fpath(path_std);
    defo!("fpath {:?}", fpath);

    defo!("tempfile::Builder::new()");
    let ntf: NamedTempFile = match Builder::new()
        .prefix("s4-")
        .suffix(suffix)
        .tempfile()
    {
        Ok(val) => val,
        Err(err) => {
            defx!("tempfile::Builder::new().tempfile() Error, return {:?}", err);
            return err_from_err_path_result_dtn!(
                &err, &fpath, Some("tempfile::Builder::new() failed")
            );
        }
    };
    let path_ntf = ntf.path();
    defo!("path_ntf {:?}", path_ntf);

    let mut open_options = FileOpenOptions::new();

    defo!("open_options.write().open({:?})", path_ntf);
    let file_ntf: File = match open_options
        .read(false)
        .write(true)
        .open(path_ntf)
    {
        Ok(val) => val,
        Err(err) => {
            defx!("open_options.read({:?}) Error, return {:?}", path_ntf, err);
            return err_from_err_path_result_dtn!(
                &err,
                &fpath,
                Some(format!("open_options for NTF file failed {:?}", path_ntf).as_str())
            );
        }
    };
    defo!("file_ntf {:?}", file_ntf);

    let file_compressed: File;
    let file_compressed_metadata: FileMetadata;
    match file_type_archive {
        FileTypeArchive::Normal
        | FileTypeArchive::Bz2
        | FileTypeArchive::Gz
        | FileTypeArchive::Lz4
        | FileTypeArchive::Xz
        => {
            defo!("open_options.read().open({:?})", path_std);
            file_compressed = match open_options
                .read(true)
                .open(path_std)
            {
                Ok(val) => val,
                Err(err) => {
                    defx!("open_options.read({:?}) Error, return {:?}", path_std, err);
                    return err_from_err_path_result_dtn!(
                        &err, &fpath, Some("open_options failed")
                    );
                }
            };
            file_compressed_metadata = match file_compressed.metadata() {
                Ok(val) => val,
                Err(err) => {
                    defx!("file_compressed.metadata() Error, return {:?}", err);
                    return err_from_err_path_result_dtn!(
                        &err, &fpath, Some("file_compressed.metadata() failed")
                    );
                }
            };
        }
        FileTypeArchive::Tar
        => {
            // TODO: [2024/05] most code in this block repeats code in
            //       BlockReader::open_tar(). Both should be refactored to reduce
            //       duplicate code.

            // for files embedded in `.tar` files, the path will be two paths within one `FPath`.
            // Each path is separated by `SUBPATH_SEP`,
            // e.g. `path/to/tarfile.tar|path/in/tarfile.journal`

            // split the passed path into the path and subpath
            let (path_, subpath_) = match fpath.rsplit_once(SUBPATH_SEP) {
                Some(val) => val,
                None => {
                    defx!(
                        "Tar: filetype {:?} but failed to find delimiter {:?} in {:?}",
                        file_type_archive,
                        SUBPATH_SEP,
                        fpath,
                    );
                    return DecompressToNtfResult::Err(Error::new(
                        // TODO: TRACKING: use `ErrorKind::InvalidFilename` when it is stable
                        //       <https://github.com/rust-lang/rust/issues/86442>
                        ErrorKind::InvalidInput,
                        format!(
                            "Given Filetype {:?} but failed to find delimiter {:?} in {:?}",
                            file_type_archive, SUBPATH_SEP, fpath
                        ),
                    ));
                }
            };
            let subpath_opt = Some(subpath_.to_string());
            defo!("Tar: subpath_opt {:?}", subpath_opt);
            let subpath: &String = subpath_opt.as_ref().unwrap();
            defo!("Tar: subpath     {:?}", subpath);
            let fpath_tar: FPath = FPath::from(path_);
            defo!("Tar: fpath_tar   {:?}", fpath_tar);
            let path_tar = fpath_to_path(&fpath_tar);

            // open the .tar file

            let mut archive: TarHandle = BlockReader::open_tar(path_tar)?;
            let entry_iter: tar::Entries<File> = match archive.entries_with_seek() {
                Ok(val) => val,
                Err(err) => {
                    defx!("Tar: Err {:?}", err);
                    return err_from_err_path_result_dtn!(
                        &err, &fpath, Some("archive.entries_with_seek() failed")
                    );
                }
            };

            // in the .tar file, find the entry with the matching subpath

            let mut entry_opt: Option<tar::Entry<File>> = None;
            let mut filesz_header: FileSz = 0;
            for (_index, entry_res) in entry_iter.enumerate() {
                defo!("Tar: index {}", _index);
                let entry: tar::Entry<File> = match entry_res {
                    Ok(val) => val,
                    Err(_err) => {
                        defo!("Tar: match entry_res Err {:?}", _err);
                        continue;
                    }
                };
                let subpath_cow: Cow<Path> = match entry.path() {
                    Ok(val) => val,
                    Err(_err) => {
                        defo!("Tar: entry.path() Err {:?}", _err);
                        continue;
                    }
                };
                let subfpath: FPath = subpath_cow
                    .to_string_lossy()
                    .to_string();
                defo!("Tar: subfpath {:?}", subfpath);
                if subpath != &subfpath {
                    defo!("Tar: skip {:?}, looking for {:?}", subfpath, subpath);
                    continue;
                }
                // found the matching subpath
                defo!("Tar: found {:?}", subpath);
                filesz_header = match entry.header().size() {
                    Ok(val) => val,
                    Err(err) => {
                        defx!("Tar: entry.header().size() Err {:?}", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("FileTar header().size")
                        );
                    }
                };
                defo!("Tar: filesz_header {:?}", filesz_header);
                let _checksum: TarChecksum = match entry.header().cksum() {
                    Ok(val) => val,
                    Err(_err) => {
                        defo!("Tar: entry.header().cksum() Err {:?}", _err);

                        0
                    }
                };
                defo!("Tar: checksum 0x{:08X}", _checksum);
                let mtime: TarMTime = match entry.header().mtime() {
                    Ok(val) => val,
                    Err(_err) => {
                        defo!("Tar: entry.header().mtime() Err {:?}", _err);

                        0
                    }
                };
                defo!("Tar: mtime {:?}", mtime);
                mtime_opt = match mtime {
                    0 => None,
                    _ => {
                        Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(mtime))
                    },
                };

                entry_opt = Some(entry);

                break;
            }

            // read the tar entry and write it into the temporary file

            let mut entry = match entry_opt {
                None => {
                    defx!("Tar: entry_opt is None; return Ok(None)");
                    return Ok(None);
                }
                Some(entry) => entry,
            };
            let mut bufwriter: BufWriter<File> = BufWriter::new(file_ntf);
            defo!("Tar: bufwriter {:?}", bufwriter);
            let mut bytes_written: usize = 0;
            loop {
                match entry.read(&mut buf) {
                    Ok(num_bytes) => {
                        defo!("Tar: entry.read(buf) returned {}", num_bytes);
                        bytes_written += num_bytes;
                        if num_bytes == 0 {
                            break;
                        }
                        match bufwriter.write_all(&buf[..num_bytes]) {
                            Ok(_) => {
                                defo!("Tar: bufwriter.write_all(buf[..{}]) returned Ok", num_bytes);
                            }
                            Err(err) => {
                                defx!("Tar: bufwriter.write_all() Err {:?}", err);
                                return err_from_err_path_result_dtn!(
                                    &err, &fpath, Some("FileTar bufwriter.write_all()")
                                );
                            }
                        }
                    }
                    Err(err) => {
                        defx!("Tar: entry.read() Err {:?}", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("FileTar entry.read()")
                        );
                    }
                }
            }
            bufwriter.flush()?;

            // get the size of the decompressed temporary file
            let file_sz: FileSz = path_filesz_or_return_err!(path_ntf);
            // sanity check
            if filesz_header != file_sz {
                e_wrn!(
                    "File {:?}, tar header file size {} != {} file size of the temporary file {:?}",
                    fpath, filesz_header, file_sz, path_ntf,
                );
            } else if filesz_header != (bytes_written as FileSz) {
                e_wrn!(
                    "File {:?}, tar header file size {} != {} bytes written to temporary file {:?}",
                    fpath, filesz_header, bytes_written, path_ntf,
                );
            }

            defx!("Tar: return Ok(Some(({:?}, {:?}, {:?}))", ntf.path(), mtime_opt, file_sz);
            return Ok(Some((ntf, mtime_opt, file_sz)));
        }
    }

    // TODO: [2024/05/12] handle process early exit and removal of temporary files
    //       this is about where this temporary file would be noted in some global
    //       container, and later removed during an unexpected exit (like user presses ctrl+c).

    match file_type_archive {
        FileTypeArchive::Normal => {
            debug_panic!("Unexpected {:?}", file_type_archive);
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("Unexpected FileTypeArchive::Normal for {:?}", path_std)
                )
            )
        },
        FileTypeArchive::Bz2
        => {
            let mut bufwriter: BufWriter<File> = BufWriter::new(file_ntf);
            defo!("bufwriter {:?}", bufwriter);
            let mut bz2_decoder: Bz2DecoderReader<File> = Bz2DecoderReader::new(file_compressed);
            defo!("bz2_decoder");
            let mut _filesz_uncompressed: FileSz = 0;

            let mut _loop_count: usize = 0;
            loop {
                defo!("{:3} bz2_decoder.read(buf size {})", _loop_count, BUF_SZ);
                let bytes_read: usize = match bz2_decoder.read(&mut buf) {
                    Ok(sz) => sz,
                    Err(err) => {
                        defx!("bz2_decoder.read() Error, return {:?}", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("bz2_decoder.read() failed")
                        );
                    }
                };
                if bytes_read == 0 {
                    break;
                }
                defo!("{:3} bufwriter.write_all(buf size {})", _loop_count, bytes_read);
                match bufwriter.write_all(&buf[..bytes_read]) {
                    Ok(_) => {}
                    Err(err) => {
                        defx!("bufwriter.write_all() Error, return {:?}", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("bufwriter.write_all() failed")
                        );
                    }
                }
                _filesz_uncompressed += bytes_read as FileSz;
                _loop_count += 1;
            }
            defo!("_filesz_uncompressed {}", _filesz_uncompressed);
        }
        FileTypeArchive::Gz
        => {
            // TODO: [2024/05] most code in this block repeats code in
            //       BlockReader::new(). Both should be refactored to reduce
            //       duplicate code.

            let mut decoder: GzDecoder<File> = GzDecoder::new(file_compressed);
            defo!("GzDecoder: {:?}", decoder);
            let header_opt: Option<&GzHeader> = decoder.header();
            let mut _filename: String = String::with_capacity(0);

            //
            // GZIP binary format https://datatracker.ietf.org/doc/html/rfc1952#page-5
            //
            // Each member has the following structure:
            //
            // +---+---+---+---+---+---+---+---+---+---+
            // |ID1|ID2|CM |FLG|     MTIME     |XFL|OS | (more-->)
            // +---+---+---+---+---+---+---+---+---+---+
            //
            // MTIME (Modification TIME)
            // This gives the most recent modification time of the original
            // file being compressed.  The time is in Unix format, i.e.,
            // seconds since 00:00:00 GMT, Jan.  1, 1970.  (Note that this
            // may cause problems for MS-DOS and other systems that use
            // local rather than Universal time.)  If the compressed data
            // did not come from a file, MTIME is set to the time at which
            // compression started.  MTIME = 0 means no time stamp is
            // available.
            //
            let mut mtime: SystemTime = SystemTime::UNIX_EPOCH;
            match header_opt {
                Some(header) => {
                    let filename_: &[u8] = header
                        .filename()
                        .unwrap_or(&[]);
                    _filename = match String::from_utf8(filename_.to_vec()) {
                        Ok(val) => val,
                        Err(_err) => String::with_capacity(0),
                    };
                    mtime = header.mtime_as_datetime().unwrap_or(SystemTime::UNIX_EPOCH);
                }
                None => {
                    defo!("FileGz: GzDecoder::header() is None for {:?}", path_std);
                }
            };
            defo!("filename {:?}", _filename);
            defo!("mtime    {:?}", mtime);
            mtime_opt = if mtime == SystemTime::UNIX_EPOCH {
                None
            } else {
                Some(mtime)
            };
            defo!("mtime_opt {:?}", mtime_opt);

            let mut bufwriter: BufWriter<File> = BufWriter::new(file_ntf);
            defo!("bufwriter {:?}", bufwriter);
            let mut _bytes_read_total: usize = 0;
            defo!("GzDecoder start");
            loop {
                let bytes_read: usize = match decoder.read(&mut buf) {
                    Ok(val) => val,
                    Err(err) => {
                        defx!("GzDecoder::read() Error, return Err({:?})", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("GzDecoder.read() failed")
                        );
                    }
                };
                defo!("GzDecoder read {:?} bytes", bytes_read);
                _bytes_read_total += bytes_read;
                if bytes_read == 0 {
                    break;
                }
                match bufwriter.write_all(&buf[..bytes_read]) {
                    Ok(_) => {}
                    Err(err) => {
                        defx!("bufwriter.write_all() Error, return Err({:?})", err);
                        return err_from_err_path_result_dtn!(
                            &err,
                            &fpath,
                            Some("bufwriter.write_all() failed")
                        );
                    }
                }
            }
            defo!("GzDecoder complete");
            defo!("bytes decompressed {:?}", _bytes_read_total);
        }
        FileTypeArchive::Lz4
        => {
            // TODO: [2024/05] most code in this block repeats code in
            //       BlockReader::new(). Both should be refactored to reduce
            //       duplicate code.

            let bufreader: BufReader<File> = BufReader::new(file_compressed);
            defo!("bufreader {:?}", bufreader);
            let mut bufwriter: BufWriter<File> = BufWriter::new(file_ntf);
            defo!("bufwriter {:?}", bufwriter);
            let mut lz4_decoder: Lz4FrameReader = Lz4FrameReader::new(bufreader);
            defo!("lz4_decoder {:?}", lz4_decoder);
            let mut _filesz_uncompressed: FileSz = 0;
            
            let mut _loop_count: usize = 0;
            loop {
                defo!("{:3} lz4_decoder.read(buf size {})", _loop_count, BUF_SZ);
                let bytes_read: usize = match lz4_decoder.read(&mut buf) {
                    Ok(sz) => sz,
                    Err(err) => {
                        defx!("lz4_decoder.read() Error, return {:?}", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("lz4_decoder.read() failed")
                        );
                    }
                };
                if bytes_read == 0 {
                    break;
                }
                defo!("{:3} bufwriter.write_all(buf size {})", _loop_count, bytes_read);
                match bufwriter.write_all(&buf[..bytes_read]) {
                    Ok(_) => {}
                    Err(err) => {
                        defx!("bufwriter.write_all() Error, return {:?}", err);
                        return err_from_err_path_result_dtn!(
                            &err, &fpath, Some("bufwriter.write_all() failed")
                        );
                    }
                }
                _filesz_uncompressed += bytes_read as FileSz;
                _loop_count += 1;
            }
            defo!("_filesz_uncompressed {}", _filesz_uncompressed);
        }
        FileTypeArchive::Tar
        => {
            debug_panic!("FileTypeArchive::Tar already handled, should not reach here");
            return Err(
                Error::new(
                    ErrorKind::Other,
                    format!("Unexpected FileTypeArchive::Tar for {:?}", path_std)
                )
            );
        }
        FileTypeArchive::Xz
        => {
            let mut bufwriter: BufWriter<File> = BufWriter::new(file_ntf);
            defo!("bufwriter {:?}", bufwriter);
            let mut bufreader: BufReader<File> = BufReader::new(file_compressed);
            defo!("bufreader {:?}", bufreader);
            defo!("xz_decompress()");
            match lzma_rs::xz_decompress(&mut bufreader, &mut bufwriter) {
                Ok(_) => {}
                Err(err) => {
                    match &err {
                        lzma_rs::error::Error::IoError(ref ioerr) => {
                            defo!("ioerr.kind() {:?}", ioerr.kind());
                            return Err(
                                err_from_err_path(ioerr, &fpath, Some("(xz_decompress failed)"))
                            );
                        }
                        _err => {
                            defo!("err {:?}", _err);
                        }
                    }
                    defx!("xz_decompress Error, return Err({:?})", err);
                    return Err(
                        Error::new(
                            ErrorKind::Other,
                            format!("xz_decompress failed: {} for file {:?}", err, fpath),
                        )
                    );
                }
            }
            defo!("xz_decompress() complete");
        }
    }

    // in case the mtime was not found in the header, try to get it from the file metadata
    mtime_opt = match mtime_opt {
        Some(val) => {Some(val)}
        None => {
            // no mtime found from the header so try get it from the file metadata
            match file_compressed_metadata.modified() {
                Result::Ok(systemtime) => {
                    match file_type.to_filetypearchive() {
                        FileTypeArchive::Bz2
                        | FileTypeArchive::Gz
                        | FileTypeArchive::Lz4
                        | FileTypeArchive::Xz
                        => {
                            // get the file modified time from the original
                            // compressed file, not the temporary file
                            match std::fs::metadata(path_std) {
                                Result::Ok(val) => {
                                    match val.modified() {
                                        Result::Ok(systemtime) => {
                                            defo!(
                                                "mtime from metadata {:?} from file {:?}",
                                                systemtime, path_std);

                                            Some(systemtime)
                                        },
                                        Result::Err(_err) => {
                                            defo!("metadata({:?}).modified() Err {}", path_std, _err);
                                            defo!("mtime from fallback {:?}", SystemTime::UNIX_EPOCH);

                                            Some(SystemTime::UNIX_EPOCH)
                                        }
                                    }
                                },
                                Result::Err(_err) => {
                                    defo!("std::fs::metadata({:?}) Err {}", path_std, _err);
                                    defo!("mtime from fallback {:?}", SystemTime::UNIX_EPOCH);

                                    Some(SystemTime::UNIX_EPOCH)
                                }
                            }
                        }
                        FileTypeArchive::Tar => {
                            debug_panic!("Unexpected FileTypeArchive::Tar");
                            e_wrn!("Unexpected FileTypeArchive::Tar for {:?}", path_std);
                            defo!("mtime from fallback {:?}", SystemTime::UNIX_EPOCH);

                            Some(SystemTime::UNIX_EPOCH)
                        }
                        FileTypeArchive::Normal => {
                            debug_panic!("Unexpected FileTypeArchive::Normal");
                            e_wrn!("Unexpected FileTypeArchive::Normal for {:?}", path_std);
                            defo!(
                                "mtime from metadata {:?} from file {:?}",
                                systemtime, path_std
                            );

                            Some(systemtime)
                        }
                    }
                },
                Result::Err(_err) => {
                    defo!("Err {:?}", _err);
                    defo!("mtime from fallback {:?}", SystemTime::UNIX_EPOCH);

                    Some(SystemTime::UNIX_EPOCH)
                }
            }
        }
    };

    // and return the size of the decompressed temporary file
    let file_sz: FileSz = path_filesz_or_return_err!(path_ntf);

    defx!("return Ok(Some(({:?}, {:?}, {:?}))", ntf.path(), mtime_opt, file_sz);
    Ok(Some((ntf, mtime_opt, file_sz)))
}
