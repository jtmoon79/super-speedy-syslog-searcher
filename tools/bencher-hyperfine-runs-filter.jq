#!/usr/bin/env jq -es
#
# Run with jq -es, --argjson benchmarks, and --argjson runs.
# benchmarks: [{name: "a.log", block_sizes: [4096,8192]}, {name: "b.evtx", block_sizes: []}]
#
# One Hyperfine 2 input per benchmark, in manifest order.
# Reports time_wall_clock in nanoseconds and memory_peak_resident in bytes.
#

def require($ok; $message):
    if $ok then . else error($message) end;

def nonnegative:
    type == "number" and isfinite and . >= 0;

require(($benchmarks | type) == "array" and ($benchmarks | length) > 0
        and all($benchmarks[];
            (.name | type) == "string" and (.name | length) > 0
            and (.block_sizes | type) == "array"
            and all(.block_sizes[]; nonnegative and . > 0 and . == floor)
            and (.block_sizes | unique | length) == (.block_sizes | length))
        and ($benchmarks | map(.name) | unique | length) == ($benchmarks | length);
        "benchmark names must be unique; block sizes must be unique positive integers")
| require(($runs | nonnegative) and $runs >= 2 and $runs == ($runs | floor);
          "at least two runs required")
| require(length == ($benchmarks | length); "missing or extra benchmark results")
| to_entries
| reduce .[] as $input ({};
    . as $accumulated
    | $benchmarks[$input.key] as $benchmark
    | $benchmark.block_sizes as $sizes
    | $input.value
    | require(.schema_version == 2; "expected Hyperfine schema_version 2")
    | require((.results | type) == "array"; "missing benchmark results")
    | require(all(.results[]; .name == $benchmark.name);
              "unexpected benchmark name")
    | require((.results | length) == (if ($sizes | length) == 0 then 1 else $sizes | length end);
              "missing or extra variants")
    | .results
    | map(
        if ($sizes | length) == 0 then
            require(.parameters == null or .parameters == {}; "unexpected parameters")
            | {result: ., block_size: null}
        else
            require((.parameters | type) == "object"
                    and (.parameters | keys) == ["S4_BLOCKSZ"]
                    and (.parameters.S4_BLOCKSZ.value | type) == "string";
                    "missing or unexpected variant parameters")
            | .parameters.S4_BLOCKSZ.value as $value
            | require($value | test("^[1-9][0-9]*$"); "invalid block size parameter")
            | {result: ., block_size: ($value | tonumber)}
        end)
    | require((map(.block_size) | sort) ==
              (if ($sizes | length) == 0 then [null] else $sizes | sort end);
              "missing, duplicate, or unexpected block-size variant")
    | sort_by(if .block_size == null then 0 else .block_size as $size | $sizes | index($size) end)
    | reduce .[] as $result ($accumulated;
        . as $accumulator
        | $result.block_size as $block_size
        | $result.result
        | .name as $name
        | require((.measurements | type) == "array"
                  and (.measurements | length) == $runs;
                  "\($name): incomplete measurements")
        | require(all(.measurements[];
                      .exit_code == 0
                      and .time_wall_clock.unit == "second"
                      and (.time_wall_clock.value | nonnegative)
                      and .memory_peak_resident.unit == "byte"
                      and (.memory_peak_resident.value | nonnegative));
                  "\($name): failed sample or invalid wall-clock/memory measurement")
        | .summary.memory_peak_resident as $memory
        | require($memory.unit == "byte" and $memory.count == $runs
                  and all($memory.mean, $memory.min, $memory.max; nonnegative);
                  "\($name): invalid peak resident memory summary")
        | require($memory.min <= $memory.mean and $memory.mean <= $memory.max;
                  "\($name): inconsistent peak resident memory summary")
        | .summary.time_wall_clock
        | require(.unit == "second" and .count == $runs
                  and all(.mean, .min, .max; nonnegative);
                  "\($name): invalid wall-clock summary")
        | require(.min <= .mean and .mean <= .max;
                  "\($name): inconsistent wall-clock summary")
        | . as $summary
        | $accumulator
        | .[$name] += [({
            measures: {
                "time_wall_clock": {
                    average: ($summary.mean * 1000000000)
                },
                "memory_peak_resident": {
                    average: $memory.mean
                }
            }
          } + if $block_size == null then {} else {parameters: {S4_BLOCKSZ: $block_size}} end)]
      )
  )
