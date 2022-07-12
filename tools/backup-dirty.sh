#!/usr/bin/env bash
#
# backup-dirty.sh
#
# a quick and dirty backup script using 7zip
#

set -euo pipefail

cd "$(dirname "${0}")/.."

HERE="$(basename -- "$(realpath .)")"
ZIPFILE="../${HERE}-$(date '+%Y%m%dT%H%M%S').zip"

Zz=$(which 7z)

set -x

"${Zz}" a -bb1 -bt -stl -snl -tzip "${ZIPFILE}" \
    ./Cargo.toml \
    ./Cargo.lock \
    ./CHANGELOG.md \
    ./benches \
    ./.github \
    ./.gitignore \
    ./LICENSE.txt \
    ./logs \
    ./performance-data \
    ./README.md \
    ./src \
    ./tools \

"${Zz}" l "${ZIPFILE}"

echo -e "\n\n\n"

ls -lh "${ZIPFILE}"
