// bench_syslinereader.rs
//
// benchmark functions of `crate::Readers::syslinereader::SyslineReader`
//

extern crate s4lib;

use s4lib::common::{
    FPath,
    FileType,
};

use s4lib::Data::datetime::{
    FixedOffset,
};

use s4lib::Readers::blockreader::{
    BlockSz,
};

use s4lib::Readers::syslinereader::SyslineReader;

use s4lib::Readers::filepreprocessor::{
    ProcessPathResult,
    ProcessPathResults,
    process_path,
};

extern crate criterion;
use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const BLOCKSZ: BlockSz = 0x200;


fn new_SyslineReader(path: FPath, filetype: FileType) -> SyslineReader {
    let tz_offset: FixedOffset = FixedOffset::east(0);
    
    let syslinereader1 = match SyslineReader::new(path.clone(), filetype, BLOCKSZ, tz_offset) {
        Result::Ok(val) => val,
        Result::Err(err) => {
            panic!("Sysline::new({:?}, {:?}, {:?}, {:?}) error {}", path, filetype, BLOCKSZ, tz_offset, err);
        }
    };

    syslinereader1
}

#[inline(never)]
fn syslinereader_baseline_init() {
    let path: FPath = FPath::from("./logs/other/tests/dtf2-2.log");
    let syslinereader1 = new_SyslineReader(path, FileType::File);

    black_box(syslinereader1);
}

// TODO: measure various functions of `SyslineReader`

// criterion runners

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("SyslineReader");
    bg.bench_function("syslinereader_baseline_init", |b| b.iter(syslinereader_baseline_init));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
