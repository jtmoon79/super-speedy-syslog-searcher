#!/usr/bin/env bash
#
# compare-current-and-expected-update.sh
#
# helper to compare-current-and-expected.sh
# code in this script must agree with code in that script, especially the
# s4 command-line arguments and the `find` search for logs
#

set -euo pipefail

cd "$(dirname "${0}")/.."

PROGRAM=${PROGRAM-./target/release/s4}
(set -x; "${PROGRAM}" --version)

expect1=./tools/compare-current-and-expected_expected.out

if ! chmod +w -- "${expect1}"; then
    echo "ERROR unable to remove wx from file '${expect1}'" >&2
    exit 1
fi

if ! touch "${expect1}"; then
    echo "ERROR unable to write to file '${expect1}'" >&2
    exit 1
fi

(
    set -x
    (find ./logs -xdev -type f -size -3M | sort) \
    | "${PROGRAM}" \
        --color=never \
        --prepend-filename \
        '--tz-offset=+08:00' \
        '-' 2>/dev/null
) > "${expect1}" || true

if ! chmod -wx -- "${expect1}"; then
    echo "ERROR unable to remove wx from file '${expect1}'" >&2
    exit 1
fi
