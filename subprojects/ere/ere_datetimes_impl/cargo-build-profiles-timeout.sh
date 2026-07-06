#!/usr/bin/env bash
#
# Build the ere_datetimes_impl subproject once for each custom cargo profile
# defined in its Cargo.toml.
#
# To better understand build times with different optimization settings.
#
# Extra arguments are passed through to `cargo build`.
# Timeout behavior may be adjusted with:
#   TIMEOUT_DURATION   default: 45m
#   TIMEOUT_KILL_AFTER default: 50m
#

set -euo pipefail

cd "$(dirname -- "${BASH_SOURCE[0]}")"

readonly MANIFEST_PATH='Cargo.toml'
readonly TIMEOUT_DURATION=${TIMEOUT_DURATION:-30m}
readonly TIMEOUT_KILL_AFTER=${TIMEOUT_KILL_AFTER:-1m}
readonly C_ON='\033[0;36m' # light blue
readonly C_OFF='\033[0m'    # reset color
readonly S='|'

if [[ ! -f "${MANIFEST_PATH}" ]]; then
    echo "Manifest not found: ${MANIFEST_PATH}" >&2
    exit 1
fi

if ! command -v timeout >/dev/null 2>&1; then
    echo "Required command not found: timeout" >&2
    exit 1
fi

profiles=(
    dev
    release
    release_O0_cgu512_pa
    release_O1_cgu512_pa
    release_O2_cgu512_pa
    release_O3_cgu512_pa
    release_O1_cgu256_pa
    release_O2_cgu256_pa
    release_O3_cgu256_pa
    release_O0_cgu1
    release_O1_cgu1
    release_O2_cgu1
    release_O3_cgu1_pa
)

if [[ ${#profiles[@]} -eq 0 ]]; then
    echo "No custom profiles found in ${MANIFEST_PATH}" >&2
    exit 1
fi

declare -ar timeout_prefix=(
    timeout
    #--preserve-status
    -k "${TIMEOUT_KILL_AFTER}"
    -v
    "${TIMEOUT_DURATION}"
)
declare -a runs=()

function exit_() {
    echo
    printf '%s\n' "${runs[@]}" \
        | column -t -s "${S}" -N 'Profile,Return Code,Duration (s)'
}
trap exit_ EXIT

echo "Using manifest: ${MANIFEST_PATH}" >&2
echo "Profiles: ${profiles[*]}" >&2
echo "Timeout: ${TIMEOUT_DURATION} (kill-after ${TIMEOUT_KILL_AFTER})" >&2
echo >&2

for profile in "${profiles[@]}"; do
    declare -a cmd=(
        "${timeout_prefix[@]}"
        cargo
        build
        --profile "${profile}"
    )

    if [[ $# -ne 0 ]]; then
        cmd+=("$@")
    fi

    echo -e "=== profile: ${C_ON}${profile}${C_OFF} ===" >&2
    set +e
    (
        set -x
        killall cargo
        killall rustc
    )
    sleep 1
    declare -i TIME_START=$EPOCHSECONDS
    (
        set -x
        "${cmd[@]}"
    )
    rc=$?
    declare -i TIME_STOP=$EPOCHSECONDS
    set -e

    if [[ ${rc} -ne 0 ]]; then
        runs+=("${profile}${S}${rc}${S}$((TIME_STOP-TIME_START))")
    fi

    echo >&2
done
