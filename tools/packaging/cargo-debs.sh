#!/usr/bin/env bash
#
# run `cargo deb` with preferred options
#
# install:
#   cargo install cargo-deb --locked
#

set -eu

cd "$(dirname -- "${0}")/../.."

if [[ -z "${DIROUT-}" ]]; then
    echo "ERROR DIROUT is not set" >&2
    exit 1
fi

declare -a build_passed=()
declare -a build_failed=()

function exit_() {
    echo
    echo "Build passed for targets:"
    for target in "${build_passed[@]}"; do
        echo "${target}"
    done
    echo

    echo "Build failed for targets:"
    for target in "${build_failed[@]}"; do
        echo "${target}"
    done
}
trap exit_ EXIT

for target in \
    aarch64-unknown-linux-gnu \
    aarch64-unknown-linux-gnueabihf \
    aarch64-unknown-linux-musl \
    aarch64_be-unknown-linux-gnu \
    armv7-unknown-linux-gnueabihf \
    armv7-unknown-linux-musl \
    i686-unknown-linux-gnu \
    i686-unknown-linux-gnueabihf \
    i686-unknown-linux-musl \
    mips64el-unknown-linux-muslabi64 \
    powerpc-unknown-linux-musl \
    riscv64gc-unknown-linux-musl \
    riscv64a23-unknown-linux-gnu \
    x86_64-unknown-linux-gnu \
    x86_64-unknown-linux-gnueabihf \
    x86_64-unknown-linux-musl \
    s390x-unknown-linux-gnu \

do
    (
        set -x
        rustup target add "${target}"
        ./tools/packaging/cargo-deb.sh --target "${target}" --output  "${DIROUT}/s4_${target}.deb" "${@}"
    ) || {
        echo "Failed for target ${target}" >&2
        echo >&2
        build_failed+=("${target}")
        continue
    }
    build_passed+=("${target}")
    echo >&2
done
