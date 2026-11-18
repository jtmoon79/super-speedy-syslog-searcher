# Performance plot information for gen-99999-1-Motley_Crue.log

- Hyperfine runs per data point: 5

- File: ./logs/other/tests/gen-99999-1-Motley_Crue.log
- Maximum file count: 100
- File Size: 22168 KB (22700000 bytes) (347 blocks)

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

- RSS/Time measurements: 08:28:12
- BlockSz measurements: 00:14:11
- Total time: 08:42:24

## Max RSS by file count

[`performance-plot__gen-99999-1-Motley_Crue.log__100__rss.svg`](performance-plot__gen-99999-1-Motley_Crue.log__100__rss.svg)
[`performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.md`](performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.md)
[`performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.csv`](performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.csv)

- Plot: Max RSS (KB) per N file
- Command: `./target/release/s4 gen-99999-1-Motley_Crue.log …`
- Max max RSS difference per 1 File: 1212 KB (×0.1 file size) (×18.9 Blocks)
- Avg max RSS difference per 1 File: 688 KB (×0.0 file size) (×10.8 Blocks)
- Min max RSS difference per 1 File: 244 KB (×0.0 file size) (×3.8 Blocks)

## Time by file count

[`performance-plot__gen-99999-1-Motley_Crue.log__100__time.svg`](performance-plot__gen-99999-1-Motley_Crue.log__100__time.svg)
[`performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.md`](performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.md)
[`performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.csv`](performance-plot__gen-99999-1-Motley_Crue.log__100__rss_time_data.csv)

- Plot: Time (ms) mean per N file
- Command: `./target/release/s4 gen-99999-1-Motley_Crue.log …`
- Time Difference per 1 File Max: 11368 ms
- Time Difference per 1 File Avg: 1165 ms
- Time Difference per 1 File Min: -8028 ms

## Disk reads by file count

[``]()
[`performance-plot__gen-99999-1-Motley_Crue.log__100__diskio.md`](performance-plot__gen-99999-1-Motley_Crue.log__100__diskio.md)
[`performance-plot__gen-99999-1-Motley_Crue.log__100__diskio.csv`](performance-plot__gen-99999-1-Motley_Crue.log__100__diskio.csv)

- Plot: Disk reads (syscr, rchar) per N file
- Maximum file count: 100
- Command: `./target/release/s4 gen-99999-1-Motley_Crue.log …`
- Disk I/O from `/proc/<pid>/io` (one run per file count)
- Page cache: dropped when permitted (at least one fnum)
- Max Read Count (syscr) per 1 File: 370 (×1.1 file blocks)
- Avg Read Count (syscr) per 1 File: 347 (×1.0 file blocks)
- Min Read Count (syscr) per 1 File: 347 (×1.0 file blocks)
- Max Read Bytes (rchar) per 1 File: 22715040 (×1.0 file size) (×346.6 Blocks)
- Avg Read Bytes (rchar) per 1 File: 22700779 (×1.0 file size) (×346.4 Blocks)
- Min Read Bytes (rchar) per 1 File: 22700150 (×1.0 file size) (×346.4 Blocks)

## Time by block size

[`performance-plot__gen-99999-1-Motley_Crue.log__blocksz_8192.svg`](performance-plot__gen-99999-1-Motley_Crue.log__blocksz_8192.svg)
[`performance-plot__gen-99999-1-Motley_Crue.log__blocksz_8192.md`](performance-plot__gen-99999-1-Motley_Crue.log__blocksz_8192.md)
[`performance-plot__gen-99999-1-Motley_Crue.log__blocksz_8192.csv`](performance-plot__gen-99999-1-Motley_Crue.log__blocksz_8192.csv)

- Plot: Time (ms) mean per 8192 BlockSz step for 10 runs
- Command: `65535 ./target/release/s4 gen-99999-1-Motley_Crue.log …`
- Time Difference per 1 File Max: 11368 ms
- Time Difference per 1 File Avg: 1165 ms
- Time Difference per 1 File Min: -8028 ms
