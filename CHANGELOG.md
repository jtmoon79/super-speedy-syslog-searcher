# CHANGELOG<!-- omit in toc -->

<!--
Helper script `tools/changelog-link-gen.sh` can generate the addendum of markdown links for this CHANGELOG.md.
-->

Manual changelog for [super speedy syslog searcher](https://github.com/jtmoon79/super-speedy-syslog-searcher).

---

<!-- TODO per release: update TOC -->
<!-- Table of Contents updated by "Markdown All In One" extension for Visual Studio Code -->

- [Unreleased](#unreleased)
- [0.6.69](#0669)
  - [New](#new)
  - [Changes](#changes)
- [0.6.68](#0668)
  - [New](#new-1)
  - [Changes](#changes-1)
- [0.6.67](#0667)
  - [New](#new-2)
  - [Changes](#changes-2)
  - [Fixes](#fixes)
- [0.6.66](#0666)
  - [Changes](#changes-3)
  - [Fixes](#fixes-1)
- [0.6.65](#0665)
  - [New](#new-3)
  - [Changes](#changes-4)
- [0.6.64](#0664)
  - [New](#new-4)
  - [Changes](#changes-5)
  - [Fixes](#fixes-2)
- [0.6.63](#0663)
  - [New](#new-5)
  - [Changes](#changes-6)
  - [Fixes](#fixes-3)
- [0.6.62](#0662)
  - [Fixes](#fixes-4)
- [0.6.61](#0661)
  - [New](#new-6)
  - [Changes](#changes-7)
  - [Fixes](#fixes-5)
- [0.6.60](#0660)
  - [New](#new-7)
- [0.5.59](#0559)
  - [Changes](#changes-8)
  - [Fixes](#fixes-6)
- [0.5.58](#0558)
  - [New](#new-8)
  - [Changes](#changes-9)
- [0.4.57](#0457)
  - [Changes](#changes-10)
  - [Fixes](#fixes-7)
- [0.4.56](#0456)
  - [New](#new-9)
  - [Changes](#changes-11)
  - [Fixes](#fixes-8)
- [0.3.55](#0355)
  - [New](#new-10)
  - [Changes](#changes-12)
  - [Fixes](#fixes-9)
- [0.3.54](#0354)
  - [New](#new-11)
  - [Fixes](#fixes-10)
- [0.3.53](#0353)
  - [New](#new-12)
  - [Changes](#changes-13)
- [0.2.52](#0252)
  - [New](#new-13)
- [0.2.51](#0251)
  - [New](#new-14)
- [0.2.50](#0250)
  - [New](#new-15)
  - [Changes](#changes-14)
  - [Fixes](#fixes-11)
- [0.2.49](#0249)
  - [Changes](#changes-15)
  - [Fixes](#fixes-12)
- [0.2.48](#0248)
  - [New](#new-16)
  - [Changes](#changes-16)
  - [Fixes](#fixes-13)
- [0.2.47](#0247)
- [0.2.46](#0246)
  - [New](#new-17)
  - [Changes](#changes-17)
  - [Fixes](#fixes-14)
- [0.1.45](#0145)
  - [New](#new-18)
  - [Changes](#changes-18)
- [0.1.44](#0144)
  - [New](#new-19)
  - [Changes](#changes-19)
  - [Fixes](#fixes-15)
- [0.1.43](#0143)
  - [New](#new-20)
  - [Changes](#changes-20)
- [0.1.42](#0142)
  - [Changes](#changes-21)
- [0.1.41](#0141)
  - [Changes](#changes-22)
  - [Fixes](#fixes-16)
- [0.1.40](#0140)
  - [New](#new-21)
  - [Changes](#changes-23)
- [0.1.39](#0139)
  - [Changes](#changes-24)
- [0.1.38](#0138)
  - [New](#new-22)
  - [Changes](#changes-25)
- [0.0.37](#0037)
  - [New](#new-23)
  - [Changes](#changes-26)
- [0.0.36](#0036)
  - [New](#new-24)
  - [Changes](#changes-27)
  - [Fixes](#fixes-17)
- [0.0.35](#0035)
  - [New](#new-25)
  - [Fixes](#fixes-18)
- [0.0.34](#0034)
  - [New](#new-26)
  - [Fixes](#fixes-19)
- [0.0.33](#0033)
  - [New](#new-27)
- [0.0.32](#0032)
  - [New](#new-28)
  - [Fixes](#fixes-20)
- [0.0.31](#0031)
  - [New](#new-29)
- [0.0.30](#0030)
  - [New](#new-30)
  - [Changes](#changes-28)
- [0.0.29](#0029)
  - [Changes](#changes-29)
- [0.0.28](#0028)
  - [New](#new-31)
  - [Changes](#changes-30)
  - [Fixes](#fixes-21)
- [0.0.27](#0027)
  - [New](#new-32)
  - [Changes](#changes-31)
- [0.0.26](#0026)
  - [New](#new-33)
  - [Changes](#changes-32)
  - [Fixes](#fixes-22)
- [0.0.25](#0025)
  - [New](#new-34)
  - [Changes](#changes-33)
  - [Fixes](#fixes-23)
- [0.0.24](#0024)
  - [New](#new-35)
  - [Changes](#changes-34)
- [0.0.23](#0023)
  - [New](#new-36)
  - [Changes](#changes-35)
  - [Fixes](#fixes-24)
- [0.0.22](#0022)
  - [New](#new-37)
  - [Changes](#changes-36)
  - [Fixes](#fixes-25)
- [0.0.21](#0021)
  - [New](#new-38)
  - [Fixes](#fixes-26)

---

<!--
TODO per release:

1. Developers must manually create the sections. Do not create empty sections.
2. Developers must manually prefix categories (listed below).

Recommend using `tools/changelog-link-gen.sh` after done editing the sections.

---

Sections:

  ### New

  ### Fixes

  ### Changes

Categories:

  (LIB) - changes to the library source
  (BIN) - changes to the binary source (bin.rs, CLI options, etc.)
  (DEBUG) - changes to the either source only affecting debug builds
  (BUILD) - changes to the build (i.e. Cargo.toml)
  (DOCS) - changes to docstrings ("rustdocs"), docs.rs stuff, READMEs
  (CI) - changes to github workflows, codecov
  (TEST) - changes to tests
  (TOOLS) - changes to scripts under `tools/`
  (PROJECT) - changes to READMEs, CHANGELOG, other non-source tweaks

Helpful `git log` command for generating changelog entries:

    git log --pretty=format:'- %s %Cred([%H])%Creset%C(yellow)%d%Creset' --abbrev-commit <tag-previous-release>...<tag-current-release>
-->

## Unreleased

[unreleased diff]

[unreleased diff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/latest...main

<!-- TODO per release: Add Section(s) -->

---

## 0.6.69

_Released 2024-03-17_

[0.6.68..0.6.69]

_MSRV 1.67.1_

### New

- (DOCS) syslogprocessor.rs docstring about "syslog file" ([c618a199f59706ad2cfca64e2c37bbe4b615faf1])
- (DOCS) datetime.rs NFC docstring Result_Filter_DateTime1 ([4c7b6f1adf7398dfa570224ca470f8a71870a831])
- (LIB) refactor for fixedstruct [Issue #100] [Issue #108] [Issue #109] [Issue #121] [Issue #171] [Issue #217] ([1aed86246f06e1e3f68f692d81b45c2e22be60b5])
- (LIB) replace uapi for utmpx processing [Issue #100] [Issue #108] [Issue #109] [Issue #121] [Issue #171] [Issue #217] ([e8406c56c77869ad8e70e9e1e7e448a0f458a204])
- (LIB) datetime.rs,bin add DateTimeParseDatasCompiledCount summary stat ([9a2384992a0bdb78769a7aeba8393ab4767713d1])
- (PROJECT) add file logs/programs/CPUID/CT-Log 2023-09-04 05-44-16.csv ([48c5a5e40fcf29e581fb329dd8c2e778a4e08b92])
- (PROJECT) add log ./logs/other/tests/dtf9c-23-12x2.log.xz ([0c1f5655a108865ec3fbccc964d59d455f7e27a1])
- (PROJECT) add logs MacOS11 MacOS12.6 NetBSD9.3 FreeBSD13.1 OpenBSD7.2 ([7d3305cc028ee0f963e0def854350e9d3eb69cb0])
- (PROJECT) add logs/Android9 ([c9a5de8fb611785c5d8bfd6e6942f48006cf9814])
- (PROJECT) add logs gen-1000-3-foobar.log.{tar,xz} ([8ec4d012fffb16013940d723077cdc44af0b156a])
- (TEST) blockreader_tests.rs mark should_panic [Issue #201] ([a746b065aa719a05f477224eba7fe551e62ebddc])
- (TEST) add datetime pattern dtf14a ([70a30a2b0b651a438223f7249c6ce47931acaa92])
- (TEST) datetime_tests.rs add test cases to test_slice_contains_X_2 ([a468ffadc41916adb608b633acf0dd8f45d255a9])
- (TOOLS) add tools/cargo-udeps.sh ([df2bcdef317ceb778093641fe97b8cf5664bf4bd])

### Changes

- (BIN) bin.rs NFC debug print syslogprocessor stages more clearly ([9bfb3ca7c5eeeaa20a9a5e6071206a98a3e7fa17]
- (BIN) bin.rs ordered invalid results ([e75daad23339e77fe0f36b3ef666c68f9d28b60a])
- (BIN) bin.rs do not print after printing error [Issue #241] [Issue #250] ([157d54a4dda13b1f0b4743daa55b77d20887e82a])
- (CI) rust.yml NFC move log-files-time-update.sh ([25b35ee8e1f7b7ed7d5119bca73becc6ad721617])
- (CI) rust.yml build 1.76.0, NFC vertical declaration ([baff6da9a9bc313db65a613d5edae82d67aee4c2])
- (CI) bump MSRV 1.67.1 ([505280afb5459be37c9905f1e7b23983b2e7e287])
- (CI) rust.yml clippy uses default profile ([89f67f3024dd9805313a2cff67ca6e0dc901fb40])
- (CI) rust.yml use moonrepo/setup-rust@v1 ([cef66e50bd7b149177a635d0f2bb17e1b77799ec])
- (CI) rust.yml add job_test_wasm ([aec8cebca812844afec8050e30d93fb8fa3bb203])
- (CI) rust.yml comment job_test_wasm [Issue #171] ([da6f91faf961dff4f1adbf528cd4025d98cd3624])
- (CI) rust.yml use download-artifact@v4 ([8c23fd059412310208b811d5d771caf617f3d0c0])
- (CI) rust.yml NFC adjust S4_TEST_FILES indent ([92b97540ecc5b69f957552118a47498022d5a9c1])
- (CI) rust.yml update setup-python@v5 ([f46be6a032569b5726d4df69efc519cec1e8fb29])
- (CI) rust.yml update upload-artifact@v4 ([e4f8059e97c3ba25401f8752607a66fea4dee10d])
- (CI) rust.yml add more S4_TEST_FILES ([8f8e8b7a856f3b3fd3a529d85830032e73510a88])
- (CI) rust.yml update checkout@v4 ([7432b7c5ea20165e443d6440fcc3cb84393f3a96])
- (CI) rust.yml run for Cargo.lock ([6a327d18341b245c61839b70cba29dc91f888f1b])
- (CI) rust.yml correctly override rust version ([068a0a2893f6007a51aa9e65c997b9e08e72c3a5])
- (CI) rust.yml add job_cross_targets ([d1dfe28abda8f6f8b46a1aebaee5750521fb5854])
- (CI) rust.yml remove tar logs.tgz upload ([ef7d59a60d323655aa6b3616f0d10a78ab11b565])
- (CI) rust.yml test on Windows, Mac OS, binstall nextest [Issue #202] [Issue #219] [Issue #218] ([659c824e36450d279d6fd684bdf848530da137f5])
- (CI) rust.yml job grcov uses rust 1.67.0 [Issue #170] ([78579a9158dc463e33c7b5ce1d248258bac89ae3])
- (CI) rust.yml uploaded artifacts have specific name ([007f752fdd69eb37df38171a7e485f5ae026ec6c])
- (CI) rust.yml remove job_grcov ([b2a0ab53877db5bf91b216baf3ba5e08853da559])
- (CI) rust.yml use script compare-debug-release.sh ([35914891bd90cfadbe17867c370c65429883e879])
- (CI) rust.yml tweak `on` for `pull_request` ([616bef16cacceb26dae625be830141b8ab2252e7])
- (CI) rust.yml use newer rust for job llvm_cov ([f363682cfb202a8fcfaf591c0db5f6fbbd472fa1])
- (CI) rust.yml use script compare-debug-release.sh ([231fcc46d921f781a1cdaa188c9f7e189e2709cd])
- (CI) rust.yml tweak `on` for `pull_request` ([24c21a15728132fcbcc6c3ce9724ed8ff19ec8a1])
- (CI) rust.yml msrv add 1.72.0, drop 1.67.1 ([f968b462d74e05c806bf5560f356799aa40b7104])
- (CI) rust.yml use newer rust for job llvm_cov ([69767ad626e27e0fda881c1e62b374165bd17825])
- (CI) rust.yml remove call cargo-llvm-cov-run.sh ([2ff0d197842c39450275d6d09bdd7ed06db1e735])
- (CI) rust.yml remove tar logs.tgz upload ([9fbc6318f6931ba60d43843b387c4bf049d4742e])
- (CI) rust.yml test on Windows, Mac OS, binstall nextest ([2e10dd1cab1dd3a68fa66207f03d66e4e2e72c0c])
- (CI) rust.yml NFC move log-files-time-update.sh ([00171bbdf238fd9c1ba6d89fa29a730318332d7e])
- (DOCS) README add more sections to examples in "logging chaos" ([071536e7a77bc4fefcbba6874040d0a4c77ce4b])
- (DOCS) README fix missing link to local file ([c13d0820b77a54b1e15a0f42ecea6d6b250a9fc2])
- (DOCS) README update section HACKS ([c4f86ec51cd2e3b21260d7314398b34d0661fdd7])
- (LIB) update chrono 0.4.28 ([0021f0576d0d629c72028443f2a266f957e5b084])
- (LIB) bump chrono 0.4.27, add MINUS SIGN tests ([498ad5f4cc0af31ac552e7aa6f9ee1b7ef030e13])
- (LIB) bump MSRV 1.67.0, lock clap 4.1.0 ([4db00567813d9b236c77a49a33e399ad5c0c94ab])
- (LIB) Cargo.toml criterion 0.5.1 ([2f1fc58e6727c42cfc83298c0e421743f63899af])
- (LIB) Cargo.toml clap downgrade 4.1.0 to support MSRV 1.66.0 ([afc0dab53064bef4aec0f5181e25b8f96e0169f4])
- (LIB) Cargo.toml remove dependency `backtrace` ([b339c881ae94eb1b14c02462042c4c8e8416e951])
- (LIB) Cargo remove flamegraph as a dependency ([f08dd9c4bdc950c70d380d0a98c9546d8efd8c00])
- (LIB) dependabot: bump bstr from 1.6.0 to 1.7.0 [(#199)] ([cff93366ae59c85d01f5d818ea2e8c8c73cedb87])
- (LIB) dependabot: bump bstr from 1.7.0 to 1.9.0 [(#237)] ([a09e9f660ce2de3327a34879a5e184b3ef91a79e])
- (LIB) dependabot: bump clap from 4.3.12 to 4.3.19 ([2c1a38eaca7a66c54938c66abd046fc21e34b58e])
- (LIB) dependabot: bump clap from 4.3.19 to 4.3.21 [(#172)] ([1c6ccb188129272267fd14d0f16fff42f67d81c6])
- (LIB) dependabot: bump clap from 4.3.21 to 4.3.23 [(#175)] ([0f4521cd7cfe059fddab74cfd29f7920d6070ad7])
- (LIB) dependabot: bump clap from 4.3.23 to 4.4.6 [(#193)] ([50e3c5235ab8aa95ffe58b7114bdf257d4bdeff5])
- (LIB) dependabot: bump dlopen2 from 0.5.0 to 0.6.1 [(#166)] ([f42039339adc2bbb24d983232ba5c9f52cf03316])
- (LIB) dependabot: bump encoding_rs from 0.8.32 to 0.8.33 [(#177)] ([bdafe96f41ea33ec27a840dbda74ed909f6f7532])
- (LIB) dependabot: bump filetime from 0.2.21 to 0.2.22 ([0c6ba914d48380fae289077ea08b282484e075b5])
- (LIB) dependabot: bump flate2 from 1.0.26 to 1.0.27 [(#176)] ([53e8a75974dfe4bf11740ad80c0fe769dfa0ebdb])
- (LIB) dependabot: bump itertools from 0.11.0 to 0.12.1 [(#246)] ([e37557772d193a2e812598ed06ea0ab8656dd293])
- (LIB) dependabot: bump libc from 0.2.149 to 0.2.153 [(#247)] ([24691784f79d33a5dd5497f53e064302dfb161d3])
- (LIB) dependabot: bump lru from 0.11.0 to 0.12.0 [(#197)] ([08fed12cf9f2e7a4003a02d2a3e3efecddf49c80])
- (LIB) dependabot: bump memchr from 2.5.0 to 2.6.4 [(#195)] ([8123ef1843ffed2f79a403105d3bdc819c9bb0ba])
- (LIB) dependabot: bump nix from 0.26.2 to 0.27.1 [(#178)] ([68583d84a3722a27bec69a77984cd9e1167929bc])
- (LIB) dependabot: bump regex from 1.9.1 to 1.9.3 [(#169)] ([ddc2a8036d9ea83b75bdc5cf506c365f5b09a3a7])
- (LIB) dependabot: bump regex from 1.9.3 to 1.9.4 [(#181)] ([eb5a1c333d495d5ffdd95c390992de1e2a26e92d])
- (LIB) dependabot: bump regex from 1.9.4 to 1.10.2 [(#203)] ([a01e57d237f55bba7e9541559c7bc0b6286cf8c0])
- (LIB) dependabot: bump rustix from 0.36.9 to 0.36.16 [(#200)] ([3e70a605e7642596337bc49deda5a542c75aaddf])
- (LIB) dependabot: bump tar from 0.4.39 to 0.4.40 [(#173)] ([e3b58f922ce8b312ee1fc9b04b39e5dcf75cf1c4])
- (LIB) dependabot: bump tempfile from 3.6.0 to 3.7.0 ([5543ff7204b50723dbdcd9042bd9747b74821bfb])
- (LIB) dependabot: bump tempfile from 3.7.0 to 3.7.1 [(#168)] ([46c1502f07d0481c3aaab5e05899296f25c6ea13])
- (LIB) dependabot: bump tempfile from 3.7.1 to 3.8.0 [(#174)] ([5d5ce99c92198d0e843259c1d32f08ba87d0039b])
- (LIB) dependabot: bump webpki from 0.22.0 to 0.22.4 [(#204)] ([6671e40e458f0068097135fb37f7f5a279367396])
- (LIB) dependabot: bump zerocopy from 0.7.25 to 0.7.32 [(#249)] ([f3749f5ba0323c8b5c685ff5bed0b63f472be3e9])
- (LIB) src/ remove debug prints with addresses [Issue #213] ([fdc1899ac00ddde0355f09a5c6aaf6d79a1aeec7])
- (LIB) fixedstruct.rs swap numtoa for lexical ([4e32f5ceab4b77a533fcc62ea68377d209b7a282])
- (LIB) src/ NFC code comment updates, docstring links ([bd9b494e083a2861f8c991cfe75f80f61d72ddef])
- (LIB) read gz files sequentially [Issue #182] ([02261be4a6779a73ee08a3075bccb5effc31818f])
- (LIB) blockreader.rs docstring tweaks BlockReader ([4b80fa51f0034f4adf03fad5fc66329e23602f07])
- (LIB) blockreader.rs don't panic for missing Block, return Error ([7f9535da6c2513ffa99d5b4888864d0c911000d6])
- (LIB) blockreader.rs NFC debug print mtime() [Issue #245] ([daded23ce694301aadd19c09a07bb1d384668ce9])
- (LIB) datetime.rs remove to_byte_array; use built-in const bytes ([699015e02fa89058bb1379f5944bde296b0603e6])
- (LIB) syslinereader.rs InvalidInput error tells short line length ([6c825065687ef0469d4a4d1a64b9ef9e75e9ebea])
- (LIB) syslogprocess.rs stage runtime check is debug-only ([463d93f309ab8a320b53711f67a52072566de69c])
- (LIB) syslogprocessor.rs Year min_diff is global static ([1e552fe9a673dc759b583ff1c434b00385015025])
- (LIB) syslinereader.rs use stable feature `pop_last` ([4268bc6bc4657036534627f43966754a4a419ed5])
- (LIB) syslinereader.rs fix mixup summary regex_captures_attempted get_boxptrs_singleptr ([f94d2a0dddd5aa8afe73eb06963af6c3b40e3b01])
- (LIB) syslogprocessor.rs utmpxreader.rs set_error prints error ([4370adae232ecac190d46a6828e6a7661cc0d96e])
- (LIB) src/ cargo fmt suggestions ([8fc4181814ff995835076b5ad2dbb77492c52e6a])
- (LIB) journalreader.rs explicit static ([97b69b23cba21ec59e6a30a5e1fc1d6a642fccda])
- (LIB) codespell fixes ([8f0eee74270820d5b04eb0c6f48934969ed5bc4c])
- (LIB) src/ tweak drop_block return value logic ([c240b82c292586648b2dbd345eba39d716ddd43f])
- (LIB) sd_journal_h.rs allow(clippy::all) ([487138fca7e5cac02347e8597f90b9e1237bd531])
- (LIB) line.rs sysline.rs debug print no address ([e083e73e8fbdeab7c9421e729521c08bd9c77fbc])
- (LIB) src/ refactor error messages: include paths ([c83e433c50687c9611cb298e64823ba9a2dcec6a])
- (PROJECT) logs truncate large logs, rm very large .journal ([9c4e8e9b5006801ea8310baad780daffe6a7e0a9])
- (TEST) datetime_tests.rs use test_case::test_matrix ([a5ad717dc7d37586785b7375068defe352927e24])
- (TEST) tests strategic skip journalreader utmpx on macos windows ([5618d051819a9874a5db33747523553ba1f906c9])
- (TEST) syslinereader_tests.rs fix test ntf_gz_8byte_fpath ([58d9d79a5482fa0d1a555623e33f588de9665bbd])
- (TEST) src/ fix code and tests that fail on target_os="windows" [Issue #202] ([a7cfd10ee270b2ed0c25a952c83b8ffe7235ea02])
- (TEST) tests/mod.rs utmpxreader_tests, utmpx_tests not on "macos" [Issue #217] ([2c8bf0f1bf9c7237b20849b195c52f926c0e43ff])
- (TOOLS) log-files-clean*sh less aggressive cleaning ([2c34a47fc1adc5d59ce6edd35b377659084f4819])
- (TOOLS) flamegraph.sh modify SVG title with more info ([04558a3054bc365543fcc40e33123248cd49f66e])
- (TOOLS) flamegraph.sh xmllint ([2a2d372e0c7bccb98ae3577a256caa710bff2e7d])
- (TOOLS) compare-*.sh allow colordiff ([1d6bc01b3f26c8362f08a4adc73c24ae5b968d8e])
- (TOOLS) hexdump.py fix offset option, allow hex ([fcfc7294018d7e2e559b42be2f70fd1df853514f])
- (TOOLS) backup.sh allow exported BACKUPDIR ([50cc3c2d086b15af580ec6e190059f2b59c0233c])
- (TOOLS) backup.sh backup flamegraph* releases/ ([4a43a7b33b59744e5b070498613ec1fb61e440e2])
- (TOOLS) compare-current-and-expected use individual logs [Issue #213] ([d0736743fca8af8a4dde7d8317b72de269f5655b])
- (TOOLS) compare-current-and-expected check file hashes, fix stdin [Issue #206] ([34b75c5ba38708d8ed2f63b3135c0b42b57ab065])
- (TOOLS) compare-current-and-expected comment "ERROR: " [Issue #224] ([c7ec4ebfe41385a409265ef9dcb3ff4fa9222b03])

## 0.6.68

_Released 2023-07-22_

[0.6.67..0.6.68]

_MSRV 1.67.0_

### New

- (BIN) bin.rs parse -a -b fractional seconds %6f [Issue #145] ([54b0860302e2b691ef6ca54c1bde09fa97e1e3b2])
- (BIN) bin.rs better pattern coverage for -a -b ([05d974c6c4ced7b380343cbff1710e99a2a2ce28])

### Changes

- (LIB) dependabot: bump clap from 4.3.10 to 4.3.12 ([844c1e06cd698eced1c6cd6f50645180b340ee82])
- (LIB) dependabot: bump tar from 0.4.38 to 0.4.39 ([26daee68627b16262717b7091fb192a029896cf5])
- (LIB) dependabot: bump const-str from 0.5.5 to 0.5.6 ([2f9a434cd13200d95d8ce5e5f0a3f8af1b822a92])
- (LIB) dependabot: bump lru from 0.10.0 to 0.11.0 ([1e3e789ba02d8378d590f61487c0beff5bb39d4f])
- (LIB) dependabot: bump bstr from 1.5.0 to 1.6.0 ([6f36d21e829ced48a2de9dc1ee6ed4e51b02aa78])
- (LIB) dependabot: bump itertools from 0.10.5 to 0.11.0 ([d205c419687d0908828ff4f06f4e56351a7ea2f4])
- (LIB) dependabot: bump regex from 1.8.4 to 1.9.1 ([a0edb157905810d46d3418098b829744b3444d0f])
- (LIB) bump si_trace_print to 0.3.10 ([0997ecdc607501cf45e1e8c043210660a290646e])
- (BIN) bin.rs adjust --help wording, spacing ([93a58cc0254e8f4965fe7e3d5cd702489a237ee0])
- (DOCS) README update --help ([9948f0a9035b0883644f0a37d63d16a77158be5b])
- (DOCS) README add color to some other logs ([3805df7db5cc3090568a8ae2316b136f758dc962])
- (DOCS) README coveralls.io badge ([7d97c64e3d28115e395e28c89e56fadc8b26f0af])
- (CI) rust.yml nextest all early on ([b8a8c34b650c47b815fc307346aecc69f35d192a])
- (CI) rust.yml consistent toolchains, use MSRV ([7daa34c8b08e3f3d05aa8257b172d96441015321])
- (CI) rust.yml add coverage llvm-cov to coveralls ([d0f4166b6610b624b6bb2d28a7acba407aea7ca5])
- (BIN) bin.rs NFC clippy recommendations ([3aaddac4d39967807fa2156e11fe5ef31dac8bc8])
- (BIN) bin.rs NFC fixup test case names ([4ac84307c9432001c1b010ff2aafec0be3b2d4cc])

## 0.6.67

_Released 2023-07-08_

[0.6.66..0.6.67]

_MSRV 1.66.0_

### New

- (BIN) bin.rs parse -a -b fractional seconds %3f [Issue #145] ([4b328c5a416cc6daeeeedf81f1dc76bc9bbf849a])

### Changes

- (LIB) dependabot: bump bstr from 1.4.0 to 1.5.0 ([f5bd4e3a8260d8bc5224c5cb851ac0dfe854ee7e])
- (LIB) dependabot: bump chrono from 0.4.24 to 0.4.25 ([bea60aae98c6f7b6ffbb23a30fc58d825397a3e0])
- (LIB) dependabot: bump chrono from 0.4.25 to 0.4.26 ([9522093d0c183c35dec4c457214a219da905baa6])
- (LIB) dependabot: bump clap from 4.2.7 to 4.3.0 ([87a884bbfc07b43cf6b2cf8dadc64eab8bf7a702])
- (LIB) dependabot: bump clap from 4.3.0 to 4.3.8 ([7cc5fbcc0c214c4daedfd3cc447fd788864fd9f9])
- (LIB) dependabot: bump clap from 4.3.8 to 4.3.10 ([d340bd2985295b3ccf4559c4ab1ac3588501ca4b])
- (LIB) dependabot: bump const-str from 0.4.3 to 0.5.4 ([56adf88149e87aebbf87c70fc4531545d2c11daa])
- (LIB) dependabot: bump const-str from 0.5.4 to 0.5.5 ([2030a7392e792e00727f80f9a2d83257b851f519])
- (LIB) dependabot: bump const_format from 0.2.30 to 0.2.31 ([861be6713cfc5f4996251fe23e26f67dd80001d8])
- (LIB) dependabot: bump dlopen2 from 0.4.1 to 0.5.0 ([eeb3632bd465d4937204f1d4c3e5f72a953bcfa6])
- (LIB) dependabot: bump filetime from 0.2.20 to 0.2.21 ([11c9d0bbe8651fb8e057e88166afb450534d03f4])
- (LIB) dependabot: bump flamegraph from 0.6.2 to 0.6.3 ([addbc642be40f93ba3df1588dcb165cbc9b4f0d1])
- (LIB) dependabot: bump libc from 0.2.144 to 0.2.147 ([53596672c4cc4e9b47ee60d4e96af69aeb21d3dd])
- (LIB) dependabot: bump lzma-rs from 0.2.0 to 0.3.0 ([d31c14564c2bc27ed4e7790a54b16d09a01c3be9])
- (LIB) dependabot: bump phf from 0.11.1 to 0.11.2 ([ebf4fd3f494ad12521f1f9ef1d4548282447e8d0])
- (LIB) dependabot: bump regex from 1.8.1 to 1.8.3 ([f4cdd62e98bd1edb356650f70f116c44927f9673])
- (LIB) dependabot: bump regex from 1.8.3 to 1.8.4 ([ee4c9d0c34ef366f008f83767e0b2b88a9e90a4d])
- (LIB) dependabot: bump tempfile from 3.5.0 to 3.6.0 ([ff0f46959254dd193a3b7abb63699ac58106e204])
- (LIB) dependabot: bump test-case from 2.2.2 to 3.1.0 ([2e772970ab86a9541ad56a1702b4a219412ea88b])
- (TOOLS) flamegraph.sh use `--profile flamegraph` ([d4f5e0af96e5ce9d83c12e46a345dc5525d27a95])
- (TEST) bin.rs add 3 more test cases for datetime parsing ([943619bec76c9f49eac11ca7e94543bba2b8d8d7])
- (CI) rust.yml update grcov flags ([707d472928022d51dc7da2fe5322194928871f5b])
- (CI) rust.yml upload binary for all platforms at MSRV [Issue #152] ([6ff633ccb93a9e75e0e0b7291a2571921d85092a])
- (CI) rust.yml expand version matrix ([597c0807e426e2d17f6a6b49a37665899b6bc074])
- (CI) rust.yml NFC reword job name ([1677328e42072057bcac2622726bd973255477c5])
- (CI) rust.yml add job codecov.yml validation ([a4eabe11f15c788abeabbad8d11a447a99d3414c])
- (CI) rust.yml fix upload if statements ([a7e93c6068199f6b826e7aa1d21e2397d4c8e390]) (HEAD -> main)
- (CI) rust.yml update action rust-toolchain@stable ([40e428d6022a4b800d96c72b57f46263d3bf3212])

### Fixes

- (BIN) bin.rs fix parsing of example input strings ([99b8c469d683365998e278c50e7a4a400cfc61c6])
- (LIB) summary.rs clippy fix unused warning BLOCKSZ_MIN ([e317c87ece614341553f2d4b7926f1614a1a5b5a])
- (LIB) src/ cargo clippy fix missing ymdhmsl ([b50ce9932b2b8502a113b85714f6ac9564c2645d])
- (CI) rust.yml relax yamllint line-length [Issue #120] ([01b80ffaf666111674a7c11f33d913f8a0118d19])

---

## 0.6.66

_Released 2023-05-13_

[0.6.65..0.6.66]

_MSRV 1.66.0_

### Changes

- (LIB) (TESTS) (TOOLS) improve tests and logs for `BlockReader::mtime()` and files that are `FileType::Unknown` ([b2530a582f9edcab94d80f9e53142ee801c8335f]) ([f5abc7a12684e6ebf12721a64c95e76a7a620c6b])

### Fixes

- (LIB) blockreader.rs fix panic on `FileType::Unknown` in mtime() ([307c86c22c96ca90ca5456e8dcaf6a83534efbf6])

---

## 0.6.65

_Released 2023-05-11_

[0.6.64..0.6.65]

_MSRV 1.66.0_

### New

- (CI) rust.yml add job_yamllint [Issue #120] ([8bdeafddf2131da83ad916da83ddacb27c363132]) ([6e68808588e0bb24fee292f2b236ed4adcbcbfd2])
- (TEST) add logs dtf13a.log dtf13b.log dtf13c.log dtf13d.log ([c3d0621bef3a9d3ca2c3d9967860f839b4389fd6])

### Changes

- (LIB) datetime.rs allow lenient matching timezone ([fe422d64df17d550cac10ae4306b02f5bf99964b])
- (TOOLS) compare-current-and-expected.sh --prepend-dt-format='%Y%m%dT%H%M%S.%9f' ([e5e7f45a1bc577211908f98bc9a9bbbf335cf332])

---

## 0.6.64

_Released 2023-05-09_

[0.6.63..0.6.64]

_MSRV 1.66.0_

### New

- (BIN) bin.rs print datetimes as UTC dimmed ([e51c30f16a3fb478829bade3350a429d54ee3e94])
- (LIB) parse Red Hat Audit logs, parse epochs [Issue #112] ([0fceba274b8dbefb01ed890d3c211fd85211822b]) ([69ef9f7b8d04b0afa5885040b51ef50c18873fea])
- (BIN) (LIB) src/ Summary statistics for regex capture attempts ([281adc0d2ebea05a6f47fca2ccabffe865295c16])
- (TOOLS) yamlllint.yml add rules for yamllint.sh ([fa5ff7329049623be8379968adf2946360a780cb])

### Changes

- (LIB) dependabot: bump clap from 4.2.1 to 4.2.7  ([33447dd116c091bd968eedf78675dc8c94b46982])
- (LIB) dependabot: bump crossbeam-channel from 0.5.7 to 0.5.8 ([06640e3218bbbe8bdf97c9a54907fcb1a9491876])
- (LIB) dependabot: bump libc from 0.2.141 to 0.2.144 ([8e98a8f387132a3a13f53d359086a80caa484cfd])
- (LIB) dependabot: bump lru from 0.8.1 to 0.10.0 ([75f7c9fa0fdb16e471281c701b71759e728df81d])
- (LIB) dependabot: bump regex from 1.7.1 to 1.8.1 ([66414e9db930cd116e78a692fa0590a3f574aea2])
- (LIB) dependabot: bump tempfile from 3.4.0 to 3.5.0  ([210f01c36f0e7b8415ae595fbda857cff44277fb])
- (BIN) bin.rs refactor channel data passing  [Issue #104] [Issue #60] ([0ea897a7665eff58d9c148ee53559504301e4a52])
- (LIB) journalreader.rs efficient key tracking in `next_short` [Issue #84] ([781063204d0437481e6033a3f1cf5c6c66db102f])
- (LIB) (BIN) miscellaneous codepspell fixes ([524e269e8b6584fdcd60ff551a4f0a5d49e7384e]) ([0c6af5d6d031fd90fd472452bd42ddffab313da4]) ([5bb8a5d1c4331d8e4b0391509abae2277012215d]) ([860b213f7690873f076c098c74b83bb8822a1ba9]) ([af93d662852bbed6a3c13ca4f54ae4a63af56c20])
- (LIB) datetime.rs remove duplicate enum `DTFS_Hour::Hs` ([cc1cb8aa305b3dc17f9df7c0ad8c898bc931b0c2])
- (LIB) syslogprocessor.rs add `blockzero_analysis_bytes` ([cdd64dfe9773aa85ccdcf1099290b273519169d6])
- (TEST) datetime_test.rs test builtins using slice ([b8989f3f0e848138b6de90b81b2c774e775a015d])
- (TOOLS) compare-current-and-expected common args, more args ([d395d94cddeea82f7117682882407feb35258fad])
- (TOOLS) compare-debug-release.sh pass more args to s4 ([dfab1e709a370d468ffb3540f3c6d3e280e97017])
- (CI) github add `dependabot.yml` ([877177bc4a0ca42544ece0facd2f40273b86c239])

### Fixes

- (LIB) syslinereader.rs fix sort of indexes ([f1baa4d5f07e31c179c983a0b855cbc240903859])
- (LIB) datetime.rs fix too short slice recommendation ([2af24cbfbb1645e2cd364a9ab4434e0892619939])

---

## 0.6.63

_Released 2023-05-01_

[0.6.62..0.6.63]

_MSRV 1.66.0_

### New

- (LIB) datetime.rs match single-digit days and hours [Issue #98] ([830dbbd5e18ad8d53727026536b1b07c58411c35])

### Changes

- (LIB) datetime.rs use compile-time map timezone names to values [Issue #84] ([98ebe687ce608c985a5bce2d3e9410fa234a931a])
- (LIB) syslinreader.rs pre-create FixedOffset strings [Issue #84] ([3b950014411d743e3e5527f652e5a2d4aff9a847])
- (LIB) datetime.rs support RFC 2822 timezone "UT" ([dd8248c388a6f8df54c12f5dd010de613a0e21ee])

### Fixes

- (LIB) datetime.rs fix missing and out-of-order timezones ([cf9153bc3cec7f038ae47397c9d0a9942d5f364e])
- (LIB) src/ allow `FileType::Unknown`, fix panic on Unknown ([2cb0412d714078b17402d5bcfa2b1175f4f71bb3])

---

## 0.6.62

_Released 2023-04-27_

[0.6.61..0.6.62]

_MSRV 1.66.0_

### Fixes

- (LIB) fails to build on Debian 11 aarch64 [Issue #108] ([67cb45a47f6c277bc0afc9ac9689b2a05d7b5049])

---

## 0.6.61

_Released 2023-04-23_

[0.6.60...0.6.61]

_MSRV 1.66.0_

### New

- (LIB) filepreprocessor.rs handle trailing junk like "~" ([23dfeb32d0a9d8a7b272ef748fca9b8556b5b0c1])
- (TEST) add `summary()` tests for various readers ([57e2a4d7c2b8a169d83f52162b87c52e09d23f67]) ([8be5e30f4b82fc97cb03e05d086412e050b333db]) ([efb694dfe1f34fee33210b9b5e3a749cb9468be4])
- (PROJECT) add logs `SIH.20230422.034724.362.1.etl` `FreeBSD13_utx.lastlogin.utmp` ([ce8518ead73f36025e708c567cfaa1d9d74d5f2c]) ([ccd4f0c94c53343c239d61e5cee680e7df8d312b])

### Changes

- (TOOLS) journal_print.py allow user passing fields ([3c5a18a47f168dfc463411e81b07f3250ba68df0])

### Fixes

- (LIB) s4 panics on some .etl files [Issue #105] ([5f77a0f7ddbc194ffdc1e45556e2c85910002af6])

---

## 0.6.60

_Released 2023-04-22_

[0.5.59...0.6.60]

_MSRV 1.66.0_

### New

- (LIB) (BIN) (DOCS) (TOOLS) (PROJECT) (BUILD) systemd journal parsing [Issue #17] ([3a6eac6bab6e45b5cb413176a614cb329c4d3f67])

---

## 0.5.59

_Released 2023-03-31_

[0.5.58...0.5.59]

_MSRV 1.66.0_

### Changes

- (LIB) Efficiency hack EZCHECK12D2 ([6f7831f10b187cb72f0ec7568db8ae9c8482a146]) ([08738c41a371749b9aac26c0ab319129d8be0c9f])
- (BIN) bin.rs summary spacing, linerize About section ([dc7b7c27f7d239fcf02d78981ea13a5563c88f88])
- (TOOLS) compare-current-and-expected-update checks stdout and stderr ([f5f2be2dd7d45cf1cc4df2638b6ec3e98a0075b3]) ([cf91c1d2808a7658da8eb6263c3aca0ff3e5fb04]) ([85d5ba25c3b919f1c4b1159630de4702e126d5a9]) ([f5bf771e6f26407fd2066f4765193adb250955c9]) ([52777c1eb5ff968430cb678630f01a100763b967]) ([3df00ac9e826042b31d9617d81f54df998525031]) ([05f04e30dbf5985f01dabc1daa2fa36d10e900a1]) ([81f94b8ba9c8e0d35fddd828b1a1c4f10a9202bc])
- (CI) split up jobs into more parallel jobs [Issue #63] ([2edda45071e3593c83d16514bcfa2a81192a6d35]) ([c1262d43fcdfdf9b2d3604786757bdf3a8ed77cf])

### Fixes

- (LIB) datetime.rs fix one-off error in slice_contiains_X_2 ([d159553e8eaf2166c6d3b6187c007ad3dfc21400])
- (DOCS) src/ fix various docs, pub some functions ([476ed604e7b4201efe5b6e5f7c4a588c3abaa157])

---

## 0.5.58

_Released 2023-03-29_

[0.4.57...0.5.58]

_MSRV 1.66.0_

### New

- (BIN) Allow user-passed timezone for prepended datetime [Issue #27] ([630b8ce945dd2f87d88c357afec26a0a5bdbed60])
- (PROJECT) add logs/programs/{AWS,Microsoft IIS,apache} ([ee4515f1fd7e5161b5eab5bce0262971996f843f])
- (LIB) Parse RFC 2822 [Issue #29] ([38d1c47305125d9bd4e9275ef99d9767af3f1380])

### Changes

- (TEST) add test test_PrinterLogMessage_print_evtx ([e6931ed967f7ea795ecdecfaeeead533642445f5])
- (BUILD) clap 0.4.21 ([f8f977d1bde282c350758aa2ebcca56eaef81c4a]) ([8f7509161b267921fa4f4703c57280e6f1ede86f])

---

## 0.4.57

_Released 2023-03-26_

[0.4.56...0.4.57]

_MSRV 1.66.0_

### Changes

- (LIB) Print evtx files in chronological order [Issue #86] ([e42d021c20b90e50c464541fb3d358ac24ce3b3a])
- (BIN) bin.rs summary linerize more ([cebf2818fe60d8509af94fe623bf0d1f7ff44b17])
- (LIB) datetime.rs NFC refactor dt_pass_filters ([bbdb2cb61fabac44421596f4e3c64e725532e5c7])
- (PROJECT) CHANGLOG revise h2 naming ([ef80a0d5d844ce2b8ec80391305f0b71fc18b518])

### Fixes

- (BIN) bin.rs tweak --help wording ([3c34d099f162ee65423dbee77946622b391955a3])

---

## 0.4.56

_Released 2023-03-24_

[0.3.55...0.4.56]

_MSRV 1.66.0_

### New

- (LIB) process Windows Event Log evtx files [Issue #86] [Issue #87] ([368eba9b473b0c31ebd232bd89bc2aabd5a15d53])

### Changes

- (TEST) add various `Summary` tests ([7f3911b07cc4788fe2cdb4e8d421fe5f156cac59]) ([9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d]) ([5b2b6f808c100077fc94c7019821a01896f7b652]) ([0923408bff8036c1b1c37bfba0a71012845c0935]) ([d03737cda4c53aba353a32f33fd32f7fa74738ad])

### Fixes

- (BIN) bin.rs fix -a "" (passing empty string) ([d4ed03e046d292888e555de3b6955b396ef7fad0])
- (BIN) bin.rs fix panic for multiple non-existent paths ([da980d8bdcb4ac506db0862b11987de8eb859179])

---

## 0.3.55

_Released 2023-03-18_

[0.3.54...0.3.55]

_MSRV 1.66.0_

### New

- (LIB) syslinereader.rs track EZCHECK12 use ([29072ac5c184215f8c10547e5019bf1845864296])
- (LIB) datetime.rs add patterns derived from hawkeye.log  ([d091a792aa369ea4bff566bd321a4a9c9cbb589c])
- (TOOLS) add tools/cargo-outdated.sh ([44fa812ad1f50f90cf5fcf88603fad3a44d09783])
- (TOOLS) add tools/cargo-upgrade.sh ([cb698ec9a74329fc7947db489769472f0cbaf4e8])

### Changes

- (LIB) datetime.rs use slice as ref TryInto ([b723fed816b98dc1bfa9484909c53a8078a1335d])
- (BUILD) Cargo cargo update and upgrade ([f2199b30ca34e9d46d1e51436b2cfba7c9b2f64c])

### Fixes

- (BIN) bin.rs fix --separator, use for utmpx ([24f00e77839701e01123b61e4d7daefcab264a9b])
- (LIB) utmpxreader.rs fix errant println! ([50fec201f1094269a1dc53bca88b25e33d7ceec4])

---

## 0.3.54

_Released 2023-03-17_

[0.2.53...0.3.54]

_MSRV 1.66.0_

### New

- (BIN) use shrink_to_fit on maps [Issue #84] ([e9b501fc77259d0c1c050bedc5a61c3516e4c307])
- (BUILD) Cargo.toml improve release optimizations ([06e500f1d0148e0f9b50ab5907d7f6103533d5f7])

### Fixes

- (BIN) bin.rs user-passed %Z for filters [Issue #85]([98c4b362743dbf5b5ef95234caa389e74dcac1ac])
- (LIB) fix FreeBSD compile of `uapi` and define `umptx` ([62e89e2917a36d73110a860a5f490e4fbb19a6b2])

---

## 0.3.53

_Released 2023-03-10_

[0.2.52...0.3.53]

_MSRV 1.66.0_

### New

- (LIB) add support for utmpx login records (major refactoring) [Issue #70] ([b227f531a6f348cdd9b3fa5fe010adf979dd8e98])

### Changes

- (BUILD) MSRV 1.66.0

---

## 0.2.52

_Released 2023-02-15_

[0.2.51...0.2.52]

_MSRV 1.64.0_

### New

- (LIB) datetime.rs add format catalina apache access  [Issue #82] ([5337dd907a456236ebd038f7b3df6fa4e1687a68]) ([997a365d6a6c72f8a3e847f1c253b1f236f05a5f])

---

## 0.2.51

_Released 2023-02-09_

[0.2.50...0.2.51]

_MSRV 1.64.0_

### New

- (BIN) bin.rs option --sysline-separator [Issue #80] ([b6d359fe3efb94ba8f85c7eaa1788665c392021d])
- (BIN) print bytes counts in hex ([e46b1f943753dc0a5bf1b45b458f0fde643ebdf5])
- (LIB) datetime pattern for tomcat catalina [Issue #81] ([8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f])

---

## 0.2.50

_Released 2023-01-29_

[0.2.49...0.2.50]

_MSRV 1.64.0_

### New

- (BUILD) github code coverage --all-targets [Issue #77] ([6baae7cc71bf42de7584025bf53843f3c0ff8f6c])
- (TEST) printers_tests.rs add initial tests for printers.rs ([5cabf7b91b44fb508cbb90ea8299fd78088323be])
- (TEST) syslinereader_tests.rs add basic tests SyslineReader::new ([0bee4492533b7a88dfb43a9965b9026bcdefc705])
- (TEST) tests/common.rs add eprint_file_blocks ([361e986710d8c97932b87bffc096e6af122ef58e])
- (TOOLS) add valgrind-massif.sh ([84f30592bad9b4395dc770d44dc807125d2ced02])
- (TOOLS) add heaptrack.sh ([749f8ce7aab2be9f0bf16133127a0d7fde3046c3])
- (TOOLS) add s4-wait.sh ([e1dfb0a281d3d922ada33f53013accb2c765bd9d])

### Changes

- (BUILD) bump MSRV to 1.64.0 ([ac5749d32d335f800fc8f3636cfecd321ebefb42])
- (BUILD) rust.yml remove build 1.68.0 ([aee27e45bc52c5a6839a66266d03a304d2608351])
- (LIB) bump si_trace_print = "0.3.9" ([9055612289c8499748001d18c2a232cbf23fe30f])
- (LIB) syslinereader.rs add ezcheck12_min ([713bb7354358091926e524d3f29330f16da3646e])

### Fixes

- (BIN) bin.rs bounded channel queue [Issue #76] ([ac509a2c0f12541ac4db4107a423ada59732c4dd])

---

## 0.2.49

_Released 2023-01-26_

[0.2.48...0.2.49]

_MSRV 1.64.0_

### Changes

- (LIB) src/ refactor find_line_in_block find_sysline_in_block partial ([cda6e991ea22413221c90eacd9b5a16c875ef316])
- (LIB) rust.yml remove build 1.68.0 ([aee27e45bc52c5a6839a66266d03a304d2608351])
- (LIB) syslinereader.rs add ezcheck12_min ([713bb7354358091926e524d3f29330f16da3646e])
- (LIB) debug/printers.rs improve buffer_to_String_noraw improve `buffer_to_String_noraw` to be more robust ([0c7efef500543e3176b1538c90065cad3d624c50])
- (LIB) src/ refactor patterns analysis [Issue #74] [Issue #75] ([8575cd87bd06ba3ad185a1be33aadd4022bbae40])
- (LIB) src/ revise all si_trace_prints calls ([df628a72730e677e16a3053988983f752d71940a])
- (TEST) debug/helpers.rs add create_temp_file_data ([e3c0e0a430d6e27060b00db05c17f01d68361547])
- (TEST) syslogprocessor_tests.rs add tests for short files ([d48099c07d95b49914e4e4271679b4846dd6b608])

### Fixes

- (LIB) syslinereader.rs fix one-off error in get_boxptr ([619da4158649e2fc038bc0ecb9b36e82931508b6])
- (LIB) syslogprocessor.rs fix expected sysline count in blockzero ([e3ca0e225065cf4fe610fd0f49748dc8cab48f71])
- (TEST) common.rs fix PartialEq for FileProcessingResult ([85d51b6b68b108f4a7c8cb9455961420c2cfff43])

---

## 0.2.48

_Released 2023-01-15_

_MSRV 1.61.0_

### New

- (BUILD) rust.yml build on more versions of rust ([c80a0449b8729d3c64775e56de8fe27f21017c6f])
- (LIB) datetime.rs add more general purpose matches ([d6bb2d1da026c16c4a301fa675653d8a0688a679])
- (LOGS) add logs/FreeBSD12.3 ([9128a71a8131362709c35d506cf413db5b0bda00])
- (LOGS) add logs dtf12-*.log ([715cff55bf0dd38c2538a3a522fa7503f2e86ec1])
- (LOGS) add logs/programs/ntp/ ([8fbb9f8e4a058e79ebd9ea45752c62133c14cac8])
- (TEST) syslinereader_tests.rs add basic tests SyslineReader::new ([0bee4492533b7a88dfb43a9965b9026bcdefc705])
- (TEST) syslogprocessor_tests.rs add tests for short files ([d48099c07d95b49914e4e4271679b4846dd6b608])
- (TOOLS) add cargo-call-stack.sh ([6b47b9c1bc8b1cf297c987b3d4321cfe654238f5])

### Changes

- (BUILD) rust.yml explicit shell, print version more often ([9ba41e3564b3058b238f0a05787373f788583b6e])
- (DOCS) README fix shields link for github ([08d198ae57fc5b97013bdda5e883d7df383755f9])
- (DOCS) blockreader.rs doc string links ([7d8d35aa8649386937ec73db7b20ea67eb7bd54f])
- (LIB) src/ revise all si_trace_prints calls ([df628a72730e677e16a3053988983f752d71940a])
- (LIB) debug/printers.rs improve buffer_to_String_noraw ([0c7efef500543e3176b1538c90065cad3d624c50])
- (LIB) syslinereader.rs debug print DTPI attempts ([35fbb1dade0bbfd40042b5154430df5754caa92e])
- (LIB) datetime.rs add test for a DTPD ([d9f70ce89f21bc8e48184856257fddac0a0372e1])
- (LIB) datetime.rs refactor DTFSSet::fmt::Debug ([0d9d80be29fc5051429cf53924d4a7ac3f6010a7])
- (LIB) datetime.rs more precise syslog matching, tests, notes ([7db097f35da98d6166b671a714d7c307b4f8958f])
- (LIB) syslinereader.rs manual pop_last ([c225eb65b2330d6f61580c37504421144308febc])
- (TEST) tests/common.rs add eprint_file_blocks ([361e986710d8c97932b87bffc096e6af122ef58e])
- (TEST) blockreader_tests.rs add basic new BlockReader tests [Issue #22] ([d3f723ed85b0c433c1c6c0a424ccf33ccb11a17d])
- (TEST) linereader.rs add basic new LineReader tests ([b7a25d0905f7aa8426eb97ada89a516620d81e77])
- (TEST) debug/helpers.rs add create_temp_file_data ([e3c0e0a430d6e27060b00db05c17f01d68361547])
- (TOOLS) cargo-test.sh remove --test-threads=1 ([308628ccfa8cef32aa093817b78983739f52548f])

### Fixes

- (LIB) src/ refactor patterns analysis [Issue #75] [Issue #74] ([8575cd87bd06ba3ad185a1be33aadd4022bbae40])
- (LIB) src/ refactor find_line_in_block find_sysline_in_block partial [Issue #22] ([cda6e991ea22413221c90eacd9b5a16c875ef316])
- (LIB) common.rs fix PartialEq for FileProcessingResult ([85d51b6b68b108f4a7c8cb9455961420c2cfff43])
- (LIB) syslogprocessor.rs break when fo_prev >= fo_prev_prev [Issue #75] ([822599689d7cef3844b5b602352e3e18197a00b7])
- (LIB) syslinereader.rs fix one-off error in get_boxptr ([619da4158649e2fc038bc0ecb9b36e82931508b6])
- (LIB) syslogprocessor.rs fix expected sysline count in blockzero ([e3ca0e225065cf4fe610fd0f49748dc8cab48f71])

---

## 0.2.47

_Released 2023-01-09_

[0.2.46...0.2.47]

_MSRV 1.61.0_

- (BIN) bin.rs fix typo in clap help ([b03da48883f07bd1e089f080dc4bc6fa9cfc8578])
- (DOCS) README update --help ([cdaad462bfea78e0e079853e198a32ec89a5d7bc])

---

## 0.2.46

_Released 2023-01-09_

[0.1.45...0.2.46]

_MSRV 1.61.0_

### New

- (BIN) bin.rs print canonical path ([9ceb5b48698e16b62a380d2c1f577f54156c4ac2])
- (BIN) bin.rs use verbatim_doc_comment for all args ([a88c662583ec0222b1842048b0d2f021582ebb6f])
- (LIB) datetime.rs refactor day matching [Issue #58] [Issue #42] [Issue #47] ([1e58094eafae95c9c09b35c63aa000a0edfd5845])
- (LIB) datetime.rs support synoreport.log [Issue #45] ([7d6d9f2d701713046452cae3eb740a7ea6c2ea59])
- (LIB) datetime add pattern for pacman.log [Issue #41] ([989ecdd98ce86d9e4156dbd693c067f9a185a8ba])
- (LIB) datetime pattern for explicit syslog format ([ef73bd5c114916a2f430dcd9c26eb49ec98f3fcc])
- (LIB) datetime.rs add Windows 10 ReportingEvents.log, fix missing tests ([50870c1bf3cca434ec2bd03624fd690fa59dd588]) ([c4c7f3014b51280932244d5c132031f23642cf79])
- (CI) rust-workflow.sh add MSRV verify ([6e284ff06182c3f684c16d49c6bfba8795a862b6])
- (CI) github add task compare-current-and-expected.sh ([133cb5c7dcab6f018c0422bde1f8ee6f9a304258]) ([82255c0ccbe5a57f2906ebb5626b75047f1ce20e]) ([26ec11b7fff8c478b4aa48ed1a4cec01b683a318]) ([b1dc6f927b19e6f1d722454b6792d467834096df]) ([ee95b6364d51c7d8a6bd4259ceda8ec63d13f56b])
- (CI) rust-workflow.sh run compare scripts ([3ac5374edd67a53e0c1492e487db90e9d36a91fd])
- (CI) github run log-files-time-update.sh ([705dd66a0d7771b67d2d1b57de9619cd969939f7]) ([69abc77c352e813dc24128e9952da72c77979f1a])
- (TOOLS) add tools/compare-debug-release.sh ([acc9b5b8722b130d1551ea716628f096e7805c9b]) ([5e975901312e13a35d8599fc06bd0536f4c61e9e]) ([b1dc6f927b19e6f1d722454b6792d467834096df]) ([8514bb9e5dd831640c5a05509c67ed7573c23975])
- (TOOLS) tools add cargo-msrv.sh ([e43d48bfd451fe4aac2f90b0e19a357bf5a1c1b9])
- (TOOLS) add tools/compare-current-and-expected ([f0146fc0172a0f95718c22f531d43494740166f7]) ([8c9a919deb3aed74a11f45ca375f28ded421f4c5]) ([c87ceff11912ec3788f390cf454b1a84db5fd8a3]) ([dd63214e4877ab17811d0e1db6867cff6bb72e61]) ([80d06ddf9245d7653827efa9aa8315ed2c634b11])
- (TEST) add logs Windows10Pro/Panther/* ([2975c9af59b515ee71824cd156c0b3b1bfba3f7d])
- (TEST) add log MPDetection-12162017-091732.log ([8e6fc80beb3c1cbc52fcab7bdd8aad57c84806fe])
- (TEST) add logs Orbi/tmp ([19adf7ec9e2a687b6df19d2e3121c2683f3fc840])
- (TEST) add busybox logs ([e1e4606847459e742f9c5e51a860b8903b2bc5ce])

### Changes

- (BIN) filepreprocessor.rs allow crossing filesystems [Issue #71] ([c6f189916c9fc9cbc4f69ea7a42c110497e7e819])
- (BIN) bin.rs refactor summary printing ([5f5606def37b70ac96d7045fa2ee36156b4d4f28])
- (BUILD) Cargo downgrade const-str for MSRV support ([dfd2898d64411f280bbe7d04280a9c73d3a3b310])
- (LIB) datetime refactor ([1e58094eafae95c9c09b35c63aa000a0edfd5845])
- (LIB) src/ refactor process_path ([e80df38691580c8377c5e3fd30a02617765ee69d])
- (LIB) blockreader.rs change tar path separator char ([9a37f841ad435b4c36bf8b4fe93da7645fe61865])
- (LIB) src/ store path in Summary ([34320a79819fceba1810067606990ab35bcf45b0])
- (LIB) filepreprocessor.rs fix unreachable match ([70dcb6e4bebf26ed60cd26df4eb321417f106da5])
- (LIB) cargo fmt, cargo clippy warnings ([09a885de20cffeabbfaae72f2d597e007c9b6593]) ([a25699295eed0a20eeb3571e0c401d4c901928eb]) ([bab5ee53d297fd4d3cb21ce411cef4c01748d082]) ([8a50df10142c2c8d6c81eaabfb10919d1c3efa0b]) ([a8a5f364fc531b08d8f5fb6245d64b0c70ab95ba])
- (PROJECT) add tools/.gitignore ([44bd6b10290eaa4e9ede11765d25eba4b171cbe2])

### Fixes

- (CI) github fix MSRV check ([58d2f2059a7f43f7bdaff90043799e64ede338b6])
- (PROJECT) .gitignore add leading path for dirs ([f355964d7b4c6bcc0d5cd726df4ff360f2adac23])
- (TOOLS) hexdump.py flush stderr, stdout ([bbe3b00626693af8310454616c08b8358fedb042])

---

## 0.1.45

_Released 2023-01-01_

[0.1.44...0.1.45]

_MSRV 1.61.0_

### New

- (LIB) datetime.rs add pattern for apport.log [Issue #55] ([7557a59e99faf297d2055d5d9ea86b4fbfe8ba5e])
- (LIB) datetime.rs add pattern for openftp.log ([Issue #48]) ([fda61f8ffc7ddd95556f4109b9e735cdde2c1b93])

### Changes

- (LIB) be more sure of matching year ([fda61f8ffc7ddd95556f4109b9e735cdde2c1b93])
- (TEST) more stringent and precise DTPD check ([fda61f8ffc7ddd95556f4109b9e735cdde2c1b93]) ([60aa5d1c1e983aad9b0921e3e066935742605b52])

---

## 0.1.44

_Released 2022-12-29_

[0.1.43...0.1.44]

_MSRV 1.61.0_

### New

- (LIB) datetime.rs add 6 DTPD! entries ([d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd])

### Changes

- (BIN) print summary information for files with bad permissions [Issue #69]([3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16])
- (BIN) better align summary output ([f6a72ff1328766f733fe6314ecdbc1429bb57e61])
- (CI) github do more runs of s4 with default blocksz ([48bd3aa9c24f5420f54ffdddabd061ec5a25d55b])
- (LIB) src/ refactor FileType ([d882f968ae9011b112cb8f195171e5357747a6af])
- (LIB) src/ add ProcessPathResult::FileErrNotExist,FileErrNotParseable ([e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a])
- (TOOLS) tools add rust-workflow.sh ([8e3b72a7c1e70dad9eacc62cb3171754799c79a6])

### Fixes

- (LIB) syslinereader do not panic in case of unexpected Done [Issue #67] ([2da339822a4f62266149b8d53925840c0860c9a2])

---

## 0.1.43

_Released 2022-12-26_

[0.1.42...0.1.43]

_MSRV 1.61.0_

### New

- (CI) add codecov.yml ([bd49cdc8220e8adcfea71f04c6ebcfb51946336b])

### Changes

- (BUILD) adjust "filetime" dependency ([ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048])
- (DOCS) update README ([cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870])
- (BIN) update --help for `--blocksz` ([2157861027eff2cade51aa950a6a4300e86a1e50])

---

## 0.1.42

_Released 2022-12-19_

[0.1.41...0.1.42]

_MSRV 1.61.0_

### Changes

- (BIN) (BUILD) update clap from 3 to 4 ([f58f506f17d6b76343d5bd814749259e3b380cc2])
- (BUILD) cargo update ([41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd])

---

## 0.1.41

_Released 2022-12-18_

[0.1.40...0.1.41]

_MSRV 1.61.0_

### Changes

- (TOOLS) gen-log.sh add option for extra lines ([610785f3d98a4032fe7053076f9db45d4c1d1717])
- (BIN) re-arrange trailing summary print of datetimes ([a80facf0bb435346b0c8c3a05d22b8e428ba680f])
- (LIB) for logs without year, skip processing outside filters [Issue #65] ([33418d0311fc75fa7fda97ac621ddf2da493c128])
- (PROJECT) add gen-20-2-2-faces.log ([8a47ad83cd68c7eec60db4ff734f8ead3d54b977])

### Fixes

- (LIB)(BIN) fix drop, add more summary stats, tests [Issue #66] ([916259ba70c903d2b2d85b4bd3eddffa98cec370])

---

## 0.1.40

_Released 2022-11-22_

[0.1.39...0.1.40]

_MSRV 1.61.0_

### New

- (BIN) add CLI option `--prepend-separator` ([467b14dbc59a60a808e7a71a1083f2490cf31d48])

### Changes

- (BIN) add summary _syslines stored high_ ([d1f5895f1e5a55cbbcbfc4072bbde53a7a85fc])

---

## 0.1.39

_Released 2022-10-19_

[0.1.38...0.1.39]

_MSRV 1.61.0_

### Changes

- (LIB) Cargo.toml rust MSRV 1.61.0 ([3c4e8b1b37415ad0662019d1792525ab0b00a8f9]) 
- (BUILD) rust.yml add job_rust_versions, jo_rust_msrv_os 1.61.0 ([94d6862e0d558e69f0e5b07db5a63ad7700d515b])
- (LIB) const-str downgrade to 0.4.3 ([7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13])
- (DOCS) README add more Windows examples, wording ([db5b6a5fbc301716f84682c4dae7e1691fcba413])
- (LIB) (BIN) codespell fixes ([b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5]) ([aaaf78e17cfeeea087fe9562fc65907b3847bc9e])

---

## 0.1.38

_Released 2022-10-16_

[0.0.37...0.1.38]

_MSRV 1.62.0_

### New

- (BIN) bin.rs --summary print -a -b as UTC ([186c74720db2b33e5c0df17ee690eddcdee360a7])
- (BIN) bin.rs allow relative offset datetimes [Issue #35] ([4eef6221708137928458ed8445b4f67196500082])

### Changes

- (DOCS) README add Windows log snippets, tweak wording ([65c007844cc6c275b86b36a2ff1b48340622a681])

---

## 0.0.37

_Released 2022-10-12_

[0.0.36...0.0.37]

_MSRV 1.62.0_

### New

- (LIB) datetime.rs patterns for Windows compsetup.log mrt.log ([0f225cee04b5443a58369b95bc8e6f10ed3f6401])

### Changes

- (LIB) blockreader.rs eprintln path ([3e1607f076afe7a6e10578776a07d3feb0a2b9a8])
- (TEST) add logs Windows10Pro ([1c746c24b7e0ad7e7481cce626fb6488eb0076d6])

---

## 0.0.36

_Released 2022-10-10_

[0.0.35...0.0.36]

_MSRV 1.62.0_

### New

- (LIB) datetime.rs parsing nginx apache logs [Issue #53] ([003b29bab508b32750cb303c70db9dc75cc04eab])

### Changes

- (BIN) bin.rs --help tweak, comment cleanup ([3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2])
- (TEST) add more logs ([9ddaaeedecdd175672c38ba3d39c7521f08acc68]) ([effffe87f8390d5894ab8dcf1806b2dd5b54e493]) ([66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6]) ([0743e4157daa108569d99746d8a6314cfe6e0248]) ([e736a714f4b2a84e4b5d578c8789049c1bbc4df6])
- (DOCS) README changes, docstring changes ([657948516a05c40cd0d9c35dc639d05eeafa5dc5]) ([001f0c3db2c5751a35946f572aca6bf07c9efcaf]) ([4b51b30d598a6e076f3d2a8b9d3e170deea1325f]) ([fda30e592981b402a192fe6f74ac36febdc946c8]) ([8f1437483337f24a4c728b61d1754f9455ee0f5b]) ([44291c749bfae647cf130fdc298dd2cc5d1876ba])

### Fixes

- (LIB) datetime.rs add RP_NOALPHA after CGP_TZZ ([1cfc72e99382ab47b55c9410ab531c0baf8ac46e])

---

## 0.0.35

_Released 2022-10-09_

[0.0.34...0.0.35]

_MSRV 1.62.0_

### New

- (LIB) datetime.rs handle format with syslog levels [Issue #57] ([d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59])
- (LIB) datetime.rs add RP_NOALPHA after CGP_TZZ ([1cfc72e99382ab47b55c9410ab531c0baf8ac46e])
- (DOCS) rustdocs improvements ([1de420a5907cf62ae91a06732a8ef43e01f17598]) ([1b88a1e35a66004ea5016525bcbb1e125aa64db9])
- (PROJECT) README add section "syslog definition chaos" ([ec82a5009bcf7a16aaa694eb478216b9567c87c1])
- (BUILD) Cargo update to latest dependencies ([7185ba477d0d184f9cdf28eb485e3ec4e5963f3b])
- (BUILD) Cargo.toml exclude more READMEs ([bd44896a30627bafefa64c1cbc78229113130b9d])

### Fixes

- (TEST) cargo-test.sh fix --version ([2a1b10859a31649a7ef31db9474e3a6ed526c9a4])

---

## 0.0.34

_Released 2022-10-07_

[0.0.33...0.0.34]

_MSRV 1.62.0_

### New

- (BIN) bin.rs allow user opt dt format string [Issue #28] ([6660f686f02ca2d98c9cdfe3c72cc906e446df1f])
- (TEST) add logs ISO8601\*log RFC3164\*log RFC3339\*log ([3980d5b67bbd371d84cbb313f51e950dae436d54])
- (TEST, TOOLS) log files add systems, cleaning scripts, touch set ([55a1e55ed94f8e8a4202098c1fd4f85e337bfae4])
- (LIB) datetime.rs allow Unicode "minus sign" [Issue #38] ([fc2a8379ad2f848990c749418ebe4123cacbcf8b])
- (PROJECT) README updated Features, Limitations ([0ca431ce8b510b6714420a8954f587eccd84a01d])
- (PROJECT) README README fill section About ([aa3992cc919c644dc7fe3bc41abc2dd970fe3d2e])

### Fixes

- (TEST) cargo-test.sh `rm -f` to avoid possible error during exit_ ([2343d26300c5a139066081648054e5e299eb8a80])

---

## 0.0.33

_Released 2022-09-21_

[0.0.32...0.0.33]

_MSRV 1.62.0_

### New

- (BIN) bin.rs allow user opt dt format string [Issue #28] ([6660f686f02ca2d98c9cdfe3c72cc906e446df1f])
- (DEBUG) use `si_trace_print` 0.2.5 ([fc5482359615f1f1a0d83c4f34a1ca89834d38ff])

---

## 0.0.32

_Released 2022-09-20_

[0.0.31...0.0.32]

_MSRV 1.62.0_

### New

- (TEST) datetime_tests.rs add tests cases datetime_parse_from_str ([c9bc19ecd6cad88742cfa3758e48fd606f489220])

### Fixes

- (LIB) datetime.rs fix copying fractional [Issue #23] ([764689fe0693c6a8588d13cde1c73f42e08b2a39])

---

## 0.0.31

_Released 2022-09-19_

[0.0.30...0.0.31]

_MSRV 1.62.0_

### New

- (DOCS) improved README badges ([30553b7989b55c802704c42deefe9424347092ee]) ([b5d4d91779599bae9fc15d78c5e3db3f4a43f18b]) ([17cd497307d04f3d8a9b058a72e3ea415a9a9f89])

---

## 0.0.30

_Released 2022-09-18_

[0.0.29...0.0.30]

_MSRV 1.62.0_

### New

- (LIB) allow -t %Z or %z, allow Zulu ([a71c5e81761deb547c315296004167e13f82fe9b])

### Changes

- (TEST) add log dtf11-A.log ([56078e8bb713fa861ccf9ebd1a58415ee6173819])
- (BIN) bin.rs stack_offset_set only debug builds ([d2158ee2b1b23a68b3c4dd764863acadec08d6bb])

---

## 0.0.29

_Released 2022-09-17_

[0.0.28...0.0.29]

### Changes

- (DOCS) README update install and run instructions ([01f395903cff248be11ecf6f12974a3951aa7e92])
- (BUILD) Cargo.toml remove unused bench, exclude unused benches ([b088be725c367aabae07d4b60553693a5c2ddd80])
- (LIB) Cargo.toml rust-version 1.62 ([aaf976b84513bcdb2395fab5349fc035e4601068])
- (BUILD) rust.yml add "Check Publish" ([7f751c12debde6b2dcd7377d880b20d2aa834f40])
- (TEST) add logs gen-100-10. gz xz tar tgz ([33a492b9c01c57a71191d7f1b46d457d5ff67059])
- (TOOLS) valgrind-dhat.sh add more logs to parse ([6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7])
- (LIB) Cargo update dev-dependencies ([6805e2b9257cecb545417531a008ec139a0b5c54])

---

## 0.0.28

_Released 2022-09-17_

[0.0.27...0.0.28]

### New

- (BUILD) Cargo.toml rust-version 1.62 ([aaf976b84513bcdb2395fab5349fc035e4601068])
- (BUILD) rust.yml add "Check Publish" ([7f751c12debde6b2dcd7377d880b20d2aa834f40])
- (BUILD) Cargo update dev-dependencies ([6805e2b9257cecb545417531a008ec139a0b5c54])
- (BUILD) Cargo.toml ready to publish ([d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da])
- (CI) Check clippy ([e7172e4519383c352ed147aa42b3aeca646a690e]) ([ac8d29bb53de9b0bc06572f85073a1ac06f54087])
- (CI) rust.yml add grcov job ([fcf91c96ea0dd598594aec0fac23726426b4cd3b])
- (CI) rust.yml add step for "cargo check" ([17f89020870b8bc8ad8322e314c187b6e0836226])
- (TOOLS) add cargo-clippy.sh ([676633a72f464a1f71b369281207390fb1c2efd5])

### Changes

- (TEST) add new test logs
- (LIB) src/ clippy recommendations ([84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b])
- (LIB) src/ mv src/printer_debug -> src/debug ([d70104fb19ee3e133188a14d49f2c57ab0a55e06])

### Fixes

- (TOOLS) tools chmod 100755 ([a13786623e5b9117418dc6ff86c1f0519e9074f0])

---

## 0.0.27

_Released 2022-09-16_

[0.0.26...0.0.27]

### New

- (LIB) datetime.rs allow lowercase named timezone ([cae987706e31a6c223e5af997fee32b537714efd])
- (TOOLS) add cargo-test-cov-tarpaulin.sh ([5f93e4ad56fdbda6b5ceeaeca94848063064cc9a])
- (TOOLS) add cargo-test-cov-llvm.sh ([97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9])
- (TOOLS) add cargo-test-cov-grcov.sh ([55113bc5705d5c9ace1da6bde8b05c1260ddb935])
- (TOOLS) add changelog-link-gen.sh ([0579522ff7609e22c14b33aa6c6a70cec6372226])

### Changes

- (BIN) bin.rs prepend only files with Syslines [Issue #21] ([e189fd21f8689048e404ddf19c279ad743203924])
- (LIB) src/ cargo fmt ([4fda60f22505ebba9ff86873386d0524d364765c]) ([53892a3a2d46c3b7dcad3b0fd7b4141118485e9e]) ([a1e1a680278843d4f871f5556bee679282a8d268]) ([dff2927698abcac250fd3f0df7910c02818f6776]) ([4b784a723b8c02c7bdb4b51e7d7b76147f97d569]) ([78581dba9d33c9565fa25f0a829ca383471335f2])
- (LIB) use crate si_trace_print, remove local impl ([f49fb33dab085714a8050d36442c04bf504f731e])
- (TOOLS) rename compare-cat.sh ([25e4bf65d9d5af300b99092e189f0caea3164f5f])
- (TOOLS) rename cargo-test.sh ([0a5ce1e0011920909cfa5bc022f95b3a502ff244])
- (TOOLS) rename rust-gdb.sh ([f00ebc4ccb3e82ae2d54787d9e39a6bce3044032])
- (TOOLS) rename cargo-doc.sh ([9957cc56452652f87ac037175d3b16f273a735ea])
- (TOOLS) rename compare-grep-sort.sh ([acc34edc4502c691381df03a3bf9c2aebde1a038])
- (TESTS) datetime.rs add begin,end indexes to test cases ([5dfc932d8f62e295f93accafb98c533fd8e39625])
- (LIB) Cargo.* update dependencies ([7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550]) ([d8faf4fd010e303dad42c8a0a51520c03fd197b8])
- (LIB) datetime.rs improve precision of patterns, var names ([238df6c7b1b569f724778c85bfead20cb14be59d])
- (LIB) datetime.rs add x4 DTPI for a pattern lie `VERBOSE Tuesday Jun 28 2022 01:51:12 +1230` ([34595fe2693385b0cdff69ecf6306071d058b638])
- (LIB) datetime.rs match named pattern `<dayIgnore>` ([07214abde6479431cc1a9f87f50f3b713e5ea503])

---

## 0.0.26

_Released 2022-08-12_

[0.0.25...0.0.26]

### New

- (DOCS) rustdocs
- (LIB) add datetime patterns for history.log ([6955a7b5c389a9b16651bf7e2350e12df2bc22a2])
- (BIN) bin.rs allow user to pass only dates ([0a46b5aee7eb99e19a9a2a91ed81d759978b6024])
- (TOOLS) add rust-doc.sh ([f56045aa6c147246f30635240835e92bea224520])
- (TOOLS) add valgrind-dhat-extract.sh ([aa5bdbbcdbd2d36b08f11c0a252603526b7adce8])

### Changes

- (BIN) refactor logic for process return code ([09df0b6551fec2ea22cee7dca2cd308cf11b531a])
- (CI) github tests build documentation ([f7b4533f180ccc94c27f8e42b9806199d147f5c1])
- (BIN) remove unsupported `%Z` in `--help` [Issue #20] ([b55341ecd717344211bd79557f56f7fecaad2479])
- (BIN) bin.rs simplify check for printing error ([79c1ea1edbed94e3376aed37b382d069144d6fab])
- (BIN) bin.rs print trailing `'\n'` if needed ([3c7984d49df0d91037729a45c24a2a7b5a109687])
- (BIN) bin.rs remove panics in processing_loop ([dc30ca638c88714942f282de4cd464336e41f8de])
- (LIB) line.rs printers.rs refactor some slice statements ([aab71b50e4464cae19f1add8b28613260345d9db])
- (DOCS) blockreader.rs NFC docstring about read_block_FileTar [Issue #13] ([a503554d9c0bbae7751b1e448156a7dc43f32def])
- (LIB) filepreprocessor.rs NFC comment [Issue #15] ([ed3d1feb788121161ba66f9c1826a67ded941337])
- (LIB) src/ NFC comment [Issue #16] ([c35066cd2cc01344259f00559186fbd1a12db527])
- (LIB) blockreader.rs NFC comment [Issue #13] ([1c58a778dc5bd05e455ea25af60e8600b8b72857])
- (LIB) blockreader.rs NFC comment [Issue #12] ([bc4112866bb713538fc48c209408313c634306b2])
- (LIB) blockreader.rs NFC comment [Issue #11] ([908b2f594fdbc1aa51313bba5f26db74ee332a4a])
- (LIB) blockreader.rs NFC comment [Issue #10] ([c9dc70a51be61bc46b43082e7227f873cb77ac10])
- (LIB) blockreader.rs NFC comment [Issue #9] ([8cd40b522d2e87dd69dd21704c5f128d6d05847b])
- (LIB) blockreader.rs NFC comment [Issue #8] ([c8328f3ab256bf76a92b205f8eeebc49447bd25e])
- (LIB) blockreader.rs NFC comment [Issue #7] ([943ad6258c6d01c3df3f97e35b7d0a2aa4f00136])
- (LIB) datetime.rs NFC comment [Issue #6] ([af46851919ced5582dd8d6c5b236edd3ac078061])
- (LIB) datetime.rs NFC add commented [Issue #4] ([d8d56414c28f5ca7ba2db10420c1805270d80d7b])
- (BIN) bin.rs NFC link [Issue #5] ([81c437b02b967b56dcb9f5fa0a25b083dfa3ed25])
- (LIB) improve supplementary filetype matching in `path_to_filetype` ([48687e8a65af56cf9c6279702ccaa6a66c127a06])
- (BIN) bin.rs consistent eprintln for failed parsing ([880e35aeedfb5449626a03c9131a1ccd33e017e3])

### Fixes

- (TOOLS) tools chmod 100755 ([a13786623e5b9117418dc6ff86c1f0519e9074f0])
- (LIB) src/blockreader* fix handling zero byte gz, xz ([f708e15eab0ca601699461565b7a396f84394526]) ([21745ee99eb04a4204164825ca5c50e6f8b34fee])
- (BIN) bin.rs fix -a -b parsing ([f6b52fc20a8893ce30443bdd27f8da11108d0e17])
- (LIB) Massive amount of code cleanup
- (TEST) Many more tests

---

## 0.0.25

_Released 2022-07-28_

[0.0.24...0.0.25]

### New

- (LIB) add handling for missing year in datetime ([ab579207ea14141d3d4327f39b5fd23830a89f3a])

### Changes

- (BIN) Fix flood of error messages due to printing failure ([c332a73363492a1e1874e68fc0c12e3bfd2b96ae])

### Fixes

- (LIB) Fix handling GZIP XZ ([8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c])
- (DEBUG) Fix debug print Sysline String with unknown glyph ([b2d6de5072f1506077fa649b15912b7cb3064211])
- (DEBUG) Fix debug print Line String with unknown glyph ([ed5c04ade1af13f2e22afc184336f9713f2b76e0])
- (TEST) Many more tests

---


## 0.0.24

_Released 2022-07-20_

[0.0.23...0.0.24]

### New

- (LIB) handle tar files ([adf400700122f4eb23fd63971b3f048e014d1781])

### Changes

- (LIB) datetime transforms `%b` `%B` to `%m` ([22980abf582aa61c5e4c9ce94d8298997fb5bbbc])

---

## 0.0.23

_Released 2022-07-12_

[0.0.22...0.0.23]

### New

- (LIB) WIP handle tar files ([b8deef3439f8e8b9a949a0a1cfa16d2c027c391f])
- (PROJECT) add CHANGELOG.md ([ca1c967a1dd169b73f3002f120c40c7127060041])

### Changes

- (LIB) add enum_BoxPtrs::DoublePtr ([61f15e13d086a5d6c0e5a18d44c730ebe77a046a]) ([cb74da327e27b73e9724d8a28aafc164e6c9e0df])
- (LIB) refactor to use `regex::bytes::Regex` instead of `str`-based `regex::Regex` ([dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa]) ([3d78b0d0b6918dab784bbe2332b3a26928bb8f90])
- (LIB) refactor name `enum_BoxPtrs` to `LinePartPtrs` ([b5505730100a9780877eb3e1cb4d280f02845863])
- (TOOLS) rust-test.sh use nextest if available ([1bf2784185df479a3a17975f773e3a505f735e26])
- (TEST) faster tests reuse single `NamedTempFile` more often ([db2e8f3cf4db912d32e74fcbdf09094c8b2f5128])
- (CI) github run args change ([a82e25b56c80e37c5ea6450c4a27a9ff1feb021b]) ([c8fc525dff93e1b29c0df61bf6cc593376910043]) ([febfd00d66ac8586584882ec6c7a5b2a97683571])

### Fixes

- (BIN) fix --blocksz minimum check ([07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8])
- (LIB) printers.rs fix macro `print_color_highlight_dt` ([6659509095d19163bd65bd24a9a554cf25207395])
- (DEBUG) line.rs impl LinePart::count_bytes,len ([9d9179cf63c4167ac46b5c398b2c6b718ea9a022])
  Fix `LinePart::count_bytes`
- (DEBUG) printers.rs fix `char_to_char_noraw` ([ced4667fd5f16682a46e70d435a9a473885c70b6])
- (DEBUG) line.rs fix `_to_String_raw` ([d5af77deed057d599fd1c4b5c1f6222a7edba4c3])

---

## 0.0.22

_Released 2022-07-10_

[0.0.21...0.0.22]

### New

- (LIB) refactor datetime string matching ([3562638d37272b2befa7f9007307fd4088cdd00c])
  refactor datetime string matching within a `Line` to use regex.
- (TOOLS) add hexdump.py ([031434f4d9dfb4e0f8190a720f8db57a3772e3a2])
- (LIB) printers.rs highlight datetime in call cases ([a4fd91f4b1340a754754b8bec841eb60102988bf])
- (LIB) printer.rs all color lines highlight dt ([9c5fa576899d1529b06acf89221d44d262092d04])
- (LIB) filepreprocessor also check supplement with ext removed ([0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399])
  During filetype search, also call supplmentary check based on name
  using a path with file extension removed. Allows matching files like
  `kern.log.gz.1`.

### Changes

- (BUILD) remove crate chain-cmp ([7109c46d835f4d6f32b6284681a6286b68179abc])
- (LIB) set `const` for funcs `slice_contains...` ([eeb20bb8431bf75c9e2be3fbba8e64daafae3098])

### Fixes

- (LIB) fix errant �� printed at block edges ([5caa8dd6b7f8f2735366a23ab1005df89aaf565f])

---

## 0.0.21

_Released 2022-06-24_

[0.0.1...0.0.21]

### New

- (LIB) add XZ support ([607a23c00aff0d9b34fb3d678bdfd5c14290582d])
- (BIN) bin.rs set default -t in help message ([e346e184d9ab0af7969a796ef4c43814267aa7a3])

### Fixes

- (LIB) src/ print summary invalid, fix dir walk error ([09a04c14146af1916aeda14e8134d02baf088d5d])
  Print summary of each invalid file and it's MimeGuess and FileType result. This helps the use understand why a file was not parsed.
  (LIB) Fix directory walk in `process_path` only checking the root directoy and giving errant `FileType` to files.

---

<!--
All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](http://semver.org/) and [Keep a Changelog](http://keepachangelog.com/).
-->

This document template generated by [changelog-cli](https://pypi.org/project/changelog-cli/).

<!--
TODO per release: update links (run `tools/changelog-link-gen.sh`)

DO NOT CHANGE THE FOLLOWING COMMENT
EVERYTHING AFTER THE FOLLOWING COMMENT WILL BE DELETED AND REPLACED BY `tools/changelog-link-gen.sh`
-->

<!-- LINKS BEGIN -->

[Issue #4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/4
[Issue #5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/5
[Issue #6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/6
[Issue #7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/7
[Issue #8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/8
[Issue #9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/9
[Issue #10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/10
[Issue #11]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/11
[Issue #12]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/12
[Issue #13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/13
[Issue #15]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/15
[Issue #16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/16
[Issue #17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/17
[Issue #20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/20
[Issue #21]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/21
[Issue #22]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/22
[Issue #23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/23
[Issue #27]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/27
[Issue #28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/28
[Issue #29]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/29
[Issue #35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/35
[Issue #38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/38
[Issue #41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/41
[Issue #42]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/42
[Issue #45]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/45
[Issue #47]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/47
[Issue #48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/48
[Issue #53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/53
[Issue #55]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/55
[Issue #57]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/57
[Issue #58]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/58
[Issue #60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/60
[Issue #63]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/63
[Issue #65]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/65
[Issue #66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/66
[Issue #67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/67
[Issue #69]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/69
[Issue #70]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/70
[Issue #71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/71
[Issue #74]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/74
[Issue #75]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/75
[Issue #76]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/76
[Issue #77]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/77
[Issue #80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/80
[Issue #81]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/81
[Issue #82]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/82
[Issue #84]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/84
[Issue #85]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/85
[Issue #86]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/86
[Issue #87]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/87
[Issue #98]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/98
[Issue #100]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/100
[Issue #104]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/104
[Issue #105]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/105
[Issue #108]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/108
[Issue #109]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/109
[Issue #112]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/112
[Issue #120]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/120
[Issue #121]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/121
[Issue #145]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/145
[Issue #152]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/152
[Issue #170]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/170
[Issue #171]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/171
[Issue #182]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/182
[Issue #201]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/201
[Issue #202]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/202
[Issue #206]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/206
[Issue #213]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/213
[Issue #217]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/217
[Issue #218]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/218
[Issue #219]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/219
[Issue #224]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/224
[Issue #241]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/241
[Issue #245]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/245
[Issue #250]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/250
[(#166)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/166
[(#168)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/168
[(#169)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/169
[(#172)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/172
[(#173)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/173
[(#174)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/174
[(#175)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/175
[(#176)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/176
[(#177)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/177
[(#178)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/178
[(#181)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/181
[(#193)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/193
[(#195)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/195
[(#197)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/197
[(#199)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/199
[(#200)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/200
[(#203)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/203
[(#204)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/204
[(#237)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/237
[(#246)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/246
[(#247)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/247
[(#249)]: https://github.com/jtmoon79/super-speedy-syslog-searcher/pull/249
[0.0.1...0.0.21]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.1...0.0.21
[0.0.21...0.0.22]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.21...0.0.22
[0.0.22...0.0.23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.22...0.0.23
[0.0.23...0.0.24]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.23...0.0.24
[0.0.24...0.0.25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.24...0.0.25
[0.0.25...0.0.26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.25...0.0.26
[0.0.26...0.0.27]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.26...0.0.27
[0.0.27...0.0.28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.27...0.0.28
[0.0.28...0.0.29]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.28...0.0.29
[0.0.29...0.0.30]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.29...0.0.30
[0.0.30...0.0.31]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.30...0.0.31
[0.0.31...0.0.32]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.31...0.0.32
[0.0.32...0.0.33]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.32...0.0.33
[0.0.33...0.0.34]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.33...0.0.34
[0.0.34...0.0.35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.34...0.0.35
[0.0.35...0.0.36]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.35...0.0.36
[0.0.36...0.0.37]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.36...0.0.37
[0.0.37...0.1.38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.0.37...0.1.38
[0.1.38...0.1.39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.38...0.1.39
[0.1.39...0.1.40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.39...0.1.40
[0.1.40...0.1.41]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.40...0.1.41
[0.1.41...0.1.42]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.41...0.1.42
[0.1.42...0.1.43]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.42...0.1.43
[0.1.43...0.1.44]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.43...0.1.44
[0.1.44...0.1.45]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.44...0.1.45
[0.1.45...0.2.46]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.45...0.2.46
[0.2.46...0.2.47]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.46...0.2.47
[0.2.47...0.2.48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.47...0.2.48
[0.2.48...0.2.49]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.48...0.2.49
[0.2.49...0.2.50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.49...0.2.50
[0.2.50...0.2.51]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.50...0.2.51
[0.2.51...0.2.52]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.51...0.2.52
[0.2.52...0.3.53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.52...0.3.53
[0.2.53...0.3.54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.2.53...0.3.54
[0.3.54...0.3.55]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.3.54...0.3.55
[0.3.55...0.4.56]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.3.55...0.4.56
[0.4.56...0.4.57]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.4.56...0.4.57
[0.4.57...0.5.58]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.4.57...0.5.58
[0.5.58...0.5.59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.5.58...0.5.59
[0.5.59...0.6.60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.5.59...0.6.60
[0.6.60...0.6.61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.60...0.6.61
[0.6.61...0.6.61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.61...0.6.61
[0.6.61...0.6.62]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.61...0.6.62
[0.6.61..0.6.62]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.61..0.6.62
[0.6.62..0.6.63]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.62..0.6.63
[0.6.62..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.62..main
[0.6.63..0.6.63]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.63..0.6.63
[0.6.63..0.6.64]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.63..0.6.64
[0.6.63..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.63..main
[0.6.64..0.6.65]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.64..0.6.65
[0.6.64..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.64..main
[0.6.65..0.6.66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.65..0.6.66
[0.6.65..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.65..main
[0.6.66..0.6.67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.66..0.6.67
[0.6.66..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.66..main
[0.6.67..0.6.68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.67..0.6.68
[0.6.67..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.67..main
[0.6.68..0.6.69]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.68..0.6.69
[0.6.68..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.68..main
[0.6.69..main]: https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.6.69..main
[00171bbdf238fd9c1ba6d89fa29a730318332d7e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/00171bbdf238fd9c1ba6d89fa29a730318332d7e
[001f0c3db2c5751a35946f572aca6bf07c9efcaf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/001f0c3db2c5751a35946f572aca6bf07c9efcaf
[0021f0576d0d629c72028443f2a266f957e5b084]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0021f0576d0d629c72028443f2a266f957e5b084
[003b29bab508b32750cb303c70db9dc75cc04eab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/003b29bab508b32750cb303c70db9dc75cc04eab
[007f752fdd69eb37df38171a7e485f5ae026ec6c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/007f752fdd69eb37df38171a7e485f5ae026ec6c
[01b80ffaf666111674a7c11f33d913f8a0118d19]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01b80ffaf666111674a7c11f33d913f8a0118d19
[01f395903cff248be11ecf6f12974a3951aa7e92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01f395903cff248be11ecf6f12974a3951aa7e92
[02261be4a6779a73ee08a3075bccb5effc31818f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/02261be4a6779a73ee08a3075bccb5effc31818f
[031434f4d9dfb4e0f8190a720f8db57a3772e3a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/031434f4d9dfb4e0f8190a720f8db57a3772e3a2
[04558a3054bc365543fcc40e33123248cd49f66e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/04558a3054bc365543fcc40e33123248cd49f66e
[0579522ff7609e22c14b33aa6c6a70cec6372226]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0579522ff7609e22c14b33aa6c6a70cec6372226
[05d974c6c4ced7b380343cbff1710e99a2a2ce28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/05d974c6c4ced7b380343cbff1710e99a2a2ce28
[05f04e30dbf5985f01dabc1daa2fa36d10e900a1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/05f04e30dbf5985f01dabc1daa2fa36d10e900a1
[06640e3218bbbe8bdf97c9a54907fcb1a9491876]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/06640e3218bbbe8bdf97c9a54907fcb1a9491876
[068a0a2893f6007a51aa9e65c997b9e08e72c3a5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/068a0a2893f6007a51aa9e65c997b9e08e72c3a5
[06e500f1d0148e0f9b50ab5907d7f6103533d5f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/06e500f1d0148e0f9b50ab5907d7f6103533d5f7
[07214abde6479431cc1a9f87f50f3b713e5ea503]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07214abde6479431cc1a9f87f50f3b713e5ea503
[0743e4157daa108569d99746d8a6314cfe6e0248]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0743e4157daa108569d99746d8a6314cfe6e0248
[07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8
[08738c41a371749b9aac26c0ab319129d8be0c9f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08738c41a371749b9aac26c0ab319129d8be0c9f
[08d198ae57fc5b97013bdda5e883d7df383755f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08d198ae57fc5b97013bdda5e883d7df383755f9
[08fed12cf9f2e7a4003a02d2a3e3efecddf49c80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08fed12cf9f2e7a4003a02d2a3e3efecddf49c80
[0923408bff8036c1b1c37bfba0a71012845c0935]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0923408bff8036c1b1c37bfba0a71012845c0935
[0997ecdc607501cf45e1e8c043210660a290646e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0997ecdc607501cf45e1e8c043210660a290646e
[09a04c14146af1916aeda14e8134d02baf088d5d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a04c14146af1916aeda14e8134d02baf088d5d
[09a885de20cffeabbfaae72f2d597e007c9b6593]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a885de20cffeabbfaae72f2d597e007c9b6593
[09df0b6551fec2ea22cee7dca2cd308cf11b531a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09df0b6551fec2ea22cee7dca2cd308cf11b531a
[0a46b5aee7eb99e19a9a2a91ed81d759978b6024]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a46b5aee7eb99e19a9a2a91ed81d759978b6024
[0a5ce1e0011920909cfa5bc022f95b3a502ff244]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a5ce1e0011920909cfa5bc022f95b3a502ff244
[0bee4492533b7a88dfb43a9965b9026bcdefc705]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0bee4492533b7a88dfb43a9965b9026bcdefc705
[0c1f5655a108865ec3fbccc964d59d455f7e27a1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c1f5655a108865ec3fbccc964d59d455f7e27a1
[0c45a5c30d2546af0789f12c0497ce3d0ddeef38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c45a5c30d2546af0789f12c0497ce3d0ddeef38
[0c6af5d6d031fd90fd472452bd42ddffab313da4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c6af5d6d031fd90fd472452bd42ddffab313da4
[0c6ba914d48380fae289077ea08b282484e075b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c6ba914d48380fae289077ea08b282484e075b5
[0c7efef500543e3176b1538c90065cad3d624c50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c7efef500543e3176b1538c90065cad3d624c50
[0ca431ce8b510b6714420a8954f587eccd84a01d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0ca431ce8b510b6714420a8954f587eccd84a01d
[0d9d80be29fc5051429cf53924d4a7ac3f6010a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0d9d80be29fc5051429cf53924d4a7ac3f6010a7
[0ea897a7665eff58d9c148ee53559504301e4a52]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0ea897a7665eff58d9c148ee53559504301e4a52
[0f225cee04b5443a58369b95bc8e6f10ed3f6401]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f225cee04b5443a58369b95bc8e6f10ed3f6401
[0f4521cd7cfe059fddab74cfd29f7920d6070ad7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f4521cd7cfe059fddab74cfd29f7920d6070ad7
[0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399
[0fceba274b8dbefb01ed890d3c211fd85211822b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0fceba274b8dbefb01ed890d3c211fd85211822b
[11c9d0bbe8651fb8e057e88166afb450534d03f4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/11c9d0bbe8651fb8e057e88166afb450534d03f4
[133cb5c7dcab6f018c0422bde1f8ee6f9a304258]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/133cb5c7dcab6f018c0422bde1f8ee6f9a304258
[157d54a4dda13b1f0b4743daa55b77d20887e82a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/157d54a4dda13b1f0b4743daa55b77d20887e82a
[1677328e42072057bcac2622726bd973255477c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1677328e42072057bcac2622726bd973255477c5
[17cd497307d04f3d8a9b058a72e3ea415a9a9f89]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17cd497307d04f3d8a9b058a72e3ea415a9a9f89
[17f89020870b8bc8ad8322e314c187b6e0836226]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17f89020870b8bc8ad8322e314c187b6e0836226
[186c74720db2b33e5c0df17ee690eddcdee360a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/186c74720db2b33e5c0df17ee690eddcdee360a7
[19adf7ec9e2a687b6df19d2e3121c2683f3fc840]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/19adf7ec9e2a687b6df19d2e3121c2683f3fc840
[1aed86246f06e1e3f68f692d81b45c2e22be60b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1aed86246f06e1e3f68f692d81b45c2e22be60b5
[1b88a1e35a66004ea5016525bcbb1e125aa64db9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1b88a1e35a66004ea5016525bcbb1e125aa64db9
[1bf2784185df479a3a17975f773e3a505f735e26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1bf2784185df479a3a17975f773e3a505f735e26
[1c58a778dc5bd05e455ea25af60e8600b8b72857]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c58a778dc5bd05e455ea25af60e8600b8b72857
[1c6ccb188129272267fd14d0f16fff42f67d81c6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c6ccb188129272267fd14d0f16fff42f67d81c6
[1c746c24b7e0ad7e7481cce626fb6488eb0076d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c746c24b7e0ad7e7481cce626fb6488eb0076d6
[1cfc72e99382ab47b55c9410ab531c0baf8ac46e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1cfc72e99382ab47b55c9410ab531c0baf8ac46e
[1d6bc01b3f26c8362f08a4adc73c24ae5b968d8e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1d6bc01b3f26c8362f08a4adc73c24ae5b968d8e
[1de420a5907cf62ae91a06732a8ef43e01f17598]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1de420a5907cf62ae91a06732a8ef43e01f17598
[1e3e789ba02d8378d590f61487c0beff5bb39d4f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e3e789ba02d8378d590f61487c0beff5bb39d4f
[1e552fe9a673dc759b583ff1c434b00385015025]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e552fe9a673dc759b583ff1c434b00385015025
[1e58094eafae95c9c09b35c63aa000a0edfd5845]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e58094eafae95c9c09b35c63aa000a0edfd5845
[2030a7392e792e00727f80f9a2d83257b851f519]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2030a7392e792e00727f80f9a2d83257b851f519
[210f01c36f0e7b8415ae595fbda857cff44277fb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/210f01c36f0e7b8415ae595fbda857cff44277fb
[2157861027eff2cade51aa950a6a4300e86a1e50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2157861027eff2cade51aa950a6a4300e86a1e50
[21745ee99eb04a4204164825ca5c50e6f8b34fee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/21745ee99eb04a4204164825ca5c50e6f8b34fee
[22980abf582aa61c5e4c9ce94d8298997fb5bbbc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/22980abf582aa61c5e4c9ce94d8298997fb5bbbc
[231fcc46d921f781a1cdaa188c9f7e189e2709cd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/231fcc46d921f781a1cdaa188c9f7e189e2709cd
[2343d26300c5a139066081648054e5e299eb8a80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2343d26300c5a139066081648054e5e299eb8a80
[238df6c7b1b569f724778c85bfead20cb14be59d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/238df6c7b1b569f724778c85bfead20cb14be59d
[23dfeb32d0a9d8a7b272ef748fca9b8556b5b0c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/23dfeb32d0a9d8a7b272ef748fca9b8556b5b0c1
[24691784f79d33a5dd5497f53e064302dfb161d3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24691784f79d33a5dd5497f53e064302dfb161d3
[24c21a15728132fcbcc6c3ce9724ed8ff19ec8a1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24c21a15728132fcbcc6c3ce9724ed8ff19ec8a1
[24f00e77839701e01123b61e4d7daefcab264a9b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24f00e77839701e01123b61e4d7daefcab264a9b
[25b35ee8e1f7b7ed7d5119bca73becc6ad721617]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25b35ee8e1f7b7ed7d5119bca73becc6ad721617
[25e4bf65d9d5af300b99092e189f0caea3164f5f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25e4bf65d9d5af300b99092e189f0caea3164f5f
[26daee68627b16262717b7091fb192a029896cf5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/26daee68627b16262717b7091fb192a029896cf5
[26ec11b7fff8c478b4aa48ed1a4cec01b683a318]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/26ec11b7fff8c478b4aa48ed1a4cec01b683a318
[281adc0d2ebea05a6f47fca2ccabffe865295c16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/281adc0d2ebea05a6f47fca2ccabffe865295c16
[29072ac5c184215f8c10547e5019bf1845864296]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/29072ac5c184215f8c10547e5019bf1845864296
[2975c9af59b515ee71824cd156c0b3b1bfba3f7d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2975c9af59b515ee71824cd156c0b3b1bfba3f7d
[2a1b10859a31649a7ef31db9474e3a6ed526c9a4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2a1b10859a31649a7ef31db9474e3a6ed526c9a4
[2a2d372e0c7bccb98ae3577a256caa710bff2e7d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2a2d372e0c7bccb98ae3577a256caa710bff2e7d
[2af24cbfbb1645e2cd364a9ab4434e0892619939]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2af24cbfbb1645e2cd364a9ab4434e0892619939
[2c1a38eaca7a66c54938c66abd046fc21e34b58e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c1a38eaca7a66c54938c66abd046fc21e34b58e
[2c34a47fc1adc5d59ce6edd35b377659084f4819]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c34a47fc1adc5d59ce6edd35b377659084f4819
[2c8bf0f1bf9c7237b20849b195c52f926c0e43ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2c8bf0f1bf9c7237b20849b195c52f926c0e43ff
[2cb0412d714078b17402d5bcfa2b1175f4f71bb3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2cb0412d714078b17402d5bcfa2b1175f4f71bb3
[2da339822a4f62266149b8d53925840c0860c9a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2da339822a4f62266149b8d53925840c0860c9a2
[2e10dd1cab1dd3a68fa66207f03d66e4e2e72c0c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2e10dd1cab1dd3a68fa66207f03d66e4e2e72c0c
[2e772970ab86a9541ad56a1702b4a219412ea88b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2e772970ab86a9541ad56a1702b4a219412ea88b
[2edda45071e3593c83d16514bcfa2a81192a6d35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2edda45071e3593c83d16514bcfa2a81192a6d35
[2f1fc58e6727c42cfc83298c0e421743f63899af]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2f1fc58e6727c42cfc83298c0e421743f63899af
[2f9a434cd13200d95d8ce5e5f0a3f8af1b822a92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2f9a434cd13200d95d8ce5e5f0a3f8af1b822a92
[2ff0d197842c39450275d6d09bdd7ed06db1e735]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2ff0d197842c39450275d6d09bdd7ed06db1e735
[30553b7989b55c802704c42deefe9424347092ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/30553b7989b55c802704c42deefe9424347092ee
[307c86c22c96ca90ca5456e8dcaf6a83534efbf6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/307c86c22c96ca90ca5456e8dcaf6a83534efbf6
[308628ccfa8cef32aa093817b78983739f52548f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/308628ccfa8cef32aa093817b78983739f52548f
[33418d0311fc75fa7fda97ac621ddf2da493c128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33418d0311fc75fa7fda97ac621ddf2da493c128
[33447dd116c091bd968eedf78675dc8c94b46982]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33447dd116c091bd968eedf78675dc8c94b46982
[33a492b9c01c57a71191d7f1b46d457d5ff67059]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33a492b9c01c57a71191d7f1b46d457d5ff67059
[34320a79819fceba1810067606990ab35bcf45b0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34320a79819fceba1810067606990ab35bcf45b0
[34595fe2693385b0cdff69ecf6306071d058b638]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34595fe2693385b0cdff69ecf6306071d058b638
[34b75c5ba38708d8ed2f63b3135c0b42b57ab065]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34b75c5ba38708d8ed2f63b3135c0b42b57ab065
[3562638d37272b2befa7f9007307fd4088cdd00c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3562638d37272b2befa7f9007307fd4088cdd00c
[35914891bd90cfadbe17867c370c65429883e879]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/35914891bd90cfadbe17867c370c65429883e879
[35fbb1dade0bbfd40042b5154430df5754caa92e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/35fbb1dade0bbfd40042b5154430df5754caa92e
[361e986710d8c97932b87bffc096e6af122ef58e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/361e986710d8c97932b87bffc096e6af122ef58e
[368eba9b473b0c31ebd232bd89bc2aabd5a15d53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/368eba9b473b0c31ebd232bd89bc2aabd5a15d53
[3805df7db5cc3090568a8ae2316b136f758dc962]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3805df7db5cc3090568a8ae2316b136f758dc962
[38d1c47305125d9bd4e9275ef99d9767af3f1380]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/38d1c47305125d9bd4e9275ef99d9767af3f1380
[3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2
[3980d5b67bbd371d84cbb313f51e950dae436d54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3980d5b67bbd371d84cbb313f51e950dae436d54
[3a6eac6bab6e45b5cb413176a614cb329c4d3f67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3a6eac6bab6e45b5cb413176a614cb329c4d3f67
[3aaddac4d39967807fa2156e11fe5ef31dac8bc8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3aaddac4d39967807fa2156e11fe5ef31dac8bc8
[3ac5374edd67a53e0c1492e487db90e9d36a91fd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ac5374edd67a53e0c1492e487db90e9d36a91fd
[3b950014411d743e3e5527f652e5a2d4aff9a847]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3b950014411d743e3e5527f652e5a2d4aff9a847
[3c34d099f162ee65423dbee77946622b391955a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c34d099f162ee65423dbee77946622b391955a3
[3c4e8b1b37415ad0662019d1792525ab0b00a8f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c4e8b1b37415ad0662019d1792525ab0b00a8f9
[3c5a18a47f168dfc463411e81b07f3250ba68df0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c5a18a47f168dfc463411e81b07f3250ba68df0
[3c7984d49df0d91037729a45c24a2a7b5a109687]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c7984d49df0d91037729a45c24a2a7b5a109687
[3d78b0d0b6918dab784bbe2332b3a26928bb8f90]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3d78b0d0b6918dab784bbe2332b3a26928bb8f90
[3df00ac9e826042b31d9617d81f54df998525031]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3df00ac9e826042b31d9617d81f54df998525031
[3e1607f076afe7a6e10578776a07d3feb0a2b9a8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3e1607f076afe7a6e10578776a07d3feb0a2b9a8
[3e70a605e7642596337bc49deda5a542c75aaddf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3e70a605e7642596337bc49deda5a542c75aaddf
[3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16
[40e428d6022a4b800d96c72b57f46263d3bf3212]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/40e428d6022a4b800d96c72b57f46263d3bf3212
[41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd
[4268bc6bc4657036534627f43966754a4a419ed5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4268bc6bc4657036534627f43966754a4a419ed5
[4370adae232ecac190d46a6828e6a7661cc0d96e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4370adae232ecac190d46a6828e6a7661cc0d96e
[44291c749bfae647cf130fdc298dd2cc5d1876ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44291c749bfae647cf130fdc298dd2cc5d1876ba
[44bd6b10290eaa4e9ede11765d25eba4b171cbe2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44bd6b10290eaa4e9ede11765d25eba4b171cbe2
[44fa812ad1f50f90cf5fcf88603fad3a44d09783]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44fa812ad1f50f90cf5fcf88603fad3a44d09783
[463d93f309ab8a320b53711f67a52072566de69c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/463d93f309ab8a320b53711f67a52072566de69c
[467b14dbc59a60a808e7a71a1083f2490cf31d48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/467b14dbc59a60a808e7a71a1083f2490cf31d48
[46c1502f07d0481c3aaab5e05899296f25c6ea13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/46c1502f07d0481c3aaab5e05899296f25c6ea13
[476ed604e7b4201efe5b6e5f7c4a588c3abaa157]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/476ed604e7b4201efe5b6e5f7c4a588c3abaa157
[48687e8a65af56cf9c6279702ccaa6a66c127a06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48687e8a65af56cf9c6279702ccaa6a66c127a06
[487138fca7e5cac02347e8597f90b9e1237bd531]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/487138fca7e5cac02347e8597f90b9e1237bd531
[48bd3aa9c24f5420f54ffdddabd061ec5a25d55b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48bd3aa9c24f5420f54ffdddabd061ec5a25d55b
[48c5a5e40fcf29e581fb329dd8c2e778a4e08b92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48c5a5e40fcf29e581fb329dd8c2e778a4e08b92
[498ad5f4cc0af31ac552e7aa6f9ee1b7ef030e13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/498ad5f4cc0af31ac552e7aa6f9ee1b7ef030e13
[4a43a7b33b59744e5b070498613ec1fb61e440e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4a43a7b33b59744e5b070498613ec1fb61e440e2
[4ac84307c9432001c1b010ff2aafec0be3b2d4cc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4ac84307c9432001c1b010ff2aafec0be3b2d4cc
[4b328c5a416cc6daeeeedf81f1dc76bc9bbf849a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b328c5a416cc6daeeeedf81f1dc76bc9bbf849a
[4b51b30d598a6e076f3d2a8b9d3e170deea1325f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b51b30d598a6e076f3d2a8b9d3e170deea1325f
[4b784a723b8c02c7bdb4b51e7d7b76147f97d569]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b784a723b8c02c7bdb4b51e7d7b76147f97d569
[4b80fa51f0034f4adf03fad5fc66329e23602f07]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b80fa51f0034f4adf03fad5fc66329e23602f07
[4bde867488ad891f614def796cff5c8f460d975d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4bde867488ad891f614def796cff5c8f460d975d
[4c7b6f1adf7398dfa570224ca470f8a71870a831]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4c7b6f1adf7398dfa570224ca470f8a71870a831
[4db00567813d9b236c77a49a33e399ad5c0c94ab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4db00567813d9b236c77a49a33e399ad5c0c94ab
[4e32f5ceab4b77a533fcc62ea68377d209b7a282]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4e32f5ceab4b77a533fcc62ea68377d209b7a282
[4eef6221708137928458ed8445b4f67196500082]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4eef6221708137928458ed8445b4f67196500082
[4fda60f22505ebba9ff86873386d0524d364765c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4fda60f22505ebba9ff86873386d0524d364765c
[505280afb5459be37c9905f1e7b23983b2e7e287]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/505280afb5459be37c9905f1e7b23983b2e7e287
[50870c1bf3cca434ec2bd03624fd690fa59dd588]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50870c1bf3cca434ec2bd03624fd690fa59dd588
[50cc3c2d086b15af580ec6e190059f2b59c0233c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50cc3c2d086b15af580ec6e190059f2b59c0233c
[50e3c5235ab8aa95ffe58b7114bdf257d4bdeff5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50e3c5235ab8aa95ffe58b7114bdf257d4bdeff5
[50fec201f1094269a1dc53bca88b25e33d7ceec4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50fec201f1094269a1dc53bca88b25e33d7ceec4
[524e269e8b6584fdcd60ff551a4f0a5d49e7384e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/524e269e8b6584fdcd60ff551a4f0a5d49e7384e
[52777c1eb5ff968430cb678630f01a100763b967]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/52777c1eb5ff968430cb678630f01a100763b967
[5337dd907a456236ebd038f7b3df6fa4e1687a68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5337dd907a456236ebd038f7b3df6fa4e1687a68
[53596672c4cc4e9b47ee60d4e96af69aeb21d3dd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53596672c4cc4e9b47ee60d4e96af69aeb21d3dd
[53892a3a2d46c3b7dcad3b0fd7b4141118485e9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53892a3a2d46c3b7dcad3b0fd7b4141118485e9e
[53e8a75974dfe4bf11740ad80c0fe769dfa0ebdb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53e8a75974dfe4bf11740ad80c0fe769dfa0ebdb
[54b0860302e2b691ef6ca54c1bde09fa97e1e3b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/54b0860302e2b691ef6ca54c1bde09fa97e1e3b2
[55113bc5705d5c9ace1da6bde8b05c1260ddb935]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55113bc5705d5c9ace1da6bde8b05c1260ddb935
[5543ff7204b50723dbdcd9042bd9747b74821bfb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5543ff7204b50723dbdcd9042bd9747b74821bfb
[55a1e55ed94f8e8a4202098c1fd4f85e337bfae4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55a1e55ed94f8e8a4202098c1fd4f85e337bfae4
[56078e8bb713fa861ccf9ebd1a58415ee6173819]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/56078e8bb713fa861ccf9ebd1a58415ee6173819
[5618d051819a9874a5db33747523553ba1f906c9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5618d051819a9874a5db33747523553ba1f906c9
[56adf88149e87aebbf87c70fc4531545d2c11daa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/56adf88149e87aebbf87c70fc4531545d2c11daa
[57e2a4d7c2b8a169d83f52162b87c52e09d23f67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/57e2a4d7c2b8a169d83f52162b87c52e09d23f67
[58d2f2059a7f43f7bdaff90043799e64ede338b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/58d2f2059a7f43f7bdaff90043799e64ede338b6
[58d9d79a5482fa0d1a555623e33f588de9665bbd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/58d9d79a5482fa0d1a555623e33f588de9665bbd
[597c0807e426e2d17f6a6b49a37665899b6bc074]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/597c0807e426e2d17f6a6b49a37665899b6bc074
[5b2b6f808c100077fc94c7019821a01896f7b652]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5b2b6f808c100077fc94c7019821a01896f7b652
[5bb8a5d1c4331d8e4b0391509abae2277012215d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5bb8a5d1c4331d8e4b0391509abae2277012215d
[5caa8dd6b7f8f2735366a23ab1005df89aaf565f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5caa8dd6b7f8f2735366a23ab1005df89aaf565f
[5cabf7b91b44fb508cbb90ea8299fd78088323be]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5cabf7b91b44fb508cbb90ea8299fd78088323be
[5d5ce99c92198d0e843259c1d32f08ba87d0039b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5d5ce99c92198d0e843259c1d32f08ba87d0039b
[5dfc932d8f62e295f93accafb98c533fd8e39625]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5dfc932d8f62e295f93accafb98c533fd8e39625
[5e9243e125e7f075ac533b6cd68fdcbef12368cf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5e9243e125e7f075ac533b6cd68fdcbef12368cf
[5e975901312e13a35d8599fc06bd0536f4c61e9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5e975901312e13a35d8599fc06bd0536f4c61e9e
[5f5606def37b70ac96d7045fa2ee36156b4d4f28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f5606def37b70ac96d7045fa2ee36156b4d4f28
[5f77a0f7ddbc194ffdc1e45556e2c85910002af6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f77a0f7ddbc194ffdc1e45556e2c85910002af6
[5f93e4ad56fdbda6b5ceeaeca94848063064cc9a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f93e4ad56fdbda6b5ceeaeca94848063064cc9a
[607a23c00aff0d9b34fb3d678bdfd5c14290582d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d
[60aa5d1c1e983aad9b0921e3e066935742605b52]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/60aa5d1c1e983aad9b0921e3e066935742605b52
[610785f3d98a4032fe7053076f9db45d4c1d1717]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/610785f3d98a4032fe7053076f9db45d4c1d1717
[616bef16cacceb26dae625be830141b8ab2252e7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/616bef16cacceb26dae625be830141b8ab2252e7
[619da4158649e2fc038bc0ecb9b36e82931508b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/619da4158649e2fc038bc0ecb9b36e82931508b6
[61f15e13d086a5d6c0e5a18d44c730ebe77a046a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/61f15e13d086a5d6c0e5a18d44c730ebe77a046a
[62e89e2917a36d73110a860a5f490e4fbb19a6b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/62e89e2917a36d73110a860a5f490e4fbb19a6b2
[630b8ce945dd2f87d88c357afec26a0a5bdbed60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/630b8ce945dd2f87d88c357afec26a0a5bdbed60
[657948516a05c40cd0d9c35dc639d05eeafa5dc5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/657948516a05c40cd0d9c35dc639d05eeafa5dc5
[659c824e36450d279d6fd684bdf848530da137f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/659c824e36450d279d6fd684bdf848530da137f5
[65c007844cc6c275b86b36a2ff1b48340622a681]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/65c007844cc6c275b86b36a2ff1b48340622a681
[66414e9db930cd116e78a692fa0590a3f574aea2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/66414e9db930cd116e78a692fa0590a3f574aea2
[6659509095d19163bd65bd24a9a554cf25207395]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6659509095d19163bd65bd24a9a554cf25207395
[6660f686f02ca2d98c9cdfe3c72cc906e446df1f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6660f686f02ca2d98c9cdfe3c72cc906e446df1f
[6671e40e458f0068097135fb37f7f5a279367396]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6671e40e458f0068097135fb37f7f5a279367396
[66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6
[676633a72f464a1f71b369281207390fb1c2efd5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/676633a72f464a1f71b369281207390fb1c2efd5
[67cb45a47f6c277bc0afc9ac9689b2a05d7b5049]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/67cb45a47f6c277bc0afc9ac9689b2a05d7b5049
[6805e2b9257cecb545417531a008ec139a0b5c54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6805e2b9257cecb545417531a008ec139a0b5c54
[68583d84a3722a27bec69a77984cd9e1167929bc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/68583d84a3722a27bec69a77984cd9e1167929bc
[6955a7b5c389a9b16651bf7e2350e12df2bc22a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6955a7b5c389a9b16651bf7e2350e12df2bc22a2
[69767ad626e27e0fda881c1e62b374165bd17825]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69767ad626e27e0fda881c1e62b374165bd17825
[699015e02fa89058bb1379f5944bde296b0603e6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/699015e02fa89058bb1379f5944bde296b0603e6
[69abc77c352e813dc24128e9952da72c77979f1a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69abc77c352e813dc24128e9952da72c77979f1a
[69ef9f7b8d04b0afa5885040b51ef50c18873fea]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69ef9f7b8d04b0afa5885040b51ef50c18873fea
[6a327d18341b245c61839b70cba29dc91f888f1b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6a327d18341b245c61839b70cba29dc91f888f1b
[6b47b9c1bc8b1cf297c987b3d4321cfe654238f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6b47b9c1bc8b1cf297c987b3d4321cfe654238f5
[6baae7cc71bf42de7584025bf53843f3c0ff8f6c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6baae7cc71bf42de7584025bf53843f3c0ff8f6c
[6c825065687ef0469d4a4d1a64b9ef9e75e9ebea]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6c825065687ef0469d4a4d1a64b9ef9e75e9ebea
[6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7
[6e284ff06182c3f684c16d49c6bfba8795a862b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6e284ff06182c3f684c16d49c6bfba8795a862b6
[6e68808588e0bb24fee292f2b236ed4adcbcbfd2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6e68808588e0bb24fee292f2b236ed4adcbcbfd2
[6f36d21e829ced48a2de9dc1ee6ed4e51b02aa78]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6f36d21e829ced48a2de9dc1ee6ed4e51b02aa78
[6f7831f10b187cb72f0ec7568db8ae9c8482a146]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6f7831f10b187cb72f0ec7568db8ae9c8482a146
[6ff633ccb93a9e75e0e0b7291a2571921d85092a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6ff633ccb93a9e75e0e0b7291a2571921d85092a
[705dd66a0d7771b67d2d1b57de9619cd969939f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/705dd66a0d7771b67d2d1b57de9619cd969939f7
[707d472928022d51dc7da2fe5322194928871f5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/707d472928022d51dc7da2fe5322194928871f5b
[70a30a2b0b651a438223f7249c6ce47931acaa92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/70a30a2b0b651a438223f7249c6ce47931acaa92
[70dcb6e4bebf26ed60cd26df4eb321417f106da5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/70dcb6e4bebf26ed60cd26df4eb321417f106da5
[7109c46d835f4d6f32b6284681a6286b68179abc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7109c46d835f4d6f32b6284681a6286b68179abc
[713bb7354358091926e524d3f29330f16da3646e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/713bb7354358091926e524d3f29330f16da3646e
[715cff55bf0dd38c2538a3a522fa7503f2e86ec1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/715cff55bf0dd38c2538a3a522fa7503f2e86ec1
[7185ba477d0d184f9cdf28eb485e3ec4e5963f3b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7185ba477d0d184f9cdf28eb485e3ec4e5963f3b
[718fe829388a82143bf2a7120fc72c4ba50c8ce5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/718fe829388a82143bf2a7120fc72c4ba50c8ce5
[7432b7c5ea20165e443d6440fcc3cb84393f3a96]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7432b7c5ea20165e443d6440fcc3cb84393f3a96
[749f8ce7aab2be9f0bf16133127a0d7fde3046c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/749f8ce7aab2be9f0bf16133127a0d7fde3046c3
[7557a59e99faf297d2055d5d9ea86b4fbfe8ba5e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7557a59e99faf297d2055d5d9ea86b4fbfe8ba5e
[75f7c9fa0fdb16e471281c701b71759e728df81d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/75f7c9fa0fdb16e471281c701b71759e728df81d
[764689fe0693c6a8588d13cde1c73f42e08b2a39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/764689fe0693c6a8588d13cde1c73f42e08b2a39
[781063204d0437481e6033a3f1cf5c6c66db102f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/781063204d0437481e6033a3f1cf5c6c66db102f
[78579a9158dc463e33c7b5ce1d248258bac89ae3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/78579a9158dc463e33c7b5ce1d248258bac89ae3
[78581dba9d33c9565fa25f0a829ca383471335f2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/78581dba9d33c9565fa25f0a829ca383471335f2
[79c1ea1edbed94e3376aed37b382d069144d6fab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/79c1ea1edbed94e3376aed37b382d069144d6fab
[7cc5fbcc0c214c4daedfd3cc447fd788864fd9f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7cc5fbcc0c214c4daedfd3cc447fd788864fd9f9
[7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13
[7d3305cc028ee0f963e0def854350e9d3eb69cb0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d3305cc028ee0f963e0def854350e9d3eb69cb0
[7d6d9f2d701713046452cae3eb740a7ea6c2ea59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d6d9f2d701713046452cae3eb740a7ea6c2ea59
[7d8d35aa8649386937ec73db7b20ea67eb7bd54f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d8d35aa8649386937ec73db7b20ea67eb7bd54f
[7d97c64e3d28115e395e28c89e56fadc8b26f0af]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d97c64e3d28115e395e28c89e56fadc8b26f0af
[7daa34c8b08e3f3d05aa8257b172d96441015321]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7daa34c8b08e3f3d05aa8257b172d96441015321
[7db097f35da98d6166b671a714d7c307b4f8958f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7db097f35da98d6166b671a714d7c307b4f8958f
[7f3911b07cc4788fe2cdb4e8d421fe5f156cac59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f3911b07cc4788fe2cdb4e8d421fe5f156cac59
[7f751c12debde6b2dcd7377d880b20d2aa834f40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f751c12debde6b2dcd7377d880b20d2aa834f40
[7f9535da6c2513ffa99d5b4888864d0c911000d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f9535da6c2513ffa99d5b4888864d0c911000d6
[7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550
[80d06ddf9245d7653827efa9aa8315ed2c634b11]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/80d06ddf9245d7653827efa9aa8315ed2c634b11
[8123ef1843ffed2f79a403105d3bdc819c9bb0ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8123ef1843ffed2f79a403105d3bdc819c9bb0ba
[81c437b02b967b56dcb9f5fa0a25b083dfa3ed25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81c437b02b967b56dcb9f5fa0a25b083dfa3ed25
[81f94b8ba9c8e0d35fddd828b1a1c4f10a9202bc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81f94b8ba9c8e0d35fddd828b1a1c4f10a9202bc
[82255c0ccbe5a57f2906ebb5626b75047f1ce20e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/82255c0ccbe5a57f2906ebb5626b75047f1ce20e
[822599689d7cef3844b5b602352e3e18197a00b7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/822599689d7cef3844b5b602352e3e18197a00b7
[830dbbd5e18ad8d53727026536b1b07c58411c35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/830dbbd5e18ad8d53727026536b1b07c58411c35
[844c1e06cd698eced1c6cd6f50645180b340ee82]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/844c1e06cd698eced1c6cd6f50645180b340ee82
[84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b
[84f30592bad9b4395dc770d44dc807125d2ced02]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84f30592bad9b4395dc770d44dc807125d2ced02
[8514bb9e5dd831640c5a05509c67ed7573c23975]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8514bb9e5dd831640c5a05509c67ed7573c23975
[8575cd87bd06ba3ad185a1be33aadd4022bbae40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8575cd87bd06ba3ad185a1be33aadd4022bbae40
[85d51b6b68b108f4a7c8cb9455961420c2cfff43]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/85d51b6b68b108f4a7c8cb9455961420c2cfff43
[85d5ba25c3b919f1c4b1159630de4702e126d5a9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/85d5ba25c3b919f1c4b1159630de4702e126d5a9
[860b213f7690873f076c098c74b83bb8822a1ba9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/860b213f7690873f076c098c74b83bb8822a1ba9
[861be6713cfc5f4996251fe23e26f67dd80001d8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/861be6713cfc5f4996251fe23e26f67dd80001d8
[877177bc4a0ca42544ece0facd2f40273b86c239]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/877177bc4a0ca42544ece0facd2f40273b86c239
[87a884bbfc07b43cf6b2cf8dadc64eab8bf7a702]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/87a884bbfc07b43cf6b2cf8dadc64eab8bf7a702
[880e35aeedfb5449626a03c9131a1ccd33e017e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/880e35aeedfb5449626a03c9131a1ccd33e017e3
[89f67f3024dd9805313a2cff67ca6e0dc901fb40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/89f67f3024dd9805313a2cff67ca6e0dc901fb40
[8a47ad83cd68c7eec60db4ff734f8ead3d54b977]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a47ad83cd68c7eec60db4ff734f8ead3d54b977
[8a50df10142c2c8d6c81eaabfb10919d1c3efa0b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a50df10142c2c8d6c81eaabfb10919d1c3efa0b
[8b28a796072ec619470e61539ea6803be8f6da36]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8b28a796072ec619470e61539ea6803be8f6da36
[8bdeafddf2131da83ad916da83ddacb27c363132]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8bdeafddf2131da83ad916da83ddacb27c363132
[8be5e30f4b82fc97cb03e05d086412e050b333db]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8be5e30f4b82fc97cb03e05d086412e050b333db
[8c23fd059412310208b811d5d771caf617f3d0c0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8c23fd059412310208b811d5d771caf617f3d0c0
[8c9a919deb3aed74a11f45ca375f28ded421f4c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8c9a919deb3aed74a11f45ca375f28ded421f4c5
[8cd40b522d2e87dd69dd21704c5f128d6d05847b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8cd40b522d2e87dd69dd21704c5f128d6d05847b
[8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c
[8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f
[8e3b72a7c1e70dad9eacc62cb3171754799c79a6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e3b72a7c1e70dad9eacc62cb3171754799c79a6
[8e6fc80beb3c1cbc52fcab7bdd8aad57c84806fe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e6fc80beb3c1cbc52fcab7bdd8aad57c84806fe
[8e98a8f387132a3a13f53d359086a80caa484cfd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e98a8f387132a3a13f53d359086a80caa484cfd
[8ec4d012fffb16013940d723077cdc44af0b156a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8ec4d012fffb16013940d723077cdc44af0b156a
[8f0eee74270820d5b04eb0c6f48934969ed5bc4c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f0eee74270820d5b04eb0c6f48934969ed5bc4c
[8f1437483337f24a4c728b61d1754f9455ee0f5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f1437483337f24a4c728b61d1754f9455ee0f5b
[8f7509161b267921fa4f4703c57280e6f1ede86f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f7509161b267921fa4f4703c57280e6f1ede86f
[8f8e8b7a856f3b3fd3a529d85830032e73510a88]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f8e8b7a856f3b3fd3a529d85830032e73510a88
[8fbb9f8e4a058e79ebd9ea45752c62133c14cac8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8fbb9f8e4a058e79ebd9ea45752c62133c14cac8
[8fc4181814ff995835076b5ad2dbb77492c52e6a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8fc4181814ff995835076b5ad2dbb77492c52e6a
[9055612289c8499748001d18c2a232cbf23fe30f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9055612289c8499748001d18c2a232cbf23fe30f
[908b2f594fdbc1aa51313bba5f26db74ee332a4a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/908b2f594fdbc1aa51313bba5f26db74ee332a4a
[9128a71a8131362709c35d506cf413db5b0bda00]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9128a71a8131362709c35d506cf413db5b0bda00
[916259ba70c903d2b2d85b4bd3eddffa98cec370]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/916259ba70c903d2b2d85b4bd3eddffa98cec370
[92b97540ecc5b69f957552118a47498022d5a9c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/92b97540ecc5b69f957552118a47498022d5a9c1
[93a58cc0254e8f4965fe7e3d5cd702489a237ee0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/93a58cc0254e8f4965fe7e3d5cd702489a237ee0
[943619bec76c9f49eac11ca7e94543bba2b8d8d7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/943619bec76c9f49eac11ca7e94543bba2b8d8d7
[943ad6258c6d01c3df3f97e35b7d0a2aa4f00136]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/943ad6258c6d01c3df3f97e35b7d0a2aa4f00136
[94d6862e0d558e69f0e5b07db5a63ad7700d515b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/94d6862e0d558e69f0e5b07db5a63ad7700d515b
[9522093d0c183c35dec4c457214a219da905baa6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9522093d0c183c35dec4c457214a219da905baa6
[97b69b23cba21ec59e6a30a5e1fc1d6a642fccda]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/97b69b23cba21ec59e6a30a5e1fc1d6a642fccda
[97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9
[989ecdd98ce86d9e4156dbd693c067f9a185a8ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/989ecdd98ce86d9e4156dbd693c067f9a185a8ba
[98c4b362743dbf5b5ef95234caa389e74dcac1ac]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/98c4b362743dbf5b5ef95234caa389e74dcac1ac
[98ebe687ce608c985a5bce2d3e9410fa234a931a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/98ebe687ce608c985a5bce2d3e9410fa234a931a
[9948f0a9035b0883644f0a37d63d16a77158be5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9948f0a9035b0883644f0a37d63d16a77158be5b
[9957cc56452652f87ac037175d3b16f273a735ea]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9957cc56452652f87ac037175d3b16f273a735ea
[997a365d6a6c72f8a3e847f1c253b1f236f05a5f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/997a365d6a6c72f8a3e847f1c253b1f236f05a5f
[99b8c469d683365998e278c50e7a4a400cfc61c6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/99b8c469d683365998e278c50e7a4a400cfc61c6
[9a2384992a0bdb78769a7aeba8393ab4767713d1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9a2384992a0bdb78769a7aeba8393ab4767713d1
[9a37f841ad435b4c36bf8b4fe93da7645fe61865]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9a37f841ad435b4c36bf8b4fe93da7645fe61865
[9ba41e3564b3058b238f0a05787373f788583b6e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ba41e3564b3058b238f0a05787373f788583b6e
[9bfb3ca7c5eeeaa20a9a5e6071206a98a3e7fa17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9bfb3ca7c5eeeaa20a9a5e6071206a98a3e7fa17
[9c4e8e9b5006801ea8310baad780daffe6a7e0a9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9c4e8e9b5006801ea8310baad780daffe6a7e0a9
[9c5fa576899d1529b06acf89221d44d262092d04]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9c5fa576899d1529b06acf89221d44d262092d04
[9ceb5b48698e16b62a380d2c1f577f54156c4ac2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ceb5b48698e16b62a380d2c1f577f54156c4ac2
[9d9179cf63c4167ac46b5c398b2c6b718ea9a022]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9d9179cf63c4167ac46b5c398b2c6b718ea9a022
[9ddaaeedecdd175672c38ba3d39c7521f08acc68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ddaaeedecdd175672c38ba3d39c7521f08acc68
[9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d
[9fbc6318f6931ba60d43843b387c4bf049d4742e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9fbc6318f6931ba60d43843b387c4bf049d4742e
[a01e57d237f55bba7e9541559c7bc0b6286cf8c0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a01e57d237f55bba7e9541559c7bc0b6286cf8c0
[a09e9f660ce2de3327a34879a5e184b3ef91a79e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a09e9f660ce2de3327a34879a5e184b3ef91a79e
[a0edb157905810d46d3418098b829744b3444d0f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a0edb157905810d46d3418098b829744b3444d0f
[a13786623e5b9117418dc6ff86c1f0519e9074f0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a13786623e5b9117418dc6ff86c1f0519e9074f0
[a1e1a680278843d4f871f5556bee679282a8d268]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a1e1a680278843d4f871f5556bee679282a8d268
[a25699295eed0a20eeb3571e0c401d4c901928eb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a25699295eed0a20eeb3571e0c401d4c901928eb
[a264d18db3089d067687f3f6e9f31e62379cd38a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a264d18db3089d067687f3f6e9f31e62379cd38a
[a468ffadc41916adb608b633acf0dd8f45d255a9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a468ffadc41916adb608b633acf0dd8f45d255a9
[a4eabe11f15c788abeabbad8d11a447a99d3414c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a4eabe11f15c788abeabbad8d11a447a99d3414c
[a4fd91f4b1340a754754b8bec841eb60102988bf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a4fd91f4b1340a754754b8bec841eb60102988bf
[a503554d9c0bbae7751b1e448156a7dc43f32def]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a503554d9c0bbae7751b1e448156a7dc43f32def
[a5ad717dc7d37586785b7375068defe352927e24]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a5ad717dc7d37586785b7375068defe352927e24
[a71c5e81761deb547c315296004167e13f82fe9b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a71c5e81761deb547c315296004167e13f82fe9b
[a746b065aa719a05f477224eba7fe551e62ebddc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a746b065aa719a05f477224eba7fe551e62ebddc
[a7cfd10ee270b2ed0c25a952c83b8ffe7235ea02]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a7cfd10ee270b2ed0c25a952c83b8ffe7235ea02
[a7e93c6068199f6b826e7aa1d21e2397d4c8e390]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a7e93c6068199f6b826e7aa1d21e2397d4c8e390
[a80facf0bb435346b0c8c3a05d22b8e428ba680f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a80facf0bb435346b0c8c3a05d22b8e428ba680f
[a82e25b56c80e37c5ea6450c4a27a9ff1feb021b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a82e25b56c80e37c5ea6450c4a27a9ff1feb021b
[a88c662583ec0222b1842048b0d2f021582ebb6f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a88c662583ec0222b1842048b0d2f021582ebb6f
[a8a5f364fc531b08d8f5fb6245d64b0c70ab95ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a8a5f364fc531b08d8f5fb6245d64b0c70ab95ba
[aa3992cc919c644dc7fe3bc41abc2dd970fe3d2e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa3992cc919c644dc7fe3bc41abc2dd970fe3d2e
[aa5bdbbcdbd2d36b08f11c0a252603526b7adce8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa5bdbbcdbd2d36b08f11c0a252603526b7adce8
[aaaf78e17cfeeea087fe9562fc65907b3847bc9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aaaf78e17cfeeea087fe9562fc65907b3847bc9e
[aab71b50e4464cae19f1add8b28613260345d9db]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aab71b50e4464cae19f1add8b28613260345d9db
[aaf976b84513bcdb2395fab5349fc035e4601068]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aaf976b84513bcdb2395fab5349fc035e4601068
[ab579207ea14141d3d4327f39b5fd23830a89f3a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ab579207ea14141d3d4327f39b5fd23830a89f3a
[ac509a2c0f12541ac4db4107a423ada59732c4dd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac509a2c0f12541ac4db4107a423ada59732c4dd
[ac5749d32d335f800fc8f3636cfecd321ebefb42]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac5749d32d335f800fc8f3636cfecd321ebefb42
[ac8d29bb53de9b0bc06572f85073a1ac06f54087]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac8d29bb53de9b0bc06572f85073a1ac06f54087
[acc34edc4502c691381df03a3bf9c2aebde1a038]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/acc34edc4502c691381df03a3bf9c2aebde1a038
[acc9b5b8722b130d1551ea716628f096e7805c9b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/acc9b5b8722b130d1551ea716628f096e7805c9b
[addbc642be40f93ba3df1588dcb165cbc9b4f0d1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/addbc642be40f93ba3df1588dcb165cbc9b4f0d1
[adf400700122f4eb23fd63971b3f048e014d1781]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/adf400700122f4eb23fd63971b3f048e014d1781
[aec8cebca812844afec8050e30d93fb8fa3bb203]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aec8cebca812844afec8050e30d93fb8fa3bb203
[aee27e45bc52c5a6839a66266d03a304d2608351]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aee27e45bc52c5a6839a66266d03a304d2608351
[af46851919ced5582dd8d6c5b236edd3ac078061]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/af46851919ced5582dd8d6c5b236edd3ac078061
[af93d662852bbed6a3c13ca4f54ae4a63af56c20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/af93d662852bbed6a3c13ca4f54ae4a63af56c20
[afc0dab53064bef4aec0f5181e25b8f96e0169f4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/afc0dab53064bef4aec0f5181e25b8f96e0169f4
[b03da48883f07bd1e089f080dc4bc6fa9cfc8578]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b03da48883f07bd1e089f080dc4bc6fa9cfc8578
[b088be725c367aabae07d4b60553693a5c2ddd80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b088be725c367aabae07d4b60553693a5c2ddd80
[b1dc6f927b19e6f1d722454b6792d467834096df]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b1dc6f927b19e6f1d722454b6792d467834096df
[b227f531a6f348cdd9b3fa5fe010adf979dd8e98]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b227f531a6f348cdd9b3fa5fe010adf979dd8e98
[b2530a582f9edcab94d80f9e53142ee801c8335f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2530a582f9edcab94d80f9e53142ee801c8335f
[b2a0ab53877db5bf91b216baf3ba5e08853da559]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2a0ab53877db5bf91b216baf3ba5e08853da559
[b2d6de5072f1506077fa649b15912b7cb3064211]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2d6de5072f1506077fa649b15912b7cb3064211
[b339c881ae94eb1b14c02462042c4c8e8416e951]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b339c881ae94eb1b14c02462042c4c8e8416e951
[b50ce9932b2b8502a113b85714f6ac9564c2645d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b50ce9932b2b8502a113b85714f6ac9564c2645d
[b5505730100a9780877eb3e1cb4d280f02845863]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5505730100a9780877eb3e1cb4d280f02845863
[b55341ecd717344211bd79557f56f7fecaad2479]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b55341ecd717344211bd79557f56f7fecaad2479
[b5d4d91779599bae9fc15d78c5e3db3f4a43f18b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5d4d91779599bae9fc15d78c5e3db3f4a43f18b
[b6d359fe3efb94ba8f85c7eaa1788665c392021d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b6d359fe3efb94ba8f85c7eaa1788665c392021d
[b723fed816b98dc1bfa9484909c53a8078a1335d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b723fed816b98dc1bfa9484909c53a8078a1335d
[b7a25d0905f7aa8426eb97ada89a516620d81e77]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b7a25d0905f7aa8426eb97ada89a516620d81e77
[b8989f3f0e848138b6de90b81b2c774e775a015d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8989f3f0e848138b6de90b81b2c774e775a015d
[b8a8c34b650c47b815fc307346aecc69f35d192a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8a8c34b650c47b815fc307346aecc69f35d192a
[b8deef3439f8e8b9a949a0a1cfa16d2c027c391f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8deef3439f8e8b9a949a0a1cfa16d2c027c391f
[b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5
[bab5ee53d297fd4d3cb21ce411cef4c01748d082]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bab5ee53d297fd4d3cb21ce411cef4c01748d082
[baff6da9a9bc313db65a613d5edae82d67aee4c2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/baff6da9a9bc313db65a613d5edae82d67aee4c2
[bbdb2cb61fabac44421596f4e3c64e725532e5c7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bbdb2cb61fabac44421596f4e3c64e725532e5c7
[bbe3b00626693af8310454616c08b8358fedb042]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bbe3b00626693af8310454616c08b8358fedb042
[bc4112866bb713538fc48c209408313c634306b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bc4112866bb713538fc48c209408313c634306b2
[bd44896a30627bafefa64c1cbc78229113130b9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd44896a30627bafefa64c1cbc78229113130b9d
[bd49cdc8220e8adcfea71f04c6ebcfb51946336b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd49cdc8220e8adcfea71f04c6ebcfb51946336b
[bd9b494e083a2861f8c991cfe75f80f61d72ddef]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd9b494e083a2861f8c991cfe75f80f61d72ddef
[bdafe96f41ea33ec27a840dbda74ed909f6f7532]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bdafe96f41ea33ec27a840dbda74ed909f6f7532
[bea60aae98c6f7b6ffbb23a30fc58d825397a3e0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bea60aae98c6f7b6ffbb23a30fc58d825397a3e0
[c0f32a8351b2738429e4583169a1bed3781deb73]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c0f32a8351b2738429e4583169a1bed3781deb73
[c1262d43fcdfdf9b2d3604786757bdf3a8ed77cf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c1262d43fcdfdf9b2d3604786757bdf3a8ed77cf
[c13d0820b77a54b1e15a0f42ecea6d6b250a9fc2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c13d0820b77a54b1e15a0f42ecea6d6b250a9fc2
[c225eb65b2330d6f61580c37504421144308febc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c225eb65b2330d6f61580c37504421144308febc
[c240b82c292586648b2dbd345eba39d716ddd43f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c240b82c292586648b2dbd345eba39d716ddd43f
[c332a73363492a1e1874e68fc0c12e3bfd2b96ae]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c332a73363492a1e1874e68fc0c12e3bfd2b96ae
[c35066cd2cc01344259f00559186fbd1a12db527]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c35066cd2cc01344259f00559186fbd1a12db527
[c3d0621bef3a9d3ca2c3d9967860f839b4389fd6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c3d0621bef3a9d3ca2c3d9967860f839b4389fd6
[c4c7f3014b51280932244d5c132031f23642cf79]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c4c7f3014b51280932244d5c132031f23642cf79
[c4f86ec51cd2e3b21260d7314398b34d0661fdd7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c4f86ec51cd2e3b21260d7314398b34d0661fdd7
[c618a199f59706ad2cfca64e2c37bbe4b615faf1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c618a199f59706ad2cfca64e2c37bbe4b615faf1
[c6f189916c9fc9cbc4f69ea7a42c110497e7e819]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c6f189916c9fc9cbc4f69ea7a42c110497e7e819
[c7ec4ebfe41385a409265ef9dcb3ff4fa9222b03]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c7ec4ebfe41385a409265ef9dcb3ff4fa9222b03
[c80a0449b8729d3c64775e56de8fe27f21017c6f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c80a0449b8729d3c64775e56de8fe27f21017c6f
[c8328f3ab256bf76a92b205f8eeebc49447bd25e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8328f3ab256bf76a92b205f8eeebc49447bd25e
[c83e433c50687c9611cb298e64823ba9a2dcec6a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c83e433c50687c9611cb298e64823ba9a2dcec6a
[c87ceff11912ec3788f390cf454b1a84db5fd8a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c87ceff11912ec3788f390cf454b1a84db5fd8a3
[c8fc525dff93e1b29c0df61bf6cc593376910043]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8fc525dff93e1b29c0df61bf6cc593376910043
[c9a5de8fb611785c5d8bfd6e6942f48006cf9814]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9a5de8fb611785c5d8bfd6e6942f48006cf9814
[c9bc19ecd6cad88742cfa3758e48fd606f489220]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9bc19ecd6cad88742cfa3758e48fd606f489220
[c9dc70a51be61bc46b43082e7227f873cb77ac10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9dc70a51be61bc46b43082e7227f873cb77ac10
[ca1c967a1dd169b73f3002f120c40c7127060041]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ca1c967a1dd169b73f3002f120c40c7127060041
[cae987706e31a6c223e5af997fee32b537714efd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cae987706e31a6c223e5af997fee32b537714efd
[cb698ec9a74329fc7947db489769472f0cbaf4e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb698ec9a74329fc7947db489769472f0cbaf4e8
[cb74da327e27b73e9724d8a28aafc164e6c9e0df]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb74da327e27b73e9724d8a28aafc164e6c9e0df
[cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870
[cc1cb8aa305b3dc17f9df7c0ad8c898bc931b0c2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cc1cb8aa305b3dc17f9df7c0ad8c898bc931b0c2
[ccd4f0c94c53343c239d61e5cee680e7df8d312b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ccd4f0c94c53343c239d61e5cee680e7df8d312b
[cda6e991ea22413221c90eacd9b5a16c875ef316]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cda6e991ea22413221c90eacd9b5a16c875ef316
[cdaad462bfea78e0e079853e198a32ec89a5d7bc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cdaad462bfea78e0e079853e198a32ec89a5d7bc
[cdd64dfe9773aa85ccdcf1099290b273519169d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cdd64dfe9773aa85ccdcf1099290b273519169d6
[ce8518ead73f36025e708c567cfaa1d9d74d5f2c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ce8518ead73f36025e708c567cfaa1d9d74d5f2c
[cebf2818fe60d8509af94fe623bf0d1f7ff44b17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cebf2818fe60d8509af94fe623bf0d1f7ff44b17
[ced4667fd5f16682a46e70d435a9a473885c70b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ced4667fd5f16682a46e70d435a9a473885c70b6
[cef66e50bd7b149177a635d0f2bb17e1b77799ec]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cef66e50bd7b149177a635d0f2bb17e1b77799ec
[cf9153bc3cec7f038ae47397c9d0a9942d5f364e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cf9153bc3cec7f038ae47397c9d0a9942d5f364e
[cf91c1d2808a7658da8eb6263c3aca0ff3e5fb04]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cf91c1d2808a7658da8eb6263c3aca0ff3e5fb04
[cff93366ae59c85d01f5d818ea2e8c8c73cedb87]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cff93366ae59c85d01f5d818ea2e8c8c73cedb87
[d03737cda4c53aba353a32f33fd32f7fa74738ad]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d03737cda4c53aba353a32f33fd32f7fa74738ad
[d0736743fca8af8a4dde7d8317b72de269f5655b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d0736743fca8af8a4dde7d8317b72de269f5655b
[d091a792aa369ea4bff566bd321a4a9c9cbb589c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d091a792aa369ea4bff566bd321a4a9c9cbb589c
[d0f4166b6610b624b6bb2d28a7acba407aea7ca5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d0f4166b6610b624b6bb2d28a7acba407aea7ca5
[d159553e8eaf2166c6d3b6187c007ad3dfc21400]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d159553e8eaf2166c6d3b6187c007ad3dfc21400
[d1dfe28abda8f6f8b46a1aebaee5750521fb5854]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d1dfe28abda8f6f8b46a1aebaee5750521fb5854
[d205c419687d0908828ff4f06f4e56351a7ea2f4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d205c419687d0908828ff4f06f4e56351a7ea2f4
[d2158ee2b1b23a68b3c4dd764863acadec08d6bb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d2158ee2b1b23a68b3c4dd764863acadec08d6bb
[d31c14564c2bc27ed4e7790a54b16d09a01c3be9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d31c14564c2bc27ed4e7790a54b16d09a01c3be9
[d340bd2985295b3ccf4559c4ab1ac3588501ca4b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d340bd2985295b3ccf4559c4ab1ac3588501ca4b
[d395d94cddeea82f7117682882407feb35258fad]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d395d94cddeea82f7117682882407feb35258fad
[d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd
[d3f723ed85b0c433c1c6c0a424ccf33ccb11a17d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f723ed85b0c433c1c6c0a424ccf33ccb11a17d
[d48099c07d95b49914e4e4271679b4846dd6b608]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d48099c07d95b49914e4e4271679b4846dd6b608
[d4ed03e046d292888e555de3b6955b396ef7fad0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d4ed03e046d292888e555de3b6955b396ef7fad0
[d4f5e0af96e5ce9d83c12e46a345dc5525d27a95]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d4f5e0af96e5ce9d83c12e46a345dc5525d27a95
[d5af77deed057d599fd1c4b5c1f6222a7edba4c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d5af77deed057d599fd1c4b5c1f6222a7edba4c3
[d6bb2d1da026c16c4a301fa675653d8a0688a679]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d6bb2d1da026c16c4a301fa675653d8a0688a679
[d70104fb19ee3e133188a14d49f2c57ab0a55e06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d70104fb19ee3e133188a14d49f2c57ab0a55e06
[d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59
[d882f968ae9011b112cb8f195171e5357747a6af]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d882f968ae9011b112cb8f195171e5357747a6af
[d8d56414c28f5ca7ba2db10420c1805270d80d7b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8d56414c28f5ca7ba2db10420c1805270d80d7b
[d8faf4fd010e303dad42c8a0a51520c03fd197b8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8faf4fd010e303dad42c8a0a51520c03fd197b8
[d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da
[d9f70ce89f21bc8e48184856257fddac0a0372e1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d9f70ce89f21bc8e48184856257fddac0a0372e1
[da6f91faf961dff4f1adbf528cd4025d98cd3624]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da6f91faf961dff4f1adbf528cd4025d98cd3624
[da980d8bdcb4ac506db0862b11987de8eb859179]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da980d8bdcb4ac506db0862b11987de8eb859179
[da99c2da85e51527402ce80a2876c7bb64c1d2e7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da99c2da85e51527402ce80a2876c7bb64c1d2e7
[daded23ce694301aadd19c09a07bb1d384668ce9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/daded23ce694301aadd19c09a07bb1d384668ce9
[db2e8f3cf4db912d32e74fcbdf09094c8b2f5128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db2e8f3cf4db912d32e74fcbdf09094c8b2f5128
[db5b6a5fbc301716f84682c4dae7e1691fcba413]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db5b6a5fbc301716f84682c4dae7e1691fcba413
[dc30ca638c88714942f282de4cd464336e41f8de]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc30ca638c88714942f282de4cd464336e41f8de
[dc7b7c27f7d239fcf02d78981ea13a5563c88f88]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc7b7c27f7d239fcf02d78981ea13a5563c88f88
[dd63214e4877ab17811d0e1db6867cff6bb72e61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dd63214e4877ab17811d0e1db6867cff6bb72e61
[dd8248c388a6f8df54c12f5dd010de613a0e21ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dd8248c388a6f8df54c12f5dd010de613a0e21ee
[ddc2a8036d9ea83b75bdc5cf506c365f5b09a3a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ddc2a8036d9ea83b75bdc5cf506c365f5b09a3a7
[df2bcdef317ceb778093641fe97b8cf5664bf4bd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/df2bcdef317ceb778093641fe97b8cf5664bf4bd
[df628a72730e677e16a3053988983f752d71940a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/df628a72730e677e16a3053988983f752d71940a
[dfab1e709a370d468ffb3540f3c6d3e280e97017]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfab1e709a370d468ffb3540f3c6d3e280e97017
[dfd2898d64411f280bbe7d04280a9c73d3a3b310]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd2898d64411f280bbe7d04280a9c73d3a3b310
[dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa
[dff2927698abcac250fd3f0df7910c02818f6776]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dff2927698abcac250fd3f0df7910c02818f6776
[e083e73e8fbdeab7c9421e729521c08bd9c77fbc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e083e73e8fbdeab7c9421e729521c08bd9c77fbc
[e189fd21f8689048e404ddf19c279ad743203924]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e189fd21f8689048e404ddf19c279ad743203924
[e1dfb0a281d3d922ada33f53013accb2c765bd9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e1dfb0a281d3d922ada33f53013accb2c765bd9d
[e1e4606847459e742f9c5e51a860b8903b2bc5ce]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e1e4606847459e742f9c5e51a860b8903b2bc5ce
[e317c87ece614341553f2d4b7926f1614a1a5b5a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e317c87ece614341553f2d4b7926f1614a1a5b5a
[e346e184d9ab0af7969a796ef4c43814267aa7a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e346e184d9ab0af7969a796ef4c43814267aa7a3
[e37557772d193a2e812598ed06ea0ab8656dd293]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e37557772d193a2e812598ed06ea0ab8656dd293
[e3b58f922ce8b312ee1fc9b04b39e5dcf75cf1c4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3b58f922ce8b312ee1fc9b04b39e5dcf75cf1c4
[e3c0e0a430d6e27060b00db05c17f01d68361547]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3c0e0a430d6e27060b00db05c17f01d68361547
[e3ca0e225065cf4fe610fd0f49748dc8cab48f71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3ca0e225065cf4fe610fd0f49748dc8cab48f71
[e42d021c20b90e50c464541fb3d358ac24ce3b3a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e42d021c20b90e50c464541fb3d358ac24ce3b3a
[e43d48bfd451fe4aac2f90b0e19a357bf5a1c1b9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e43d48bfd451fe4aac2f90b0e19a357bf5a1c1b9
[e46b1f943753dc0a5bf1b45b458f0fde643ebdf5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e46b1f943753dc0a5bf1b45b458f0fde643ebdf5
[e4f8059e97c3ba25401f8752607a66fea4dee10d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e4f8059e97c3ba25401f8752607a66fea4dee10d
[e51c30f16a3fb478829bade3350a429d54ee3e94]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e51c30f16a3fb478829bade3350a429d54ee3e94
[e5e7f45a1bc577211908f98bc9a9bbbf335cf332]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e5e7f45a1bc577211908f98bc9a9bbbf335cf332
[e6931ed967f7ea795ecdecfaeeead533642445f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e6931ed967f7ea795ecdecfaeeead533642445f5
[e7172e4519383c352ed147aa42b3aeca646a690e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e7172e4519383c352ed147aa42b3aeca646a690e
[e736a714f4b2a84e4b5d578c8789049c1bbc4df6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e736a714f4b2a84e4b5d578c8789049c1bbc4df6
[e75daad23339e77fe0f36b3ef666c68f9d28b60a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e75daad23339e77fe0f36b3ef666c68f9d28b60a
[e80df38691580c8377c5e3fd30a02617765ee69d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e80df38691580c8377c5e3fd30a02617765ee69d
[e8406c56c77869ad8e70e9e1e7e448a0f458a204]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e8406c56c77869ad8e70e9e1e7e448a0f458a204
[e9b501fc77259d0c1c050bedc5a61c3516e4c307]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9b501fc77259d0c1c050bedc5a61c3516e4c307
[e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a
[eb5a1c333d495d5ffdd95c390992de1e2a26e92d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eb5a1c333d495d5ffdd95c390992de1e2a26e92d
[ebf4fd3f494ad12521f1f9ef1d4548282447e8d0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ebf4fd3f494ad12521f1f9ef1d4548282447e8d0
[ec82a5009bcf7a16aaa694eb478216b9567c87c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ec82a5009bcf7a16aaa694eb478216b9567c87c1
[ed3d1feb788121161ba66f9c1826a67ded941337]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed3d1feb788121161ba66f9c1826a67ded941337
[ed5c04ade1af13f2e22afc184336f9713f2b76e0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed5c04ade1af13f2e22afc184336f9713f2b76e0
[ee4515f1fd7e5161b5eab5bce0262971996f843f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee4515f1fd7e5161b5eab5bce0262971996f843f
[ee4c9d0c34ef366f008f83767e0b2b88a9e90a4d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee4c9d0c34ef366f008f83767e0b2b88a9e90a4d
[ee95b6364d51c7d8a6bd4259ceda8ec63d13f56b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee95b6364d51c7d8a6bd4259ceda8ec63d13f56b
[eeb20bb8431bf75c9e2be3fbba8e64daafae3098]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eeb20bb8431bf75c9e2be3fbba8e64daafae3098
[eeb3632bd465d4937204f1d4c3e5f72a953bcfa6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eeb3632bd465d4937204f1d4c3e5f72a953bcfa6
[ef73bd5c114916a2f430dcd9c26eb49ec98f3fcc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef73bd5c114916a2f430dcd9c26eb49ec98f3fcc
[ef7d59a60d323655aa6b3616f0d10a78ab11b565]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef7d59a60d323655aa6b3616f0d10a78ab11b565
[ef80a0d5d844ce2b8ec80391305f0b71fc18b518]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef80a0d5d844ce2b8ec80391305f0b71fc18b518
[efb694dfe1f34fee33210b9b5e3a749cb9468be4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/efb694dfe1f34fee33210b9b5e3a749cb9468be4
[effffe87f8390d5894ab8dcf1806b2dd5b54e493]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/effffe87f8390d5894ab8dcf1806b2dd5b54e493
[f00ebc4ccb3e82ae2d54787d9e39a6bce3044032]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f00ebc4ccb3e82ae2d54787d9e39a6bce3044032
[f0146fc0172a0f95718c22f531d43494740166f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f0146fc0172a0f95718c22f531d43494740166f7
[f08dd9c4bdc950c70d380d0a98c9546d8efd8c00]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f08dd9c4bdc950c70d380d0a98c9546d8efd8c00
[f126782888b04516748ec2ce1740a1e8db2f75c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f126782888b04516748ec2ce1740a1e8db2f75c1
[f1baa4d5f07e31c179c983a0b855cbc240903859]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f1baa4d5f07e31c179c983a0b855cbc240903859
[f2199b30ca34e9d46d1e51436b2cfba7c9b2f64c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f2199b30ca34e9d46d1e51436b2cfba7c9b2f64c
[f355964d7b4c6bcc0d5cd726df4ff360f2adac23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f355964d7b4c6bcc0d5cd726df4ff360f2adac23
[f363682cfb202a8fcfaf591c0db5f6fbbd472fa1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f363682cfb202a8fcfaf591c0db5f6fbbd472fa1
[f3749f5ba0323c8b5c685ff5bed0b63f472be3e9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f3749f5ba0323c8b5c685ff5bed0b63f472be3e9
[f42039339adc2bbb24d983232ba5c9f52cf03316]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f42039339adc2bbb24d983232ba5c9f52cf03316
[f46be6a032569b5726d4df69efc519cec1e8fb29]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f46be6a032569b5726d4df69efc519cec1e8fb29
[f49fb33dab085714a8050d36442c04bf504f731e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f49fb33dab085714a8050d36442c04bf504f731e
[f4cdd62e98bd1edb356650f70f116c44927f9673]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f4cdd62e98bd1edb356650f70f116c44927f9673
[f56045aa6c147246f30635240835e92bea224520]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f56045aa6c147246f30635240835e92bea224520
[f58f506f17d6b76343d5bd814749259e3b380cc2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f58f506f17d6b76343d5bd814749259e3b380cc2
[f5abc7a12684e6ebf12721a64c95e76a7a620c6b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5abc7a12684e6ebf12721a64c95e76a7a620c6b
[f5bd4e3a8260d8bc5224c5cb851ac0dfe854ee7e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5bd4e3a8260d8bc5224c5cb851ac0dfe854ee7e
[f5bf771e6f26407fd2066f4765193adb250955c9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5bf771e6f26407fd2066f4765193adb250955c9
[f5f2be2dd7d45cf1cc4df2638b6ec3e98a0075b3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f5f2be2dd7d45cf1cc4df2638b6ec3e98a0075b3
[f6a72ff1328766f733fe6314ecdbc1429bb57e61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6a72ff1328766f733fe6314ecdbc1429bb57e61
[f6b52fc20a8893ce30443bdd27f8da11108d0e17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6b52fc20a8893ce30443bdd27f8da11108d0e17
[f708e15eab0ca601699461565b7a396f84394526]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f708e15eab0ca601699461565b7a396f84394526
[f7b4533f180ccc94c27f8e42b9806199d147f5c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f7b4533f180ccc94c27f8e42b9806199d147f5c1
[f8f977d1bde282c350758aa2ebcca56eaef81c4a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f8f977d1bde282c350758aa2ebcca56eaef81c4a
[f94d2a0dddd5aa8afe73eb06963af6c3b40e3b01]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f94d2a0dddd5aa8afe73eb06963af6c3b40e3b01
[f968b462d74e05c806bf5560f356799aa40b7104]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f968b462d74e05c806bf5560f356799aa40b7104
[fa5ff7329049623be8379968adf2946360a780cb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fa5ff7329049623be8379968adf2946360a780cb
[fc2a8379ad2f848990c749418ebe4123cacbcf8b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc2a8379ad2f848990c749418ebe4123cacbcf8b
[fc5482359615f1f1a0d83c4f34a1ca89834d38ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc5482359615f1f1a0d83c4f34a1ca89834d38ff
[fcf91c96ea0dd598594aec0fac23726426b4cd3b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fcf91c96ea0dd598594aec0fac23726426b4cd3b
[fcfc7294018d7e2e559b42be2f70fd1df853514f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fcfc7294018d7e2e559b42be2f70fd1df853514f
[fda30e592981b402a192fe6f74ac36febdc946c8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda30e592981b402a192fe6f74ac36febdc946c8
[fda61f8ffc7ddd95556f4109b9e735cdde2c1b93]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda61f8ffc7ddd95556f4109b9e735cdde2c1b93
[fdc1899ac00ddde0355f09a5c6aaf6d79a1aeec7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fdc1899ac00ddde0355f09a5c6aaf6d79a1aeec7
[fe422d64df17d550cac10ae4306b02f5bf99964b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fe422d64df17d550cac10ae4306b02f5bf99964b
[febfd00d66ac8586584882ec6c7a5b2a97683571]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/febfd00d66ac8586584882ec6c7a5b2a97683571
[ff0f46959254dd193a3b7abb63699ac58106e204]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ff0f46959254dd193a3b7abb63699ac58106e204
[ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048
