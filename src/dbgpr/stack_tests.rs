// dbgpr/stack_tests.rs
//
// tests for `stack.rs`
//

use super::stack::{so, sn, snx, sx, stack_offset};

extern crate debug_print;
use debug_print::debug_eprintln;


/// XXX: `test_stack_offset` requires human visual inspection
#[test]
fn test_stack_offset() {
    debug_eprintln!("{}test_stack_offset", sn());
    debug_eprintln!("{}stack_offset {}", so(), stack_offset());
    debug_eprintln!("{}stack_offset() in test_stack_offset {}", so(), stack_offset());
    fn test1a() {
        debug_eprintln!("{}stack_offset() in test_stack_offset in test1a {}", so(), stack_offset());
    }
    test1a();
    fn test1b() {
        debug_eprintln!("{}stack_offset() in test_stack_offset in test1b {}", so(), stack_offset());
        fn test2a() {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2a {}", so(), stack_offset());
        }
        test2a();
        fn test2b(_a: u128, _b: u128, _c: u128) {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2b {}", so(), stack_offset());
        }
        test2b(1, 2, 3);
        fn test2c() {
            debug_eprintln!("{}stack_offset() in test_stack_offset in test1b in test2c {}", so(), stack_offset());
        }
        test2c();
        test2b(1, 2, 3);
    }
    test1b();
    debug_eprintln!("{}test_stack_offset", sx());
}

/// quickie test for debug helpers `sn`, `so`, `snx`, `sx`
#[test]
pub fn test_sn_so_sx() {
    fn depth1() {
        debug_eprintln!("{}depth1 enter", sn());
        fn depth2() {
            debug_eprintln!("{}depth2 enter", sn());
            fn depth3() {
                debug_eprintln!("{}depth3 enter", sn());
                fn depth4() {
                    debug_eprintln!("{}depth4 enter", sn());
                    fn depth5() {
                        debug_eprintln!("{}depth5 enter exit", snx());
                    }
                    debug_eprintln!("{}depth4 middle", so());
                    depth5();
                    debug_eprintln!("{}depth4 exit", sx());
                }
                debug_eprintln!("{}depth3 middle before", so());
                depth4();
                debug_eprintln!("{}depth3 middle after", so());
                debug_eprintln!("{}depth3 exit", sx());
            }
            debug_eprintln!("{}depth2 middle before", so());
            depth3();
            debug_eprintln!("{}depth2 middle after", so());
            debug_eprintln!("{}depth2 exit", sx());
        }
        debug_eprintln!("{}depth1 middle before", so());
        depth2();
        debug_eprintln!("{}depth1 middle after", so());
        debug_eprintln!("{}depth1 exit", sx());
    }
    depth1();
}
