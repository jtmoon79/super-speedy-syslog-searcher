#!/usr/bin/env bash
#
# run `cargo modules` with preferred options
#
# to install:
#     cargo install cargo-modules

set -eu

cd "$(dirname -- "${0}")/.."

set -x

cargo modules --version

exec cargo modules structure "${@}"
