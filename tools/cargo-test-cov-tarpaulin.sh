#!/usr/bin/env bash
#
# run `cargo test` with code coverage report by tarpaulin
#
# instructions from
# - https://github.com/xd009642/tarpaulin
# - https://eipi.xyz/blog/rust-code-coverage-with-github-workflows/

set -eu

cd "$(dirname -- "${0}")/.."

if !(cargo install --list | grep -q '^tarpaulin '); then
    echo "tarpaulin is not installed" >&2
    echo "    cargo install cargo-tarpaulin" >&2
    exit 1
fi

lcov="./target/debug/lcov.info"
profraw="instrument-coverage-grcov.profraw"
coverage="./target/debug/coverage.grcov"
coveralls="coveralls.json"

rm -vf -- "${lcov}" "${profraw}" "${coverage}" "${coveralls}"

if [[ "${$COVERALLS_REPO_TOKEN+x}" ]]; then
    set -x
    exec cargo tarpaulin --ciserver github-ci --coveralls $COVERALLS_REPO_TOKEN
else
    set -x
    exec cargo tarpaulin
fi
