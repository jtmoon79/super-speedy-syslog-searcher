# Performance plot information for dtf5-6a.log

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/dtf5-6a.log
- Maximum file count: 300
- File Size: 1 KB (194 bytes) (1 blocks)

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

- RSS/Time measurements: 00:03:01
- BlockSz measurements: 00:00:01
- Total time: 00:03:03

## Max RSS by file count

[`performance-plot__dtf5-6a.log__300__rss.svg`](performance-plot__dtf5-6a.log__300__rss.svg)
[`performance-plot__dtf5-6a.log__300__rss_time_data.md`](performance-plot__dtf5-6a.log__300__rss_time_data.md)
[`performance-plot__dtf5-6a.log__300__rss_time_data.csv`](performance-plot__dtf5-6a.log__300__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 dtf5-6a.log …`
- Max max RSS difference per 1 File: 1296 KB (×1296.0 file size) (×20.3 Blocks)
- Avg max RSS difference per 1 File: 150 KB (×150.0 file size) (×2.3 Blocks)
- Min max RSS difference per 1 File: -544 KB (×-544.0 file size) (×-8.5 Blocks)

## Time by file count

[`performance-plot__dtf5-6a.log__300__time.svg`](performance-plot__dtf5-6a.log__300__time.svg)
[`performance-plot__dtf5-6a.log__300__rss_time_data.md`](performance-plot__dtf5-6a.log__300__rss_time_data.md)
[`performance-plot__dtf5-6a.log__300__rss_time_data.csv`](performance-plot__dtf5-6a.log__300__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 dtf5-6a.log …`
- Time Difference per 1 File Max: 26 ms
- Time Difference per 1 File Avg: 0 ms
- Time Difference per 1 File Min: -15 ms

## Disk reads by file count

[``]()
[`performance-plot__dtf5-6a.log__300__diskio.md`](performance-plot__dtf5-6a.log__300__diskio.md)
[`performance-plot__dtf5-6a.log__300__diskio.csv`](performance-plot__dtf5-6a.log__300__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 300
- Command: `./target/release/s4 dtf5-6a.log …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 23 (×23.0 file blocks)
- Avg Read Count (syscr) per 1 File: 1 (×1.0 file blocks)
- Min Read Count (syscr) per 1 File: 1 (×1.0 file blocks)
- Max Read Bytes (rchar) per 1 File: 15233 (×78.5 file size) (×0.2 Blocks)
- Avg Read Bytes (rchar) per 1 File: 508 (×2.6 file size) (×0.0 Blocks)
- Min Read Bytes (rchar) per 1 File: 244 (×1.3 file size) (×0.0 Blocks)

## Time by block size

[`performance-plot__dtf5-6a.log__blocksz_8192.svg`](performance-plot__dtf5-6a.log__blocksz_8192.svg)
[`performance-plot__dtf5-6a.log__blocksz_8192.md`](performance-plot__dtf5-6a.log__blocksz_8192.md)
[`performance-plot__dtf5-6a.log__blocksz_8192.csv`](performance-plot__dtf5-6a.log__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 dtf5-6a.log …`
- Time Difference per 1 File Max: 26 ms
- Time Difference per 1 File Avg: 0 ms
- Time Difference per 1 File Min: -15 ms
