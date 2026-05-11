#![cfg(feature = "unstable-attr-regex")]

#[test]
fn unbound_named_field() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/unbound_named_field.rs");
}

#[test]
fn missing_named_capture_field() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/missing_named_capture_field.rs");
}

#[test]
fn group_index_out_of_bounds() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/group_index_out_of_bounds.rs");
}

#[test]
fn bind_strict_missing_unnamed() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/bind_strict_missing_unnamed.rs");
}

#[test]
fn bind_named_missing_named() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/bind_named_missing_named.rs");
}

#[test]
fn bind_named_unbound_field() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/bind_named_unbound_field.rs");
}

#[test]
fn engine_one_pass_not_applicable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/engine_one_pass_not_applicable.rs");
}
