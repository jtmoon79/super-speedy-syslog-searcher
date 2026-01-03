#!/usr/bin/env bash
#
# run `cargo-test` on each individual test.
# this is to ensure tests also succeed in isolation
#

set -eu

cd "$(dirname -- "${0}")/.."

export RUST_BACKTRACE=1

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

function echo_line() {
    python -Bc "print('─' * ${COLUMNS:-100})"
    echo
}

filter=${1-}
declare -i test_count=0
declare -i failed_tests=0

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
    ) || {
        let failed_tests+=1 || true
        echo "Failed test '${testname}'" >&2
    }
    let test_count+=1 || true
    echo >&2
    echo_line
done <<< "$(cat "${tmpf}")"

echo >&2
echo "Total tests run: ${test_count}" >&2
echo "Total failed tests: ${failed_tests}" >&2

declare -i ret=0
if [[ ${failed_tests} -ne 0 ]]; then
    ret=1
fi

exit ${ret}
