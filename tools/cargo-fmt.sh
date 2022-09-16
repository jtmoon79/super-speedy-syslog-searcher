#!/usr/bin/env bash
#
# call `cargo fmt` with preferred options
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo fmt --verbose "${@}"

