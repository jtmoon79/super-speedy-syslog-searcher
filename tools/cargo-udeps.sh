#!/usr/bin/env bash
#
# run `cargo udeps` with preferred options
#
# to install:
#     cargo install cargo-udeps --locked

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo +nightly udeps "${@}"
