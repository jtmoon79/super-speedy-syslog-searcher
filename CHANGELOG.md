# CHANGELOG<!-- omit in toc -->

<!--
Helper script `tools/changelog-link-gen.sh` can generate the addendum of markdown links for this CHANGELOG.md.
-->

Manual changelog for [super speedy syslog searcher](https://github.com/jtmoon79/super-speedy-syslog-searcher).

---

<!-- TODO per release: update TOC -->
<!-- Table of Contents updated by "Markdown All In One" extension for Visual Studio Code -->

- [Unreleased](#unreleased)
- [0.5.58](#0558)
  - [New](#new)
  - [Changes](#changes)
- [0.4.57](#0457)
  - [Fixes](#fixes)
  - [Changes](#changes-1)
- [0.4.56](#0456)
  - [New](#new-1)
  - [Fixes](#fixes-1)
  - [Changes](#changes-2)
- [0.3.55](#0355)
  - [New](#new-2)
  - [Changes](#changes-3)
  - [Fixes](#fixes-2)
- [0.3.54](#0354)
  - [New](#new-3)
  - [Fixes](#fixes-3)
- [0.3.53](#0353)
  - [New](#new-4)
  - [Changes](#changes-4)
- [0.2.52](#0252)
  - [New](#new-5)
- [0.2.51](#0251)
  - [New](#new-6)
- [0.2.50](#0250)
  - [New](#new-7)
  - [Changes](#changes-5)
  - [Fixes](#fixes-4)
- [0.2.49](#0249)
  - [Changes](#changes-6)
  - [Fixes](#fixes-5)
- [0.2.48](#0248)
  - [New](#new-8)
  - [Changes](#changes-7)
  - [Fixes](#fixes-6)
- [0.2.47](#0247)
- [0.2.46](#0246)
  - [New](#new-9)
  - [Changes](#changes-8)
  - [Fixes](#fixes-7)
- [0.1.45](#0145)
  - [New](#new-10)
  - [Changes](#changes-9)
- [0.1.44](#0144)
  - [New](#new-11)
  - [Changes](#changes-10)
  - [Fixes](#fixes-8)
- [0.1.43](#0143)
  - [New](#new-12)
  - [Changes](#changes-11)
- [0.1.42](#0142)
  - [Changes](#changes-12)
- [0.1.41](#0141)
  - [Changes](#changes-13)
  - [Fixes](#fixes-9)
- [0.1.40](#0140)
  - [New](#new-13)
  - [Changes](#changes-14)
- [0.1.39](#0139)
  - [Changes](#changes-15)
- [0.1.38](#0138)
  - [New](#new-14)
  - [Changes](#changes-16)
- [0.0.37](#0037)
  - [New](#new-15)
  - [Changes](#changes-17)
- [0.0.36](#0036)
  - [New](#new-16)
  - [Fixes](#fixes-10)
  - [Changes](#changes-18)
- [0.0.35](#0035)
  - [New](#new-17)
  - [Fixes](#fixes-11)
- [0.0.34](#0034)
  - [New](#new-18)
  - [Fixes](#fixes-12)
- [0.0.33](#0033)
  - [New](#new-19)
- [0.0.32](#0032)
  - [New](#new-20)
  - [Fixes](#fixes-13)
- [0.0.31](#0031)
  - [New](#new-21)
- [0.0.30](#0030)
  - [New](#new-22)
  - [Changes](#changes-19)
- [0.0.29](#0029)
  - [Changes](#changes-20)
- [0.0.28](#0028)
  - [New](#new-23)
  - [Changes](#changes-21)
  - [Fixes](#fixes-14)
- [0.0.27](#0027)
  - [New](#new-24)
  - [Changes](#changes-22)
- [0.0.26](#0026)
  - [New](#new-25)
  - [Changes](#changes-23)
  - [Fixes](#fixes-15)
- [0.0.25](#0025)
  - [New](#new-26)
  - [Changes](#changes-24)
  - [Fixes](#fixes-16)
- [0.0.24](#0024)
  - [New](#new-27)
  - [Changes](#changes-25)
- [0.0.23](#0023)
  - [New](#new-28)
  - [Changes](#changes-26)
  - [Fixes](#fixes-17)
- [0.0.22](#0022)
  - [New](#new-29)
  - [Fixes](#fixes-18)
  - [Changes](#changes-27)
- [0.0.21](#0021)
  - [New](#new-30)
  - [Fixes](#fixes-19)

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

-->

## Unreleased

<!-- TODO per release: Change Version in the following URL -->

[unreleased-diff](https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.5.58...HEAD)

<!-- TODO per release: Add Section(s) -->

---

## 0.5.58

_Released 2023-03-29_

[0.4.57...0.5.58]

### New

- (BIN) Allow user-passed timezone for prepended datetime. ([630b8ce945dd2f87d88c357afec26a0a5bdbed60])
- (PROJECT) add logs/programs/{AWS,Microsoft IIS,apache} ([ee4515f1fd7e5161b5eab5bce0262971996f843f])

### Changes

- (TEST) add test test_PrinterLogMessage_print_evtx ([e6931ed967f7ea795ecdecfaeeead533642445f5])
- (BUILD) clap 0.4.21 ([f8f977d1bde282c350758aa2ebcca56eaef81c4a]) ([8f7509161b267921fa4f4703c57280e6f1ede86f])

---

## 0.4.57

_Released 2023-03-26_

[0.4.56...0.4.57]

### Fixes

- (BIN) bin.rs tweak --help wording ([3c34d099f162ee65423dbee77946622b391955a3])

### Changes

- (LIB) Print evtx files in chronological order [Issue #86] ([e42d021c20b90e50c464541fb3d358ac24ce3b3a])
- (BIN) bin.rs summary linerize more ([cebf2818fe60d8509af94fe623bf0d1f7ff44b17])
- (LIB) datetime.rs NFC refactor dt_pass_filters ([bbdb2cb61fabac44421596f4e3c64e725532e5c7])
- (PROJECT) CHANGLOG revise h2 naming ([ef80a0d5d844ce2b8ec80391305f0b71fc18b518])

---

## 0.4.56

_Released 2023-03-24_

[0.3.55...0.4.56]

### New

- (LIB) process Windows Event Log evtx files [Issue #86] [Issue #87] ([368eba9b473b0c31ebd232bd89bc2aabd5a15d53])

### Fixes

- (BIN) bin.rs fix -a "" (passing empty string) ([d4ed03e046d292888e555de3b6955b396ef7fad0])
- (BIN) bin.rs fix panic for multiple non-existent paths ([da980d8bdcb4ac506db0862b11987de8eb859179])

### Changes

- (TEST) add various `Summary` tests ([7f3911b07cc4788fe2cdb4e8d421fe5f156cac59]) ([9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d]) ([5b2b6f808c100077fc94c7019821a01896f7b652]) ([0923408bff8036c1b1c37bfba0a71012845c0935]) ([d03737cda4c53aba353a32f33fd32f7fa74738ad])

---

## 0.3.55

_Released 2023-03-18_

[0.3.54...0.3.55]

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

### New

- (LIB) add support for utmpx login records (major refactoring) [Issue #70] ([b227f531a6f348cdd9b3fa5fe010adf979dd8e98])

### Changes

- (BUILD) MSRV 1.66.0

---

## 0.2.52

_Released 2023-02-15_

[0.2.51...0.2.52]

### New

- (LIB) datetime.rs add format catalina apache access  [Issue #82] ([5337dd907a456236ebd038f7b3df6fa4e1687a68]) ([997a365d6a6c72f8a3e847f1c253b1f236f05a5f])

---

## 0.2.51

_Released 2023-02-09_

[0.2.50...0.2.51]

### New

- (BIN) bin.rs option --sysline-separator [Issue #80] ([b6d359fe3efb94ba8f85c7eaa1788665c392021d])
- (BIN) print bytes counts in hex ([e46b1f943753dc0a5bf1b45b458f0fde643ebdf5])
- (LIB) datetime pattern for tomcat catalina [Issue #81] ([8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f])

---

## 0.2.50

_Released 2023-01-29_

[0.2.49...0.2.50]

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

- (BIN) bin.rs fix typo in clap help ([b03da48883f07bd1e089f080dc4bc6fa9cfc8578])
- (DOCS) README update --help ([cdaad462bfea78e0e079853e198a32ec89a5d7bc])

---

## 0.2.46

_Released 2023-01-09_

[0.1.45...0.2.46]

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

### Changes

- (BIN) (BUILD) update clap from 3 to 4 ([f58f506f17d6b76343d5bd814749259e3b380cc2])
- (BUILD) cargo update ([41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd])

---

## 0.1.41

_Released 2022-12-18_

[0.1.40...0.1.41]

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

### New

- (BIN) add CLI option `--prepend-separator` ([467b14dbc59a60a808e7a71a1083f2490cf31d48])

### Changes

- (BIN) add summary _syslines stored high_ ([d1f5895f1e5a55cbbcbfc4072bbde53a7a85fc])

---

## 0.1.39

_Released 2022-10-19_

[0.1.38...0.1.39]

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

### New

- (BIN) bin.rs --summary print -a -b as UTC ([186c74720db2b33e5c0df17ee690eddcdee360a7])
- (BIN) bin.rs allow relative offset datetimes [Issue #35] ([4eef6221708137928458ed8445b4f67196500082])

### Changes

- (DOCS) README add Windows log snippets, tweak wording ([65c007844cc6c275b86b36a2ff1b48340622a681])

---

## 0.0.37

_Released 2022-10-12_

[0.0.36...0.0.37]

### New

- (LIB) datetime.rs patterns for Windows compsetup.log mrt.log ([0f225cee04b5443a58369b95bc8e6f10ed3f6401])

### Changes

- (LIB) blockreader.rs eprintln path ([3e1607f076afe7a6e10578776a07d3feb0a2b9a8])
- (TEST) add logs Windows10Pro ([1c746c24b7e0ad7e7481cce626fb6488eb0076d6])

---

## 0.0.36

_Released 2022-10-10_

[0.0.35...0.0.36]

### New

- (LIB) datetime.rs parsing nginx apache logs [Issue #53] ([003b29bab508b32750cb303c70db9dc75cc04eab])

### Fixes

- (LIB) datetime.rs add RP_NOALPHA after CGP_TZZ ([1cfc72e99382ab47b55c9410ab531c0baf8ac46e])

### Changes

- (BIN) bin.rs --help tweak, comment cleanup ([3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2])
- (TEST) add more logs ([9ddaaeedecdd175672c38ba3d39c7521f08acc68]) ([effffe87f8390d5894ab8dcf1806b2dd5b54e493]) ([66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6]) ([0743e4157daa108569d99746d8a6314cfe6e0248]) ([e736a714f4b2a84e4b5d578c8789049c1bbc4df6])
- (DOCS) README changes, docstring changes ([657948516a05c40cd0d9c35dc639d05eeafa5dc5]) ([001f0c3db2c5751a35946f572aca6bf07c9efcaf]) ([4b51b30d598a6e076f3d2a8b9d3e170deea1325f]) ([fda30e592981b402a192fe6f74ac36febdc946c8]) ([8f1437483337f24a4c728b61d1754f9455ee0f5b]) ([44291c749bfae647cf130fdc298dd2cc5d1876ba])

---

## 0.0.35

_Released 2022-10-09_

[0.0.34...0.0.35]

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

### New

- (BIN) bin.rs allow user opt dt format string [Issue #28] ([6660f686f02ca2d98c9cdfe3c72cc906e446df1f])
- (DEBUG) use `si_trace_print` 0.2.5 ([fc5482359615f1f1a0d83c4f34a1ca89834d38ff])

---

## 0.0.32

_Released 2022-09-20_

[0.0.31...0.0.32]

### New

- (TEST) datetime_tests.rs add tests cases datetime_parse_from_str ([c9bc19ecd6cad88742cfa3758e48fd606f489220])

### Fixes

- (LIB) datetime.rs fix copying fractional [Issue #23] ([764689fe0693c6a8588d13cde1c73f42e08b2a39])

---

## 0.0.31

_Released 2022-09-19_

[0.0.30...0.0.31]

### New

- (DOCS) improved README badges ([30553b7989b55c802704c42deefe9424347092ee]) ([b5d4d91779599bae9fc15d78c5e3db3f4a43f18b]) ([17cd497307d04f3d8a9b058a72e3ea415a9a9f89])

---

## 0.0.30

_Released 2022-09-18_

[0.0.29...0.0.30]

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

### Fixes

- (LIB) fix errant �� printed at block edges ([5caa8dd6b7f8f2735366a23ab1005df89aaf565f])

### Changes

- (BUILD) remove crate chain-cmp ([7109c46d835f4d6f32b6284681a6286b68179abc])
- (LIB) set `const` for funcs `slice_contains...` ([eeb20bb8431bf75c9e2be3fbba8e64daafae3098])

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
[Issue #20]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/20
[Issue #21]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/21
[Issue #22]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/22
[Issue #23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/23
[Issue #28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/28
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
[001f0c3db2c5751a35946f572aca6bf07c9efcaf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/001f0c3db2c5751a35946f572aca6bf07c9efcaf
[003b29bab508b32750cb303c70db9dc75cc04eab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/003b29bab508b32750cb303c70db9dc75cc04eab
[01f395903cff248be11ecf6f12974a3951aa7e92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01f395903cff248be11ecf6f12974a3951aa7e92
[031434f4d9dfb4e0f8190a720f8db57a3772e3a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/031434f4d9dfb4e0f8190a720f8db57a3772e3a2
[0579522ff7609e22c14b33aa6c6a70cec6372226]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0579522ff7609e22c14b33aa6c6a70cec6372226
[06e500f1d0148e0f9b50ab5907d7f6103533d5f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/06e500f1d0148e0f9b50ab5907d7f6103533d5f7
[07214abde6479431cc1a9f87f50f3b713e5ea503]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07214abde6479431cc1a9f87f50f3b713e5ea503
[0743e4157daa108569d99746d8a6314cfe6e0248]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0743e4157daa108569d99746d8a6314cfe6e0248
[07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8
[08d198ae57fc5b97013bdda5e883d7df383755f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/08d198ae57fc5b97013bdda5e883d7df383755f9
[0923408bff8036c1b1c37bfba0a71012845c0935]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0923408bff8036c1b1c37bfba0a71012845c0935
[09a04c14146af1916aeda14e8134d02baf088d5d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a04c14146af1916aeda14e8134d02baf088d5d
[09a885de20cffeabbfaae72f2d597e007c9b6593]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a885de20cffeabbfaae72f2d597e007c9b6593
[09df0b6551fec2ea22cee7dca2cd308cf11b531a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09df0b6551fec2ea22cee7dca2cd308cf11b531a
[0a46b5aee7eb99e19a9a2a91ed81d759978b6024]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a46b5aee7eb99e19a9a2a91ed81d759978b6024
[0a5ce1e0011920909cfa5bc022f95b3a502ff244]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a5ce1e0011920909cfa5bc022f95b3a502ff244
[0bee4492533b7a88dfb43a9965b9026bcdefc705]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0bee4492533b7a88dfb43a9965b9026bcdefc705
[0c7efef500543e3176b1538c90065cad3d624c50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0c7efef500543e3176b1538c90065cad3d624c50
[0ca431ce8b510b6714420a8954f587eccd84a01d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0ca431ce8b510b6714420a8954f587eccd84a01d
[0d9d80be29fc5051429cf53924d4a7ac3f6010a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0d9d80be29fc5051429cf53924d4a7ac3f6010a7
[0f225cee04b5443a58369b95bc8e6f10ed3f6401]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f225cee04b5443a58369b95bc8e6f10ed3f6401
[0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399
[133cb5c7dcab6f018c0422bde1f8ee6f9a304258]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/133cb5c7dcab6f018c0422bde1f8ee6f9a304258
[17cd497307d04f3d8a9b058a72e3ea415a9a9f89]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17cd497307d04f3d8a9b058a72e3ea415a9a9f89
[17f89020870b8bc8ad8322e314c187b6e0836226]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17f89020870b8bc8ad8322e314c187b6e0836226
[186c74720db2b33e5c0df17ee690eddcdee360a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/186c74720db2b33e5c0df17ee690eddcdee360a7
[19adf7ec9e2a687b6df19d2e3121c2683f3fc840]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/19adf7ec9e2a687b6df19d2e3121c2683f3fc840
[1b88a1e35a66004ea5016525bcbb1e125aa64db9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1b88a1e35a66004ea5016525bcbb1e125aa64db9
[1bf2784185df479a3a17975f773e3a505f735e26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1bf2784185df479a3a17975f773e3a505f735e26
[1c58a778dc5bd05e455ea25af60e8600b8b72857]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c58a778dc5bd05e455ea25af60e8600b8b72857
[1c746c24b7e0ad7e7481cce626fb6488eb0076d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c746c24b7e0ad7e7481cce626fb6488eb0076d6
[1cfc72e99382ab47b55c9410ab531c0baf8ac46e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1cfc72e99382ab47b55c9410ab531c0baf8ac46e
[1de420a5907cf62ae91a06732a8ef43e01f17598]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1de420a5907cf62ae91a06732a8ef43e01f17598
[1e58094eafae95c9c09b35c63aa000a0edfd5845]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1e58094eafae95c9c09b35c63aa000a0edfd5845
[2157861027eff2cade51aa950a6a4300e86a1e50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2157861027eff2cade51aa950a6a4300e86a1e50
[21745ee99eb04a4204164825ca5c50e6f8b34fee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/21745ee99eb04a4204164825ca5c50e6f8b34fee
[22980abf582aa61c5e4c9ce94d8298997fb5bbbc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/22980abf582aa61c5e4c9ce94d8298997fb5bbbc
[2343d26300c5a139066081648054e5e299eb8a80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2343d26300c5a139066081648054e5e299eb8a80
[238df6c7b1b569f724778c85bfead20cb14be59d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/238df6c7b1b569f724778c85bfead20cb14be59d
[24f00e77839701e01123b61e4d7daefcab264a9b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/24f00e77839701e01123b61e4d7daefcab264a9b
[25e4bf65d9d5af300b99092e189f0caea3164f5f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25e4bf65d9d5af300b99092e189f0caea3164f5f
[26ec11b7fff8c478b4aa48ed1a4cec01b683a318]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/26ec11b7fff8c478b4aa48ed1a4cec01b683a318
[29072ac5c184215f8c10547e5019bf1845864296]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/29072ac5c184215f8c10547e5019bf1845864296
[2975c9af59b515ee71824cd156c0b3b1bfba3f7d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2975c9af59b515ee71824cd156c0b3b1bfba3f7d
[2a1b10859a31649a7ef31db9474e3a6ed526c9a4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2a1b10859a31649a7ef31db9474e3a6ed526c9a4
[2da339822a4f62266149b8d53925840c0860c9a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2da339822a4f62266149b8d53925840c0860c9a2
[30553b7989b55c802704c42deefe9424347092ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/30553b7989b55c802704c42deefe9424347092ee
[308628ccfa8cef32aa093817b78983739f52548f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/308628ccfa8cef32aa093817b78983739f52548f
[33418d0311fc75fa7fda97ac621ddf2da493c128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33418d0311fc75fa7fda97ac621ddf2da493c128
[33a492b9c01c57a71191d7f1b46d457d5ff67059]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33a492b9c01c57a71191d7f1b46d457d5ff67059
[34320a79819fceba1810067606990ab35bcf45b0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34320a79819fceba1810067606990ab35bcf45b0
[34595fe2693385b0cdff69ecf6306071d058b638]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34595fe2693385b0cdff69ecf6306071d058b638
[3562638d37272b2befa7f9007307fd4088cdd00c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3562638d37272b2befa7f9007307fd4088cdd00c
[35fbb1dade0bbfd40042b5154430df5754caa92e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/35fbb1dade0bbfd40042b5154430df5754caa92e
[361e986710d8c97932b87bffc096e6af122ef58e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/361e986710d8c97932b87bffc096e6af122ef58e
[368eba9b473b0c31ebd232bd89bc2aabd5a15d53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/368eba9b473b0c31ebd232bd89bc2aabd5a15d53
[3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2
[3980d5b67bbd371d84cbb313f51e950dae436d54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3980d5b67bbd371d84cbb313f51e950dae436d54
[3ac5374edd67a53e0c1492e487db90e9d36a91fd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ac5374edd67a53e0c1492e487db90e9d36a91fd
[3c34d099f162ee65423dbee77946622b391955a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c34d099f162ee65423dbee77946622b391955a3
[3c4e8b1b37415ad0662019d1792525ab0b00a8f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c4e8b1b37415ad0662019d1792525ab0b00a8f9
[3c7984d49df0d91037729a45c24a2a7b5a109687]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c7984d49df0d91037729a45c24a2a7b5a109687
[3d78b0d0b6918dab784bbe2332b3a26928bb8f90]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3d78b0d0b6918dab784bbe2332b3a26928bb8f90
[3e1607f076afe7a6e10578776a07d3feb0a2b9a8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3e1607f076afe7a6e10578776a07d3feb0a2b9a8
[3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16
[41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd
[44291c749bfae647cf130fdc298dd2cc5d1876ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44291c749bfae647cf130fdc298dd2cc5d1876ba
[44bd6b10290eaa4e9ede11765d25eba4b171cbe2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44bd6b10290eaa4e9ede11765d25eba4b171cbe2
[44fa812ad1f50f90cf5fcf88603fad3a44d09783]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44fa812ad1f50f90cf5fcf88603fad3a44d09783
[467b14dbc59a60a808e7a71a1083f2490cf31d48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/467b14dbc59a60a808e7a71a1083f2490cf31d48
[48687e8a65af56cf9c6279702ccaa6a66c127a06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48687e8a65af56cf9c6279702ccaa6a66c127a06
[48bd3aa9c24f5420f54ffdddabd061ec5a25d55b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48bd3aa9c24f5420f54ffdddabd061ec5a25d55b
[4b51b30d598a6e076f3d2a8b9d3e170deea1325f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b51b30d598a6e076f3d2a8b9d3e170deea1325f
[4b784a723b8c02c7bdb4b51e7d7b76147f97d569]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b784a723b8c02c7bdb4b51e7d7b76147f97d569
[4bde867488ad891f614def796cff5c8f460d975d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4bde867488ad891f614def796cff5c8f460d975d
[4eef6221708137928458ed8445b4f67196500082]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4eef6221708137928458ed8445b4f67196500082
[4fda60f22505ebba9ff86873386d0524d364765c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4fda60f22505ebba9ff86873386d0524d364765c
[50870c1bf3cca434ec2bd03624fd690fa59dd588]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50870c1bf3cca434ec2bd03624fd690fa59dd588
[50fec201f1094269a1dc53bca88b25e33d7ceec4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/50fec201f1094269a1dc53bca88b25e33d7ceec4
[5337dd907a456236ebd038f7b3df6fa4e1687a68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5337dd907a456236ebd038f7b3df6fa4e1687a68
[53892a3a2d46c3b7dcad3b0fd7b4141118485e9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53892a3a2d46c3b7dcad3b0fd7b4141118485e9e
[55113bc5705d5c9ace1da6bde8b05c1260ddb935]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55113bc5705d5c9ace1da6bde8b05c1260ddb935
[55a1e55ed94f8e8a4202098c1fd4f85e337bfae4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55a1e55ed94f8e8a4202098c1fd4f85e337bfae4
[56078e8bb713fa861ccf9ebd1a58415ee6173819]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/56078e8bb713fa861ccf9ebd1a58415ee6173819
[58d2f2059a7f43f7bdaff90043799e64ede338b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/58d2f2059a7f43f7bdaff90043799e64ede338b6
[5b2b6f808c100077fc94c7019821a01896f7b652]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5b2b6f808c100077fc94c7019821a01896f7b652
[5caa8dd6b7f8f2735366a23ab1005df89aaf565f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5caa8dd6b7f8f2735366a23ab1005df89aaf565f
[5cabf7b91b44fb508cbb90ea8299fd78088323be]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5cabf7b91b44fb508cbb90ea8299fd78088323be
[5dfc932d8f62e295f93accafb98c533fd8e39625]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5dfc932d8f62e295f93accafb98c533fd8e39625
[5e9243e125e7f075ac533b6cd68fdcbef12368cf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5e9243e125e7f075ac533b6cd68fdcbef12368cf
[5e975901312e13a35d8599fc06bd0536f4c61e9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5e975901312e13a35d8599fc06bd0536f4c61e9e
[5f5606def37b70ac96d7045fa2ee36156b4d4f28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f5606def37b70ac96d7045fa2ee36156b4d4f28
[5f93e4ad56fdbda6b5ceeaeca94848063064cc9a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f93e4ad56fdbda6b5ceeaeca94848063064cc9a
[607a23c00aff0d9b34fb3d678bdfd5c14290582d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d
[60aa5d1c1e983aad9b0921e3e066935742605b52]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/60aa5d1c1e983aad9b0921e3e066935742605b52
[610785f3d98a4032fe7053076f9db45d4c1d1717]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/610785f3d98a4032fe7053076f9db45d4c1d1717
[619da4158649e2fc038bc0ecb9b36e82931508b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/619da4158649e2fc038bc0ecb9b36e82931508b6
[61f15e13d086a5d6c0e5a18d44c730ebe77a046a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/61f15e13d086a5d6c0e5a18d44c730ebe77a046a
[62e89e2917a36d73110a860a5f490e4fbb19a6b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/62e89e2917a36d73110a860a5f490e4fbb19a6b2
[630b8ce945dd2f87d88c357afec26a0a5bdbed60]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/630b8ce945dd2f87d88c357afec26a0a5bdbed60
[657948516a05c40cd0d9c35dc639d05eeafa5dc5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/657948516a05c40cd0d9c35dc639d05eeafa5dc5
[65c007844cc6c275b86b36a2ff1b48340622a681]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/65c007844cc6c275b86b36a2ff1b48340622a681
[6659509095d19163bd65bd24a9a554cf25207395]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6659509095d19163bd65bd24a9a554cf25207395
[6660f686f02ca2d98c9cdfe3c72cc906e446df1f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6660f686f02ca2d98c9cdfe3c72cc906e446df1f
[66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6
[676633a72f464a1f71b369281207390fb1c2efd5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/676633a72f464a1f71b369281207390fb1c2efd5
[6805e2b9257cecb545417531a008ec139a0b5c54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6805e2b9257cecb545417531a008ec139a0b5c54
[6955a7b5c389a9b16651bf7e2350e12df2bc22a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6955a7b5c389a9b16651bf7e2350e12df2bc22a2
[69abc77c352e813dc24128e9952da72c77979f1a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/69abc77c352e813dc24128e9952da72c77979f1a
[6b47b9c1bc8b1cf297c987b3d4321cfe654238f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6b47b9c1bc8b1cf297c987b3d4321cfe654238f5
[6baae7cc71bf42de7584025bf53843f3c0ff8f6c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6baae7cc71bf42de7584025bf53843f3c0ff8f6c
[6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7
[6e284ff06182c3f684c16d49c6bfba8795a862b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6e284ff06182c3f684c16d49c6bfba8795a862b6
[705dd66a0d7771b67d2d1b57de9619cd969939f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/705dd66a0d7771b67d2d1b57de9619cd969939f7
[70dcb6e4bebf26ed60cd26df4eb321417f106da5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/70dcb6e4bebf26ed60cd26df4eb321417f106da5
[7109c46d835f4d6f32b6284681a6286b68179abc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7109c46d835f4d6f32b6284681a6286b68179abc
[713bb7354358091926e524d3f29330f16da3646e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/713bb7354358091926e524d3f29330f16da3646e
[715cff55bf0dd38c2538a3a522fa7503f2e86ec1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/715cff55bf0dd38c2538a3a522fa7503f2e86ec1
[7185ba477d0d184f9cdf28eb485e3ec4e5963f3b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7185ba477d0d184f9cdf28eb485e3ec4e5963f3b
[749f8ce7aab2be9f0bf16133127a0d7fde3046c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/749f8ce7aab2be9f0bf16133127a0d7fde3046c3
[7557a59e99faf297d2055d5d9ea86b4fbfe8ba5e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7557a59e99faf297d2055d5d9ea86b4fbfe8ba5e
[764689fe0693c6a8588d13cde1c73f42e08b2a39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/764689fe0693c6a8588d13cde1c73f42e08b2a39
[78581dba9d33c9565fa25f0a829ca383471335f2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/78581dba9d33c9565fa25f0a829ca383471335f2
[79c1ea1edbed94e3376aed37b382d069144d6fab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/79c1ea1edbed94e3376aed37b382d069144d6fab
[7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13
[7d6d9f2d701713046452cae3eb740a7ea6c2ea59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d6d9f2d701713046452cae3eb740a7ea6c2ea59
[7d8d35aa8649386937ec73db7b20ea67eb7bd54f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7d8d35aa8649386937ec73db7b20ea67eb7bd54f
[7db097f35da98d6166b671a714d7c307b4f8958f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7db097f35da98d6166b671a714d7c307b4f8958f
[7f3911b07cc4788fe2cdb4e8d421fe5f156cac59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f3911b07cc4788fe2cdb4e8d421fe5f156cac59
[7f751c12debde6b2dcd7377d880b20d2aa834f40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f751c12debde6b2dcd7377d880b20d2aa834f40
[7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550
[80d06ddf9245d7653827efa9aa8315ed2c634b11]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/80d06ddf9245d7653827efa9aa8315ed2c634b11
[81c437b02b967b56dcb9f5fa0a25b083dfa3ed25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81c437b02b967b56dcb9f5fa0a25b083dfa3ed25
[82255c0ccbe5a57f2906ebb5626b75047f1ce20e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/82255c0ccbe5a57f2906ebb5626b75047f1ce20e
[822599689d7cef3844b5b602352e3e18197a00b7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/822599689d7cef3844b5b602352e3e18197a00b7
[84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b
[84f30592bad9b4395dc770d44dc807125d2ced02]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84f30592bad9b4395dc770d44dc807125d2ced02
[8514bb9e5dd831640c5a05509c67ed7573c23975]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8514bb9e5dd831640c5a05509c67ed7573c23975
[8575cd87bd06ba3ad185a1be33aadd4022bbae40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8575cd87bd06ba3ad185a1be33aadd4022bbae40
[85d51b6b68b108f4a7c8cb9455961420c2cfff43]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/85d51b6b68b108f4a7c8cb9455961420c2cfff43
[880e35aeedfb5449626a03c9131a1ccd33e017e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/880e35aeedfb5449626a03c9131a1ccd33e017e3
[8a47ad83cd68c7eec60db4ff734f8ead3d54b977]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a47ad83cd68c7eec60db4ff734f8ead3d54b977
[8a50df10142c2c8d6c81eaabfb10919d1c3efa0b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a50df10142c2c8d6c81eaabfb10919d1c3efa0b
[8b28a796072ec619470e61539ea6803be8f6da36]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8b28a796072ec619470e61539ea6803be8f6da36
[8c9a919deb3aed74a11f45ca375f28ded421f4c5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8c9a919deb3aed74a11f45ca375f28ded421f4c5
[8cd40b522d2e87dd69dd21704c5f128d6d05847b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8cd40b522d2e87dd69dd21704c5f128d6d05847b
[8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c
[8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8def2f69f1d0b55c73ccb0fe7e35435b67d79c6f
[8e3b72a7c1e70dad9eacc62cb3171754799c79a6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e3b72a7c1e70dad9eacc62cb3171754799c79a6
[8e6fc80beb3c1cbc52fcab7bdd8aad57c84806fe]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e6fc80beb3c1cbc52fcab7bdd8aad57c84806fe
[8f1437483337f24a4c728b61d1754f9455ee0f5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f1437483337f24a4c728b61d1754f9455ee0f5b
[8f7509161b267921fa4f4703c57280e6f1ede86f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f7509161b267921fa4f4703c57280e6f1ede86f
[8fbb9f8e4a058e79ebd9ea45752c62133c14cac8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8fbb9f8e4a058e79ebd9ea45752c62133c14cac8
[9055612289c8499748001d18c2a232cbf23fe30f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9055612289c8499748001d18c2a232cbf23fe30f
[908b2f594fdbc1aa51313bba5f26db74ee332a4a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/908b2f594fdbc1aa51313bba5f26db74ee332a4a
[9128a71a8131362709c35d506cf413db5b0bda00]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9128a71a8131362709c35d506cf413db5b0bda00
[916259ba70c903d2b2d85b4bd3eddffa98cec370]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/916259ba70c903d2b2d85b4bd3eddffa98cec370
[943ad6258c6d01c3df3f97e35b7d0a2aa4f00136]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/943ad6258c6d01c3df3f97e35b7d0a2aa4f00136
[94d6862e0d558e69f0e5b07db5a63ad7700d515b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/94d6862e0d558e69f0e5b07db5a63ad7700d515b
[97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9
[989ecdd98ce86d9e4156dbd693c067f9a185a8ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/989ecdd98ce86d9e4156dbd693c067f9a185a8ba
[98c4b362743dbf5b5ef95234caa389e74dcac1ac]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/98c4b362743dbf5b5ef95234caa389e74dcac1ac
[9957cc56452652f87ac037175d3b16f273a735ea]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9957cc56452652f87ac037175d3b16f273a735ea
[997a365d6a6c72f8a3e847f1c253b1f236f05a5f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/997a365d6a6c72f8a3e847f1c253b1f236f05a5f
[9a37f841ad435b4c36bf8b4fe93da7645fe61865]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9a37f841ad435b4c36bf8b4fe93da7645fe61865
[9ba41e3564b3058b238f0a05787373f788583b6e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ba41e3564b3058b238f0a05787373f788583b6e
[9c5fa576899d1529b06acf89221d44d262092d04]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9c5fa576899d1529b06acf89221d44d262092d04
[9ceb5b48698e16b62a380d2c1f577f54156c4ac2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ceb5b48698e16b62a380d2c1f577f54156c4ac2
[9d9179cf63c4167ac46b5c398b2c6b718ea9a022]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9d9179cf63c4167ac46b5c398b2c6b718ea9a022
[9ddaaeedecdd175672c38ba3d39c7521f08acc68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ddaaeedecdd175672c38ba3d39c7521f08acc68
[9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9f5391ba1ff4b7c8aa43d6ab3da57ee7693e0b9d
[a13786623e5b9117418dc6ff86c1f0519e9074f0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a13786623e5b9117418dc6ff86c1f0519e9074f0
[a1e1a680278843d4f871f5556bee679282a8d268]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a1e1a680278843d4f871f5556bee679282a8d268
[a25699295eed0a20eeb3571e0c401d4c901928eb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a25699295eed0a20eeb3571e0c401d4c901928eb
[a4fd91f4b1340a754754b8bec841eb60102988bf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a4fd91f4b1340a754754b8bec841eb60102988bf
[a503554d9c0bbae7751b1e448156a7dc43f32def]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a503554d9c0bbae7751b1e448156a7dc43f32def
[a71c5e81761deb547c315296004167e13f82fe9b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a71c5e81761deb547c315296004167e13f82fe9b
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
[adf400700122f4eb23fd63971b3f048e014d1781]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/adf400700122f4eb23fd63971b3f048e014d1781
[aee27e45bc52c5a6839a66266d03a304d2608351]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aee27e45bc52c5a6839a66266d03a304d2608351
[af46851919ced5582dd8d6c5b236edd3ac078061]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/af46851919ced5582dd8d6c5b236edd3ac078061
[b03da48883f07bd1e089f080dc4bc6fa9cfc8578]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b03da48883f07bd1e089f080dc4bc6fa9cfc8578
[b088be725c367aabae07d4b60553693a5c2ddd80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b088be725c367aabae07d4b60553693a5c2ddd80
[b1dc6f927b19e6f1d722454b6792d467834096df]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b1dc6f927b19e6f1d722454b6792d467834096df
[b227f531a6f348cdd9b3fa5fe010adf979dd8e98]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b227f531a6f348cdd9b3fa5fe010adf979dd8e98
[b2d6de5072f1506077fa649b15912b7cb3064211]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2d6de5072f1506077fa649b15912b7cb3064211
[b5505730100a9780877eb3e1cb4d280f02845863]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5505730100a9780877eb3e1cb4d280f02845863
[b55341ecd717344211bd79557f56f7fecaad2479]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b55341ecd717344211bd79557f56f7fecaad2479
[b5d4d91779599bae9fc15d78c5e3db3f4a43f18b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5d4d91779599bae9fc15d78c5e3db3f4a43f18b
[b6d359fe3efb94ba8f85c7eaa1788665c392021d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b6d359fe3efb94ba8f85c7eaa1788665c392021d
[b723fed816b98dc1bfa9484909c53a8078a1335d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b723fed816b98dc1bfa9484909c53a8078a1335d
[b7a25d0905f7aa8426eb97ada89a516620d81e77]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b7a25d0905f7aa8426eb97ada89a516620d81e77
[b8deef3439f8e8b9a949a0a1cfa16d2c027c391f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8deef3439f8e8b9a949a0a1cfa16d2c027c391f
[b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5
[bab5ee53d297fd4d3cb21ce411cef4c01748d082]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bab5ee53d297fd4d3cb21ce411cef4c01748d082
[bbdb2cb61fabac44421596f4e3c64e725532e5c7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bbdb2cb61fabac44421596f4e3c64e725532e5c7
[bbe3b00626693af8310454616c08b8358fedb042]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bbe3b00626693af8310454616c08b8358fedb042
[bc4112866bb713538fc48c209408313c634306b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bc4112866bb713538fc48c209408313c634306b2
[bd44896a30627bafefa64c1cbc78229113130b9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd44896a30627bafefa64c1cbc78229113130b9d
[bd49cdc8220e8adcfea71f04c6ebcfb51946336b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd49cdc8220e8adcfea71f04c6ebcfb51946336b
[c225eb65b2330d6f61580c37504421144308febc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c225eb65b2330d6f61580c37504421144308febc
[c332a73363492a1e1874e68fc0c12e3bfd2b96ae]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c332a73363492a1e1874e68fc0c12e3bfd2b96ae
[c35066cd2cc01344259f00559186fbd1a12db527]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c35066cd2cc01344259f00559186fbd1a12db527
[c4c7f3014b51280932244d5c132031f23642cf79]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c4c7f3014b51280932244d5c132031f23642cf79
[c6f189916c9fc9cbc4f69ea7a42c110497e7e819]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c6f189916c9fc9cbc4f69ea7a42c110497e7e819
[c80a0449b8729d3c64775e56de8fe27f21017c6f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c80a0449b8729d3c64775e56de8fe27f21017c6f
[c8328f3ab256bf76a92b205f8eeebc49447bd25e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8328f3ab256bf76a92b205f8eeebc49447bd25e
[c87ceff11912ec3788f390cf454b1a84db5fd8a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c87ceff11912ec3788f390cf454b1a84db5fd8a3
[c8fc525dff93e1b29c0df61bf6cc593376910043]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8fc525dff93e1b29c0df61bf6cc593376910043
[c9bc19ecd6cad88742cfa3758e48fd606f489220]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9bc19ecd6cad88742cfa3758e48fd606f489220
[c9dc70a51be61bc46b43082e7227f873cb77ac10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9dc70a51be61bc46b43082e7227f873cb77ac10
[ca1c967a1dd169b73f3002f120c40c7127060041]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ca1c967a1dd169b73f3002f120c40c7127060041
[cae987706e31a6c223e5af997fee32b537714efd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cae987706e31a6c223e5af997fee32b537714efd
[cb698ec9a74329fc7947db489769472f0cbaf4e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb698ec9a74329fc7947db489769472f0cbaf4e8
[cb74da327e27b73e9724d8a28aafc164e6c9e0df]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb74da327e27b73e9724d8a28aafc164e6c9e0df
[cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870
[cda6e991ea22413221c90eacd9b5a16c875ef316]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cda6e991ea22413221c90eacd9b5a16c875ef316
[cdaad462bfea78e0e079853e198a32ec89a5d7bc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cdaad462bfea78e0e079853e198a32ec89a5d7bc
[cebf2818fe60d8509af94fe623bf0d1f7ff44b17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cebf2818fe60d8509af94fe623bf0d1f7ff44b17
[ced4667fd5f16682a46e70d435a9a473885c70b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ced4667fd5f16682a46e70d435a9a473885c70b6
[d03737cda4c53aba353a32f33fd32f7fa74738ad]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d03737cda4c53aba353a32f33fd32f7fa74738ad
[d091a792aa369ea4bff566bd321a4a9c9cbb589c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d091a792aa369ea4bff566bd321a4a9c9cbb589c
[d2158ee2b1b23a68b3c4dd764863acadec08d6bb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d2158ee2b1b23a68b3c4dd764863acadec08d6bb
[d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd
[d3f723ed85b0c433c1c6c0a424ccf33ccb11a17d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f723ed85b0c433c1c6c0a424ccf33ccb11a17d
[d48099c07d95b49914e4e4271679b4846dd6b608]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d48099c07d95b49914e4e4271679b4846dd6b608
[d4ed03e046d292888e555de3b6955b396ef7fad0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d4ed03e046d292888e555de3b6955b396ef7fad0
[d5af77deed057d599fd1c4b5c1f6222a7edba4c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d5af77deed057d599fd1c4b5c1f6222a7edba4c3
[d6bb2d1da026c16c4a301fa675653d8a0688a679]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d6bb2d1da026c16c4a301fa675653d8a0688a679
[d70104fb19ee3e133188a14d49f2c57ab0a55e06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d70104fb19ee3e133188a14d49f2c57ab0a55e06
[d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59
[d882f968ae9011b112cb8f195171e5357747a6af]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d882f968ae9011b112cb8f195171e5357747a6af
[d8d56414c28f5ca7ba2db10420c1805270d80d7b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8d56414c28f5ca7ba2db10420c1805270d80d7b
[d8faf4fd010e303dad42c8a0a51520c03fd197b8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8faf4fd010e303dad42c8a0a51520c03fd197b8
[d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da
[d9f70ce89f21bc8e48184856257fddac0a0372e1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d9f70ce89f21bc8e48184856257fddac0a0372e1
[da980d8bdcb4ac506db0862b11987de8eb859179]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da980d8bdcb4ac506db0862b11987de8eb859179
[da99c2da85e51527402ce80a2876c7bb64c1d2e7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/da99c2da85e51527402ce80a2876c7bb64c1d2e7
[db2e8f3cf4db912d32e74fcbdf09094c8b2f5128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db2e8f3cf4db912d32e74fcbdf09094c8b2f5128
[db5b6a5fbc301716f84682c4dae7e1691fcba413]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db5b6a5fbc301716f84682c4dae7e1691fcba413
[dc30ca638c88714942f282de4cd464336e41f8de]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc30ca638c88714942f282de4cd464336e41f8de
[dd63214e4877ab17811d0e1db6867cff6bb72e61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dd63214e4877ab17811d0e1db6867cff6bb72e61
[df628a72730e677e16a3053988983f752d71940a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/df628a72730e677e16a3053988983f752d71940a
[dfd2898d64411f280bbe7d04280a9c73d3a3b310]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd2898d64411f280bbe7d04280a9c73d3a3b310
[dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa
[dff2927698abcac250fd3f0df7910c02818f6776]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dff2927698abcac250fd3f0df7910c02818f6776
[e189fd21f8689048e404ddf19c279ad743203924]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e189fd21f8689048e404ddf19c279ad743203924
[e1dfb0a281d3d922ada33f53013accb2c765bd9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e1dfb0a281d3d922ada33f53013accb2c765bd9d
[e1e4606847459e742f9c5e51a860b8903b2bc5ce]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e1e4606847459e742f9c5e51a860b8903b2bc5ce
[e346e184d9ab0af7969a796ef4c43814267aa7a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e346e184d9ab0af7969a796ef4c43814267aa7a3
[e3c0e0a430d6e27060b00db05c17f01d68361547]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3c0e0a430d6e27060b00db05c17f01d68361547
[e3ca0e225065cf4fe610fd0f49748dc8cab48f71]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e3ca0e225065cf4fe610fd0f49748dc8cab48f71
[e42d021c20b90e50c464541fb3d358ac24ce3b3a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e42d021c20b90e50c464541fb3d358ac24ce3b3a
[e43d48bfd451fe4aac2f90b0e19a357bf5a1c1b9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e43d48bfd451fe4aac2f90b0e19a357bf5a1c1b9
[e46b1f943753dc0a5bf1b45b458f0fde643ebdf5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e46b1f943753dc0a5bf1b45b458f0fde643ebdf5
[e6931ed967f7ea795ecdecfaeeead533642445f5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e6931ed967f7ea795ecdecfaeeead533642445f5
[e7172e4519383c352ed147aa42b3aeca646a690e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e7172e4519383c352ed147aa42b3aeca646a690e
[e736a714f4b2a84e4b5d578c8789049c1bbc4df6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e736a714f4b2a84e4b5d578c8789049c1bbc4df6
[e80df38691580c8377c5e3fd30a02617765ee69d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e80df38691580c8377c5e3fd30a02617765ee69d
[e9b501fc77259d0c1c050bedc5a61c3516e4c307]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9b501fc77259d0c1c050bedc5a61c3516e4c307
[e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a
[ec82a5009bcf7a16aaa694eb478216b9567c87c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ec82a5009bcf7a16aaa694eb478216b9567c87c1
[ed3d1feb788121161ba66f9c1826a67ded941337]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed3d1feb788121161ba66f9c1826a67ded941337
[ed5c04ade1af13f2e22afc184336f9713f2b76e0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed5c04ade1af13f2e22afc184336f9713f2b76e0
[ee4515f1fd7e5161b5eab5bce0262971996f843f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee4515f1fd7e5161b5eab5bce0262971996f843f
[ee95b6364d51c7d8a6bd4259ceda8ec63d13f56b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ee95b6364d51c7d8a6bd4259ceda8ec63d13f56b
[eeb20bb8431bf75c9e2be3fbba8e64daafae3098]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eeb20bb8431bf75c9e2be3fbba8e64daafae3098
[ef73bd5c114916a2f430dcd9c26eb49ec98f3fcc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef73bd5c114916a2f430dcd9c26eb49ec98f3fcc
[ef80a0d5d844ce2b8ec80391305f0b71fc18b518]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ef80a0d5d844ce2b8ec80391305f0b71fc18b518
[effffe87f8390d5894ab8dcf1806b2dd5b54e493]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/effffe87f8390d5894ab8dcf1806b2dd5b54e493
[f00ebc4ccb3e82ae2d54787d9e39a6bce3044032]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f00ebc4ccb3e82ae2d54787d9e39a6bce3044032
[f0146fc0172a0f95718c22f531d43494740166f7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f0146fc0172a0f95718c22f531d43494740166f7
[f126782888b04516748ec2ce1740a1e8db2f75c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f126782888b04516748ec2ce1740a1e8db2f75c1
[f2199b30ca34e9d46d1e51436b2cfba7c9b2f64c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f2199b30ca34e9d46d1e51436b2cfba7c9b2f64c
[f355964d7b4c6bcc0d5cd726df4ff360f2adac23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f355964d7b4c6bcc0d5cd726df4ff360f2adac23
[f49fb33dab085714a8050d36442c04bf504f731e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f49fb33dab085714a8050d36442c04bf504f731e
[f56045aa6c147246f30635240835e92bea224520]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f56045aa6c147246f30635240835e92bea224520
[f58f506f17d6b76343d5bd814749259e3b380cc2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f58f506f17d6b76343d5bd814749259e3b380cc2
[f6a72ff1328766f733fe6314ecdbc1429bb57e61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6a72ff1328766f733fe6314ecdbc1429bb57e61
[f6b52fc20a8893ce30443bdd27f8da11108d0e17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6b52fc20a8893ce30443bdd27f8da11108d0e17
[f708e15eab0ca601699461565b7a396f84394526]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f708e15eab0ca601699461565b7a396f84394526
[f7b4533f180ccc94c27f8e42b9806199d147f5c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f7b4533f180ccc94c27f8e42b9806199d147f5c1
[f8f977d1bde282c350758aa2ebcca56eaef81c4a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f8f977d1bde282c350758aa2ebcca56eaef81c4a
[fc2a8379ad2f848990c749418ebe4123cacbcf8b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc2a8379ad2f848990c749418ebe4123cacbcf8b
[fc5482359615f1f1a0d83c4f34a1ca89834d38ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc5482359615f1f1a0d83c4f34a1ca89834d38ff
[fcf91c96ea0dd598594aec0fac23726426b4cd3b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fcf91c96ea0dd598594aec0fac23726426b4cd3b
[fda30e592981b402a192fe6f74ac36febdc946c8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda30e592981b402a192fe6f74ac36febdc946c8
[fda61f8ffc7ddd95556f4109b9e735cdde2c1b93]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda61f8ffc7ddd95556f4109b9e735cdde2c1b93
[febfd00d66ac8586584882ec6c7a5b2a97683571]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/febfd00d66ac8586584882ec6c7a5b2a97683571
[ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048
