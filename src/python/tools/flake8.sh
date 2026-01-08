
#!/usr/bin/env bash
#
# helper script to run flake8 linter with pyproject.toml config
#

D=$(dirname "$0")

set -eux

cd "$D/../s4_event_readers"

exec \
    python -m flake8 --toml-config pyproject.toml "${@}"
