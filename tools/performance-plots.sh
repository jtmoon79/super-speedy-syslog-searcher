#!/usr/bin/env bash
#
# run performance-plot.sh on multiple files
# env. var FILES may refer to a file with one file per-line
# otherwise a default set of files will be used
#

set -euo pipefail

cd "$(dirname "${0}")/.."

readonly SEP='|'

if [[ "${FILES:-}" ]]; then
    if [[ ! -f "${FILES}" ]]; then
        echo "FILES is set but not a file: ${FILES}" >&2
        exit 1
    fi
    mapfile -t FILES < <(cat "${FILES}")
else
    # Default files to run performance-plot.sh on
    # optional suffix `|FILE_NUM_MAX` to limit the maximum FILE_NUM for that file
    FILES=(
        ./logs/NetBSD9.3/Xorg.0.log
        ./logs/OpenBSD7.4/x86_64/utmp
        ./logs/other/tests/dtf5-6a.log
        # TODO: [2026/05] log file gen-99999-1-Motley_Crue.log tends to stall around 125 FILE_NUM
        #       probably worth understanding why
        "./logs/other/tests/gen-99999-1-Motley_Crue.log${SEP}100"
        ./logs/other/tests/gen-1000-3-foobar-noyear.log
        ./logs/other/tests/gen-1000-3-foobar.log
        ./logs/other/tests/gen-1000-3-foobar.log.bz2
        ./logs/other/tests/gen-1000-3-foobar.log.gz
        ./logs/other/tests/gen-1000-3-foobar.log.lz4
        ./logs/other/tests/gen-1000-3-foobar.log.tar
        ./logs/other/tests/gen-1000-3-foobar.log.xz
        ./logs/other/tests/gen-1000-3-foobar.UTF-16BE.log
        ./logs/other/tests/gen-1000-3-foobar.UTF-16LE.log
        ./logs/other/tests/gen-1000-3-foobar.UTF-16LE_BOM.log
        ./logs/other/tests/gen-1000-3-foobar.UTF-32LE.log
        ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx
        ./logs/programs/journal/RHE_91_system.journal
        ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl
        ./logs/RedHatEnterprise9/audit/audit.log
    )
fi

declare -i FILE_NUM=${FILE_NUM:-300}

for FILE in "${FILES[@]}"; do
    declare -i START_TIME=${SECONDS}
    # check for optional `|FILE_NUM_MAX`
    if [[ "${FILE}" == *"${SEP}"* ]]; then
        FILE_=$(echo -n "${FILE}" | cut -d "${SEP}" -f1)
        declare -i FILE_NUM_MAX=$(echo -n "${FILE}" | cut -d "${SEP}" -f2)
        if [[ ${FILE_NUM_MAX} -le 0 ]]; then
            echo "Invalid FILE_NUM_MAX ${FILE_NUM_MAX} for string '${FILE}'" >&2
            exit 1
        fi
        if [[ "${FILE_NUM}" -gt "${FILE_NUM_MAX}" ]]; then
            FILE_NUM=${FILE_NUM_MAX}
        fi
        FILE=${FILE_}
    fi
    (
        set -x
        env FILE="${FILE}" FILE_NUM="${FILE_NUM}" ./tools/performance-plot.sh "${@}"
    )
    declare -i ELAPSED_TIME=$((SECONDS - START_TIME))
    echo
    echo "Elapsed ${ELAPSED_TIME} seconds to process ${FILE_NUM} instances of ${FILE}"
    echo
done
