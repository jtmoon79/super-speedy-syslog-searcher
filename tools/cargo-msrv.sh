#!/usr/bin/env bash
#
# run `cargo msrv` with preferred options
#
# to install:
#     cargo install cargo-msrv

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo msrv verify "${@}"
