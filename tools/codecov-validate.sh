#!/usr/bin/env bash
#
# YAML validate the codecov configuration
# See https://docs.codecov.com/docs/codecovyml-reference

set -eu

HOST_VAL="codecov.io"
URL_VAL="https://${HOST_VAL}/validate"

cd "$(dirname -- "${0}")/.."

if which dig &>/dev/null; then
    dig +short "${HOST_VAL}" &>/dev/null || {
        echo "ERROR: cannot DNS resolve '${HOST_VAL}'; skip YAML validation and exit 0" >&2
        exit 0
    }
fi

(set -x; curl --version)
echo

set -x

exec curl -X POST --data-binary "@./.github/codecov.yml" "${URL_VAL}" "${@}"
