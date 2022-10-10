#!/usr/bin/env bash
#
# changelog-link-gen.sh
#
# Helper script for updating the `CHANGELOG.md`.
#
# Read the `CHANGELOG.md`, print the addendum of full URL links.
# The script-user can copy+paste this over the bottom of the `CHANGELOG.md`.
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

readonly URL_PROJECT='https://github.com/jtmoon79/super-speedy-syslog-searcher'
CHANGELOG='./CHANGELOG.md'

# match Issue link
#
#    [Issue #3]
#
# prints
#
#    [Issue #3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/3
#
grep -oEe '\[Issue #([[:digit:]]+)\]' -- "${CHANGELOG}" \
    | tr -d '[]' \
    | sort -n -t '#' -k2 \
    | sed -Ee 's|^Issue #([[:digit:]]+)$|[Issue #\1]: '"${URL_PROJECT}"'/issues/\1|g' \
    | uniq

# match tag comparison link, e.g.
#
#    [0.0.25...0.0.26]
#
# prints
#
#    [0.0.25...0.0.26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.25...0.0.26
#
grep -oEe '\[[[:digit:]]{1,2}\.[[:digit:]]{1,2}\.[[:digit:]]{1,2}\.\.\.[[:digit:]]{1,2}\.[[:digit:]]{1,2}\.[[:digit:]]{1,2}\]' -- "${CHANGELOG}" \
    | tr -d '[]' \
    | sort -n -t '.' -k1 -k2 -k3 \
    | sed -Ee 's|^(.+)$|[\1]: '"${URL_PROJECT}"'/compare/\1|g' \
    | uniq

# match full git hash link, e.g.
#
#    - add XZ support ([607a23c00aff0d9b34fb3d678bdfd5c14290582d])
#
# prints
#
#    [607a23c00aff0d9b34fb3d678bdfd5c14290582d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/adf400700122f4eb23fd63971b3f048e014d1781
#
grep -oEe '\[[[:xdigit:]]{40}\]' -- "${CHANGELOG}" \
    | tr -d '[]' \
    | sed -Ee 's|^(.+)$|[\1]: '"${URL_PROJECT}"'/commit/\1|g' \
    | sort | uniq
