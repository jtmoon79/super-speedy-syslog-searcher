#!/usr/bin/env bash
#
# run `osv-scanner` with preferred options
#
# https://google.github.io/osv-scanner/installation/
#

set -eu

cd "$(dirname -- "${0}")/.."

export PATH="${PATH}:${HOME}/.cargo/bin"

set -x

which osv-scanner
osv-scanner --version
exec osv-scanner scan --recursive --call-analysis=rust "$@" .
