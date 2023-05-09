#!/usr/bin/env bash
#
# run yamllint in one command with parameters I prefer
#
# Requires Python package `yamllint`
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

"${PYTHON-python3}" -m yamllint --version

exec \
  "${PYTHON-python3}" -m yamllint \
    --config-file ./tools/yamllint.yml \
    "./tools/yamllint.yml" \
    "./.github/workflows/rust.yml" \
    "./.github/codecov.yml" \
    "./.github/dependabot.yml" \
   "${@}"
