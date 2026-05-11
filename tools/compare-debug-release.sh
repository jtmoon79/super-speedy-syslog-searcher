#!/usr/bin/env bash
#
# compare-debug-release.sh
#
# run the local debug build and release build, compare the outputs.
#

set -euo pipefail

cd "$(dirname "${0}")/.."

set -x

exec \
    /usr/bin/env \
        PROGRAM_A=${PROGRAM_A-./target/release/s4} \
        PROGRAM_B=${PROGRAM_B-./target/debug/s4} \
        LOGS_LISTING=${LOGS_LISTING-./tools/compare-debug-release.txt} \
            ./tools/compare-two-s4.sh "$@"
