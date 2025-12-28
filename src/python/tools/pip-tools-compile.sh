#!/usr/bin/env bash
#
# helper script to run pip-compile to refresh requirements.txt
# extra arguments passed to pip-compile
# install `requirements-dev.in` prior to have piptools available
#

# must match pyprojec.toml:requires-python
readonly PYTHON_VERSION_REQUIRED="3.9"

set -eu

function version_major_minor_eq() {
    # return 0 if major.minor parts of two versions are equal, 1 otherwise
    python -c "
import sys
from packaging.version import Version

v1 = Version('$1')
v2 = Version('$2')
result = (v1.major == v2.major and v1.minor == v2.minor)
sys.exit(0 if result else 1)
"
}

function print_python_version() {
    python -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")'
}

version=$(print_python_version)

if ! version_major_minor_eq "$version" "$PYTHON_VERSION_REQUIRED"; then
    echo "ERROR: this script must be run with Python $PYTHON_VERSION_REQUIRED (current: $version)" >&2
    exit 1
fi

cd "$(dirname "$0")/../s4_event_readers"

for outfile_infile in \
    "requirements.txt|requirements.in" \
    `#"requirements-3.9.txt|requirements-3.9.in"` \
    "requirements-dev.txt|requirements-dev.in"
do
    outfile=$(echo -n "$outfile_infile" | cut -d'|' -f1)
    rm -f "$outfile"
    infile=$(echo -n "$outfile_infile" | cut -d'|' -f2)
    (
        set -x
        # passing `--generate-hashes` here causes pip-compile to fail
        # see https://github.com/jazzband/pip-tools/issues/2299
        python -m piptools compile \
            --strip-extras \
            --annotate \
            --emit-find-links \
            --output-file="$outfile" \
            "$infile" \
            "${@}"
    )
done
