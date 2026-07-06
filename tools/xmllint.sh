#!/usr/bin/env bash
#
# wrapper to run `xmllint` with preferred options

set -euo pipefail

# check for xmllint
if ! XMLLINT=$(which xmllint); then
    echo "ERROR: xmllint not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install libxml2-utils" >&2
    exit 1
fi

readonly XMLLINT
(set -ueo pipefail; (set -x; "$XMLLINT" --version 2>&1) | head -n1)

if [[ $# -lt 1 ]]; then
    echo "Usage: $(basename -- "${0}") <file.xml>" >&2
    exit 1
fi

file="${1}"
shift

tmp_file=$(mktemp "/tmp/$(basename "${file}").tmp.XXXXXX")
function exit_ () {
    rm -f "${tmp_file}"
}
trap exit_ EXIT

arg_html=''
if [[ "${file}" == *.html ]]; then
    arg_html='--html'
fi

(
    set -x
    "$XMLLINT" --pretty 1 ${arg_html} --recover --huge --format "${file}" --output "${tmp_file}" "${@}"
)

mv -vf "${tmp_file}" "${file}"
