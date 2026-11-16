#!/usr/bin/env bash
#
# run `cargo criterion` and print a markdown table of the results
#
# install with:
#    cargo install --locked cargo-criterion
#
# Requires jq.
#

set -euo pipefail

(
    set -x
    cargo criterion --version >&2
)

if [[ -z "${DIROUT:-}" ]]; then
    echo "DIROUT is not set" >&2
    exit 1
fi
mkdir -p "${DIROUT}"

(
    set -x
    cargo criterion --message-format json "$@"
) | jq -rs '
    def ns($value; $unit):
        $value * ({"ns": 1, "us": 1000, "µs": 1000, "ms": 1000000, "s": 1000000000}[$unit] // 1);
    def number($value):
        ($value * 100 | round) as $rounded
        | if $rounded == 0 then "0" else ($rounded / 100 | tostring) end;
    def time($value; $unit):
        if ["ns", "us", "µs", "ms", "s"] | index($unit) then
            ns($value; $unit) as $value_ns
            | if $value_ns >= 1000000000 then "\(number($value_ns / 1000000000)) s"
              elif $value_ns >= 1000000 then "\(number($value_ns / 1000000)) ms"
              elif $value_ns >= 1000 then "\(number($value_ns / 1000)) us"
              else "\(number($value_ns)) ns"
              end
        else "\(number($value)) \($unit)"
        end;
    def throughput($value; $duration_ns):
        if $value == null then "-"
        elif $value.unit == "bytes" or $value.unit == "byte" then
            ($value.per_iteration * 1000000000 / $duration_ns) as $bytes_per_second
            | if $bytes_per_second >= 1073741824 then "\(number($bytes_per_second / 1073741824)) GiB/s"
              elif $bytes_per_second >= 1048576 then "\(number($bytes_per_second / 1048576)) MiB/s"
              elif $bytes_per_second >= 1024 then "\(number($bytes_per_second / 1024)) KiB/s"
              else "\(number($bytes_per_second)) B/s"
              end
        else "\(number($value.per_iteration * 1000000000 / $duration_ns)) \($value.unit)/s"
        end;
    def change($value):
        if $value == null then "-"
        else
            ({"NoChange": "No change", "Improved": "Improved", "Regressed": "Regressed"}[$value.change] // $value.change) as $label
            | "\($label) (\(number($value.mean.estimate * 100))%)"
        end;
    def row:
        . as $benchmark
        | ns($benchmark.typical.estimate; $benchmark.typical.unit) as $duration_ns
        | "| "
          + ($benchmark.id | gsub("\\|"; "\\\\|"))
          + " | "
          + time($benchmark.typical.estimate; $benchmark.typical.unit)
          + " ("
          + time($benchmark.typical.lower_bound; $benchmark.typical.unit)
          + " - "
          + time($benchmark.typical.upper_bound; $benchmark.typical.unit)
          + ") | "
          + throughput($benchmark.throughput[0] // null; $duration_ns)
          + " | "
          + change($benchmark.change // null)
          + " |";

    ["| Benchmark | Typical time (95% CI) | Throughput | Change |",
     "| --- | ---: | ---: | --- |"]
    + [
        .[]
        | select(.reason == "benchmark-complete")
        | row
    ]
    | .[]
' > "${DIROUT}/benchmark.md"

./tools/glow.sh "${DIROUT}/benchmark.md"

set -x
./tools/mdtohtml.sh "${DIROUT}/benchmark.md"
