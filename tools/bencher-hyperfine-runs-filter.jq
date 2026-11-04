#!/usr/bin/env jq -es
#
# Run with jq -es, --argjson benchmarks, and --argjson runs.
# benchmarks: [{name: "a.log", variant: "FILE_NUM", values: [5,50]}]
#
# One Hyperfine 2 input per variant family, in manifest order.
# Reports time_wall_clock in nanoseconds and memory_peak_resident in bytes.
#

def require($ok; $message):
    if $ok then . else error($message) end;

def nonnegative:
    type == "number" and isfinite and . >= 0;

require(($benchmarks | type) == "array" and ($benchmarks | length) > 0
        and all($benchmarks[];
            (.name | type) == "string" and (.name | length) > 0
            and (.variant == "FILE_NUM" or .variant == "BLOCKSZ")
            and (.values | type) == "array" and (.values | length) > 0
            and all(.values[]; nonnegative and . > 0 and . == floor)
            and (.values | unique | length) == (.values | length)
            and (.variant != "FILE_NUM" or .values == [5, 50]))
        and ($benchmarks | map([.name, .variant]) | unique | length) == ($benchmarks | length);
        "benchmark names and variants must be unique; variant values must be unique positive integers")
| require(($runs | nonnegative) and $runs >= 2 and $runs == ($runs | floor);
          "at least two runs required")
| require(length == ($benchmarks | length); "missing or extra benchmark results")
| to_entries
| reduce .[] as $input ({};
    . as $accumulated
    | $benchmarks[$input.key] as $benchmark
    | $benchmark.variant as $variant
    | $benchmark.values as $values
    | $input.value
    | require(.schema_version == 2; "expected Hyperfine schema_version 2")
    | require((.results | type) == "array"; "missing benchmark results")
    | require(all(.results[]; .name == $benchmark.name);
              "unexpected benchmark name")
    | require((.results | length) == ($values | length);
              "missing or extra variants")
    | .results
    | map(
        require((.parameters | type) == "object"
                and (.parameters | keys) == [$variant]
                and (.parameters[$variant].value | type) == "string";
                "missing or unexpected variant parameters")
        | .parameters[$variant].value as $value
        | require($value | test("^[1-9][0-9]*$"); "invalid variant parameter")
        | {result: ., value: ($value | tonumber)})
    | require((map(.value) | sort) == ($values | sort);
              "missing, duplicate, or unexpected variant value")
    | sort_by(.value)
    | reduce .[] as $result ($accumulated;
        . as $accumulator
        | $result.value as $value
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
          } + {parameters: {($variant): $value}})]
      )
  )
