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

# work with a copy of the CHANGELOG
cat "${CHANGELOG}" > "${tmp_CHANGELOG}"

# delete all lines after "<!-- LINKS BEGIN -->"
sed -i -e '1,/<!-- LINKS BEGIN -->/!d' -- "${tmp_CHANGELOG}"
echo >> "${tmp_CHANGELOG}"

# replaces all visible long commit hash with short commit hash
#
#   ([c3e42d74f6d4ae9cfe2701566843830cb4a6d0de]) becomes ([c3e42d7])
while read -r hash_long; do
    hash_short=$(git log -n1 '--pretty=%h' "${hash_long}") || {
        echo "Warning: failed to get short hash for ${hash_long}" >&2
        continue
    }
    sed -i -Ee "s|\(\[${hash_long}\]\)|([${hash_short}])|" -- "${tmp_CHANGELOG}"
done < <(grep -oEe '\(\[[[:xdigit:]]{40}\]\)' -- "${tmp_CHANGELOG}" | grep -oEe '[[:xdigit:]]+')

# replace dependabot pull-request PR link notation
#
#    dependabot: bump bstr from 1.8.0 to 1.9.1 (#279) ([475529e])
#
# to a markdown link notation
#
#    dependabot: bump bstr from 1.8.0 to 1.9.1 [(#279)] ([475529e])
#
sed -iEe 's| \((#[[:digit:]]{2,4})\) | ([\1]) |g' -- "${tmp_CHANGELOG}"

# match Issue link
#
#    [Issue #3]
#
# prints
#
#    [Issue #3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/3
#
grep -oEe '\[Issue #([[:digit:]]+)\]' -- "${tmp_CHANGELOG}" \
    | tr -d '[]' \
    | sort -n -t '#' -k2 \
    | sed -Ee 's|^Issue #([[:digit:]]+)$|[Issue #\1]: '"${URL_PROJECT}"'/issues/\1|g' \
    | uniq \
    >> "${tmp_links}"

# add markdown link for markdown link notation for dependabot PR
# from commit message
#
#    dependabot: bump bstr from 1.7.0 to 1.9.0 [(#237)] ([112539e])
#
# the substring
#
#    [(#237)]
#
# prints
#
#    [(#237)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/237
#
grep -oEe ' \[\(#([[:digit:]]{2,4})\)\]' -- "${tmp_CHANGELOG}" \
    | tr -d ' ' \
    | sort -n -t '#' -k2 \
    | sed -Ee 's|^\[\(#([[:digit:]]{2,4})\)\]$|[(#\1)]: '"${URL_PROJECT}"'/pull/\1|g' \
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
grep -oEe '\[[[:alnum:]\._]+\.\.[[:alnum:]\._]+\]' -- "${tmp_CHANGELOG}" \
    | tr -d '[]' \
    | sort -n -t '.' -k1 -k2 -k3 \
    | sed -Ee 's|^(.+)$|[\1]: '"${URL_PROJECT}"'/compare/\1|g' \
    | uniq \
    >> "${tmp_links}"

# link short git hash to full commit URL
#
#    - add XZ support ([607a23c])
#
# prints
#
#    [607a23c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d
#
while read -r hash_short; do
    hash_long=$(git log -n1 '--pretty=%H' "${hash_short}") || continue
    echo "[${hash_short}]: ${URL_PROJECT}/commit/${hash_long}" >> "${tmp_links}"
done < <(grep -oEe '\(\[[[:xdigit:]]{6,7}\]\)' -- "${tmp_CHANGELOG}" | grep -oEe '[[:xdigit:]]+' | sort | uniq)

# append links
cat "${tmp_links}" >> "${tmp_CHANGELOG}"
# copy temp CHANGELOG back to original CHANGELOG
cat "${tmp_CHANGELOG}" > "${CHANGELOG}"
