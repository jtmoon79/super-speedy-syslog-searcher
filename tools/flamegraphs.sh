#!/usr/bin/env bash
#
# Run `flamegraph.sh` against the main supported `FileType`s.

set -euo pipefail

cd "$(dirname -- "${0}")/.."

SCRIPT="./tools/flamegraph.sh"

DIROUT=${DIROUT-.}

export OUT="${DIROUT}/flamegraph-help.svg"
FREQ=9999 "${SCRIPT}" '--help'
echo
echo

export FREQ=8000

for FLAMGRAPHOUT_S4FILE in \
    "flamegraph-evtx__Microsoft-Windows-Kernel-PnP__Configuration.evtx.svg|./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx" \
    "flamegraph-asl__2023.10.31.asl.svg|./logs/MacOS11/DiagnosticMessages/2023.10.31.asl" \
    "flamegraph-etl__WindowsUpdate.20251008.140245.443.8.etl.svg|./logs/programs/Event_Trace_Log/WindowsUpdate.20251008.140245.443.8.etl" \
    "flamegraph-odl__FileCoAuth-2025-12-21.1216.11020.2.odl.svg|./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl" \
    "flamegraph-journal__RHE_91_system.journal.svg|./logs/programs/journal/RHE_91_system.journal" \
    "flamegraph-journal-bz2__RHE_91_system.journal.bz2.svg|./logs/programs/journal/RHE_91_system.journal.bz2" \
    "flamegraph-journal-gz__RHE_91_system.journal.gz.svg|./logs/programs/journal/RHE_91_system.journal.gz" \
    "flamegraph-journal-lz4__RHE_91_system.journal.lz4.svg|./logs/programs/journal/RHE_91_system.journal.lz4" \
    "flamegraph-journal-xz__RHE_91_system.journal.xz.svg|./logs/programs/journal/RHE_91_system.journal.xz" \
    "flamegraph-journal-tar__RHE_91_system.tar.svg|./logs/programs/journal/RHE_91_system.tar" \
    "flamegraph-syslog-empty.svg|./logs/other/tests/empty.log" \
    "flamegraph-syslog-no-matches__numbers3.log.svg|./logs/other/tests/numbers3.log" \
    "flamegraph-syslog__gen-99999-1-Motley_Crue.log.svg|./logs/other/tests/gen-99999-1-Motley_Crue.log" \
    "flamegraph-syslog__gen-99999-1-Motley_Crue.tar.svg|./logs/other/tests/gen-99999-1-Motley_Crue.tar" \
    "flamegraph-syslog-bz2__gen-1000-3-foobar.log.bz2.svg|./logs/other/tests/gen-1000-3-foobar.log.bz2" \
    "flamegraph-syslog-gz__gen-1000-3-foobar.log.gz.svg|./logs/other/tests/gen-1000-3-foobar.log.gz" \
    "flamegraph-syslog-lz4__gen-1000-3-foobar.log.lz4.svg|./logs/other/tests/gen-1000-3-foobar.log.lz4" \
    "flamegraph-syslog-xz__gen-1000-3-foobar.log.xz.svg|./logs/other/tests/gen-1000-3-foobar.log.xz" \
    "flamegraph-syslog-tar__gen-1000-3-foobar.log.tar.svg|./logs/other/tests/gen-1000-3-foobar.log.tar" \
    "flamegraph-syslog-noyear__gen-1000-3-foobar-noyear.log.svg|./logs/other/tests/gen-1000-3-foobar-noyear.log" \
    "flamegraph-utmp__CentOS7_x86_64_wtmp.svg|./logs/CentOS7/x86_64/wtmp" \
; do
    (
        export OUT="${DIROUT}/${FLAMGRAPHOUT_S4FILE%%|*}"
        S4FILE="${FLAMGRAPHOUT_S4FILE##*|}"
        set -x
        "${SCRIPT}" "${S4FILE}"
    )
    echo
    echo
done
