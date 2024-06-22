#!/usr/bin/env bash
#
# valgrind-callgrind.sh
#
# Run valgrind with Call Grind.
# https://valgrind.org/docs/manual/cl-manual.html
# This script runs `valgrind --tool=callgrind`
#
# Article with specific tips for valgrind and rust
# https://nnethercote.github.io/2022/01/05/rust-and-valgrind.html
#
# User may set environment variable $PROGRAM.
# Passed arguments are passed to $PROGRAM and override default arguments.
#
# build the program with `--profile valgrind`
#

set -euo pipefail

cd "$(dirname "${0}")/.."

# use full path to Unix tools
if ! valgrind=$(which valgrind); then
    echo "valgrind not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install valgrind g++" >&2
    exit 1
fi
if ! callgrind=$(which callgrind_annotate); then
    echo "callgrind not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install valgrind" >&2
    exit 1
fi

declare -r PROGRAM=${PROGRAM-./target/valgrind/s4}

if [[ ! -x "${PROGRAM}" ]]; then
    echo "PROGRAM does not exist '${PROGRAM}'" >&2
    echo "build with:" >&2
    echo "     RUSTFLAGS=-g cargo build --profile valgrind" >&2
    exit 1
fi

(set -x; uname -a)
(set -x; git log -n1 --format='%h %D')
(set -x; "${PROGRAM}" --version)
(set -x; dot -V)
(set -x; "${valgrind}" --version) | head -n1
if "${valgrind}" --version | grep -qFe '3.18'; then
    echo "ERROR: valgrind 3.18 is known to have issues with rust
     see https://nnethercote.github.io/2022/01/05/rust-and-valgrind.html
     section 'Missing inline stack frames'

Compile the latest valgrind:
1. download from https://valgrind.org/downloads/current.html
2. untar
3. cd valgrind
4. ./configure --prefix=/usr/local
5. make
6. sudo make install
" >&2
    exit 1
fi
(set -x; "${callgrind}" --version) || true  # --version causes process return code 255
(set -x; gprof2dot --help &>/dev/null) || {
    echo "gprof2dot not found in PATH" >&2
    echo "install:" >&2
    echo "    pip install -r tools/requirements.txt" >&2
    exit 1
}

echo

declare -a args=(
    -a 20000101T000000
    -b 20000101T080000
    # ./logs/other/tests/gen-100-10-......
    # ./logs/other/tests/gen-100-10-BRAAAP.log
    # ./logs/other/tests/gen-100-10-FOOBAR.log
    # ./logs/other/tests/gen-100-10-______.log
    # ./logs/other/tests/gen-100-10-skullcrossbones.log
    # ./logs/other/tests/gen-100-4-happyface.log
    ./logs/other/tests/gen-1000-3-foobar.log
    # ./logs/other/tests/gen-200-1-jajaja.log
    # ./logs/other/tests/gen-400-4-shamrock.log
)

if [[ ${#} -ge 1 ]]; then
    # use user-passed arguments
    args=()
    for arg in "${@}"; do
        args+=("${arg}")
    done
fi

DIROUT=${DIROUT-.}
OUT=${DIROUT}/callgrind
OUTOUT="${OUT}.out"
OUTDOT="${OUT}.dot"
OUTPNG="${OUT}.png"
OUTSVG="${OUT}.svg"

rm -f -- "${OUTOUT}" "${OUTDOT}" "${OUTPNG}" "${OUTSVG}"

set -x

"${valgrind}" \
    --tool=callgrind \
    --collect-bus=yes \
    --collect-systime=yes \
    `#--separate-threads=yes` \
    --callgrind-out-file="${OUTOUT}" \
    -- \
    "${PROGRAM}" \
        "${args[@]}" \
    >/dev/null

gprof2dot \
    --format=callgrind \
    --output="${OUTDOT}" \
    "${OUTOUT}"

dot -T png "${OUTDOT}" -o "${OUTPNG}"

dot -T svg "${OUTDOT}" -o "${OUTSVG}"

"${callgrind}" \
    --tree=both \
    --show-percs=yes \
    $(find ./src -xdev -type d -exec echo -n '--include={} ' \;) \
    "${OUTOUT}"
