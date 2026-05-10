## 0.2.4

-   Added `DFAU8` engine
-   Renamed `PikeVM` and `U8PikeVM` to `FlatLockstepNFA` and `FlatLockstepNFAU8`
    since they are not really PikeVMs.
-   Various organizational improvements for internals.

## 0.2.3

-   Fixed import issues in macros, where `ere_core` was referenced rather than `ere`.
-   Added functional-style codegen implementation for u8 one-pass engine
    (for improved performance in certain cases).
-   Added fixed-offset implementation for `exec`, where if capture groups always appear at the same offsets,
    we can just run `test` and then index those offsets if it matches.
-   Reduced redirection overhead for calls to `exec` and `test` (by unifying the interface across engines).
-   Added run-based optimizations to avoid short-circuiting on non-branching NFA paths
    for the functional-style codegen in the u8 one-pass engine
    (to reduce branch prediction misses and enable vectorization or in the future possibly SIMD).
-   Expanded testing.
-   Added initial benchmarking.

## 0.2.2

-   Added u8 one-pass engine.
-   Improved performance via suffix optimizations (merging states with the same outgoing transitions).
-   Fixed a bug where the first branch for priority-shortest 'upto' node NFA generation had priority inverted.
-   Improved performance by actually implementing the greedy-mode thread culling for PikeVM
-   Added this changelog.

## 0.2.1

-   Added unstable struct attribute-based method for defining regexes.

## 0.2.0

-   Added PikeVM engine (including `exec` functionality).
-   Added U8PikeVM engine (including `exec` functionality).

## 0.1.0

-   Added initial parsing.
-   Added initial `test` functionality on basic engine.
