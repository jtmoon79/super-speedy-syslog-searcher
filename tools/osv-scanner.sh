#!/usr/bin/env bash
#
# run `osv-scanner` with preferred options
#
# https://google.github.io/osv-scanner/installation/
#

set -eu

cd "$(dirname -- "${0}")/.."

export PATH=${PATH}:~/.cargo/bin
set -x

osv-scanner help
exec osv-scanner scan -r --call-analysis=rust .
