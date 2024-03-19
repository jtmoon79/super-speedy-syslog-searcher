#!/usr/bin/env bash
#
# run yamllint in one command with parameters I prefer
#
# Requires Python package `yamllint`, see `requirements.txt`
#

set -eu

cd "$(dirname -- "${0}")/.."

PYTHON=${PYTHON-$(which python 2>/dev/null || which python3 2>/dev/null)}

set -x

"${PYTHON}" -m yamllint --version

exec \
  "${PYTHON}" -m yamllint \
    --config-file ./tools/yamllint.yml \
    "./tools/yamllint.yml" \
    "./.github/workflows/rust.yml" \
    "./.github/codecov.yml" \
    "./.github/dependabot.yml" \
   "${@}"
