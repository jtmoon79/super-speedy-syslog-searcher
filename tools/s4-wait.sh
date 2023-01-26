#!/usr/bin/env bash
#
# quickie script to pause s4 soon after it starts.
# Useful for manual monitoring steps like attaching a debug or
# starting other monitoring tools.
#
# User can overrode $PROGRAM to select the binary to run.
# User can overrode $NICE to set the $PROGRAM nice level.
#

set -euo pipefail

cd "$(dirname -- "${0}")/.."

declare -r S4R=./target/release/s4
PROGRAM=${PROGRAM-${S4R}}

NICE_CMD=
if [[ "${NICE+x}" ]]; then
    NICE_CMD="nice -n ${NICE} "
fi
echo "${PS4}${NICE_CMD}${PROGRAM}" "${@}" '&' >&2
${NICE_CMD} "${PROGRAM}" "${@}" &
declare -r PID=$!
kill -SIGSTOP "${PID}"

echo "The process ID is ${PID}." >&2
read -p "Press enter to SIGCONT the process."
kill -SIGCONT "${PID}"
wait "${PID}"
