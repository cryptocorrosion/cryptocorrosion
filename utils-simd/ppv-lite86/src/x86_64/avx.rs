
/*
#[cfg(all(feature = "avx2", target_feature = "avx2"))]
mod double_channel {
    use super::*;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct X2(__m256i);
    impl X2 {
        #[inline(always)]
        pub fn new(a: u128x1, b: u128x1) -> Self {
            X2(unsafe { _mm256_inserti128_si256(_mm256_castsi128_si256(a.raw()), b.raw(), 1) })
        }
        #[inline(always)]
        pub fn andnot(self, rhs: Self) -> Self {
            X2(unsafe { _mm256_andnot_si256(self.0, rhs.0) })
        }
        #[inline(always)]
        pub fn split(self) -> (u128x1, u128x1) {
            unsafe {
                (
                    u128x1::from_raw(_mm256_castsi256_si128(self.0)),
                    u128x1::from_raw(_mm256_extracti128_si256(self.0, 1)),
                )
            }
        }
    }
    impl BitXorAssign for X2 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            self.0 = unsafe { _mm256_xor_si256(self.0, rhs.0) };
        }
    }
    impl BitOr for X2 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            X2(unsafe { _mm256_or_si256(self.0, rhs.0) })
        }
    }
    impl BitAnd for X2 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            X2(unsafe { _mm256_and_si256(self.0, rhs.0) })
        }
    }
}

#[cfg(all(feature = "avx2", target_feature = "avx2"))]
use double_channel::X2;
*/
