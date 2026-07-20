#!/usr/bin/env bash
#
# Run `stack-size.sh` against a variety of files.

set -euo pipefail

cd "$(dirname -- "${0}")/.."

SCRIPT="./tools/stack-size.sh"

export DIROUT=${DIROUT-.}
mkdir -p "${DIROUT}"
export OUT_MD=${DIROUT}/stack-sizes.md

"${SCRIPT}" \
    ./logs/CentOS7/x86_64/wtmp \
    ./logs/MacOS11/DiagnosticMessages/2023.10.31.asl \
    ./logs/MacOS11/powermanagement/2023.10.26.asl \
    ./logs/MacOS11/powermanagement/2023.10.26.asl.bz2 \
    ./logs/MacOS11/powermanagement/2023.10.26.asl.gz \
    ./logs/MacOS11/powermanagement/2023.10.26.asl.lz4 \
    ./logs/MacOS11/powermanagement/2023.10.26.asl.xz \
    ./logs/MacOS11/powermanagement/2023.10.26.tar \
    ./logs/other/tests/dtf5-6b.UTF-16BE_BOM.log \
    ./logs/other/tests/dtf5-6b.UTF-16BE.log \
    ./logs/other/tests/dtf5-6b.UTF-16LE_BOM.log \
    ./logs/other/tests/dtf5-6b.UTF-16LE.log \
    ./logs/other/tests/dtf5-6b.UTF-32BE_BOM.log \
    ./logs/other/tests/dtf5-6b.UTF-32BE.log \
    ./logs/other/tests/dtf5-6b.UTF-32LE_BOM.log \
    ./logs/other/tests/dtf5-6b.UTF-32LE.log \
    ./logs/other/tests/dtf5-6b.UTF-8_BOM.log \
    ./logs/other/tests/dtf5-6b.UTF-8.log \
    ./logs/other/tests/gen-1000-3-foobar-noyear.log \
    ./logs/other/tests/gen-1000-3-foobar.log.bz2 \
    ./logs/other/tests/gen-1000-3-foobar.log.gz \
    ./logs/other/tests/gen-1000-3-foobar.log.lz4 \
    ./logs/other/tests/gen-1000-3-foobar.log.tar \
    ./logs/other/tests/gen-1000-3-foobar.log.xz \
    ./logs/other/tests/gen-99999-1-Motley_Crue.log \
    ./logs/other/tests/gen-99999-1-Motley_Crue.tar \
    ./logs/other/tests/numbers3.log \
    ./logs/programs/Event_Trace_Log/WindowsUpdate.20251008.140245.443.8.etl \
    ./logs/programs/evtx/Microsoft-Windows-Kernel-PnP__Configuration.evtx \
    ./logs/programs/journal/RHE_91_system.journal \
    ./logs/programs/journal/RHE_91_system.journal.bz2 \
    ./logs/programs/journal/RHE_91_system.journal.gz \
    ./logs/programs/journal/RHE_91_system.journal.lz4 \
    ./logs/programs/journal/RHE_91_system.journal.xz \
    ./logs/programs/journal/RHE_91_system.tar \
    ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl \
    ./logs/Windows10Pro/comsetup.log \
