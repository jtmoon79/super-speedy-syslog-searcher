#!/usr/bin/env bash
#
# print `-Zmacro-stats`
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo +nightly rustc --bin s4 -- -Zmacro-stats "$@"
