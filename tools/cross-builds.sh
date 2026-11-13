#!/usr/bin/env bash
#
# attempt to build all targets with various build commands.
# some build commands may succeed where others fail,
# e.g. `cargo cross build ...` fails but `cross build ...` succeeds.
#
# requires:
#     cargo install --locked cross cargo-cross
#
# Tier platforms:
#    https://doc.rust-lang.org/1.88.0/rustc/platform-support.html
#
# Ask AI to help with platform updates by using prompt:
#    .github/prompts/sync-rust-platform-tiers.prompt.md
#
# args $@ are passed to the build commands, e.g. `--release` will be passed
# along to build commands.
#
# env. vars:
#   DIROUT when set, the s4 binary built for each target will be copied to
#          DIROUT with meaningful names
#   FILTER when set, only targets matching the regex pattern in FILTER will be built
#
# Concurrent runs must select different targets. Each target has a separate
# build cache and a separate probe cache under target/cross-builds/TARGET. Selecting
# parallel non-conflicting targets is the responsibility of the user.
# cargo's built-in lockfile mechanism should protect against corruption.
#
# Option --print-targets will print all supported targets and exit.
#
# Must support running on Ubuntu and Darwin OS

set -euo pipefail

readonly SEP="|"

declare -a TIER_TARGETS=(
    # Tier 1 platforms
    "1${SEP}aarch64-apple-darwin"
    "1${SEP}aarch64-unknown-linux-gnu"
    "1${SEP}i686-pc-windows-msvc"
    "1${SEP}i686-unknown-linux-gnu"
    "1${SEP}x86_64-apple-darwin"
    "1${SEP}x86_64-pc-windows-gnu"
    "1${SEP}x86_64-pc-windows-msvc"
    "1${SEP}x86_64-unknown-linux-gnu"
    # Tier 2 platforms
    "2${SEP}aarch64-pc-windows-msvc"
    "2${SEP}aarch64-unknown-linux-musl"
    "2${SEP}aarch64-unknown-linux-ohos"
    "2${SEP}arm-unknown-linux-gnueabi"
    "2${SEP}arm-unknown-linux-gnueabihf"
    "2${SEP}armv7-unknown-linux-gnueabihf"
    "2${SEP}armv7-unknown-linux-ohos"
    "2${SEP}loongarch64-unknown-linux-gnu"
    "2${SEP}loongarch64-unknown-linux-musl"
    "2${SEP}i686-pc-windows-gnu"
    "2${SEP}powerpc-unknown-linux-gnu"
    "2${SEP}powerpc64-unknown-linux-gnu"
    "2${SEP}powerpc64le-unknown-linux-gnu"
    "2${SEP}powerpc64le-unknown-linux-musl"
    "2${SEP}riscv64gc-unknown-linux-gnu"
    "2${SEP}riscv64gc-unknown-linux-musl"
    "2${SEP}s390x-unknown-linux-gnu"
    "2${SEP}x86_64-unknown-freebsd"
    "2${SEP}x86_64-unknown-illumos"
    "2${SEP}x86_64-unknown-linux-musl"
    "2${SEP}x86_64-unknown-linux-ohos"
    "2${SEP}x86_64-unknown-netbsd"
    "2${SEP}aarch64-apple-ios"
    "2${SEP}aarch64-apple-ios-macabi"
    "2${SEP}aarch64-apple-ios-sim"
    "2${SEP}aarch64-linux-android"
    "2${SEP}aarch64-pc-windows-gnullvm"
    "2${SEP}aarch64-unknown-fuchsia"
    "2${SEP}aarch64-unknown-none"
    "2${SEP}aarch64-unknown-none-softfloat"
    "2${SEP}aarch64-unknown-uefi"
    "2${SEP}arm-linux-androideabi"
    "2${SEP}arm-unknown-linux-musleabi"
    "2${SEP}arm-unknown-linux-musleabihf"
    "2${SEP}arm64ec-pc-windows-msvc"
    "2${SEP}armebv7r-none-eabi"
    "2${SEP}armebv7r-none-eabihf"
    "2${SEP}armv5te-unknown-linux-gnueabi"
    "2${SEP}armv5te-unknown-linux-musleabi"
    "2${SEP}armv7-linux-androideabi"
    "2${SEP}armv7-unknown-linux-gnueabi"
    "2${SEP}armv7-unknown-linux-musleabi"
    "2${SEP}armv7-unknown-linux-musleabihf"
    "2${SEP}armv7a-none-eabi"
    "2${SEP}armv7r-none-eabi"
    "2${SEP}armv7r-none-eabihf"
    "2${SEP}i586-unknown-linux-gnu"
    "2${SEP}i586-unknown-linux-musl"
    "2${SEP}i686-linux-android"
    "2${SEP}i686-pc-windows-gnullvm"
    "2${SEP}i686-unknown-freebsd"
    "2${SEP}i686-unknown-linux-musl"
    "2${SEP}i686-unknown-uefi"
    "2${SEP}loongarch64-unknown-none"
    "2${SEP}loongarch64-unknown-none-softfloat"
    "2${SEP}nvptx64-nvidia-cuda"
    "2${SEP}riscv32i-unknown-none-elf"
    "2${SEP}riscv32im-unknown-none-elf"
    "2${SEP}riscv32imac-unknown-none-elf"
    "2${SEP}riscv32imafc-unknown-none-elf"
    "2${SEP}riscv32imc-unknown-none-elf"
    "2${SEP}riscv64gc-unknown-none-elf"
    "2${SEP}riscv64imac-unknown-none-elf"
    "2${SEP}sparc64-unknown-linux-gnu"
    "2${SEP}sparcv9-sun-solaris"
    "2${SEP}thumbv6m-none-eabi"
    "2${SEP}thumbv7em-none-eabi"
    "2${SEP}thumbv7em-none-eabihf"
    "2${SEP}thumbv7m-none-eabi"
    "2${SEP}thumbv7neon-linux-androideabi"
    "2${SEP}thumbv7neon-unknown-linux-gnueabihf"
    "2${SEP}thumbv8m.base-none-eabi"
    "2${SEP}thumbv8m.main-none-eabi"
    "2${SEP}thumbv8m.main-none-eabihf"
    "2${SEP}wasm32-unknown-emscripten"
    "2${SEP}wasm32-unknown-unknown"
    "2${SEP}wasm32-wasip1"
    "2${SEP}wasm32-wasip1-threads"
    "2${SEP}wasm32-wasip2"
    "2${SEP}wasm32v1-none"
    "2${SEP}x86_64-apple-ios"
    "2${SEP}x86_64-apple-ios-macabi"
    "2${SEP}x86_64-fortanix-unknown-sgx"
    "2${SEP}x86_64-linux-android"
    "2${SEP}x86_64-pc-solaris"
    "2${SEP}x86_64-pc-windows-gnullvm"
    "2${SEP}x86_64-unknown-fuchsia"
    "2${SEP}x86_64-unknown-linux-gnux32"
    "2${SEP}x86_64-unknown-none"
    "2${SEP}x86_64-unknown-redox"
    "2${SEP}x86_64-unknown-uefi"
    # Tier 3 platforms
    "3${SEP}aarch64-apple-tvos"
    "3${SEP}aarch64-apple-tvos-sim"
    "3${SEP}aarch64-apple-visionos"
    "3${SEP}aarch64-apple-visionos-sim"
    "3${SEP}aarch64-apple-watchos"
    "3${SEP}aarch64-apple-watchos-sim"
    "3${SEP}aarch64-kmc-solid_asp3"
    "3${SEP}aarch64-nintendo-switch-freestanding"
    "3${SEP}aarch64-unknown-freebsd"
    "3${SEP}aarch64-unknown-hermit"
    "3${SEP}aarch64-unknown-illumos"
    "3${SEP}aarch64-unknown-linux-gnu_ilp32"
    "3${SEP}aarch64-unknown-netbsd"
    "3${SEP}aarch64-unknown-nto-qnx700"
    "3${SEP}aarch64-unknown-nto-qnx710"
    "3${SEP}aarch64-unknown-nto-qnx710_iosock"
    "3${SEP}aarch64-unknown-nto-qnx800"
    "3${SEP}aarch64-unknown-nuttx"
    "3${SEP}aarch64-unknown-openbsd"
    "3${SEP}aarch64-unknown-redox"
    "3${SEP}aarch64-unknown-teeos"
    "3${SEP}aarch64-unknown-trusty"
    "3${SEP}aarch64-uwp-windows-msvc"
    "3${SEP}aarch64-wrs-vxworks"
    "3${SEP}aarch64_be-unknown-linux-gnu"
    "3${SEP}aarch64_be-unknown-linux-gnu_ilp32"
    "3${SEP}aarch64_be-unknown-netbsd"
    "3${SEP}amdgcn-amd-amdhsa"
    "3${SEP}arm64_32-apple-watchos"
    "3${SEP}arm64e-apple-darwin"
    "3${SEP}arm64e-apple-ios"
    "3${SEP}arm64e-apple-tvos"
    "3${SEP}armeb-unknown-linux-gnueabi"
    "3${SEP}armv4t-none-eabi"
    "3${SEP}armv4t-unknown-linux-gnueabi"
    "3${SEP}armv5te-none-eabi"
    "3${SEP}armv5te-unknown-linux-uclibceabi"
    "3${SEP}armv6-unknown-freebsd"
    "3${SEP}armv6-unknown-netbsd-eabihf"
    "3${SEP}armv6k-nintendo-3ds"
    "3${SEP}armv7-rtems-eabihf"
    "3${SEP}armv7-sony-vita-newlibeabihf"
    "3${SEP}armv7-unknown-freebsd"
    "3${SEP}armv7-unknown-linux-uclibceabi"
    "3${SEP}armv7-unknown-linux-uclibceabihf"
    "3${SEP}armv7-unknown-netbsd-eabihf"
    "3${SEP}armv7-unknown-trusty"
    "3${SEP}armv7-wrs-vxworks-eabihf"
    "3${SEP}armv7a-kmc-solid_asp3-eabi"
    "3${SEP}armv7a-kmc-solid_asp3-eabihf"
    "3${SEP}armv7a-none-eabihf"
    "3${SEP}armv7k-apple-watchos"
    "3${SEP}armv7s-apple-ios"
    "3${SEP}armv8r-none-eabihf"
    "3${SEP}armv7a-nuttx-eabi"
    "3${SEP}armv7a-nuttx-eabihf"
    "3${SEP}avr-none"
    "3${SEP}bpfeb-unknown-none"
    "3${SEP}bpfel-unknown-none"
    "3${SEP}csky-unknown-linux-gnuabiv2"
    "3${SEP}csky-unknown-linux-gnuabiv2hf"
    "3${SEP}hexagon-unknown-linux-musl"
    "3${SEP}hexagon-unknown-none-elf"
    "3${SEP}i386-apple-ios"
    "3${SEP}i586-unknown-netbsd"
    "3${SEP}i586-unknown-redox"
    "3${SEP}i686-apple-darwin"
    "3${SEP}i686-pc-nto-qnx700"
    "3${SEP}i686-unknown-haiku"
    "3${SEP}i686-unknown-hurd-gnu"
    "3${SEP}i686-unknown-netbsd"
    "3${SEP}i686-unknown-openbsd"
    "3${SEP}i686-uwp-windows-gnu"
    "3${SEP}i686-uwp-windows-msvc"
    "3${SEP}i686-win7-windows-gnu"
    "3${SEP}i686-win7-windows-msvc"
    "3${SEP}i686-wrs-vxworks"
    "3${SEP}loongarch64-unknown-linux-ohos"
    "3${SEP}m68k-unknown-linux-gnu"
    "3${SEP}m68k-unknown-none-elf"
    "3${SEP}mips-unknown-linux-gnu"
    "3${SEP}mips-unknown-linux-musl"
    "3${SEP}mips-unknown-linux-uclibc"
    "3${SEP}mips64-openwrt-linux-musl"
    "3${SEP}mips64-unknown-linux-gnuabi64"
    "3${SEP}mips64-unknown-linux-muslabi64"
    "3${SEP}mips64el-unknown-linux-gnuabi64"
    "3${SEP}mips64el-unknown-linux-muslabi64"
    "3${SEP}mipsel-sony-psp"
    "3${SEP}mipsel-sony-psx"
    "3${SEP}mipsel-unknown-linux-gnu"
    "3${SEP}mipsel-unknown-linux-musl"
    "3${SEP}mipsel-unknown-linux-uclibc"
    "3${SEP}mipsel-unknown-netbsd"
    "3${SEP}mipsel-unknown-none"
    "3${SEP}mips-mti-none-elf"
    "3${SEP}mipsel-mti-none-elf"
    "3${SEP}mipsisa32r6-unknown-linux-gnu"
    "3${SEP}mipsisa32r6el-unknown-linux-gnu"
    "3${SEP}mipsisa64r6-unknown-linux-gnuabi64"
    "3${SEP}mipsisa64r6el-unknown-linux-gnuabi64"
    "3${SEP}msp430-none-elf"
    "3${SEP}powerpc-unknown-freebsd"
    "3${SEP}powerpc-unknown-linux-gnuspe"
    "3${SEP}powerpc-unknown-linux-musl"
    "3${SEP}powerpc-unknown-linux-muslspe"
    "3${SEP}powerpc-unknown-netbsd"
    "3${SEP}powerpc-unknown-openbsd"
    "3${SEP}powerpc-wrs-vxworks"
    "3${SEP}powerpc-wrs-vxworks-spe"
    "3${SEP}powerpc64-ibm-aix"
    "3${SEP}powerpc64-unknown-freebsd"
    "3${SEP}powerpc64-unknown-linux-musl"
    "3${SEP}powerpc64-unknown-openbsd"
    "3${SEP}powerpc64-wrs-vxworks"
    "3${SEP}powerpc64le-unknown-freebsd"
    "3${SEP}riscv32-wrs-vxworks"
    "3${SEP}riscv32e-unknown-none-elf"
    "3${SEP}riscv32em-unknown-none-elf"
    "3${SEP}riscv32emc-unknown-none-elf"
    "3${SEP}riscv32gc-unknown-linux-gnu"
    "3${SEP}riscv32gc-unknown-linux-musl"
    "3${SEP}riscv32im-risc0-zkvm-elf"
    "3${SEP}riscv32ima-unknown-none-elf"
    "3${SEP}riscv32imac-esp-espidf"
    "3${SEP}riscv32imac-unknown-nuttx-elf"
    "3${SEP}riscv32imac-unknown-xous-elf"
    "3${SEP}riscv32imafc-esp-espidf"
    "3${SEP}riscv32imafc-unknown-nuttx-elf"
    "3${SEP}riscv32imc-esp-espidf"
    "3${SEP}riscv32imc-unknown-nuttx-elf"
    "3${SEP}riscv64-linux-android"
    "3${SEP}riscv64-wrs-vxworks"
    "3${SEP}riscv64gc-unknown-freebsd"
    "3${SEP}riscv64gc-unknown-fuchsia"
    "3${SEP}riscv64gc-unknown-hermit"
    "3${SEP}riscv64gc-unknown-netbsd"
    "3${SEP}riscv64gc-unknown-nuttx-elf"
    "3${SEP}riscv64gc-unknown-openbsd"
    "3${SEP}riscv64imac-unknown-nuttx-elf"
    "3${SEP}s390x-unknown-linux-musl"
    "3${SEP}sparc-unknown-linux-gnu"
    "3${SEP}sparc-unknown-none-elf"
    "3${SEP}sparc64-unknown-netbsd"
    "3${SEP}sparc64-unknown-openbsd"
    "3${SEP}thumbv4t-none-eabi"
    "3${SEP}thumbv5te-none-eabi"
    "3${SEP}thumbv6m-nuttx-eabi"
    "3${SEP}thumbv7a-pc-windows-msvc"
    "3${SEP}thumbv7a-uwp-windows-msvc"
    "3${SEP}thumbv7a-nuttx-eabi"
    "3${SEP}thumbv7a-nuttx-eabihf"
    "3${SEP}thumbv7em-nuttx-eabi"
    "3${SEP}thumbv7em-nuttx-eabihf"
    "3${SEP}thumbv7m-nuttx-eabi"
    "3${SEP}thumbv7neon-unknown-linux-musleabihf"
    "3${SEP}thumbv8m.base-nuttx-eabi"
    "3${SEP}thumbv8m.main-nuttx-eabi"
    "3${SEP}thumbv8m.main-nuttx-eabihf"
    "3${SEP}wasm64-unknown-unknown"
    "3${SEP}wasm32-wali-linux-musl"
    "3${SEP}x86_64-apple-tvos"
    "3${SEP}x86_64-apple-watchos-sim"
    "3${SEP}x86_64-lynx-lynxos178"
    "3${SEP}x86_64-pc-cygwin"
    "3${SEP}x86_64-pc-nto-qnx710"
    "3${SEP}x86_64-pc-nto-qnx710_iosock"
    "3${SEP}x86_64-pc-nto-qnx800"
    "3${SEP}x86_64-unikraft-linux-musl"
    "3${SEP}x86_64-unknown-dragonfly"
    "3${SEP}x86_64-unknown-haiku"
    "3${SEP}x86_64-unknown-hermit"
    "3${SEP}x86_64-unknown-hurd-gnu"
    "3${SEP}x86_64-unknown-l4re-uclibc"
    "3${SEP}x86_64-unknown-linux-none"
    "3${SEP}x86_64-unknown-openbsd"
    "3${SEP}x86_64-unknown-trusty"
    "3${SEP}x86_64-uwp-windows-gnu"
    "3${SEP}x86_64-uwp-windows-msvc"
    "3${SEP}x86_64-win7-windows-gnu"
    "3${SEP}x86_64-win7-windows-msvc"
    "3${SEP}x86_64-wrs-vxworks"
    "3${SEP}x86_64h-apple-darwin"
    "3${SEP}xtensa-esp32-espidf"
    "3${SEP}xtensa-esp32-none-elf"
    "3${SEP}xtensa-esp32s2-espidf"
    "3${SEP}xtensa-esp32s2-none-elf"
    "3${SEP}xtensa-esp32s3-espidf"
    "3${SEP}xtensa-esp32s3-none-elf"
)

declare -ag results=()
declare -ag targets_built=()
declare -ag targets_failed=()
declare -ag targets_skipped=()
declare -i i=1

# check column supports newer advanced options, i.e., support Darwin OS
COLUMN_NEW=false
if (echo | column --table --separator "$SEP" --output-separator "$SEP" --table-columns 'foo') &>/dev/null; then
    COLUMN_NEW=true
fi
readonly COLUMN_NEW

cd "$(dirname "$0")/.."

readonly PROJECT_ROOT=$(pwd)
if [[ "${PROJECT_ROOT}" =~ [[:cntrl:]] ]]; then
    echo "ERROR: project directory must not contain control characters" >&2
    exit 1
fi

readonly PROJECT_MANIFEST="${PROJECT_ROOT}/Cargo.toml"
if [[ ! -f "${PROJECT_MANIFEST}" ]]; then
    echo "ERROR cannot find PROJECT_MANIFEST at ${PROJECT_MANIFEST}" >&2
    exit 1
fi

# if passed `--print-targets` then pretty print the targets and exit.
# useful for seeing the numeric indexes and then choosing an index FILTER value.
if [[ "${1-}" == '--print-targets' ]]; then
    (
        declare -i i=0
        while [[ $i -lt ${#TIER_TARGETS[@]} ]]; do
            echo "${i}${SEP}${TIER_TARGETS[$i]}"
            ((i++)) || true
        done
    ) | column -t -s "${SEP}"
    exit 0
fi

if [[ ! "${DIROUT-}" ]]; then
    echo "ERROR must set DIROUT" >&2
    exit 1
fi

for arg in "$@"; do
    case "$arg" in
        --target-dir|--target-dir=*|--cargo-target-dir|--cargo-target-dir=*)
            echo "ERROR: $arg cannot be used with isolated cross builds" >&2
            exit 1
            ;;
    esac
done

function print_msrv() {
    grep -o -m1 -Ee '^rust-version[[:space:]]?=[[:space:]]?".*"' "${PROJECT_MANIFEST}" | sed -E 's/^rust-version[[:blank:]]*=[[:blank:]]*"([[:digit:]\.]+)"/\1/'
}

if [[ ! "${MSRV-}" ]]; then
    MSRV=$(print_msrv)
fi

readonly CLR_RESET="\e[39m"  # reset color to default
readonly CLR_LG="\e[92m"  # color light green
readonly CLR_RED="\e[91m"  # color red
readonly CLR_YELLOW="\e[93m"  # color yellow

function print_hl() {
    echo -e "${CLR_LG}"
    printf '%0.s─' $(seq 1 ${COLUMNS-$(tput cols)})
    echo -e "${CLR_RESET}"
}

# print the $results array in a almost-ready markdown table
# (requires small tweaks to display properly)
function print_results() {
    if [[ "${#results[@]}" -eq 0 ]]; then
        return 0
    fi
    echo -e "${CLR_LG}"
    # args for old column, i.e., support Darwin OS
    declare -a args=(
        "-t"
        "-s" "$SEP"
    )
    if $COLUMN_NEW; then
        # args for new column
        args=(
            --table \
            --separator "$SEP" \
            --output-separator " $SEP " \
            --table-columns '_,Tier,Target,Time,Result,Command' \
        )
    fi
    column "${args[@]}" \
    < <(
        # header delimiter row
        echo "${SEP}---${SEP}---${SEP}---${SEP}---${SEP}---${SEP}"
        # results rows
        for target_tier_result_command in "${results[@]}"; do
            echo "${SEP}${target_tier_result_command}${SEP}"
        done | sort
    )
    echo -e "${CLR_RESET}"
}

function print_results_md() {
    if [[ "${REPORT_PATH-}" ]]; then
        print_results | tee "${REPORT_PATH}"
    else
        print_results
    fi
}

RUN_DIR=

# print results
function exit_ () {
    set +e
    if [[ "${RUN_DIR-}" && -d "${RUN_DIR}" ]]; then
        rm -rf -- "${RUN_DIR}" || echo "ERROR: could not remove staging directory '${RUN_DIR}'" >&2
    fi
    print_results_md
    echo
    echo -e "${CLR_LG}Successfully built for ${#targets_built[@]} targets:${CLR_RESET}"
    for target in "${targets_built[@]}"; do
        echo "${target}"
    done
    echo
    echo -e "${CLR_RED}Failed to build for ${#targets_failed[@]} targets:${CLR_RESET}"
    for target in "${targets_failed[@]}"; do
        echo "${target}"
    done
    echo
    echo -e "${CLR_YELLOW}Skipped build for ${#targets_skipped[@]} targets:${CLR_RESET}"
    for target in "${targets_skipped[@]}"; do
        echo "${target}"
    done
    echo
    echo "FILTER=${FILTER-}"
    echo
}

trap exit_ EXIT

# print passed seconds integer as HH:MM:SS
function seconds_to_hms() {
    declare -i total_seconds="$1"
    declare -i hours=$((total_seconds / 3600))
    declare -i minutes=$(((total_seconds % 3600) / 60))
    declare -i seconds=$((total_seconds % 60))
    printf "%02d:%02d:%02d" "$hours" "$minutes" "$seconds"
}

mkdir -p "${DIROUT}"
DIROUT=$(readlink -f "${DIROUT}")
readonly DIROUT
RELEASE_DIR="${DIROUT}/release"
mkdir -p "${RELEASE_DIR}"
readonly RELEASE_DIR
RUN_DIR=$(mktemp -d "${DIROUT}/cross-build.XXXXXXXXXX")
readonly RUN_DIR
readonly RUN_ID="${RUN_DIR##*/}"
readonly DATE_NOW=$(date '+%Y%m%dT%H%M%S')
readonly REPORT_PATH="${DIROUT}/cross-builds-${FILTER--}${DATE_NOW}.md"
echo "Report: ${REPORT_PATH}" >&2

# publish_package publishes a built package to the appropriate directories, handling staging and conflicts.
# return code 0 means success
# return code 1 means failure
# return code 2 means already published
# return code 3 means error
function publish_package() {
    local s4_file="$1"
    local target="$2"
    local version="$3"
    local stage
    local attempt
    local ext
    local dest_name
    local zip_name
    local output
    local file
    local -a staged_files
    local -a destinations
    local -a published_files=()
    stage=$(mktemp -d "${RUN_DIR}/package.XXXXXXXXXX") || return 1
    if ! env DIROUT="${stage}" "${PROJECT_ROOT}/tools/package-binary.sh" "${s4_file}" "${target}" "${version}"; then
        echo "ERROR: packaging failed for '${target}'" >&2
        rm -rf -- "${stage}" || echo "ERROR: could not remove staging directory '${stage}'" >&2
        return 1
    fi

    ext=
    [[ "${s4_file}" == *.exe ]] && ext=.exe
    dest_name="${BIN}_${target}_v${version}${ext}"
    zip_name="${BIN}_${target}_v${version}.zip"
    staged_files=("${stage}/${dest_name}" "${stage}/${dest_name}.sha256" "${stage}/release/${zip_name}.sha256" "${stage}/release/${zip_name}")
    destinations=("${DIROUT}/${dest_name}" "${DIROUT}/${dest_name}.sha256" "${RELEASE_DIR}/${zip_name}.sha256" "${RELEASE_DIR}/${zip_name}")

    if [[ -e "${RELEASE_DIR}/${zip_name}" ]]; then
        rm -rf -- "${stage}" || {
            echo "ERROR: could not remove staging directory '${stage}'" >&2
            return 1
        }
        return 2
    fi
    for output in "${destinations[@]}"; do
        if [[ -e "${output}" ]]; then
            echo "ERROR: destination already exists '${output}'" >&2
            rm -rf -- "${stage}" || echo "ERROR: could not remove staging directory '${stage}'" >&2
            return 1
        fi
    done

    for ((attempt=0; attempt<${#staged_files[@]}; attempt++)); do
        file="${staged_files[$attempt]}"
        output="${destinations[$attempt]}"
        if ! (set -eux; mv -- "${file}" "${output}"); then
            echo "ERROR: could not publish '${output}'; publication stopped. Already published files remain:" >&2
            for file in "${published_files[@]}"; do
                echo "  ${file}" >&2
            done
            rm -rf -- "${stage}" || echo "ERROR: could not remove staging directory '${stage}'" >&2
            return 3
        fi
        published_files+=("${output}")
    done
    rm -rf -- "${stage}" || {
        echo "ERROR: could not remove staging directory '${stage}'" >&2
        return 1
    }
}

# print the profile arguments based on $1
function output_profile() {
    local profile=debug
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --release|-r) profile=release ;;
            --profile)
                shift
                if [[ $# -gt 0 ]]; then profile="$1"; fi
                ;;
            --profile=*) profile="${1#--profile=}" ;;
        esac
        if [[ $# -gt 0 ]]; then shift; fi
    done
    case "${profile}" in
        dev|test) profile=debug ;;
        bench) profile=release ;;
    esac
    printf '%s\n' "${profile}"
}

BUILD_OUTPUT_PROFILE=$(output_profile "$@")
readonly BUILD_OUTPUT_PROFILE

# print the grepped project version from `Cargo.toml`
function print_version() {
    grep -o -m1 -Ee '^version[[:space:]]*=[[:space:]]*".*"' "${PROJECT_ROOT}/Cargo.toml" | sed -Ee 's/^version[[:blank:]]*=[[:blank:]]*"([[:digit:]\.]+)"/\1/'
}

if ! command -v cross &>/dev/null; then
    echo "ERROR: cargo cross not found, please install it with 'cargo install --locked cross cargo-cross'" >&2
    exit 1
fi

readonly BIN="s4"
VERSION=$(print_version)
readonly VERSION

set +e

# return 0 if target is acceptable by the `$FILTER`
# else return 1
function filter_target() {
    declare -r target=${1}
    declare -ir index=${2}

    if [[ ! "${FILTER-}" ]]; then
        return 0
    fi
    if [[ "${FILTER}" =~ ^[0-9]+$ ]]; then
        # allow a filter that is just a number.
        # indexes at or greater than that number are matched
        # e.g. FILTER=5 will match indexes 5 and above.
        # useful for resuming from an interrupted run of this script.
        if [[ ${index} -ge "${FILTER}" ]]; then
            return 0
        fi
    elif [[ "${FILTER}" =~ ^[0-9]+-[0-9]+$ ]]; then
        # allow a filter that is a number range `a-b`
        # e.g. `5-33` is all targets at indexes 5 through 33 inclusive.
        local start=${FILTER%-*}
        local end=${FILTER#*-}
        if [[ ${index} -ge "$start" && ${index} -le "$end" ]]; then
            return 0
        fi
    elif [[ "${target}" =~ ${FILTER} ]]; then
        # filter is a glob pattern; match the target against it
        return 0
    fi
    return 1
}

function rustup_toolchain_install_try() (
    set -x
    rustup toolchain install --no-self-update --profile minimal --target "$1" "$2"
)

function rustup_target_add_try() (
    set -x
    rustup target add "$1"
)

function rustup_component_add() (
    set -x
    rustup component add --toolchain "nightly" --target "$1" rustc rust-src rust-std
)

function set_build_dir() {
    export CARGO_TARGET_DIR="$1"
    export CARGO_BUILD_TARGET_DIR="${CARGO_TARGET_DIR}"
    export CARGO_BUILD_BUILD_DIR="${CARGO_TARGET_DIR}"
    local config_target_dir="${CARGO_TARGET_DIR//\\/\\\\}"
    config_target_dir="${config_target_dir//\"/\\\"}"
    build_dir_args=(
        --config "build.target-dir=\"${config_target_dir}\""
        --config "build.build-dir=\"${config_target_dir}\""
    )
}

function cross_build() {
    local target="$1"
    local stage
    shift
    printf -v cross_build_command '%q ' cross build --quiet --target "${target}" "$@" --target-dir "${CARGO_TARGET_DIR}"
    cross_build_command="${cross_build_command% }"
    for stage in probe regular; do
        (
            export S4_BUILD_REGEX_PRINT=0
            if [[ "${stage}" == probe ]]; then
                set_build_dir "${CARGO_TARGET_DIR}/probe"
                export S4_BUILD_REGEX=1
            fi
            unset CARGO_BUILD_BUILD_DIR
            if [[ ! "${CROSS_CONTAINER_OPTS+x}" && "${DOCKER_OPTS+x}" ]]; then
                CROSS_CONTAINER_OPTS="${DOCKER_OPTS}"
                unset DOCKER_OPTS
            fi
            export CROSS_CONTAINER_OPTS="${CROSS_CONTAINER_OPTS-} -e S4_BUILD_REGEX_PRINT=0"
            if [[ "${stage}" == probe ]]; then
                CROSS_CONTAINER_OPTS="${CROSS_CONTAINER_OPTS} -e S4_BUILD_REGEX=1"
            fi
            set -x
            env CROSS_CONTAINER_OPTS="${CROSS_CONTAINER_OPTS}" \
                cross build --quiet --target "${target}" "$@" --target-dir "${CARGO_TARGET_DIR}"
        ) || return $?
    done
}

function cargo_build_nightly() {
    local target="$1"
    local stage
    shift
    printf -v cargo_build_nightly_command '%q ' cargo +nightly build --quiet -Zbuild-std --target "${target}" "$@" "${build_dir_args[@]}"
    cargo_build_nightly_command="${cargo_build_nightly_command% }"
    for stage in probe regular; do
        (
            export S4_BUILD_REGEX_PRINT=0
            if [[ "${stage}" == probe ]]; then
                set_build_dir "${CARGO_TARGET_DIR}/probe"
                export S4_BUILD_REGEX=1
            fi
            set -x
            env export S4_BUILD_REGEX=${S4_BUILD_REGEX} S4_BUILD_REGEX_PRINT=${S4_BUILD_REGEX_PRINT} \
                cargo +nightly build --quiet -Zbuild-std --target "${target}" "$@" "${build_dir_args[@]}"
        ) || return $?
    done
}

function cargo_cross_build() {
    local target="$1"
    local stage
    shift
    printf -v cargo_cross_build_command '%q ' cargo cross build --quiet --target "${target}" "$@" "${build_dir_args[@]}"
    cargo_cross_build_command="${cargo_cross_build_command% }"
    for stage in probe regular; do
        (
            export S4_BUILD_REGEX_PRINT=0
            if [[ "${stage}" == probe ]]; then
                set_build_dir "${CARGO_TARGET_DIR}/probe"
                export S4_BUILD_REGEX=1
            fi
            set -x
            env S4_BUILD_REGEX=${S4_BUILD_REGEX} S4_BUILD_REGEX_PRINT=${S4_BUILD_REGEX_PRINT} \
                cargo cross build --quiet --target "${target}" "$@" "${build_dir_args[@]}"
        ) || return $?
    done
}

# determine targets to build
print_hl
declare -a tiertargets_to_build=()
declare -i i=-1
for TIER_TARGET in "${TIER_TARGETS[@]}"; do
    TIER=$(echo -n "${TIER_TARGET}" | cut -d "$SEP" -f 1)
    TARGET=$(echo -n "${TIER_TARGET}" | cut -d "$SEP" -f 2-)
    let i+=1 || true
    # check for matching filter
    if ! filter_target "$TARGET" "$i"; then
        echo -e "${CLR_YELLOW}skip ${TARGET}${CLR_RESET}" >&2
        continue
    fi
    tiertargets_to_build+=("$TIER_TARGET")
done
echo >&2
echo "Building for ${#tiertargets_to_build[@]} targets..." >&2
for tt in "${tiertargets_to_build[@]}"; do
    TARGET=$(echo -n "${tt}" | cut -d "$SEP" -f 2-)
    echo -e "${CLR_LG}Build ${TARGET}${CLR_RESET}" >&2
done
echo >&2

# foreach target; build and copy the binary to $DIROUT with meaningful names
declare -i i=1
for TIER_TARGET in "${tiertargets_to_build[@]}"; do
    print_hl
    TIER=$(echo -n "${TIER_TARGET}" | cut -d "$SEP" -f 1)
    TARGET=$(echo -n "${TIER_TARGET}" | cut -d "$SEP" -f 2-)
    echo >&2
    echo -e "${CLR_YELLOW}Try ${i} of ${#tiertargets_to_build[@]}: tier ${TIER} target ${TARGET}${CLR_RESET}" >&2
    echo >&2
    let i+=1 || true
    declare -i start_time=${SECONDS}
    zip_name="${BIN}_${TARGET}_v${VERSION}.zip"
    zip_path="${RELEASE_DIR}/${zip_name}"
    # check if it already exists
    if [[ -e "${zip_path}" ]]; then
        echo -e "${CLR_LG}Already exists '${zip_path}' for target ${TARGET}, skipping build...${CLR_RESET}" >&2
        declare -i total_time=$((SECONDS - start_time))
        time_hms=$(seconds_to_hms "$total_time")
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}⚠️ exists${SEP}_"
        targets_skipped+=("$TARGET")
        # long running script; print progress in real-time
        print_results_md
        continue
    fi
    set_build_dir "${PROJECT_ROOT}/target/cross-builds/${TARGET}"
    echo "Build directory: ${CARGO_TARGET_DIR}" >&2

    rustup_toolchain_install_try_result=true
    if ! rustup_toolchain_install_try "${TARGET}" "${MSRV}"; then
        rustup_toolchain_install_try_result=false
        echo -e "${CLR_RED}ERROR: could not install toolchain for target '${TARGET}'${CLR_RESET}" >&2
        echo >&2
    fi

    rustup_target_add_try_result=true
    if ! rustup_target_add_try "${TARGET}"; then
        rustup_target_add_try_result=false
        echo -e "${CLR_RED}ERROR: could not add target '${TARGET}'${CLR_RESET}" >&2
        echo >&2
    fi

    built_command='_'
    cross_build_result=true
    if ! cross_build "${TARGET}" "$@"; then
        cross_build_result=false
    else
        built_command="${cross_build_command}"
    fi

    rustup_component_add_result=false
    cargo_build_nightly_result=false
    if ! $cross_build_result; then
        rustup_component_add_result=true
        if ! rustup_component_add "${TARGET}"; then
            rustup_component_add_result=false
            echo -e "${CLR_RED}ERROR: could not add nightly components for target '${TARGET}'${CLR_RESET}" >&2
        fi
    fi
    if $rustup_component_add_result; then
        cargo_build_nightly_result=true
        if ! cargo_build_nightly "${TARGET}" "$@"; then
            cargo_build_nightly_result=false
        else
            built_command="${cargo_build_nightly_command}"
        fi
    fi

    cargo_cross_build_result=false
    if ! $cross_build_result && ! $cargo_build_nightly_result; then
        cargo_cross_build_result=true
        if ! cargo_cross_build "${TARGET}" "$@"; then
            cargo_cross_build_result=false
        else
            built_command="${cargo_cross_build_command}"
        fi
    fi

    package_status=1
    if $cross_build_result || $cargo_build_nightly_result || $cargo_cross_build_result; then
        s4_files=()
        for s4_file in "${CARGO_TARGET_DIR}/${TARGET}/${BUILD_OUTPUT_PROFILE}/${BIN}" "${CARGO_TARGET_DIR}/${TARGET}/${BUILD_OUTPUT_PROFILE}/${BIN}.exe"; do
            if [[ -f "${s4_file}" ]]; then
                s4_files+=("${s4_file}")
            fi
        done
        if [[ "${#s4_files[@]}" -eq 1 ]]; then
            publish_package "${s4_files[0]}" "${TARGET}" "${VERSION}"
            package_status=$?
        else
            echo "ERROR: found ${#s4_files[@]} binaries for '${TARGET}', expected 1" >&2
            exit 1
        fi
    fi

    declare -i total_time=$((SECONDS - start_time))
    time_hms=$(seconds_to_hms "$total_time")
    if [[ "${built_command}" == '_' ]]; then
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}❌ fail${SEP}_"
        targets_failed+=("$TARGET")
    elif [[ "${package_status}" -eq 0 ]]; then
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}✅ pass${SEP}${built_command}"
        targets_built+=("$TARGET")
    elif [[ "${package_status}" -eq 2 ]]; then
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}⚠️ exists${SEP}${built_command}"
        targets_skipped+=("$TARGET")
    else
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}❌ package${SEP}${built_command}"
        targets_failed+=("$TARGET")
    fi
    if [[ "${package_status}" -eq 3 ]]; then
        exit 1
    fi
    # long running script; print progress in real-time
    print_results_md
done
