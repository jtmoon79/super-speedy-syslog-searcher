# Performance plot information for gen-1000-3-foobar.UTF-32LE.log

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/gen-1000-3-foobar.UTF-32LE.log
- Maximum file count: 100
- File Size: 6135 KB (6281988 bytes) (96 blocks)

- Build profile: release
- Version: 0.11.84
- Commit: 8ce20292e705cac0c418a6ce78ff2fe489e3ef28
- Tag: 0.11.85
- MSRV: 1.99.1
- Allocator: system
- Platform: x86_64-unknown-linux-gnu
- Optimization level: 3
- Run on: Ubuntu 26.04
- CPU: AMD Ryzen 7 7700X 8-Core Processor (16 cores)
- RAM: 63873 MB

## Run durations

- RSS/Time measurements: 00:11:07
- BlockSz measurements: 00:00:17
- Total time: 00:11:25

## Max RSS by file count

[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss.svg`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss.svg)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.md`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 gen-1000-3-foobar.UTF-32LE.log …`
- Max max RSS difference per 1 File: 1036 KB (×0.2 file size) (×16.2 Blocks)
- Avg max RSS difference per 1 File: 547 KB (×0.1 file size) (×8.5 Blocks)
- Min max RSS difference per 1 File: 92 KB (×0.0 file size) (×1.4 Blocks)

## Time by file count

[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__time.svg`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__time.svg)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.md`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 gen-1000-3-foobar.UTF-32LE.log …`
- Time Difference per 1 File Max: 66 ms
- Time Difference per 1 File Avg: 22 ms
- Time Difference per 1 File Min: -8 ms

## Disk reads by file count

[``]()
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__diskio.md`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__diskio.md)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__diskio.csv`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__100__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 100
- Command: `./target/release/s4 gen-1000-3-foobar.UTF-32LE.log …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 118 (×1.2 file blocks)
- Avg Read Count (syscr) per 1 File: 96 (×1.0 file blocks)
- Min Read Count (syscr) per 1 File: 96 (×1.0 file blocks)
- Max Read Bytes (rchar) per 1 File: 6297027 (×1.0 file size) (×96.1 Blocks)
- Avg Read Bytes (rchar) per 1 File: 6282767 (×1.0 file size) (×95.9 Blocks)
- Min Read Bytes (rchar) per 1 File: 6282138 (×1.0 file size) (×95.9 Blocks)

## Time by block size

[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__blocksz_8192.svg`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__blocksz_8192.svg)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__blocksz_8192.md`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__blocksz_8192.md)
[`performance-plot__gen-1000-3-foobar.UTF-32LE.log__blocksz_8192.csv`](performance-plot__gen-1000-3-foobar.UTF-32LE.log__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 gen-1000-3-foobar.UTF-32LE.log …`
- Time Difference per 1 File Max: 66 ms
- Time Difference per 1 File Avg: 22 ms
- Time Difference per 1 File Min: -8 ms
