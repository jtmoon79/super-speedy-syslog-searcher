# Performance plot information for gen-1000-3-foobar.log.tar

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/gen-1000-3-foobar.log.tar
- Maximum file count: 300
- File Size: 2271 KB (2324480 bytes) (36 blocks)

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

- RSS/Time measurements: 01:21:41
- BlockSz measurements: 00:00:12
- Total time: 01:21:54

## Max RSS by file count

[`performance-plot__gen-1000-3-foobar.log.tar__300__rss.svg`](performance-plot__gen-1000-3-foobar.log.tar__300__rss.svg)
[`performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 gen-1000-3-foobar.log.tar …`
- Max max RSS difference per 1 File: 1496 KB (×0.7 file size) (×23.4 Blocks)
- Avg max RSS difference per 1 File: 891 KB (×0.4 file size) (×13.9 Blocks)
- Min max RSS difference per 1 File: 268 KB (×0.1 file size) (×4.2 Blocks)

## Time by file count

[`performance-plot__gen-1000-3-foobar.log.tar__300__time.svg`](performance-plot__gen-1000-3-foobar.log.tar__300__time.svg)
[`performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.log.tar__300__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 gen-1000-3-foobar.log.tar …`
- Time Difference per 1 File Max: 58 ms
- Time Difference per 1 File Avg: 19 ms
- Time Difference per 1 File Min: -24 ms

## Disk reads by file count

[``]()
[`performance-plot__gen-1000-3-foobar.log.tar__300__diskio.md`](performance-plot__gen-1000-3-foobar.log.tar__300__diskio.md)
[`performance-plot__gen-1000-3-foobar.log.tar__300__diskio.csv`](performance-plot__gen-1000-3-foobar.log.tar__300__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 300
- Command: `./target/release/s4 gen-1000-3-foobar.log.tar …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 134 (×3.7 file blocks)
- Avg Read Count (syscr) per 1 File: 111 (×3.1 file blocks)
- Min Read Count (syscr) per 1 File: 111 (×3.1 file blocks)
- Max Read Bytes (rchar) per 1 File: 4649205 (×2.0 file size) (×70.9 Blocks)
- Avg Read Bytes (rchar) per 1 File: 4634479 (×2.0 file size) (×70.7 Blocks)
- Min Read Bytes (rchar) per 1 File: 4634215 (×2.0 file size) (×70.7 Blocks)

## Time by block size

[`performance-plot__gen-1000-3-foobar.log.tar__blocksz_8192.svg`](performance-plot__gen-1000-3-foobar.log.tar__blocksz_8192.svg)
[`performance-plot__gen-1000-3-foobar.log.tar__blocksz_8192.md`](performance-plot__gen-1000-3-foobar.log.tar__blocksz_8192.md)
[`performance-plot__gen-1000-3-foobar.log.tar__blocksz_8192.csv`](performance-plot__gen-1000-3-foobar.log.tar__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 gen-1000-3-foobar.log.tar …`
- Time Difference per 1 File Max: 58 ms
- Time Difference per 1 File Avg: 19 ms
- Time Difference per 1 File Min: -24 ms
