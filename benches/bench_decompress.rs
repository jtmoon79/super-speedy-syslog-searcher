// benches/bench_decompress.rs

//! Compare gzip decompression with `flate2` and `sirocco`.

use std::hint::black_box;
use std::io::{
    Read,
    Result,
};
use std::path::Path;

use ::criterion::{
    BenchmarkId,
    criterion_group,
    criterion_main,
    Criterion,
    Throughput,
};
use ::flate2::read::GzDecoder;
use ::sirocco::{
    inflate,
    Format,
};

const READ_SIZE: usize = 2056;
const FIXTURES: [(&str, &str); 3] = [
    ("generated_small", "logs/other/tests/gen-1000-3-foobar.log.gz"),
    ("generated_large", "logs/other/tests/gen-99999-1-Motley_Crue.log.gz"),
    ("real_small", "logs/MacOS11/install.log.0.gz"),
];

fn read_stream(mut reader: impl Read, output_size: usize) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(output_size);
    let mut buffer = [0; READ_SIZE];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        output.extend_from_slice(&buffer[..count]);
    }
    Ok(output)
}

fn read_whole(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    reader.read_to_end(&mut output)?;
    Ok(output)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut streaming = c.benchmark_group("decompress/gzip/streaming");
    for (name, path) in FIXTURES {
        let compressed = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
            .expect("failed to read gzip benchmark fixture");
        let expected = read_whole(GzDecoder::new(compressed.as_slice()))
            .expect("flate2 failed to decompress gzip fixture");
        let output_size = expected.len();
        let sirocco_output = read_whole(inflate::Reader::new(Format::Gzip, compressed.as_slice()))
            .expect("sirocco failed to decompress gzip fixture");
        assert_eq!(expected, sirocco_output, "whole-buffer output differs for {name}");
        let flate2_output = read_stream(GzDecoder::new(compressed.as_slice()), output_size)
            .expect("flate2 failed to stream gzip fixture");
        let sirocco_output = read_stream(
            inflate::Reader::new(Format::Gzip, compressed.as_slice()),
            output_size,
        )
        .expect("sirocco failed to stream gzip fixture");
        assert_eq!(expected, flate2_output, "flate2 streaming output differs for {name}");
        assert_eq!(expected, sirocco_output, "sirocco streaming output differs for {name}");

        streaming.throughput(Throughput::Bytes(output_size as u64));
        streaming.bench_with_input(
            BenchmarkId::new("flate2", name),
            &compressed,
            |b, input| b.iter(|| {
                black_box(read_stream(GzDecoder::new(black_box(input.as_slice())), output_size)
                    .expect("flate2 failed to stream gzip fixture"))
            }),
        );
        streaming.bench_with_input(
            BenchmarkId::new("sirocco", name),
            &compressed,
            |b, input| b.iter(|| {
                black_box(read_stream(
                    inflate::Reader::new(Format::Gzip, black_box(input.as_slice())),
                    output_size,
                )
                .expect("sirocco failed to stream gzip fixture"))
            }),
        );
    }
    streaming.finish();

    let mut whole = c.benchmark_group("decompress/gzip/whole_buffer");
    for (name, path) in FIXTURES {
        let compressed = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
            .expect("failed to read gzip benchmark fixture");
        let output_size = read_whole(GzDecoder::new(compressed.as_slice()))
            .expect("flate2 failed to decompress gzip fixture")
            .len();
        whole.throughput(Throughput::Bytes(output_size as u64));
        whole.bench_with_input(
            BenchmarkId::new("flate2", name),
            &compressed,
            |b, input| b.iter(|| {
                black_box(read_whole(GzDecoder::new(black_box(input.as_slice())))
                    .expect("flate2 failed to decompress gzip fixture"))
            }),
        );
        whole.bench_with_input(
            BenchmarkId::new("sirocco", name),
            &compressed,
            |b, input| b.iter(|| {
                black_box(read_whole(inflate::Reader::new(Format::Gzip, black_box(input.as_slice())))
                    .expect("sirocco failed to decompress gzip fixture"))
            }),
        );
    }
    whole.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
