#!/usr/bin/env bash
#
# run `cargo doc` with preferred options
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo doc --locked --release --frozen --no-deps -v "${@}"
