# Performance plot information for gen-1000-3-foobar.UTF-16LE_BOM.log

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/gen-1000-3-foobar.UTF-16LE_BOM.log
- Maximum file count: 300
- File Size: 3068 KB (3140996 bytes) (48 blocks)

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

- RSS/Time measurements: 01:49:51
- BlockSz measurements: 00:00:17
- Total time: 01:50:09

## Max RSS by file count

[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss.svg`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss.svg)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 gen-1000-3-foobar.UTF-16LE_BOM.log …`
- Max max RSS difference per 1 File: 1376 KB (×0.4 file size) (×21.5 Blocks)
- Avg max RSS difference per 1 File: 626 KB (×0.2 file size) (×9.8 Blocks)
- Min max RSS difference per 1 File: -48 KB (×-0.0 file size) (×-0.8 Blocks)

## Time by file count

[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__time.svg`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__time.svg)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.md`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.md)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.csv`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 gen-1000-3-foobar.UTF-16LE_BOM.log …`
- Time Difference per 1 File Max: 219 ms
- Time Difference per 1 File Avg: 26 ms
- Time Difference per 1 File Min: -119 ms

## Disk reads by file count

[``]()
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__diskio.md`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__diskio.md)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__diskio.csv`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__300__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 300
- Command: `./target/release/s4 gen-1000-3-foobar.UTF-16LE_BOM.log …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 70 (×1.5 file blocks)
- Avg Read Count (syscr) per 1 File: 48 (×1.0 file blocks)
- Min Read Count (syscr) per 1 File: 48 (×1.0 file blocks)
- Max Read Bytes (rchar) per 1 File: 3156035 (×1.0 file size) (×48.2 Blocks)
- Avg Read Bytes (rchar) per 1 File: 3141310 (×1.0 file size) (×47.9 Blocks)
- Min Read Bytes (rchar) per 1 File: 3141046 (×1.0 file size) (×47.9 Blocks)

## Time by block size

[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__blocksz_8192.svg`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__blocksz_8192.svg)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__blocksz_8192.md`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__blocksz_8192.md)
[`performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__blocksz_8192.csv`](performance-plot__gen-1000-3-foobar.UTF-16LE_BOM.log__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 gen-1000-3-foobar.UTF-16LE_BOM.log …`
- Time Difference per 1 File Max: 219 ms
- Time Difference per 1 File Avg: 26 ms
- Time Difference per 1 File Min: -119 ms
