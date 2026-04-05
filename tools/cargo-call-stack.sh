#!/usr/bin/env bash
#
# run `cargo call-stack`
#

set -eu

cd "$(dirname -- "${0}")/.."

if !(cargo install --list | grep -q '^cargo-call-stack'); then
    echo "cargo-call-stack is not installed" >&2
    echo "see section Installation https://lib.rs/crates/cargo-call-stack" >&2
    exit 1
fi

DIROUT=${DIROUT-.}
CALLGRAPH_DOT="${DIROUT}/s4.callgraph.dot"
CALLGRAPH_SVG="${DIROUT}/s4.callgraph.svg"

TRIPLE=$(rustc -vV | sed -n 's|host: ||p')

set -x
cargo call-stack --version
cargo clean
cargo +nightly build --release --config lto='"fat"'
RUST_BACKTRACE=1 cargo +nightly-2023-11-13 call-stack --bin s4 --target "${TRIPLE}" s4::main > "${CALLGRAPH_DOT}"
dot -Tsvg "${CALLGRAPH_DOT}" > "${CALLGRAPH_SVG}"
