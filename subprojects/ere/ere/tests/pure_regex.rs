use std::hash::Hasher;

use ere::{compile_regex_dfa_u8, prelude::*};
use ere_macros::{
    compile_regex_fixed_offset, compile_regex_flat_lockstep_nfa,
    compile_regex_flat_lockstep_nfa_u8, compile_regex_u8onepass,
};

/// Checks both [`Regex::exec`] and [`Regex::test`]
macro_rules! assert_match {
    ($re:ident, $text:expr, $some:expr$(,)?) => {
        assert_eq!(
            ::ere::prelude::Regex::exec(&$re, &$text),
            ::core::option::Option::Some($some)
        );
        assert!(::ere::prelude::Regex::test(&$re, &$text));
    };
}
/// Checks both [`Regex::exec`] and [`Regex::test`]
macro_rules! assert_nomatch {
    ($re:ident, $text:expr) => {
        assert_eq!(
            ::ere::prelude::Regex::exec(&$re, &$text),
            ::core::option::Option::None,
        );
        assert!(!::ere::prelude::Regex::test(&$re, &$text));
    };
    ($re:ident, $text:expr, $($rest:expr),+$(,)?) => {
        assert_nomatch!($re, $text);
        assert_nomatch!($re, $($rest),+)
    }
}

#[test]
fn phone_number() {
    const REGEXES: [Regex<2>; 4] = [
        compile_regex_dfa_u8!(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$"),
        compile_regex_flat_lockstep_nfa!(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$"),
        compile_regex_flat_lockstep_nfa_u8!(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$"),
        compile_regex_u8onepass!(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$"),
    ];

    for regex in REGEXES {
        assert_match!(regex, "012-345-6789", [Some("012-345-6789"), None]);
        assert_match!(regex, "987-654-3210", [Some("987-654-3210"), None]);
        assert_match!(
            regex,
            "+1 555-555-5555",
            [Some("+1 555-555-5555"), Some("+1 ")],
        );
        assert_match!(regex, "123-555-9876", [Some("123-555-9876"), None]);

        assert_nomatch!(
            regex,
            "abcd",
            "0123456789",
            "012--345-6789",
            "(555) 555-5555",
            "1 555-555-5555",
        );
    }
}

#[test]
fn byte_value_exec() {
    const REGEXES: [Regex<2>; 3] = [
        compile_regex_dfa_u8!(r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"),
        compile_regex_flat_lockstep_nfa!(r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"),
        compile_regex_flat_lockstep_nfa_u8!(r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"),
        // one pass not working yet, needs further optimizations
    ];
    for regex in REGEXES {
        for i in 0u8..=255u8 {
            let text = i.to_string();

            assert_match!(regex, text, [Some(text.as_str()), Some(text.as_str())]);
        }

        assert_nomatch!(regex, "abcd", "00", "256",);
    }
}

#[test]
fn ipv4_exec() {
    const REGEXES: [Regex<5>; 3] = [
        compile_regex_dfa_u8!(
            r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"
        ),
        compile_regex_flat_lockstep_nfa!(
            r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"
        ),
        compile_regex_flat_lockstep_nfa_u8!(
            r"^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])$"
        ),
        // one pass not working yet, needs further optimizations
    ];

    for regex in REGEXES {
        for i in 0..=10000 {
            // testing deterministic pseudo-random numbers via hashing
            let mut hasher = std::hash::DefaultHasher::new();
            hasher.write_u32(i);
            let i = hasher.finish();
            let [_, _, a, b, c, d, _, _] = i.to_be_bytes();
            let a = a.to_string();
            let b = b.to_string();
            let c = c.to_string();
            let d = d.to_string();
            let text = format!("{a}.{b}.{c}.{d}");

            assert_match!(
                regex,
                text,
                [Some(text.as_str()), Some(&a), Some(&b), Some(&c), Some(&d)],
            );
        }
        assert_nomatch!(regex, "abcd", "1.1.1", "...", "1::", "256.0.0.0",);
    }
}

#[test]
fn needle() {
    const REGEXES: [Regex; 3] = [
        compile_regex_dfa_u8!(r"nee+dle"),
        compile_regex_flat_lockstep_nfa!(r"nee+dle"),
        compile_regex_flat_lockstep_nfa_u8!(r"nee+dle"),
        // not one-pass because it is not start/end anchored
    ];
    for regex in REGEXES {
        assert_match!(regex, "needle", [Some("needle")]);
        assert_match!(regex, "haystackhaysneedletackhaystack", [Some("needle")]);
        assert_match!(
            regex,
            "haystackneeeeeeeeedlehaystack",
            [Some("neeeeeeeeedle")]
        );
        assert_match!(regex, "needneedlele", [Some("needle")]);

        assert_nomatch!(
            regex,
            "haystackhaystack",
            "0123456789",
            "nothinghere",
            "npuowahpeoifjap098uq09p3ior",
            "nedle",
        );
    }
}

#[test]
fn dot() {
    const REGEXES: [Regex; 4] = [
        compile_regex_dfa_u8!("^.$"),
        compile_regex_flat_lockstep_nfa!("^.$"),
        compile_regex_flat_lockstep_nfa_u8!("^.$"),
        compile_regex_u8onepass!("^.$"),
    ];
    for regex in REGEXES {
        for c in '\u{0001}'..=char::MAX {
            let text = c.to_string();

            assert_match!(regex, text, [Some(text.as_str())]);
        }

        assert_nomatch!(regex, "\0", "12", "å©", "");
    }
}

#[test]
fn duplicate_paths() {
    const REGEXES: [Regex<3>; 5] = [
        compile_regex_dfa_u8!("^(ab|bc|ab|bc)(xy|yz|yz|xy)$"),
        compile_regex_flat_lockstep_nfa!("^(ab|bc|ab|bc)(xy|yz|yz|xy)$"),
        compile_regex_flat_lockstep_nfa_u8!("^(ab|bc|ab|bc)(xy|yz|yz|xy)$"),
        // one-pass because it can be simplified to one-pass
        // since its branching paths are actually the same and get merged
        compile_regex_u8onepass!("^(ab|bc|ab|bc)(xy|yz|yz|xy)$"),
        compile_regex_fixed_offset!("^(ab|bc|ab|bc)(xy|yz|yz|xy)$"),
    ];
    for regex in &REGEXES {
        assert_match!(regex, "abxy", [Some("abxy"), Some("ab"), Some("xy")]);
        assert_match!(regex, "abyz", [Some("abyz"), Some("ab"), Some("yz")]);
        assert_match!(regex, "bcxy", [Some("bcxy"), Some("bc"), Some("xy")]);
        assert_match!(regex, "bcyz", [Some("bcyz"), Some("bc"), Some("yz")]);

        assert_nomatch!(regex, "acxy", "abxz", "bc", "yz");
    }
}

#[test]
fn hex_color() {
    const REGEXES: [Regex<4>; 5] = [
        compile_regex_dfa_u8!("^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$"),
        compile_regex_flat_lockstep_nfa!("^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$"),
        compile_regex_flat_lockstep_nfa_u8!("^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$"),
        compile_regex_u8onepass!("^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$"),
        compile_regex_fixed_offset!("^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$"),
    ];
    for regex in &REGEXES {
        assert_match!(
            regex,
            "#abcdef",
            [Some("#abcdef"), Some("ab"), Some("cd"), Some("ef")]
        );
        assert_match!(
            regex,
            "#FfFfFf",
            [Some("#FfFfFf"), Some("Ff"), Some("Ff"), Some("Ff")]
        );
        assert_match!(
            regex,
            "#000000",
            [Some("#000000"), Some("00"), Some("00"), Some("00")]
        );

        assert_nomatch!(regex, "#qaaaaa", "#12345", "#1234567");
    }
}

#[test]
fn iso8601_date_extended() {
    const REGEXES: [Regex<4>; 5] = [
        compile_regex_dfa_u8!("^([0-9]{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"),
        compile_regex_flat_lockstep_nfa!("^([0-9]{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"),
        compile_regex_flat_lockstep_nfa_u8!(
            "^([0-9]{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"
        ),
        compile_regex_u8onepass!("^([0-9]{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"),
        compile_regex_fixed_offset!("^([0-9]{4})-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$"),
    ];
    for regex in &REGEXES {
        assert_match!(
            regex,
            "2000-01-01",
            [Some("2000-01-01"), Some("2000"), Some("01"), Some("01")]
        );

        assert_nomatch!(regex, "a", "1999-1-1", "2000-00-00");
    }
}

#[test]
fn iso8601_time_extended() {
    // excluding brevity rules bc that would be really complex
    const REGEXES: [Regex<5>; 4] = [
        compile_regex_dfa_u8!(r"^T?([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]([.,][0-9]+)?)$"),
        compile_regex_flat_lockstep_nfa!(
            r"^T?([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]([.,][0-9]+)?)$"
        ),
        compile_regex_flat_lockstep_nfa_u8!(
            r"^T?([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]([.,][0-9]+)?)$"
        ),
        compile_regex_u8onepass!(r"^T?([01][0-9]|2[0-3]):([0-5][0-9]):([0-5][0-9]([.,][0-9]+)?)$"),
    ];
    for regex in &REGEXES {
        assert_match!(
            regex,
            "T00:00:00",
            [Some("T00:00:00"), Some("00"), Some("00"), Some("00"), None]
        );
        assert_match!(
            regex,
            "23:59:59.9999",
            [
                Some("23:59:59.9999"),
                Some("23"),
                Some("59"),
                Some("59.9999"),
                Some(".9999")
            ]
        );

        assert_nomatch!(regex, "24:00:00", "212345", "00:60:00", "00:00:60");
    }
}

#[test]
fn line_abc() {
    const REGEXES: [Regex<3>; 3] = [
        compile_regex_dfa_u8!("(^|\n)abc(\n|$)"),
        compile_regex_flat_lockstep_nfa!("(^|\n)abc(\n|$)"),
        compile_regex_flat_lockstep_nfa_u8!("(^|\n)abc(\n|$)"),
    ];
    for regex in &REGEXES {
        assert_match!(regex, "abc", [Some("abc"), Some(""), Some("")]);
        assert_match!(regex, "\nabc", [Some("\nabc"), Some("\n"), Some("")]);
        assert_match!(regex, "abc\n", [Some("abc\n"), Some(""), Some("\n")]);
        assert_match!(
            regex,
            "\n\nabc\n",
            [Some("\nabc\n"), Some("\n"), Some("\n")],
        );
        assert_nomatch!(regex, "ac", "\n\n", "\nab", "bc\n");
    }
}

#[test]
fn us_state_abbreviations() {
    const REGEXES: [Regex; 3] = [
        compile_regex_dfa_u8!("AL|AK|AZ|AR|CA|CO|CT|DE|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY"),
        compile_regex_flat_lockstep_nfa!("AL|AK|AZ|AR|CA|CO|CT|DE|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY"),
        compile_regex_flat_lockstep_nfa_u8!("AL|AK|AZ|AR|CA|CO|CT|DE|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY"),
    ];
    const STATES_STR: &str = "AL|AK|AZ|AR|CA|CO|CT|DE|FL|GA|HI|ID|IL|IN|IA|KS|KY|LA|ME|MD|MA|MI|MN|MS|MO|MT|NE|NV|NH|NJ|NM|NY|NC|ND|OH|OK|OR|PA|RI|SC|SD|TN|TX|UT|VT|VA|WA|WV|WI|WY";
    for regex in &REGEXES {
        for first in 'A'..='Z' {
            for second in 'A'..='Z' {
                let combined = format!("{first}{second}");
                if STATES_STR.contains(&combined) {
                    assert_match!(regex, combined, [Some(combined.as_str())]);
                } else {
                    assert_nomatch!(regex, combined);
                }
            }
        }

        assert_match!(regex, "ALABAMA", [Some("AL")]);
        assert_match!(regex, "REAL", [Some("AL")]);
        assert_match!(regex, "TACO", [Some("CO")]);
        assert_match!(regex, "TXT", [Some("TX")]);
        assert_match!(regex, "MEME", [Some("ME")]);
        assert_match!(regex, "ORCA", [Some("OR")]);
        assert_match!(regex, "WYVERN", [Some("WY")]);
        assert_match!(regex, "WHINE", [Some("HI")]);

        assert_nomatch!(regex, "Alabama", "ct");
    }
}

#[test]
fn find_quoted() {
    const REGEXES: [Regex<3>; 3] = [
        compile_regex_dfa_u8!(r#""((\\"|[^"])*)""#),
        compile_regex_flat_lockstep_nfa!(r#""((\\"|[^"])*)""#),
        compile_regex_flat_lockstep_nfa_u8!(r#""((\\"|[^"])*)""#),
    ];
    for regex in &REGEXES {
        assert_match!(regex, r#""""#, [Some(r#""""#), Some(""), None]);
        assert_match!(
            regex,
            r#""a\"""#,
            [Some(r#""a\"""#), Some(r#"a\""#), Some(r#"\""#)]
        );
        assert_match!(
            regex,
            r#""a\"c""#,
            [Some(r#""a\"c""#), Some(r#"a\"c"#), Some(r#"c"#)]
        );
        assert_match!(
            regex,
            r#"before"a\"c""#,
            [Some(r#""a\"c""#), Some(r#"a\"c"#), Some(r#"c"#)]
        );
        assert_match!(
            regex,
            r#""a\" e"after"#,
            [Some(r#""a\" e""#), Some(r#"a\" e"#), Some(r#"e"#)]
        );

        assert_nomatch!(
            regex,
            "unquoted",
            r#""unended"#,
            r#"unstarted""#,
            "'single quotes'"
        );
    }
}

#[test]
fn find_discord_emoji() {
    const REGEXES: [Regex; 3] = [
        compile_regex_dfa_u8!(":[[:alnum:]_]{2,}:"),
        compile_regex_flat_lockstep_nfa!(":[[:alnum:]_]{2,}:"),
        compile_regex_flat_lockstep_nfa_u8!(":[[:alnum:]_]{2,}:"),
    ];
    for regex in &REGEXES {
        assert_match!(regex, ":crab:", [Some(":crab:")]);
        assert_match!(regex, "this crate is fire :fire:", [Some(":fire:")]);
        assert_match!(
            regex,
            ":regional_indicator_e::regional_indicator_r::regional_indicator_e:",
            [Some(":regional_indicator_e:")],
        );
        assert_match!(regex, "writing :pencil2:", [Some(":pencil2:")],);

        assert_nomatch!(
            regex,
            "without emojis",
            "unended :emoji",
            "emoji is unstarted:",
            "turbofish::<>",
            "single char :a:",
            ":with-dash:",
            "this is a noun: apple\nthis is a verb: jump"
        );
    }
}

#[test]
fn html_comment() {
    const REGEXES: [Regex; 3] = [
        compile_regex_dfa_u8!("<!--.*?-->"),
        compile_regex_flat_lockstep_nfa!("<!--.*?-->"),
        compile_regex_flat_lockstep_nfa_u8!("<!--.*?-->"),
    ];
    for regex in &REGEXES {
        assert_match!(regex, "<!---->", [Some("<!---->")]);
        assert_match!(regex, "<!-- comment -->", [Some("<!-- comment -->")]);
        assert_match!(
            regex,
            "<div><!-- comment --></div>",
            [Some("<!-- comment -->")]
        );
        assert_match!(
            regex,
            "<div><!-- comment -->--></div>",
            [Some("<!-- comment -->")]
        );
        assert_match!(
            regex,
            "<div><!--<!-- comment -->--></div>",
            [Some("<!--<!-- comment -->")]
        );
        assert_match!(
            regex,
            "<!--<!-- comment -->-->",
            [Some("<!--<!-- comment -->")]
        );
    }
}

#[test]
fn html_link_extract() {
    // a very gross regex, but isn't really a task I'd use a regex for anyway.
    // multiline is just for formatting, is ignored using `\`
    const REGEXES: [Regex<8>; 2] = [
        // compile_regex_dfa_u8!(
        //     "<a\
        // ([[:space:]]*[^[:space:][:cntrl:]\"'>/=]+\
        //     ([[:space:]]*=\
        //     [[:space:]]*([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*'))?\
        // )*?\
        // [[:space:]]*href[[:space:]]*=[[:space:]]*\
        //     ([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*')\
        // ([[:space:]]*[^[:space:][:cntrl:]\"'>/=]+\
        //     ([[:space:]]*=\
        //     [[:space:]]*([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*'))?\
        // )*?\
        // [[:space:]]*>\
        // [^<]*\
        // </a[[:space:]]*>"
        // ),
        compile_regex_flat_lockstep_nfa!(
            "<a\
        ([[:space:]]*[^[:space:][:cntrl:]\"'>/=]+\
            ([[:space:]]*=\
            [[:space:]]*([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*'))?\
        )*?\
        [[:space:]]*href[[:space:]]*=[[:space:]]*\
            ([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*')\
        ([[:space:]]*[^[:space:][:cntrl:]\"'>/=]+\
            ([[:space:]]*=\
            [[:space:]]*([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*'))?\
        )*?\
        [[:space:]]*>\
        [^<]*\
        </a[[:space:]]*>"
        ),
        compile_regex_flat_lockstep_nfa_u8!(
            "<a\
        ([[:space:]]*[^[:space:][:cntrl:]\"'>/=]+\
            ([[:space:]]*=\
            [[:space:]]*([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*'))?\
        )*?\
        [[:space:]]*href[[:space:]]*=[[:space:]]*\
            ([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*')\
        ([[:space:]]*[^[:space:][:cntrl:]\"'>/=]+\
            ([[:space:]]*=\
            [[:space:]]*([^[:space:]\"'=<>`]+|\"[^\"]*\"|'[^']*'))?\
        )*?\
        [[:space:]]*>\
        [^<]*\
        </a[[:space:]]*>"
        ),
    ];

    for regex in &REGEXES {
        assert_match!(
            regex,
            "<a href='my_url_here'></a>",
            [
                Some("<a href='my_url_here'></a>"),
                None,
                None,
                None,
                Some("'my_url_here'"),
                None,
                None,
                None,
            ]
        );
        assert_match!(
            regex,
            "<div><a class= 'my_class' href='my_url_here'></a></div>",
            [
                Some("<a class= 'my_class' href='my_url_here'></a>"),
                Some(" class= 'my_class'"),
                Some("= 'my_class'"),
                Some("'my_class'"),
                Some("'my_url_here'"),
                None,
                None,
                None,
            ]
        );
        assert_match!(
            regex,
            "<a
                class= 'my_class'
                href='my_url_here'
                disabled
            >This is the label</a >",
            [
                Some(
                    "<a
                class= 'my_class'
                href='my_url_here'
                disabled
            >This is the label</a >"
                ),
                Some(
                    "
                class= 'my_class'"
                ),
                Some("= 'my_class'"),
                Some("'my_class'"),
                Some("'my_url_here'"),
                Some(
                    "
                disabled"
                ),
                None,
                None,
            ]
        );
    }
}

#[test]
fn http_request() {
    // usually better to use a proper parser, but for testing purposes here:
    // based on https://www.rfc-editor.org/rfc/rfc9112.pdf
    const REGEXES: [Regex<7>; 3] = [
        compile_regex_dfa_u8!(
            "^([A-Z]+) ([^[:space:]]+) (HTTP/[0-9].[0-9])\r\n\
            (([!#$%&'*+-.^_`|~[:alnum:]]+:[ \t]*[^\r\n]*?[ \t]*\r\n)*)\
            \r\n\
            ([\0-\u{10FFFF}]*)$"
        ),
        compile_regex_flat_lockstep_nfa!(
            "^([A-Z]+) ([^[:space:]]+) (HTTP/[0-9].[0-9])\r\n\
            (([!#$%&'*+-.^_`|~[:alnum:]]+:[ \t]*[^\r\n]*?[ \t]*\r\n)*)\
            \r\n\
            ([\0-\u{10FFFF}]*)$"
        ),
        compile_regex_flat_lockstep_nfa_u8!(
            "^([A-Z]+) ([^[:space:]]+) (HTTP/[0-9].[0-9])\r\n\
            (([!#$%&'*+-.^_`|~[:alnum:]]+:[ \t]*[^\r\n]*?[ \t]*\r\n)*)\
            \r\n\
            ([\0-\u{10FFFF}]*)$"
        ),
    ];
    for regex in &REGEXES {
        let text = "GET /index.html HTTP/1.1\r\n\
        header-one: header-text\r\n\
        header-two: header-text-two\r\n\
        \r\n\
        HTTP body here";
        assert_match!(
            regex,
            text,
            [
                Some(text),
                Some("GET"),
                Some("/index.html"),
                Some("HTTP/1.1"),
                Some(
                    "header-one: header-text\r\n\
                    header-two: header-text-two\r\n"
                ),
                Some("header-two: header-text-two\r\n"),
                Some("HTTP body here"),
            ],
        );

        let text = "GET / HTTP/1.1\r\n\r\n";
        assert_match!(
            regex,
            text,
            [
                Some(text),
                Some("GET"),
                Some("/"),
                Some("HTTP/1.1"),
                Some(""),
                None,
                Some(""),
            ],
        );

        assert_nomatch!(
            regex,
            "obviously not http",
            "",
            "GET / HTTP/1.1\n\nwithout carriage return",
            "first GET / HTTP/1.1\r\n\r\npreceded",
            "OPTION /invalid url HTTP/1.1\r\n\r\nbody",
        );
    }
}

#[test]
fn uri_with_authority() {
    const REGEXES: [Regex<5>; 4] = [
        compile_regex_dfa_u8!(r"^([[:alpha:]][[:alnum:]-+.]+)://([^?#]+)([?][^#]*)?(#[^#]*)?$"),
        compile_regex_flat_lockstep_nfa!(
            r"^([[:alpha:]][[:alnum:]-+.]+)://([^?#]+)([?][^#]*)?(#[^#]*)?$"
        ),
        compile_regex_flat_lockstep_nfa_u8!(
            r"^([[:alpha:]][[:alnum:]-+.]+)://([^?#]+)([?][^#]*)?(#[^#]*)?$"
        ),
        compile_regex_u8onepass!(r"^([[:alpha:]][[:alnum:]-+.]+)://([^?#]+)([?][^#]*)?(#[^#]*)?$"),
    ];
    for regex in &REGEXES {
        assert_match!(
            regex,
            "https://example.com",
            [
                Some("https://example.com"),
                Some("https"),
                Some("example.com"),
                None,
                None
            ]
        );
        assert_match!(
            regex,
            "https://subdomain.example.com/category/page2",
            [
                Some("https://subdomain.example.com/category/page2"),
                Some("https"),
                Some("subdomain.example.com/category/page2"),
                None,
                None
            ]
        );
        assert_match!(
            regex,
            "https://subdomain.example.com/category/page2?query=1234",
            [
                Some("https://subdomain.example.com/category/page2?query=1234"),
                Some("https"),
                Some("subdomain.example.com/category/page2"),
                Some("?query=1234"),
                None
            ]
        );
        assert_match!(
            regex,
            "https://subdomain.example.com/category/page2?query=1234#fragment",
            [
                Some("https://subdomain.example.com/category/page2?query=1234#fragment"),
                Some("https"),
                Some("subdomain.example.com/category/page2"),
                Some("?query=1234"),
                Some("#fragment")
            ]
        );
    }
}

/// Anchor interaction with greedy priority ambiguous submatching
///
/// Where priority is set via alternation
#[test]
fn greedy_anchor_priority_alternation() {
    const REGEXES: [Regex<5>; 3] = [
        compile_regex_dfa_u8!(r"(^a$)|(^a)|(a$)|(a)"),
        compile_regex_flat_lockstep_nfa!(r"(^a$)|(^a)|(a$)|(a)"),
        compile_regex_flat_lockstep_nfa_u8!(r"(^a$)|(^a)|(a$)|(a)"),
    ];
    for regex in REGEXES {
        assert_match!(regex, "a", [Some("a"), Some("a"), None, None, None]);
        assert_match!(regex, "ax", [Some("a"), None, Some("a"), None, None]);
        assert_match!(regex, "xa", [Some("a"), None, None, Some("a"), None]);
        assert_match!(regex, "xax", [Some("a"), None, None, None, Some("a")]);
    }
}

#[test]
fn greedy() {
    const REGEX1: Regex<2> = compile_regex!(r"^(a|ab)b?cd$");
    assert_match!(REGEX1, "abcd", [Some("abcd"), Some("a")]);

    const REGEX2: Regex<2> = compile_regex!(r"^(ab|a)b?cd$");
    assert_match!(REGEX2, "abcd", [Some("abcd"), Some("ab")]);

    const REGEX3: Regex<3> = compile_regex!(r"^(a*)(a*)$");
    assert_match!(
        REGEX3,
        "aaaaaaaa",
        [Some("aaaaaaaa"), Some("aaaaaaaa"), Some("")],
    );

    const REGEX4: Regex<1> = compile_regex!(r"a*");
    assert_match!(REGEX4, "aaaaaaaa", [Some("aaaaaaaa")]);
    assert_match!(REGEX4, "aabaaaaaa", [Some("aa")]);

    // with shortest
    const REGEX5: Regex<1> = compile_regex!(r"a*?");
    assert_match!(REGEX5, "aaaaaaaa", [Some("")]);

    const REGEX6: Regex<1> = compile_regex!(r"a*?|a*");
    assert_match!(REGEX6, "aaaaaaaa", [Some("")]);
    assert_match!(REGEX6, "aaaabaaaa", [Some("")]);

    // Matches the second alternation because it is greedy, so it matches the `a` first
    // and does not wait to find the `b`. This is consistent with other engines.
    const REGEX7: Regex<1> = compile_regex!(r"ba*?b|a*");
    assert_match!(REGEX7, "abaaabaaaa", [Some("a")]);

    const REGEX8: Regex<1> = compile_regex!(r"a??");
    assert_match!(REGEX8, "abaaabaaaa", [Some("")]);

    const REGEX9: Regex<3> = compile_regex!("()^a|()c");
    assert_match!(REGEX9, "a", [Some("a"), Some(""), None]);
    assert_match!(REGEX9, "c", [Some("c"), None, Some("")]);
    assert_match!(REGEX9, "bc", [Some("c"), None, Some("")]);
}
