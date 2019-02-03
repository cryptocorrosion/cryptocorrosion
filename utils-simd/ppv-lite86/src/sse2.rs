use core::arch::x86_64::*;
use core::marker::PhantomData;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
};
use crate::crypto_simd_new::*;
use crate::crypto_simd_new_types::*;
use crate::{vec128_storage, vec256_storage, vec512_storage, NoS3, Store, StoreBytes, YesS3};

macro_rules! impl_binop {
    ($vec:ident, $trait:ident, $fn:ident, $impl_fn:ident) => {
        impl<S3, S4, NI> $trait for $vec<S3, S4, NI> {
            type Output = Self;
            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self::new(unsafe { $impl_fn(self.x, rhs.x) })
            }
        }
    };
}

macro_rules! impl_binop_assign {
    ($vec:ident, $trait:ident, $fn_assign:ident, $fn:ident) => {
        impl<S3, S4, NI> $trait for $vec<S3, S4, NI>
        where
            $vec<S3, S4, NI>: Copy,
        {
            #[inline(always)]
            fn $fn_assign(&mut self, rhs: Self) {
                *self = self.$fn(rhs);
            }
        }
    };
}

macro_rules! def_vec {
    ($vec:ident, $word:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone)]
        pub struct $vec<S3, S4, NI> {
            x: __m128i,
            s3: PhantomData<S3>,
            s4: PhantomData<S4>,
            ni: PhantomData<NI>,
        }

        impl<S3, S4, NI> StoreBytes for $vec<S3, S4, NI> {
            #[inline(always)]
            fn write_le(self, out: &mut [u8]) {
                assert_eq!(out.len(), 16);
                unsafe { _mm_storeu_si128(out as *mut _ as *mut _, self.x) }
            }
        }

        impl<S3, S4, NI> Store<vec128_storage> for $vec<S3, S4, NI> {
            #[inline(always)]
            unsafe fn unpack(x: vec128_storage) -> Self {
                Self::new(x.sse2)
            }
            #[inline(always)]
            fn pack(self) -> vec128_storage {
                vec128_storage { sse2: self.x }
            }
        }
        impl<S3, S4, NI> $vec<S3, S4, NI> {
            fn new(x: __m128i) -> Self {
                $vec {
                    x,
                    s3: PhantomData,
                    s4: PhantomData,
                    ni: PhantomData,
                }
            }
        }

        impl<S3, S4, NI> Default for $vec<S3, S4, NI> {
            #[inline(always)]
            fn default() -> Self {
                Self::new(unsafe { _mm_setzero_si128() })
            }
        }

        impl<S3, S4, NI> Not for $vec<S3, S4, NI> {
            type Output = Self;
            #[inline(always)]
            fn not(self) -> Self::Output {
                unsafe {
                    let ff = _mm_set1_epi64x(-1i64);
                    self ^ Self::new(ff)
                }
            }
        }

        impl<S3: Copy, S4: Copy, NI: Copy> BitOps0 for $vec<S3, S4, NI> {}
        impl_binop!($vec, BitAnd, bitand, _mm_and_si128);
        impl_binop!($vec, BitOr, bitor, _mm_or_si128);
        impl_binop!($vec, BitXor, bitxor, _mm_xor_si128);
        impl_binop_assign!($vec, BitAndAssign, bitand_assign, bitand);
        impl_binop_assign!($vec, BitOrAssign, bitor_assign, bitor);
        impl_binop_assign!($vec, BitXorAssign, bitxor_assign, bitxor);
        impl<S3: Copy, S4: Copy, NI: Copy> AndNot for $vec<S3, S4, NI> {
            type Output = Self;
            #[inline(always)]
            fn andnot(self, rhs: Self) -> Self {
                Self::new(unsafe { _mm_andnot_si128(self.x, rhs.x) })
            }
        }
    };
}

macro_rules! impl_bitops32 {
    ($vec:ident) => {
        impl<S3: Copy, S4: Copy, NI: Copy> BitOps32 for $vec<S3, S4, NI> where
            $vec<S3, S4, NI>: RotateEachWord32
        {}
    };
}

macro_rules! impl_bitops64 {
    ($vec:ident) => {
        impl_bitops32!($vec);
        impl<S3: Copy, S4: Copy, NI: Copy> BitOps64 for $vec<S3, S4, NI> where
            $vec<S3, S4, NI>: RotateEachWord64 + RotateEachWord32
        {}
    };
}

macro_rules! impl_bitops128 {
    ($vec:ident) => {
        impl_bitops64!($vec);
        impl<S3: Copy, S4: Copy, NI: Copy> BitOps128 for $vec<S3, S4, NI> where
            $vec<S3, S4, NI>: RotateEachWord128
        {}
    };
}

macro_rules! rotr_32_s3 {
    ($name:ident, $k0:expr, $k1:expr) => {
    #[inline(always)]
    fn $name(self) -> Self {
        Self::new(unsafe {
                _mm_shuffle_epi8(
                    self.x,
                    _mm_set_epi64x($k0, $k1),
                )
            })
        }
    };
}
macro_rules! rotr_32 {
    ($name:ident, $i:expr) => {
    #[inline(always)]
    fn $name(self) -> Self {
        Self::new(unsafe {
            _mm_or_si128(
                _mm_srli_epi32(self.x, $i as i32),
                _mm_slli_epi32(self.x, 32 - $i as i32),
            )
        })
    }
    };
}
impl<S4: Copy, NI: Copy> RotateEachWord32 for u32x4_sse2<YesS3, S4, NI> {
    rotr_32!(rotate_each_word_right7, 7);
    rotr_32_s3!(
        rotate_each_word_right8,
        0x0c0f0e0d_080b0a09,
        0x04070605_00030201
    );
    rotr_32!(rotate_each_word_right11, 11);
    rotr_32!(rotate_each_word_right12, 12);
    rotr_32_s3!(
        rotate_each_word_right16,
        0x0d0c0f0e_09080b0a,
        0x05040706_01000302
    );
    rotr_32!(rotate_each_word_right20, 20);
    rotr_32_s3!(
        rotate_each_word_right24,
        0x0e0d0c0f_0a09080b,
        0x06050407_02010003
    );
    rotr_32!(rotate_each_word_right25, 25);
}
impl<S4: Copy, NI: Copy> RotateEachWord32 for u32x4_sse2<NoS3, S4, NI> {
    rotr_32!(rotate_each_word_right7, 7);
    rotr_32!(rotate_each_word_right8, 8);
    rotr_32!(rotate_each_word_right11, 11);
    rotr_32!(rotate_each_word_right12, 12);
    // TODO: shufflehi/shufflelo impl
    rotr_32!(rotate_each_word_right16, 16);
    rotr_32!(rotate_each_word_right20, 20);
    rotr_32!(rotate_each_word_right24, 24);
    rotr_32!(rotate_each_word_right25, 25);
}

macro_rules! rotr_64_s3 {
    ($name:ident, $k0:expr, $k1:expr) => {
    #[inline(always)]
    fn $name(self) -> Self {
        Self::new(unsafe {
                _mm_shuffle_epi8(
                    self.x,
                    _mm_set_epi64x($k0, $k1),
                )
            })
        }
    };
}
macro_rules! rotr_64 {
    ($name:ident, $i:expr) => {
    #[inline(always)]
    fn $name(self) -> Self {
        Self::new(unsafe {
            _mm_or_si128(
                _mm_srli_epi64(self.x, $i as i32),
                _mm_slli_epi64(self.x, 64 - $i as i32),
            )
        })
    }
    };
}
impl<S4: Copy, NI: Copy> RotateEachWord32 for u64x2_sse2<YesS3, S4, NI> {
    rotr_64!(rotate_each_word_right7, 7);
    // TODO
    rotr_64!(rotate_each_word_right8, 8);
    rotr_64!(rotate_each_word_right11, 11);
    rotr_64!(rotate_each_word_right12, 12);
    rotr_64_s3!(
        rotate_each_word_right16,
        0x0908_0f0e_0d0c_0b0a,
        0x0100_0706_0504_0302
    );
    rotr_64!(rotate_each_word_right20, 20);
    // TODO
    rotr_64!(rotate_each_word_right24, 24);
    rotr_64!(rotate_each_word_right25, 25);
}
impl<S4: Copy, NI: Copy> RotateEachWord32 for u64x2_sse2<NoS3, S4, NI> {
    rotr_64!(rotate_each_word_right7, 7);
    rotr_64!(rotate_each_word_right8, 8);
    rotr_64!(rotate_each_word_right11, 11);
    rotr_64!(rotate_each_word_right12, 12);
    // TODO: shufflehi/shufflelo impl
    rotr_64!(rotate_each_word_right16, 16);
    rotr_64!(rotate_each_word_right20, 20);
    rotr_64!(rotate_each_word_right24, 24);
    rotr_64!(rotate_each_word_right25, 25);
}
impl<S3: Copy, S4: Copy, NI: Copy> RotateEachWord64 for u64x2_sse2<S3, S4, NI> {
    #[inline(always)]
    fn rotate_each_word_right32(self) -> Self {
        Self::new(unsafe { _mm_shuffle_epi32(self.x, 0b10110001) })
    }
}

macro_rules! rotr_128 {
    ($name:ident, $i:expr) => {
    #[inline(always)]
    fn $name(self) -> Self {
        Self::new(unsafe {
            _mm_or_si128(
                _mm_srli_si128(self.x, $i as i32),
                _mm_slli_si128(self.x, 128 - $i as i32),
            )
        })
    }
    };
}
// TODO: completely unoptimized
impl<S3: Copy, S4: Copy, NI: Copy> RotateEachWord32 for u128x1_sse2<S3, S4, NI> {
    rotr_128!(rotate_each_word_right7, 7);
    rotr_128!(rotate_each_word_right8, 8);
    rotr_128!(rotate_each_word_right11, 11);
    rotr_128!(rotate_each_word_right12, 12);
    rotr_128!(rotate_each_word_right16, 16);
    rotr_128!(rotate_each_word_right20, 20);
    rotr_128!(rotate_each_word_right24, 24);
    rotr_128!(rotate_each_word_right25, 25);
}
// TODO: completely unoptimized
impl<S3: Copy, S4: Copy, NI: Copy> RotateEachWord64 for u128x1_sse2<S3, S4, NI> {
    rotr_128!(rotate_each_word_right32, 32);
}
impl<S3: Copy, S4: Copy, NI: Copy> RotateEachWord128 for u128x1_sse2<S3, S4, NI> {}

def_vec!(u32x4_sse2, u32);
def_vec!(u64x2_sse2, u64);
def_vec!(u128x1_sse2, u128);

impl_bitops32!(u32x4_sse2);
impl_bitops64!(u64x2_sse2);
impl_bitops128!(u128x1_sse2);

impl<S3: Copy, S4: Copy, NI: Copy> ArithOps for u32x4_sse2<S3, S4, NI> {}
impl<S3: Copy, S4: Copy, NI: Copy> ArithOps for u64x2_sse2<S3, S4, NI> {}
impl_binop!(u32x4_sse2, Add, add, _mm_add_epi32);
impl_binop!(u64x2_sse2, Add, add, _mm_add_epi64);
impl_binop_assign!(u32x4_sse2, AddAssign, add_assign, add);
impl_binop_assign!(u64x2_sse2, AddAssign, add_assign, add);

impl<S3: Copy, S4: Copy, NI: Copy> u32x4 for u32x4_sse2<S3, S4, NI> where
    u32x4_sse2<S3, S4, NI>: RotateEachWord32
{}
impl<S3: Copy, S4: Copy, NI: Copy> u64x2 for u64x2_sse2<S3, S4, NI> where
    u64x2_sse2<S3, S4, NI>: RotateEachWord64 + RotateEachWord32
{}
impl<S3: Copy, S4: Copy, NI: Copy> u128x1 for u128x1_sse2<S3, S4, NI> where
    u128x1_sse2<S3, S4, NI>: Swap64 + RotateEachWord64 + RotateEachWord32
{}

impl<S3, S4, NI> UnsafeFrom<[u32; 4]> for u32x4_sse2<S3, S4, NI> {
    #[inline(always)]
    unsafe fn unsafe_from(xs: [u32; 4]) -> Self {
        Self::new(_mm_set_epi32(
            xs[3] as i32,
            xs[2] as i32,
            xs[1] as i32,
            xs[0] as i32,
        ))
    }
}

impl<S3, S4, NI> Vec4<u32> for u32x4_sse2<S3, S4, NI> {
    #[inline(always)]
    fn extract(self, i: u32) -> u32 {
        unsafe {
            match i {
                0 => _mm_extract_epi32(self.x, 0) as u32,
                1 => _mm_extract_epi32(self.x, 1) as u32,
                2 => _mm_extract_epi32(self.x, 2) as u32,
                3 => _mm_extract_epi32(self.x, 3) as u32,
                _ => unreachable!(),
            }
        }
    }
    #[inline(always)]
    fn insert(self, v: u32, i: u32) -> Self {
        Self::new(unsafe {
            match i {
                0 => _mm_insert_epi32(self.x, v as i32, 0),
                1 => _mm_insert_epi32(self.x, v as i32, 1),
                2 => _mm_insert_epi32(self.x, v as i32, 2),
                3 => _mm_insert_epi32(self.x, v as i32, 3),
                _ => unreachable!(),
            }
        })
    }
}

impl<S3, S4, NI> LaneWords4 for u32x4_sse2<S3, S4, NI> {
    #[inline(always)]
    fn shuffle_lane_words2301(self) -> Self {
        self.shuffle2301()
    }
    #[inline(always)]
    fn shuffle_lane_words1230(self) -> Self {
        self.shuffle1230()
    }
    #[inline(always)]
    fn shuffle_lane_words3012(self) -> Self {
        self.shuffle3012()
    }
}

impl<S3, S4, NI> Words4 for u32x4_sse2<S3, S4, NI> {
    #[inline(always)]
    fn shuffle2301(self) -> Self {
        Self::new(unsafe { _mm_shuffle_epi32(self.x, 0b0100_1110) })
    }
    #[inline(always)]
    fn shuffle1230(self) -> Self {
        Self::new(unsafe { _mm_shuffle_epi32(self.x, 0b1001_0011) })
    }
    #[inline(always)]
    fn shuffle3012(self) -> Self {
        Self::new(unsafe { _mm_shuffle_epi32(self.x, 0b0011_1001) })
    }
}

impl<S3, S4, NI> Words4 for u64x2x2_sse2<S3, S4, NI> {
    #[inline(always)]
    fn shuffle2301(self) -> Self {
        x2([u64x2_sse2::new(self.0[1].x), u64x2_sse2::new(self.0[0].x)])
    }
    #[inline(always)]
    fn shuffle3012(self) -> Self {
        unsafe {
            x2([
                u64x2_sse2::new(_mm_alignr_epi8(self.0[1].x, self.0[0].x, 8)),
                u64x2_sse2::new(_mm_alignr_epi8(self.0[0].x, self.0[1].x, 8)),
            ])
        }
    }
    #[inline(always)]
    fn shuffle1230(self) -> Self {
        unsafe {
            x2([
                u64x2_sse2::new(_mm_alignr_epi8(self.0[0].x, self.0[1].x, 8)),
                u64x2_sse2::new(_mm_alignr_epi8(self.0[1].x, self.0[0].x, 8)),
            ])
        }
    }
}

impl<S3, S4, NI> UnsafeFrom<[u64; 2]> for u64x2_sse2<S3, S4, NI> {
    #[inline(always)]
    unsafe fn unsafe_from(xs: [u64; 2]) -> Self {
        Self::new(_mm_set_epi64x(xs[1] as i64, xs[0] as i64))
    }
}

impl<S3, S4, NI> Vec2<u64> for u64x2_sse2<S3, S4, NI> {
    #[inline(always)]
    fn extract(self, i: u32) -> u64 {
        unsafe {
            match i {
                0 => _mm_extract_epi64(self.x, 0) as u64,
                1 => _mm_extract_epi64(self.x, 1) as u64,
                _ => unreachable!(),
            }
        }
    }
    #[inline(always)]
    fn insert(self, x: u64, i: u32) -> Self {
        Self::new(unsafe {
            match i {
                0 => _mm_insert_epi64(self.x, x as i64, 0),
                1 => _mm_insert_epi64(self.x, x as i64, 1),
                _ => unreachable!(),
            }
        })
    }
}

macro_rules! swapi {
    ($x:expr, $i:expr, $k:expr) => {
        unsafe {
            const K: u8 = $k;
            let k = _mm_set1_epi8(K as i8);
            u128x1_sse2::new(_mm_or_si128(
                _mm_srli_epi16(_mm_and_si128($x.x, k), $i),
                _mm_and_si128(_mm_slli_epi16($x.x, $i), k),
            ))
        }
    };
}
impl<S4, NI> Swap64 for u128x1_sse2<YesS3, S4, NI> {
    #[inline(always)]
    fn swap1(self) -> Self {
        swapi!(self, 1, 0xaa)
    }
    #[inline(always)]
    fn swap2(self) -> Self {
        swapi!(self, 2, 0xcc)
    }
    #[inline(always)]
    fn swap4(self) -> Self {
        swapi!(self, 4, 0xf0)
    }
    #[inline(always)]
    fn swap8(self) -> Self {
        u128x1_sse2::new(unsafe {
            let k = _mm_set_epi64x(0x0e0f_0c0d_0a0b_0809, 0x0607_0405_0203_0001);
            _mm_shuffle_epi8(self.x, k)
        })
    }
    #[inline(always)]
    fn swap16(self) -> Self {
        u128x1_sse2::new(unsafe {
            let k = _mm_set_epi64x(0x0d0c_0f0e_0908_0b0a, 0x0504_0706_0100_0302);
            _mm_shuffle_epi8(self.x, k)
        })
    }
    #[inline(always)]
    fn swap32(self) -> Self {
        u128x1_sse2::new(unsafe { _mm_shuffle_epi32(self.x, 0b1011_0001) })
    }
    #[inline(always)]
    fn swap64(self) -> Self {
        u128x1_sse2::new(unsafe { _mm_shuffle_epi32(self.x, 0b0100_1110) })
    }
}
impl<S4, NI> Swap64 for u128x1_sse2<NoS3, S4, NI> {
    #[inline(always)]
    fn swap1(self) -> Self {
        swapi!(self, 1, 0xaa)
    }
    #[inline(always)]
    fn swap2(self) -> Self {
        swapi!(self, 2, 0xcc)
    }
    #[inline(always)]
    fn swap4(self) -> Self {
        swapi!(self, 4, 0xf0)
    }
    #[inline(always)]
    fn swap8(self) -> Self {
        u128x1_sse2::new(unsafe {
            _mm_or_si128(_mm_slli_epi16(self.x, 8), _mm_srli_epi16(self.x, 8))
        })
    }
    #[inline(always)]
    fn swap16(self) -> Self {
        u128x1_sse2::new(unsafe {
            _mm_shufflehi_epi16(_mm_shufflelo_epi16(self.x, 0b1011_0001), 0b1011_0001)
        })
    }
    #[inline(always)]
    fn swap32(self) -> Self {
        u128x1_sse2::new(unsafe { _mm_shuffle_epi32(self.x, 0b1011_0001) })
    }
    #[inline(always)]
    fn swap64(self) -> Self {
        u128x1_sse2::new(unsafe { _mm_shuffle_epi32(self.x, 0b0100_1110) })
    }
}

#[derive(Copy, Clone, Default)]
#[allow(non_camel_case_types)]
pub struct x2<W>([W; 2]);
macro_rules! fwd_binop_x2 {
    ($trait:ident, $fn:ident) => {
        impl<W: $trait + Copy> $trait for x2<W> {
            type Output = x2<W::Output>;
            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                x2([self.0[0].$fn(rhs.0[0]), self.0[1].$fn(rhs.0[1])])
            }
        }
    };
}
macro_rules! fwd_binop_assign_x2 {
    ($trait:ident, $fn_assign:ident) => {
        impl<W: $trait + Copy> $trait for x2<W> {
            #[inline(always)]
            fn $fn_assign(&mut self, rhs: Self) {
                (self.0[0]).$fn_assign(rhs.0[0]);
                (self.0[1]).$fn_assign(rhs.0[1]);
            }
        }
    };
}
macro_rules! fwd_unop_x2 {
    ($fn:ident) => {
        #[inline(always)]
        fn $fn(self) -> Self {
            x2([self.0[0].$fn(), self.0[1].$fn()])
        }
    };
}
impl<W> RotateEachWord32 for x2<W>
where
    W: Copy + RotateEachWord32,
{
    fwd_unop_x2!(rotate_each_word_right7);
    fwd_unop_x2!(rotate_each_word_right8);
    fwd_unop_x2!(rotate_each_word_right11);
    fwd_unop_x2!(rotate_each_word_right12);
    fwd_unop_x2!(rotate_each_word_right16);
    fwd_unop_x2!(rotate_each_word_right20);
    fwd_unop_x2!(rotate_each_word_right24);
    fwd_unop_x2!(rotate_each_word_right25);
}
impl<W> RotateEachWord64 for x2<W>
where
    W: Copy + RotateEachWord64,
{
    fwd_unop_x2!(rotate_each_word_right32);
}
impl<W> RotateEachWord128 for x2<W> where W: RotateEachWord128 {}
impl<W> BitOps0 for x2<W> where W: BitOps0 {}
impl<W> BitOps32 for x2<W> where W: BitOps32 + BitOps0 {}
impl<W> BitOps64 for x2<W> where W: BitOps64 + BitOps0 {}
impl<W> BitOps128 for x2<W> where W: BitOps128 + BitOps0 {}
fwd_binop_x2!(BitAnd, bitand);
fwd_binop_x2!(BitOr, bitor);
fwd_binop_x2!(BitXor, bitxor);
fwd_binop_x2!(AndNot, andnot);
fwd_binop_assign_x2!(BitAndAssign, bitand_assign);
fwd_binop_assign_x2!(BitOrAssign, bitor_assign);
fwd_binop_assign_x2!(BitXorAssign, bitxor_assign);
impl<W> ArithOps for x2<W> where W: ArithOps {}
fwd_binop_x2!(Add, add);
fwd_binop_assign_x2!(AddAssign, add_assign);
impl<W: Not + Copy> Not for x2<W> {
    type Output = x2<W::Output>;
    #[inline(always)]
    fn not(self) -> Self::Output {
        x2([self.0[0].not(), self.0[1].not()])
    }
}
impl<W> UnsafeFrom<[W; 2]> for x2<W> {
    #[inline(always)]
    unsafe fn unsafe_from(xs: [W; 2]) -> Self {
        x2(xs)
    }
}
impl<W: Copy> Vec2<W> for x2<W> {
    #[inline(always)]
    fn extract(self, i: u32) -> W {
        self.0[i as usize]
    }
    #[inline(always)]
    fn insert(mut self, w: W, i: u32) -> Self {
        self.0[i as usize] = w;
        self
    }
}
impl<W: Copy + Store<vec128_storage>> Store<vec256_storage> for x2<W> {
    #[inline(always)]
    unsafe fn unpack(p: vec256_storage) -> Self {
        x2([W::unpack(p.sse2[0]), W::unpack(p.sse2[1])])
    }
    #[inline(always)]
    fn pack(self) -> vec256_storage {
        vec256_storage {
            sse2: [self.0[0].pack(), self.0[1].pack()],
        }
    }
}
impl<W> Swap64 for x2<W>
where
    W: Swap64 + Copy,
{
    fwd_unop_x2!(swap1);
    fwd_unop_x2!(swap2);
    fwd_unop_x2!(swap4);
    fwd_unop_x2!(swap8);
    fwd_unop_x2!(swap16);
    fwd_unop_x2!(swap32);
    fwd_unop_x2!(swap64);
}
impl<W: Copy> MultiLane<[W; 2]> for x2<W> {
    fn to_lanes(self) -> [W; 2] {
        self.0
    }
    fn from_lanes(lanes: [W; 2]) -> Self {
        x2(lanes)
    }
}

#[derive(Copy, Clone, Default)]
#[allow(non_camel_case_types)]
pub struct x4<W>([W; 4]);
macro_rules! fwd_binop_x4 {
    ($trait:ident, $fn:ident) => {
        impl<W: $trait + Copy> $trait for x4<W> {
            type Output = x4<W::Output>;
            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                x4([
                    self.0[0].$fn(rhs.0[0]),
                    self.0[1].$fn(rhs.0[1]),
                    self.0[2].$fn(rhs.0[2]),
                    self.0[3].$fn(rhs.0[3]),
                ])
            }
        }
    };
}
macro_rules! fwd_binop_assign_x4 {
    ($trait:ident, $fn_assign:ident) => {
        impl<W: $trait + Copy> $trait for x4<W> {
            #[inline(always)]
            fn $fn_assign(&mut self, rhs: Self) {
                self.0[0].$fn_assign(rhs.0[0]);
                self.0[1].$fn_assign(rhs.0[1]);
                self.0[2].$fn_assign(rhs.0[2]);
                self.0[3].$fn_assign(rhs.0[3]);
            }
        }
    };
}
macro_rules! fwd_unop_x4 {
    ($fn:ident) => {
        #[inline(always)]
        fn $fn(self) -> Self {
            x4([self.0[0].$fn(), self.0[1].$fn(), self.0[2].$fn(), self.0[3].$fn()])
        }
    };
}
impl<W> RotateEachWord32 for x4<W>
where
    W: Copy + RotateEachWord32,
{
    fwd_unop_x4!(rotate_each_word_right7);
    fwd_unop_x4!(rotate_each_word_right8);
    fwd_unop_x4!(rotate_each_word_right11);
    fwd_unop_x4!(rotate_each_word_right12);
    fwd_unop_x4!(rotate_each_word_right16);
    fwd_unop_x4!(rotate_each_word_right20);
    fwd_unop_x4!(rotate_each_word_right24);
    fwd_unop_x4!(rotate_each_word_right25);
}
impl<W> RotateEachWord64 for x4<W>
where
    W: Copy + RotateEachWord64,
{
    fwd_unop_x4!(rotate_each_word_right32);
}
impl<W> RotateEachWord128 for x4<W> where W: RotateEachWord128 {}
impl<W> BitOps0 for x4<W> where W: BitOps0 {}
impl<W> BitOps32 for x4<W> where W: BitOps32 + BitOps0 {}
impl<W> BitOps64 for x4<W> where W: BitOps64 + BitOps0 {}
impl<W> BitOps128 for x4<W> where W: BitOps128 + BitOps0 {}
fwd_binop_x4!(BitAnd, bitand);
fwd_binop_x4!(BitOr, bitor);
fwd_binop_x4!(BitXor, bitxor);
fwd_binop_x4!(AndNot, andnot);
fwd_binop_assign_x4!(BitAndAssign, bitand_assign);
fwd_binop_assign_x4!(BitOrAssign, bitor_assign);
fwd_binop_assign_x4!(BitXorAssign, bitxor_assign);
impl<W> ArithOps for x4<W> where W: ArithOps {}
fwd_binop_x4!(Add, add);
fwd_binop_assign_x4!(AddAssign, add_assign);
impl<W: Not + Copy> Not for x4<W> {
    type Output = x4<W::Output>;
    #[inline(always)]
    fn not(self) -> Self::Output {
        x4([
            self.0[0].not(),
            self.0[1].not(),
            self.0[2].not(),
            self.0[3].not(),
        ])
    }
}
impl<W> UnsafeFrom<[W; 4]> for x4<W> {
    #[inline(always)]
    unsafe fn unsafe_from(xs: [W; 4]) -> Self {
        x4(xs)
    }
}
impl<W: Copy> Vec4<W> for x4<W> {
    #[inline(always)]
    fn extract(self, i: u32) -> W {
        self.0[i as usize]
    }
    #[inline(always)]
    fn insert(mut self, w: W, i: u32) -> Self {
        self.0[i as usize] = w;
        self
    }
}
impl<W: Copy + Store<vec128_storage>> Store<vec512_storage> for x4<W> {
    #[inline(always)]
    unsafe fn unpack(p: vec512_storage) -> Self {
        x4([
            W::unpack(p.sse2[0]),
            W::unpack(p.sse2[1]),
            W::unpack(p.sse2[2]),
            W::unpack(p.sse2[3]),
        ])
    }
    #[inline(always)]
    fn pack(self) -> vec512_storage {
        vec512_storage {
            sse2: [
                self.0[0].pack(),
                self.0[1].pack(),
                self.0[2].pack(),
                self.0[3].pack(),
            ],
        }
    }
}
impl<W> Swap64 for x4<W>
where
    W: Swap64 + Copy,
{
    fwd_unop_x4!(swap1);
    fwd_unop_x4!(swap2);
    fwd_unop_x4!(swap4);
    fwd_unop_x4!(swap8);
    fwd_unop_x4!(swap16);
    fwd_unop_x4!(swap32);
    fwd_unop_x4!(swap64);
}
impl<W: Copy> MultiLane<[W; 4]> for x4<W> {
    fn to_lanes(self) -> [W; 4] {
        self.0
    }
    fn from_lanes(lanes: [W; 4]) -> Self {
        x4(lanes)
    }
}
impl<W: Copy + LaneWords4> LaneWords4 for x4<W> {
    #[inline(always)]
    fn shuffle_lane_words2301(self) -> Self {
        x4([
            self.0[0].shuffle_lane_words2301(),
            self.0[1].shuffle_lane_words2301(),
            self.0[2].shuffle_lane_words2301(),
            self.0[3].shuffle_lane_words2301(),
        ])
    }
    #[inline(always)]
    fn shuffle_lane_words1230(self) -> Self {
        x4([
            self.0[0].shuffle_lane_words1230(),
            self.0[1].shuffle_lane_words1230(),
            self.0[2].shuffle_lane_words1230(),
            self.0[3].shuffle_lane_words1230(),
        ])
    }
    #[inline(always)]
    fn shuffle_lane_words3012(self) -> Self {
        x4([
            self.0[0].shuffle_lane_words3012(),
            self.0[1].shuffle_lane_words3012(),
            self.0[2].shuffle_lane_words3012(),
            self.0[3].shuffle_lane_words3012(),
        ])
    }
}

#[allow(non_camel_case_types)]
pub type u32x4x2_sse2<S3, S4, NI> = x2<u32x4_sse2<S3, S4, NI>>;
#[allow(non_camel_case_types)]
pub type u64x2x2_sse2<S3, S4, NI> = x2<u64x2_sse2<S3, S4, NI>>;
#[allow(non_camel_case_types)]
pub type u128x2_sse2<S3, S4, NI> = x2<u128x1_sse2<S3, S4, NI>>;

#[allow(non_camel_case_types)]
pub type u32x4x4_sse2<S3, S4, NI> = x4<u32x4_sse2<S3, S4, NI>>;
#[allow(non_camel_case_types)]
pub type u64x2x4_sse2<S3, S4, NI> = x4<u64x2_sse2<S3, S4, NI>>;
#[allow(non_camel_case_types)]
pub type u128x4_sse2<S3, S4, NI> = x4<u128x1_sse2<S3, S4, NI>>;

impl<S3: Copy, S4: Copy, NI: Copy> u32x4x2<u32x4_sse2<S3, S4, NI>> for u32x4x2_sse2<S3, S4, NI> where
    u32x4_sse2<S3, S4, NI>: RotateEachWord32
{}
impl<S3: Copy, S4: Copy, NI: Copy> u64x2x2<u64x2_sse2<S3, S4, NI>> for u64x2x2_sse2<S3, S4, NI> where
    u64x2_sse2<S3, S4, NI>: RotateEachWord64 + RotateEachWord32
{}
impl<S3: Copy, S4: Copy, NI: Copy> u128x2<u128x1_sse2<S3, S4, NI>> for u128x2_sse2<S3, S4, NI> where
    u128x1_sse2<S3, S4, NI>: Swap64
{}

impl<S3: Copy, S4: Copy, NI: Copy> u32x4x4<u32x4_sse2<S3, S4, NI>> for u32x4x4_sse2<S3, S4, NI> where
    u32x4_sse2<S3, S4, NI>: RotateEachWord32
{}
impl<S3: Copy, S4: Copy, NI: Copy> u64x2x4<u64x2_sse2<S3, S4, NI>> for u64x2x4_sse2<S3, S4, NI> where
    u64x2_sse2<S3, S4, NI>: RotateEachWord64 + RotateEachWord32
{}
impl<S3: Copy, S4: Copy, NI: Copy> u128x4<u128x1_sse2<S3, S4, NI>> for u128x4_sse2<S3, S4, NI> where
    u128x1_sse2<S3, S4, NI>: Swap64
{}
