#!/usr/bin/env bash
#
# run `mdtohtml` with preferred options
#
#     go install github.com/gomarkdown/mdtohtml@latest
#

set -eu

cd "$(dirname -- "${0}")/.."

# in case `mdtohtml` is not in the PATH, add the default Go bin directory to the PATH
export PATH="${PATH}:${HOME}/go/bin"

file=${1}
file_html="${file}.html"
shift

css_snippet=$(cat <<'CSS'
table {  border-collapse: collapse; width: 100%;}

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
CSS
)

set -x

which mdtohtml

# convert MD to HTML and use `awk` to embed the CSS midstream for writing into `$file_html`
mdtohtml -page "$file" "${@}" | \
  awk -v css="${css_snippet}" '
  BEGIN { style = "<style>\n" css "\n</style>"; inserted = 0 }
  {
    if (!inserted && $0 ~ /<\/[Hh][Ee][Aa][Dd]>/) {
      print style
      inserted = 1
    }
    print
  }
  END {
    if (!inserted) {
      print style
    }
  }
' > "${file_html}"
