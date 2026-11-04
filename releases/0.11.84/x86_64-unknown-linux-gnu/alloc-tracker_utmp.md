# `utmp`

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

`$ ./target/alloc_tracker/s4 ./logs/OpenBSD7.4/x86_64/utmp`

## Allocator Tracking results

| **File:line:col**<br/>**Call Site** | **Thread<br/>ID** | **Thread<br/>Name** | **Allocations** | **Bytes** | **Bytes<br/>per Allocation** |
| :-- | ---: | :--- | ---: | ---: | ---: |
| [`src/s4/s4.rs:3866:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3866)<br/>`s4[382436636be345a4]::s4::cli_process_args` | 1 | `main` | 165 | 17,831 (17.41 KiB) | 108 (108 B) |
| [`src/s4/s4.rs:2918:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2918)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 38 | 9,709 (9.48 KiB) | 255 (255 B) |
| [`src/s4/s4.rs:4175:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L4175)<br/>`s4[382436636be345a4]::s4::set_signal_handler` | 1 | `main` | 6 | 149 (149 B) | 24 (24 B) |
| [`src/s4/s4.rs:5819:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5819)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 4 | 213 (213 B) | 53 (53 B) |
| [`src/readers/filehandlemanager.rs:423:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L423)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManagerState>::new` | 1 | `main` | 3 | 34,880 (34.06 KiB) | 11,626 (11.35 KiB) |
| [`src/s4/s4.rs:379:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L379)<br/>`s4[382436636be345a4]::s4::LOCAL_NOW::__rust_std_internal_init_fn::{closure#0}` | 1 | `main` | 3 | 5,924 (5.79 KiB) | 1,974 (1.93 KiB) |
| [`src/s4/s4.rs:5917:66`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5917)<br/>`s4[382436636be345a4]::s4::processing_loop::recv_many_chan` | 1 | `main` | 3 | 240 (240 B) | 80 (80 B) |
| [`src/readers/blockreader.rs:2546:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L2546)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::new` | 3 | `utmp` | 3 | 216 (216 B) | 72 (72 B) |
| [`src/s4/s4.rs:456:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L456)<br/>`<s4[382436636be345a4]::s4::CLI_Color_Choice as clap_builder[639015aad05df3ec]::derive::ValueEnum>::to_possible_value` | 1 | `main` | 3 | 24 (24 B) | 8 (8 B) |
| [`src/s4/s4.rs:5812:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5812)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 2 | 3,960 (3.87 KiB) | 1,980 (1.93 KiB) |
| [`src/s4/s4.rs:5609:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5609)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 2 | 648 (648 B) | 324 (324 B) |
| [`src/data/fixedstruct.rs:4308:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/data/fixedstruct.rs#L4308)<br/>`s4lib[25d088bf9007c21a]::data::fixedstruct::buffer_to_fixedstructptr` | 3 | `utmp` | 2 | 608 (608 B) | 304 (304 B) |
| [`src/readers/fixedstructreader.rs:1090:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/fixedstructreader.rs#L1090)<br/>`<s4lib[25d088bf9007c21a]::readers::fixedstructreader::FixedStructReader>::score_file` | 3 | `utmp` | 2 | 80 (80 B) | 40 (40 B) |
| [`src/s4/s4.rs:3160:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3160)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 20,480 (20.00 KiB) | 20,480 (20.00 KiB) |
| [`src/s4/s4.rs:3066:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3066)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 10,240 (10.00 KiB) | 10,240 (10.00 KiB) |
| [`src/readers/blockreader.rs:3576:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L3576)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::read_block_File` | 3 | `utmp` | 1 | 6,992 (6.83 KiB) | 6,992 (6.83 KiB) |
| [`src/s4/s4.rs:3004:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3004)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 5,120 (5.00 KiB) | 5,120 (5.00 KiB) |
| [`src/s4/s4.rs:5642:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5642)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 2,580 (2.52 KiB) | 2,580 (2.52 KiB) |
| [`src/s4/s4.rs:2954:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2954)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 2,560 (2.50 KiB) | 2,560 (2.50 KiB) |
| [`src/printer/printers.rs:832:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L832)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/printer/printers.rs:833:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L833)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 2,056 (2.01 KiB) | 2,056 (2.01 KiB) |
| [`src/s4/s4.rs:6112:46`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L6112)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 1,288 (1.26 KiB) | 1,288 (1.26 KiB) |
| [`src/printer/printers.rs:796:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L796)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 1,024 (1.00 KiB) | 1,024 (1.00 KiB) |
| [`src/s4/s4.rs:5985:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5985)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 980 (980 B) | 980 (980 B) |
| [`src/readers/fixedstructreader.rs:840:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/fixedstructreader.rs#L840)<br/>`<s4lib[25d088bf9007c21a]::readers::fixedstructreader::FixedStructReader>::insert_cache_entry` | 3 | `utmp` | 1 | 808 (808 B) | 808 (808 B) |
| [`src/readers/filehandlemanager.rs:876:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L876)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManager>::new_open_max` | 1 | `main` | 1 | 384 (384 B) | 384 (384 B) |
| [`src/s4/s4.rs:5541:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5541)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 320 (320 B) | 320 (320 B) |
| [`src/readers/fixedstructreader.rs:1325:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/fixedstructreader.rs#L1325)<br/>`<s4lib[25d088bf9007c21a]::readers::fixedstructreader::FixedStructReader>::preprocess_timevalues` | 3 | `utmp` | 1 | 280 (280 B) | 280 (280 B) |
| [`src/readers/filehandlemanager.rs:947:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L947)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManager>::request_open_managed` | 3 | `utmp` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/s4/s4.rs:5418:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5418)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 276 (276 B) | 276 (276 B) |
| [`src/s4/s4.rs:5814:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5814)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 232 (232 B) | 232 (232 B) |
| [`src/s4/s4.rs:4133:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L4133)<br/>`s4[382436636be345a4]::s4::main` | 1 | `main` | 1 | 224 (224 B) | 224 (224 B) |
| [`src/readers/fixedstructreader.rs:507:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/fixedstructreader.rs#L507)<br/>`<s4lib[25d088bf9007c21a]::readers::fixedstructreader::FixedStructReader>::new` | 3 | `utmp` | 1 | 192 (192 B) | 192 (192 B) |
| [`src/readers/blockreader.rs:3511:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L3511)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::store_block_in_storage` | 3 | `utmp` | 1 | 192 (192 B) | 192 (192 B) |
| [`src/s4/s4.rs:5425:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5425)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 148 (148 B) | 148 (148 B) |
| [`src/s4/s4.rs:5958:22`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5958)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 128 (128 B) | 128 (128 B) |
| [`src/readers/blockreader.rs:3523:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L3523)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::store_block_in_storage` | 3 | `utmp` | 1 | 104 (104 B) | 104 (104 B) |
| [`src/s4/s4.rs:5427:39`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5427)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 100 (100 B) | 100 (100 B) |
| [`src/s4/s4.rs:2963:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2963)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::FromArgMatches>::from_arg_matches_mut::{closure#0}` | 1 | `main` | 1 | 96 (96 B) | 96 (96 B) |
| [`src/s4/s4.rs:5644:36`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5644)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:5963:38`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5963)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/s4/s4.rs:5962:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5962)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 84 (84 B) | 84 (84 B) |
| [`src/readers/filepreprocessor.rs:1777:31`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1777)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 76 (76 B) | 76 (76 B) |
| [`src/readers/filepreprocessor.rs:1820:49`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1820)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 56 (56 B) | 56 (56 B) |
| [`src/data/fixedstruct.rs:3772:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/data/fixedstruct.rs#L3772)<br/>`s4lib[25d088bf9007c21a]::data::fixedstruct::filesz_to_types` | 3 | `utmp` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5640:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5640)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5442:35`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5442)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5440:44`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5440)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:5444:41`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5444)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 52 (52 B) | 52 (52 B) |
| [`src/s4/s4.rs:3884:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3884)<br/>`s4[382436636be345a4]::s4::cli_process_args` | 1 | `main` | 1 | 48 (48 B) | 48 (48 B) |
| [`src/readers/blockreader.rs:3597:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L3597)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::read_block_File` | 3 | `utmp` | 1 | 40 (40 B) | 40 (40 B) |
| [`src/s4/s4.rs:3219:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3219)<br/>`s4[382436636be345a4]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 40 (40 B) | 40 (40 B) |
| [`src/s4/s4.rs:5964:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5964)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:6306:62`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L6306)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/s4/s4.rs:5991:52`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5991)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 36 (36 B) | 36 (36 B) |
| [`src/readers/blockreader.rs:3491:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L3491)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::store_block_in_LRU_cache` | 3 | `utmp` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:5815:47`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5815)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:5661:30`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5661)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 32 (32 B) | 32 (32 B) |
| [`src/s4/s4.rs:4621:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L4621)<br/>`s4[382436636be345a4]::s4::exec_fixedstructprocessor` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/fixedstructreader.rs:355:68`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/fixedstructreader.rs#L355)<br/>`<s4lib[25d088bf9007c21a]::readers::fixedstructreader::FixedStructReader>::new` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/blockreader.rs:1322:40`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L1322)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::new` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/filehandlemanager.rs:374:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L374)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileEntryManaged>::new` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/blockreader.rs:1354:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/blockreader.rs#L1354)<br/>`<s4lib[25d088bf9007c21a]::readers::blockreader::BlockReader>::new` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/fixedstructreader.rs:1563:32`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/fixedstructreader.rs#L1563)<br/>`<s4lib[25d088bf9007c21a]::readers::fixedstructreader::FixedStructReader>::summary_complete` | 3 | `utmp` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/filepreprocessor.rs:1773:33`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1773)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/s4/s4.rs:5541:61`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5541)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/s4/s4.rs:5700:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5700)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/filepreprocessor.rs:1820:88`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1820)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::process_path` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/s4/s4.rs:5700:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5700)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/s4/s4.rs:3911:34`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3911)<br/>`s4[382436636be345a4]::s4::cli_process_args` | 1 | `main` | 1 | 29 (29 B) | 29 (29 B) |
| [`src/readers/filehandlemanager.rs:543:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filehandlemanager.rs#L543)<br/>`<s4lib[25d088bf9007c21a]::readers::filehandlemanager::FileHandleManagerState>::touch` | 3 | `utmp` | 1 | 24 (24 B) | 24 (24 B) |
| [`src/s4/s4.rs:3218:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3218)<br/>`s4[382436636be345a4]::s4::cli_process_tz_offset` | 1 | `main` | 1 | 20 (20 B) | 20 (20 B) |
| [`src/s4/s4.rs:3016:17`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3016)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3146:19`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3146)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2984:13`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2984)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3088:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3088)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3030:18`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3030)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3076:23`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3076)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3045:20`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3045)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3134:21`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3134)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3158:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3158)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2973:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2973)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3122:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3122)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3098:25`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3098)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3064:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3064)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3002:16`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3002)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:2963:12`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2963)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3168:14`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3168)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/s4/s4.rs:3109:24`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3109)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args` | 1 | `main` | 1 | 16 (16 B) | 16 (16 B) |
| [`src/printer/printers.rs:798:28`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/printer/printers.rs#L798)<br/>`<s4lib[25d088bf9007c21a]::printer::printers::PrinterLogMessage>::new` | 1 | `main` | 1 | 14 (14 B) | 14 (14 B) |
| [`src/s4/s4.rs:5800:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5800)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3842:51`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3842)<br/>`s4[382436636be345a4]::s4::unescape::unescape_str` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:2999:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L2999)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#0}` | 1 | `main` | 1 | 8 (8 B) | 8 (8 B) |
| [`src/s4/s4.rs:3131:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3131)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#3}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/s4/s4.rs:3155:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3155)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#5}` | 1 | `main` | 1 | 5 (5 B) | 5 (5 B) |
| [`src/readers/filepreprocessor.rs:1002:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/filepreprocessor.rs#L1002)<br/>`s4lib[25d088bf9007c21a]::readers::filepreprocessor::pathbuf_to_filetype_impl` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:3143:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3143)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#4}` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:5817:29`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5817)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:6002:26`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L6002)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:5670:10`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L5670)<br/>`s4[382436636be345a4]::s4::processing_loop` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/readers/helpers.rs:116:5`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/readers/helpers.rs#L116)<br/>`s4lib[25d088bf9007c21a]::readers::helpers::basename` | 1 | `main` | 1 | 4 (4 B) | 4 (4 B) |
| [`src/s4/s4.rs:3106:27`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3106)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |
| [`src/s4/s4.rs:3106:9`](https://github.com/jtmoon79/super-speedy-syslog-searcher/blob/69b737cec3b304f16ed036b0c29c1f683b560353/src/s4/s4.rs#L3106)<br/>`<s4[382436636be345a4]::s4::CLI_Args as clap_builder[639015aad05df3ec]::derive::Args>::augment_args::{closure#1}` | 1 | `main` | 1 | 1 (1 B) | 1 (1 B) |


## Allocator Tracking summary

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| normal allocations | 135,332 | 326 | normal program allocations; this is the most useful number |
| total deallocations | 187,405,408 | 229,506 | includes normal program deallocations and tracking deallocations |
| current outstanding | 75,440,664 | | outstanding allocated bytes as of this print |

## Allocator Tracking internals

| **tracked** | **bytes** | **calls** | **about** |
| :--- | ---: | ---: | :--- |
| total from tracking | 262,710,715 | 336,604 | tracking allocations; not part of the normal program allocations |
| tracking from backtrace | 260,997,578 | | tracking allocations specifically for `backtrace::trace` and `backtrace::resolve_frame`; subset of "total from tracking" |
| tracking from other | 1,713,137 | | other tracking allocations, not "from backtrace"; subset of "total from tracking" |
| ratio tracking to normal| 100 to 0 | 100 to 0 | ratio of tracking allocations/calls to normal program allocations/calls |
| diff table vs total | 0 | 0 | sanity check of total numbers and table numbers; should be 0 |

| **parameter** | **value** | **about** |
| :--- | ---: | :--- |
| frame depth | 1 | max depth of backtraced frames for each allocation call site; env var "S4_ALLOC_TRACKER_DEPTH" |
| call sites | 103 | entries in the table above |
| cached file names | 8 | |
| cached function names | 37 | |
| cached thread names | 2 | |

*2026-10-31 16:05:37.059451146 -07:00*
