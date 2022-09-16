#!/usr/bin/env bash
#
# call `cargo fmt` with preferred options
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

cargo fmt --version

exec cargo fmt --verbose "${@}"

