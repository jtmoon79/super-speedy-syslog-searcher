#!/usr/bin/env bash
#
# run `cargo clippy` with preferred options
#

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

set -x

cargo clippy --version

exec cargo clippy \
    --verbose \
    --no-deps \
    --all-targets \
    --all-features \
    "${@}" \
    -- \
    -D warnings \
