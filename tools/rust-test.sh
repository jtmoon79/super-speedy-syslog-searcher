#!/usr/bin/env bash
#
# run `cargo-test` in one command with parameters I strongly prefer

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

# if `nextest` is installed and can list tests then use `nextest`
if cargo nextest list 2>/dev/null; then
    set -x
    exec cargo nextest run --locked --verbose "${args1[@]}" --test-threads=1 -- "${args2[@]}"
# else use plain `cargo test`
else
    set -x
    exec cargo test --locked -j1 "${args1[@]}" -- --test-threads=1 "${args2[@]}"
fi