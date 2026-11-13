#!/bin/sh
#
# easy installer for s4 release binaries.
# POSIX shell compatible.
#
# Uses curl/wget and unzip if available, fallback to python/python3, then perl.
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

VER=${VER-0.11.85}  # PROJECT VERSION LAST PUBLISHED
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
DOWNLOADER=
EXTRACT=
CHECKSUM=

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

has_command() {
    command -v "$1" >/dev/null 2>&1
}

# find and set a HTTPS downloader program for fetching files
# for python and perl, verify they can load necessary modules
set_downloader() {
    DOWNLOADER=
    for downloader_program in curl wget python python3 perl; do
        has_command "${downloader_program}" || continue
        case "${downloader_program}" in
            python|python3)
                if ! "${downloader_program}" -c '
import shutil
import ssl
import sys
try:
    from urllib.request import Request, urlopen, HTTPSHandler
    from urllib.error import HTTPError, URLError
except ImportError:
    from urllib2 import Request, urlopen, HTTPSHandler, HTTPError, URLError
ssl.create_default_context()
' >/dev/null 2>&1
                then
                    printf '%s\n' "${downloader_program} download fallback requires shutil, ssl with HTTPS support, and urllib.request/urllib2. Install these Python modules or use curl, wget, or Perl." >&2
                    continue
                fi
                ;;
            perl)
                if ! perl -MHTTP::Tiny -e '
my ($ok, $reason) = HTTP::Tiny->can_ssl();
die "$reason\n" unless $ok;
' >/dev/null 2>&1; then
                    printf '%s\n' "Perl download fallback requires HTTP::Tiny with HTTPS support (IO::Socket::SSL and Net::SSLeay). Install these Perl modules or use curl, wget, or Python." >&2
                    continue
                fi
                ;;
        esac
        DOWNLOADER=${downloader_program}
        return 0
    done
    return 1
}

# find and set an extraction program for ZIP files
# for python and perl, verify they can load necessary modules
set_extract() {
    EXTRACT=
    for extract_program in unzip python python3 perl; do
        has_command "${extract_program}" || continue
        case "${extract_program}" in
            python|python3)
                if ! "${extract_program}" -c 'import sys, zipfile, zlib' >/dev/null 2>&1; then
                    printf '%s\n' "${extract_program} ZIP extraction fallback requires zipfile and zlib. Install these Python modules or use unzip or Perl." >&2
                    continue
                fi
                ;;
            perl)
                if ! perl -MIO::Uncompress::Unzip='$UnzipError' -MFile::Path=make_path -MFile::Basename=dirname -e '0' >/dev/null 2>&1; then
                    printf '%s\n' "Perl ZIP extraction fallback requires IO::Uncompress::Unzip, File::Path (make_path), and File::Basename (dirname). Install these Perl modules or use unzip or Python." >&2
                    continue
                fi
                ;;
        esac
        EXTRACT=${extract_program}
        return 0
    done
    return 1
}

# find and set a checksum program for SHA-256 verification
# for python and perl, verify they can load necessary modules
set_checksum() {
    CHECKSUM=
    for checksum_program in sha256sum cksum shasum python python3 perl; do
        has_command "${checksum_program}" || continue
        case "${checksum_program}" in
            cksum)
                printf '%s  /dev/null\n' 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855' \
                    | cksum -a sha256 -c >/dev/null 2>&1 || continue
                ;;
            shasum)
                shasum -a 256 /dev/null >/dev/null 2>&1 || continue
                ;;
            python|python3)
                if ! "${checksum_program}" -c 'import hashlib, re, sys; hashlib.sha256()' >/dev/null 2>&1; then
                    printf '%s\n' "${checksum_program} SHA-256 fallback requires hashlib with SHA-256 support and re. Install these Python modules or use a native checksum program or Perl." >&2
                    continue
                fi
                ;;
            perl)
                if ! perl -MDigest::SHA=sha256_hex -e 'sha256_hex("")' >/dev/null 2>&1; then
                    printf '%s\n' "Perl SHA-256 fallback requires Digest::SHA. Install this Perl module or use a native checksum program or Python." >&2
                    continue
                fi
                ;;
        esac
        CHECKSUM=${checksum_program}
        return 0
    done
    return 1
}

# run the checksum program on a given manifest file $1
checksum() {
    case "${CHECKSUM}" in
        '') return 0 ;;
        sha256sum) (set -eux; sha256sum -c "$1" ) ;;
        cksum) (set -eux; cksum -a sha256 -c "$1" ) ;;
        shasum) (set -eux; shasum -a 256 -c "$1" ) ;;
        python|python3)
            printf "Running checksum with %s on %s\n" "${CHECKSUM}" "$1" >&2
            "${CHECKSUM}" -c '
import hashlib
import re
import sys

failed = False
entries = 0
with open(sys.argv[1], "rb") as manifest:
    for line_number, line in enumerate(manifest, 1):
        match = re.match(br"\A([0-9a-fA-F]{64}) [ *](.+)\Z", line.rstrip(b"\r\n"))
        if match is None:
            sys.stderr.write("%s:%d: malformed SHA-256 checksum line\n" %
                             (sys.argv[1], line_number))
            sys.exit(1)
        expected, path = match.groups()
        digest = hashlib.sha256()
        with open(path, "rb") as source:
            for block in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(block)
        matches = digest.hexdigest().encode("ascii") == expected.lower()
        sys.stdout.write("%s: %s\n" %
                         (path.decode("utf-8", "replace"), "OK" if matches else "FAILED"))
        failed = failed or not matches
        entries += 1
if not entries:
    sys.stderr.write("%s: no SHA-256 checksums found\n" % sys.argv[1])
sys.exit(1 if failed or not entries else 0)
' "$1"
            ;;
        perl)
            printf "Running checksum with %s on %s\n" "${CHECKSUM}" "$1" >&2
            perl -MDigest::SHA -e '
my $manifest = $ARGV[0];
open my $checksums, "<", $manifest or die "Cannot open $manifest: $!\n";
my ($entries, $failed) = (0, 0);
while (my $line = <$checksums>) {
    $line =~ s/\r?\n\z//;
    my ($expected, $path) = $line =~ /\A([0-9a-fA-F]{64}) [ *](.+)\z/
        or die "$manifest:$.: malformed SHA-256 checksum line\n";
    open my $source, "<", $path or die "Cannot open $path: $!\n";
    binmode $source or die "Cannot set binary mode for $path: $!\n";
    my $actual = Digest::SHA->new(256)->addfile($source)->hexdigest();
    close $source or die "Cannot close $path: $!\n";
    my $matches = $actual eq lc($expected);
    print "$path: ", ($matches ? "OK" : "FAILED"), "\n";
    $failed ||= !$matches;
    $entries++;
}
close $checksums or die "Cannot close $manifest: $!\n";
die "$manifest: no SHA-256 checksums found\n" unless $entries;
exit($failed ? 1 : 0);
' "$1"
            ;;
        *) die "Unknown checksum program '${CHECKSUM}'" ;;
    esac
}

# download the file from URL $1 to destination $2 using the `DOWNLOADER` program
download() {
    case "${DOWNLOADER}" in
        curl) (set -eux; curl --silent --show-error --fail --location --output "$2" "$1") ;;
        wget) (set -eux; wget --quiet --output-document="$2" -- "$1") ;;
        python|python3)
            printf "Downloading %s to %s using %s\n" "$1" "$2" "${DOWNLOADER}" >&2
            "${DOWNLOADER}" -c '
import shutil
import sys
try:
    from urllib.request import urlopen
except ImportError:
    from urllib2 import urlopen
response = urlopen(sys.argv[1])
try:
    with open(sys.argv[2], "wb") as output:
        shutil.copyfileobj(response, output)
finally:
    response.close()
' "$1" "$2"
            ;;
        perl)
            printf "Downloading %s to %s using %s\n" "$1" "$2" "${DOWNLOADER}" >&2
            perl -MHTTP::Tiny -e '
my ($url, $path) = @ARGV;
my $response = HTTP::Tiny->new(verify_SSL => 1)->mirror($url, $path);
die "$url: $response->{status} $response->{reason}\n"
    unless $response->{success};
' "$1" "$2"
            ;;
        *) die "No suitable downloader selected" ;;
    esac
}

# check if the URL $1 exists using the `DOWNLOADER` program
url_exists() {
    case "${DOWNLOADER}" in
        curl) (set -eux; curl --head --fail --location --silent --output /dev/null "$1") ;;
        wget) (set -eux; wget --quiet --spider -- "$1") ;;
        python|python3)
            "${DOWNLOADER}" -c '
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
            ;;
        perl)
            printf "Checking URL existence for %s using %s\n" "$1" "${DOWNLOADER}" >&2
            perl -MHTTP::Tiny -e '
my $response = HTTP::Tiny->new(verify_SSL => 1)->head($ARGV[0]);
die "$ARGV[0]: $response->{status} $response->{reason}\n"
    unless $response->{success};
' "$1"
            ;;
        *) die "No suitable downloader selected" ;;
    esac
}

# extract the ZIP file $1 to destination $2 using the `EXTRACT` program
extract() {
    case "${EXTRACT}" in
        unzip) unzip -o -d "$2" "$1" ;;
        python|python3)
            "${EXTRACT}" -c '
import sys
import zipfile
with zipfile.ZipFile(sys.argv[1]) as archive:
    archive.extractall(sys.argv[2])
' "$1" "$2"
            ;;
        perl)
            perl -MIO::Uncompress::Unzip='$UnzipError' -MFile::Path=make_path -MFile::Basename=dirname -e '
my ($path, $destination) = @ARGV;
my $archive = IO::Uncompress::Unzip->new($path, Strict => 1)
    or die "Cannot open $path: $UnzipError\n";
my $status;
do {
    my $name = $archive->getHeaderInfo()->{Name};
    die "Unsafe ZIP entry: $name\n"
        if $name =~ m{\A/|\\|\A[A-Za-z]:|(?:\A|/)\.\.(?:/|\z)};
    my $output_path = "$destination/$name";
    if ($name =~ m{/\z}) {
        make_path($output_path);
    } else {
        make_path(dirname($output_path));
        open my $output, ">", $output_path or die "Cannot open $output_path: $!\n";
        binmode $output or die "Cannot set binary mode for $output_path: $!\n";
        my $read;
        while (($read = $archive->read(my $buffer)) > 0) {
            print {$output} $buffer or die "Cannot write $output_path: $!\n";
        }
        die "Cannot extract $name: $UnzipError\n" if $read < 0;
        close $output or die "Cannot close $output_path: $!\n";
    }
    $status = $archive->nextStream();
} while ($status > 0);
die "Cannot read $path: $UnzipError\n" if $status < 0;
$archive->close() or die "Cannot close $path: $UnzipError\n";
' "$1" "$2"
            ;;
        *) die "No suitable ZIP extractor selected" ;;
    esac
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

# print a suitable installation directory
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
        printf "%s" "${INSTALL_DIR}"
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
        printf "%s" "${dir}"
        return 0
    done

    # now just search PATH for a writable directory; hopefully the user is okay with this
    IFS_OLD=${IFS}
    IFS=:
    for dir in ${PATH}; do
        [ -n "${dir}" ] || continue
        [ -d "${dir}" ] || continue
        [ -w "${dir}" ] || continue
        printf "%s" "${dir}"
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
        printf "%s" "${home_bin}"
        touch "${NOT_IN_PATH}"
        return 0
    fi

    return 1
}

set_downloader || die "No suitable downloader found. Install 'curl', 'wget', 'python'/'python3' with HTTPS modules, or 'perl' with HTTP::Tiny and HTTPS support."
set_extract || die "No suitable ZIP extractor found. Install 'unzip', 'python'/'python3' with zipfile and zlib, or 'perl' with IO::Uncompress::Unzip."
if ! set_checksum; then
    warn "No SHA-256 checksum program found; verification will be skipped. Consider installing 'sha256sum', SHA-256-capable 'cksum', 'shasum', 'python'/'python3' with hashlib, or 'perl' with Digest::SHA."
fi

main() {
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
    # flag file to indicate installation directory is not in PATH
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
        download 'https://sh.rustup.rs' "${WORKDIR}/rustup-init.sh"
        chmod +x "${WORKDIR}/rustup-init.sh"
        target_triple=$(
            env RUSTUP_INIT_SH_PRINT=arch sh "${WORKDIR}/rustup-init.sh"
        )
    fi
    if [ -r /etc/os-release ] && (set -eu; . /etc/os-release; [ "${ID-}" = 'nixos' ]); then
        printf "%s" "\
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
    download "${url_zip}" "${zip_path}"

    if [ -n "${CHECKSUM}" ] && url_exists "${url_checksum}"; then
        info "download checksum file ..."
        download "${url_checksum}" "${checksum_path}"
        info "verify SHA-256 checksum of zip file ..."
        (
            cd "${WORKDIR}"
            if ! checksum "${checksum_name}"; then
                if [ "${IGNORE_CHECKSUM-}" = "1" ]; then
                    warn "checksum verification failed for ${zip_name} from URL ${url_zip}, but IGNORE_CHECKSUM=1 is set."
                    return 0
                else
                    die "checksum verification failed for ${zip_name} from URL ${url_zip}; if you want to chance it, try running with env. var. IGNORE_CHECKSUM=1"
                fi
            fi
        )
    elif [ -n "${CHECKSUM}" ]; then
        warn "checksum file not found at ${url_checksum}"
    fi
    printf '\n' >&2

    info "extract archive ..."
    extract "${zip_path}" "${WORKDIR}"

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

    if [ -n "${CHECKSUM}" ]; then
        binary_checksum_name="${binary_name}.sha256"
        if [ -f "${WORKDIR}/${binary_checksum_name}" ]; then
            info "verify SHA-256 checksum of binary file ..."
            (
                cd "${WORKDIR}"
                if ! (set -x; checksum "${binary_checksum_name}"); then
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
