#![cfg(feature = "unstable-attr-regex")]

use ere::regex;

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
