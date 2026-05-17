#!/usr/bin/env bash
#
# run `mdtohtml` with preferred options
#
#     go install github.com/gomarkdown/mdtohtml@latest
#

set -eu

cd "$(dirname -- "${0}")/.."

export PATH="${PATH}:${HOME}/go/bin"

file=${1}
file_html="${file}.html"
shift

set -x

which mdtohtml
exec mdtohtml -page "$file" "${@}" > "${file_html}"
