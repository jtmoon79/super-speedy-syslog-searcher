#!/usr/bin/env bash
#
# run `cargo bloat` with preferred options
#
# install with:
#    cargo install --locked cargo-bloat
#

set -eu

DIROUT=${DIROUT-.}

(
    set -x
    cargo bloat --version
)

export OUT="${DIROUT}/bloat.txt"
(
    set -x
    cargo bloat --locked --release --all-features --wide "${@}"
) | tee "${OUT}"

export OUT="${DIROUT}/bloat-s4lib.txt"
(
    set -x
    cargo bloat --locked --release --all-features --wide -n 9999 | grep -Ee '^ File |  s4lib '
) | tee "${OUT}"

export OUT="${DIROUT}/bloat-crates.txt"
(
    set -x
    cargo bloat --locked --release --all-features --wide --crates "${@}"
) | tee "${OUT}"
