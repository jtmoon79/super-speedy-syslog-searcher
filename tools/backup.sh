#!/usr/bin/env bash
#
# backup.sh
#
# a quick manual backup script using 7zip
#

set -euo pipefail

cd "$(dirname "${0}")/.."

HERE="$(basename -- "$(realpath .)")"
ZIPFILE="../${HERE}-$(date '+%Y%m%dT%H%M%S')-$(hostname).zip"

Zz=$(which 7z)

declare -a logs=()
while read log; do
    logs[${#logs[@]}]=${log}
done <<< $(find ./logs -xdev -type f -size -20M | sort)

(
set -x

"${Zz}" a -spf -bb1 -bt -stl -snl -tzip -- "${ZIPFILE}" \
    ./benches/ \
    $(find ./bindgen/ \
        -mindepth 1 -maxdepth 1 \
        -not -name 'target') \
    ./Cargo.toml \
    ./Cargo.lock \
    ./CHANGELOG.md \
    ./Extended-Thoughts.md \
    ./.github/ \
    ./.gitignore \
    ./LICENSE.txt \
    "${logs[@]}" \
    $(ls -d1 \
        ./performance-data \
        ./valgrind \
        ./Notes.txt \
        ./flamegraph.svg* \
        ./tests \
        ./.vscode \
        2>/dev/null || true
    ) \
    ./README.md \
    ./rustfmt.toml \
    ./src/ \
    ./tools/ \

"${Zz}" l "${ZIPFILE}"
)

chmod -w -- "${ZIPFILE}"

echo -e "\n"

ls -lh "${ZIPFILE}"
