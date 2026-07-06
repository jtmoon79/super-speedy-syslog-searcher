// src/tests/filehandlemanager_tests.rs
//
// this file initially created by Co-pilot + GPT-5.5, heavily revised by human @jtmoon79

//! Tests for `FileHandleManager`.

use std::io::{
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom,
    Write,
};

use crate::common::{
    FileMetadata,
    PathId,
    summary_stat,
    summary_stats_enable,
};
use crate::debug::helpers::{
    create_temp_file,
    NamedTempFile,
};
use crate::readers::filehandlemanager::{
    FileHandleManager,
    FileHandleRole,
    FileHandleManaged,
    OpenOptionsManaged,
    OpenMaxCountType,
};

const PATH_ID_A: PathId = 99_000;
const PATH_ID_B: PathId = 99_001;
const PATH_ID_C: PathId = 99_002;

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
fn test_request_open_read_seek_and_metadata_update_summary() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);
    let handle = manager
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
        .unwrap();

    assert_eq!(manager.open_max(), OpenMaxCountType::new(2).unwrap());
    assert_eq!(metadata_helper(&manager, &handle).unwrap().len(), 6);
    assert_eq!(seek_helper(&manager, &handle, SeekFrom::Start(1)).unwrap(), 1);

    let mut buf = [0_u8; 3];
    assert_eq!(read_helper(&manager, &handle, &mut buf).unwrap(), 3);
    assert_eq!(&buf, b"bcd");

    let summary = manager.summary();
    assert_eq!(summary.open_max_default, 2);
    assert_eq!(summary.request_open_calls, 1);
    assert_eq!(summary.metadata_calls, 1);
    assert_eq!(summary.seek_calls, 1);
    assert_eq!(summary.read_calls, 1);
    assert_eq!(summary.physical_open_calls, 1);
    assert_eq!(summary.physical_open_error_calls, 0);
    assert_eq!(summary.managed_open_count_hi, 1);
    assert_eq!(summary.unmanaged_count_hi, 0);
    assert_eq!(summary.count_hi, 1);
}

#[test]
fn test_evicted_handle_reopens_at_saved_position() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(1);

    let handle_a = manager
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let mut first = [0_u8; 2];
    assert_eq!(read_helper(&manager, &handle_a, &mut first).unwrap(), 2);
    assert_eq!(&first, b"ab");

    let _handle_b = manager
        .request_open(PATH_ID_B, FileHandleRole::PrimaryRead, ntf_b.path(), OpenOptionsManaged::read_only())
        .unwrap();

    let handle_a = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    let mut second = [0_u8; 2];
    assert_eq!(read_helper(&manager, &handle_a, &mut second).unwrap(), 2);
    assert_eq!(&second, b"cd");

    let summary = manager.summary();
    assert_eq!(summary.managed_open_count_hi, 1);
    assert_eq!(summary.unmanaged_count_hi, 0);
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
        .request_open(PATH_ID_C, FileHandleRole::PrimaryRead, &missing_path, OpenOptionsManaged::read_only())
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
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    assert_eq!(manager.open_count(), 1);

    let handle_b = manager
        .request_open(
            PATH_ID_B,
            FileHandleRole::PrimaryRead,
            ntf_b.path(),
            OpenOptionsManaged::read_only_force_open_error(raw_os_error_too_many_open_files(), 1),
        )
        .unwrap();

    assert_eq!(manager.open_max(), OpenMaxCountType::new(1).unwrap());
    assert_eq!(manager.open_count(), 1);
    let summary = manager.summary();
    assert_eq!(summary.open_max_default, 3);
    assert_eq!(summary.physical_open_calls, 2);
    assert_eq!(summary.physical_open_error_calls, 1);
    assert_eq!(summary.evict_succeed, 1);
    assert_eq!(summary.evict_fails, 0);

    let mut buf = [0_u8; 2];
    assert_eq!(read_helper(&manager, &handle_b, &mut buf).unwrap(), 2);
    assert_eq!(&buf, b"wx");
    drop(handle_a);
}

#[test]
fn test_other_open_errors_do_not_reduce_open_max_or_evict() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(3);

    let _handle_a = manager
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();

    let err = manager
        .request_open(
            PATH_ID_B,
            FileHandleRole::PrimaryRead,
            ntf_b.path(),
            OpenOptionsManaged::read_only_force_open_error(raw_os_error_not_found(), 1),
        )
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::NotFound);
    assert_eq!(manager.open_max(), OpenMaxCountType::new(3).unwrap());
    assert_eq!(manager.open_count(), 1);
    let summary = manager.summary();
    assert_eq!(summary.open_max_default, 3);
    assert_eq!(summary.physical_open_calls, 1);
    assert_eq!(summary.physical_open_error_calls, 1);
    assert_eq!(summary.evict_succeed, 0);
    assert_eq!(summary.evict_fails, 0);
}

#[test]
fn test_write_existing_updates_file_and_summary() {
    let ntf: NamedTempFile = create_temp_file("0000");
    let manager = manager(2);
    let handle = manager
        .request_open(
            PATH_ID_A,
            FileHandleRole::SecondaryWrite,
            ntf.path(),
            OpenOptionsManaged::write_existing(),
        )
        .unwrap();

    assert_eq!(write_helper(&manager, &handle, b"xy").unwrap(), 2);
    flush_helper(&manager, &handle).unwrap();

    assert_eq!(std::fs::read(ntf.path()).unwrap(), b"xy00");
    let summary = manager.summary();
    assert_eq!(summary.request_open_calls, 1);
    assert_eq!(summary.write_calls, 1);
    assert_eq!(summary.physical_open_calls, 1);
}

#[test]
fn test_drop_last_handle_closes_real_file() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);

    {
        let _handle = manager
            .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
            .unwrap();
        assert_eq!(manager.open_count(), 1);
        assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 1);
    }

    assert_eq!(manager.open_count(), 0);
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 0);

    let handle = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    assert_eq!(manager.open_count(), 1);
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 1);
    drop(handle);
    assert_eq!(manager.open_count(), 0);
}

#[test]
fn test_clone_drop_keeps_file_open_until_last_clone() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);
    let handle = manager
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let handle_clone = handle.clone();

    assert_eq!(manager.open_count(), 1);
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 2);

    drop(handle);
    assert_eq!(manager.open_count(), 1);
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 1);

    drop(handle_clone);
    assert_eq!(manager.open_count(), 0);
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 0);
}

#[test]
fn test_drop_saves_seek_position_for_later_request_read() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);

    {
        let mut handle = manager
            .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf.path(), OpenOptionsManaged::read_only())
            .unwrap();
        let mut first = [0_u8; 2];
        assert_eq!(handle.read(&mut first).unwrap(), 2);
        assert_eq!(&first, b"ab");
    }

    assert_eq!(manager.open_count(), 0);
    assert_eq!(manager.handles_managed_active_helper(PATH_ID_A, FileHandleRole::PrimaryRead), 0);

    let mut handle = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    let mut second = [0_u8; 2];
    assert_eq!(handle.read(&mut second).unwrap(), 2);
    assert_eq!(&second, b"cd");
}

#[test]
fn test_unmanaged_handle_reservation_releases_on_drop() {
    let ntf: NamedTempFile = create_temp_file("abcdef");
    let manager = manager(2);

    {
        let _handle = manager
            .request_unmanaged_open(PATH_ID_A, FileHandleRole::Unmanaged, ntf.path())
            .unwrap();
        assert_eq!(manager.open_count(), 0);
        assert_eq!(manager.total_open_count(), 1);
        assert_eq!(manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged), 1);
    }

    assert_eq!(manager.open_count(), 0);
    assert_eq!(manager.total_open_count(), 0);
    assert_eq!(manager.handles_unmanaged_helper(PATH_ID_A, FileHandleRole::Unmanaged), 0);

    let summary = manager.summary();
    assert_eq!(summary.request_unmanaged_open_calls, 1);
    assert_eq!(summary.managed_open_count_hi, 0);
    assert_eq!(summary.unmanaged_count_hi, 1);
    assert_eq!(summary.count_hi, 1);
}

#[test]
fn test_unmanaged_and_managed_high_water_counts_can_coexist() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(2);

    let _handle_a = manager
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    let _unmanaged = manager
        .request_unmanaged_open(PATH_ID_B, FileHandleRole::Unmanaged, ntf_b.path())
        .unwrap();

    assert_eq!(manager.open_count(), 1);
    assert_eq!(manager.total_open_count(), 2);

    let summary = manager.summary();
    assert_eq!(summary.managed_open_count_hi, 1);
    assert_eq!(summary.unmanaged_count_hi, 1);
    assert_eq!(summary.count_hi, 2);
}

#[test]
fn test_unmanaged_handle_reservation_forces_managed_eviction() {
    let ntf_a: NamedTempFile = create_temp_file("abcdef");
    let ntf_b: NamedTempFile = create_temp_file("wxyz");
    let manager = manager(1);

    let handle_a = manager
        .request_open(PATH_ID_A, FileHandleRole::PrimaryRead, ntf_a.path(), OpenOptionsManaged::read_only())
        .unwrap();
    assert_eq!(manager.open_count(), 1);

    let unmanaged = manager
        .request_unmanaged_open(PATH_ID_B, FileHandleRole::Unmanaged, ntf_b.path())
        .unwrap();
    assert_eq!(manager.open_count(), 0);
    assert_eq!(manager.total_open_count(), 1);
    assert_eq!(manager.handles_unmanaged_helper(PATH_ID_B, FileHandleRole::Unmanaged), 1);

    let err = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::WouldBlock);

    drop(unmanaged);
    let handle_a_reopened = manager
        .request_read(PATH_ID_A, FileHandleRole::PrimaryRead)
        .unwrap();
    assert_eq!(manager.open_count(), 1);
    assert_eq!(manager.total_open_count(), 1);
    drop(handle_a_reopened);
    drop(handle_a);

    let summary = manager.summary();
    assert_eq!(summary.request_unmanaged_open_calls, 1);
    assert_eq!(summary.evict_succeed, 1);
    assert_eq!(summary.evict_fails, 1);
    assert_eq!(summary.managed_open_count_hi, 1);
    assert_eq!(summary.unmanaged_count_hi, 1);
    assert_eq!(summary.count_hi, 1);
}
