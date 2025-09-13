// benches/bench_prints.rs
//
// benchmark different printing approaches

#![allow(
    non_upper_case_globals,
    dead_code,
    non_snake_case
)]

use ::bstr::ByteSlice;
use ::criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};
use ::lazy_static::lazy_static;
use ::s4lib::common::NLu8;

const CHARSZ: usize = 1;

//
// test data
//

lazy_static! {
    static ref Data1: Vec<u8> = Vec::from(b"2000-01-01 00:00:00 abcdefghijklmnopqrstuvwxyz".as_slice());
}

//
// differing decoding functions
//

/// do common set of activities
#[inline(never)]
fn print_baseline() {
    black_box(0);
}

/// use one global `termcolor::ClrOut` instance
#[inline(never)]
fn print_termcolor_one() {
    unimplemented!();
}

/// create `termcolor::ClrOut` instance each loop
#[inline(never)]
fn print_termcolor_many() {
    unimplemented!();
}

/// call `stdout::write` directly
#[inline(never)]
fn print_write() {
    unimplemented!();
}

const DATA_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="OpenSSH" Guid="C4BB5D35-0136-5BC3-A262-37EF24EF9802">
    </Provider>
    <EventID>2</EventID>
    <Version>0</Version>
    <Level>2</Level>
    <Task>0</Task>
    <Opcode>0</Opcode>
    <Keywords>0x8000000000000000</Keywords>
    <TimeCreated SystemTime="2023-03-16T20:20:23.130640Z">
    </TimeCreated>
    <EventRecordID>3</EventRecordID>
    <Correlation>
    </Correlation>
    <Execution ProcessID="25223" ThreadID="30126">
    </Execution>
    <Channel>OpenSSH</Channel>
    <Computer>host1</Computer>
    <Security UserID="S-1-2-20">
    </Security>
  </System>
  <EventData>
    <Data Name="process">sshd.exe</Data>
    <Data Name="payload">error: kex_exchange_identification: Connection closed by remote host</Data>
  </EventData>
</Event>"#;

const DATA_XML_U8: &[u8] = DATA_XML.as_bytes();

/// various byte range iteration strategies
/// for `s4lib::printer::printers::print_evtx_prepend`

#[inline(never)]
fn get_byteslice_find_byte_and_repeatlast() {
    let data = DATA_XML_U8;
    let mut a: usize = 0;
    while let Some(b) = data[a..].find_byte(NLu8) {
        let line = &data[a..a + b + CHARSZ];
        a += b + CHARSZ;
        black_box(line);
    }
    // print the last line
    let line = &data[a..];
    black_box(line);
}

#[inline(never)]
fn get_byteslice_find_byte_iter() {
    let data = DATA_XML_U8;
    let mut a: usize = 0;
    for b in data[a..]
        .find_byte(NLu8)
        .iter()
    {
        let line = &data[a..a + b + CHARSZ];
        a += b + CHARSZ;
        black_box(line);
    }
    // print the last line
    let line = &data[a..];
    black_box(line);
}

//
// criterion runners
//

fn criterion_benchmark(c: &mut Criterion) {
    let mut bg = c.benchmark_group("bench_prints");
    bg.bench_function("print_baseline", |b| b.iter(print_baseline));
    bg.bench_function("get_byteslice_find_byte_and_repeatlast", |b| b.iter(get_byteslice_find_byte_and_repeatlast));
    bg.bench_function("get_byteslice_find_byte_iter", |b| b.iter(get_byteslice_find_byte_iter));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
