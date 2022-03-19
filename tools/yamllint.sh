#!/usr/bin/env bash

set -eu
cd "$(dirname -- "${0}")/.."
set -x
exec \
  yamllint \
    "./.github/workflows/rust.yml" \
   "${@}"

