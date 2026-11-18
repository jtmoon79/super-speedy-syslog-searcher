# Performance plot information for utmp

- Hyperfine runs per data point: 5

- File: ./logs/OpenBSD7.4/x86_64/utmp
- Maximum file count: 300
- File Size: 7 KB (6992 bytes) (1 blocks)

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

- RSS/Time measurements: 00:01:49
- BlockSz measurements: 00:00:01
- Total time: 00:01:52

## Max RSS by file count

[`performance-plot__utmp__300__rss.svg`](performance-plot__utmp__300__rss.svg)
[`performance-plot__utmp__300__rss_time_data.md`](performance-plot__utmp__300__rss_time_data.md)
[`performance-plot__utmp__300__rss_time_data.csv`](performance-plot__utmp__300__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 utmp …`
- Max max RSS difference per 1 File: 608 KB (×86.9 file size) (×9.5 Blocks)
- Avg max RSS difference per 1 File: 36 KB (×5.1 file size) (×0.6 Blocks)
- Min max RSS difference per 1 File: -432 KB (×-61.7 file size) (×-6.8 Blocks)

## Time by file count

[`performance-plot__utmp__300__time.svg`](performance-plot__utmp__300__time.svg)
[`performance-plot__utmp__300__rss_time_data.md`](performance-plot__utmp__300__rss_time_data.md)
[`performance-plot__utmp__300__rss_time_data.csv`](performance-plot__utmp__300__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 utmp …`
- Time Difference per 1 File Max: 2 ms
- Time Difference per 1 File Avg: 0 ms
- Time Difference per 1 File Min: -2 ms

## Disk reads by file count

[``]()
[`performance-plot__utmp__300__diskio.md`](performance-plot__utmp__300__diskio.md)
[`performance-plot__utmp__300__diskio.csv`](performance-plot__utmp__300__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 300
- Command: `./target/release/s4 utmp …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 23 (×23.0 file blocks)
- Avg Read Count (syscr) per 1 File: 1 (×1.0 file blocks)
- Min Read Count (syscr) per 1 File: 1 (×1.0 file blocks)
- Max Read Bytes (rchar) per 1 File: 22031 (×3.2 file size) (×0.3 Blocks)
- Avg Read Bytes (rchar) per 1 File: 7306 (×1.0 file size) (×0.1 Blocks)
- Min Read Bytes (rchar) per 1 File: 7042 (×1.0 file size) (×0.1 Blocks)

## Time by block size

[`performance-plot__utmp__blocksz_8192.svg`](performance-plot__utmp__blocksz_8192.svg)
[`performance-plot__utmp__blocksz_8192.md`](performance-plot__utmp__blocksz_8192.md)
[`performance-plot__utmp__blocksz_8192.csv`](performance-plot__utmp__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 utmp …`
- Time Difference per 1 File Max: 2 ms
- Time Difference per 1 File Avg: 0 ms
- Time Difference per 1 File Min: -2 ms
