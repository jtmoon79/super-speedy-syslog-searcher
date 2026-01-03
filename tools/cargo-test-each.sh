#!/usr/bin/env bash
#
# run `cargo-test` on each individual test.
# this is to ensure tests also succeed in isolation
#

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

function exit_() {
    # manually cleanup NamedTempFile
    # See https://github.com/Stebalien/tempfile/issues/183
    rm -f /tmp/tmp-s4-test-*
}

trap exit_ EXIT

tmpf=$(mktemp /tmp/tmp-s4-test-cargo-test-each-list-XXXXXX.txt)

(
    set -x
    cargo --version
    cargo test -- --test --list --format=terse | sort > "${tmpf}"
)

# output of `cargo test -- --list --format=terse` is like:
#
#     ...
#     tests::s4::test_unescape_str::r_e_some_u_1b_expects: test
#     tests::s4::test_unescape_str::r_none_expects: test
#     tests::s4::test_unescape_str::r_t_some_t_expects: test
#     tests::s4::test_unescape_str::r_v_some_u_0b_expects: test
#     tests::s4::test_unescape_str::r_x_none_expects: test
#         Doc-tests s4lib
#     src/data/datetime.rs - data::datetime::DTFSSet (line 687): test
#     src/data/datetime.rs - data::datetime::DTFSSet (line 748): test
#     ...
#

filter=${1-}

while read line; do
    echo "Processing line: '${line}'" >&2
    if ! (echo -n "${line}" | grep -E -q '^tests::|^bindings::'); then
        continue
    fi
    testname=$(echo -n "${line}" | sed -Ee 's/^(.*): test$/\1/')
    if [[ -z "${testname}" ]]; then
        continue
    fi
    if [[ ! -z "${filter}" ]] && [[ "${testname}" != *"${filter}"* ]]; then
        continue
    fi
    (
        set -x
        cargo test \
            --future-incompat-report \
            --locked \
            -j1 \
            "${testname}" \
            -- \
            --test-threads=1
    )
done <<< "$(cat "${tmpf}")"
