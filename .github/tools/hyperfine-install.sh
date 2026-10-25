#!/usr/bin/env bash
#
# install hyperfine

set -euo pipefail

cd /tmp

readonly HYPERFINE_INSTALL="${HYPERFINE_INSTALL:-$HOME/.local/bin/hyperfine}"
readonly HYPERFINE_INSTALL_DIR=$(dirname "${HYPERFINE_INSTALL}")

if [[ -x "${HYPERFINE_INSTALL}" ]]; then
    hv=$("${HYPERFINE_INSTALL}" --version)
    if [[ "${hv}" == 'hyperfine 2.'* ]]; then
        echo "Hyperfine already installed: ${hv}"
        exit 0
    fi
fi

readonly HYPERFINE_VERSION="${HYPERFINE_VERSION:-v2.0.0}"
readonly HYPERFINE_ARCHIVE="hyperfine-${HYPERFINE_VERSION}-x86_64-unknown-linux-musl.tar.gz"
readonly HYPERFINE_DIR="hyperfine-${HYPERFINE_VERSION}-x86_64-unknown-linux-musl"

set -x

wget \
    --no-verbose \
    --tries=3 \
    "https://github.com/sharkdp/hyperfine/releases/download/${HYPERFINE_VERSION}/${HYPERFINE_ARCHIVE}"

tar -xzvf "${HYPERFINE_ARCHIVE}"
mkdir -vp "${HYPERFINE_INSTALL_DIR}"
mv -v "${HYPERFINE_DIR}/hyperfine" "${HYPERFINE_INSTALL}"
chmod -v +x "${HYPERFINE_INSTALL}"

hv=$("${HYPERFINE_INSTALL}" --version)
echo "${hv}"
if [[ "${hv}" != 'hyperfine 2.'* ]]; then
    echo "ERROR: hyperfine 2.x required, found ${hv}" >&2
    exit 1
fi
