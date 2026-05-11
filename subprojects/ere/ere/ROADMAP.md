# Roadmap

## Features

-   [ ] Improved definition and calling convention for regular expressions: using a `struct` or `enum` to represent the regex and the engine, rather than a function pointer in the `Regex` struct.
-   [ ] POSIX-compilant ambiguous submatching rules: we currently implement the Perl-like greedy submatching when submatches are ambiguous (this should only affect `exec`, not `test`, and only more complex regexes). While known implementations are more expensive, I plan on also supporting the POSIX rules.
-   [ ] Case-insensitive mode: while regexes can be modified to support case-insensitivity (and this can also be done on ascii by just lower-casing the text first), I intend to implement case-insensitive mode.
-   [x] Non-capturing groups: `(?:non-capturing)` — not POSIX ERE standard-compliant, but a commonly used feature that improves performance by not tracking matches. Works without a feature flag in `compile_regex!`; requires `unstable-attr-regex` for the struct form.
-   [x] Named capture groups: `(?<name>...)` — bind a capture group to a name, usable with `(?<name>...)` syntax in `compile_regex!` and with named struct fields in `#[regex(...)]` (requires `unstable-attr-regex`).

## Performance Improvements

-   [x] `u8`-based engines: performance improvements can be made for many regexes (such as those with only ascii) by using `u8`s instead of extracting variably one to four-byte `char`s from strings.
-   [ ] Additional limited-feature engines. This is relatively open-ended, but major improvements can be made for regexes with certain properties.
    -   [x] One-pass engine: performance improvements can be made for 'one-pass' regexes where there is never more than one path to take when consuming the next symbol.
    -   [x] TDFA engine: running a DFA on a regex is O(n) in terms of input size, though compilation time and binary size is worst-case exponential, so
