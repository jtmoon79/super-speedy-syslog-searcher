#!/usr/bin/env bash
#
# hardcoded performance comparison of old versions of s4.
# Run this after running `build-versions.sh` to build old versions.
#

set -eu

INFO=${1-}

if [[ ! -f "${INFO}" ]]; then
    echo "Usage: ${0} <build-info>" >&2
    echo "Example: ${0} /tmp/s4_releases/builds.tsv" >&2
    exit 1
fi

BIN_DIR=$(realpath "$(dirname "${INFO}")")

cd "$(dirname "${0}")/../.."

# use full path to Unix tools
time=$(which time)
(set -x; $time --version) | head -n1

# check for hyperfine
hyperfine=$(which hyperfine) || {
    echo "ERROR: hyperfine not found in PATH" >&2
    echo "install:" >&2
    echo "    cargo install --locked hyperfine" >&2
    exit 1
}
(set -x; hyperfine --version)

# check for jq
if ! which jq &>/dev/null; then
    echo "ERROR: jq not found in PATH" >&2
    echo "install:" >&2
    echo "    sudo apt install jq" >&2
    exit 1
fi
JQ=$(which jq)

if ! which python3 &>/dev/null; then
    echo "ERROR: python3 not found in PATH" >&2
    exit 1
fi

readonly HRUNS=5

declare -ar FILES_ALL=(
    './tools/compare-log-mergers/gen-5000-1-facesA.log'
    './tools/compare-log-mergers/gen-5000-1-facesB.log'
    './tools/compare-log-mergers/gen-5000-1-facesC.log'
    './tools/compare-log-mergers/gen-5000-1-facesD.log'
    './tools/compare-log-mergers/gen-5000-1-facesE.log'
    './tools/compare-log-mergers/gen-5000-1-facesF.log'
    './tools/compare-log-mergers/gen-5000-1-facesG.log'
    './tools/compare-log-mergers/gen-5000-1-facesH.log'
    './tools/compare-log-mergers/gen-5000-1-facesI.log'
    './tools/compare-log-mergers/gen-5000-1-facesJ.log'
)

# the upcoming `git checkout` may remove some of the above log files
# so copy them to the temporary directory
TDIR_LOGS=/tmp/s4-compare-versions-logs
mkdir -vp "${TDIR_LOGS}"
declare -a files=()
for file in "${FILES_ALL[@]}"; do
    filenew="${TDIR_LOGS}/$(basename "${file}")"
    cp -av "${file}" "${filenew}"
    files+=( "${filenew}" )
done
readonly files

if [[ ${#files[@]} -lt 1 ]]; then
    echo "ERROR cannot find any log files" >&2
    exit 1
fi

function echo_line() {
    python -Bc "print('─' * ${COLUMNS:-100})"
    echo
}

function file_size() {
    stat --printf='%s' "${1}"
}

function file_isempty() {
    if [[ ! -f "${1}" ]]; then
        return 1
    fi
    [[ $(file_size "${1}") -eq 0 ]]
}

# datetime range for s4
declare -r after_dt="2000-01-01T00:20:00"
declare -r befor_dt="2000-01-01T00:50:00"

tmpD=$(mktemp -d -t "compare-versions_XXXXX")

while read s4_info; do
    echo_line
    # this must match the builds.tsv format:
    #   BIN_NAME<TAB>GIT_REF<TAB>GIT_DATE<TAB>BUILD_PROFILE
    BIN_NAME=$(echo "${s4_info}" | cut -f1 -d $'\t')
    GIT_REF=$(echo "${s4_info}" | cut -f2 -d $'\t')
    GIT_DATE=$(echo "${s4_info}" | cut -f3 -d $'\t')
    BUILD_PROFILE=$(echo "${s4_info}" | cut -f4 -d $'\t')

    S4="${BIN_DIR}/${BIN_NAME}"

    echo "Testing '${S4}'"
    echo

    (set -x; "${S4}" --version)
    echo

    json1="${tmpD}/${BIN_NAME}.json"
    tmp1="${tmpD}/${BIN_NAME}.time"

    (
        set -x
        ${hyperfine} \
            --warmup=2 \
            --style=color \
            --time-unit=millisecond \
            --runs=${HRUNS} \
            --export-json "${json1}" \
            -N \
            --command-name "${BIN_NAME}" \
            -- \
                "'${S4}' -a='${after_dt}' -b='${befor_dt}' --color=never ${files[*]}"
    )
    echo

    # %M = Maximum resident set size in KB
    # %P = CPU percentage
    # %E = Elapsed real time
    # see https://www.man7.org/linux/man-pages/man1/time.1.html
    # Note: metrics %t %K and other memory metrics always returned 0
    TIME_FORMAT="%M|%P|%E|${GIT_REF}|${GIT_DATE}|${BUILD_PROFILE}"

    (
        set -x
        ${time} \
            --format="${TIME_FORMAT}" \
            --output="${tmp1}" \
            -- \
                "${S4}" \
                "-a=${after_dt}" \
                "-b=${befor_dt}" \
                "--color=never" \
                "${files[@]}" > /dev/null
    )
    echo
done <<< "$(cat "${INFO}")"

echo_line

function to_milliseconds() {
    # from seconds to milliseconds; '0.0034125904' -> '3.0'
    # $1 must be a number
    if [[ 1 -ne ${#} ]]; then
        echo "ERROR: wrong number of arguments" >&2
        exit 1
    fi
    if [[ -z "${1}" ]]; then
        echo "ERROR: empty value" >&2
        exit 1
    fi
    python3 -c "print('%.1f' % (${1} * 1000))"
}

mddraft="${tmpD}/compare-versions-draft.md"

# markdown table header
echo "\
|Command|Git Ref|Git Date|Profile|Mean (ms)|Min (ms)|Max (ms)|Max RSS (KB)|Max RSS (KB) (GNU time)|CPU % (GNU time)|
|:---   |:---   |:---    |:---   |---:     |---:    |---:    |---:        |---:                   |---:            |" > "${mddraft}"
# markdown table rows
for json in $(find "${tmpD}" -name '*.json' | sort); do
    if file_isempty "${json}"; then
        echo "skip JSON file ${json}" >&2
        continue
    fi
    tm=$(echo "${json}" | sed 's/\.json$/.time/')
    if file_isempty "${tm}"; then
        echo "skip time file ${tm}" >&2
        continue
    fi

    (
        # example hyperfine JSON output:
        #
        # {
        #   "results": [
        #     {
        #       "command": "s4_0.7.77",
        #       "mean": 0.34721515733333336,
        #       "stddev": 0.0022061126997868254,
        #       "median": 0.34669891150000004,
        #       "user": 0.29701803333333326,
        #       "system": 0.44120176666666666,
        #       "min": 0.34394707700000005,
        #       "max": 0.35471939,
        #       "times": [
        #         0.35471939,
        #         ...,
        #         0.346103957
        #       ],
        #       "memory_usage_byte": [
        #         138768384,
        #         ...,
        #         138899456
        #       ],
        #       "exit_codes": [
        #         0,
        #         ...,
        #         0
        #       ]
        #     }
        #   ]
        # }

        command=$($JQ '.results[0].command' < "${json}" | tr -d '"')
        mean=$(to_milliseconds $($JQ '.results[0].mean' < "${json}"))
        stddev=$(to_milliseconds $($JQ '.results[0].stddev' < "${json}"))
        min=$(to_milliseconds $($JQ '.results[0].min' < "${json}"))
        max=$(to_milliseconds $($JQ '.results[0].max' < "${json}"))
        mss=$($JQ '.results[0].memory_usage_byte | max / 1024' < "${json}")
        # follows from TIME_FORMAT
        mss_time=$(cat "${tm}" | cut -d'|' -f1)
        cpup=$(cat "${tm}" | cut -d'|' -f2)
        git_ref=$(cat "${tm}" | cut -d'|' -f4)
        git_date=$(cat "${tm}" | cut -d'|' -f5)
        build_profile=$(cat "${tm}" | cut -d'|' -f6)
        echo "|\`${command}\`|${git_ref}|${git_date}|${build_profile}|${mean} ± ${stddev}|${min}|${max}|${mss}|${mss_time}|${cpup}|"
    ) >> "${mddraft}"
done

mdfinal="${tmpD}/compare-versions.md"

cat "${mddraft}" | column -t -s '|' -o '|' > "${mdfinal}"

(set -x; cat "${mdfinal}")

if which glow &>/dev/null; then
    glow --width=${COLUMNS} --preserve-new-lines "${mdfinal}"
else
    echo "install 'glow' for pretty markdown viewing" >&2
    echo "    go install github.com/charmbracelet/glow/v2@latest" >&2
fi
