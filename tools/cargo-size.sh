#!/usr/bin/env bash
#
# run `cargo size` with preferred options
#
# install with:
#    cargo install --locked cargo-size
#

set -eu

DIROUT=${DIROUT-.}

(
    set -x
    cargo-size --version
)

export OUT="${DIROUT}/size.txt"
(
    set -x
    cargo-size check --release
) | tee "${OUT}"
