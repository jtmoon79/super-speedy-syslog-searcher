# `FileCoAuth-2025-12-21.1216.11020.2.odl`

## `--version`

```text
s4 (Super Speedy Syslog Searcher)
Version: 0.9.82
MSRV: 1.88.0
Profile: alloc_tracker
Allocator: alloc_tracker
Features: alloc_tracker
Platform: x86_64-unknown-linux-gnu
Target OS: linux
Target OS Family: unix
Arch: x86_64
Compiled Regular Expressions: 188
Compiler Version: 1.88.0
Rust Build Flags: 
Optimization Level: 0
Git Commit: 4d4a4cc9f70f8f1548d32fd3660414050e3d6b0e
Build Date: 2026-07-03T16:43:55
License: MIT
Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
Author: James Thomas Moon

```

## Command

`$ ./target/alloc_tracker/s4 ./logs/programs/OneDrive/Local/Microsoft/OneDrive/logs/Common/FileCoAuth-2025-12-21.1216.11020.2.odl`

## Allocator Tracking results

| ***File:line:col***<br/>***Call Site*** | Thread<br/>ID | Thread<br/>Name | Allocations | Bytes | Bytes<br/>per Allocation |
| :-- | ---: | :--- | ---: | ---: | ---: |
| `src/s4/s4.rs:3466:16`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 204 | 25,376 (24.78 KiB) | 124 (124 B) |
| `src/python/pyrunner.rs:1104:23`<br/>`s4lib::python::pyrunner::PyRunner::write_read::hbe080931cac19bbf` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 176 | 22,528 (22.00 KiB) | 128 (128 B) |
| `src/python/pyrunner.rs:1225:63`<br/>`s4lib::python::pyrunner::PyRunner::write_read::hbe080931cac19bbf` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 103 | 63,506 (62.02 KiB) | 616 (616 B) |
| `src/s4/s4.rs:5406:53`<br/>`s4::s4::processing_loop::recv_many_chan::hf2b1c552a15b43da` | 1 | `main` | 73 | 9,344 (9.12 KiB) | 128 (128 B) |
| `src/python/pyrunner.rs:1170:60`<br/>`s4lib::python::pyrunner::PyRunner::write_read::hbe080931cac19bbf` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 35,693 (34.86 KiB) | 502 (502 B) |
| `src/readers/pyeventreader.rs:561:13`<br/>`s4lib::readers::pyeventreader::PyEventReader::process_bytes_to_pydataevent::hca2ae367369454a6` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 34,202 (33.40 KiB) | 481 (481 B) |
| `src/readers/pyeventreader.rs:587:32`<br/>`s4lib::readers::pyeventreader::PyEventReader::ts_data_to_datetime::hdba3524ff3547a93` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 710 (710 B) | 10 (10 B) |
| `src/readers/pyeventreader.rs:493:30`<br/>`s4lib::readers::pyeventreader::PyEventReader::process_bytes_to_pydataevent::hca2ae367369454a6` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 142 (142 B) | 2 (2 B) |
| `src/readers/pyeventreader.rs:488:30`<br/>`s4lib::readers::pyeventreader::PyEventReader::process_bytes_to_pydataevent::hca2ae367369454a6` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 71 | 71 (71 B) | 1 (1 B) |
| `src/s4/s4.rs:2517:10`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 52 | 48,677 (47.54 KiB) | 936 (936 B) |
| `src/readers/pyeventreader.rs:629:36`<br/>`s4lib::readers::pyeventreader::PyEventReader::next::h09854740678f41a7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 35 | 280 (280 B) | 8 (8 B) |
| `src/readers/pyeventreader.rs:632:30`<br/>`s4lib::readers::pyeventreader::PyEventReader::next::h09854740678f41a7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 35 | 120 (120 B) | 3 (3 B) |
| `src/python/pyrunner.rs:794:22`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 18 | 983 (983 B) | 54 (54 B) |
| `src/python/pyrunner.rs:370:30`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 10 | 592 (592 B) | 59 (59 B) |
| `src/python/pyrunner.rs:817:23`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 9 | 231 (231 B) | 25 (25 B) |
| `src/s4/s4.rs:3825:5`<br/>`s4::s4::set_signal_handler::h1be51f5b5f6b4d86` | 1 | `main` | 7 | 181 (181 B) | 25 (25 B) |
| `src/s4/s4.rs:5318:15`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 5 | 303 (303 B) | 60 (60 B) |
| `src/python/pyrunner.rs:998:21`<br/>`s4lib::python::pyrunner::PyRunner::stderr_all_add::h4200654aa7455a91` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 4 | 3,060 (2.99 KiB) | 765 (765 B) |
| `src/python/pyrunner.rs:350:13`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 4 | 1,792 (1.75 KiB) | 448 (448 B) |
| `src/s4/s4.rs:363:13`<br/>`s4::s4::LOCAL_NOW::__init::{{closure}}::h3a2963b7ba27b12e` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| `src/python/pyrunner.rs:1126:24`<br/>`s4lib::python::pyrunner::PyRunner::write_read::hbe080931cac19bbf` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 3 | 240 (240 B) | 80 (80 B) |
| `src/s4/s4.rs:440:5`<br/>`<s4::s4::CLI_Color_Choice as clap_builder::derive::ValueEnum>::to_possible_value::hea38343b8f5aa490` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| `src/s4/s4.rs:5314:13`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 2 | 3,800 (3.71 KiB) | 1,900 (1.86 KiB) |
| `src/python/pyrunner.rs:365:21`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 1,024 (1.00 KiB) | 512 (512 B) |
| `src/s4/s4.rs:5248:13`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 2 | 200 (200 B) | 100 (100 B) |
| `src/s4/s4.rs:2855:16`<br/>`s4::s4::cli_process_blocksz::hdb09b92225a953df` | 1 | `main` | 2 | 156 (156 B) | 78 (78 B) |
| `src/s4/s4.rs:5435:59`<br/>`s4::s4::processing_loop::recv_many_chan::hf2b1c552a15b43da` | 1 | `main` | 2 | 144 (144 B) | 72 (72 B) |
| `src/python/pyrunner.rs:359:28`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 112 (112 B) | 56 (56 B) |
| `src/python/pyrunner.rs:354:43`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 68 (68 B) | 34 (34 B) |
| `src/python/pyrunner.rs:355:45`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 46 (46 B) | 23 (23 B) |
| `src/python/pyrunner.rs:367:63`<br/>`s4lib::python::pyrunner::PipeStreamReader::new::h58a8d02baf3621f7` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 2 | 46 (46 B) | 23 (23 B) |
| `src/s4/s4.rs:5205:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 2,452 (2.39 KiB) | 2,452 (2.39 KiB) |
| `src/printer/printers.rs:829:32`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/printer/printers.rs:828:21`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| `src/s4/s4.rs:5603:29`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 1,248 (1.22 KiB) | 1,248 (1.22 KiB) |
| `src/printer/printers.rs:792:22`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| `src/s4/s4.rs:5483:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 980 (980 B) | 980 (980 B) |
| `src/readers/pyeventreader.rs:433:26`<br/>`s4lib::readers::pyeventreader::PyEventReader::new::ha5b112e3f0f94258` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 512 (512 B) | 512 (512 B) |
| `src/s4/s4.rs:5121:17`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 368 (368 B) | 368 (368 B) |
| `src/s4/s4.rs:5316:9`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 280 (280 B) | 280 (280 B) |
| `src/s4/s4.rs:5006:34`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| `src/s4/s4.rs:3762:51`<br/>`s4::s4::main::h0da61b24b42f6b8a` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| `src/python/pyrunner.rs:815:37`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 216 (216 B) | 216 (216 B) |
| `src/readers/pyeventreader.rs:407:9`<br/>`s4lib::readers::pyeventreader::PyEventReader::new::ha5b112e3f0f94258` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 160 (160 B) | 160 (160 B) |
| `src/readers/filepreprocessor.rs:1472:22`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 153 (153 B) | 153 (153 B) |
| `src/s4/s4.rs:5012:49`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| `src/python/pyrunner.rs:767:36`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 144 (144 B) | 144 (144 B) |
| `src/s4/s4.rs:5014:40`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 116 (116 B) | 116 (116 B) |
| `src/s4/s4.rs:4660:9`<br/>`s4::s4::exec_pyeventprocessor::h1fe830bcd980deab` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 100 (100 B) | 100 (100 B) |
| `src/readers/pyeventreader.rs:939:20`<br/>`s4lib::readers::pyeventreader::PyEventReader::summary_complete::h2f3c0ca234f7d499` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 100 (100 B) | 100 (100 B) |
| `src/s4/s4.rs:5121:56`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| `src/readers/filepreprocessor.rs:1468:33`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| `src/s4/s4.rs:3511:29`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| `src/readers/filepreprocessor.rs:1502:87`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| `src/s4/s4.rs:2566:12`<br/>`<s4::s4::CLI_Args as clap_builder::derive::FromArgMatches>::from_arg_matches_mut::{{closure}}::h8f4c636549f04554` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| `src/s4/s4.rs:5203:32`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5215:28`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5029:35`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5027:44`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/s4/s4.rs:5031:41`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| `src/python/pyrunner.rs:1018:29`<br/>`s4lib::python::pyrunner::PyRunner::stderr_all_add::h4200654aa7455a91` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 81 (81 B) | 81 (81 B) |
| `src/readers/pyeventreader.rs:400:24`<br/>`s4lib::readers::pyeventreader::PyEventReader::new::ha5b112e3f0f94258` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 80 (80 B) | 80 (80 B) |
| `src/python/pyrunner.rs:259:25`<br/>`s4lib::python::pyrunner::find_python_executable::{{closure}}::h0868c70ef8c2696c` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 60 (60 B) | 60 (60 B) |
| `src/readers/filepreprocessor.rs:1502:53`<br/>`s4lib::readers::filepreprocessor::process_path::h940fb84aa9eeba43` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| `src/python/pyrunner.rs:255:25`<br/>`s4lib::python::pyrunner::find_python_executable::{{closure}}::h0868c70ef8c2696c` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5489:52`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5473:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/s4/s4.rs:5784:62`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| `src/readers/pyeventreader.rs:399:45`<br/>`s4lib::readers::pyeventreader::PyEventReader::new::ha5b112e3f0f94258` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 48 (48 B) | 48 (48 B) |
| `src/readers/pyeventreader.rs:364:41`<br/>`s4lib::readers::pyeventreader::PyEventReader::new::ha5b112e3f0f94258` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 48 (48 B) | 48 (48 B) |
| `src/s4/s4.rs:3484:33`<br/>`s4::s4::cli_process_args::hbb85d08869f2b49d` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| `src/python/venv.rs:95:5`<br/>`s4lib::python::venv::venv_path::h2ab41d82f1e91eb5` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 40 (40 B) | 40 (40 B) |
| `src/s4/s4.rs:2918:5`<br/>`s4::s4::cli_process_tz_offset::h3231a46bb9d02e0c` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| `src/readers/helpers.rs:36:5`<br/>`s4lib::readers::helpers::path_to_fpath::h139b82511f099fe3` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 38 (38 B) | 38 (38 B) |
| `src/python/pyrunner.rs:877:26`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 38 (38 B) | 38 (38 B) |
| `src/readers/helpers.rs:30:5`<br/>`s4lib::readers::helpers::basename::hb2b98f4ae9040d70` | 1 | `main` | 1 | 38 (38 B) | 38 (38 B) |
| `src/s4/s4.rs:5317:32`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 38 (38 B) | 38 (38 B) |
| `src/s4/s4.rs:5319:19`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 38 (38 B) | 38 (38 B) |
| `src/python/pyrunner.rs:258:45`<br/>`s4lib::python::pyrunner::find_python_executable::{{closure}}::h0868c70ef8c2696c` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 30 (30 B) | 30 (30 B) |
| `src/python/pyrunner.rs:823:20`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 30 (30 B) | 30 (30 B) |
| `src/readers/pyeventreader.rs:393:36`<br/>`s4lib::readers::pyeventreader::PyEventReader::new::ha5b112e3f0f94258` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 27 (27 B) | 27 (27 B) |
| `src/python/pyrunner.rs:253:40`<br/>`s4lib::python::pyrunner::find_python_executable::{{closure}}::h0868c70ef8c2696c` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 26 (26 B) | 26 (26 B) |
| `src/python/venv.rs:94:5`<br/>`s4lib::python::venv::venv_path::h2ab41d82f1e91eb5` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2917:28`<br/>`s4::s4::cli_process_tz_offset::h3231a46bb9d02e0c` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| `src/s4/s4.rs:2589:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2716:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2607:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2651:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2578:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2817:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2635:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2729:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2703:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2741:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2555:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2681:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2780:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2756:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2693:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2803:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2568:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2768:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2621:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/s4/s4.rs:2669:5`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::h6a34b75517bcc036` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| `src/printer/printers.rs:794:28`<br/>`s4lib::printer::printers::PrinterLogMessage::new::hb43a01fede4ab37f` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| `src/python/venv.rs:87:35`<br/>`s4lib::python::venv::venv_path::h2ab41d82f1e91eb5` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 10 (10 B) | 10 (10 B) |
| `src/s4/s4.rs:2890:23`<br/>`s4::s4::cli_parse_blocksz::h7d22fd388ec587af` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5305:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2602:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h79a17608cfbbd63c` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:5500:26`<br/>`s4::s4::processing_loop::h3dbe51f1dd9389a6` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:2811:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hcf68c27431597d26` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/s4/s4.rs:3440:9`<br/>`s4::s4::unescape::unescape_str::h8985d470cc66fb45` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| `src/python/pyrunner.rs:861:13`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 6 (6 B) | 6 (6 B) |
| `src/python/pyrunner.rs:850:13`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 6 (6 B) | 6 (6 B) |
| `src/s4/s4.rs:2811:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::hcf68c27431597d26` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2793:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h740ed821b86ae503` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2751:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h5b3e8b9496df1070` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2889:32`<br/>`s4::s4::cli_parse_blocksz::h7d22fd388ec587af` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2775:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h9f9bfb2cadb5032e` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/s4/s4.rs:2736:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h6b55cb440a7a965d` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| `src/python/pyrunner.rs:855:13`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 4 (4 B) | 4 (4 B) |
| `src/python/pyrunner.rs:866:13`<br/>`s4lib::python::pyrunner::PyRunner::new::h5a0a9355fccf6b40` | 3 | `FileCoAuth-2025-12-21.1216.11020.2.odl` | 1 | 4 (4 B) | 4 (4 B) |
| `src/s4/s4.rs:2763:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::h961ed6d112333819` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| `src/readers/filepreprocessor.rs:666:31`<br/>`s4lib::readers::filepreprocessor::pathbuf_to_filetype_impl::h64191eb604206c63` | 1 | `main` | 1 | 3 (3 B) | 3 (3 B) |
| `src/s4/s4.rs:2709:9`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha45a23112ac7249a` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| `src/s4/s4.rs:2709:27`<br/>`<s4::s4::CLI_Args as clap_builder::derive::Args>::augment_args::{{closure}}::ha45a23112ac7249a` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |



## Allocator Tracking summary

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| normal allocations | 274,607 | 1,212 | normal program allocations; this is the most useful number |
| total deallocations | 180,196,573 | 217,210 | includes normal program deallocations and tracking deallocations |
| current outstanding | 67,895,400 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| tracked | bytes | calls | about |
| :--- | ---: | ---: | :--- |
| total from tracking | 247,817,343 | 314,947 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 245,922,729 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,894,614 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 0 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| parameter | value | about |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 126 | entries in the table above |
| cached file names | 7 | |
| cached function names | 38 | |
| cached thread names | 2 | |

2026-07-03 19:48:26.130542707 -07:00
