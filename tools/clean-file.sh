#!/usr/bin/env bash
#
# quick helper to remove specific names from a file

set -euo pipefail

function re_escape () {
    ${PYTHON-python3} -c 'import re; s = """ '"${1}"' """; se = re.escape(s[1:-1]); print(se)'
}

function clean_file () {
    # remove specific names from the passed file path $1
    # carefully add exprssions for `sed`

    declare -a args=(
        -e "s|$(hostname)|host|g"
    )
    declare -r rp=$(re_escape "$(readlink -f .)")
    if [[ "$rp" ]]; then
        args+=(-e "s|${rp}|.|g")
    fi
    declare -r home=$(re_escape "${HOME}")
    if [[ "$home" ]]; then
        args+=(-e "s|${home}|/home|g")
    fi
    if [[ "$USER" ]]; then
        args+=(-e "s|${USER}|user|g")
    fi
    set -x
    exec sed -i \
        "${args[@]}" \
        -- \
        "${1}"
}

clean_file "${1}"
