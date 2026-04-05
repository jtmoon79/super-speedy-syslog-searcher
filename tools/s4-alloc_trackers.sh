#!/usr/bin/env bash
#
# run s4-alloc_tracker.sh with a predefined set of log files

set -euo pipefail

DIROUT=${DIROUT-$PWD}

cd "$(dirname -- "${0}")/.."

for LOGFILE in \
    "./logs/CentOS7/x86_64/wtmp" \
    "./logs/MacOS11/DiagnosticMessages/2023.10.31.asl" \
    "./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx" \
    "./logs/programs/Event_Trace_Log/WindowsUpdate.20251008.140245.443.8.etl" \
    "./logs/programs/journal/RHE_91_system.journal" \
    "./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl" \
    "./logs/other/tests/dtf2-2.log" \
    "./logs/other/tests/dtf2-2.log.bz2" \
    "./logs/other/tests/dtf2-2.log.gz" \
    "./logs/other/tests/dtf2-2.log.lz4" \
    "./logs/other/tests/dtf2-2.log.xz" \
    "./logs/other/tests/dtf2-2.log.tar" \
; do
    LOGNAME=$(basename -- "${LOGFILE}")
    OUT="${DIROUT}/alloc-tracker_${LOGNAME}.txt"
    (
        set -x
        env S4_ALLOC_TRACKER_PRINT= \
            "./tools/s4-alloc_tracker.sh" \
                "${LOGFILE}" \
                    2> "${OUT}" 1> /dev/null
    ) || {
        echo "ERROR: s4-alloc_tracker.sh failed for ${LOGFILE}" >&2
        cat "${OUT}" >&2 || true
        exit 1
    }
    echo "Output written to '${OUT}'" >&2
    echo >&2
done
