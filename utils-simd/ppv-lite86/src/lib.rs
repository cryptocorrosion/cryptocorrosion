#![no_std]

use core::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, BitXorAssign, Not};
use crypto_simd::*;

//#[cfg(all(feature = "simd", target_feature = "sse2"))]
mod sse2 {
    use super::*;
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    #[repr(transparent)]
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct u128x1(__m128i);
    macro_rules! swapi {
        ($x:expr, $i:expr, $k:expr) => {
            unsafe {
                const K: u8 = $k;
                let k = _mm_set1_epi8(K as i8);
                u128x1(_mm_or_si128(
                    _mm_srli_epi16(_mm_and_si128($x.0, k), $i),
                    _mm_and_si128(_mm_slli_epi16($x.0, $i), k),
                ))
            }
        };
    }
    impl u128x1 {
        #[inline(always)]
        pub fn new(x: u128) -> Self {
            u128x1(unsafe { core::mem::transmute(x) })
        }
        #[inline(always)]
        pub fn extract(self, i: u32) -> u128 {
            debug_assert_eq!(i, 0);
            unsafe { core::mem::transmute(self) }
        }
        #[cfg(not(all(feature = "avx2", target_feature = "avx2")))]
        #[inline(always)]
        pub fn andnot(self, rhs: Self) -> Self {
            u128x1(unsafe { _mm_andnot_si128(self.0, rhs.0) })
        }
        #[inline(always)]
        pub fn swap1(self) -> Self {
            swapi!(self, 1, 0xaa)
        }
        #[inline(always)]
        pub fn swap2(self) -> Self {
            swapi!(self, 2, 0xcc)
        }
        #[inline(always)]
        pub fn swap4(self) -> Self {
            swapi!(self, 4, 0xf0)
        }
        #[cfg(target_feature = "ssse3")]
        #[inline(always)]
        pub fn swap8(self) -> Self {
            u128x1(unsafe {
                let k = _mm_set_epi64x(0x0e0f_0c0d_0a0b_0809, 0x0607_0405_0203_0001);
                _mm_shuffle_epi8(self.0, k)
            })
        }
        #[cfg(not(target_feature = "ssse3"))]
        #[inline(always)]
        pub fn swap8(self) -> Self {
            u128x1(unsafe { _mm_or_si128(_mm_slli_epi16(self.0, 8), _mm_srli_epi16(self.0, 8)) })
        }
        #[cfg(target_feature = "ssse3")]
        #[inline(always)]
        pub fn swap16(self) -> Self {
            u128x1(unsafe {
                let k = _mm_set_epi64x(0x0d0c_0f0e_0908_0b0a, 0x0504_0706_0100_0302);
                _mm_shuffle_epi8(self.0, k)
            })
        }
        #[cfg(not(target_feature = "ssse3"))]
        #[inline(always)]
        pub fn swap16(self) -> Self {
            u128x1(unsafe {
                _mm_shufflehi_epi16(_mm_shufflelo_epi16(self.0, 0b10110001), 0b10110001)
            })
        }
        #[inline(always)]
        pub fn swap32(self) -> Self {
            u128x1(unsafe { _mm_shuffle_epi32(self.0, 0b10110001) })
        }
        #[inline(always)]
        pub fn swap64(self) -> Self {
            u128x1(unsafe { _mm_shuffle_epi32(self.0, 0b01001110) })
        }
    }
    impl BitXor for u128x1 {
        type Output = u128x1;
        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            u128x1(unsafe { _mm_xor_si128(self.0, rhs.0) })
        }
    }
    impl BitOr for u128x1 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            u128x1(unsafe { _mm_or_si128(self.0, rhs.0) })
        }
    }
    impl BitAnd for u128x1 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            u128x1(unsafe { _mm_and_si128(self.0, rhs.0) })
        }
    }
    impl BitXorAssign for u128x1 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            *self = *self ^ rhs;
        }
    }

    #[repr(transparent)]
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct u32x4(__m128i);
    impl u32x4 {
        #[inline(always)]
        pub fn new(a: u32, b: u32, c: u32, d: u32) -> Self {
            u32x4(unsafe { _mm_set_epi32(d as i32, c as i32, b as i32, a as i32) })
        }
        #[inline(always)]
        pub fn from_slice_unaligned(xs: &[u32]) -> Self {
            assert_eq!(xs.len(), 4);
            u32x4(unsafe { _mm_loadu_si128(xs.as_ptr() as *const _) })
        }
        #[inline(always)]
        pub fn write_to_slice_unaligned(self, xs: &mut [u32]) {
            assert_eq!(xs.len(), 4);
            unsafe { _mm_storeu_si128(xs.as_mut_ptr() as *mut _, self.0) };
        }
        #[inline(always)]
        pub fn splat(x: u32) -> Self {
            u32x4(unsafe { _mm_set1_epi32(x as i32) })
        }
        #[inline(always)]
        pub fn extract(self, i: u32) -> u32 {
            unsafe {
                match i {
                    0 => _mm_extract_epi32(self.0, 0) as u32,
                    1 => _mm_extract_epi32(self.0, 1) as u32,
                    2 => _mm_extract_epi32(self.0, 2) as u32,
                    3 => _mm_extract_epi32(self.0, 3) as u32,
                    _ => unreachable!(),
                }
            }
        }
        #[inline(always)]
        pub fn replace(self, i: usize, v: u32) -> Self {
            u32x4(unsafe {
                match i {
                    0 => _mm_insert_epi32(self.0, v as i32, 0),
                    1 => _mm_insert_epi32(self.0, v as i32, 1),
                    2 => _mm_insert_epi32(self.0, v as i32, 2),
                    3 => _mm_insert_epi32(self.0, v as i32, 3),
                    _ => unreachable!(),
                }
            })
        }
    }
    impl BitXor for u32x4 {
        type Output = u32x4;
        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            u32x4(unsafe { _mm_xor_si128(self.0, rhs.0) })
        }
    }
    impl BitOr for u32x4 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            u32x4(unsafe { _mm_or_si128(self.0, rhs.0) })
        }
    }
    impl BitAnd for u32x4 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            u32x4(unsafe { _mm_and_si128(self.0, rhs.0) })
        }
    }
    impl BitXorAssign for u32x4 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            *self = *self ^ rhs;
        }
    }
    impl Add for u32x4 {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: Self) -> Self::Output {
            unsafe { u32x4(_mm_add_epi32(self.0, rhs.0)) }
        }
    }
    impl AddAssign for u32x4 {
        #[inline(always)]
        fn add_assign(&mut self, rhs: Self) {
            *self = *self + rhs
        }
    }
    impl RotateWordsRight for u32x4 {
        type Output = Self;
        #[inline(always)]
        fn rotate_words_right(self, i: u32) -> Self::Output {
            debug_assert_eq!(i & !3, 0);
            u32x4(unsafe {
                match i & 3 {
                    1 => _mm_shuffle_epi32(self.0, 0b10010011),
                    2 => _mm_shuffle_epi32(self.0, 0b01001110),
                    3 => _mm_shuffle_epi32(self.0, 0b00111001),
                    0 => self.0,
                    _ => unreachable!(),
                }
            })
        }
    }
    impl SplatRotateRight for u32x4 {
        type Output = Self;
        #[inline(always)]
        fn splat_rotate_right(self, i: u32) -> Self::Output {
            macro_rules! rotr {
                ($i:expr) => {
                    _mm_or_si128(
                        _mm_srli_epi32(self.0, $i as i32),
                        _mm_slli_epi32(self.0, 32 - $i as i32),
                    )
                };
            }
            u32x4(unsafe {
                match i {
                    7 => rotr!(7),
                    8 => _mm_shuffle_epi8(
                        self.0,
                        _mm_set_epi64x(0x0c0f0e0d_080b0a09, 0x04070605_00030201),
                    ),
                    12 => rotr!(12),
                    16 => _mm_shuffle_epi8(
                        self.0,
                        _mm_set_epi64x(0x0d0c0f0e_09080b0a, 0x05040706_01000302),
                    ),
                    20 => rotr!(20),
                    24 => _mm_shuffle_epi8(
                        self.0,
                        _mm_set_epi64x(0x0e0d0c0f_0a09080b, 0x06050407_02010003),
                    ),
                    25 => rotr!(25),
                    _ => unimplemented!("TODO: complete table..."),
                }
            })
        }
    }

    // TODO: avx2 version
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct u32x4x4(u32x4, u32x4, u32x4, u32x4);
    impl u32x4x4 {
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
    impl BitXor for u32x4x4 {
        type Output = u32x4x4;
        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            u32x4x4(
                self.0 ^ rhs.0,
                self.1 ^ rhs.1,
                self.2 ^ rhs.2,
                self.3 ^ rhs.3,
            )
        }
    }
    impl BitOr for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            u32x4x4(
                self.0 | rhs.0,
                self.1 | rhs.1,
                self.2 | rhs.2,
                self.3 | rhs.3,
            )
        }
    }
    impl BitAnd for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            u32x4x4(
                self.0 & rhs.0,
                self.1 & rhs.1,
                self.2 & rhs.2,
                self.3 & rhs.3,
            )
        }
    }
    impl BitXorAssign for u32x4x4 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            self.0 = self.0 ^ rhs.0;
            self.1 = self.1 ^ rhs.1;
            self.2 = self.2 ^ rhs.2;
            self.3 = self.3 ^ rhs.3;
        }
    }
    impl Add for u32x4x4 {
        type Output = Self;
        #[inline(always)]
        fn add(self, rhs: Self) -> Self::Output {
            u32x4x4(
                self.0 + rhs.0,
                self.1 + rhs.1,
                self.0 + rhs.0,
                self.1 + rhs.1,
            )
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

    #[repr(C)]
    #[allow(non_camel_case_types)]
    #[derive(Copy, Clone)]
    pub struct u64x4(__m128i, __m128i);
    impl u64x4 {
        #[inline(always)]
        pub fn new(a: u64, b: u64, c: u64, d: u64) -> Self {
            unsafe {
                u64x4(
                    _mm_set_epi64x(b as i64, a as i64),
                    _mm_set_epi64x(d as i64, c as i64),
                )
            }
        }
        #[inline(always)]
        pub fn from_slice_unaligned(xs: &[u64]) -> Self {
            assert_eq!(xs.len(), 4);
            unsafe {
                u64x4(
                    _mm_loadu_si128(xs.as_ptr() as *const __m128i),
                    _mm_loadu_si128((xs.as_ptr() as *const __m128i).offset(1)),
                )
            }
        }
        #[inline(always)]
        pub fn write_to_slice_unaligned(self, xs: &mut [u64]) {
            assert_eq!(xs.len(), 4);
            unsafe { _mm_storeu_si128(xs.as_mut_ptr() as *mut __m128i, self.0) };
            unsafe { _mm_storeu_si128((xs.as_mut_ptr() as *mut __m128i).offset(1), self.1) };
        }
        #[inline(always)]
        pub fn splat(x: u64) -> Self {
            unsafe { u64x4(_mm_set1_epi64x(x as i64), _mm_set1_epi64x(x as i64)) }
        }
        #[inline(always)]
        pub fn extract(self, i: u32) -> u64 {
            unsafe {
                match i {
                    0 => _mm_extract_epi64(self.0, 0) as u64,
                    1 => _mm_extract_epi64(self.0, 1) as u64,
                    2 => _mm_extract_epi64(self.1, 0) as u64,
                    3 => _mm_extract_epi64(self.1, 1) as u64,
                    _ => unreachable!(),
                }
            }
        }
    }
    impl BitXor for u64x4 {
        type Output = u64x4;
        #[inline(always)]
        fn bitxor(self, rhs: Self) -> Self::Output {
            unsafe { u64x4(_mm_xor_si128(self.0, rhs.0), _mm_xor_si128(self.1, rhs.1)) }
        }
    }
    impl BitOr for u64x4 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            unsafe { u64x4(_mm_or_si128(self.0, rhs.0), _mm_or_si128(self.1, rhs.1)) }
        }
    }
    impl BitAnd for u64x4 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            unsafe { u64x4(_mm_and_si128(self.0, rhs.0), _mm_and_si128(self.1, rhs.1)) }
        }
    }
    impl BitXorAssign for u64x4 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            *self = *self ^ rhs;
        }
    }
    impl AddAssign for u64x4 {
        #[inline(always)]
        fn add_assign(&mut self, rhs: Self) {
            unsafe {
                self.0 = _mm_add_epi64(self.0, rhs.0);
                self.1 = _mm_add_epi64(self.1, rhs.1);
            }
        }
    }
    impl RotateWordsRight for u64x4 {
        type Output = Self;
        #[inline(always)]
        fn rotate_words_right(self, i: u32) -> Self::Output {
            debug_assert_eq!(i & !3, 0);
            unsafe {
                match i & 3 {
                    0 => self,
                    1 => u64x4(
                        _mm_alignr_epi8(self.0, self.1, 8),
                        _mm_alignr_epi8(self.1, self.0, 8),
                    ),
                    2 => u64x4(self.1, self.0),
                    3 => u64x4(
                        _mm_alignr_epi8(self.1, self.0, 8),
                        _mm_alignr_epi8(self.0, self.1, 8),
                    ),
                    _ => unreachable!(),
                }
            }
        }
    }
    impl SplatRotateRight for u64x4 {
        type Output = Self;
        #[inline(always)]
        fn splat_rotate_right(self, i: u32) -> Self::Output {
            macro_rules! rotr {
                ($i:expr) => {
                    u64x4(
                        _mm_or_si128(
                            _mm_srli_epi64(self.0, $i as i32),
                            _mm_slli_epi64(self.0, 64 - $i as i32),
                        ),
                        _mm_or_si128(
                            _mm_srli_epi64(self.1, $i as i32),
                            _mm_slli_epi64(self.1, 64 - $i as i32),
                        ),
                    )
                };
            }
            unsafe {
                let k16 = _mm_set_epi64x(0x09080f0e0d0c0b0a, 0x0100070605040302);
                match i {
                    11 => rotr!(11),
                    16 => u64x4(_mm_shuffle_epi8(self.0, k16), _mm_shuffle_epi8(self.1, k16)),
                    25 => rotr!(25),
                    32 => u64x4(
                        _mm_shuffle_epi32(self.0, 0b10110001),
                        _mm_shuffle_epi32(self.1, 0b10110001),
                    ),
                    _ => unimplemented!("TODO: complete table..."),
                }
            }
        }
    }
}
//#[cfg(all(feature = "simd", target_feature = "sse2"))]
pub use self::sse2::*;

//#[cfg(not(all(feature = "avx2", target_feature = "avx2")))]
mod single_channel {
    use super::*;
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct u128x2(u128x1, u128x1);
    impl u128x2 {
        #[inline(always)]
        pub fn new(a: u128, b: u128) -> Self {
            unsafe { u128x2(core::mem::transmute(a), core::mem::transmute(b)) }
        }
        #[inline(always)]
        pub fn extract(self, i: u32) -> u128 {
            let xs = [self.0, self.1];
            unsafe { core::mem::transmute(xs[i as usize]) }
        }
        #[inline(always)]
        pub fn andnot(self, rhs: Self) -> Self {
            u128x2(self.0.andnot(rhs.0), self.1.andnot(rhs.1))
        }
    }
    impl BitXorAssign for u128x2 {
        #[inline(always)]
        fn bitxor_assign(&mut self, rhs: Self) {
            self.0 = self.0 ^ rhs.0;
            self.1 = self.1 ^ rhs.1;
        }
    }
    impl BitOr for u128x2 {
        type Output = Self;
        #[inline(always)]
        fn bitor(self, rhs: Self) -> Self::Output {
            u128x2(self.0 | rhs.0, self.1 | rhs.1)
        }
    }
    impl BitAnd for u128x2 {
        type Output = Self;
        #[inline(always)]
        fn bitand(self, rhs: Self) -> Self::Output {
            u128x2(self.0 & rhs.0, self.1 & rhs.1)
        }
    }
    impl Not for u128x2 {
        type Output = Self;
        #[inline(always)]
        fn not(self) -> Self::Output {
            u128x2(
                self.0 ^ u128x1::new(0xffffffffffffffffffffffffffffffff),
                self.1 ^ u128x1::new(0xffffffffffffffffffffffffffffffff),
            )
        }
    }
}
#[cfg(not(all(feature = "avx2", target_feature = "avx2")))]
pub use self::single_channel::u128x2;

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
