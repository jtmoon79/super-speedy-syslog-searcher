## ETL

Input: `.\logs\programs\Event_Trace_Log\PITR\PointInTimeRestore.etl`

| Program                   | Version         | Allocator | Platform               | CPU Features                  | Opt Level | Wall Clock (ms) |
| ------------------------- | --------------- | --------- | ---------------------- | ----------------------------- | --------- | --------------: |
| s4 [mimalloc]             | 0.11.84         | mimalloc  | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    13.68 ± 0.94 |
| s4 [release]              | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    13.57 ± 0.55 |
| s4 [release_O0_cgu1]      | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    18.55 ± 2.12 |
| s4 [release_O0_cgu512_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    18.32 ± 0.85 |
| s4 [release_O1_cgu1]      | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    14.07 ± 0.67 |
| s4 [release_O1_cgu256_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    15.09 ± 3.04 |
| s4 [release_O1_cgu512_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    13.62 ± 0.45 |
| s4 [release_O2_cgu1]      | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    13.48 ± 0.60 |
| s4 [release_O2_cgu256_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    14.34 ± 1.12 |
| s4 [release_O2_cgu512_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    13.80 ± 0.68 |
| s4 [release_O3_cgu1_pa]   | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    13.82 ± 1.06 |
| s4 [release_O3_cgu256_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    13.45 ± 0.87 |
| s4 [release_O3_cgu512_pa] | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    13.53 ± 0.64 |
| s4 [release_Opt0]         | 0.11.84         | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    18.96 ± 1.37 |
| s4 [rpmalloc]             | 0.11.84         | rpmalloc  | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    14.52 ± 1.77 |
| tracerpt.exe              | 10.0.26100.1    |           | Windows                |                               |           |  120.84 ± 25.55 |
| Get-WinEvent              | 7.6.0-preview.2 |           | pwsh                   |                               |           |  454.66 ± 17.71 |

Get-WinEvent timings include PowerShell startup.
