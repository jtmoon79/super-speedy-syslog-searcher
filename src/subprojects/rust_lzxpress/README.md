# `MagnetForensics/rust-lzxpress` rip

Project [`rust-lzxpress`](https://github.com/MagnetForensics/rust-lzxpress) v0.7.1 partially copied into here.
Only the MS-XCA plain LZ77 `decompress` function (`src/data.rs`) and `Error` (`src/error.rs`) are kept.
The `compress` function, the LZNT1 codec, the `cc` build script, and the Windows-only `ntapi` dependency are dropped.

Used by the `EtlReader` to decompress `.etl` buffers marked `ETW_BUFFER_FLAG_COMPRESSED`.

Licensed under the MIT License; see `LICENSE`.
