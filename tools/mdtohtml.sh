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

css_file=$(mktemp --suffix=.css)
trap 'rm -f -- "${css_file}"' EXIT
echo '
table {
  border-collapse: collapse;
  width: 100%;
}

th,
td {
  border: 1px solid #666;
  padding: 0.5rem 0.75rem;
  text-align: left;
}

th {
  background: #e9ecef;
  font-weight: 700;
  border-bottom: 2px solid #333;
}
' > "${css_file}"

set -x

which mdtohtml
exec mdtohtml  -css "${css_file}" -page "$file" "${@}" > "${file_html}"
