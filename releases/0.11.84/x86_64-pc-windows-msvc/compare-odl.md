## ODL

Input: `C:\Users\jtmmo\Projects\super-speedy-syslog-searcher\logs\programs\OneDrive\Local\Microsoft\OneDrive\logs\Personal\SyncEngine-2025-12-21.1217.11144.2.odl`

| Program                       | Version    | Allocator | Platform               | CPU Features                  | Opt Level | Wall Clock (ms) |
| ----------------------------- | ---------- | --------- | ---------------------- | ----------------------------- | --------- | --------------: |
| s4 [mimalloc]                 | 0.11.84    | mimalloc  | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |   31.86 ± 11.33 |
| s4 [release]                  | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    27.47 ± 2.61 |
| s4 [release_O0_cgu1]          | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    53.00 ± 2.36 |
| s4 [release_O0_cgu512_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |   70.76 ± 21.32 |
| s4 [release_O1_cgu1]          | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |   34.04 ± 10.05 |
| s4 [release_O1_cgu256_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    28.75 ± 0.88 |
| s4 [release_O1_cgu512_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    30.76 ± 2.95 |
| s4 [release_O2_cgu1]          | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    26.82 ± 0.80 |
| s4 [release_O2_cgu256_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    26.98 ± 0.81 |
| s4 [release_O2_cgu512_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |   33.24 ± 13.91 |
| s4 [release_O3_cgu1_pa]       | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    26.37 ± 1.14 |
| s4 [release_O3_cgu256_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    28.97 ± 8.18 |
| s4 [release_O3_cgu512_pa]     | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    27.26 ± 1.59 |
| s4 [release_Opt0]             | 0.11.84    | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    50.43 ± 2.33 |
| s4 [rpmalloc]                 | 0.11.84    | rpmalloc  | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |   31.29 ± 11.84 |
| odl.py                        | 2.0        |           | Python 3.13.14         |                               | 2         |  243.82 ± 11.71 |
| OneDriveExplorer (parser API) | 2026.09.15 |           | Python 3.13.14         |                               | 1         |  756.27 ± 42.62 |

OneDriveExplorer uses its parser API at Python optimization level 1; the readers may decode different numbers of messages. OneDriveExplorer ran without cstructs.
