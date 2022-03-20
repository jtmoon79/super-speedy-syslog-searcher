#!/usr/bin/env bash

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec cargo test -j1 "${@}" -- --test-threads=1
