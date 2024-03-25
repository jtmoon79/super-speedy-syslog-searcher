#!/usr/bin/env bash
#
# run `cargo outdated` with preferred options
#
# to install:
#     cargo install cargo-outdated

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo outdated "${@}"
