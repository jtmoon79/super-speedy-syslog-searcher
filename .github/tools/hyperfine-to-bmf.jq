# Run with jq -es, --argjson block_sizes, --argjson names, and --argjson runs.
# Input files must be in block_sizes order.
def require($ok; $message):
    if $ok then . else error($message) end;

def nonnegative:
    type == "number" and isfinite and . >= 0;

require(($block_sizes | length) > 0
        and all($block_sizes[]; nonnegative and . > 0 and . == floor)
        and ($block_sizes | unique | length) == ($block_sizes | length);
        "block sizes must be nonempty, unique positive integers")
| require(($names | length) > 0 and ($names | unique | length) == ($names | length);
          "benchmark names must be nonempty and unique")
| require($runs >= 2 and $runs == ($runs | floor); "at least two runs required")
| require(length == ($block_sizes | length); "missing or extra block-size results")
| to_entries
| reduce .[] as $input ({};
    . as $benchmarks
    | $block_sizes[$input.key] as $block_size
    | $input.value
    | require(.schema_version == 2; "expected Hyperfine schema_version 2")
    | require((.results | type) == "array"; "missing benchmark results")
    | require((.results | map(.name) | sort) == ($names | sort);
              "missing, duplicate, or unexpected benchmark names")
    | .results
    | reduce .[] as $result ($benchmarks;
        . as $accumulator
        | $result.name as $name
        | $result
        | require((.measurements | type) == "array"
                  and (.measurements | length) == $runs;
                  "\($name): incomplete measurements")
        | require(all(.measurements[];
                      .exit_code == 0
                      and .time_wall_clock.unit == "second"
                      and (.time_wall_clock.value | nonnegative));
                  "\($name): failed sample or invalid wall-clock measurement")
        | .summary.time_wall_clock
        | require(.unit == "second" and .count == $runs
                  and all(.mean, .min, .max; nonnegative);
                  "\($name): invalid wall-clock summary")
        | require(.min <= .mean and .mean <= .max;
                  "\($name): inconsistent wall-clock summary")
        | . as $summary
        | $accumulator
        | .[$name] += [{
            parameters: {S4_BLOCKSZ: $block_size},
            measures: {
                "time_wall_clock": {
                    average: ($summary.mean * 1e9)
                }
            }
          }]
      )
  )
