#!/usr/bin/env bash
#
# run `glow` with preferred options
#
#     go install github.com/gomarkdown/mdtohtml@latest
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

# in case `glow` is not in the PATH, add the default Go bin directory to the PATH
if ! command -v glow &>/dev/null; then
    export PATH="${PATH}:${HOME}/go/bin"
    if ! command -v glow &>/dev/null; then
        echo "ERROR: glow is not installed." >&2
        echo "       go install 'github.com/charmbracelet/glow/v2@latest'" >&2
        exit 1
    fi
fi

# add a sensible default width
declare -ir col=$(tput cols)
declare -a width_args=("--width" "${col}")
# if user passed width then drop the sensible default
for arg in "$@"; do
    if [[ "$arg" = "--width" || "$arg" = "-w" ]]; then
        width_args=()
        break
    fi
done

set -x
glow "${width_args[@]}" "${@}"
