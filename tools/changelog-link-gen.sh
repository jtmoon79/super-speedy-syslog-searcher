#!/usr/bin/env bash
#
# changelog-link-gen.sh
#
# Helper script for updating the `CHANGELOG.md`.
#
# Read the `CHANGELOG.md`, print the addendum of full URL links.
# AUTOMATICALLY MODIFIES CHANGELOG.md!
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

readonly URL_PROJECT=${URL_PROJECT-'https://github.com/jtmoon79/super-speedy-syslog-searcher'}
CHANGELOG='./CHANGELOG.md'
tmp_CHANGELOG=$(mktemp)
tmp_links=$(mktemp)
trap "rm -f -- ${tmp_CHANGELOG} ${tmp_links}" EXIT

cat "${CHANGELOG}" > "${tmp_CHANGELOG}"

# delete all lines after "<!-- LINKS BEGIN -->"
sed -i -e '1,/<!-- LINKS BEGIN -->/!d' -- "${tmp_CHANGELOG}"
echo >> "${tmp_CHANGELOG}"

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
    | uniq \
    >> "${tmp_links}"

# match tag comparison link, e.g.
#
#    [0.0.26..main]
#    [0.0.25...0.0.26]
#
# prints
#
#    [0.0.26..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.26..main
#    [0.0.25...0.0.26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.25...0.0.26
#
grep -oEe '\[[[:alnum:]\._]+\.\.[[:alnum:]\._]+\]' -- "${CHANGELOG}" \
    | tr -d '[]' \
    | sort -n -t '.' -k1 -k2 -k3 \
    | sed -Ee 's|^(.+)$|[\1]: '"${URL_PROJECT}"'/compare/\1|g' \
    | uniq \
    >> "${tmp_links}"

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
    | sort | uniq \
    >> "${tmp_links}"

# append links
cat "${tmp_links}" >> "${tmp_CHANGELOG}"
# copy temp CHANGELOG back to original CHANGELOG
cat "${tmp_CHANGELOG}" > "${CHANGELOG}"
