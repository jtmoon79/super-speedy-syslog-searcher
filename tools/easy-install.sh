#!/usr/bin/env bash
#
# easy installer for s4 release binaries
#
# Usage:
#   VER=0.9.81 ./tools/easy-install.sh
#
# to run this file remotely:
#    curl --silent 'https://raw.githubusercontent.com/jtmoon79/super-speedy-syslog-searcher/main/tools/easy-install.sh' | bash --norc --noprofile

set -euo pipefail

readonly VER=${VER-0.8.80}
readonly COLOR_GREEN='\033[32m'
readonly COLOR_YELLOW='\033[33m'
readonly COLOR_RED='\033[31m'
readonly COLOR_RESET='\033[0m'

function info() {
    echo -e "\n${COLOR_GREEN}info: $*${COLOR_RESET}" >&2
}

function warn() {
    echo -e "${COLOR_YELLOW}warning: $*${COLOR_RESET}" >&2
}

function die() {
    echo -e "${COLOR_RED}error: $*${COLOR_RESET}" >&2
    exit 1
}

function require_program() {
    local -r program="$1"
    local -r help_text="$2"
    if ! command -v "${program}" >/dev/null 2>&1; then
        die "required program '${program}' was not found. ${help_text}"
    fi
}

function check_optional_program() {
    local -r program="$1"
    local -r help_text="${2-}"
    if ! command -v "${program}" >/dev/null 2>&1; then
        if [[ -n "${help_text}" ]]; then
            echo "Consider installing '${program}' via your package manager. ${help_text}" >&2
        fi
        return 1
    fi
    return 0
}

function choose_install_dir() {
    # Preferred install locations in order.
    local -ar preferred_dirs=(
        "${HOME}/.cargo/bin"
        "${HOME}/bin"
        "/usr/local/bin"
    )

    local dir
    for dir in "${preferred_dirs[@]}"; do
        [[ -d "${dir}" ]] || continue
        [[ -w "${dir}" ]] || continue
        echo "${dir}"
        return 0
    done

    local -r old_ifs="${IFS}"
    IFS=:
    for dir in ${PATH}; do
        [[ -n "${dir}" ]] || continue
        [[ -d "${dir}" ]] || continue
        [[ -w "${dir}" ]] || continue
        echo "${dir}"
        IFS="${old_ifs}"
        return 0
    done
    IFS="${old_ifs}"

    return 1
}

function install_default_target_if_needed() {
    if command -v default-target >/dev/null 2>&1; then
        return 0
    fi

    warn "required program 'default-target' was not found."
    if ! command -v cargo >/dev/null 2>&1; then
        die "cannot auto-install 'default-target' because 'cargo' is not available.\n       Install Rust (see https://rustup.rs/), then run:\n       cargo install --locked default-target"
    fi

    info "attempting to install 'default-target' using cargo..."
    if ! cargo install --locked default-target; then
        die "failed to install 'default-target'. Try manually: cargo install --locked default-target"
    fi

    if ! command -v default-target >/dev/null 2>&1; then
        die "'default-target' still not found after install. Ensure cargo bin directory is in PATH."
    fi
}

readonly SCRIPT_NAME=$(basename "$0")

function main() {
    require_program curl "Install curl via your package manager."
    require_program unzip "Install unzip via your package manager."

    check_optional_program sha256sum "SHA-256 verification will be skipped."
    declare -i has_sha256sum=$?

    check_optional_program which "The script will use command -v instead."
    declare -i has_which=$?

    install_default_target_if_needed

    local target_triple
    target_triple=$(default-target)
    [[ -n "${target_triple}" ]] || die "default-target returned an empty target triple."

    WORKDIR=$(mktemp -d "${SCRIPT_NAME}.XXXXXXXX")
    readonly WORKDIR
    trap "rm -rf -- '${WORKDIR}'" EXIT

    local zip_path="${WORKDIR}/s4.zip"
    local url="https://github.com/jtmoon79/super-speedy-syslog-searcher/releases/download/${VER}/s4_${target_triple}_v${VER}.zip"

    info "download release ${VER} for target ${target_triple}..."
    (set -x; curl -fL --retry 2 --output "${zip_path}" "${url}")

    info "extract archive..."
    (set -x; unzip -o -d "${WORKDIR}" "${zip_path}") >/dev/null

    if [[ ! -f "${WORKDIR}/s4" ]]; then
        die "downloaded archive did not contain 's4' binary. URL used: ${url}"
    fi

    chmod +x -- "${WORKDIR}/s4"

    if [[ "${has_sha256sum}" -eq 0 ]]; then
        if [[ -f "${WORKDIR}/s4.sha256" ]]; then
            info "verify SHA-256 checksum..."
            (
                cd -- "${WORKDIR}"
                (set -x; sha256sum -c s4.sha256)
            )
        else
            warn "checksum file 's4.sha256' was not found in archive; skipping verification."
        fi
    fi

    info "check downloaded binary..."
    (set -x; "${WORKDIR}/s4" --version)

    local install_dir
    install_dir=$(choose_install_dir) || die "could not find or create a writable install directory."

    info "install to ${install_dir}/s4"
    (set -x; cp -vf -- "${WORKDIR}/s4" "${install_dir}/s4")

    info "verify installed binary path..."
    if [[ "${has_which}" -eq 0 ]]; then
        if which s4 >/dev/null 2>&1; then
            (set -x; which s4)
        else
            warn "'which s4' did not find s4 in current PATH."
        fi
    else
        if command -v s4 >/dev/null 2>&1; then
            (set -x; command -v s4)
        else
            warn "'command -v s4' did not find s4 in current PATH."
        fi
    fi

    info "check installed binary:"
    (set -x; s4 --version)

    info "done."
}

main "$@"
