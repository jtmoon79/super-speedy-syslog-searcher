#!/usr/bin/env bash
#
# run yamllint in one command with parameters I prefer
#

set -eu

cd "$(dirname -- "${0}")/.."

set -x

"${PYTHON-python}" -m yamllint --version

exec \
  "${PYTHON-python}" -m yamllint \
    "./.github/workflows/rust.yml" \
    "./.github/codecov.yml" \
   "${@}"
