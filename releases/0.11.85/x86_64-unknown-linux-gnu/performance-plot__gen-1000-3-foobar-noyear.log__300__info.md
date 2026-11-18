# Performance plot information for gen-1000-3-foobar-noyear.log

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/gen-1000-3-foobar-noyear.log
- Maximum file count: 300
- File Size: 874 KB (894569 bytes) (14 blocks)

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

- RSS/Time measurements: 01:29:26
- BlockSz measurements: 00:00:15
- Total time: 01:29:42

## Max RSS by file count

[`performance-plot__gen-1000-3-foobar-noyear.log__300__rss.svg`](performance-plot__gen-1000-3-foobar-noyear.log__300__rss.svg)
[`performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 gen-1000-3-foobar-noyear.log …`
- Max max RSS difference per 1 File: 2860 KB (×3.3 file size) (×44.7 Blocks)
- Avg max RSS difference per 1 File: 2150 KB (×2.5 file size) (×33.6 Blocks)
- Min max RSS difference per 1 File: 976 KB (×1.1 file size) (×15.3 Blocks)

## Time by file count

[`performance-plot__gen-1000-3-foobar-noyear.log__300__time.svg`](performance-plot__gen-1000-3-foobar-noyear.log__300__time.svg)
[`performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar-noyear.log__300__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 gen-1000-3-foobar-noyear.log …`
- Time Difference per 1 File Max: 3047 ms
- Time Difference per 1 File Avg: 21 ms
- Time Difference per 1 File Min: -2539 ms

## Disk reads by file count

[``]()
[`performance-plot__gen-1000-3-foobar-noyear.log__300__diskio.md`](performance-plot__gen-1000-3-foobar-noyear.log__300__diskio.md)
[`performance-plot__gen-1000-3-foobar-noyear.log__300__diskio.csv`](performance-plot__gen-1000-3-foobar-noyear.log__300__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 300
- Command: `./target/release/s4 gen-1000-3-foobar-noyear.log …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 36 (×2.6 file blocks)
- Avg Read Count (syscr) per 1 File: 14 (×1.0 file blocks)
- Min Read Count (syscr) per 1 File: 14 (×1.0 file blocks)
- Max Read Bytes (rchar) per 1 File: 909608 (×1.0 file size) (×13.9 Blocks)
- Avg Read Bytes (rchar) per 1 File: 894883 (×1.0 file size) (×13.7 Blocks)
- Min Read Bytes (rchar) per 1 File: 894619 (×1.0 file size) (×13.7 Blocks)

## Time by block size

[`performance-plot__gen-1000-3-foobar-noyear.log__blocksz_8192.svg`](performance-plot__gen-1000-3-foobar-noyear.log__blocksz_8192.svg)
[`performance-plot__gen-1000-3-foobar-noyear.log__blocksz_8192.md`](performance-plot__gen-1000-3-foobar-noyear.log__blocksz_8192.md)
[`performance-plot__gen-1000-3-foobar-noyear.log__blocksz_8192.csv`](performance-plot__gen-1000-3-foobar-noyear.log__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 gen-1000-3-foobar-noyear.log …`
- Time Difference per 1 File Max: 3047 ms
- Time Difference per 1 File Avg: 21 ms
- Time Difference per 1 File Min: -2539 ms
