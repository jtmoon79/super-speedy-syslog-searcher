#!/usr/bin/env bash
#
# run `cargo test` with code coverage report by grcov
#
# instructions from https://github.com/mozilla/grcov#lcov-output

set -eu

cd "$(dirname -- "${0}")/.."

if !(cargo install --list | grep -q '^grcov '); then
    echo "grcov is not installed" >&2
    echo "    cargo install grcov" >&2
    exit 1
fi

lcov="./target/debug/lcov.info"
profraw="instrument-coverage-grcov.profraw"
coverage="./target/debug/coverage.grcov"
coveralls="coveralls.json"

rm -vf -- "${lcov}" "${profraw}" "${coverage}" "${coveralls}"

set -x

cargo clean

export RUSTFLAGS="-Cinstrument-coverage"

cargo build

#export LLVM_PROFILE_FILE="${profraw}"
cargo test

# generate a coverate report from coverage artifacts
grcov . \
    -s . \
    --binary-path ./target/debug/ \
    -t lcov \
    --branch \
    --ignore-not-existing \
    -o "${coverage}" \

# generate coveralls format
grcov . \
    -s . \
    --binary-path ./target/debug/ \
    -t coveralls \
    --branch \
    --ignore-not-existing \
    `#--token YOUR_COVERALLS_TOKEN` > "${coveralls}"

# generate HTML from LCOV
genhtml \
    -o "${coverage}" \
    --show-details --highlight --ignore-errors source --legend \
    "${lcov}"
