Python scripts intended to be called by Super Speedy Syslog Searcher (s4) via a Python interpreter.

These Python scripts use log parsers implemented in Python and not available in Rust.

Script `ccl_asldb.py` reads Apple System Log (`.asl`) files.

The build script stages only ASL package sources and dependency manifests before
embedding them. Local `build`, `dist`, and bytecode caches are not embedded.
