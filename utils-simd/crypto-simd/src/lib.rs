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
extern crate packed_simd;
#[cfg(feature = "packed_simd")]
mod packed_simd_impls {
    use super::*;
    use packed_simd::{u128x1, u128x2, u32x4, u64x4};
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
}
