# Performance plot information for gen-1000-3-foobar.log.bz2

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/gen-1000-3-foobar.log.bz2
- Maximum file count: 300
- File Size: 13 KB (12993 bytes) (1 blocks), Uncompressed Size: 2262 KB (2315829 bytes) (36 blocks)

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

- RSS/Time measurements: 02:06:35
- BlockSz measurements: 00:00:18
- Total time: 02:06:55

## Max RSS by file count

[`performance-plot__gen-1000-3-foobar.log.bz2__300__rss.svg`](performance-plot__gen-1000-3-foobar.log.bz2__300__rss.svg)
[`performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 gen-1000-3-foobar.log.bz2 …`
- Max max RSS difference per 1 File: 73232 KB (×32.4 file size) (×1144.3 Blocks)
- Avg max RSS difference per 1 File: 4950 KB (×2.2 file size) (×77.3 Blocks)
- Min max RSS difference per 1 File: -54768 KB (×-24.2 file size) (×-855.8 Blocks)

## Time by file count

[`performance-plot__gen-1000-3-foobar.log.bz2__300__time.svg`](performance-plot__gen-1000-3-foobar.log.bz2__300__time.svg)
[`performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.log.bz2__300__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 gen-1000-3-foobar.log.bz2 …`
- Time Difference per 1 File Max: 409 ms
- Time Difference per 1 File Avg: 27 ms
- Time Difference per 1 File Min: -413 ms

## Disk reads by file count

[``]()
[`performance-plot__gen-1000-3-foobar.log.bz2__300__diskio.md`](performance-plot__gen-1000-3-foobar.log.bz2__300__diskio.md)
[`performance-plot__gen-1000-3-foobar.log.bz2__300__diskio.csv`](performance-plot__gen-1000-3-foobar.log.bz2__300__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 300
- Command: `./target/release/s4 gen-1000-3-foobar.log.bz2 …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 56 (×1.6 file blocks)
- Avg Read Count (syscr) per 1 File: 34 (×0.9 file blocks)
- Min Read Count (syscr) per 1 File: 34 (×0.9 file blocks)
- Max Read Bytes (rchar) per 1 File: 41025 (×0.0 file size) (×0.6 Blocks)
- Avg Read Bytes (rchar) per 1 File: 26300 (×0.0 file size) (×0.4 Blocks)
- Min Read Bytes (rchar) per 1 File: 26036 (×0.0 file size) (×0.4 Blocks)

## Time by block size

[`performance-plot__gen-1000-3-foobar.log.bz2__blocksz_8192.svg`](performance-plot__gen-1000-3-foobar.log.bz2__blocksz_8192.svg)
[`performance-plot__gen-1000-3-foobar.log.bz2__blocksz_8192.md`](performance-plot__gen-1000-3-foobar.log.bz2__blocksz_8192.md)
[`performance-plot__gen-1000-3-foobar.log.bz2__blocksz_8192.csv`](performance-plot__gen-1000-3-foobar.log.bz2__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 gen-1000-3-foobar.log.bz2 …`
- Time Difference per 1 File Max: 409 ms
- Time Difference per 1 File Avg: 27 ms
- Time Difference per 1 File Min: -413 ms
