#!/bin/sh
#
# easy installer for s4 release binaries.
# POSIX shell compatible.
# Uses curl and unzip if available, fallback to python/python3.
#
# Usage:
#   VER=0.9.81 ./tools/s4-easy-install.sh
#
# Optional environment variables:
#   VER: version to install
#   IGNORE_CHECKSUM: if set to 1 then ignore a failed checksum result
#   ABI: optional override of target triple last ABI field; e.g. `gnu` or `musl`
#   INSTALL_DIR: optional override of install directory
#   TEST: if set then do not install
#
# to run this file remotely:
#    curl --silent 'https://raw.githubusercontent.com/jtmoon79/super-speedy-syslog-searcher/main/tools/s4-easy-install.sh' | sh

set -eu

# enable tracing
if [ ! -z "${S4_EASY_INSTALL_DEBUG-}" ]; then
    set -x
fi

readonly SCRIPT_NAME='s4-easy-install.sh'

VER=${VER-0.11.84}  # PROJECT VERSION LAST PUBLISHED
readonly VER

# optional override of target triple last ABI field
# e.g. `gnu` vs `musl` for Linux targets
ABI=${ABI-}
readonly ABI

# hidden override for dry run mode; i.e. testing in CI
DRY_RUN=${DRY_RUN-0}
readonly DRY_RUN

# allow override of install directory via environment variable
INSTALL_DIR=${INSTALL_DIR-}

COLOR_BOLD=$(printf '\033[1m')
COLOR_BOLD_OFF=$(printf '\033[22m')
COLOR_GREEN=$(printf '\033[32m')
COLOR_YELLOW=$(printf '\033[33m')
COLOR_RED=$(printf '\033[31m')
COLOR_RESET=$(printf '\033[0m')
readonly COLOR_GREEN COLOR_YELLOW COLOR_RED COLOR_RESET

# if set and file exists then s4 copied to location not in PATH
NOT_IN_PATH=

info() {
    # shellcheck disable=SC2059
    printf '%sinfo: %s%s\n' "${COLOR_GREEN}" "$*" "${COLOR_RESET}" >&2
}

warn() {
    # shellcheck disable=SC2059
    printf '%swarning: %s%s\n' "${COLOR_YELLOW}" "$*" "${COLOR_RESET}" >&2
    sleep 2
}

die() {
    # shellcheck disable=SC2059
    printf '%serror: %s%s\n' "${COLOR_RED}" "$*" "${COLOR_RESET}" >&2
    exit 1
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

has_command() {
    command -v "$1" >/dev/null 2>&1
}

download_file() {
    if has_command curl; then
        (set -eux; curl --silent --show-error --fail --location --output "$2" "$1")
    else
        "${PYTHON}" -c '
import shutil
import sys
try:
    from urllib.request import urlopen
except ImportError:
    from urllib2 import urlopen
with urlopen(sys.argv[1]) as response, open(sys.argv[2], "wb") as output:
    shutil.copyfileobj(response, output)
' "$1" "$2"
    fi
}

url_exists() {
    if has_command curl; then
        (set -eux; curl --head --fail --location --silent --output /dev/null "$1")
    elif [ -n "${PYTHON-}" ]; then
        "${PYTHON}" -c '
import sys
try:
    from urllib.request import Request, urlopen
    from urllib.error import HTTPError, URLError
except ImportError:
    from urllib2 import Request, urlopen, HTTPError, URLError
request = Request(sys.argv[1])
request.get_method = lambda: "HEAD"
try:
    urlopen(request).close()
except (HTTPError, URLError) as error:
    sys.stderr.write("%s\n" % error)
    sys.exit(1)
' "$1"
    else
        die "missing 'curl' or 'python'"
    fi
}

extract_zip() {
    if has_command unzip; then
        unzip -o -d "$2" "$1"
    elif [ -n "${PYTHON-}" ]; then
        "${PYTHON}" -c '
import sys
import zipfile
with zipfile.ZipFile(sys.argv[1]) as archive:
    archive.extractall(sys.argv[2])
' "$1" "$2"
    else
        die "missing 'unzip' or 'python'"
    fi
}

is_in_path() {
    dir_is_in_path="$1"
    IFS_OLD=${IFS}
    IFS=:
    for path_dir in ${PATH}; do
        if [ "${path_dir}" = "${dir_is_in_path}" ]; then
            IFS="${IFS_OLD}"
            return 0
        fi
    done
    IFS="${IFS_OLD}"
    return 1
}

choose_install_dir() {
    if [ -n "${INSTALL_DIR-}" ]; then
        if [ ! -e "${INSTALL_DIR}" ]; then
            die "INSTALL_DIR '${INSTALL_DIR}' does not exist."
        fi
        if [ ! -d "${INSTALL_DIR}" ]; then
            die "INSTALL_DIR '${INSTALL_DIR}' is not a directory."
        fi
        if [ ! -w "${INSTALL_DIR}" ]; then
            die "INSTALL_DIR '${INSTALL_DIR}' is not writable."
        fi
        echo "${INSTALL_DIR}"
        return 0
    fi
    # search these common user bin directories first
    for dir in \
        "${HOME-}/.cargo/bin" \
        "${HOME-}/.local/bin" \
        "${HOME-}/bin" \
        "/usr/local/bin"
    do
        [ -d "${dir}" ] || continue
        [ -w "${dir}" ] || continue
        if ! is_in_path "${dir}"; then
            touch "${NOT_IN_PATH}"
        fi
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

    # try creating a directory in the user's home directory
    if [ -z "${HOME-}" ]; then
        return 1
    fi
    home_bin="${HOME-}/bin"
    if mkdir -p "${home_bin}" && [ -w "${home_bin}" ]; then
        echo "${home_bin}"
        touch "${NOT_IN_PATH}"
        return 0
    fi

    return 1
}

main() {
    PYTHON=
    if ! has_command curl || ! has_command unzip; then
        if has_command python; then
            PYTHON=python
        elif has_command python3; then
            PYTHON=python3
        else
            die "missing 'curl' or 'unzip', and neither 'python' nor 'python3' was found. Install the missing programs via your package manager."
        fi
    fi
    readonly PYTHON

    if check_optional_program sha256sum "SHA-256 verification will be skipped."; then
        has_sha256sum=0
    else
        has_sha256sum=1
    fi

    WORKDIR=$(
        set -eu
        if ! mktemp -d "${SCRIPT_NAME}.tmpd.XXXXXXXX" 2>/dev/null; then
            # `mktemp -d` may fail depending upon current $PWD
            # so try with a few common temp directories explicitly set
            for dir in /tmp /var/tmp "${HOME-}/tmp"; do
                if [ -d "${dir}" ] && [ -w "${dir}" ]; then
                    mktemp -p "${dir}" -d "${SCRIPT_NAME}.tmpd.XXXXXXXX" 2>/dev/null || continue
                    break
                fi
            done
        fi
    )
    readonly WORKDIR
    if [ ! -d "${WORKDIR}" ]; then
        die "could not create temporary working directory"
    fi
    trap 'rm -rf "$WORKDIR"' 0
    NOT_IN_PATH="${WORKDIR}/not_in_path"
    readonly NOT_IN_PATH

    info "determine platform target ..."
    # methods to get the target triple
    # 1. if rust then grep `rustc -vV` output
    # 2. run the shell script from `https://sh.rustup.rs`
    if has_command rustc && has_command grep && has_command cut; then
         target_triple=$(rustc -vV | grep -m1 -Ee '^host: ' | cut -d ' ' -f 2-)
    else
        info "run sh.rustup.rs ..."
        download_file 'https://sh.rustup.rs' "${WORKDIR}/rustup-init.sh"
        chmod +x "${WORKDIR}/rustup-init.sh"
        target_triple=$(
            env RUSTUP_INIT_SH_PRINT=arch sh "${WORKDIR}/rustup-init.sh"
        )
    fi
    if [ -r /etc/os-release ] && (set -eu; . /etc/os-release; [ "${ID-}" = 'nixos' ]); then
        echo "\
NixOS requires inserting this into /etc/nixos/configuration.nix

    programs.nix-ld.enable = true;
    programs.nix-ld.libraries = [
      pkgs.stdenv.cc.cc.lib
      pkgs.systemd
    ];
    environment.variables.S4_LIBSYSTEMD = \"\${pkgs.systemd}/lib/libsystemd.so\";

Then run

    sudo nixos-rebuild switch

then logout and login, and re-run ${SCRIPT_NAME}
"
    fi
    info "platform target is ${target_triple}"
    printf '\n' >&2

    [ -n "${target_triple}" ] || die "Platform target could not be determined."

    # override target ABI if user requested it
    if [ -n "${ABI}" ]; then
        if ! has_command rev || ! has_command cut; then
            die "Cannot override target triple with ABI '${ABI}' because required commands 'rev' or 'cut' are not available."
        fi
        target_triple="$(echo "${target_triple}" | rev | cut -f 2- -d '-' | rev)-${ABI}"
        info "overriding target triple with ABI '${ABI}'; new target triple is '${target_triple}'"
        printf '\n' >&2
    fi

    zip_name="s4_${target_triple}_v${VER}.zip"
    zip_path="${WORKDIR}/${zip_name}"
    url_zip="https://github.com/jtmoon79/super-speedy-syslog-searcher/releases/download/${VER}/${zip_name}"

    checksum_name="${zip_name}.sha256"
    checksum_path="${WORKDIR}/${checksum_name}"
    url_checksum="${url_zip}.sha256"

    if [ "${DRY_RUN-}" = 1 ]; then
        info "DRY_RUN: would download ${url_zip}, exiting..." >&2
        exit 0
    fi

    info "download release ${VER} for target ${target_triple} ..."
    download_file "${url_zip}" "${zip_path}"

    if [ "${has_sha256sum}" -eq 0 ] \
       && url_exists "${url_checksum}"; then
        info "download checksum file ..."
        download_file "${url_checksum}" "${checksum_path}"
        info "verify SHA-256 checksum of zip file ..."
        (
            cd "${WORKDIR}"
            if ! (set -x; sha256sum -c "${checksum_name}"); then
                if [ "${IGNORE_CHECKSUM-}" = "1" ]; then
                    warn "checksum verification failed for ${zip_name} from URL ${url_zip}, but IGNORE_CHECKSUM=1 is set."
                    return 0
                else
                    die "checksum verification failed for ${zip_name} from URL ${url_zip}; if you want to chance it, try running with env. var. IGNORE_CHECKSUM=1"
                fi
            fi
        )
    else
        warn "checksum file not found at ${url_checksum}"
    fi
    printf '\n' >&2

    info "extract archive ..."
    extract_zip "${zip_path}" "${WORKDIR}"

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

    chmod +x "${binary_path}"
    chmod -w "${binary_path}"

    if [ "${has_sha256sum}" -eq 0 ]; then
        binary_checksum_name="${binary_name}.sha256"
        if [ -f "${WORKDIR}/${binary_checksum_name}" ]; then
            info "verify SHA-256 checksum of binary file ..."
            (
                cd "${WORKDIR}"
                if ! (set -x; sha256sum -c "${binary_checksum_name}"); then
                    if [ "${IGNORE_CHECKSUM-}" = "1" ]; then
                        warn "checksum verification failed for ${zip_name} from URL ${url_zip}, but IGNORE_CHECKSUM=1 is set."
                        return 0
                    else
                        die "checksum verification failed for ${zip_name} from URL ${url_zip}; if you want to chance it, try running with env. var. IGNORE_CHECKSUM=1 set"
                    fi
                fi
            )
        else
            warn "checksum file '${binary_checksum_name}' was not found in archive; skipping verification."
        fi
    fi
    printf '\n' >&2

    info "check downloaded binary can run ..."
    (set -x; "${binary_path}" --version)

    install_dir=$(choose_install_dir) || die "could not find or create a writable install directory."

    install_path="${install_dir}/${binary_name}"
    info "install binary to ${install_path}"
    if [ -n "${TEST-}" ]; then
        info "TEST environment variable is set; skipping actual installation."
        return 0
    fi
    (set -x; cp -vaf "${binary_path}" "${install_path}")

    info "verify installed binary path ..."
    if ! (set -x; which -a "${binary_name}"); then
        warn "Did not find ${binary_name} in current PATH."
    fi
    printf '\n' >&2

    info "check installed binary can run ..."
    if [ -f "${NOT_IN_PATH}" ]; then
        (set -x; "${install_path}" --version)
    else
        (set -x; "${binary_name}" --version)
    fi

    info "installed ${COLOR_BOLD}${binary_name}${COLOR_BOLD_OFF} for platform ${COLOR_BOLD}${target_triple}${COLOR_BOLD_OFF} version ${COLOR_BOLD}${VER}${COLOR_BOLD_OFF} to ${COLOR_BOLD}${install_path}${COLOR_BOLD_OFF}"
    if [ -f "${NOT_IN_PATH}" ]; then
        warn "Binary was installed to a directory not currently in PATH."
    fi
}

main "$@"
