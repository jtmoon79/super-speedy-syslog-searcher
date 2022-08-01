// src/printer_debug/stack_tests.rs
//
// tests for `stack.rs`
//

use super::stack::{
    so,
    sn,
    snx,
    sx,
    stack_offset,
    function_name,
    function_name_full,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// XXX: `test_stack_offset` requires human visual inspection
#[test]
fn test_stack_offset() {
    eprintln!("{}test_stack_offset", sn());
    eprintln!("{}stack_offset {}", so(), stack_offset());
    eprintln!("{}stack_offset() in test_stack_offset {}", so(), stack_offset());
    fn test1a() {
        eprintln!("{}stack_offset() in test_stack_offset in test1a {}", so(), stack_offset());
    }
    test1a();
    fn test1b() {
        eprintln!("{}stack_offset() in test_stack_offset in test1b {}", so(), stack_offset());
        fn test2a() {
            eprintln!("{}stack_offset() in test_stack_offset in test1b in test2a {}", so(), stack_offset());
        }
        test2a();
        fn test2b(_a: u128, _b: u128, _c: u128) {
            eprintln!("{}stack_offset() in test_stack_offset in test1b in test2b {}", so(), stack_offset());
        }
        test2b(1, 2, 3);
        fn test2c() {
            eprintln!("{}stack_offset() in test_stack_offset in test1b in test2c {}", so(), stack_offset());
        }
        test2c();
        test2b(1, 2, 3);
    }
    test1b();
    eprintln!("{}test_stack_offset", sx());
}

/// quickie test for debug helpers `sn`, `so`, `snx`, `sx`
#[test]
pub fn test_sn_so_sx() {
    fn depth1() {
        eprintln!("{}depth1 enter", sn());
        fn depth2() {
            eprintln!("{}depth2 enter", sn());
            fn depth3() {
                eprintln!("{}depth3 enter", sn());
                fn depth4() {
                    eprintln!("{}depth4 enter", sn());
                    fn depth5() {
                        eprintln!("{}depth5 enter exit", snx());
                    }
                    eprintln!("{}depth4 middle", so());
                    depth5();
                    eprintln!("{}depth4 exit", sx());
                }
                eprintln!("{}depth3 middle before", so());
                depth4();
                eprintln!("{}depth3 middle after", so());
                eprintln!("{}depth3 exit", sx());
            }
            eprintln!("{}depth2 middle before", so());
            depth3();
            eprintln!("{}depth2 middle after", so());
            eprintln!("{}depth2 exit", sx());
        }
        eprintln!("{}depth1 middle before", so());
        depth2();
        eprintln!("{}depth1 middle after", so());
        eprintln!("{}depth1 exit", sx());
    }
    depth1();
}

#[test]
fn test_function_name_full() {
    let expect_: &str = "s4lib::printer_debug::stack_tests::test_function_name_full";
    let actual: &str = function_name_full!();
    assert_eq!(actual, expect_, "macro function_name returned {:?}, expected {:?}", actual, expect_);
}

#[test]
fn test_function_name() {
    let expect_: &str = "test_function_name";
    let actual: &str = function_name!();
    assert_eq!(actual, expect_, "macro function_name returned {:?}, expected {:?}", actual, expect_);
}
