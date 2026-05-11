// slice_contains.rs

//! Search a slice quickly (loop unroll version).
//!
//! Loop unrolled implementation of `slice.contains` for a byte slice and a
//! hardcoded array. Uses [`unroll_for_loops`].
//! Benchmark `benches/bench_slice_contains.rs` demonstrates this is faster
//! than `slice.contains`.
//!
//! [`unroll_for_loops`]: https://docs.rs/unroll/0.1.5/unroll/

use ::memchr;
#[cfg(feature = "bench_jetscii")]
use ::jetscii;
#[cfg(feature = "bench_stringzilla")]
use ::stringzilla;
use ::unroll::unroll_for_loops;


#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_2_2(
    slice_: &[u8; 2],
    search: &[u8; 2],
) -> bool {
    for i in 0..2 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_3_2(
    slice_: &[u8; 3],
    search: &[u8; 2],
) -> bool {
    for i in 0..3 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_4_2(
    slice_: &[u8; 4],
    search: &[u8; 2],
) -> bool {
    for i in 0..4 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_5_2(
    slice_: &[u8; 5],
    search: &[u8; 2],
) -> bool {
    for i in 0..5 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_6_2(
    slice_: &[u8; 6],
    search: &[u8; 2],
) -> bool {
    for i in 0..6 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_7_2(
    slice_: &[u8; 7],
    search: &[u8; 2],
) -> bool {
    for i in 0..7 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_8_2(
    slice_: &[u8; 8],
    search: &[u8; 2],
) -> bool {
    for i in 0..8 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_9_2(
    slice_: &[u8; 9],
    search: &[u8; 2],
) -> bool {
    for i in 0..9 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_10_2(
    slice_: &[u8; 10],
    search: &[u8; 2],
) -> bool {
    for i in 0..10 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_11_2(
    slice_: &[u8; 11],
    search: &[u8; 2],
) -> bool {
    for i in 0..11 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_12_2(
    slice_: &[u8; 12],
    search: &[u8; 2],
) -> bool {
    for i in 0..12 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_13_2(
    slice_: &[u8; 13],
    search: &[u8; 2],
) -> bool {
    for i in 0..13 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_14_2(
    slice_: &[u8; 14],
    search: &[u8; 2],
) -> bool {
    for i in 0..14 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_15_2(
    slice_: &[u8; 15],
    search: &[u8; 2],
) -> bool {
    for i in 0..15 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_16_2(
    slice_: &[u8; 16],
    search: &[u8; 2],
) -> bool {
    for i in 0..16 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_17_2(
    slice_: &[u8; 17],
    search: &[u8; 2],
) -> bool {
    for i in 0..17 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_18_2(
    slice_: &[u8; 18],
    search: &[u8; 2],
) -> bool {
    for i in 0..18 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_19_2(
    slice_: &[u8; 19],
    search: &[u8; 2],
) -> bool {
    for i in 0..19 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_20_2(
    slice_: &[u8; 20],
    search: &[u8; 2],
) -> bool {
    for i in 0..20 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_21_2(
    slice_: &[u8; 21],
    search: &[u8; 2],
) -> bool {
    for i in 0..21 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_22_2(
    slice_: &[u8; 22],
    search: &[u8; 2],
) -> bool {
    for i in 0..22 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_23_2(
    slice_: &[u8; 23],
    search: &[u8; 2],
) -> bool {
    for i in 0..23 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_24_2(
    slice_: &[u8; 24],
    search: &[u8; 2],
) -> bool {
    for i in 0..24 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_25_2(
    slice_: &[u8; 25],
    search: &[u8; 2],
) -> bool {
    for i in 0..25 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_26_2(
    slice_: &[u8; 26],
    search: &[u8; 2],
) -> bool {
    for i in 0..26 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_27_2(
    slice_: &[u8; 27],
    search: &[u8; 2],
) -> bool {
    for i in 0..27 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_28_2(
    slice_: &[u8; 28],
    search: &[u8; 2],
) -> bool {
    for i in 0..28 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_29_2(
    slice_: &[u8; 29],
    search: &[u8; 2],
) -> bool {
    for i in 0..29 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_30_2(
    slice_: &[u8; 30],
    search: &[u8; 2],
) -> bool {
    for i in 0..30 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_31_2(
    slice_: &[u8; 31],
    search: &[u8; 2],
) -> bool {
    for i in 0..31 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_32_2(
    slice_: &[u8; 32],
    search: &[u8; 2],
) -> bool {
    for i in 0..32 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_33_2(
    slice_: &[u8; 33],
    search: &[u8; 2],
) -> bool {
    for i in 0..33 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_34_2(
    slice_: &[u8; 34],
    search: &[u8; 2],
) -> bool {
    for i in 0..34 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_35_2(
    slice_: &[u8; 35],
    search: &[u8; 2],
) -> bool {
    for i in 0..35 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_36_2(
    slice_: &[u8; 36],
    search: &[u8; 2],
) -> bool {
    for i in 0..36 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_37_2(
    slice_: &[u8; 37],
    search: &[u8; 2],
) -> bool {
    for i in 0..37 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_38_2(
    slice_: &[u8; 38],
    search: &[u8; 2],
) -> bool {
    for i in 0..38 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_39_2(
    slice_: &[u8; 39],
    search: &[u8; 2],
) -> bool {
    for i in 0..39 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_40_2(
    slice_: &[u8; 40],
    search: &[u8; 2],
) -> bool {
    for i in 0..40 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_41_2(
    slice_: &[u8; 41],
    search: &[u8; 2],
) -> bool {
    for i in 0..41 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_42_2(
    slice_: &[u8; 42],
    search: &[u8; 2],
) -> bool {
    for i in 0..42 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_43_2(
    slice_: &[u8; 43],
    search: &[u8; 2],
) -> bool {
    for i in 0..43 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_44_2(
    slice_: &[u8; 44],
    search: &[u8; 2],
) -> bool {
    for i in 0..44 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_45_2(
    slice_: &[u8; 45],
    search: &[u8; 2],
) -> bool {
    for i in 0..45 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_46_2(
    slice_: &[u8; 46],
    search: &[u8; 2],
) -> bool {
    for i in 0..46 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_47_2(
    slice_: &[u8; 47],
    search: &[u8; 2],
) -> bool {
    for i in 0..47 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_48_2(
    slice_: &[u8; 48],
    search: &[u8; 2],
) -> bool {
    for i in 0..48 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_49_2(
    slice_: &[u8; 49],
    search: &[u8; 2],
) -> bool {
    for i in 0..49 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_50_2(
    slice_: &[u8; 50],
    search: &[u8; 2],
) -> bool {
    for i in 0..50 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_51_2(
    slice_: &[u8; 51],
    search: &[u8; 2],
) -> bool {
    for i in 0..51 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_52_2(
    slice_: &[u8; 52],
    search: &[u8; 2],
) -> bool {
    for i in 0..52 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_53_2(
    slice_: &[u8; 53],
    search: &[u8; 2],
) -> bool {
    for i in 0..53 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_54_2(
    slice_: &[u8; 54],
    search: &[u8; 2],
) -> bool {
    for i in 0..54 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_55_2(
    slice_: &[u8; 55],
    search: &[u8; 2],
) -> bool {
    for i in 0..55 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_56_2(
    slice_: &[u8; 56],
    search: &[u8; 2],
) -> bool {
    for i in 0..56 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_57_2(
    slice_: &[u8; 57],
    search: &[u8; 2],
) -> bool {
    for i in 0..57 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_58_2(
    slice_: &[u8; 58],
    search: &[u8; 2],
) -> bool {
    for i in 0..58 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_59_2(
    slice_: &[u8; 59],
    search: &[u8; 2],
) -> bool {
    for i in 0..59 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_60_2(
    slice_: &[u8; 60],
    search: &[u8; 2],
) -> bool {
    for i in 0..60 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_61_2(
    slice_: &[u8; 61],
    search: &[u8; 2],
) -> bool {
    for i in 0..61 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_62_2(
    slice_: &[u8; 62],
    search: &[u8; 2],
) -> bool {
    for i in 0..62 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_63_2(
    slice_: &[u8; 63],
    search: &[u8; 2],
) -> bool {
    for i in 0..63 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_64_2(
    slice_: &[u8; 64],
    search: &[u8; 2],
) -> bool {
    for i in 0..64 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_65_2(
    slice_: &[u8; 65],
    search: &[u8; 2],
) -> bool {
    for i in 0..65 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_66_2(
    slice_: &[u8; 66],
    search: &[u8; 2],
) -> bool {
    for i in 0..66 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_67_2(
    slice_: &[u8; 67],
    search: &[u8; 2],
) -> bool {
    for i in 0..67 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_68_2(
    slice_: &[u8; 68],
    search: &[u8; 2],
) -> bool {
    for i in 0..68 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_69_2(
    slice_: &[u8; 69],
    search: &[u8; 2],
) -> bool {
    for i in 0..69 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_70_2(
    slice_: &[u8; 70],
    search: &[u8; 2],
) -> bool {
    for i in 0..70 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}
#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_71_2(
    slice_: &[u8; 71],
    search: &[u8; 2],
) -> bool {
    for i in 0..71 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_72_2(
    slice_: &[u8; 72],
    search: &[u8; 2],
) -> bool {
    for i in 0..72 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_73_2(
    slice_: &[u8; 73],
    search: &[u8; 2],
) -> bool {
    for i in 0..73 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_74_2(
    slice_: &[u8; 74],
    search: &[u8; 2],
) -> bool {
    for i in 0..74 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_75_2(
    slice_: &[u8; 75],
    search: &[u8; 2],
) -> bool {
    for i in 0..75 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_76_2(
    slice_: &[u8; 76],
    search: &[u8; 2],
) -> bool {
    for i in 0..76 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_77_2(
    slice_: &[u8; 77],
    search: &[u8; 2],
) -> bool {
    for i in 0..77 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_78_2(
    slice_: &[u8; 78],
    search: &[u8; 2],
) -> bool {
    for i in 0..78 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_79_2(
    slice_: &[u8; 79],
    search: &[u8; 2],
) -> bool {
    for i in 0..79 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_80_2(
    slice_: &[u8; 80],
    search: &[u8; 2],
) -> bool {
    for i in 0..80 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_81_2(
    slice_: &[u8; 81],
    search: &[u8; 2],
) -> bool {
    for i in 0..81 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_82_2(
    slice_: &[u8; 82],
    search: &[u8; 2],
) -> bool {
    for i in 0..82 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_83_2(
    slice_: &[u8; 83],
    search: &[u8; 2],
) -> bool {
    for i in 0..83 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_84_2(
    slice_: &[u8; 84],
    search: &[u8; 2],
) -> bool {
    for i in 0..84 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_85_2(
    slice_: &[u8; 85],
    search: &[u8; 2],
) -> bool {
    for i in 0..85 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_86_2(
    slice_: &[u8; 86],
    search: &[u8; 2],
) -> bool {
    for i in 0..86 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_87_2(
    slice_: &[u8; 87],
    search: &[u8; 2],
) -> bool {
    for i in 0..87 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_88_2(
    slice_: &[u8; 88],
    search: &[u8; 2],
) -> bool {
    for i in 0..88 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_89_2(
    slice_: &[u8; 89],
    search: &[u8; 2],
) -> bool {
    for i in 0..89 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_90_2(
    slice_: &[u8; 90],
    search: &[u8; 2],
) -> bool {
    for i in 0..90 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_91_2(
    slice_: &[u8; 91],
    search: &[u8; 2],
) -> bool {
    for i in 0..91 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_92_2(
    slice_: &[u8; 92],
    search: &[u8; 2],
) -> bool {
    for i in 0..92 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_93_2(
    slice_: &[u8; 93],
    search: &[u8; 2],
) -> bool {
    for i in 0..93 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_94_2(
    slice_: &[u8; 94],
    search: &[u8; 2],
) -> bool {
    for i in 0..94 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_95_2(
    slice_: &[u8; 95],
    search: &[u8; 2],
) -> bool {
    for i in 0..95 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_96_2(
    slice_: &[u8; 96],
    search: &[u8; 2],
) -> bool {
    for i in 0..96 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_97_2(
    slice_: &[u8; 97],
    search: &[u8; 2],
) -> bool {
    for i in 0..97 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_98_2(
    slice_: &[u8; 98],
    search: &[u8; 2],
) -> bool {
    for i in 0..98 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}
#[inline(always)]
#[unroll_for_loops]
const fn slice_contains_99_2(
    slice_: &[u8; 99],
    search: &[u8; 2],
) -> bool {
    for i in 0..99 {
        if slice_[i] == search[0] || slice_[i] == search[1] {
            return true;
        }
    }
    false
}

/// Loop unrolled implementation of [`slice.contains`].
/// Returns `true` if any byte in `search` is found in `slice_`.
/// Uses crate [`unroll`].
///
/// Hardcoded implementation for [`u8`] slices up to 99 length. Falls back to
/// using `slice.contains` for slices longer than 99.
///
/// Runs very fast according to benches in `src/benches/slice_contains.rs`.
///
/// [`unroll`]: https://docs.rs/unroll/0.1.5/unroll/index.html
/// [`slice.contains`]: https://doc.rust-lang.org/1.66.0/std/primitive.slice.html#method.contains
//
// slice index values for the `DTPD!` declarations can be reviewed with:
//
//     $ grep -Ee '^[[:space:]]+DTFSS_' -- src/data/datetime.rs  | sort -t ',' -n -k2 -k3 | column -t
//
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_X_2_unroll(
    slice_: &[u8],
    search: &[u8; 2],
) -> bool {
    match slice_.len() {
        2 => slice_contains_2_2(<&[u8; 2]>::try_from(slice_).unwrap(), search),
        3 => slice_contains_3_2(<&[u8; 3]>::try_from(slice_).unwrap(), search),
        4 => slice_contains_4_2(<&[u8; 4]>::try_from(slice_).unwrap(), search),
        5 => slice_contains_5_2(<&[u8; 5]>::try_from(slice_).unwrap(), search),
        6 => slice_contains_6_2(<&[u8; 6]>::try_from(slice_).unwrap(), search),
        7 => slice_contains_7_2(<&[u8; 7]>::try_from(slice_).unwrap(), search),
        8 => slice_contains_8_2(<&[u8; 8]>::try_from(slice_).unwrap(), search),
        9 => slice_contains_9_2(<&[u8; 9]>::try_from(slice_).unwrap(), search),
        10 => slice_contains_10_2(<&[u8; 10]>::try_from(slice_).unwrap(), search),
        11 => slice_contains_11_2(<&[u8; 11]>::try_from(slice_).unwrap(), search),
        12 => slice_contains_12_2(<&[u8; 12]>::try_from(slice_).unwrap(), search),
        13 => slice_contains_13_2(<&[u8; 13]>::try_from(slice_).unwrap(), search),
        14 => slice_contains_14_2(<&[u8; 14]>::try_from(slice_).unwrap(), search),
        15 => slice_contains_15_2(<&[u8; 15]>::try_from(slice_).unwrap(), search),
        16 => slice_contains_16_2(<&[u8; 16]>::try_from(slice_).unwrap(), search),
        17 => slice_contains_17_2(<&[u8; 17]>::try_from(slice_).unwrap(), search),
        18 => slice_contains_18_2(<&[u8; 18]>::try_from(slice_).unwrap(), search),
        19 => slice_contains_19_2(<&[u8; 19]>::try_from(slice_).unwrap(), search),
        20 => slice_contains_20_2(<&[u8; 20]>::try_from(slice_).unwrap(), search),
        21 => slice_contains_21_2(<&[u8; 21]>::try_from(slice_).unwrap(), search),
        22 => slice_contains_22_2(<&[u8; 22]>::try_from(slice_).unwrap(), search),
        23 => slice_contains_23_2(<&[u8; 23]>::try_from(slice_).unwrap(), search),
        24 => slice_contains_24_2(<&[u8; 24]>::try_from(slice_).unwrap(), search),
        25 => slice_contains_25_2(<&[u8; 25]>::try_from(slice_).unwrap(), search),
        26 => slice_contains_26_2(<&[u8; 26]>::try_from(slice_).unwrap(), search),
        27 => slice_contains_27_2(<&[u8; 27]>::try_from(slice_).unwrap(), search),
        28 => slice_contains_28_2(<&[u8; 28]>::try_from(slice_).unwrap(), search),
        29 => slice_contains_29_2(<&[u8; 29]>::try_from(slice_).unwrap(), search),
        30 => slice_contains_30_2(<&[u8; 30]>::try_from(slice_).unwrap(), search),
        31 => slice_contains_31_2(<&[u8; 31]>::try_from(slice_).unwrap(), search),
        32 => slice_contains_32_2(<&[u8; 32]>::try_from(slice_).unwrap(), search),
        33 => slice_contains_33_2(<&[u8; 33]>::try_from(slice_).unwrap(), search),
        34 => slice_contains_34_2(<&[u8; 34]>::try_from(slice_).unwrap(), search),
        35 => slice_contains_35_2(<&[u8; 35]>::try_from(slice_).unwrap(), search),
        36 => slice_contains_36_2(<&[u8; 36]>::try_from(slice_).unwrap(), search),
        37 => slice_contains_37_2(<&[u8; 37]>::try_from(slice_).unwrap(), search),
        38 => slice_contains_38_2(<&[u8; 38]>::try_from(slice_).unwrap(), search),
        39 => slice_contains_39_2(<&[u8; 39]>::try_from(slice_).unwrap(), search),
        40 => slice_contains_40_2(<&[u8; 40]>::try_from(slice_).unwrap(), search),
        41 => slice_contains_41_2(<&[u8; 41]>::try_from(slice_).unwrap(), search),
        42 => slice_contains_42_2(<&[u8; 42]>::try_from(slice_).unwrap(), search),
        43 => slice_contains_43_2(<&[u8; 43]>::try_from(slice_).unwrap(), search),
        44 => slice_contains_44_2(<&[u8; 44]>::try_from(slice_).unwrap(), search),
        45 => slice_contains_45_2(<&[u8; 45]>::try_from(slice_).unwrap(), search),
        46 => slice_contains_46_2(<&[u8; 46]>::try_from(slice_).unwrap(), search),
        47 => slice_contains_47_2(<&[u8; 47]>::try_from(slice_).unwrap(), search),
        48 => slice_contains_48_2(<&[u8; 48]>::try_from(slice_).unwrap(), search),
        49 => slice_contains_49_2(<&[u8; 49]>::try_from(slice_).unwrap(), search),
        50 => slice_contains_50_2(<&[u8; 50]>::try_from(slice_).unwrap(), search),
        51 => slice_contains_51_2(<&[u8; 51]>::try_from(slice_).unwrap(), search),
        52 => slice_contains_52_2(<&[u8; 52]>::try_from(slice_).unwrap(), search),
        53 => slice_contains_53_2(<&[u8; 53]>::try_from(slice_).unwrap(), search),
        54 => slice_contains_54_2(<&[u8; 54]>::try_from(slice_).unwrap(), search),
        55 => slice_contains_55_2(<&[u8; 55]>::try_from(slice_).unwrap(), search),
        56 => slice_contains_56_2(<&[u8; 56]>::try_from(slice_).unwrap(), search),
        57 => slice_contains_57_2(<&[u8; 57]>::try_from(slice_).unwrap(), search),
        58 => slice_contains_58_2(<&[u8; 58]>::try_from(slice_).unwrap(), search),
        59 => slice_contains_59_2(<&[u8; 59]>::try_from(slice_).unwrap(), search),
        60 => slice_contains_60_2(<&[u8; 60]>::try_from(slice_).unwrap(), search),
        61 => slice_contains_61_2(<&[u8; 61]>::try_from(slice_).unwrap(), search),
        62 => slice_contains_62_2(<&[u8; 62]>::try_from(slice_).unwrap(), search),
        63 => slice_contains_63_2(<&[u8; 63]>::try_from(slice_).unwrap(), search),
        64 => slice_contains_64_2(<&[u8; 64]>::try_from(slice_).unwrap(), search),
        65 => slice_contains_65_2(<&[u8; 65]>::try_from(slice_).unwrap(), search),
        66 => slice_contains_66_2(<&[u8; 66]>::try_from(slice_).unwrap(), search),
        67 => slice_contains_67_2(<&[u8; 67]>::try_from(slice_).unwrap(), search),
        68 => slice_contains_68_2(<&[u8; 68]>::try_from(slice_).unwrap(), search),
        69 => slice_contains_69_2(<&[u8; 69]>::try_from(slice_).unwrap(), search),
        70 => slice_contains_70_2(<&[u8; 70]>::try_from(slice_).unwrap(), search),
        71 => slice_contains_71_2(<&[u8; 71]>::try_from(slice_).unwrap(), search),
        72 => slice_contains_72_2(<&[u8; 72]>::try_from(slice_).unwrap(), search),
        73 => slice_contains_73_2(<&[u8; 73]>::try_from(slice_).unwrap(), search),
        74 => slice_contains_74_2(<&[u8; 74]>::try_from(slice_).unwrap(), search),
        75 => slice_contains_75_2(<&[u8; 75]>::try_from(slice_).unwrap(), search),
        76 => slice_contains_76_2(<&[u8; 76]>::try_from(slice_).unwrap(), search),
        77 => slice_contains_77_2(<&[u8; 77]>::try_from(slice_).unwrap(), search),
        78 => slice_contains_78_2(<&[u8; 78]>::try_from(slice_).unwrap(), search),
        79 => slice_contains_79_2(<&[u8; 79]>::try_from(slice_).unwrap(), search),
        80 => slice_contains_80_2(<&[u8; 80]>::try_from(slice_).unwrap(), search),
        81 => slice_contains_81_2(<&[u8; 81]>::try_from(slice_).unwrap(), search),
        82 => slice_contains_82_2(<&[u8; 82]>::try_from(slice_).unwrap(), search),
        83 => slice_contains_83_2(<&[u8; 83]>::try_from(slice_).unwrap(), search),
        84 => slice_contains_84_2(<&[u8; 84]>::try_from(slice_).unwrap(), search),
        85 => slice_contains_85_2(<&[u8; 85]>::try_from(slice_).unwrap(), search),
        86 => slice_contains_86_2(<&[u8; 86]>::try_from(slice_).unwrap(), search),
        87 => slice_contains_87_2(<&[u8; 87]>::try_from(slice_).unwrap(), search),
        88 => slice_contains_88_2(<&[u8; 88]>::try_from(slice_).unwrap(), search),
        89 => slice_contains_89_2(<&[u8; 89]>::try_from(slice_).unwrap(), search),
        90 => slice_contains_90_2(<&[u8; 90]>::try_from(slice_).unwrap(), search),
        91 => slice_contains_91_2(<&[u8; 91]>::try_from(slice_).unwrap(), search),
        92 => slice_contains_92_2(<&[u8; 92]>::try_from(slice_).unwrap(), search),
        93 => slice_contains_93_2(<&[u8; 93]>::try_from(slice_).unwrap(), search),
        94 => slice_contains_94_2(<&[u8; 94]>::try_from(slice_).unwrap(), search),
        95 => slice_contains_95_2(<&[u8; 95]>::try_from(slice_).unwrap(), search),
        96 => slice_contains_96_2(<&[u8; 96]>::try_from(slice_).unwrap(), search),
        97 => slice_contains_97_2(<&[u8; 97]>::try_from(slice_).unwrap(), search),
        98 => slice_contains_98_2(<&[u8; 98]>::try_from(slice_).unwrap(), search),
        99 => slice_contains_99_2(<&[u8; 99]>::try_from(slice_).unwrap(), search),
        _ => {
            // fallback to `slice_.contains`
            // surprisingly good performance according to benches in `bench_slice_contains`
            slice_.contains(&search[0]) || slice_.contains(&search[1])
        }
   }
}

/// `jetscii` implementation of `slice.contains` for a byte slice and a
/// hardcoded array.
#[inline(always)]
#[allow(non_snake_case)]
#[cfg(feature = "bench_jetscii")]
pub fn slice_contains_X_2_jetscii(
    slice_: &[u8],
    search: &[u8; 2],
) -> bool {
    jetscii::bytes!(search[0], search[1]).find(slice_).is_some()
}

/// `memchr` implementation of `slice.contains` for a byte slice and a
/// hardcoded array.
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_X_2_memchr(
    slice_: &[u8],
    search: &[u8; 2],
) -> bool {
    memchr::memchr2(
        search[0],
        search[1],
        slice_,
    ).is_some()
}

/// Stringzilla implementation of `slice.contains` for a byte slice and a
/// hardcoded array.
/// Uses crate [`stringzilla`].
///
/// [`stringzilla`]: https://crates.io/crates/stringzilla
#[inline(always)]
#[allow(non_snake_case)]
#[cfg(feature = "bench_stringzilla")]
pub fn slice_contains_X_2_stringzilla(
    slice_: &[u8],
    search: &[u8; 2],
) -> bool {
    stringzilla::sz::find_char_from(slice_, search).is_some()
}

/// Wrapper to call the preferred `slice_contains_X_2` function.
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_X_2(
    slice_: &[u8],
    search: &[u8; 2],
) -> bool {
    slice_contains_X_2_memchr(slice_, search)
}

/// Returns `true` if `slice_` contains consecutive "digit" chars (as UTF8 bytes).
/// Custom implementation.
/// Hack efficiency helper.
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_D2_custom(
    slice_: &[u8],
) -> bool {
    let mut byte_last_d: bool = false;
    for byte_ in slice_.iter() {
        match byte_ {
            b'0'
            | b'1'
            | b'2'
            | b'3'
            | b'4'
            | b'5'
            | b'6'
            | b'7'
            | b'8'
            | b'9' => {
                if byte_last_d {
                    return true;
                }
                byte_last_d = true;
            },
            _ => byte_last_d = false,
        }
    }

    false
}

/// Returns `true` if `slice_` contains consecutive "digit" chars (as UTF8 bytes).
/// jetscii implementation.
/// Hack efficiency helper.
#[inline(always)]
#[allow(non_snake_case)]
#[cfg(feature = "bench_jetscii")]
pub fn slice_contains_D2_jetscii(
    slice_: &[u8],
) -> bool {
    let mut atn: usize = 0;
    let mut lastn: isize = -1;
    let bytes_ = jetscii::bytes!(b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9');
    while let Some(n) = bytes_.find(&slice_[atn..]) {
        if lastn != -1 && atn + n == ((lastn + 1) as usize) {
            return true;
        }
        lastn = (atn + n) as isize;
        atn = atn + n + 1;
    }

    false
}

/// Returns `true` if `slice_` contains consecutive "digit" chars (as UTF8 bytes).
/// Stringzilla implementation.
/// Hack efficiency helper.
#[inline(always)]
#[allow(non_snake_case)]
#[cfg(feature = "bench_stringzilla")]
pub fn slice_contains_D2_stringzilla(
    slice_: &[u8],
) -> bool {
    let mut atn: usize = 0;
    let mut lastn: isize = -1;
    while let Some(n) = stringzilla::sz::find_char_from(&slice_[atn..], b"0123456789") {
        if lastn != -1 && atn + n == ((lastn + 1) as usize) {
            return true;
        }
        lastn = (atn + n) as isize;
        atn = atn + n + 1;
    }

    false
}

/// Wrapper to call the preferred `slice_contains_D2` function.
/// Returns `true` if `slice_` contains consecutive "digit" chars (as UTF8 bytes).
/// Hack efficiency helper.
#[inline(always)]
#[allow(non_snake_case)]
pub fn slice_contains_D2(
    slice_: &[u8],
) -> bool {
    slice_contains_D2_custom(slice_)
}

/// Combination of prior functions `slice_contains_X_2` and
/// `slice_contains_D2`.
///
/// This combined hack check function is more efficient.
///
/// - Returns `true` if `slice_` contains `'1'` or `'2'` (as UTF8 bytes).
/// - Returns `true` if `slice_` contains two consecutive "digit" chars
///   (as UTF8 bytes).
#[inline(always)]
#[allow(non_snake_case)]
pub (crate) fn slice_contains_12_D2(
    slice_: &[u8],
) -> bool {
    let mut byte_last_d: bool = false;
    for byte_ in slice_.iter() {
        match byte_ {
            b'1'
            | b'2' => {
                return true;
            }
            b'0'
            | b'3'
            | b'4'
            | b'5'
            | b'6'
            | b'7'
            | b'8'
            | b'9' => {
                if byte_last_d {
                    return true;
                }
                byte_last_d = true;
            },
            _ => byte_last_d = false,
        }
    }

    false
}
