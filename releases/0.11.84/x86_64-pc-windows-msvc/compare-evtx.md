## EVTX

Input: `C:\Users\jtmmo\Projects\super-speedy-syslog-searcher\logs\programs\evtx\Microsoft-Windows-Kernel-PnP__Configuration.evtx`

| Program                   | Version   | Allocator | Platform               | CPU Features                  | Opt Level | Wall Clock (ms) |
| ------------------------- | --------- | --------- | ---------------------- | ----------------------------- | --------- | --------------: |
| s4 [mimalloc]             | 0.11.84   | mimalloc  | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    11.48 ± 0.61 |
| s4 [release]              | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    10.25 ± 0.48 |
| s4 [release_O0_cgu1]      | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    15.41 ± 1.30 |
| s4 [release_O0_cgu512_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    15.02 ± 0.85 |
| s4 [release_O1_cgu1]      | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    10.77 ± 0.45 |
| s4 [release_O1_cgu256_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    10.95 ± 0.44 |
| s4 [release_O1_cgu512_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 1         |    10.85 ± 0.49 |
| s4 [release_O2_cgu1]      | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    10.65 ± 0.42 |
| s4 [release_O2_cgu256_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    11.46 ± 0.75 |
| s4 [release_O2_cgu512_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 2         |    10.72 ± 0.58 |
| s4 [release_O3_cgu1_pa]   | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    10.56 ± 0.65 |
| s4 [release_O3_cgu256_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    11.14 ± 3.51 |
| s4 [release_O3_cgu512_pa] | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    10.76 ± 0.57 |
| s4 [release_Opt0]         | 0.11.84   | system    | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 0         |    14.92 ± 0.41 |
| s4 [rpmalloc]             | 0.11.84   | rpmalloc  | x86_64-pc-windows-msvc | cmpxchg16b,fxsr,sse,sse2,sse3 | 3         |    13.26 ± 0.92 |
| evtx_dump.py              | 0.7.4     |           | Python 3.13.14         |                               | 2         |  549.29 ± 28.78 |
| evtx_dump (Rust)          | 0.15.1    |           | Windows                |                               |           |     6.97 ± 0.48 |
| EvtxECmd                  | 2026.10.2 |           | .NET                   |                               |           |  478.86 ± 15.48 |
