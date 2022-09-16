#!/usr/bin/env bash
#
# extract significant data from a dhat file.
# call `valgrind-dhat.sh` before calling this script.
#

set -euo pipefail

echo "WORK-IN-PROGRESS: this script is an experiment" >&2

dhat=${1-}

if ! [[ -r "${dhat}" ]]; then
    echo "Unable to read dhat file '${dhat}'" >&2
    exit 1
fi

if ! which jq &>/dev/null; then
    echo "No program 'jq' found" >&2
    exit 1
fi

echo "time of program end:"
echo -n "t-end: "
cat -- "${dhat}" | jq '."te"' | tr -d '\n'
echo " instructions"
echo

echo "time of global heap maximum:"
echo -n "t-gmax: "
cat -- "${dhat}" | jq '."tg"' | tr -d '\n'
echo " instructions"
