#!/usr/bin/env bash
#
# run performance-plot.sh on multiple files
#

set -euo pipefail

cd "$(dirname "${0}")/.."

FILES=(
    ./logs/other/tests/gen-1000-3-foobar.log
    ./logs/other/tests/gen-1000-3-foobar.log.bz2
    ./logs/other/tests/gen-1000-3-foobar.log.lz4
    ./logs/other/tests/gen-1000-3-foobar.log.tar
    ./logs/other/tests/gen-1000-3-foobar.log.xz
    ./logs/other/tests/gen-1000-3-foobar-noyear.log
    ./logs/programs/journal/RHE_91_system.journal
    "./logs/programs/evtx/Microsoft-Windows-Kernel-PnP%4Configuration.evtx"
    ./logs/OpenBSD7.4/x86_64/utmp
    ./logs/NetBSD9.3/Xorg.0.log
    ./logs/RedHatEnterprise9/audit/audit.log
)
FNUM_MAX=${FNUM_MAX:-300}

for FILE_ in "${FILES[@]}"; do
    (
        set -x
        env FILE="${FILE_}" FNUM_MAX="${FNUM_MAX}" ./tools/performance-plot.sh "${@}"
    )
done
