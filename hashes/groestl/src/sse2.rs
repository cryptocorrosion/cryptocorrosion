use core::arch::x86_64::*;
use core::ops::BitXor;

trait Map2 {
    type Output;
    fn map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(__m128i, __m128i) -> __m128i,
        Self: Sized;
}

#[derive(Copy, Clone)]
pub struct X4(__m128i, __m128i, __m128i, __m128i);

#[derive(Copy, Clone)]
pub struct X8(
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
    __m128i,
);

impl X4 {
    #[inline(always)]
    fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(__m128i) -> __m128i,
    {
        X4(f(self.0), f(self.1), f(self.2), f(self.3))
    }
}

impl BitXor for X4 {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        (self, rhs).map(|x, y| unsafe { _mm_xor_si128(x, y) })
    }
}

impl Map2 for (X4, X4) {
    type Output = X4;
    #[inline(always)]
    fn map<F>(self: Self, mut f: F) -> Self::Output
    where
        F: FnMut(__m128i, __m128i) -> __m128i,
    {
        let (a, b) = self;
        X4(f(a.0, b.0), f(a.1, b.1), f(a.2, b.2), f(a.3, b.3))
    }
}

impl X8 {
    #[inline(always)]
    fn map<F>(self, mut f: F) -> Self
    where
        F: FnMut(__m128i) -> __m128i,
    {
        X8(
            f(self.0),
            f(self.1),
            f(self.2),
            f(self.3),
            f(self.4),
            f(self.5),
            f(self.6),
            f(self.7),
        )
    }
    #[inline(always)]
    fn shuffle(
        self,
        i0: usize,
        i1: usize,
        i2: usize,
        i3: usize,
        i4: usize,
        i5: usize,
        i6: usize,
        i7: usize,
    ) -> Self {
        let xs = [
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7,
        ];
        X8(
            xs[i0], xs[i1], xs[i2], xs[i3], xs[i4], xs[i5], xs[i6], xs[i7],
        )
    }
    #[inline(always)]
    fn rotl1(self) -> Self {
        self.shuffle(1, 2, 3, 4, 5, 6, 7, 0)
    }
    #[inline(always)]
    fn rotl2(self) -> Self {
        self.shuffle(2, 3, 4, 5, 6, 7, 0, 1)
    }
    #[inline(always)]
    fn rotl3(self) -> Self {
        self.shuffle(3, 4, 5, 6, 7, 0, 1, 2)
    }
    #[inline(always)]
    fn rotl4(self) -> Self {
        self.shuffle(4, 5, 6, 7, 0, 1, 2, 3)
    }
    #[inline(always)]
    fn rotl6(self) -> Self {
        self.shuffle(6, 7, 0, 1, 2, 3, 4, 5)
    }
}

impl BitXor for X8 {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        (self, rhs).map(|x, y| unsafe { _mm_xor_si128(x, y) })
    }
}

impl Map2 for (X8, X8) {
    type Output = X8;
    #[inline(always)]
    fn map<F>(self: Self, mut f: F) -> Self::Output
    where
        F: FnMut(__m128i, __m128i) -> __m128i,
    {
        let (a, b) = self;
        X8(
            f(a.0, b.0),
            f(a.1, b.1),
            f(a.2, b.2),
            f(a.3, b.3),
            f(a.4, b.4),
            f(a.5, b.5),
            f(a.6, b.6),
            f(a.7, b.7),
        )
    }
}

#[inline(always)]
fn mul2(i: __m128i) -> __m128i {
    unsafe {
        let all_1b = _mm_set1_epi64x(0x1b1b1b1b1b1b1b1b);
        let j = _mm_and_si128(_mm_cmpgt_epi8(_mm_cvtsi64_si128(0), i), all_1b);
        let i = _mm_add_epi8(i, i);
        _mm_xor_si128(i, j)
    }
}

/// Combined subtract and mix; common to Large and Small variants.
#[inline(always)]
unsafe fn submix(a: X8) -> X8 {
    let b0 = _mm_cvtsi64_si128(0);
    let a = a.map(|x| _mm_aesenclast_si128(x, b0));
    // MixBytes
    // t_i = a_i + a_{i+1}
    let t = a ^ a.rotl1();
    // build y4 y5 y6 ... by adding t_i
    let b = a.rotl2() ^ t.rotl4() ^ t.rotl6();
    // compute x_i = t_i + t_{i+3}
    let a = t ^ t.rotl3();
    // compute z_i : double x_i
    // compute w_i : add y_{i+4}
    let a = a.map(mul2) ^ b;
    // compute v_i : double w_i
    // add to y_4 y_5 .. v3, v4, ...
    b ^ a.rotl3().map(mul2)
}

/// Matrix Transpose Step 1
/// input: a 512-bit state with two columns in one xmm
/// output: a 512-bit state with two rows in one xmm
#[inline(always)]
unsafe fn transpose_a(i: X4) -> X4 {
    let mask = _mm_set_epi64x(0x0f070b030e060a02, 0x0d0509010c040800);
    let i = i.map(|x| _mm_shuffle_epi8(x, mask));
    let z = X4(
        _mm_unpacklo_epi16(i.0, i.1),
        _mm_unpackhi_epi16(i.0, i.1),
        _mm_unpacklo_epi16(i.2, i.3),
        _mm_unpackhi_epi16(i.2, i.3),
    ).map(|x| _mm_shuffle_epi32(x, 0b11011000));
    X4(
        _mm_unpacklo_epi32(z.0, z.2),
        _mm_unpacklo_epi32(z.1, z.3),
        _mm_unpackhi_epi32(z.0, z.2),
        _mm_unpackhi_epi32(z.1, z.3),
    )
}

/// Matrix Transpose Step 2
/// input: two 512-bit states with two rows in one xmm
/// output: two 512-bit states with one row of each state in one xmm
#[inline(always)]
unsafe fn transpose_b(i: X8) -> X8 {
    X8(
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
    X8(
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
    X8(
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
    (X4(i.0, i.2, i.4, i.6), X4(i.1, i.3, i.5, i.7)).map(|e, o| _mm_unpacklo_epi64(e, o))
}

#[inline(always)]
unsafe fn round(i: i64, a: X8) -> X8 {
    // AddRoundConstant
    let ff = 0xffffffffffffffffu64 as i64;
    let l0 = _mm_set_epi64x(ff, (i * 0x0101010101010101) ^ 0x7060504030201000);
    let lx = _mm_set_epi64x(ff, 0);
    let l7 = _mm_set_epi64x((i * 0x0101010101010101) ^ 0x8f9fafbfcfdfefffu64 as i64, 0);
    let a = a ^ X8(l0, lx, lx, lx, lx, lx, lx, l7);
    // ShiftBytes + SubBytes (interleaved)
    let mask = X8(
        _mm_set_epi64x(0x03060a0d08020509, 0x0c0f0104070b0e00),
        _mm_set_epi64x(0x04070c0f0a03060b, 0x0e090205000d0801),
        _mm_set_epi64x(0x05000e090c04070d, 0x080b0306010f0a02),
        _mm_set_epi64x(0x0601080b0e05000f, 0x0a0d040702090c03),
        _mm_set_epi64x(0x0702090c0f060108, 0x0b0e0500030a0d04),
        _mm_set_epi64x(0x00030b0e0907020a, 0x0d080601040c0f05),
        _mm_set_epi64x(0x01040d080b00030c, 0x0f0a0702050e0906),
        _mm_set_epi64x(0x02050f0a0d01040e, 0x090c000306080b07),
    );
    let a = (a, mask).map(|x, y| _mm_shuffle_epi8(x, y));
    submix(a)
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
unsafe fn tf512_impl(cv: &mut X4, data: *const __m128i) {
    let d0 = _mm_loadu_si128(data);
    let d1 = _mm_loadu_si128(data.offset(1));
    let d2 = _mm_loadu_si128(data.offset(2));
    let d3 = _mm_loadu_si128(data.offset(3));
    let y = transpose_a(X4(d0, d1, d2, d3));
    let x = (*cv, y).map(|c, x| _mm_xor_si128(c, x));
    let p = transpose_b(X8(x.0, x.1, x.2, x.3, y.0, y.1, y.2, y.3));
    let p = rounds_p_q(p);
    let p = transpose_b_inv(p);
    let x = X4(
        _mm_xor_si128(p.0, p.4),
        _mm_xor_si128(p.1, p.5),
        _mm_xor_si128(p.2, p.6),
        _mm_xor_si128(p.3, p.7),
    );
    *cv = *cv ^ x;
}

#[inline(always)]
unsafe fn of512_impl(cv: &mut X4) {
    let p = transpose_o_b(*cv);
    let p = rounds_p_q(p);
    let p = *cv ^ transpose_o_b_inv(p);
    let X4(_, _, x9, x11) = transpose_a(p);
    cv.2 = x9;
    cv.3 = x11;
}

#[inline(always)]
unsafe fn init512_impl(cv: X4) -> X4 {
    core::mem::transmute(transpose_a(core::mem::transmute(cv)))
}

#[inline(always)]
unsafe fn transpose(i: X8) -> X8 {
    let i = i.map(|x| _mm_shuffle_epi8(x, _mm_set_epi64x(0x0f070b030e060a02, 0x0d0509010c040800)));
    let (eve, odd) = (X4(i.0, i.2, i.4, i.6), X4(i.1, i.3, i.5, i.7));
    let i = (eve, odd).map(|e, o| _mm_shuffle_epi32(_mm_unpacklo_epi16(e, o), 0b11011000));
    let t = (eve, odd).map(|e, o| _mm_shuffle_epi32(_mm_unpackhi_epi16(e, o), 0b11011000));
    let t = X8(
        _mm_unpacklo_epi32(t.0, t.1),
        _mm_unpacklo_epi32(i.0, i.1),
        _mm_unpacklo_epi32(t.2, t.3),
        _mm_unpacklo_epi32(i.2, i.3),
        _mm_unpackhi_epi32(i.0, i.1),
        _mm_unpackhi_epi32(t.0, t.1),
        _mm_unpackhi_epi32(i.2, i.3),
        _mm_unpackhi_epi32(t.2, t.3),
    );
    X8(
        _mm_unpacklo_epi64(t.1, t.3),
        _mm_unpackhi_epi64(t.1, t.3),
        _mm_unpacklo_epi64(t.0, t.2),
        _mm_unpackhi_epi64(t.0, t.2),
        _mm_unpacklo_epi64(t.4, t.6),
        _mm_unpackhi_epi64(t.4, t.6),
        _mm_unpacklo_epi64(t.5, t.7),
        _mm_unpackhi_epi64(t.5, t.7),
    )
}

/// transpose matrix to get output format
#[inline(always)]
unsafe fn transpose_inv(i: X8) -> X8 {
    let i = X8(
        _mm_unpacklo_epi64(i.0, i.1),
        _mm_unpackhi_epi64(i.0, i.1),
        _mm_unpacklo_epi64(i.2, i.3),
        _mm_unpackhi_epi64(i.2, i.3),
        _mm_unpacklo_epi64(i.4, i.5),
        _mm_unpackhi_epi64(i.4, i.5),
        _mm_unpacklo_epi64(i.6, i.7),
        _mm_unpackhi_epi64(i.6, i.7),
    ).map(|x| _mm_shuffle_epi8(x, _mm_set_epi64x(0x0f070b030e060a02, 0x0d0509010c040800)));
    let i = X8(
        _mm_unpacklo_epi16(i.0, i.2),
        _mm_unpacklo_epi16(i.1, i.3),
        _mm_unpackhi_epi16(i.0, i.2),
        _mm_unpackhi_epi16(i.1, i.3),
        _mm_unpacklo_epi16(i.4, i.6),
        _mm_unpacklo_epi16(i.5, i.7),
        _mm_unpackhi_epi16(i.4, i.6),
        _mm_unpackhi_epi16(i.5, i.7),
    ).map(|x| _mm_shuffle_epi32(x, 0b11011000));
    X8(
        _mm_unpacklo_epi32(i.0, i.4),
        _mm_unpacklo_epi32(i.2, i.6),
        _mm_unpackhi_epi32(i.0, i.4),
        _mm_unpackhi_epi32(i.2, i.6),
        _mm_unpacklo_epi32(i.1, i.5),
        _mm_unpacklo_epi32(i.3, i.7),
        _mm_unpackhi_epi32(i.1, i.5),
        _mm_unpackhi_epi32(i.3, i.7),
    )
}

#[inline(always)]
unsafe fn rounds_p(mut x: X8) -> X8 {
    const O1: i64 = 0x0101010101010101;
    let mut const_p = [_mm_cvtsi64_si128(0); 14];
    for (i, p) in (0..).zip(&mut const_p) {
        *p = _mm_set_epi64x(
            (i * O1) ^ 0xf0e0d0c0b0a09080u64 as i64,
            (i * O1) ^ 0x7060504030201000,
        );
    }
    let mask = X8(
        _mm_set_epi64x(0x0306090c0f020508, 0x0b0e0104070a0d00),
        _mm_set_epi64x(0x04070a0d00030609, 0x0c0f0205080b0e01),
        _mm_set_epi64x(0x05080b0e0104070a, 0x0d000306090c0f02),
        _mm_set_epi64x(0x06090c0f0205080b, 0x0e0104070a0d0003),
        _mm_set_epi64x(0x070a0d000306090c, 0x0f0205080b0e0104),
        _mm_set_epi64x(0x080b0e0104070a0d, 0x000306090c0f0205),
        _mm_set_epi64x(0x090c0f0205080b0e, 0x0104070a0d000306),
        _mm_set_epi64x(0x0e0104070a0d0003, 0x06090c0f0205080b),
    );
    for p in const_p.chunks_exact(2) {
        // 2 rounds at a time so we can flip-flop between register sets
        x.0 = _mm_xor_si128(x.0, p[0]);
        x = (x, mask).map(|x, m| _mm_shuffle_epi8(x, m));
        x = submix(x);
        x.0 = _mm_xor_si128(x.0, p[1]);
        x = (x, mask).map(|x, m| _mm_shuffle_epi8(x, m));
        x = submix(x);
    }
    x
}

#[inline(always)]
unsafe fn rounds_q(mut x: X8) -> X8 {
    const O1: i64 = 0x0101010101010101;
    let mut const_q = [_mm_cvtsi64_si128(0); 14];
    for (i, q) in (0..).zip(&mut const_q) {
        *q = _mm_set_epi64x(
            (i * O1) ^ 0x0f1f2f3f4f5f6f7f,
            (i * O1) ^ 0x8f9fafbfcfdfefffu64 as i64,
        );
    }
    let mask = X8(
        _mm_set_epi64x(0x0306090c0f020508, 0x0b0e0104070a0d00),
        _mm_set_epi64x(0x04070a0d00030609, 0x0c0f0205080b0e01),
        _mm_set_epi64x(0x05080b0e0104070a, 0x0d000306090c0f02),
        _mm_set_epi64x(0x06090c0f0205080b, 0x0e0104070a0d0003),
        _mm_set_epi64x(0x070a0d000306090c, 0x0f0205080b0e0104),
        _mm_set_epi64x(0x080b0e0104070a0d, 0x000306090c0f0205),
        _mm_set_epi64x(0x090c0f0205080b0e, 0x0104070a0d000306),
        _mm_set_epi64x(0x0e0104070a0d0003, 0x06090c0f0205080b),
    ).shuffle(1, 3, 5, 7, 0, 2, 4, 6);
    let f = _mm_set1_epi64x(0xffffffffffffffffu64 as i64);
    for q in const_q.chunks_exact(2) {
        // 2 rounds at a time so we can flip-flop between register sets
        x = (x ^ X8(f, f, f, f, f, f, f, q[0]), mask).map(|x, m| _mm_shuffle_epi8(x, m));
        x = submix(x);
        x = (x ^ X8(f, f, f, f, f, f, f, q[1]), mask).map(|x, m| _mm_shuffle_epi8(x, m));
        x = submix(x);
    }
    x
}

#[inline(always)]
unsafe fn init1024_impl(cv: X8) -> X8 {
    core::mem::transmute(transpose(core::mem::transmute(cv)))
}

#[inline(always)]
unsafe fn tf1024_impl(cv: &mut X8, data: *const __m128i) {
    let p = X8(
        _mm_loadu_si128(data),
        _mm_loadu_si128(data.offset(1)),
        _mm_loadu_si128(data.offset(2)),
        _mm_loadu_si128(data.offset(3)),
        _mm_loadu_si128(data.offset(4)),
        _mm_loadu_si128(data.offset(5)),
        _mm_loadu_si128(data.offset(6)),
        _mm_loadu_si128(data.offset(7)),
    );
    let q = transpose(p);
    *cv = *cv ^ rounds_p(*cv ^ q);
    *cv = *cv ^ rounds_q(q);
}

#[inline(always)]
unsafe fn of1024_impl(cv: &mut X8) {
    let p = transpose_inv(*cv ^ rounds_p(*cv));
    cv.4 = p.4;
    cv.5 = p.5;
    cv.6 = p.6;
    cv.7 = p.7;
}

#[cfg(any(feature = "std", target_feature = "aes"))]
pub mod aes {
    use super::*;
    #[target_feature(enable = "sse2", enable = "ssse3", enable = "aes")]
    pub unsafe fn tf512(cv: &mut X4, data: *const __m128i) {
        tf512_impl(cv, data)
    }
    #[target_feature(enable = "sse2", enable = "ssse3", enable = "aes")]
    pub unsafe fn of512(cv: &mut X4) {
        of512_impl(cv)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn init512(cv: X4) -> X4 {
        init512_impl(cv)
    }
    #[target_feature(enable = "sse2", enable = "ssse3", enable = "aes")]
    pub unsafe fn tf1024(cv: &mut X8, data: *const __m128i) {
        tf1024_impl(cv, data)
    }
    #[target_feature(enable = "sse2", enable = "ssse3", enable = "aes")]
    pub unsafe fn of1024(cv: &mut X8) {
        of1024_impl(cv)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn init1024(cv: X8) -> X8 {
        init1024_impl(cv)
    }
}

#[cfg(not(target_feature = "aes"))]
#[cfg(any(feature = "std", target_feature = "ssse3"))]
pub mod ssse3 {
    use super::*;
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn tf512(cv: &mut X4, data: *const __m128i) {
        tf512_impl(cv, data)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn of512(cv: &mut X4) {
        of512_impl(cv)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn tf1024(cv: &mut X8, data: *const __m128i) {
        tf1024_impl(cv, data)
    }
    #[target_feature(enable = "sse2", enable = "ssse3")]
    pub unsafe fn of1024(cv: &mut X8) {
        of1024_impl(cv)
    }
    pub use super::aes::{init1024, init512};
}
#[cfg(target_feature = "aes")]
pub use aes as ssse3;

#[cfg(not(target_feature = "ssse3"))]
#[cfg(any(feature = "std", target_feature = "sse2"))]
pub mod sse2 {
    use super::*;
    #[target_feature(enable = "sse2")]
    pub unsafe fn tf512(cv: &mut X4, data: *const __m128i) {
        tf512_impl(cv, data)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn of512(cv: &mut X4) {
        of512_impl(cv)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn init512(cv: X4) -> X4 {
        init512_impl(cv)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn tf1024(cv: &mut X8, data: *const __m128i) {
        tf1024_impl(cv, data)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn of1024(cv: &mut X8) {
        of1024_impl(cv)
    }
    #[target_feature(enable = "sse2")]
    pub unsafe fn init1024(cv: X8) -> X8 {
        init1024_impl(cv)
    }
}
#[cfg(target_feature = "ssse3")]
pub use ssse3 as sse2;

#[cfg(all(not(feature = "std"), target_feature = "sse2"))]
pub use sse2::*;

#[cfg(feature = "std")]
mod autodetect {
    use super::*;
    type Tf<T> = unsafe fn(cv: &mut T, data: *const __m128i);
    type Of<T> = unsafe fn(cv: &mut T);
    type Init<T> = unsafe fn(cv: T) -> T;
    macro_rules! dispatch {
        ($fn:ident, $ty:ty) => {
            fn dispatch_init() -> $ty {
                if is_x86_feature_detected!("aes") {
                    aes::$fn
                } else if is_x86_feature_detected!("ssse3") {
                    ssse3::$fn
                } else if is_x86_feature_detected!("sse2") {
                    sse2::$fn
                } else {
                    panic!("groestl_aesni requires at least sse2 (not detected)")
                }
            }
            lazy_static! {
                static ref IMPL: $ty = { dispatch_init() };
            }
        };
    }
    #[inline]
    pub fn tf512(cv: &mut X4, data: *const __m128i) {
        dispatch!(tf512, Tf<X4>);
        unsafe { IMPL(cv, data) }
    }
    #[inline]
    pub fn of512(cv: &mut X4) {
        dispatch!(of512, Of<X4>);
        unsafe { IMPL(cv) }
    }
    #[inline]
    pub fn init512(cv: X4) -> X4 {
        dispatch!(init512, Init<X4>);
        unsafe { IMPL(cv) }
    }
    #[inline]
    pub fn tf1024(cv: &mut X8, data: *const __m128i) {
        dispatch!(tf1024, Tf<X8>);
        unsafe { IMPL(cv, data) }
    }
    #[inline]
    pub fn of1024(cv: &mut X8) {
        dispatch!(of1024, Of<X8>);
        unsafe { IMPL(cv) }
    }
    #[inline]
    pub fn init1024(cv: X8) -> X8 {
        dispatch!(init1024, Init<X8>);
        unsafe { IMPL(cv) }
    }
}
#[cfg(feature = "std")]
pub use autodetect::*;

#[cfg(test)]
mod test {
    use super::*;

    use core::cmp::PartialEq;
    use core::fmt::{Debug, Formatter, Result};
    impl Debug for X8 {
        fn fmt(&self, f: &mut Formatter) -> Result {
            unsafe {
                f.debug_tuple("X8")
                    .field(&(_mm_extract_epi64(self.0, 0), _mm_extract_epi64(self.0, 1)))
                    .field(&(_mm_extract_epi64(self.1, 0), _mm_extract_epi64(self.1, 1)))
                    .field(&(_mm_extract_epi64(self.2, 0), _mm_extract_epi64(self.2, 1)))
                    .field(&(_mm_extract_epi64(self.3, 0), _mm_extract_epi64(self.3, 1)))
                    .field(&(_mm_extract_epi64(self.4, 0), _mm_extract_epi64(self.4, 1)))
                    .field(&(_mm_extract_epi64(self.5, 0), _mm_extract_epi64(self.5, 1)))
                    .field(&(_mm_extract_epi64(self.6, 0), _mm_extract_epi64(self.6, 1)))
                    .field(&(_mm_extract_epi64(self.7, 0), _mm_extract_epi64(self.7, 1)))
                    .finish()
            }
        }
    }
    impl PartialEq for X8 {
        fn eq(&self, rhs: &Self) -> bool {
            unsafe {
                let e = (*self, *rhs).map(|x, y| _mm_cmpeq_epi8(x, y));
                let e = _mm_and_si128(
                    _mm_and_si128(_mm_and_si128(e.0, e.1), _mm_and_si128(e.2, e.3)),
                    _mm_and_si128(_mm_and_si128(e.4, e.5), _mm_and_si128(e.6, e.7)),
                );
                _mm_extract_epi64(e, 0) & _mm_extract_epi64(e, 1) == 0xffffffffffffffffu64 as i64
            }
        }
    }

    #[test]
    fn test_transpose_invertible() {
        unsafe {
            let x = X8(
                _mm_cvtsi64_si128(0),
                _mm_cvtsi64_si128(1),
                _mm_cvtsi64_si128(2),
                _mm_cvtsi64_si128(3),
                _mm_cvtsi64_si128(4),
                _mm_cvtsi64_si128(5),
                _mm_cvtsi64_si128(6),
                _mm_cvtsi64_si128(7),
            );
            assert_eq!(x, transpose_inv(transpose(x)));
            let y = X8(
                _mm_set_epi64x(0x0306090c0f020508, 0x0b0e0104070a0d00),
                _mm_set_epi64x(0x04070a0d00030609, 0x0c0f0205080b0e01),
                _mm_set_epi64x(0x05080b0e0104070a, 0x0d000306090c0f02),
                _mm_set_epi64x(0x06090c0f0205080b, 0x0e0104070a0d0003),
                _mm_set_epi64x(0x070a0d000306090c, 0x0f0205080b0e0104),
                _mm_set_epi64x(0x080b0e0104070a0d, 0x000306090c0f0205),
                _mm_set_epi64x(0x090c0f0205080b0e, 0x0104070a0d000306),
                _mm_set_epi64x(0x0e0104070a0d0003, 0x06090c0f0205080b),
            );
            assert_eq!(y, transpose_inv(transpose(y)));
        }
    }
}
