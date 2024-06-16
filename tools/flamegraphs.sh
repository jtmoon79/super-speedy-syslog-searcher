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
    "flamegraph-evtx.svg|./logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx" \
    "flamegraph-journal.svg|./logs/programs/journal/RHE_91_system.journal" \
    "flamegraph-journal-bz2.svg|./logs/programs/journal/RHE_91_system.journal.bz2" \
    "flamegraph-journal-gz.svg|./logs/programs/journal/RHE_91_system.journal.gz" \
    "flamegraph-journal-lz4.svg|./logs/programs/journal/RHE_91_system.journal.lz4" \
    "flamegraph-journal-xz.svg|./logs/programs/journal/RHE_91_system.journal.xz" \
    "flamegraph-journal-tar.svg|./logs/programs/journal/RHE_91_system.tar" \
    "flamegraph-syslog-empty.svg|./logs/other/tests/empty.log" \
    "flamegraph-syslog-no-matches.svg|./logs/other/tests/numbers3.log" \
    "flamegraph-syslog.svg|./logs/other/tests/gen-99999-1-Motley_Crue.log" \
    "flamegraph-syslog-bz2.svg|./logs/other/tests/gen-1000-3-foobar.log.bz2" \
    "flamegraph-syslog-gz.svg|./logs/other/tests/gen-1000-3-foobar.log.gz" \
    "flamegraph-syslog-lz4.svg|./logs/other/tests/gen-1000-3-foobar.log.lz4" \
    "flamegraph-syslog-xz.svg|./logs/other/tests/gen-1000-3-foobar.log.xz" \
    "flamegraph-syslog-tar.svg|./logs/other/tests/gen-1000-3-foobar.log.tar" \
    "flamegraph-utmp.svg|./logs/CentOS7/x86_64/wtmp" \
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
