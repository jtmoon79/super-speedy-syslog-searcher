#!/usr/bin/env bash
#
# llvm-cov-run-command.sh
#
# run `cargo llvm-cov run` with predefined arguments
#

set -euo pipefail

cd "$(dirname "${0}")/.."

# logs to process listed one per line
declare -r logs=./tools/compare-debug-release_logs.txt

# arguments taken from `compare-debug-release.sh`
declare -ar S4_ARGS=(
    --color=never
    --blocksz=0x200
    --tz-offset=+08:00
    --prepend-filename
    --prepend-file-align
    --prepend-utc
    --prepend-dt-format='%Y%m%dT%H%M%S.%9f'
    --prepend-separator='┋'
    --separator='⇳\n'
    --journal-output=export
    --dt-after='19990303T000000+0000'
    --dt-before='20230410T221032+0000'
    --summary
)

PROGRAMD=${PROGRAMD-./target/debug/s4}
(set -x; "${PROGRAMD}" --version 2>/dev/null)
readonly PROGRAMD

declare -a logs_args=()
while read log; do
    logs_args[${#logs_args[@]}]=${log}
done < "${logs}"

set -x
exec \
    cargo llvm-cov run --lcov --bin s4 "${@}" -- \
        "${S4_ARGS[@]}" \
        "${logs_args[@]}" &>/dev/null
