#!/usr/bin/env bash
#
# helpful wrapper to run s4 with the alloc_tracker feature enabled

set -euo pipefail

cd "$(dirname -- "${0}")/.."

set -x

exec env \
    S4_ALLOC_TRACKER_PRINT=${S4_ALLOC_TRACKER_PRINT-0} \
    S4_ALLOC_TRACKER_TRACKING=${S4_ALLOC_TRACKER_TRACKING-1} \
    S4_ALLOC_TRACKER_DEPTH=${S4_ALLOC_TRACKER_DEPTH-1} \
    S4_BUILD_REGEX_PRINT=${S4_BUILD_REGEX_PRINT-} \
    RUST_MIN_STACK=${RUST_MIN_STACK-20000000} \
    cargo \
    run \
    --quiet \
    --profile alloc_tracker \
    --features alloc_tracker \
    -- "$@"
