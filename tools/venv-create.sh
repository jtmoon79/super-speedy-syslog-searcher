#!/usr/bin/env bash
#
# venv-create.sh
#
# Create a Python virtual environment and install dependencies for project development.

set -euo pipefail

cd "$(dirname "${0}")/.."

PYTHON=${PYTHON-python3}

(
    set -x
    "${PYTHON}" -m venv .venv --prompt "s4/.venv" --copies --upgrade-deps
    source .venv/bin/activate
    "${PYTHON}" -m pip install -r ./tools/requirements.txt
    "${PYTHON}" -m pip install -r ./tools/compare-log-mergers/requirements.txt
    "${PYTHON}" -m pip install -r ./src/python/s4_event_readers/requirements-dev.txt
)

echo "
To activate:
    source \"${PWD}/.venv/bin/activate\"
"
