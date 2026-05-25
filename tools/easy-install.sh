#!/bin/sh
#
# easy installer for s4 release binaries
# POSIX shell compatible.
#
# Usage:
#   VER=0.9.81 ./tools/easy-install.sh
#
# to run this file remotely:
#    curl --silent 'https://raw.githubusercontent.com/jtmoon79/super-speedy-syslog-searcher/main/tools/easy-install.sh' | sh

set -eu

readonly SCRIPT_NAME='easy-install.sh'

VER=${VER-0.8.80}  # PROJECT VERSION LAST PUBLISHED
readonly VER

COLOR_GREEN=$(printf '\033[32m')
COLOR_YELLOW=$(printf '\033[33m')
COLOR_RED=$(printf '\033[31m')
COLOR_RESET=$(printf '\033[0m')
readonly COLOR_GREEN COLOR_YELLOW COLOR_RED COLOR_RESET

info() {
    # shellcheck disable=SC2059
    printf '\n%sinfo: %s%s\n' "${COLOR_GREEN}" "$*" "${COLOR_RESET}" >&2
}

warn() {
    # shellcheck disable=SC2059
    printf '%swarning: %s%s\n' "${COLOR_YELLOW}" "$*" "${COLOR_RESET}" >&2
}

die() {
    # shellcheck disable=SC2059
    printf '%serror: %s%s\n' "${COLOR_RED}" "$*" "${COLOR_RESET}" >&2
    exit 1
}

require_program() {
    program=$1
    help_text=$2
    if ! command -v "${program}" >/dev/null 2>&1; then
        die "required program '${program}' was not found. ${help_text}"
    fi
}

check_optional_program() {
    program=$1
    help_text=${2-}
    if ! command -v "${program}" >/dev/null 2>&1; then
        if [ -n "${help_text}" ]; then
            echo "Consider installing '${program}' via your package manager. ${help_text}" >&2
        fi
        return 1
    fi
    return 0
}

choose_install_dir() {
    # search these common user bin directories first
    for dir in \
        "${HOME}/.cargo/bin" \
        "${HOME}/.local/bin" \
        "${HOME}/bin" \
        "/usr/local/bin"
    do
        [ -d "${dir}" ] || continue
        [ -w "${dir}" ] || continue
        echo "${dir}"
        return 0
    done

    # now just search PATH for a writable directory; hopefully the user is okay with this
    IFS_OLD=${IFS}
    IFS=:
    for dir in ${PATH}; do
        [ -n "${dir}" ] || continue
        [ -d "${dir}" ] || continue
        [ -w "${dir}" ] || continue
        echo "${dir}"
        IFS="${IFS_OLD}"
        return 0
    done
    IFS="${IFS_OLD}"

    return 1
}

install_default_target_if_needed() {
    if command -v default-target >/dev/null 2>&1; then
        return 0
    fi

    warn "required program 'default-target' was not found."
    if ! command -v cargo >/dev/null 2>&1; then
        die "cannot auto-install 'default-target' because 'cargo' is not available. Install Rust (see https://rustup.rs/), then run: cargo install --locked default-target"
    fi

    info "attempting to install 'default-target' using cargo..."
    if ! (set -x; cargo install --locked default-target); then
        die "failed to install 'default-target'. Try manually: cargo install --locked default-target"
    fi

    if ! command -v default-target >/dev/null 2>&1; then
        die "'default-target' still not found after install. Ensure cargo bin directory is in PATH."
    fi
}

main() {
    require_program curl "Install curl via your package manager."
    require_program unzip "Install unzip via your package manager."

    if check_optional_program sha256sum "SHA-256 verification will be skipped."; then
        has_sha256sum=0
    else
        has_sha256sum=1
    fi

    if check_optional_program which "The script will use command -v instead."; then
        has_which=0
    else
        has_which=1
    fi

    # TODO: how to get default platform target triple without needing rust installed?
    install_default_target_if_needed

    target_triple=$(default-target)
    [ -n "${target_triple}" ] || die "default-target returned an empty target triple."

    WORKDIR=$(mktemp -d "${SCRIPT_NAME}.tmpd.XXXXXXXX")
    readonly WORKDIR
    trap 'rm -rf -- "$WORKDIR"' 0

    zip_name="s4_${target_triple}_v${VER}.zip"
    zip_path="${WORKDIR}/${zip_name}"
    url_zip="https://github.com/jtmoon79/super-speedy-syslog-searcher/releases/download/${VER}/${zip_name}"

    checksum_name="${zip_name}.sha256"
    checksum_path="${WORKDIR}/${checksum_name}"
    url_checksum="${url_zip}.sha256"

    info "download release ${VER} for target ${target_triple}..."
    (set -x; curl --fail --location --retry 2 --output "${zip_path}" "${url_zip}")

    if [ "${has_sha256sum}" -eq 0 ] \
       && curl --head --fail --location --retry 2 --silent --fail --output /dev/null "${url_checksum}"; then
        info "download checksum file..."
        (set -x; curl --fail --location --retry 2 --output "${checksum_path}" "${url_checksum}")
        info "verify SHA-256 checksum of zip file..."
        (
            cd -- "${WORKDIR}"
            (set -x; sha256sum -c "${checksum_name}")
        )
    else
        warn "checksum file not found at ${url_checksum}"
    fi

    info "extract archive..."
    (set -x; unzip -o -d "${WORKDIR}" "${zip_path}") >/dev/null

    # handle both Unix and Windows-style zip contents
    if [ -f "${WORKDIR}/s4" ]; then
        binary_name='s4'
    elif [ -f "${WORKDIR}/s4.exe" ]; then
        binary_name='s4.exe'
    else
        die "downloaded archive contained neither 's4' nor 's4.exe'. From URL ${url_zip}"
    fi
    binary_path="${WORKDIR}/${binary_name}"
    info "using extracted binary '${binary_name}'"

    chmod -v +x -- "${binary_path}"
    chmod -v -w -- "${binary_path}"

    if [ "${has_sha256sum}" -eq 0 ]; then
        binary_checksum_name="${binary_name}.sha256"
        if [ -f "${WORKDIR}/${binary_checksum_name}" ]; then
            info "verify SHA-256 checksum of binary file..."
            (
                cd -- "${WORKDIR}"
                (set -x; sha256sum -c "${binary_checksum_name}")
            )
        else
            warn "checksum file '${binary_checksum_name}' was not found in archive; skipping verification."
        fi
    fi

    info "check downloaded binary can run..."
    (set -x; "${binary_path}" --version)

    install_dir=$(choose_install_dir) || die "could not find or create a writable install directory."

    install_path="${install_dir}/${binary_name}"
    info "install binary to ${install_path}"
    (set -x; cp -vf -- "${binary_path}" "${install_path}")

    info "verify installed binary path..."
    if [ "${has_which}" -eq 0 ]; then
        if which "${binary_name}" >/dev/null 2>&1; then
            (set -x; which "${binary_name}")
        else
            warn "'which ${binary_name}' did not find ${binary_name} in current PATH."
        fi
    else
        if command -v "${binary_name}" >/dev/null 2>&1; then
            (set -x; command -v "${binary_name}")
        else
            warn "'command -v ${binary_name}' did not find ${binary_name} in current PATH."
        fi
    fi

    info "check installed binary can run..."
    (set -x; "${binary_name}" --version)

    info "done."
}

main "$@"
