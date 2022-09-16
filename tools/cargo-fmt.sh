#!/usr/bin/env bash
#
# call `cargo fmt` with options I prefer
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo fmt --verbose "${@}"

