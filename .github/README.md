# CI Bencher reports

The separate [Bencher workflow](../.github/workflows/bencher.yml) benchmarks
`s4` against 16 log files with Hyperfine, rather than running `cargo bench`.
It builds the release binary once, then sequentially tests `S4_BLOCKSZ` values
`4096`, `8192`, `65535`, and `131071`, with two warmups and ten measured runs
per file and size.

Each Benchmark retains its filename (basename) and the existing Testbed.
Four custom Measures distinguish the block sizes:

- `time_wall_clock-blocksz-4096`
- `time_wall_clock-blocksz-8192`
- `time_wall_clock-blocksz-65535`
- `time_wall_clock-blocksz-131071`

All four use `nanoseconds (ns)`. Their Metric values are mean wall-clock
latency, with lower/upper values one standard deviation below/above the mean,
matching the previous Hyperfine adapter. The workflow converts Hyperfine 2
JSON into [Bencher Metric Format](https://bencher.dev/docs/reference/bencher-metric-format/)
and uploads one report only after every size succeeds and all results validate.
It does not upload partial results.

The four Measures must be provisioned in Bencher with these slugs and units
before running the workflow. For example:

```sh
bencher measure create "${BENCHER_PROJECT}" \
    --name time_wall_clock-blocksz-4096 --slug time_wall_clock-blocksz-4096 \
    --units 'nanoseconds (ns)'
```

Use `BENCHER_API_KEY` in the environment for authentication. Measure setup is
one-time project configuration, not part of each benchmark run.

Existing built-in `latency` history remains available but receives no new
results from this workflow. Each new Measure starts its own history;
thresholds for `latency` do not automatically apply to the new Measures.

Run the conversion self-test from the repository root:

```sh
bash .github/tools/test-hyperfine-to-bmf.sh
```

