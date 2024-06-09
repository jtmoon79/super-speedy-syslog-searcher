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
    "${DIROUT}/flamegraph-evtx.svg|./logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx" \
    "${DIROUT}/flamegraph-journal.svg|./logs/programs/journal/RHE_91_system.journal" \
    "${DIROUT}/flamegraph-journal-bz2.svg|./logs/programs/journal/RHE_91_system.journal.bz2" \
    "${DIROUT}/flamegraph-journal-gz.svg|./logs/programs/journal/RHE_91_system.journal.gz" \
    "${DIROUT}/flamegraph-journal-lz4.svg|./logs/programs/journal/RHE_91_system.journal.lz4" \
    "${DIROUT}/flamegraph-journal-xz.svg|./logs/programs/journal/RHE_91_system.journal.xz" \
    "${DIROUT}/flamegraph-journal-tar.svg|./logs/programs/journal/RHE_91_system.tar" \
    "${DIROUT}/flamegraph-syslog-empty.svg|./logs/other/tests/empty.log" \
    "${DIROUT}/flamegraph-syslog-no-matches.svg|./logs/other/tests/numbers3.log" \
    "${DIROUT}/flamegraph-syslog.svg|./logs/other/tests/gen-99999-1-Motley_Crue.log" \
    "${DIROUT}/flamegraph-syslog-bz2.svg|./logs/other/tests/gen-1000-3-foobar.log.bz2" \
    "${DIROUT}/flamegraph-syslog-gz.svg|./logs/other/tests/gen-1000-3-foobar.log.gz" \
    "${DIROUT}/flamegraph-syslog-lz4.svg|./logs/other/tests/gen-1000-3-foobar.log.lz4" \
    "${DIROUT}/flamegraph-syslog-xz.svg|./logs/other/tests/gen-1000-3-foobar.log.xz" \
    "${DIROUT}/flamegraph-syslog-tar.svg|./logs/other/tests/gen-1000-3-foobar.log.tar" \
    "${DIROUT}/flamegraph-utmp.svg|./logs/CentOS7/x86_64/wtmp" \
; do
    export OUT=${FLAMGRAPHOUT_S4FILE%%|*}
    S4FILE=${FLAMGRAPHOUT_S4FILE##*|}
    (
        set -x
        "${SCRIPT}" "${S4FILE}"
    )
    echo
    echo
done
