# CHANGELOG<!-- omit in toc -->

<!--
Helper script `tools/changelog-link-gen.sh` can generate the addendum of markdown links for this CHANGELOG.md.
-->

Manual changelog for [super speedy syslog searcher](https://github.com/jtmoon79/super-speedy-syslog-searcher).

---

<!-- TODO per release: update TOC -->
<!-- Table of Contents updated by "Markdown All In One" extension for Visual Studio Code -->

- [Unreleased](#unreleased)
- [0.1.44 - 2022-12-29](#0144---2022-12-29)
  - [New](#new)
  - [Changes](#changes)
  - [Fixes](#fixes)
- [0.1.43 - 2022-12-26](#0143---2022-12-26)
  - [New](#new-1)
  - [Changes](#changes-1)
- [0.1.42 - 2022-12-19](#0142---2022-12-19)
  - [Changes](#changes-2)
- [0.1.41 - 2022-12-18](#0141---2022-12-18)
  - [Changes](#changes-3)
  - [Fixes](#fixes-1)
- [0.1.40 - 2022-11-22](#0140---2022-11-22)
  - [New](#new-2)
  - [Changes](#changes-4)
- [0.1.39 - 2022-10-19](#0139---2022-10-19)
  - [Changes](#changes-5)
- [0.1.38 - 2022-10-16](#0138---2022-10-16)
  - [New](#new-3)
  - [Changes](#changes-6)
- [0.0.37 - 2022-10-12](#0037---2022-10-12)
  - [New](#new-4)
  - [Changes](#changes-7)
- [0.0.36 - 2022-10-10](#0036---2022-10-10)
  - [New](#new-5)
  - [Fixes](#fixes-2)
  - [Changes](#changes-8)
- [0.0.35 - 2022-10-09](#0035---2022-10-09)
  - [New](#new-6)
  - [Fixes](#fixes-3)
- [0.0.34 - 2022-10-07](#0034---2022-10-07)
  - [New](#new-7)
  - [Fixes](#fixes-4)
- [0.0.33 - 2022-09-21](#0033---2022-09-21)
  - [New](#new-8)
- [0.0.32 - 2022-09-20](#0032---2022-09-20)
  - [New](#new-9)
  - [Fixes](#fixes-5)
- [0.0.31 - 2022-09-19](#0031---2022-09-19)
  - [New](#new-10)
- [0.0.30 - 2022-09-18](#0030---2022-09-18)
  - [New](#new-11)
  - [Changes](#changes-9)
- [0.0.29 - 2022-09-17](#0029---2022-09-17)
  - [Changes](#changes-10)
- [0.0.28 - 2022-09-17](#0028---2022-09-17)
  - [New](#new-12)
  - [Changes](#changes-11)
  - [Fixes](#fixes-6)
- [0.0.27 - 2022-09-16](#0027---2022-09-16)
  - [New](#new-13)
  - [Changes](#changes-12)
- [0.0.26 - 2022-08-12](#0026---2022-08-12)
  - [New](#new-14)
  - [Changes](#changes-13)
  - [Fixes](#fixes-7)
- [0.0.25 - 2022-07-28](#0025---2022-07-28)
  - [New](#new-15)
  - [Changes](#changes-14)
  - [Fixes](#fixes-8)
- [0.0.24 - 2022-07-20](#0024---2022-07-20)
  - [New](#new-16)
  - [Changes](#changes-15)
- [0.0.23 - 2022-07-12](#0023---2022-07-12)
  - [New](#new-17)
  - [Changes](#changes-16)
  - [Fixes](#fixes-9)
- [0.0.22 - 2022-07-10](#0022---2022-07-10)
  - [New](#new-18)
  - [Fixes](#fixes-10)
  - [Changes](#changes-17)
- [0.0.21 - 2022-06-24](#0021---2022-06-24)
  - [New](#new-19)
  - [Fixes](#fixes-11)

---

<!--
TODO per release:

1. Developers must manually create the sections. Do not create empty sections.
2. Developers must manually prefix categories.

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

<!-- TODO per release: Change Version in the URL -->
[unreleased-diff](https://github.com/jtmoon79/super-speedy-syslog-searcher/compare/0.1.38...HEAD)

---

<!-- TODO per release: Add Section(s) -->

## 0.1.44 - 2022-12-29

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

## 0.1.43 - 2022-12-26

[0.1.42...0.1.43]

### New

- (CI) add codecov.yml ([bd49cdc8220e8adcfea71f04c6ebcfb51946336b])

### Changes

- (BUILD) adjust "filetime" dependency ([ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048])
- (DOCS) update README ([cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870])
- (BIN) update --help for `--blocksz` ([2157861027eff2cade51aa950a6a4300e86a1e50])

## 0.1.42 - 2022-12-19

[0.1.41...0.1.42]

### Changes

- (BIN) (BUILD) update clap from 3 to 4 ([f58f506f17d6b76343d5bd814749259e3b380cc2])
- (BUILD) cargo update ([41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd])

## 0.1.41 - 2022-12-18

[0.1.40...0.1.41]

### Changes

- (TOOLS) gen-log.sh add option for extra lines ([610785f3d98a4032fe7053076f9db45d4c1d1717])
- (BIN) re-arrange trailing summary print of datetimes ([a80facf0bb435346b0c8c3a05d22b8e428ba680f])
- (LIB) for logs without year, skip processing outside filters [Issue #65] ([33418d0311fc75fa7fda97ac621ddf2da493c128])
- (PROJECT) add gen-20-2-2-faces.log ([8a47ad83cd68c7eec60db4ff734f8ead3d54b977])

### Fixes

- (LIB)(BIN) fix drop, add more summary stats, tests [Issue #66] ([916259ba70c903d2b2d85b4bd3eddffa98cec370])

## 0.1.40 - 2022-11-22

[0.1.39...0.1.40]

### New

- (BIN) add CLI option `--prepend-separator` ([467b14dbc59a60a808e7a71a1083f2490cf31d48])

### Changes

- (BIN) add summary _syslines stored high_ ([d1f5895f1e5a55cbbcbfc4072bbde53a7a85fc
## 0.1.39 - 2022-10-19

[0.1.38...0.1.39]

### Changes

- (LIB) Cargo.toml rust MSRV 1.61.0 ([3c4e8b1b37415ad0662019d1792525ab0b00a8f9]) 
- (BUILD) rust.yml add job_rust_versions, jo_rust_msrv_os 1.61.0 ([94d6862e0d558e69f0e5b07db5a63ad7700d515b])
- (LIB) const-str downgrade to 0.4.3 ([7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13])
- (DOCS) README add more Windows examples, wording ([db5b6a5fbc301716f84682c4dae7e1691fcba413])
- (LIB) (BIN) codespell fixes ([b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5]) ([aaaf78e17cfeeea087fe9562fc65907b3847bc9e])

---

## 0.1.38 - 2022-10-16

[0.0.37...0.1.38]

### New

- (BIN) bin.rs --summary print -a -b as UTC ([186c74720db2b33e5c0df17ee690eddcdee360a7])
- (BIN) bin.rs allow relative offset datetimes [Issue #35] ([4eef6221708137928458ed8445b4f67196500082])

### Changes

- (DOCS) README add Windows log snippets, tweak wording ([65c007844cc6c275b86b36a2ff1b48340622a681])

---

## 0.0.37 - 2022-10-12

[0.0.36...0.0.37]

### New

- (LIB) datetime.rs patterns for Windows compsetup.log mrt.log ([0f225cee04b5443a58369b95bc8e6f10ed3f6401])

### Changes

- (LIB) blockreader.rs eprintln path ([3e1607f076afe7a6e10578776a07d3feb0a2b9a8])
- (TEST) add logs Windows10Pro ([1c746c24b7e0ad7e7481cce626fb6488eb0076d6])

---

<!-- TODO per release: Add Section(s) -->

## 0.0.36 - 2022-10-10

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

## 0.0.35 - 2022-10-09

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

## 0.0.34 - 2022-10-07

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

## 0.0.33 - 2022-09-21

[0.0.32...0.0.33]

### New

- (BIN) bin.rs allow user opt dt format string [Issue #28] ([6660f686f02ca2d98c9cdfe3c72cc906e446df1f])
- (DEBUG) use `si_trace_print` 0.2.5 ([fc5482359615f1f1a0d83c4f34a1ca89834d38ff])

---

## 0.0.32 - 2022-09-20

[0.0.31...0.0.32]

### New

- (TEST) datetime_tests.rs add tests cases datetime_parse_from_str ([c9bc19ecd6cad88742cfa3758e48fd606f489220])

### Fixes

- (LIB) datetime.rs fix copying fractional [Issue #23] ([764689fe0693c6a8588d13cde1c73f42e08b2a39])

---

## 0.0.31 - 2022-09-19

[0.0.30...0.0.31]

### New

- (DOCS) improved README badges ([30553b7989b55c802704c42deefe9424347092ee]) ([b5d4d91779599bae9fc15d78c5e3db3f4a43f18b]) ([17cd497307d04f3d8a9b058a72e3ea415a9a9f89])

---

## 0.0.30 - 2022-09-18

[0.0.29...0.0.30]

### New

- (LIB) allow -t %Z or %z, allow Zulu ([a71c5e81761deb547c315296004167e13f82fe9b])

### Changes

- (TEST) add log dtf11-A.log ([56078e8bb713fa861ccf9ebd1a58415ee6173819])
- (BIN) bin.rs stack_offset_set only debug builds ([d2158ee2b1b23a68b3c4dd764863acadec08d6bb])

---

## 0.0.29 - 2022-09-17

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

## 0.0.28 - 2022-09-17

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

## 0.0.27 - 2022-09-16

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

## 0.0.26 - 2022-08-12

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

## 0.0.25 - 2022-07-28

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


## 0.0.24 - 2022-07-20

[0.0.23...0.0.24]

### New

- (LIB) handle tar files ([adf400700122f4eb23fd63971b3f048e014d1781])

### Changes

- (LIB) datetime transforms `%b` `%B` to `%m` ([22980abf582aa61c5e4c9ce94d8298997fb5bbbc])

---

## 0.0.23 - 2022-07-12

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

## 0.0.22 - 2022-07-10

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

## 0.0.21 - 2022-06-24

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
[Issue #23]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/23
[Issue #28]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/28
[Issue #35]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/35
[Issue #38]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/38
[Issue #53]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/53
[Issue #57]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/57
[Issue #65]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/65
[Issue #66]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/66
[Issue #67]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/67
[Issue #69]: https://github.com/jtmoon79/super-speedy-syslog-searcher/issues/69
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
[001f0c3db2c5751a35946f572aca6bf07c9efcaf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/001f0c3db2c5751a35946f572aca6bf07c9efcaf
[003b29bab508b32750cb303c70db9dc75cc04eab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/003b29bab508b32750cb303c70db9dc75cc04eab
[01f395903cff248be11ecf6f12974a3951aa7e92]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/01f395903cff248be11ecf6f12974a3951aa7e92
[031434f4d9dfb4e0f8190a720f8db57a3772e3a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/031434f4d9dfb4e0f8190a720f8db57a3772e3a2
[0579522ff7609e22c14b33aa6c6a70cec6372226]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0579522ff7609e22c14b33aa6c6a70cec6372226
[07214abde6479431cc1a9f87f50f3b713e5ea503]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07214abde6479431cc1a9f87f50f3b713e5ea503
[0743e4157daa108569d99746d8a6314cfe6e0248]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0743e4157daa108569d99746d8a6314cfe6e0248
[07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/07baf6df44ec3ccd2da43f3c5cb9f5ef30a6b0e8
[09a04c14146af1916aeda14e8134d02baf088d5d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09a04c14146af1916aeda14e8134d02baf088d5d
[09df0b6551fec2ea22cee7dca2cd308cf11b531a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/09df0b6551fec2ea22cee7dca2cd308cf11b531a
[0a46b5aee7eb99e19a9a2a91ed81d759978b6024]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a46b5aee7eb99e19a9a2a91ed81d759978b6024
[0a5ce1e0011920909cfa5bc022f95b3a502ff244]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0a5ce1e0011920909cfa5bc022f95b3a502ff244
[0ca431ce8b510b6714420a8954f587eccd84a01d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0ca431ce8b510b6714420a8954f587eccd84a01d
[0f225cee04b5443a58369b95bc8e6f10ed3f6401]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f225cee04b5443a58369b95bc8e6f10ed3f6401
[0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/0f4ac9ae4cb4d11247a40cf1a3c09f78a9a42399
[17cd497307d04f3d8a9b058a72e3ea415a9a9f89]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17cd497307d04f3d8a9b058a72e3ea415a9a9f89
[17f89020870b8bc8ad8322e314c187b6e0836226]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/17f89020870b8bc8ad8322e314c187b6e0836226
[186c74720db2b33e5c0df17ee690eddcdee360a7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/186c74720db2b33e5c0df17ee690eddcdee360a7
[1b88a1e35a66004ea5016525bcbb1e125aa64db9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1b88a1e35a66004ea5016525bcbb1e125aa64db9
[1bf2784185df479a3a17975f773e3a505f735e26]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1bf2784185df479a3a17975f773e3a505f735e26
[1c58a778dc5bd05e455ea25af60e8600b8b72857]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c58a778dc5bd05e455ea25af60e8600b8b72857
[1c746c24b7e0ad7e7481cce626fb6488eb0076d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1c746c24b7e0ad7e7481cce626fb6488eb0076d6
[1cfc72e99382ab47b55c9410ab531c0baf8ac46e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1cfc72e99382ab47b55c9410ab531c0baf8ac46e
[1de420a5907cf62ae91a06732a8ef43e01f17598]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/1de420a5907cf62ae91a06732a8ef43e01f17598
[2157861027eff2cade51aa950a6a4300e86a1e50]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2157861027eff2cade51aa950a6a4300e86a1e50
[21745ee99eb04a4204164825ca5c50e6f8b34fee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/21745ee99eb04a4204164825ca5c50e6f8b34fee
[22980abf582aa61c5e4c9ce94d8298997fb5bbbc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/22980abf582aa61c5e4c9ce94d8298997fb5bbbc
[2343d26300c5a139066081648054e5e299eb8a80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2343d26300c5a139066081648054e5e299eb8a80
[238df6c7b1b569f724778c85bfead20cb14be59d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/238df6c7b1b569f724778c85bfead20cb14be59d
[25e4bf65d9d5af300b99092e189f0caea3164f5f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/25e4bf65d9d5af300b99092e189f0caea3164f5f
[2a1b10859a31649a7ef31db9474e3a6ed526c9a4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2a1b10859a31649a7ef31db9474e3a6ed526c9a4
[2da339822a4f62266149b8d53925840c0860c9a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/2da339822a4f62266149b8d53925840c0860c9a2
[30553b7989b55c802704c42deefe9424347092ee]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/30553b7989b55c802704c42deefe9424347092ee
[33418d0311fc75fa7fda97ac621ddf2da493c128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33418d0311fc75fa7fda97ac621ddf2da493c128
[33a492b9c01c57a71191d7f1b46d457d5ff67059]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/33a492b9c01c57a71191d7f1b46d457d5ff67059
[34595fe2693385b0cdff69ecf6306071d058b638]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/34595fe2693385b0cdff69ecf6306071d058b638
[3562638d37272b2befa7f9007307fd4088cdd00c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3562638d37272b2befa7f9007307fd4088cdd00c
[3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3963e070fd8849ce327d9cdb4ef7bbbe52d0d7e2
[3980d5b67bbd371d84cbb313f51e950dae436d54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3980d5b67bbd371d84cbb313f51e950dae436d54
[3c4e8b1b37415ad0662019d1792525ab0b00a8f9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c4e8b1b37415ad0662019d1792525ab0b00a8f9
[3c7984d49df0d91037729a45c24a2a7b5a109687]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3c7984d49df0d91037729a45c24a2a7b5a109687
[3d78b0d0b6918dab784bbe2332b3a26928bb8f90]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3d78b0d0b6918dab784bbe2332b3a26928bb8f90
[3e1607f076afe7a6e10578776a07d3feb0a2b9a8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3e1607f076afe7a6e10578776a07d3feb0a2b9a8
[3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/3ee20b9c743ac1ab72652b4ea4ab61bd722d8a16
[41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/41bb25a10b5bc70c228f9f5930d4f0aaba9eafbd
[44291c749bfae647cf130fdc298dd2cc5d1876ba]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/44291c749bfae647cf130fdc298dd2cc5d1876ba
[467b14dbc59a60a808e7a71a1083f2490cf31d48]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/467b14dbc59a60a808e7a71a1083f2490cf31d48
[48687e8a65af56cf9c6279702ccaa6a66c127a06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48687e8a65af56cf9c6279702ccaa6a66c127a06
[48bd3aa9c24f5420f54ffdddabd061ec5a25d55b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/48bd3aa9c24f5420f54ffdddabd061ec5a25d55b
[4b51b30d598a6e076f3d2a8b9d3e170deea1325f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b51b30d598a6e076f3d2a8b9d3e170deea1325f
[4b784a723b8c02c7bdb4b51e7d7b76147f97d569]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4b784a723b8c02c7bdb4b51e7d7b76147f97d569
[4eef6221708137928458ed8445b4f67196500082]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4eef6221708137928458ed8445b4f67196500082
[4fda60f22505ebba9ff86873386d0524d364765c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/4fda60f22505ebba9ff86873386d0524d364765c
[53892a3a2d46c3b7dcad3b0fd7b4141118485e9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/53892a3a2d46c3b7dcad3b0fd7b4141118485e9e
[55113bc5705d5c9ace1da6bde8b05c1260ddb935]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55113bc5705d5c9ace1da6bde8b05c1260ddb935
[55a1e55ed94f8e8a4202098c1fd4f85e337bfae4]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/55a1e55ed94f8e8a4202098c1fd4f85e337bfae4
[56078e8bb713fa861ccf9ebd1a58415ee6173819]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/56078e8bb713fa861ccf9ebd1a58415ee6173819
[5caa8dd6b7f8f2735366a23ab1005df89aaf565f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5caa8dd6b7f8f2735366a23ab1005df89aaf565f
[5dfc932d8f62e295f93accafb98c533fd8e39625]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5dfc932d8f62e295f93accafb98c533fd8e39625
[5e9243e125e7f075ac533b6cd68fdcbef12368cf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5e9243e125e7f075ac533b6cd68fdcbef12368cf
[5f93e4ad56fdbda6b5ceeaeca94848063064cc9a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/5f93e4ad56fdbda6b5ceeaeca94848063064cc9a
[607a23c00aff0d9b34fb3d678bdfd5c14290582d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/607a23c00aff0d9b34fb3d678bdfd5c14290582d
[610785f3d98a4032fe7053076f9db45d4c1d1717]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/610785f3d98a4032fe7053076f9db45d4c1d1717
[61f15e13d086a5d6c0e5a18d44c730ebe77a046a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/61f15e13d086a5d6c0e5a18d44c730ebe77a046a
[657948516a05c40cd0d9c35dc639d05eeafa5dc5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/657948516a05c40cd0d9c35dc639d05eeafa5dc5
[65c007844cc6c275b86b36a2ff1b48340622a681]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/65c007844cc6c275b86b36a2ff1b48340622a681
[6659509095d19163bd65bd24a9a554cf25207395]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6659509095d19163bd65bd24a9a554cf25207395
[6660f686f02ca2d98c9cdfe3c72cc906e446df1f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6660f686f02ca2d98c9cdfe3c72cc906e446df1f
[66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/66eea98eb83cb5d80ff5ce094c8da7b63e8c74d6
[676633a72f464a1f71b369281207390fb1c2efd5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/676633a72f464a1f71b369281207390fb1c2efd5
[6805e2b9257cecb545417531a008ec139a0b5c54]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6805e2b9257cecb545417531a008ec139a0b5c54
[6955a7b5c389a9b16651bf7e2350e12df2bc22a2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6955a7b5c389a9b16651bf7e2350e12df2bc22a2
[6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/6d64fd6d8ee1b5338877004d22ecfaf18ed47ba7
[7109c46d835f4d6f32b6284681a6286b68179abc]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7109c46d835f4d6f32b6284681a6286b68179abc
[7185ba477d0d184f9cdf28eb485e3ec4e5963f3b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7185ba477d0d184f9cdf28eb485e3ec4e5963f3b
[764689fe0693c6a8588d13cde1c73f42e08b2a39]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/764689fe0693c6a8588d13cde1c73f42e08b2a39
[78581dba9d33c9565fa25f0a829ca383471335f2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/78581dba9d33c9565fa25f0a829ca383471335f2
[79c1ea1edbed94e3376aed37b382d069144d6fab]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/79c1ea1edbed94e3376aed37b382d069144d6fab
[7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7cdaa4b6ac2c12f3829f345c8c56bd7bf6c19b13
[7f751c12debde6b2dcd7377d880b20d2aa834f40]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7f751c12debde6b2dcd7377d880b20d2aa834f40
[7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/7fb6abe6a51d0fa63c6ef1a543d5888cd43d5550
[81c437b02b967b56dcb9f5fa0a25b083dfa3ed25]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/81c437b02b967b56dcb9f5fa0a25b083dfa3ed25
[84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/84cc63c2d8c1398a4aa11da4e4e2d07abed4c04b
[880e35aeedfb5449626a03c9131a1ccd33e017e3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/880e35aeedfb5449626a03c9131a1ccd33e017e3
[8a47ad83cd68c7eec60db4ff734f8ead3d54b977]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8a47ad83cd68c7eec60db4ff734f8ead3d54b977
[8cd40b522d2e87dd69dd21704c5f128d6d05847b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8cd40b522d2e87dd69dd21704c5f128d6d05847b
[8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8d5e6860ed3b6b5c3743bf5d9a5122a78cdccb3c
[8e3b72a7c1e70dad9eacc62cb3171754799c79a6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8e3b72a7c1e70dad9eacc62cb3171754799c79a6
[8f1437483337f24a4c728b61d1754f9455ee0f5b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/8f1437483337f24a4c728b61d1754f9455ee0f5b
[908b2f594fdbc1aa51313bba5f26db74ee332a4a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/908b2f594fdbc1aa51313bba5f26db74ee332a4a
[916259ba70c903d2b2d85b4bd3eddffa98cec370]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/916259ba70c903d2b2d85b4bd3eddffa98cec370
[943ad6258c6d01c3df3f97e35b7d0a2aa4f00136]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/943ad6258c6d01c3df3f97e35b7d0a2aa4f00136
[94d6862e0d558e69f0e5b07db5a63ad7700d515b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/94d6862e0d558e69f0e5b07db5a63ad7700d515b
[97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/97cf45e94786b87b5a2d3fb2ecf2e696aeb4d1d9
[9957cc56452652f87ac037175d3b16f273a735ea]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9957cc56452652f87ac037175d3b16f273a735ea
[9c5fa576899d1529b06acf89221d44d262092d04]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9c5fa576899d1529b06acf89221d44d262092d04
[9d9179cf63c4167ac46b5c398b2c6b718ea9a022]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9d9179cf63c4167ac46b5c398b2c6b718ea9a022
[9ddaaeedecdd175672c38ba3d39c7521f08acc68]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/9ddaaeedecdd175672c38ba3d39c7521f08acc68
[a13786623e5b9117418dc6ff86c1f0519e9074f0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a13786623e5b9117418dc6ff86c1f0519e9074f0
[a1e1a680278843d4f871f5556bee679282a8d268]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a1e1a680278843d4f871f5556bee679282a8d268
[a4fd91f4b1340a754754b8bec841eb60102988bf]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a4fd91f4b1340a754754b8bec841eb60102988bf
[a503554d9c0bbae7751b1e448156a7dc43f32def]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a503554d9c0bbae7751b1e448156a7dc43f32def
[a71c5e81761deb547c315296004167e13f82fe9b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a71c5e81761deb547c315296004167e13f82fe9b
[a80facf0bb435346b0c8c3a05d22b8e428ba680f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a80facf0bb435346b0c8c3a05d22b8e428ba680f
[a82e25b56c80e37c5ea6450c4a27a9ff1feb021b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/a82e25b56c80e37c5ea6450c4a27a9ff1feb021b
[aa3992cc919c644dc7fe3bc41abc2dd970fe3d2e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa3992cc919c644dc7fe3bc41abc2dd970fe3d2e
[aa5bdbbcdbd2d36b08f11c0a252603526b7adce8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aa5bdbbcdbd2d36b08f11c0a252603526b7adce8
[aaaf78e17cfeeea087fe9562fc65907b3847bc9e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aaaf78e17cfeeea087fe9562fc65907b3847bc9e
[aab71b50e4464cae19f1add8b28613260345d9db]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aab71b50e4464cae19f1add8b28613260345d9db
[aaf976b84513bcdb2395fab5349fc035e4601068]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/aaf976b84513bcdb2395fab5349fc035e4601068
[ab579207ea14141d3d4327f39b5fd23830a89f3a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ab579207ea14141d3d4327f39b5fd23830a89f3a
[ac8d29bb53de9b0bc06572f85073a1ac06f54087]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ac8d29bb53de9b0bc06572f85073a1ac06f54087
[acc34edc4502c691381df03a3bf9c2aebde1a038]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/acc34edc4502c691381df03a3bf9c2aebde1a038
[adf400700122f4eb23fd63971b3f048e014d1781]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/adf400700122f4eb23fd63971b3f048e014d1781
[af46851919ced5582dd8d6c5b236edd3ac078061]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/af46851919ced5582dd8d6c5b236edd3ac078061
[b088be725c367aabae07d4b60553693a5c2ddd80]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b088be725c367aabae07d4b60553693a5c2ddd80
[b2d6de5072f1506077fa649b15912b7cb3064211]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b2d6de5072f1506077fa649b15912b7cb3064211
[b5505730100a9780877eb3e1cb4d280f02845863]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5505730100a9780877eb3e1cb4d280f02845863
[b55341ecd717344211bd79557f56f7fecaad2479]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b55341ecd717344211bd79557f56f7fecaad2479
[b5d4d91779599bae9fc15d78c5e3db3f4a43f18b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b5d4d91779599bae9fc15d78c5e3db3f4a43f18b
[b8deef3439f8e8b9a949a0a1cfa16d2c027c391f]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b8deef3439f8e8b9a949a0a1cfa16d2c027c391f
[b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/b9d4c2c24c13c8f629c7ca6cab36941a1dc7a4b5
[bc4112866bb713538fc48c209408313c634306b2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bc4112866bb713538fc48c209408313c634306b2
[bd44896a30627bafefa64c1cbc78229113130b9d]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd44896a30627bafefa64c1cbc78229113130b9d
[bd49cdc8220e8adcfea71f04c6ebcfb51946336b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/bd49cdc8220e8adcfea71f04c6ebcfb51946336b
[c332a73363492a1e1874e68fc0c12e3bfd2b96ae]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c332a73363492a1e1874e68fc0c12e3bfd2b96ae
[c35066cd2cc01344259f00559186fbd1a12db527]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c35066cd2cc01344259f00559186fbd1a12db527
[c8328f3ab256bf76a92b205f8eeebc49447bd25e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8328f3ab256bf76a92b205f8eeebc49447bd25e
[c8fc525dff93e1b29c0df61bf6cc593376910043]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c8fc525dff93e1b29c0df61bf6cc593376910043
[c9bc19ecd6cad88742cfa3758e48fd606f489220]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9bc19ecd6cad88742cfa3758e48fd606f489220
[c9dc70a51be61bc46b43082e7227f873cb77ac10]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/c9dc70a51be61bc46b43082e7227f873cb77ac10
[ca1c967a1dd169b73f3002f120c40c7127060041]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ca1c967a1dd169b73f3002f120c40c7127060041
[cae987706e31a6c223e5af997fee32b537714efd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cae987706e31a6c223e5af997fee32b537714efd
[cb74da327e27b73e9724d8a28aafc164e6c9e0df]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb74da327e27b73e9724d8a28aafc164e6c9e0df
[cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/cb8ed82e9ee18dc1b0f3ffdd2c22d99402a8c870
[ced4667fd5f16682a46e70d435a9a473885c70b6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ced4667fd5f16682a46e70d435a9a473885c70b6
[d2158ee2b1b23a68b3c4dd764863acadec08d6bb]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d2158ee2b1b23a68b3c4dd764863acadec08d6bb
[d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d3f5d8a4cd60ec6007977e7ebe4558c4a14789cd
[d5af77deed057d599fd1c4b5c1f6222a7edba4c3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d5af77deed057d599fd1c4b5c1f6222a7edba4c3
[d70104fb19ee3e133188a14d49f2c57ab0a55e06]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d70104fb19ee3e133188a14d49f2c57ab0a55e06
[d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d75fdfc0fb7b34f4e6b5ac2cfbcbfca7df0ccf59
[d882f968ae9011b112cb8f195171e5357747a6af]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d882f968ae9011b112cb8f195171e5357747a6af
[d8d56414c28f5ca7ba2db10420c1805270d80d7b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8d56414c28f5ca7ba2db10420c1805270d80d7b
[d8faf4fd010e303dad42c8a0a51520c03fd197b8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d8faf4fd010e303dad42c8a0a51520c03fd197b8
[d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/d97f0ab7ba5ef0cfd4a7ea0ed9cb21f3770fc5da
[db2e8f3cf4db912d32e74fcbdf09094c8b2f5128]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db2e8f3cf4db912d32e74fcbdf09094c8b2f5128
[db5b6a5fbc301716f84682c4dae7e1691fcba413]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/db5b6a5fbc301716f84682c4dae7e1691fcba413
[dc30ca638c88714942f282de4cd464336e41f8de]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dc30ca638c88714942f282de4cd464336e41f8de
[dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dfd60d4b29ce3ba0afe581c746d643cc5a6eccfa
[dff2927698abcac250fd3f0df7910c02818f6776]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/dff2927698abcac250fd3f0df7910c02818f6776
[e189fd21f8689048e404ddf19c279ad743203924]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e189fd21f8689048e404ddf19c279ad743203924
[e346e184d9ab0af7969a796ef4c43814267aa7a3]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e346e184d9ab0af7969a796ef4c43814267aa7a3
[e7172e4519383c352ed147aa42b3aeca646a690e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e7172e4519383c352ed147aa42b3aeca646a690e
[e736a714f4b2a84e4b5d578c8789049c1bbc4df6]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e736a714f4b2a84e4b5d578c8789049c1bbc4df6
[e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/e9ea121ac4a53e44e02f63f4f5ffee16c83dd72a
[ec82a5009bcf7a16aaa694eb478216b9567c87c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ec82a5009bcf7a16aaa694eb478216b9567c87c1
[ed3d1feb788121161ba66f9c1826a67ded941337]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed3d1feb788121161ba66f9c1826a67ded941337
[ed5c04ade1af13f2e22afc184336f9713f2b76e0]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ed5c04ade1af13f2e22afc184336f9713f2b76e0
[eeb20bb8431bf75c9e2be3fbba8e64daafae3098]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/eeb20bb8431bf75c9e2be3fbba8e64daafae3098
[effffe87f8390d5894ab8dcf1806b2dd5b54e493]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/effffe87f8390d5894ab8dcf1806b2dd5b54e493
[f00ebc4ccb3e82ae2d54787d9e39a6bce3044032]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f00ebc4ccb3e82ae2d54787d9e39a6bce3044032
[f49fb33dab085714a8050d36442c04bf504f731e]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f49fb33dab085714a8050d36442c04bf504f731e
[f56045aa6c147246f30635240835e92bea224520]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f56045aa6c147246f30635240835e92bea224520
[f58f506f17d6b76343d5bd814749259e3b380cc2]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f58f506f17d6b76343d5bd814749259e3b380cc2
[f6a72ff1328766f733fe6314ecdbc1429bb57e61]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6a72ff1328766f733fe6314ecdbc1429bb57e61
[f6b52fc20a8893ce30443bdd27f8da11108d0e17]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f6b52fc20a8893ce30443bdd27f8da11108d0e17
[f708e15eab0ca601699461565b7a396f84394526]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f708e15eab0ca601699461565b7a396f84394526
[f7b4533f180ccc94c27f8e42b9806199d147f5c1]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/f7b4533f180ccc94c27f8e42b9806199d147f5c1
[fc2a8379ad2f848990c749418ebe4123cacbcf8b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc2a8379ad2f848990c749418ebe4123cacbcf8b
[fc5482359615f1f1a0d83c4f34a1ca89834d38ff]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fc5482359615f1f1a0d83c4f34a1ca89834d38ff
[fcf91c96ea0dd598594aec0fac23726426b4cd3b]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fcf91c96ea0dd598594aec0fac23726426b4cd3b
[fda30e592981b402a192fe6f74ac36febdc946c8]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/fda30e592981b402a192fe6f74ac36febdc946c8
[febfd00d66ac8586584882ec6c7a5b2a97683571]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/febfd00d66ac8586584882ec6c7a5b2a97683571
[ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048]: https://github.com/jtmoon79/super-speedy-syslog-searcher/commit/ff2cd81bbd533c59df2c8bac3c6ff2afea4c1048
