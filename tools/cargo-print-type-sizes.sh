#!/usr/bin/env bash
#
# print `-Zprint-type-sizes`
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo +nightly rustc --bin s4 -- -Zprint-type-sizes "$@"
