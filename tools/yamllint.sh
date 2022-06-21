#!/usr/bin/env bash
#
# run yamllint in one command with parameters I prefer

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec \
  yamllint \
    "./.github/workflows/rust.yml" \
   "${@}"
