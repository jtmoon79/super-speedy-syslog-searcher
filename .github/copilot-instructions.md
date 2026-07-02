# Copilot instructions for this repository

## Background

This project builds binary program `s4`.
`s4` takes a list of filesystem paths from user. Each path is a file or directory.
`s4` examines the paths of files. For those with appropriate file extensions, `s4` reads the files and searches for log messages that match user-specified search criteria.
The type of file is determined by the file extension. The file type is used to determine how to parse the file and extract log messages.

## Directories of note

- `.github/`: GitHub configuration files
- `.vscode/`: Visual Studio Code configuration files
- `benches/`: benchmarks
- `logs/`: contrived and found log files used for various testing and development
- `releases/`: release artifacts created per release including various performance measurements
- `src/s4/`: source code for the `s4` program binary
- `src/data/`: source code for the data structures used by _Readers_
- `src/readers/`: source code for the file _Readers_
- `src/tests/`: source code for unit tests and integration tests
- `subprojects/`: source code for various Rust subprojects that have been grafted into the main project
                 these have been modified to be workspaces based on the top-level `Cargo.toml` file.
- `tools/`: source code for various scripts used in development and testing

## Files of note

- `.github/copilot-instructions.md`: this file, which contains instructions for GitHub Copilot
- `Cargo.toml`: top-level Cargo configuration file for the project
- `Cargo.lock`: top-level Cargo lock file for the project
- `README.md`: top-level README file for the project
- `CHANGELOG.md`: Change log file for the project
- `LICENSE`: license file for the project

## Recommendations for code reviews

- Ignore files that match the glob patterns specificed by `package.exclude` glob patterns in file `Cargo.toml`.
- Check for rust compatible with the rust version specified by the `package.rust-version` in file `Cargo.toml`.
- Check for rust expressions or statements that could use succinct or more idiomatic alternatives.
