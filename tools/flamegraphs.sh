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

export FREQ=7000

export OUT="${DIROUT}/flamegraph-evtx.svg"
(set -x
"${SCRIPT}" './logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx'
)
echo
echo

export OUT="${DIROUT}/flamegraph-journal.svg"
(set -x
"${SCRIPT}" './logs/programs/journal/Ubuntu22-user-1000x3.journal'
)
echo
echo

export OUT="${DIROUT}/flamegraph-syslog-no-matches.svg"
(set -x
"${SCRIPT}" './logs/other/tests/numbers3.log'
)
echo

export OUT="${DIROUT}/flamegraph-syslog.svg"
(set -x
"${SCRIPT}" './logs/other/tests/gen-99999-1-Motley_Crue.log'
)
echo
echo

export OUT="${DIROUT}/flamegraph-syslog-gz.svg"
(set -x
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.gz'
)
echo
echo

export OUT="${DIROUT}/flamegraph-syslog-lz4.svg"
(set -x
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.lz4'
)
echo
echo

export OUT="${DIROUT}/flamegraph-syslog-xz.svg"
(set -x
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.xz'
)
echo
echo

export OUT="${DIROUT}/flamegraph-syslog-tar.svg"
(set -x
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.tar'
)
echo
echo

export OUT="${DIROUT}/flamegraph-utmp.svg"
(set -x
"${SCRIPT}" './logs/CentOS7/x86_64/wtmp'
)
echo
echo
