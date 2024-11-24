#!/usr/bin/env bash
#
# run `cargo deny` with preferred options
#
# https://github.com/EmbarkStudios/cargo-deny

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

set -x

cargo deny --version

exec cargo deny check --show-stats advisories ban bans sources "${@}"
