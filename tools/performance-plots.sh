#!/usr/bin/env bash
#
# run performance-plot.sh on multiple files
#

set -euo pipefail

cd "$(dirname "${0}")/.."

if [[ "${FILES:-}" ]]; then
    if [[ ! -f "${FILES}" ]]; then
        echo "FILES is set but not a file: ${FILES}" >&2
        exit 1
    fi
    mapfile -t FILES < <(cat "${FILES}")
else
    FILES=(
        ./logs/NetBSD9.3/Xorg.0.log
        ./logs/OpenBSD7.4/x86_64/utmp
        ./logs/other/tests/dtf5-6a.log
        ./logs/other/tests/gen-99999-1-Motley_Crue.log
        ./logs/other/tests/gen-1000-3-foobar-noyear.log
        ./logs/other/tests/gen-1000-3-foobar.log
        ./logs/other/tests/gen-1000-3-foobar.log.bz2
        ./logs/other/tests/gen-1000-3-foobar.log.gz
        ./logs/other/tests/gen-1000-3-foobar.log.lz4
        ./logs/other/tests/gen-1000-3-foobar.log.tar
        ./logs/other/tests/gen-1000-3-foobar.log.xz
        ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx
        ./logs/programs/journal/RHE_91_system.journal
        ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl
        ./logs/RedHatEnterprise9/audit/audit.log
    )
fi

FILE_NUM=${FILE_NUM:-300}

for FILE_ in "${FILES[@]}"; do
    (
        set -x
        env FILE="${FILE_}" FILE_NUM="${FILE_NUM}" ./tools/performance-plot.sh "${@}"
    )
done
