# `2023.10.31.asl`

## `--version`

```text
s4 (Super Speedy Syslog Searcher)
Version: 0.11.84
MSRV: 1.99.1
Profile: alloc_tracker
Allocator: alloc_tracker
Features: alloc_tracker
Platform: x86_64-unknown-linux-gnu
Target OS: linux
Target OS Family: unix
Arch: x86_64
CPU Target: default
CPU Features: fxsr,sse,sse2
Compiled Regular Expressions: 188
Compiler Version: 1.99.1
Rust Build Flags: 
Optimization Level: 0
Git Commit: 9816188dac9bc627da8325a4885ce1475f95b3b0
Build Date: 2026-10-27T00:31:19
License: MIT
Repository: https://github.com/jtmoon79/super-speedy-syslog-searcher
Author: James Thomas Moon

```

## Command

`$ ./target/alloc_tracker/s4 ./logs/MacOS11/DiagnosticMessages/2023.10.31.asl`

## Allocator Tracking results

| **File:line:col**<br/>**Call Site** | **Thread<br/>ID** | **Thread<br/>Name** | **Allocations** | **Bytes** | **Bytes<br/>per Allocation** |
| :-- | ---: | :--- | ---: | ---: | ---: |
| [`src/readers/aslparser.rs:151:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/aslparser.rs#L151)<br/>`<s4lib[25d088bf9007c21a]::readers::aslparser::AslParser<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManaged>>::string_ref` | 3 | `2023.10.31.asl` | 2,168 | 54,523 (53.25 KiB) | 25 (25 B) |
| [`src/data/asl.rs:147:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/data/asl.rs#L147)<br/>`<s4lib[25d088bf9007c21a]::data::asl::AslRecord>::render` | 3 | `2023.10.31.asl` | 764 | 22,920 (22.38 KiB) | 30 (30 B) |
| [`src/data/asl.rs:56:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/data/asl.rs#L56)<br/>`<s4lib[25d088bf9007c21a]::data::asl::RenderBuffer>::into_string` | 3 | `2023.10.31.asl` | 191 | 89,834 (87.73 KiB) | 470 (470 B) |
| [`src/readers/aslparser.rs:211:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/aslparser.rs#L211)<br/>`<s4lib[25d088bf9007c21a]::readers::aslparser::AslParser<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManaged>>::next_record` | 3 | `2023.10.31.asl` | 191 | 54,096 (52.83 KiB) | 283 (283 B) |
| [`src/readers/aslparser.rs:193:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/aslparser.rs#L193)<br/>`<s4lib[25d088bf9007c21a]::readers::aslparser::AslParser<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManaged>>::next_record` | 3 | `2023.10.31.asl` | 191 | 40,188 (39.25 KiB) | 210 (210 B) |
| [`src/data/asl.rs:147:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/data/asl.rs#L147)<br/>`<s4lib[25d088bf9007c21a]::data::asl::AslRecord>::render` | 3 | `2023.10.31.asl` | 191 | 1,528 (1.49 KiB) | 8 (8 B) |
| [`src/s4/s4.rs:3866:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3866)<br/>`s4[382436636be345a4]::s4::cli_process_args` | 1 | `main` | 165 | 17,888 (17.47 KiB) | 108 (108 B) |
| [`src/s4/s4.rs:2918:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2918)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 38 | 9,709 (9.48 KiB) | 255 (255 B) |
| [`src/readers/aslreader.rs:252:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/aslreader.rs#L252)<br/>`<s4lib[25d088bf9007c21a]::readers::aslreader::AslReader>::analyze` | 3 | `2023.10.31.asl` | 32 | 31,968 (31.22 KiB) | 999 (999 B) |
| [`src/readers/aslparser.rs:169:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/aslparser.rs#L169)<br/>`<s4lib[25d088bf9007c21a]::readers::aslparser::AslParser<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManaged>>::next_record` | 3 | `2023.10.31.asl` | 7 | 4,684 (4.57 KiB) | 669 (669 B) |
| [`src/s4/s4.rs:4175:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L4175)<br/>`s4[382436636be345a4]::s4::set_signal_handler` | 1 | `main` | 6 | 149 (149 B) | 24 (24 B) |
| [`src/s4/s4.rs:5819:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5819)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 4 | 223 (223 B) | 55 (55 B) |
| [`src/readers/filehandlemanager.rs:423:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L423)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManagerState>::new` | 1 | `main` | 3 | 34,880 (34.06 KiB) | 11,626 (11.35 KiB) |
| [`src/s4/s4.rs:379:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L379)<br/>`s4[382436636be345a4]::s4::LOCAL_NOW::__rust_std_internal_init_fn::{closure#0}` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| [`src/s4/s4.rs:5917:66`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5917)<br/>`s4[382436636be345a4]::s4::processing_loop::recv_many_chan` | 1 | `main` | 3 | 240 (240 B) | 80 (80 B) |
| [`src/s4/s4.rs:456:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L456)<br/>`<s4[382436636be345a4]::s4::CLI_Color_Choice as clap_builder[639015aad05df3ec]::derive::ValueEnum>::to_possible_value` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| [`src/s4/s4.rs:5812:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5812)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 2 | 3,960 (3.87 KiB) | 1,980 (1.93 KiB) |
| [`src/s4/s4.rs:5609:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5609)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 2 | 648 (648 B) | 324 (324 B) |
| [`src/s4/s4.rs:4320:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L4320)<br/>`s4[382436636be345a4]::s4::chan_send` | 3 | `2023.10.31.asl` | 2 | 144 (144 B) | 72 (72 B) |
| [`src/s4/s4.rs:3160:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3160)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 20,480 (20.00 KiB) | 20,480 (20.00 KiB) |
| [`src/s4/s4.rs:3066:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3066)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 10,240 (10.00 KiB) | 10,240 (10.00 KiB) |
| [`src/s4/s4.rs:3004:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3004)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 5,120 (5.00 KiB) | 5,120 (5.00 KiB) |
| [`src/s4/s4.rs:5642:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5642)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 2,580 (2.52 KiB) | 2,580 (2.52 KiB) |
| [`src/s4/s4.rs:2954:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2954)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 2,560 (2.50 KiB) | 2,560 (2.50 KiB) |
| [`src/printer/printers.rs:832:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L832)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/printer/printers.rs:833:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L833)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/s4/s4.rs:6112:46`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L6112)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 1,288 (1.26 KiB) | 1,288 (1.26 KiB) |
| [`src/printer/printers.rs:796:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L796)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| [`src/s4/s4.rs:5985:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5985)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 980 (980 B) | 980 (980 B) |
| [`src/readers/filehandlemanager.rs:876:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L876)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManager>::new_open_max` | 1 | `main` | 1 | 384 (384 B) | 384 (384 B) |
| [`src/s4/s4.rs:5541:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5541)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 320 (320 B) | 320 (320 B) |
| [`src/readers/filehandlemanager.rs:947:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L947)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `2023.10.31.asl` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/s4/s4.rs:5418:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5418)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/s4/s4.rs:5814:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5814)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 232 (232 B) | 232 (232 B) |
| [`src/s4/s4.rs:4133:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L4133)<br/>`s4[382436636be345a4]::s4::main` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| [`src/s4/s4.rs:5425:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5425)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| [`src/s4/s4.rs:5958:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5958)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/s4/s4.rs:5427:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5427)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:2963:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2963)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut::{closure#0}` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| [`src/readers/filepreprocessor.rs:1777:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1777)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 95 (95 B) | 95 (95 B) |
| [`src/readers/filehandlemanager.rs:1046:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L1046)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManager>::handle_reservations` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:5963:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5963)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:5644:36`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5644)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:5962:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5962)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/readers/filepreprocessor.rs:1820:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1820)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| [`src/s4/s4.rs:5442:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5442)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5440:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5440)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5640:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5640)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5444:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5444)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/readers/filehandlemanager.rs:374:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L374)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileEntryManaged>::new` | 3 | `2023.10.31.asl` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/aslreader.rs:285:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/aslreader.rs#L285)<br/>`<s4lib[25d088bf9007c21a]::readers::aslreader::AslReader>::summary_complete` | 3 | `2023.10.31.asl` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:5120:56`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5120)<br/>`s4[382436636be345a4]::s4::exec_aslreader` | 3 | `2023.10.31.asl` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:5541:61`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5541)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:5700:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5700)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/filepreprocessor.rs:1820:88`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1820)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:3911:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3911)<br/>`s4[382436636be345a4]::s4::cli_process_args` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:5700:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5700)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/filepreprocessor.rs:1773:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1773)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:3884:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3884)<br/>`s4[382436636be345a4]::s4::cli_process_args` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/s4/s4.rs:3219:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3219)<br/>`s4[382436636be345a4]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| [`src/s4/s4.rs:6306:62`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L6306)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:5991:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5991)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:5964:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5964)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:5815:47`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5815)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:5661:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5661)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/readers/filehandlemanager.rs:543:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L543)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManagerState>::touch` | 3 | `2023.10.31.asl` | 1 | 24 (24 B) | 24 (24 B) |
| [`src/s4/s4.rs:3218:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3218)<br/>`s4[382436636be345a4]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| [`src/s4/s4.rs:3064:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3064)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3168:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3168)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3076:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3076)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2984:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2984)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3030:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3030)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3146:19`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3146)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3002:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3002)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3134:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3134)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2963:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2963)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3158:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3158)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3045:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3045)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3098:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3098)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3016:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3016)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3088:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3088)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3109:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3109)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2973:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2973)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3122:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3122)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/readers/helpers.rs:116:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/helpers.rs#L116)<br/>`s4lib[25d088bf9007c21a]::readers::helpers::basename` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/printer/printers.rs:798:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L798)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/s4/s4.rs:5817:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5817)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/s4/s4.rs:5800:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5800)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3842:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3842)<br/>`s4[382436636be345a4]::s4::unescape::unescape_str` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:2999:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2999)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#0}` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3131:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3131)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#3}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:3155:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3155)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#5}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:5670:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5670)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:6002:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L6002)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:3143:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3143)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#4}` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/readers/filepreprocessor.rs:680:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L680)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::pathbuf_to_filetype_impl` | 1 | `main` | 1 | 3 (3 B) | 3 (3 B) |
| [`src/s4/s4.rs:3106:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3106)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| [`src/s4/s4.rs:3106:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3106)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |


## Allocator Tracking summary

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| normal allocations | 425,894 | 4,045 | normal program allocations; this is the most useful number |
| total deallocations | 187,985,054 | 242,452 | includes normal program deallocations and tracking deallocations |
| current outstanding | 75,439,322 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| total from tracking | 262,998,457 | 345,823 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 261,285,306 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,713,151 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 0 | 100 to 1 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| **parameter** | **value** | **about** |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 98 | entries in the table above |
| cached file names | 8 | |
| cached function names | 34 | |
| cached thread names | 2 | |

*2026-10-31 16:05:31.640022022 -07:00*
