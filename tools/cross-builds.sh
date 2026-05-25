#!/usr/bin/env bash
#
# run `cargo cross` checks on all supported targets.
# these targets should match those in the `rust.yml` workflow file.
# requires:
#     cargo install --locked cross cargo-cross
#
# Tier platforms:
#    https://doc.rust-lang.org/1.88.0/rustc/platform-support.html
#
# Ask AI to help with platform updates by using prompt:
#    .github/prompts/sync-rust-platform-tiers.prompt.md
#
# args $@ are passed to `cross build`
# env. vars:
#   DIROUT when set, the s4 binary built for each target will be copied to
#          DIROUT with meaningful names
#   MSRV the Rust version to install for toolchains, defaults to 1.88.0

set -euo pipefail

declare -r SEP="|"

PROJECT_MANIFEST="$(dirname -- "${0}")/../Cargo.toml"

function print_msrv() {
    grep -o -m 1 -E '^rust-version\s?=\s?".*"' "${PROJECT_MANIFEST}" | sed -E 's/^rust-version\s?=\s?"(.*)"/\1/'
}

if [[ ! "${MSRV-}" ]]; then
    MSRV=$(print_msrv)
fi

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
declare -i i=1

# print the $results array in a almost-ready markdown table
# (requires small tweaks to display properly)
function print_results() {
    echo -e "\e[92m"
    column \
        --table \
        --separator "$SEP" \
        --output-separator " $SEP " \
        --table-columns '_,Tier,Target,Time,Result,Command' \
    < <(
        # header delimiter row
        echo "${SEP}---${SEP}---${SEP}---${SEP}---${SEP}---${SEP}"
        # results rows
        for target_tier_result_command in "${results[@]}"; do
            echo "${SEP}${target_tier_result_command}${SEP}"
        done | sort
    )
    echo -e "\e[39m"
}

# print results, if $DIROUT is set, tee to `cross-builds.md`
function exit_ () {
    if [[ "${DIROUT-}" ]]; then
        print_results | tee "${DIROUT}/cross-builds.md"
    else
        print_results
    fi
    echo
    echo -e "\e[92mSuccessfully built for ${#targets_built[@]} targets:\e[39m"
    for target in "${targets_built[@]}"; do
        echo "${target}"
    done
    echo
    echo -e "\e[91mFailed to build for ${#targets_failed[@]} targets:\e[39m"
    for target in "${targets_failed[@]}"; do
        echo "${target}"
    done
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

if [[ "${DIROUT-}" ]]; then
    DIROUT=$(realpath "${DIROUT}")
fi

cd "$(dirname "$0")/.."

readonly PROJECT_ROOT=$(pwd)

# print the grepped project version from `Cargo.toml`
function print_version() {
    grep -o -m 1 -E '^version\s*=\s*".*"' "${PROJECT_ROOT}/Cargo.toml" | sed -E 's/^version\s*=\s*"(.*)"/\1/'
}

function create_sha256sum() {
    declare -r file_path="$1"
    if [[ ! -f "$file_path" ]]; then
        echo "ERROR: file not found '$file_path'" >&2
        return 1
    fi
    declare -r file_name=$(basename "$file_path")
    pushd "$(dirname "$file_path")"
    (set -x; sha256sum "$file_name") > "${file_name}.sha256"
    chmod -v -w "${file_name}.sha256"
    popd
}

readonly BIN="s4"
VERSION=$(print_version)
readonly VERSION

set +e

for TIER_TARGET in "${TIER_TARGETS[@]}"; do
    TIER=$(echo -n "${TIER_TARGET}" | cut -d "$SEP" -f 1)
    TARGET=$(echo -n "${TIER_TARGET}" | cut -d "$SEP" -f 2-)
    echo >&2
    echo -e "\e[93mTry ${i} of ${#TIER_TARGETS[@]} tier ${TIER} target ${TARGET}...\e[39m" >&2
    echo >&2
    i+=1
    declare -i start_time=${SECONDS}
    # install toolchain for the target; if it's already installed then this will be a no-op
    if ! (
        set -x
        rustup toolchain install --profile minimal --target "$TARGET" "$MSRV"
    ); then
        declare -i total_time=$((SECONDS - start_time))
        time_hms=$(seconds_to_hms "$total_time")
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}❌ toolchain install${SEP}rustup toolchain install --profile minimal --target $TARGET $MSRV"
        targets_failed+=("$TARGET")
        # long running script; print progress in real-time
        print_results
        continue
    fi
    # run cross build
    if (
        export S4_BUILD_REGEX_PRINT=1
        set -x
        cross build --target "$TARGET" "${@}"
    ); then
        declare -i total_time=$((SECONDS - start_time))
        time_hms=$(seconds_to_hms "$total_time")
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}✅ pass${SEP}cross build --target $TARGET ${*}"
        targets_built+=("$TARGET")
        # if DIROUT is set, copy the s4 binary to DIROUT with meaningful names
        if [[ "${DIROUT-}" ]]; then
            mkdir -vp "$DIROUT"
            for s4_file in $(find "target/${TARGET}" -type f \( -name "${BIN}" -o -name "${BIN}.exe" \)); do
                EXT=''
                if [[ "${s4_file}" =~ .*\.exe ]]; then
                    EXT='.exe'
                fi
                # s4_file will look like
                #     target/s390x-unknown-linux-gnu/debug/s4
                # if --release passed then
                #     target/x86_64-pc-windows-gnu/release/s4.exe
                dest_name="${BIN}_${TARGET}_v${VERSION}${EXT}"
                dest_path="${DIROUT}/${dest_name}"
                zip_name="${BIN}_${TARGET}_v${VERSION}.zip"
                # the zip file layout must match section `package.metadata.binstall` from `Cargo.toml`
                cp -av "$s4_file" "$dest_path"
                chmod -v -w "$dest_path"
                (
            set -x
                    cd "$DIROUT"
                    bin="${BIN}${EXT}"
                    rm -f "${bin}" "${bin}.sha256"
                    create_sha256sum "$dest_name"
                    chmod -v -w "${dest_name}.sha256"
                    cp -av "$dest_name" "${bin}"
                    create_sha256sum "${bin}"
                    chmod -v -w "${bin}.sha256"
                    release_dir="${DIROUT}/release"
                    mkdir -vp "${release_dir}"
                    zip_path="${release_dir}/${zip_name}"
                    zip -v9 "${zip_path}" "${bin}" "${bin}.sha256"
                    chmod -v -w "${zip_path}"
                    create_sha256sum "${zip_path}"
                    rm -vf "${bin}" "${bin}.sha256"
                )
            done
        fi
    else
        declare -i total_time=$((SECONDS - start_time))
        time_hms=$(seconds_to_hms "$total_time")
        results[${#results[@]}]="${TIER}${SEP}${TARGET}${SEP}${time_hms}${SEP}❌ fail${SEP}cross build --target $TARGET ${*}"
        targets_failed+=("$TARGET")
    fi
    # long running script; print progress in real-time
    print_results
done
