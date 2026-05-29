#!/usr/bin/env bash
#
# run s4-alloc_tracker.sh with a predefined set of log files
# or user-passed files
# save the resulting allocator tracking table to a file

set -euo pipefail

export PATH="${PATH}:${HOME}/go/bin"  # for glow
DIROUT=${DIROUT-$PWD}

PROGRAM=${PROGRAM-./target/alloc_tracker/s4}

cd "$(dirname -- "${0}")/.."

declare -a FILES=(
    ./logs/CentOS7/x86_64/wtmp
    ./logs/MacOS11/DiagnosticMessages/2023.10.31.asl
    ./logs/NetBSD9.3/Xorg.0.log
    ./logs/OpenBSD7.4/x86_64/utmp
    ./logs/other/tests/gen-99999-1-Motley_Crue.log
    ./logs/other/tests/gen-1000-3-foobar-noyear.log
    ./logs/other/tests/gen-1000-3-foobar.log
    ./logs/other/tests/gen-1000-3-foobar.log.bz2
    ./logs/other/tests/gen-1000-3-foobar.log.gz
    ./logs/other/tests/gen-1000-3-foobar.log.lz4
    ./logs/other/tests/gen-1000-3-foobar.log.tar
    ./logs/other/tests/gen-1000-3-foobar.log.xz
    ./logs/programs/Event_Trace_Log/WindowsUpdate.20251008.140245.443.8.etl
    ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx
    ./logs/programs/journal/RHE_91_system.journal
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl
    ./logs/RedHatEnterprise9/audit/audit.log
)
if [[ ${#} -gt 0 ]]; then
    FILES=("${@}")
fi

mkdir -vp "${DIROUT}"

for LOGFILE in "${FILES[@]}"; do
    LOGNAME=$(basename -- "${LOGFILE}")
    OUT="${DIROUT}/alloc-tracker_${LOGNAME}.md"
    (
        # set preliminary data in markdown file
        echo "# \`${LOGNAME}\`

## \`--version\`

\`\`\`text" > "${OUT}"
        ./target/alloc_tracker/s4 --version >> "${OUT}"
        echo "\`\`\`
" >> "${OUT}"
        set -x
        env \
            S4_ALLOC_TRACKER_DEPTH=1 \
            S4_ALLOC_TRACKER_OUTPUT="${OUT}" \
            S4_ALLOC_TRACKER_PRINT=0 \
            S4_ALLOC_TRACKER_TRACKING=1 \
            S4_BUILD_REGEX_PRINT=0 \
            RUST_MIN_STACK=${RUST_MIN_STACK-20000000} \
            "${PROGRAM}" \
            "$@" \
            "${LOGFILE}" 1>/dev/null
    ) || {
        echo "ERROR: s4-alloc_tracker.sh failed for ${LOGFILE}" >&2
        cat "${OUT}" >&2 || true
        exit 1
    }
    echo -en "\e[93m" >&2  # light yellow
    if which glow &>/dev/null; then
        glow --width=${COLUMNS} --preserve-new-lines "${OUT}" >&2
    else
        (
            cat "${OUT}"
        ) >&2
    fi
    echo -en "\e[39m" >&2  # default
    echo >&2
    echo "Output written to '${OUT}'" >&2
    echo >&2
    ./tools/mdtohtml.sh "$OUT"
done
