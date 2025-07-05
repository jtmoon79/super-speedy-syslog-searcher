# bindgen

This is ad-hoc "sub-project" to generate [`bindgen`] bindings for systemd
header file [`sd-journal.h`]. This project is used to create static
`bindings.rs`. See instructions in `build.rs`.

In the long-run, building the bindings during `cargo install` would be done.
However, supporting that correctly for many platforms is difficult.
So a hardcoded `bindings.rs` file is committed to the parent project.

[`sd-journal.h`]: https://www.man7.org/linux/man-pages/man3/sd-journal.3.html
[`bindgen`]: https://crates.io/crates/bindgen
