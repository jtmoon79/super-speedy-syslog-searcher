#!/usr/bin/env bash

set -x

exec rustfmt \
   -v \
   --config fn_call_width=120 \
   --config fn_args_layout=Compressed \
   --config max_width=120 \
   --config newline_style=unix \
   --config edition=2021 \
   "${@}"
