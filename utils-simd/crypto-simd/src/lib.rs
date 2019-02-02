macro_rules! def_swap {
    ($swap:ident, $fn:ident) => {
        pub trait $swap {
            type Output;
            fn $fn(self) -> Self::Output;
        }
    };
}
def_swap!(Swap1, swap1);
def_swap!(Swap2, swap2);
def_swap!(Swap4, swap4);
def_swap!(Swap8, swap8);
def_swap!(Swap16, swap16);
def_swap!(Swap32, swap32);
def_swap!(Swap64, swap64);
pub trait AndNot {
    type Output;
    fn andnot(self, rhs: Self) -> Self::Output;
}
pub trait RotateWordsRight {
    type Output;
    fn rotate_words_right(self, i: u32) -> Self::Output;
}
pub trait SplatRotateRight {
    type Output;
    fn splat_rotate_right(self, i: u32) -> Self::Output;
}

#[cfg(feature = "packed_simd")]
#[macro_use]
extern crate packed_simd_crate;
#[cfg(feature = "packed_simd")]
pub mod packed_simd {
    use super::*;
    use core::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, BitXorAssign};
    use packed_simd_crate::{u128x1, u128x2, u32x16, u32x4, u64x4};
    impl AndNot for u128x2 {
        type Output = u128x2;
        #[inline(always)]
        fn andnot(self, rhs: Self) -> Self::Output {
            !self & rhs
        }
    }
    #[inline(always)]
    fn swap128(x: u128x1, m: u128, i: u32) -> u128x1 {
        let m = u128x1::new(m);
        ((x & m) >> i) | ((x << i) & m)
    }
    macro_rules! impl_swap {
        ($swap:ident, $fn:ident, $mask:expr, $n:expr) => {
            impl $swap for u128x1 {
                type Output = u128x1;
                #[inline(always)]
                fn $fn(self) -> Self::Output {
                    swap128(self, $mask, $n)
                }
            }
        };
    }
    impl_swap!(Swap1, swap1, 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, 1);
    impl_swap!(Swap2, swap2, 0xcccccccccccccccccccccccccccccccc, 2);
    impl_swap!(Swap4, swap4, 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0, 4);
    impl_swap!(Swap8, swap8, 0xff00ff00ff00ff00ff00ff00ff00ff00, 8);
    impl_swap!(Swap16, swap16, 0xffff0000ffff0000ffff0000ffff0000, 16);
    impl_swap!(Swap32, swap32, 0xffffffff00000000ffffffff00000000, 32);
    impl Swap64 for u128x1 {
        type Output = u128x1;
        #[inline(always)]
        fn swap64(self) -> Self::Output {
            (self << 64) | (self >> 64)
        }
    }
    macro_rules! impl_rotate_words_right {
        ($vec:ident) => {
            impl RotateWordsRight for $vec {
                type Output = Self;
                fn rotate_words_right(self, i: u32) -> Self::Output {
                    debug_assert_eq!(i & !3, 0);
                    match i & 3 {
                        0 => self,
                        1 => shuffle!(self, [3, 0, 1, 2]),
                        2 => shuffle!(self, [2, 3, 0, 1]),
                        3 => shuffle!(self, [1, 2, 3, 0]),
                        _ => unreachable!(),
                    }
                }
            }
        };
    }
    impl_rotate_words_right!(u32x4);
    impl_rotate_words_right!(u64x4);
    macro_rules! impl_splat_rotate_right {
        ($vec:ident, $word:ty) => {
            impl SplatRotateRight for $vec {
                type Output = Self;
                fn splat_rotate_right(self, i: u32) -> Self::Output {
                    self.rotate_right(Self::splat(i as $word))
                }
            }
        };
    }
    impl_splat_rotate_right!(u32x4, u32);
    impl_splat_rotate_right!(u64x4, u64);

    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct u32x4x4(u32x16);
    impl u32x4x4 {
        #[inline(always)]
        pub fn from((a, b, c, d): (u32x4, u32x4, u32x4, u32x4)) -> Self {
            u32x4x4(u32x16::new(
                a.extract(0),
                a.extract(1),
                a.extract(2),
                a.extract(3),
                b.extract(0),
                b.extract(1),
                b.extract(2),
                b.extract(3),
                c.extract(0),
                c.extract(1),
                c.extract(2),
                c.extract(3),
                d.extract(0),
                d.extract(1),
                d.extract(2),
                d.extract(3),
            ))
        }
        #[inline(always)]
        pub fn splat(a: u32x4) -> Self {
            u32x4x4::from((a, a, a, a))
        }
        #[inline(always)]
        pub fn into_parts(self) -> (u32x4, u32x4, u32x4, u32x4) {
            let a = u32x4::new(
                self.0.extract(0),
                self.0.extract(1),
                self.0.extract(2),
                self.0.extract(3),
            );
            let b = u32x4::new(
                self.0.extract(4),
                self.0.extract(5),
                self.0.extract(6),
                self.0.extract(7),
            );
            let c = u32x4::new(
                self.0.extract(8),
                self.0.extract(9),
                self.0.extract(10),
                self.0.extract(11),
            );
            let d = u32x4::new(
                self.0.extract(12),
                self.0.extract(13),
                self.0.extract(14),
                self.0.extract(15),
            );
            (a, b, c, d)
        }
    }
    impl BitXor for u32x4x4 {
        type Output = u32x4x4;
        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            u32x4x4(self.0 ^ rhs.0)
        }
    }
    impl BitOr for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            u32x4x4(self.0 | rhs.0)
        }
    }
    impl BitAnd for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            u32x4x4(self.0 & rhs.0)
        }
    }
    impl BitXorAssign for u32x4x4 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            self.0 ^= rhs.0;
        }
    }
    impl Add for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: Self) -> Self::Output {
            u32x4x4(self.0 + rhs.0)
        }
    }
    impl AddAssign for u32x4x4 {
        #[inline(always)]
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
        }
    }
    impl RotateWordsRight for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn rotate_words_right(self, i: u32) -> Self::Output {
            match i {
                0 => self,
                1 => u32x4x4(shuffle!(
                    self.0,
                    [3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14]
                )),
                2 => u32x4x4(shuffle!(
                    self.0,
                    [2, 3, 0, 1, 6, 7, 4, 5, 10, 11, 8, 9, 14, 15, 12, 13]
                )),
                3 => u32x4x4(shuffle!(
                    self.0,
                    [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12]
                )),
                _ => panic!("rotate_words_right index must be in the range 0..4"),
            }
        }
    }
    impl SplatRotateRight for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn splat_rotate_right(self, i: u32) -> Self::Output {
            u32x4x4(self.0.rotate_right(u32x16::splat(i)))
        }
    }
}
