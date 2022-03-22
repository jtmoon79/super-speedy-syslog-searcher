#!/usr/bin/env bash

set -eux

exec head -q "${1}" > "./tmp/$(basename -- "${1}")"

