#!/usr/bin/env bash
#
# run `rust-gdb` with startup options I prefer
#

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

# allow user to pass -- to place extra arguments past the prescripted -- delimiter
declare -a args1=()
for a in "${@}"; do
    if [[ "${a}" == "--" ]]; then
        shift
        break
    fi
    args1[${#args1[@]}]=${a}
    shift
done
declare -a args2=()
for a in "${@}"; do
    args2[${#args2[@]}]=${a}
done

target=$(ls -1tr target/debug/deps/s4-* | tail -n1)

set -x

rust-gdb --version

exec rust-gdb "${target}" -ex 'layout split' "${@}"
