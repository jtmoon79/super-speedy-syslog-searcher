Path `./benches` are benchmark tests for `cargo bench`.

To run:

    cargo bench

or

    cargo bench -- name

Each `BenchmarkGroup` value `name` is set in the creation function `benchmark_group`.

Each `bench*.rs` file must defined in `Cargo.toml`.
