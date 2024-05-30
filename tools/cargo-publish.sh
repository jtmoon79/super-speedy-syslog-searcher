#!/usr/bin/env bash
#
# run `cargo publish` with preferred options
# pass --no-dry-run to actually publish
#

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

arg_dry_run='--dry-run'
if [[ ${#} -eq 1 ]] && [[ "${1}" == "--no-dry-run" ]]; then
    arg_dry_run=
    shift
fi

set -x

cargo --version

exec cargo publish \
    --verbose \
    --locked \
    --all-features \
    ${arg_dry_run} \
    "${@}" \
