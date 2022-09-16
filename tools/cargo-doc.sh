#!/usr/bin/env bash
#
# run `cargo doc` with preferred options
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

cargo doc --version

exec cargo doc --locked --release --frozen --no-deps -v "${@}"
