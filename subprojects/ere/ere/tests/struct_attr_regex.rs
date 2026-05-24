#![cfg(feature = "unstable-attr-regex")]

extern crate super_speedy_syslog_searcher_ere as ere;
use ere::regex;

pub mod ere_tests {
    use super::*;

#[test]
fn non_capturing_group() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^(?:foo)(bar)$")]
    struct FooBar<'a>(&'a str, &'a str);

    assert!(FooBar::test("foobar"));
    assert!(!FooBar::test("bar"));
    assert_eq!(FooBar::exec("foobar"), Some(FooBar("foobar", "bar")));
    assert_eq!(FooBar::exec("bar"), None);

    assert!(FooBar::test_bytes(b"foobar"));
    assert_eq!(FooBar::exec_bytes(b"foobar"), Some(FooBar("foobar", "bar")));
}

#[test]
fn named_capture_group_tuple_struct() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^Homer (?<middle>.)\. Simpson$")]
    struct HomerSimpson<'a>(&'a str, &'a str);

    assert!(HomerSimpson::test("Homer J. Simpson"));
    assert!(!HomerSimpson::test("Homer Simpson"));
    assert_eq!(
        HomerSimpson::exec("Homer J. Simpson"),
        Some(HomerSimpson("Homer J. Simpson", "J"))
    );
    assert_eq!(HomerSimpson::exec("Homer Simpson"), None);
}

#[test]
fn named_field_struct() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^Homer (?<middle>.)\. Simpson$")]
    struct HomerSimpson<'a> {
        #[group(0)]
        matched: &'a str,
        middle: &'a str,
    }

    assert!(HomerSimpson::test("Homer J. Simpson"));
    assert!(!HomerSimpson::test("Homer Simpson"));
    assert_eq!(
        HomerSimpson::exec("Homer J. Simpson"),
        Some(HomerSimpson { matched: "Homer J. Simpson", middle: "J" })
    );
    assert_eq!(HomerSimpson::exec("Homer Simpson"), None);
}

#[test]
fn unnamed_groups_not_required_in_named_struct() {
    // Unnamed capture groups in the regex don't need corresponding fields
    // in a named struct. Only named capture groups must be bound.
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^(?<year>[21][0-9]{3})(-|-=)(?<month>0[1-9]|1[0-2])(-|-=)(?<day>[0123][0-9])$")]
    struct DateMatch<'a> {
        #[group(0)]
        matched: &'a str,
        year: &'a str,
        month: &'a str,
        day: &'a str,
    }

    assert_eq!(
        DateMatch::exec("2024-03-29"),
        Some(DateMatch { matched: "2024-03-29", year: "2024", month: "03", day: "29" })
    );
    assert_eq!(
        DateMatch::exec("2024-=03-=29"),
        Some(DateMatch { matched: "2024-=03-=29", year: "2024", month: "03", day: "29" })
    );
    assert_eq!(DateMatch::exec("2024/03/29"), None);
}

#[test]
fn phone_number_struct() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^(\+1 )?[0-9]{3}-[0-9]{3}-[0-9]{4}$")]
    struct PhoneMatcher<'a>(&'a str, Option<&'a str>);

    assert!(PhoneMatcher::test("012-345-6789"));
    assert!(PhoneMatcher::test("987-654-3210"));
    assert!(PhoneMatcher::test("+1 555-555-5555"));
    assert!(PhoneMatcher::test("123-555-9876"));

    assert!(!PhoneMatcher::test("abcd"));
    assert!(!PhoneMatcher::test("0123456789"));
    assert!(!PhoneMatcher::test("012--345-6789"));
    assert!(!PhoneMatcher::test("(555) 555-5555"));
    assert!(!PhoneMatcher::test("1 555-555-5555"));

    assert_eq!(
        PhoneMatcher::exec("012-345-6789"),
        Some(PhoneMatcher("012-345-6789", None))
    );
    assert_eq!(
        PhoneMatcher::exec("987-654-3210"),
        Some(PhoneMatcher("987-654-3210", None))
    );
    assert_eq!(
        PhoneMatcher::exec("+1 555-555-5555"),
        Some(PhoneMatcher("+1 555-555-5555", Some("+1 ")))
    );
    assert_eq!(
        PhoneMatcher::exec("123-555-9876"),
        Some(PhoneMatcher("123-555-9876", None))
    );

    assert_eq!(PhoneMatcher::exec("abcd"), None);
    assert_eq!(PhoneMatcher::exec("0123456789"), None);
    assert_eq!(PhoneMatcher::exec("012--345-6789"), None);
    assert_eq!(PhoneMatcher::exec("(555) 555-5555"), None);
    assert_eq!(PhoneMatcher::exec("1 555-555-5555"), None);
}

// --- engine parameter tests (engines not covered by doctests) ---

#[test]
fn engine_auto() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$", engine = Auto)]
    struct Hex<'a>(&'a str, &'a str, &'a str, &'a str);

    assert_eq!(Hex::exec("#ff0080"), Some(Hex("#ff0080", "ff", "00", "80")));
    assert_eq!(Hex::exec("not-hex"), None);
}

#[test]
fn engine_flat_lockstep_nfa_u8() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$", engine = FlatLockstepNfaU8)]
    struct Hex<'a>(&'a str, &'a str, &'a str, &'a str);

    assert_eq!(Hex::exec("#ff0080"), Some(Hex("#ff0080", "ff", "00", "80")));
    assert_eq!(Hex::exec("not-hex"), None);

    assert!(Hex::test_bytes(b"#ff0080"));
    assert_eq!(Hex::exec_bytes(b"#ff0080"), Some(Hex("#ff0080", "ff", "00", "80")));
    assert!(!Hex::test_bytes(b"#ff\xFF80"));
}

#[test]
fn engine_flat_lockstep_nfa() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$", engine = FlatLockstepNfa)]
    struct Hex<'a>(&'a str, &'a str, &'a str, &'a str);

    assert_eq!(Hex::exec("#ff0080"), Some(Hex("#ff0080", "ff", "00", "80")));
    assert_eq!(Hex::exec("not-hex"), None);
}

#[test]
fn engine_fixed_offset() {
    #[derive(PartialEq, Eq, Debug)]
    #[regex(r"^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$", engine = FixedOffset)]
    struct Hex<'a>(&'a str, &'a str, &'a str, &'a str);

    assert_eq!(Hex::exec("#ff0080"), Some(Hex("#ff0080", "ff", "00", "80")));
    assert_eq!(Hex::exec("not-hex"), None);
}

} // mod ere_tests
