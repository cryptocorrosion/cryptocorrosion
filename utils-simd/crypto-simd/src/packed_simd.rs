use core::ops::{AddAssign, BitXorAssign};
#[macro_use]
extern crate packed_simd;

use packed_simd::Simd;
macro_rules! define_vec4 {
    ($Vec4:ident, $word:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone)]
        pub struct $Vec4(Simd<[$word; 4]>);
        impl $Vec4 {
            #[inline(always)]
            pub fn new(a: $word, b: $word, c: $word, d: $word) -> Self {
                $Vec4(Simd::<[$word; 4]>::new(a, b, c, d))
            }
            #[inline(always)]
            pub fn diagonalize((a, b, c, d): (Self, Self, Self, Self)) -> (Self, Self, Self, Self) {
                let b = Self(shuffle!(b.0, [1, 2, 3, 0]));
                let c = Self(shuffle!(c.0, [2, 3, 0, 1]));
                let d = Self(shuffle!(d.0, [3, 0, 1, 2]));
                (a, b, c, d)
            }
            #[inline(always)]
            pub fn undiagonalize(
                (a, b, c, d): (Self, Self, Self, Self),
            ) -> (Self, Self, Self, Self) {
                let b = Self(shuffle!(b.0, [3, 0, 1, 2]));
                let c = Self(shuffle!(c.0, [2, 3, 0, 1]));
                let d = Self(shuffle!(d.0, [1, 2, 3, 0]));
                (a, b, c, d)
            }
            #[inline(always)]
            pub fn rotate_right(&mut self, i: $word) {
                self.0 = self.0.rotate_right(Simd::<[$word; 4]>::splat(i));
            }
            #[inline(always)]
            pub fn load(xs: &[$word]) -> Self {
                $Vec4(Simd::<[$word; 4]>::from_slice_unaligned(xs))
            }
            #[inline(always)]
            pub fn xor_store(self, xs: &mut [$word]) {
                debug_assert_eq!(xs.len(), 4);
                xs[0] ^= self.0.extract(0);
                xs[1] ^= self.0.extract(1);
                xs[2] ^= self.0.extract(2);
                xs[3] ^= self.0.extract(3);
            }
        }
        impl AddAssign for $Vec4 {
            #[inline(always)]
            pub fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl BitXorAssign for $Vec4 {
            #[inline(always)]
            pub fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
    };
}
define_vec4!(u32x4, u32);
define_vec4!(u64x4, u64);
