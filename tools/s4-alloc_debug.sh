#!/usr/bin/env bash
#
# helpful wrapper to run s4 with the sysalloc debug feature enabled

set -euo pipefail

cd "$(dirname -- "${0}")/.."

set -x

exec env \
    S4_SYSALLOC_DEBUG_PRINT=${S4_SYSALLOC_DEBUG_PRINT-1} \
    S4_SYSALLOC_DEBUG_TRACKING=${S4_SYSALLOC_DEBUG_TRACKING-1} \
    cargo \
    run \
    --profile sysalloc_debug_release \
    --features sysalloc_debug \
    -- "$@"
