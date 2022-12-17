#!/usr/bin/env bash
#
# run `cargo-test` or `cargo nextest` with preferred options
#

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

# allow user to pass -- to place extra arguments past the prescripted -- delimiter
# most often this is `--nocapture`
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

function exit_() {
    # manually cleanup NamedTempFile
    # See https://github.com/Stebalien/tempfile/issues/183
    rm -f /tmp/tmp-s4-test-*
}

trap exit_ EXIT

# if `nextest` is installed and can list tests then use `nextest`
if cargo nextest list 2>/dev/null; then
    (
        set -x
        cargo nextest --version
        cargo nextest run \
            --locked \
            --verbose \
            --test-threads=1 \
            "${args1[@]}" \
            -- \
            "${args2[@]}"
    )
# else use plain `cargo test`
else
    (
        set -x
        cargo --version
        cargo test \
            --verbose \
            --future-incompat-report \
            --locked \
            -j1 \
            --all-features \
            "${args1[@]}" \
            -- \
            --test-threads=1 \
            "${args2[@]}"
    )
fi
