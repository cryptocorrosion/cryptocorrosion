#![feature(test)]
extern crate blake_hash;
extern crate test;

use blake_hash::simd::machine::x86;
use blake_hash::simd::Machine;
use test::Bencher;

macro_rules! mach_bench {
    ($compressor:ident, $X4:ident, $MachName:ident, $feature:expr, $enable:expr) => {
        #[allow(non_snake_case)]
        #[allow(non_snake_case)]
        #[bench]
        pub fn $MachName(b: &mut Bencher) {
            if !$enable {
                return;
            }
            let m = unsafe { x86::$MachName::instance() };
            let mut state = blake_hash::$compressor::default();
            let input = [0; 128];
            #[target_feature(enable = $feature)]
            unsafe fn runner<M: Machine>(
                m: M,
                state: &mut blake_hash::$compressor,
                input: &[u8; 128],
            ) {
                for _ in 0..10240 / (std::mem::size_of::<blake_hash::$compressor>() * 4) {
                    blake_hash::$X4::put_block(m, state, std::mem::transmute(input), (0, 0));
                }
            }
            b.iter(|| unsafe { runner(m, &mut state, &input) });
            b.bytes = 10240;
        }
    };
}

mod blake256 {
    use super::*;
    mach_bench!(
        Compressor256,
        u32x4,
        SSE2,
        "sse2",
        is_x86_feature_detected!("sse2")
    );
    mach_bench!(
        Compressor256,
        u32x4,
        SSSE3,
        "ssse3",
        is_x86_feature_detected!("ssse3")
    );
    mach_bench!(
        Compressor256,
        u32x4,
        SSE41,
        "sse4.1",
        is_x86_feature_detected!("sse4.1")
    );
    mach_bench!(
        Compressor256,
        u32x4,
        AVX,
        "avx",
        is_x86_feature_detected!("avx")
    );
    mach_bench!(
        Compressor256,
        u32x4,
        AVX2,
        "avx2",
        is_x86_feature_detected!("avx2")
    );
}

mod blake512 {
    use super::*;
    mach_bench!(
        Compressor512,
        u64x4,
        SSE2,
        "sse2",
        is_x86_feature_detected!("sse2")
    );
    mach_bench!(
        Compressor512,
        u64x4,
        SSSE3,
        "ssse3",
        is_x86_feature_detected!("ssse3")
    );
    mach_bench!(
        Compressor512,
        u64x4,
        SSE41,
        "sse4.1",
        is_x86_feature_detected!("sse4.1")
    );
    mach_bench!(
        Compressor512,
        u64x4,
        AVX,
        "avx",
        is_x86_feature_detected!("avx")
    );
    mach_bench!(
        Compressor512,
        u64x4,
        AVX2,
        "avx2",
        is_x86_feature_detected!("avx2")
    );
}
