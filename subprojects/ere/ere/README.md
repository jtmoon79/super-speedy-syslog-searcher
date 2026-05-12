***Ripped from [https://github.com/2kai2kai2/ere/pull/4](https://github.com/2kai2kai2/ere/pull/4) for exclusive use in [super-speedy-syslog-searcher](https://github.com/jtmoon79/super-speedy-syslog-searcher)***

[![Crates.io Version](https://img.shields.io/crates/v/ere)](https://crates.io/crates/ere)
[![docs.rs](https://img.shields.io/docsrs/ere)](https://docs.rs/ere/latest/ere/)

This crate provides tools for compiling and using regular expressions.
It is intended as a simple but compiler-checked version of the [`regex`](https://crates.io/crates/regex) crate, as it does regular expression compilation at compile-time, but only supports [POSIX Extended Regular Expressions](https://en.wikibooks.org/wiki/Regular_Expressions/POSIX-Extended_Regular_Expressions)*.

## Usage

```rust
use ere::prelude::*;

const PHONE_REGEX: Regex<2> = compile_regex!(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$");
fn test() {
    assert!(PHONE_REGEX.test("012-345-6789"));
    assert!(PHONE_REGEX.test("987-654-3210"));
    assert!(PHONE_REGEX.test("+1 555-555-5555"));
    assert!(PHONE_REGEX.test("123-555-9876"));

    assert!(!PHONE_REGEX.test("abcd"));
    assert!(!PHONE_REGEX.test("0123456789"));
    assert!(!PHONE_REGEX.test("012--345-6789"));
    assert!(!PHONE_REGEX.test("(555) 555-5555"));
    assert!(!PHONE_REGEX.test("1 555-555-5555"));
}

const COLOR_REGEX: Regex<5> = compile_regex!(
    r"^#?([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})([[:xdigit:]]{2})?$"
);
fn exec() {
    assert_eq!(
        COLOR_REGEX.exec("#000000"),
        Some([
            Some("#000000"),
            Some("00"),
            Some("00"),
            Some("00"),
            None,
        ]),
    );
    assert_eq!(
        COLOR_REGEX.exec("1F2e3D"),
        Some([
            Some("1F2E3D"),
            Some("1F"),
            Some("2e"),
            Some("3D"),
            None,
        ]),
    );
    assert_eq!(
        COLOR_REGEX.exec("ffffff80"),
        Some([
            Some("ffffff80"),
            Some("ff"),
            Some("ff"),
            Some("ff"),
            Some("80"),
        ]),
    );

    assert_eq!(PHONE_REGEX.exec("green"), None);
    assert_eq!(PHONE_REGEX.exec("%FFFFFF"), None);
    assert_eq!(PHONE_REGEX.exec("#2"), None);
}
```

To minimize memory overhead and binary size, it is recommended to create a single instance of each regular expression (using a `const` variable) rather than creating multiple.

*Some features are not fully implemented, such as POSIX-mode ambiguous submatch rules (we currently use greedy mode, which is the much more common and efficient method). See the [roadmap](ROADMAP.md) for more details.

## Named Groups and Non-Capturing Groups

`ere` supports two extensions beyond standard POSIX ERE syntax.

**Non-capturing groups** `(?:...)` group sub-expressions without allocating a capture-group slot, so `N` stays smaller:

```rust
use ere::prelude::*;

// (?:foo) does not count as a capture group; only (bar) does.
// So N = 2 (group 0 = whole match, group 1 = "bar").
const RE: Regex<2> = compile_regex!(r"(?:foo)(bar)");

fn example() {
    assert_eq!(RE.exec("foobar"), Some([Some("foobar"), Some("bar")]));
}
```

**Named capture groups** `(?<name>...)` attach a name to a group and behave identically to unnamed groups at the `Regex` level:

```rust
use ere::prelude::*;

// (?<area>[0-9]{3}) is capture group 1; (?<local>[0-9]{4}) is capture group 2.
const RE: Regex<3> = compile_regex!(r"(?<area>[0-9]{3})-(?<local>[0-9]{4})");

fn example() {
    assert_eq!(RE.exec("555-1234"), Some([Some("555-1234"), Some("555"), Some("1234")]));
}
```

**Named struct field binding** is available when using the `#[regex(...)]` attribute macro (requires the `unstable-attr-regex` feature). Fields can be named after capture groups or annotated with `#[group(N)]` to bind by index:

```rust,ignore
use ere::regex;

#[derive(Debug, PartialEq)]
#[regex(r"^(?<year>[0-9]{4})-(?:0[1-9]|1[0-2])-(?<day>[0-9]{2})$")]
struct Date<'a> {
    #[group(0)]
    matched: &'a str,
    day: &'a str,   // bound to (?<day>...) by name
    year: &'a str,  // bound to (?<year>...) by name; field order doesn't matter
}

assert_eq!(
    Date::exec("2024-03-15"),
    Some(Date { matched: "2024-03-15", day: "15", year: "2024" }),
);
```

See the [`#[regex]`](https://docs.rs/ere/latest/ere/macro.regex.html) documentation for full examples.

## Alternatives

`ere` is intended as an alternative to [`regex`](https://crates.io/crates/regex) that provides compile-time checking and regex compilation. However, `ere` is less featureful, so here are a few reasons you might prefer `regex`:

-   You require more complex regular expressions with features like backreferences and word boundary checking (which are unavailable in POSIX EREs).
-   You need run-time-compiled regular expressions (such as when provided by the user).
-   Your regular expression runs significantly more efficiently on a specific regex engine not currently available in `ere`.
