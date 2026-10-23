#!/usr/bin/env bash
#
# run `cargo deb` with preferred options
#
# install:
#   cargo install cargo-deb --locked
#

set -eu

cd "$(dirname -- "${0}")/../.."

(
    set -x
    cargo deb --version
    cargo deb \
        --multiarch same \
        --color always \
        --no-dbgsym \
        --locked \
        --section optional \
        "${@}"
)
