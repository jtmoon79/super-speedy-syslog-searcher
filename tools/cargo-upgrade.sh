#!/usr/bin/env bash
#
# run `cargo upgrade` with preferred options
#
# to install:
#     cargo install cargo-edit

set -eu

cd "$(dirname -- "${0}")/.."

if ! cargo upgrade --version; then
    echo "Is cargo upgrade installed?" >&2
    echo "    cargo install cargo-edit --locked" >&2
    exit 1
fi

(
set -x
cargo upgrade --verbose "${@}"
)

echo >&2
echo "To update Cargo.lock, run:" >&2
echo "    cargo update --verbose --locked" >&2
