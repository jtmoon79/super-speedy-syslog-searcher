#!/usr/bin/env bash
#
# YAML validate the codecov configuration
# See https://docs.codecov.com/docs/codecovyml-reference

set -eu

cd "$(dirname -- "${0}")/.."

set -x

exec curl -v -X POST --data-binary @./.github/codecov.yml "https://codecov.io/validate"
