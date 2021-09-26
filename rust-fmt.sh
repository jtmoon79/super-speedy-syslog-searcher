#!/usr/bin/env bash

set -x

exec rustfmt \
   -v \
   --config fn_call_width=100 \
   --config max_width=120 \
   --config newline_style=unix \
   --config edition=2021 \
   "${@}"
