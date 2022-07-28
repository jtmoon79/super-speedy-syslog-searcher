#!/usr/bin/env bash
#
# run `cargo doc` with preferred options
#

set -eux

exec cargo doc --no-deps --frozen --release
