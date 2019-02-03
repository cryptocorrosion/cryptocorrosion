use core::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, BitXorAssign, Not};
use crypto_simd::*;

macro_rules! define_vec1 {
    ($X1:ident, $word:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone)]
        pub struct $X1($word);
        impl $X1 {
            #[inline(always)]
            pub fn new(a: $word) -> Self {
                $X1(a)
            }
            #[inline(always)]
            pub fn rotate_right(&mut self, i: $word) {
                self.0 = self.0.rotate_right(i as u32);
            }
            #[inline(always)]
            pub fn load(xs: &[$word]) -> Self {
                debug_assert_eq!(xs.len(), 1);
                $X1(xs[0])
            }
            #[inline(always)]
            pub fn xor_store(self, xs: &mut [$word]) {
                debug_assert_eq!(xs.len(), 1);
                xs[0] ^= self.0;
            }
            #[inline(always)]
            pub fn into_inner(self) -> $word {
                self.0
            }
            #[inline(always)]
            fn swap(self, m: u128, i: u32) -> Self {
                $X1(((self.0 & m) >> i) | ((self.0) << i) & m)
            }
            #[inline(always)]
            pub fn swap1(self) -> Self {
                self.swap(0xaaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa_aaaa, 1)
            }
            #[inline(always)]
            pub fn swap2(self) -> Self {
                self.swap(0xcccc_cccc_cccc_cccc_cccc_cccc_cccc_cccc, 2)
            }
            #[inline(always)]
            pub fn swap4(self) -> Self {
                self.swap(0xf0f0_f0f0_f0f0_f0f0_f0f0_f0f0_f0f0_f0f0, 4)
            }
            #[inline(always)]
            pub fn swap8(self) -> Self {
                self.swap(0xff00_ff00_ff00_ff00_ff00_ff00_ff00_ff00, 8)
            }
            #[inline(always)]
            pub fn swap16(self) -> Self {
                self.swap(0xffff_0000_ffff_0000_ffff_0000_ffff_0000, 16)
            }
            #[inline(always)]
            pub fn swap32(self) -> Self {
                self.swap(0xffff_ffff_0000_0000_ffff_ffff_0000_0000, 32)
            }
            #[inline(always)]
            pub fn swap64(self) -> Self {
                $X1(self.0 << 64 | self.0 >> 64)
            }
            #[inline(always)]
            pub fn andnot(self, rhs: Self) -> Self {
                !self & rhs
            }
            #[inline(always)]
            pub fn extract(self, i: u32) -> $word {
                debug_assert_eq!(i, 0);
                self.0
            }
        }
        impl AddAssign for $X1 {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl BitXorAssign for $X1 {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
        impl BitXor for $X1 {
            type Output = Self;
            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self::Output {
                $X1(self.0 ^ rhs.0)
            }
        }
        impl BitAnd for $X1 {
            type Output = Self;
            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                $X1(self.0 & rhs.0)
            }
        }
        impl Not for $X1 {
            type Output = Self;
            #[inline(always)]
            fn not(self) -> Self::Output {
                $X1(!self.0)
            }
        }
    };
}
macro_rules! define_vec2 {
    ($X2:ident, $word:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone)]
        pub struct $X2($word, $word);
        impl $X2 {
            #[inline(always)]
            pub fn new(a: $word, b: $word) -> Self {
                $X2(a, b)
            }
            #[inline(always)]
            fn map<F>(self, mut f: F) -> Self
            where
                F: FnMut($word) -> $word,
            {
                $X2(f(self.0), f(self.1))
            }
            #[inline(always)]
            fn zipmap<F>(self, rhs: Self, mut f: F) -> Self
            where
                F: FnMut($word, $word) -> $word,
            {
                $X2(f(self.0, rhs.0), f(self.1, rhs.1))
            }
            #[inline(always)]
            pub fn rotate_right(&mut self, i: $word) {
                *self = self.map(|x| $word::rotate_right(x, i as u32));
            }
            #[inline(always)]
            pub fn load(xs: &[$word]) -> Self {
                debug_assert_eq!(xs.len(), 2);
                $X2(xs[0], xs[1])
            }
            #[inline(always)]
            pub fn xor_store(self, xs: &mut [$word]) {
                debug_assert_eq!(xs.len(), 2);
                xs[0] ^= self.0;
                xs[1] ^= self.1;
            }
            #[inline(always)]
            pub fn extract(self, i: u32) -> $word {
                let x = [self.0, self.1];
                x[i as usize]
            }
            #[inline(always)]
            pub fn andnot(self, rhs: Self) -> Self {
                !self & rhs
            }
        }
        impl AddAssign for $X2 {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = self.zipmap(rhs, $word::wrapping_add);
            }
        }
        impl BitXorAssign for $X2 {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = self.zipmap(rhs, $word::bitxor);
            }
        }
        impl BitAnd for $X2 {
            type Output = Self;
            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                $X2(self.0 & rhs.0, self.1 & rhs.1)
            }
        }
        impl Not for $X2 {
            type Output = Self;
            #[inline(always)]
            fn not(self) -> Self::Output {
                $X2(!self.0, !self.1)
            }
        }
        impl BitOr for $X2 {
            type Output = Self;
            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                $X2(self.0 | rhs.0, self.1 | rhs.1)
            }
        }
    };
}

macro_rules! zipmap_impl {
    ($vec:ident, $word:ident, $trait:ident, $fn:ident, $impl_fn:ident) => {
        impl $trait for $vec {
            type Output = Self;
            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                self.zipmap(rhs, $word::$impl_fn)
            }
        }
    };
    ($vec:ident, $word:ident, $trait:ident, $fn:ident) => {
        zipmap_impl!($vec, $word, $trait, $fn, $fn);
    };
}

macro_rules! define_vec4 {
    ($X4:ident, $word:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone)]
        pub struct $X4($word, $word, $word, $word);
        impl $X4 {
            #[inline(always)]
            pub fn new(a: $word, b: $word, c: $word, d: $word) -> Self {
                $X4(a, b, c, d)
            }
            #[inline(always)]
            fn zipmap<F>(self, rhs: Self, mut f: F) -> Self
            where
                F: FnMut($word, $word) -> $word,
            {
                $X4(
                    f(self.0, rhs.0),
                    f(self.1, rhs.1),
                    f(self.2, rhs.2),
                    f(self.3, rhs.3),
                )
            }
            #[inline(always)]
            pub fn rotate_right(&mut self, ii: Self) -> Self {
                $X4(
                    self.0.rotate_right(ii.0 as u32),
                    self.1.rotate_right(ii.1 as u32),
                    self.2.rotate_right(ii.2 as u32),
                    self.3.rotate_right(ii.3 as u32),
                )
            }
            #[inline(always)]
            pub fn from_slice_unaligned(xs: &[$word]) -> Self {
                debug_assert_eq!(xs.len(), 4);
                $X4(xs[0], xs[1], xs[2], xs[3])
            }
            #[inline(always)]
            pub fn splat(x: $word) -> Self {
                $X4(x, x, x, x)
            }
            #[inline(always)]
            pub fn write_to_slice_unaligned(self, xs: &mut [$word]) {
                debug_assert_eq!(xs.len(), 4);
                xs[0] = self.0;
                xs[1] = self.1;
                xs[2] = self.2;
                xs[3] = self.3;
            }
            #[inline(always)]
            pub fn replace(mut self, i: usize, v: $word) -> Self {
                let xs = [&mut self.0, &mut self.1, &mut self.2, &mut self.3];
                *xs[i] = v;
                self
            }
            #[inline(always)]
            pub fn extract(self, i: usize) -> $word {
                let xs = [self.0, self.1, self.2, self.3];
                xs[i]
            }
        }
        impl AddAssign for $X4 {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = self.zipmap(rhs, $word::wrapping_add);
            }
        }
        impl BitXorAssign for $X4 {
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = self.zipmap(rhs, $word::bitxor);
            }
        }
        zipmap_impl!($X4, $word, Add, add, wrapping_add);
        zipmap_impl!($X4, $word, BitXor, bitxor);
        zipmap_impl!($X4, $word, BitOr, bitor);
        zipmap_impl!($X4, $word, BitAnd, bitand);
        impl RotateWordsRight for $X4 {
            type Output = Self;
            #[inline(always)]
            fn rotate_words_right(self, i: u32) -> Self::Output {
                debug_assert_eq!(i & !3, 0);
                match i & 3 {
                    0 => self,
                    1 => $X4(self.3, self.0, self.1, self.2),
                    2 => $X4(self.2, self.3, self.0, self.1),
                    3 => $X4(self.1, self.2, self.3, self.0),
                    _ => unreachable!(),
                }
            }
        }
        impl SplatRotateRight for $X4 {
            type Output = Self;
            #[inline(always)]
            fn splat_rotate_right(self, i: u32) -> Self::Output {
                const BITS: u32 = core::mem::size_of::<$word>() as u32 * 8;
                $X4(
                    (self.0 >> i) | (self.0 << (BITS - i)),
                    (self.1 >> i) | (self.1 << (BITS - i)),
                    (self.2 >> i) | (self.2 << (BITS - i)),
                    (self.3 >> i) | (self.3 << (BITS - i)),
                )
            }
        }
    };
}

define_vec4!(u32x4, u32);
define_vec4!(u64x4, u64);
define_vec1!(u128x1, u128);
define_vec2!(u128x2, u128);

// TODO: macroize this for other types
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct u32x4x4(u32x4, u32x4, u32x4, u32x4);
impl u32x4x4 {
    #[inline(always)]
    fn zipmap<F>(self, rhs: Self, mut f: F) -> Self
    where
        F: FnMut(u32x4, u32x4) -> u32x4,
    {
        u32x4x4(
            f(self.0, rhs.0),
            f(self.1, rhs.1),
            f(self.2, rhs.2),
            f(self.3, rhs.3),
        )
    }
    #[inline(always)]
    pub fn from((a, b, c, d): (u32x4, u32x4, u32x4, u32x4)) -> Self {
        u32x4x4(a, b, c, d)
    }
    #[inline(always)]
    pub fn splat(a: u32x4) -> Self {
        u32x4x4(a, a, a, a)
    }
    #[inline(always)]
    pub fn into_parts(self) -> (u32x4, u32x4, u32x4, u32x4) {
        (self.0, self.1, self.2, self.3)
    }
}
zipmap_impl!(u32x4x4, u32x4, BitXor, bitxor);
zipmap_impl!(u32x4x4, u32x4, BitOr, bitor);
zipmap_impl!(u32x4x4, u32x4, BitAnd, bitand);
zipmap_impl!(u32x4x4, u32x4, Add, add);
impl BitXorAssign for u32x4x4 {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 = self.0 ^ rhs.0;
        self.1 = self.1 ^ rhs.1;
        self.2 = self.2 ^ rhs.2;
        self.3 = self.3 ^ rhs.3;
    }
}
impl AddAssign for u32x4x4 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
        self.1 = self.1 + rhs.1;
        self.2 = self.2 + rhs.2;
        self.3 = self.3 + rhs.3;
    }
}
impl RotateWordsRight for u32x4x4 {
    type Output = Self;
    #[inline(always)]
    fn rotate_words_right(self, i: u32) -> Self::Output {
        u32x4x4(
            self.0.rotate_words_right(i),
            self.1.rotate_words_right(i),
            self.2.rotate_words_right(i),
            self.3.rotate_words_right(i),
        )
    }
}
impl SplatRotateRight for u32x4x4 {
    type Output = Self;
    #[inline(always)]
    fn splat_rotate_right(self, i: u32) -> Self::Output {
        u32x4x4(
            self.0.splat_rotate_right(i),
            self.1.splat_rotate_right(i),
            self.2.splat_rotate_right(i),
            self.3.splat_rotate_right(i),
        )
    }
}
