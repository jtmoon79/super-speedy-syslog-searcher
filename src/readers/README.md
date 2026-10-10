# `src/readers/`

Most of these "Readers" are not Rust `Read` implementations.

However, `AslReader` implements [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) over rendered Apple System Log event bytes; it also exposes individual events for datetime-ordered merging.
`EtlReader` exposes individual events for datetime-ordered merging.
