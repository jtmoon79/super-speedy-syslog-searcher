#!/usr/bin/env bash
#
# build-version.sh
#
# Build a specific version of `super_speedy_syslog_searcher` binary.
# Useful for testing older versions, especially in regards to performance.
#
# Since `git checkout` can remove this script, this script copies itself to /tmp/
# and re-executes from there.
#

set -euo pipefail

OUTDIR=${1-}

if [[ -z "${OUTDIR}" ]]; then
    echo "Usage: ${0} <output-directory> <git-ref> [...]" >&2
    echo "Example: ${0} /tmp/s4_releases v1.2.3" >&2
    echo "Example: ${0} /tmp/s4_releases v1.2.3 v0.85 f6eb3f1810fdcbcd75cb604aa472f8c89db25a36" >&2
    exit 1
fi

shift

if [[ "${0}" != "/tmp/build-version.sh" ]]; then
    # running this script from the committed location within the PROJECT_DIR
    # copy self to /tmp/ and re-exec there to avoid issues with git checkouts
    cp -av "${0}" /tmp/build-version.sh
    export PROJECT_DIR=$(realpath "$(dirname "${0}")/../..")
    exec /tmp/build-version.sh "${OUTDIR}" "${@}"
fi
# running this script from /tmp/, the PROJECT_DIR env var should have been set

if [[ ! -d "${PROJECT_DIR:-}" ]]; then
    echo "Error: PROJECT_DIR environment variable is not set or is not a directory. Something is wrong." >&2
    exit 1
fi

if [[ ! -d "${OUTDIR}" ]]; then
    mkdir -vp "${OUTDIR}"
fi
OUTDIR=$(realpath "${OUTDIR}")
INFO=${OUTDIR}/builds.tsv

pushd "${PROJECT_DIR}"

function print_git_ref_date() {
    # print the date of the given git commit ID in YYYY-MM-DDTHH-MM-SS format
    # to be used in a file name
    git show --no-patch --format=%cI "${1}" | cut -f1-3 -d '-' | tr -s ':' '-'
}

GIT_CURRENT=$(git rev-parse HEAD)

function echo_line() {
    # print a horizontal line across the terminal
    python -Bc "print('─' * ${COLUMNS:-100})"
    echo
}

function grep_rust_version() {
    grep -m1 -Ee '^rust-version\s*=' -- Cargo.toml | cut -f2 -d '=' | tr -d ' "'
}

rust_version1=$(grep_rust_version)

function exit_() {
    echo_line
    echo
    echo "checkout back to original git commit ${GIT_CURRENT}"
    echo
    set +e
    set -x
    git restore 'src/readers/blockreader.rs'
    git checkout "${GIT_CURRENT}"
    rustup override set "${rust_version1}"
}

for GIT_REF in "${@}"; do
    if ! git show-ref "${GIT_REF}" &>/dev/null; then
        echo "ERROR ref '${GIT_REF}' is not valid" >&2
        exit 1
    fi
done

trap exit_ EXIT

for GIT_REF in "${@}"; do
    echo_line

    (set -x; git restore 'src/readers/blockreader.rs')

    (set -x; git checkout "${GIT_REF}")

    rust_version=$(grep_rust_version)
    (set -x; rustup override set "${rust_version}")
    (set -x; cargo clean)

    # patch source code to add flush to BlockReader
    # very inexplicably, older versions fail to compile due to missing `flush` method
    # ... very strange problem
    echo '
use std::io::Write;' >> 'src/readers/blockreader.rs'

    echo

    GIT_DATE=$(print_git_ref_date "${GIT_REF}")
    declare -i start=$(date +%s)

    # build release
    (set -x; cargo build --profile release)
    BIN_NAME="s4_${GIT_REF}_release"
    (set -x; cp -av "${PROJECT_DIR}/target/release/s4" "${OUTDIR}/${BIN_NAME}")
    echo -e "${BIN_NAME}\t${GIT_REF}\t${GIT_DATE}\trelease" >> "${INFO}"
    # build jemalloc
    if grep -qEe '^\[profile.jemalloc\]' -- Cargo.toml; then
        (set -x; cargo build --profile jemalloc --features jemalloc)
        BIN_NAME="s4_${GIT_REF}_jemalloc"
        (set -x; cp -av "${PROJECT_DIR}/target/jemalloc/s4" "${OUTDIR}/${BIN_NAME}")
        echo -e "${BIN_NAME}\t${GIT_REF}\t${GIT_DATE}\tjemalloc" >> "${INFO}"
    fi
    # build mimalloc
    if grep -qEe '^\[profile.mimalloc\]' -- Cargo.toml; then
        (set -x; cargo build --profile mimalloc --features mimalloc)
        BIN_NAME="s4_${GIT_REF}_mimalloc"
        (set -x; cp -av "${PROJECT_DIR}/target/mimalloc/s4" "${OUTDIR}/${BIN_NAME}")
        echo -e "${BIN_NAME}\t${GIT_REF}\t${GIT_DATE}\tmimalloc" >> "${INFO}"
    fi
    # build rpmalloc
    if grep -qEe '^\[profile.rpmalloc\]' -- Cargo.toml; then
        (set -x; cargo build --profile rpmalloc --features rpmalloc)
        BIN_NAME="s4_${GIT_REF}_rpmalloc"
        (set -x; cp -av "${PROJECT_DIR}/target/rpmalloc/s4" "${OUTDIR}/${BIN_NAME}")
        echo -e "${BIN_NAME}\t${GIT_REF}\t${GIT_DATE}\trpmalloc" >> "${INFO}"
    fi
    # build tcmalloc
    if grep -qEe '^\[profile.tcmalloc\]' -- Cargo.toml; then
        (set -x; cargo build --profile tcmalloc --features tcmalloc)
        BIN_NAME="s4_${GIT_REF}_tcmalloc"
        (set -x; cp -av "${PROJECT_DIR}/target/tcmalloc/s4" "${OUTDIR}/${BIN_NAME}")
        echo -e "${BIN_NAME}\t${GIT_REF}\t${GIT_DATE}\ttcmalloc" >> "${INFO}"
    fi

    declare -i stop=$(date +%s)
    declare -i duration=$((stop - start))
    echo
    echo "Builds completed in ${duration} seconds."
done

echo_line

{
    echo -e "BIN_NAME\tGIT_REF\tGIT_DATE\tPROFILE"
    cat "${INFO}"
} | column -t -s $'\t'
echo
