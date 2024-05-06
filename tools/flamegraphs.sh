#!/usr/bin/env bash
#
# Run `flamegraph.sh` against the main supported `FileType`s.

set -euo pipefail

cd "$(dirname -- "${0}")/.."

SCRIPT="./tools/flamegraph.sh"

DIROUT=${DIROUT-.}

set -x

export OUT="${DIROUT}/flamegraph-help.svg"
FREQ=9999 "${SCRIPT}" '--help'

export OUT="${DIROUT}/flamegraph-evtx.svg"
"${SCRIPT}" './logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx'

export OUT="${DIROUT}/flamegraph-journal.svg"
"${SCRIPT}" './logs/programs/journal/Ubuntu22-user-1000x3.journal'

export OUT="${DIROUT}/flamegraph-syslog-no-matches.svg"
"${SCRIPT}" './logs/other/tests/numbers3.log'

export OUT="${DIROUT}/flamegraph-syslog.svg"
"${SCRIPT}" './logs/other/tests/gen-99999-1-Motley_Crue.log'

export OUT="${DIROUT}/flamegraph-syslog-gz.svg"
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.gz'

export OUT="${DIROUT}/flamegraph-syslog-lz4.svg"
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.lz4'

export OUT="${DIROUT}/flamegraph-syslog-xz.svg"
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.xz'

export OUT="${DIROUT}/flamegraph-syslog-tar.svg"
"${SCRIPT}" './logs/other/tests/gen-1000-3-foobar.log.tar'

export OUT="${DIROUT}/flamegraph-utmp.svg"
"${SCRIPT}" './logs/CentOS7/x86_64/wtmp'
