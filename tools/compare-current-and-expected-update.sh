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
# verify s4 can run
(set -x; "${PROGRAM}" --version)

expect_out=./tools/compare-current-and-expected_expected.stdout
expect_err=./tools/compare-current-and-expected_expected.stderr

touch "${expect_out}" "${expect_err}" || true

if ! chmod +w -- "${expect_out}"; then
    echo "ERROR unable to write to file '${expect_out}'" >&2
    exit 1
fi

# check twice for CI environments
if ! touch "${expect_out}"; then
    echo "ERROR unable to write to file '${expect_out}'" >&2
    exit 1
fi

if ! chmod +w -- "${expect_err}"; then
    echo "ERROR unable to write to file '${expect_err}'" >&2
    exit 1
fi

# check twice for CI environments
if ! touch "${expect_err}"; then
    echo "ERROR unable to write to file '${expect_err}'" >&2
    exit 1
fi

#
# print some info for the script user, verify the s4 program can run
#

logs='./tools/compare-current-and-expected_logs.txt'

if [[ ! -e "${logs}" ]]; then
    echo "ERROR file does not exist '${logs}'" >&2
    exit 1
elif [[ ! -r "${logs}" ]]; then
    echo "ERROR file is not readable '${logs}'" >&2
    exit 1
fi

cat "${logs}" >&2
echo >&2
echo "$(wc -l < "${logs}") files in \"${logs}\"" >&2
echo >&2

PROGRAM=${PROGRAM-./target/release/s4}
(set -x; "${PROGRAM}" --version)
echo >&2

# these arguments must agree with `compare-current-and-expected.sh`
declare -ar S4_ARGS=(
    --color=never
    --tz-offset=+08:00
    --prepend-filepath
    --prepend-utc
    --summary
    '-'
    "${@}"
)

(
    set -x
    "${PROGRAM}" "${S4_ARGS[@]}" < "${logs}"
) 1> "${expect_out}" 2> "${expect_err}" || true

# XXX: the following `sed` command must match `compare-current-and-expected.sh`
# - remove the printing of the current time
# - remove the printing of the datetime first and last. It might use
#   the local system timezone
# - remove warnings as they are printed in unpredictable order
sed -i -E \
    -e '/^Datetime Now[ ]*:.*$/d' \
    -e '/^[ ]*datetime first[ ]*.*$/d' \
    -e '/^[ ]*datetime last[ ]*.*$/d' \
    -e '/^Datetime printed first[ ]*:.*$/d' \
    -e '/^Datetime printed last[ ]*:.*$/d' \
    -e '/^WARNING: no syslines found .*$/d' \
    -e '/^WARNING:.*$/d' \
    -e '/.*no syslines found.*$/d' \
    -e '/^[ ]+realpath .*$/d' \
    -e '0,/^\+ \..*$/d' \
    -- "${expect_err}"

if ! chmod -wx -- "${expect_out}"; then
    echo "WARNING unable to remove wx from file '${expect_out}'" >&2
    # on Linux running on Windows NTFS mount, this is not a fatal error
fi

if ! chmod -wx -- "${expect_err}"; then
    echo "WARNING unable to remove wx from file '${expect_err}'" >&2
    # on Linux running on Windows NTFS mount, this is not a fatal error
fi

echo >&2
echo "Updated file '${expect_out}'" >&2
echo "Updated file '${expect_err}'" >&2
echo >&2
echo -e "Now run \e[1mcompare-current-and-expected.sh\e[0m." >&2
