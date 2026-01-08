#!/usr/bin/env bash
#
# quick helper to remove specific names from a file

set -euo pipefail

function clean_file () {
    # remove specific names from the passed file path $1
    set -x
    exec sed -i \
        -e "s|$(realpath .)|.|g" \
        -e "s|${HOME}|/home|g" \
        -e "s|$(hostname)|host|g" \
        -e "s|${USER}|user|g" \
        -- \
        "${1}"
}

clean_file "${1}"
