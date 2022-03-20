#!/usr/bin/env bash
#
# run `cargo-test` in one command with parameters I strongly prefer

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo test -j1 "${@}" -- --test-threads=1
