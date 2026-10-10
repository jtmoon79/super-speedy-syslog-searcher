# About

`s4` reads log files and parses log lines, keying on datetime patterns. Each processed log uses one thread.

## Project Structure

- multiple `*Reader` to read and parse log lines.
  These are not `std::io::Read` implementations merely they "_read_" logs.
  - `JournalReader` in `src/readers/journalreader.rs` reads systemd journal files using `libsystemd`
  - `EvtxReader` in `src/readers/evtxreader.rs` reads Windows Event Log files using `EvtxParser`
  - `SyslogProcessor` in `src/readers/syslogprocessor.rs` reads syslog files using `regex` patterns. It drives a `SyslineReader` in `src/readers/syslinereader.rs` which composes `Sysline`. `SyslineReader` drives a `LineReader` in `src/readers/linereader.rs` which composes `Line`. `LineReader` drives a `BlockReader` in `src/readers/blockreader.rs` which composes `Block`. The `BlockReader` reads directly from disk. It uses the global singleton `FILE_HANDLE_MANAGER` in `src/readers/filehandlemanager.rs`.
  - `FixedStuctReader` in `src/readers/fixedstructreader.rs` reads `utmp`, `wtmp`, and similar Unix accounting files. It drives a `BlockReader`.
  - `AdlReader` in `src/readers/adlreader.rs` reads Apple System Log files. It has it's own parser `AdlParser` in `src/readers/adlparser.rs`.
  - `OdlReader` in `src/readers/odlreader.rs` reads OneDrive Log files. It has its own parser `OdlParser` in `src/readers/odlparser.rs`.

- Threads are launched in `src/s4/s4.rs` function `exec_fileprocessor_thread` which launches the appropriate `*Reader` based on the `FileType`.
  - The main "game loop" program driver is function `processing_loop`.
- A pre-processing phase driven by `s4.rs` will selectively gather a list of files from user-passed paths.
  - This includes file listings for `.tar` archive files. This uses function `process_path` from `src/readers/filepreprocessor.rs`.

## Project Directories

- `src/bindings/` are C bindings to `libsystemd` journal functions.
- `src/data/` are data various definitions.
- `src/debug/` is debug and development helpers.
- `src/libload/` is the dynamic library loader for `libsystemd`.
- `src/readers/` defines `*Reader` implementations.
- `src/printer/` defines the printer functions in `src/printer/printers.rs`. Printing to stdout is driven by the main thread.
- `src/python/` defines the Python runner and the Python scripts in `src/python/s4_event_readers/`. These Python scripts are able to read `.odl`, `.asl`, and `.etl` files.
- `src/s4/` defines the main `s4` binary program.
  - `src/s4/s4.rs` defines the main CLI program and the main driver thread.
- `src/subprojects` has copied repositories from external projects that are compiled directly into the project.
- `src/tests/` are unit tests.
- `subprojects/ere/` is a subproject for the `ere` crate which is a compile-time regex engine used by `s4`.
- `tools/` are various development scripts.

## Development tips

- start bash shells with options `--norc --noprofile`.
  Or use plain POSIX `sh` shells.
  Plain POSIX `sh` avoids common prompt error `bash: __zoxide_hook: command not found`.
- to build a _release_ build faster, build profile `release_Opt0` which inherits from `release` but no optimizations so it compiles faster
- do not make changes to files using `cargo fmt` or `rustfmt`. The format changes will obscure the intention of the important code change.
