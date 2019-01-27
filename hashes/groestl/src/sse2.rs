use core::arch::x86_64::*;

type X4 = (__m128i, __m128i, __m128i, __m128i);

type X8 = (
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
);

/// Combined subtract and mix; common to Large and Small variants.
#[inline(always)]
pub unsafe fn submix(a: X8) -> X8 {
    #[inline(always)]
    unsafe fn mul2(i: __m128i) -> __m128i {
        let all_1b = _mm_set1_epi64x(0x1b1b1b1b1b1b1b1b);
        let j = _mm_and_si128(_mm_cmpgt_epi8(_mm_cvtsi64_si128(0), i), all_1b);
        let i = _mm_add_epi8(i, i);
        _mm_xor_si128(i, j)
    }
    let b0 = _mm_cvtsi64_si128(0);
    let a = (
        _mm_aesenclast_si128(a.0, b0),
        _mm_aesenclast_si128(a.1, b0),
        _mm_aesenclast_si128(a.2, b0),
        _mm_aesenclast_si128(a.3, b0),
        _mm_aesenclast_si128(a.4, b0),
        _mm_aesenclast_si128(a.5, b0),
        _mm_aesenclast_si128(a.6, b0),
        _mm_aesenclast_si128(a.7, b0),
    );
    // MixBytes
    // t_i = a_i + a_{i+1}
    let t0 = _mm_xor_si128(a.0, a.1);
    let t1 = _mm_xor_si128(a.1, a.2);
    let t2 = _mm_xor_si128(a.2, a.3);
    let t3 = _mm_xor_si128(a.3, a.4);
    let t4 = _mm_xor_si128(a.4, a.5);
    let t5 = _mm_xor_si128(a.5, a.6);
    let t6 = _mm_xor_si128(a.6, a.7);
    let t7 = _mm_xor_si128(a.7, a.0);
    // build y4 y5 y6 ... by adding t_i
    let b0 = _mm_xor_si128(_mm_xor_si128(a.2, t4), t6);
    let b1 = _mm_xor_si128(_mm_xor_si128(a.3, t5), t7);
    let b2 = _mm_xor_si128(_mm_xor_si128(a.4, t6), t0);
    let b3 = _mm_xor_si128(_mm_xor_si128(a.5, t7), t1);
    let b4 = _mm_xor_si128(_mm_xor_si128(a.6, t0), t2);
    let b5 = _mm_xor_si128(_mm_xor_si128(a.7, t1), t3);
    let b6 = _mm_xor_si128(_mm_xor_si128(a.0, t2), t4);
    let b7 = _mm_xor_si128(_mm_xor_si128(a.1, t3), t5);
    // compute x_i = t_i + t_{i+3}
    let a0 = _mm_xor_si128(t0, t3);
    let a1 = _mm_xor_si128(t1, t4);
    let a2 = _mm_xor_si128(t2, t5);
    let a3 = _mm_xor_si128(t3, t6);
    let a4 = _mm_xor_si128(t4, t7);
    let a5 = _mm_xor_si128(t5, t0);
    let a6 = _mm_xor_si128(t6, t1);
    let a7 = _mm_xor_si128(t7, t2);
    // compute z_i : double x_i
    // compute w_i : add y_{i+4}
    let a0 = _mm_xor_si128(mul2(a0), b0);
    let a1 = _mm_xor_si128(mul2(a1), b1);
    let a2 = _mm_xor_si128(mul2(a2), b2);
    let a3 = _mm_xor_si128(mul2(a3), b3);
    let a4 = _mm_xor_si128(mul2(a4), b4);
    let a5 = _mm_xor_si128(mul2(a5), b5);
    let a6 = _mm_xor_si128(mul2(a6), b6);
    let a7 = _mm_xor_si128(mul2(a7), b7);
    // compute v_i : double w_i
    // add to y_4 y_5 .. v3, v4, ...
    (
        _mm_xor_si128(b0, mul2(a3)),
        _mm_xor_si128(b1, mul2(a4)),
        _mm_xor_si128(b2, mul2(a5)),
        _mm_xor_si128(b3, mul2(a6)),
        _mm_xor_si128(b4, mul2(a7)),
        _mm_xor_si128(b5, mul2(a0)),
        _mm_xor_si128(b6, mul2(a1)),
        _mm_xor_si128(b7, mul2(a2)),
    )
}

/// Matrix Transpose Step 1
/// input: a 512-bit state with two columns in one xmm
/// output: a 512-bit state with two rows in one xmm
#[inline(always)]
unsafe fn transpose_a(i: X4) -> X4 {
    let mask = _mm_set_epi64x(0x0f070b030e060a02, 0x0d0509010c040800);
    let i0 = _mm_shuffle_epi8(i.0, mask);
    let i1 = _mm_shuffle_epi8(i.1, mask);
    let i2 = _mm_shuffle_epi8(i.2, mask);
    let i3 = _mm_shuffle_epi8(i.3, mask);
    let o1 = i0;
    let t0 = i2;
    let i0 = _mm_unpacklo_epi16(i0, i1);
    let o1 = _mm_unpackhi_epi16(o1, i1);
    let i2 = _mm_unpacklo_epi16(i2, i3);
    let t0 = _mm_unpackhi_epi16(t0, i3);
    let i0 = _mm_shuffle_epi32(i0, 0b11011000);
    let o1 = _mm_shuffle_epi32(o1, 0b11011000);
    let i2 = _mm_shuffle_epi32(i2, 0b11011000);
    let t0 = _mm_shuffle_epi32(t0, 0b11011000);
    (
        _mm_unpacklo_epi32(i0, i2),
        _mm_unpacklo_epi32(o1, t0),
        _mm_unpackhi_epi32(i0, i2),
        _mm_unpackhi_epi32(o1, t0),
    )
}

/// Matrix Transpose Step 2
/// input: two 512-bit states with two rows in one xmm
/// output: two 512-bit states with one row of each state in one xmm
#[inline(always)]
unsafe fn transpose_b(i: X8) -> X8 {
    (
        _mm_unpacklo_epi64(i.0, i.4),
        _mm_unpackhi_epi64(i.0, i.4),
        _mm_unpacklo_epi64(i.1, i.5),
        _mm_unpackhi_epi64(i.1, i.5),
        _mm_unpacklo_epi64(i.2, i.6),
        _mm_unpackhi_epi64(i.2, i.6),
        _mm_unpacklo_epi64(i.3, i.7),
        _mm_unpackhi_epi64(i.3, i.7),
    )
}

/// Matrix Transpose Inverse Step 2
/// input: two 512-bit states with one row of each state in one xmm
/// output: two 512-bit states with two rows in one xmm
#[inline(always)]
unsafe fn transpose_b_inv(i: X8) -> X8 {
    (
        _mm_unpacklo_epi64(i.0, i.1),
        _mm_unpacklo_epi64(i.2, i.3),
        _mm_unpacklo_epi64(i.4, i.5),
        _mm_unpacklo_epi64(i.6, i.7),
        _mm_unpackhi_epi64(i.0, i.1),
        _mm_unpackhi_epi64(i.2, i.3),
        _mm_unpackhi_epi64(i.4, i.5),
        _mm_unpackhi_epi64(i.6, i.7),
    )
}

/// Matrix Transpose Output Step 2
/// input: one 512-bit state with two rows in one xmm
/// output: one 512-bit state with one row in the low bits of one xmm
#[inline(always)]
unsafe fn transpose_o_b(i: X4) -> X8 {
    let t0 = _mm_cvtsi64_si128(0);
    (
        _mm_unpacklo_epi64(i.0, t0),
        _mm_unpackhi_epi64(i.0, t0),
        _mm_unpacklo_epi64(i.1, t0),
        _mm_unpackhi_epi64(i.1, t0),
        _mm_unpacklo_epi64(i.2, t0),
        _mm_unpackhi_epi64(i.2, t0),
        _mm_unpacklo_epi64(i.3, t0),
        _mm_unpackhi_epi64(i.3, t0),
    )
}

/// Matrix Transpose Output Inverse Step 2
/// input: one 512-bit state with one row in the low bits of one xmm
/// output: one 512-bit state with two rows in one xmm
#[inline(always)]
unsafe fn transpose_o_b_inv(i: X8) -> X4 {
    (
        _mm_unpacklo_epi64(i.0, i.1),
        _mm_unpacklo_epi64(i.2, i.3),
        _mm_unpacklo_epi64(i.4, i.5),
        _mm_unpacklo_epi64(i.6, i.7),
    )
}

#[inline(always)]
unsafe fn round(i: i64, a: X8) -> X8 {
    // AddRoundConstant
    let ff = 0xffffffffffffffffu64 as i64;
    let l0 = _mm_set_epi64x(ff, (i * 0x0101010101010101) ^ 0x7060504030201000);
    let lx = _mm_set_epi64x(ff, 0);
    let l7 = _mm_set_epi64x((i * 0x0101010101010101) ^ 0x8f9fafbfcfdfefffu64 as i64, 0);
    let a0 = _mm_xor_si128(a.0, l0);
    let a1 = _mm_xor_si128(a.1, lx);
    let a2 = _mm_xor_si128(a.2, lx);
    let a3 = _mm_xor_si128(a.3, lx);
    let a4 = _mm_xor_si128(a.4, lx);
    let a5 = _mm_xor_si128(a.5, lx);
    let a6 = _mm_xor_si128(a.6, lx);
    let a7 = _mm_xor_si128(a.7, l7);
    // ShiftBytes + SubBytes (interleaved)
    let aa = (
        _mm_shuffle_epi8(a0, _mm_set_epi64x(0x03060a0d08020509, 0x0c0f0104070b0e00)),
        _mm_shuffle_epi8(a1, _mm_set_epi64x(0x04070c0f0a03060b, 0x0e090205000d0801)),
        _mm_shuffle_epi8(a2, _mm_set_epi64x(0x05000e090c04070d, 0x080b0306010f0a02)),
        _mm_shuffle_epi8(a3, _mm_set_epi64x(0x0601080b0e05000f, 0x0a0d040702090c03)),
        _mm_shuffle_epi8(a4, _mm_set_epi64x(0x0702090c0f060108, 0x0b0e0500030a0d04)),
        _mm_shuffle_epi8(a5, _mm_set_epi64x(0x00030b0e0907020a, 0x0d080601040c0f05)),
        _mm_shuffle_epi8(a6, _mm_set_epi64x(0x01040d080b00030c, 0x0f0a0702050e0906)),
        _mm_shuffle_epi8(a7, _mm_set_epi64x(0x02050f0a0d01040e, 0x090c000306080b07)),
    );
    submix(aa)
}

#[inline(always)]
unsafe fn rounds_p_q(p: X8) -> X8 {
    let p = round(0, p);
    let p = round(1, p);
    let p = round(2, p);
    let p = round(3, p);
    let p = round(4, p);
    let p = round(5, p);
    let p = round(6, p);
    let p = round(7, p);
    let p = round(8, p);
    let p = round(9, p);
    p
}

#[inline(always)]
unsafe fn tf512_impl(cv: &mut [__m128i; 4], data: *const __m128i) {
    let d0 = _mm_loadu_si128(data);
    let d1 = _mm_loadu_si128(data.offset(1));
    let d2 = _mm_loadu_si128(data.offset(2));
    let d3 = _mm_loadu_si128(data.offset(3));
    let (x12, x2, x6, x7) = transpose_a((d0, d1, d2, d3));
    let x8 = _mm_xor_si128(cv[0], x12);
    let x0 = _mm_xor_si128(cv[1], x2);
    let x4 = _mm_xor_si128(cv[2], x6);
    let x5 = _mm_xor_si128(cv[3], x7);
    let p = transpose_b((x8, x0, x4, x5, x12, x2, x6, x7));
    let p = rounds_p_q(p);
    let (x0, x1, x2, x3, x4, x5, x6, x7) = transpose_b_inv(p);
    cv[0] = _mm_xor_si128(cv[0], _mm_xor_si128(x0, x4));
    cv[1] = _mm_xor_si128(cv[1], _mm_xor_si128(x1, x5));
    cv[2] = _mm_xor_si128(cv[2], _mm_xor_si128(x2, x6));
    cv[3] = _mm_xor_si128(cv[3], _mm_xor_si128(x3, x7));
}

#[inline(always)]
unsafe fn of512_impl(cv: &mut [__m128i; 4]) {
    let p = transpose_o_b((cv[0], cv[1], cv[2], cv[3]));
    let p = rounds_p_q(p);
    let (x8, x10, x12, x14) = transpose_o_b_inv(p);
    let x8 = _mm_xor_si128(cv[0], x8);
    let x10 = _mm_xor_si128(cv[1], x10);
    let x12 = _mm_xor_si128(cv[2], x12);
    let x14 = _mm_xor_si128(cv[3], x14);
    let (_, _, x9, x11) = transpose_a((x8, x10, x12, x14));
    cv[2] = x9;
    cv[3] = x11;
}

#[inline(always)]
unsafe fn init512_impl(cv: [__m128i; 4]) -> [__m128i; 4] {
    core::mem::transmute(transpose_a(core::mem::transmute(cv)))
}

pub mod aes {
    use super::*;
    #[target_feature(enable = "sse2", enable = "ssse3", enable = "aes")]
    pub unsafe fn tf512(cv: &mut [__m128i; 4], data: *const __m128i) {
        tf512_impl(cv, data)
    }
    #[target_feature(enable = "sse2", enable = "ssse3", enable = "aes")]
    pub unsafe fn of512(cv: &mut [__m128i; 4]) {
        of512_impl(cv)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn init(cv: [__m128i; 4]) -> [__m128i; 4] {
        init512_impl(cv)
    }
}

#[cfg(not(target_feature = "aes"))]
pub mod ssse3 {
    use super::*;
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn tf512(cv: &mut [__m128i; 4], data: *const __m128i) {
        tf512_impl(cv, data)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn of512(cv: &mut [__m128i; 4]) {
        of512_impl(cv)
    }
    pub use super::aes::init;
}
#[cfg(target_feature = "aes")]
pub use aes as ssse3;

#[cfg(not(target_feature = "ssse3"))]
pub mod sse2 {
    use super::*;
    #[target_feature(enable = "sse2")]
    pub unsafe fn tf512(cv: &mut [__m128i; 4], data: *const __m128i) {
        tf512_impl(cv, data)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn of512(cv: &mut [__m128i; 4]) {
        of512_impl(cv)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn init(cv: [__m128i; 4]) -> [__m128i; 4] {
        init512_impl(cv)
    }
}
#[cfg(target_feature = "ssse3")]
pub use ssse3 as sse2;
