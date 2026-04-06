#!/usr/bin/env bash
#
# helpful wrapper to run s4 with the alloc_tracker debug feature enabled

set -euo pipefail

cd "$(dirname -- "${0}")/.."

set -x

exec env \
    S4_ALLOC_TRACKER_PRINT=${S4_ALLOC_TRACKER_PRINT-1} \
    S4_ALLOC_TRACKER_TRACKING=${S4_ALLOC_TRACKER_TRACKING-1} \
    S4_ALLOC_DEPTH=${S4_ALLOC_DEPTH-1} \
    cargo \
    run \
    --quiet \
    --profile alloc_tracker \
    --features alloc_tracker \
    -- "$@"
