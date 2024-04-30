// benches/bench_syslinereader.rs

//! Benchmark functions of `crate::readers::syslinereader::SyslineReader`

use ::s4lib::common::{
    FPath,
    FileType,
    FileTypeArchive,
    FileTypeTextEncoding,
};
use ::s4lib::data::datetime::FixedOffset;
use ::s4lib::readers::blockreader::BlockSz;
use ::s4lib::readers::syslinereader::SyslineReader;

use ::criterion::{black_box, criterion_group, criterion_main, Criterion};

const BLOCKSZ: BlockSz = 0x200;

fn new_syslinereader(
    path: FPath,
    filetype: FileType,
) -> SyslineReader {
    let tz_offset: FixedOffset = FixedOffset::east_opt(0).unwrap();
    let sr = match SyslineReader::new(path.clone(), filetype, BLOCKSZ, tz_offset) {
        | Result::Ok(val) => val,
        | Result::Err(err) => {
            panic!("Sysline::new({:?}, {:?}, {:?}, {:?}) error {}", path, filetype, BLOCKSZ, tz_offset, err);
        }
    };

    sr
}

#[inline(never)]
fn syslinereader_baseline_init() {
    let path: FPath = FPath::from("./logs/other/tests/dtf2-2.log");
    let syslinereader1 = new_syslinereader(
        path,
        FileType::Text {
            archival_type: FileTypeArchive::Normal,
            encoding_type: FileTypeTextEncoding::Utf8Ascii,
        }
    );

    black_box(syslinereader1);
}

// TODO: measure various functions of `SyslineReader`

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("syslinereader");
    bg.bench_function("syslinereader_baseline_init", |b| b.iter(syslinereader_baseline_init));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
