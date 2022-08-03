#!/usr/bin/env bash
#
# backup-dirty.sh
#
# a quick and dirty backup script using 7zip
#

set -euo pipefail

cd "$(dirname "${0}")/.."

HERE="$(basename -- "$(realpath .)")"
ZIPFILE="../${HERE}-$(date '+%Y%m%dT%H%M%S')-$(hostname).zip"

Zz=$(which 7z)

(
set -x

"${Zz}" a -spf -bb1 -bt -stl -snl -tzip "${ZIPFILE}" \
    ./Cargo.toml \
    ./Cargo.lock \
    ./CHANGELOG.md \
    ./benches \
    ./.github \
    ./.gitignore \
    ./LICENSE.txt \
    $(find ./logs -xdev -type f -size -2M | sort) \
    $(ls -d1 \
        ./performance-data \
        ./valgrind \
        ./Notes.txt \
        ./flamegraph.svg \
        ./tests \
        2>/dev/null || true
    ) \
    ./README.md \
    ./src \
    ./tools \

"${Zz}" l "${ZIPFILE}"
)

echo -e "\n\n\n"

ls -lh "${ZIPFILE}"
