// src/tests/filehandlemanager_tests.rs
//
// this file initially created by Co-pilot + GPT-5.5, heavily revised by human @jtmoon79

//! Tests for `FileHandleManager`.

use std::collections::HashMap;
use std::io::{
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom,
    Write,
};

use ::more_asserts::assert_ge;

use crate::common::{
    FileMetadata,
    FileType,
    FileTypeArchive,
    FileTypeTextEncoding,
    FPath,
    OdlSubType,
    PathId,
    summary_stat,
    summary_stats_enable,
};
use crate::debug::helpers::{
    create_temp_file,
    NamedTempFile,
};
use crate::readers::filehandlemanager::{
    filetype_handle_counts,
    FileHandleManager,
    FileHandleRole,
    FileHandleManaged,
    FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT,
    OpenOptionsManaged,
    OpenMaxCountType,
};
use crate::readers::helpers::path_to_fpath;

/// sanity check `open_max_default`  is not too small
const OPEN_MAX_DEFAULT_GT: u32 = 450;

const PATH_ID_A: PathId = 99_000;
const PATH_ID_B: PathId = 99_001;
const PATH_ID_C: PathId = 99_002;
const PATH_ID_D: PathId = 99_003;

fn raw_os_error_too_many_open_files() -> i32 {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            ::nix::errno::Errno::EMFILE as i32
        } else if #[cfg(windows)] {
            4
        } else {
            0
        }
    }
}

fn raw_os_error_not_found() -> i32 {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            ::nix::errno::Errno::ENOENT as i32
        } else if #[cfg(windows)] {
            2
        } else {
            0
        }
    }
}

fn manager(open_max: usize) -> FileHandleManager {
    summary_stats_enable();
    FileHandleManager::new_open_max(OpenMaxCountType::new(open_max).unwrap())
}

fn read_helper(
    manager: &FileHandleManager,
    handle: &FileHandleManaged,
    buf: &mut [u8],
) -> Result<usize> {
    manager.with_file_mut_helper(
        handle,
        |summary| summary_stat!(summary.read_calls += 1),
        |file| file.read(buf),
    )
}

fn write_helper(
    manager: &FileHandleManager,
    handle: &FileHandleManaged,
    buf: &[u8],
) -> Result<usize> {
    manager.with_file_mut_helper(
        handle,
        |summary| summary_stat!(summary.write_calls += 1),
        |file| file.write(buf),
    )
}

fn flush_helper(
    manager: &FileHandleManager,
    handle: &FileHandleManaged,
) -> Result<()> {
    manager.with_file_mut_helper(
        handle,
        |_| {},
        |file| file.flush(),
    )
}

fn seek_helper(
    manager: &FileHandleManager,
    handle: &FileHandleManaged,
    pos: SeekFrom,
) -> Result<u64> {
    manager.with_file_mut_helper(
        handle,
        |summary| summary_stat!(summary.seek_calls += 1),
        |file| file.seek(pos),
    )
}

fn metadata_helper(
    manager: &FileHandleManager,
    handle: &FileHandleManaged,
) -> Result<FileMetadata> {
    manager.with_file_mut_helper(
        handle,
        |summary| summary_stat!(summary.metadata_calls += 1),
        |file| file.metadata(),
    )
}

#[test]
fn test_filetype_handle_counts() {
    assert_eq!(
        filetype_handle_counts(FileType::Text {
            archival_type: FileTypeArchive::Normal,
            encoding_type: FileTypeTextEncoding::Utf8Ascii,
        }),
        (1, 0),
    );
    assert_eq!(
        filetype_handle_counts(FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        }),
        (1, FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT),
    );
    assert_eq!(
        filetype_handle_counts(FileType::Odl {
            archival_type: FileTypeArchive::Normal,
            odl_sub_type: OdlSubType::Odl,
        }),
        (1, FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT),
    );
    assert_eq!(
        filetype_handle_counts(FileType::Journal {
            archival_type: FileTypeArchive::Normal,
        }),
        (1, 1),
    );
    assert_eq!(filetype_handle_counts(FileType::Unparsable), (0, 0));
}

#[test]
fn test_request_open_read_seek_and_metadata_update_summary() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);
    let handle = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
        .unwrap();

    assert_eq!(manager.open_max(), OpenMaxCountType::new(2).unwrap(), "open_max()");
    assert_eq!(metadata_helper(&manager, &handle).unwrap().len(), 6, "metadata_helper() file length");
    assert_eq!(seek_helper(&manager, &handle, SeekFrom::Start(1)).unwrap(), 1, "seek_helper() to position 1");

    let mut buf = [0_u8; 3];
    assert_eq!(read_helper(&manager, &handle, &mut buf).unwrap(), 3, "read_helper() read 3 bytes");
    assert_eq!(&buf, b"bcd", "read_helper() content read");

    let summary = manager.summary();
    assert_ge!(summary.open_max_default, OPEN_MAX_DEFAULT_GT, "open_max_default");
    assert_eq!(summary.request_open_calls, 1, "request_open_calls");
    assert_eq!(summary.metadata_calls, 1, "metadata_calls");
    assert_eq!(summary.seek_calls, 1, "seek_calls");
    assert_eq!(summary.read_calls, 1, "read_calls");
    assert_eq!(summary.physical_open_calls, 1, "physical_open_calls");
    assert_eq!(summary.physical_open_error_calls, 0, "physical_open_error_calls");
    assert_eq!(summary.managed_count_open_hi, 1, "managed_count_open_hi");
    assert_eq!(summary.count_unmanaged_hi, 0, "count_unmanaged_hi");
    assert_eq!(summary.count_hi, 1, "count_hi");
}

#[test]
fn test_evicted_handle_reopens_at_saved_position() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(1);

    let handle_a = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let mut first = [0_u8; 2];
    assert_eq!(read_helper(&manager, &handle_a, &mut first).unwrap(), 2);
    assert_eq!(&first, b"ab");

    let _handle_b = manager
        .request_open_managed(PATH_ID_B, FileHandleRole::PrimaryRead, ntf_b.path(), OpenOptionsManaged::read_only())
        .unwrap();

    let handle_a = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    let mut second = [0_u8; 2];
    assert_eq!(read_helper(&manager, &handle_a, &mut second).unwrap(), 2);
    assert_eq!(&second, b"cd");

    let summary = manager.summary();
    assert_eq!(summary.managed_count_open_hi, 1);
    assert_eq!(summary.count_unmanaged_hi, 0);
    assert_eq!(summary.count_hi, 1);
    assert_eq!(summary.physical_open_calls, 3);
    assert_eq!(summary.physical_reopen_calls, 1);
    assert_eq!(summary.evict_succeed, 2);
}

#[test]
fn test_open_errors_are_counted() {
    let manager = manager(1);
    let missing_path = std::env::temp_dir().join("s4-filehandlemanager-tests-missing-file");

    let err = manager
        .request_open_managed(PATH_ID_C, FileHandleRole::PrimaryRead, &missing_path, OpenOptionsManaged::read_only())
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::NotFound);
    let summary = manager.summary();
    assert_eq!(summary.request_open_calls, 1);
    assert_eq!(summary.physical_open_calls, 0);
    assert_eq!(summary.physical_open_error_calls, 1);
}

#[test]
fn test_too_many_open_files_error_reduces_open_max_evicts_and_retries() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(3);

    let handle_a = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    assert_eq!(manager.count_open(), 1, "count_open() after opening managed handle_a");

    let handle_b = manager
        .request_open_managed(
            PATH_ID_B,
            FileHandleRole::PrimaryRead,
            ntf_b.path(),
            OpenOptionsManaged::read_only_force_open_error(raw_os_error_too_many_open_files(), 1),
        )
        .unwrap();

    assert_eq!(manager.open_max(), OpenMaxCountType::new(1).unwrap(), "open_max()");
    assert_eq!(manager.count_open(), 1, "count_open()");
    let summary = manager.summary();
    assert_ge!(summary.open_max_default, OPEN_MAX_DEFAULT_GT, "open_max_default");
    assert_eq!(summary.physical_open_calls, 2, "physical_open_calls");
    assert_eq!(summary.physical_open_error_calls, 1, "physical_open_error_calls");
    assert_eq!(summary.evict_succeed, 1, "evict_succeed");
    assert_eq!(summary.evict_fails, 0, "evict_fails");

    let mut buf = [0_u8; 2];
    assert_eq!(read_helper(&manager, &handle_b, &mut buf).unwrap(), 2, "read_helper()");
    assert_eq!(&buf, b"wx", "read_helper() content");
    drop(handle_a);
}

#[test]
fn test_other_open_errors_do_not_reduce_open_max_or_evict() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(3);

    let _handle_a = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();

    let err = manager
        .request_open_managed(
            PATH_ID_B,
            FileHandleRole::PrimaryRead,
            ntf_b.path(),
            OpenOptionsManaged::read_only_force_open_error(raw_os_error_not_found(), 1),
        )
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::NotFound, "ErrorKind");
    assert_eq!(manager.open_max(), OpenMaxCountType::new(3).unwrap(), "open_max()");
    assert_eq!(manager.count_open(), 1, "count_open()");
    let summary = manager.summary();
    assert_ge!(summary.open_max_default, OPEN_MAX_DEFAULT_GT, "open_max_default");
    assert_eq!(summary.physical_open_calls, 1, "physical_open_calls");
    assert_eq!(summary.physical_open_error_calls, 1, "physical_open_error_calls");
    assert_eq!(summary.evict_succeed, 0, "evict_succeed");
    assert_eq!(summary.evict_fails, 0, "evict_fails");
}

#[test]
fn test_write_existing_updates_file_and_summary() {
    let ntf: NamedTempFile = create_temp_file("0000");
    let manager = manager(2);
    let handle = manager
        .request_open_managed(
            PATH_ID_A,
            FileHandleRole::SecondaryWrite,
            ntf.path(),
            OpenOptionsManaged::write_existing(),
        )
        .unwrap();

    assert_eq!(write_helper(&manager, &handle, b"xy").unwrap(), 2);
    flush_helper(&manager, &handle).unwrap();

    assert_eq!(std::fs::read(ntf.path()).unwrap(), b"xy00", "File content after write");
    let summary = manager.summary();
    assert_eq!(summary.request_open_calls, 1, "request_open_calls");
    assert_eq!(summary.write_calls, 1, "write_calls");
    assert_eq!(summary.physical_open_calls, 1, "physical_open_calls");
}

#[test]
fn test_drop_last_handle_closes_real_file() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);

    {
        let _handle = manager
            .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
            .unwrap();
        assert_eq!(manager.count_open(), 1, "count_open()");
        assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 1);
    }

    assert_eq!(manager.count_open(), 0, "count_open() after drop");
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 0);

    let handle = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    assert_eq!(manager.count_open(), 1, "count_open()");
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 1);
    drop(handle);
    assert_eq!(manager.count_open(), 0, "count_open() after drop");
}

#[test]
fn test_clone_drop_keeps_file_open_until_last_clone() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);
    let handle = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let handle_clone = handle.clone();

    assert_eq!(manager.count_open(), 1, "count_open()");
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 2);

    drop(handle);
    assert_eq!(manager.count_open(), 1, "count_open() after drop handle");
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 1);

    drop(handle_clone);
    assert_eq!(manager.count_open(), 0, "count_open() after drop handle_clone");
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 0);
}

#[test]
fn test_drop_saves_seek_position_for_later_request_read() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);

    {
        let mut handle = manager
            .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
            .unwrap();
        let mut first = [0_u8; 2];
        assert_eq!(handle.read(&mut first).unwrap(), 2, "read first 2 bytes");
        assert_eq!(&first, b"ab", "first 2 bytes content");
    }

    assert_eq!(manager.count_open(), 0, "count_open() after drop");
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 0);

    let mut handle = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    let mut second = [0_u8; 2];
    assert_eq!(handle.read(&mut second).unwrap(), 2, "read second 2 bytes");
    assert_eq!(&second, b"cd", "second 2 bytes content");
}

#[test]
fn test_handle_unmanaged_reservation_releases_on_drop() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);

    {
        let fpath: FPath = path_to_fpath(&ntf.path());
        let _handle = manager
            .request_open_unmanaged(PATH_ID_A, FileHandleRole::Unmanaged, &fpath)
            .unwrap();
        assert_eq!(manager.count_open(), 0, "count_open() inside scope");
        assert_eq!(manager.count_open_total(), 1, "count_open_total() inside scope");
        assert_eq!(manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged), 1, "handles_unmanaged_helper() inside scope");
    }

    assert_eq!(manager.count_open(), 0, "count_open() after scope");
    assert_eq!(manager.count_open_total(), 0, "count_open_total() after scope");
    assert_eq!(manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged), 0, "handles_unmanaged_helper() after scope");

    let summary = manager.summary();
    assert_eq!(summary.request_open_unmanaged_calls, 1, "request_open_unmanaged_calls");
    assert_eq!(summary.managed_count_open_hi, 0, "managed_count_open_hi");
    assert_eq!(summary.count_unmanaged_hi, 1, "count_unmanaged_hi");
    assert_eq!(summary.count_hi, 1, "count_hi");
}

#[test]
fn test_unmanaged_and_managed_high_water_counts_can_coexist() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(2);

    let _handle_a = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let fpath: FPath = path_to_fpath(&ntf_b.path());
    let _unmanaged = manager
        .request_open_unmanaged(PATH_ID_B, FileHandleRole::Unmanaged, &fpath)
        .unwrap();

    assert_eq!(manager.count_open(), 1, "count_open() after opening managed and unmanaged");
    assert_eq!(manager.count_open_total(), 2, "count_open_total() after opening managed and unmanaged");

    let summary = manager.summary();
    assert_eq!(summary.managed_count_open_hi, 1, "managed_count_open_hi after opening managed and unmanaged");
    assert_eq!(summary.count_unmanaged_hi, 1, "count_unmanaged_hi after opening managed and unmanaged");
    assert_eq!(summary.count_hi, 2, "count_hi after opening managed and unmanaged");
}

#[test]
fn test_pending_unmanaged_plans_do_not_constrain_managed_opens() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize + 1);
    let mut list_files = HashMap::new();
    list_files.insert(
        PATH_ID_B,
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
    );
    list_files.insert(
        PATH_ID_C,
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
    );

    manager.handle_reservations(&list_files);
    assert_eq!(
        manager.handles_unmanaged_pending_helper(PATH_ID_B, FileHandleRole::Unmanaged),
        FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
        "handles_unmanaged_pending_helper() after handle_reservations()"
    );
    assert_eq!(
        manager.handles_unmanaged_pending_helper(PATH_ID_C, FileHandleRole::Unmanaged),
        FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
        "handles_unmanaged_pending_helper() after handle_reservations()"
    );

    let handle_a = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    assert_eq!(manager.count_open(), 1, "count_open() after opening managed handle_a");

    let handle_d = manager
        .request_open_managed(PATH_ID_D, FileHandleRole::PrimaryRead, ntf_b.path(), OpenOptionsManaged::read_only())
        .unwrap();
    assert_eq!(manager.count_open(), 2, "count_open() after opening managed handle_d");
    assert_eq!(
        manager.handles_unmanaged_pending_helper(PATH_ID_B, FileHandleRole::Unmanaged),
        FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
        "handles_unmanaged_pending_helper() after opening managed handle_d"
    );
    assert_eq!(
        manager.handles_unmanaged_pending_helper(PATH_ID_C, FileHandleRole::Unmanaged),
        FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
        "handles_unmanaged_pending_helper() after opening managed handle_d"
    );
    drop(handle_d);
    drop(handle_a);

    let summary = manager.summary();
    assert_eq!(summary.evict_succeed, 0, "evict_succeed after dropping handles");
    assert_eq!(summary.evict_fails, 0, "evict_fails after dropping handles");
}

#[test]
fn test_planned_unmanaged_request_consumes_multi_slot_reservation() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize);
    let mut list_files = HashMap::new();
    list_files.insert(
        PATH_ID_A,
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
    );
    list_files.insert(
        PATH_ID_B,
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
    );

    manager.handle_reservations(&list_files);
    let managed = manager
        .request_open_managed(PATH_ID_C, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let fpath: FPath = path_to_fpath(&ntf.path());
    {
        let _handle = manager
            .request_open_unmanaged(PATH_ID_A, FileHandleRole::Unmanaged, &fpath)
            .unwrap();
        assert_eq!(manager.count_open(), 0, "count_open() inside unmanaged handle scope");
        assert_eq!(manager.count_open_total(), FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as u32, "count_open_total() inside unmanaged handle scope");
        assert_eq!(
            manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged),
            FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
            "handles_unmanaged_helper() inside unmanaged handle scope"
        );
        assert_eq!(manager.handles_unmanaged_pending_helper(PATH_ID_A, FileHandleRole::Unmanaged), 0, "handles_unmanaged_pending_helper() inside unmanaged handle scope");
        assert_eq!(
            manager.handles_unmanaged_pending_helper(PATH_ID_B, FileHandleRole::Unmanaged),
            FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
            "handles_unmanaged_pending_helper() for another file inside unmanaged handle scope"
        );
    }

    assert_eq!(manager.count_open_total(), 0, "count_open_total() after unmanaged handle scope");
    assert_eq!(manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged), 0, "handles_unmanaged_helper() after unmanaged handle scope");
    drop(managed);

    let summary = manager.summary();
    assert_eq!(summary.evict_succeed, 1, "evict_succeed after activating unmanaged reservation");
    assert_eq!(summary.evict_fails, 0, "evict_fails after activating unmanaged reservation");
}

#[test]
fn test_planned_unmanaged_request_failure_preserves_reservation() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize - 1);
    let mut list_files = HashMap::new();
    list_files.insert(
        PATH_ID_A,
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
    );

    manager.handle_reservations(&list_files);
    let fpath: FPath = path_to_fpath(&ntf.path());
    let err = manager
        .request_open_unmanaged(PATH_ID_A, FileHandleRole::Unmanaged, &fpath)
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::WouldBlock, "ErrorKind after oversized unmanaged request");
    assert_eq!(manager.count_open_total(), 0, "count_open_total() after oversized unmanaged request");
    assert_eq!(manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged), 0, "handles_unmanaged_helper() after oversized unmanaged request");
    assert_eq!(
        manager.handles_unmanaged_pending_helper(PATH_ID_A, FileHandleRole::Unmanaged),
        FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize,
        "handles_unmanaged_pending_helper() after oversized unmanaged request"
    );
}

#[test]
fn test_too_many_open_files_retry_preserves_planned_managed_slot() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as usize + 1);
    let mut list_files = HashMap::new();
    list_files.insert(
        PATH_ID_A,
        FileType::Etl {
            archival_type: FileTypeArchive::Normal,
        },
    );
    list_files.insert(
        PATH_ID_B,
        FileType::Text {
            archival_type: FileTypeArchive::Normal,
            encoding_type: FileTypeTextEncoding::Utf8Ascii,
        },
    );

    manager.handle_reservations(&list_files);
    let fpath: FPath = path_to_fpath(&ntf_a.path());
    let unmanaged = manager
        .request_open_unmanaged(PATH_ID_A, FileHandleRole::Unmanaged, &fpath)
        .unwrap();
    assert_eq!(manager.count_open_total(), FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as u32);

    let managed = manager
        .request_open_managed(
            PATH_ID_B,
            FileHandleRole::PrimaryRead,
            ntf_b.path(),
            OpenOptionsManaged::read_only_force_open_error(raw_os_error_too_many_open_files(), 1),
        )
        .unwrap();
    assert_eq!(manager.count_open(), 1, "count_open() after opening managed handle");
    assert_eq!(manager.count_open_total(), FILE_HANDLE_UNMANAGED_PYRUNNER_COUNT as u32 + 1, "count_open_total() after opening managed handle");

    drop(managed);
    drop(unmanaged);
    let summary = manager.summary();
    assert_eq!(summary.physical_open_error_calls, 1, "physical_open_error_calls after dropping handles");
    assert_eq!(summary.physical_open_calls, 1, "physical_open_calls after dropping handles");
    assert_eq!(summary.evict_fails, 0, "evict_fails after dropping handles");
}

#[test]
fn test_handle_unmanaged_reservation_forces_managed_eviction() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(1);

    let handle_a = manager
        .request_open_managed(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    assert_eq!(manager.count_open(), 1, "count_open() after opening managed handle_a");

    let fpath: FPath = path_to_fpath(&ntf_b.path());
    let unmanaged = manager
        .request_open_unmanaged(PATH_ID_B, FileHandleRole::Unmanaged, &fpath)
        .unwrap();
    assert_eq!(manager.count_open(), 0, "count_open() after opening unmanaged handle");
    assert_eq!(manager.count_open_total(), 1, "count_open_total() after opening unmanaged handle");
    assert_eq!(manager.handles_unmanaged_helper(PATH_ID_B, FileHandleRole::Unmanaged), 1, "handles_unmanaged_helper() after opening unmanaged handle");

    let err = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::WouldBlock, "ErrorKind after trying to request_read() on evicted managed handle");

    drop(unmanaged);
    let handle_a_reopened = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    assert_eq!(manager.count_open(), 1, "count_open() after reopening managed handle");
    assert_eq!(manager.count_open_total(), 1, "count_open_total() after reopening managed handle");
    drop(handle_a_reopened);
    drop(handle_a);

    let summary = manager.summary();
    assert_eq!(summary.request_open_unmanaged_calls, 1, "request_open_unmanaged_calls in summary");
    assert_eq!(summary.evict_succeed, 1, "evict_succeed in summary");
    assert_eq!(summary.evict_fails, 1, "evict_fails in summary");
    assert_eq!(summary.managed_count_open_hi, 1, "managed_count_open_hi in summary");
    assert_eq!(summary.count_unmanaged_hi, 1, "count_unmanaged_hi in summary");
    assert_eq!(summary.count_hi, 1, "count_hi in summary");
}
