//#![no_std]

//use crypto_simd::*;

// Design:
// - safety: safe creation of any machine type is done only by instance methods of a
//   Machine (which is a ZST + Copy type), which can only by created unsafely or safely
//   through feature detection (e.g. fn AVX2::try_get() -> Option<Machine>).

use std::arch::x86_64::{__m128i, __m256i};

mod avx;
mod sse2;

// crate minimums: sse2, x86_64

pub mod crypto_simd_new {
    use core::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, BitXorAssign, Not};

    pub trait AndNot {
        type Output;
        fn andnot(self, rhs: Self) -> Self::Output;
    }
    pub trait BSwap {
        fn bswap(self) -> Self;
    }
    /// Ops that depend on word size
    pub trait ArithOps: Add<Output = Self> + AddAssign + Sized + Copy + Clone + BSwap {}
    /// Ops that are independent of word size and endian
    pub trait BitOps0:
        BitAnd<Output = Self>
        + BitOr<Output = Self>
        + BitXor<Output = Self>
        + BitXorAssign
        + Not<Output = Self>
        + AndNot<Output = Self>
        + Sized
        + Copy
        + Clone
    {
}

    pub trait BitOps32: BitOps0 + RotateEachWord32 {}
    pub trait BitOps64: BitOps32 + RotateEachWord64 {}
    pub trait BitOps128: BitOps64 + RotateEachWord128 {}

    pub trait RotateEachWord32 {
        fn rotate_each_word_right7(self) -> Self;
        fn rotate_each_word_right8(self) -> Self;
        fn rotate_each_word_right11(self) -> Self;
        fn rotate_each_word_right12(self) -> Self;
        fn rotate_each_word_right16(self) -> Self;
        fn rotate_each_word_right20(self) -> Self;
        fn rotate_each_word_right24(self) -> Self;
        fn rotate_each_word_right25(self) -> Self;
    }

    pub trait RotateEachWord64 {
        fn rotate_each_word_right32(self) -> Self;
    }

    pub trait RotateEachWord128 {}
}
pub use crate::crypto_simd_new::{ArithOps, BSwap, BitOps128, BitOps32, BitOps64};

#[allow(non_camel_case_types)]
pub mod crypto_simd_new_types {
    //! Vector type naming scheme:
    //! uN[xP]xL
    //! Unsigned; N-bit words * P bits per lane * L lanes
    //!
    //! A lane is always 128-bits, chosen because common SIMD architectures treat 128-bit units of
    //! wide vectors specially (supporting e.g. intra-lane shuffles), and tend to have limited and
    //! slow inter-lane operations.

    use crate::{
        vec128_storage, vec256_storage, vec512_storage, ArithOps, BitOps128, BitOps32, BitOps64,
        Machine, Store, StoreBytes,
    };

    pub trait UnsafeFrom<T> {
        unsafe fn unsafe_from(t: T) -> Self;
    }

    /// A vector composed of two elements, which may be words or themselves vectors.
    pub trait Vec2<W> {
        fn extract(self, i: u32) -> W;
        fn insert(self, w: W, i: u32) -> Self;
    }

    /// A vector composed of four elements, which may be words or themselves vectors.
    pub trait Vec4<W> {
        fn extract(self, i: u32) -> W;
        fn insert(self, w: W, i: u32) -> Self;
    }

    // TODO: multiples of 4 should inherit this
    /// A vector composed of four words; depending on their size, operations may cross lanes.
    pub trait Words4 {
        fn shuffle1230(self) -> Self;
        fn shuffle2301(self) -> Self;
        fn shuffle3012(self) -> Self;
    }

    /// A vector composed one or more lanes each composed of four words.
    pub trait LaneWords4 {
        fn shuffle_lane_words1230(self) -> Self;
        fn shuffle_lane_words2301(self) -> Self;
        fn shuffle_lane_words3012(self) -> Self;
    }

    // TODO: make this a part of BitOps
    /// Exchange neigboring ranges of bits of the specified size
    pub trait Swap64 {
        fn swap1(self) -> Self;
        fn swap2(self) -> Self;
        fn swap4(self) -> Self;
        fn swap8(self) -> Self;
        fn swap16(self) -> Self;
        fn swap32(self) -> Self;
        fn swap64(self) -> Self;
    }

    pub trait u32x4<M: Machine>:
        BitOps32
        + Store<vec128_storage>
        + ArithOps
        + Vec4<u32>
        + Words4
        + LaneWords4
        + StoreBytes
        + MultiLane<[u32; 4]>
        + Into<vec128_storage>
    {
}
    pub trait u64x2<M: Machine>:
        BitOps64
        + Store<vec128_storage>
        + ArithOps
        + Vec2<u64>
        + MultiLane<[u64; 2]>
        + Into<vec128_storage>
    {
}
    pub trait u128x1<M: Machine>:
        BitOps128
        + Store<vec128_storage>
        + Swap64
        + Into<M::u32x4>
        + Into<M::u64x2>
        + MultiLane<[u128; 1]>
        + Into<vec128_storage>
    {
}

    pub trait u32x4x2<M: Machine>:
        BitOps32
        + Store<vec256_storage>
        + Vec2<M::u32x4>
        + MultiLane<[M::u32x4; 2]>
        + ArithOps
        + Into<vec256_storage>
    {
}
    pub trait u64x2x2<M: Machine>:
        BitOps64
        + Store<vec256_storage>
        + Vec2<M::u64x2>
        + MultiLane<[M::u64x2; 2]>
        + ArithOps
        + StoreBytes
        + Into<vec256_storage>
    {
}
    pub trait u64x4<M: Machine>:
        BitOps64
        + Store<vec256_storage>
        + Vec4<u64>
        + MultiLane<[u64; 4]>
        + ArithOps
        + Words4
        + StoreBytes
        + Into<vec256_storage>
    {
}
    pub trait u128x2<M: Machine>:
        BitOps128
        + Store<vec256_storage>
        + Vec2<M::u128x1>
        + MultiLane<[M::u128x1; 2]>
        + Swap64
        + Into<M::u32x4x2>
        + Into<M::u64x2x2>
        + Into<M::u64x4>
        + Into<vec256_storage>
    {
}

    pub trait u32x4x4<M: Machine>:
        BitOps32
        + Store<vec512_storage>
        + Vec4<M::u32x4>
        + MultiLane<[M::u32x4; 4]>
        + ArithOps
        + LaneWords4
        + Into<vec512_storage>
    {
}
    pub trait u64x2x4<M: Machine>:
        BitOps64
        + Store<vec512_storage>
        + Vec4<M::u64x2>
        + MultiLane<[M::u64x2; 4]>
        + ArithOps
        + Into<vec512_storage>
    {
}
    // TODO: Words4
    pub trait u128x4<M: Machine>:
        BitOps128
        + Store<vec512_storage>
        + Vec4<M::u128x1>
        + MultiLane<[M::u128x1; 4]>
        + Swap64
        + Into<M::u32x4x4>
        + Into<M::u64x2x4>
        + Into<vec512_storage>
    {
}

    /// A vector composed of multiple 128-bit lanes.
    pub trait MultiLane<Lanes> {
        /// Split a multi-lane vector into single-lane vectors.
        fn to_lanes(self) -> Lanes;
        /// Build a multi-lane vector from individual lanes.
        fn from_lanes(lanes: Lanes) -> Self;
    }

    /// Combine single vectors into a multi-lane vector.
    pub trait VZip<V> {
        fn vzip(self) -> V;
    }

    impl<V, T> VZip<V> for T
    where
        V: MultiLane<T>,
    {
        #[inline(always)]
        fn vzip(self) -> V {
            V::from_lanes(self)
        }
    }
}
pub use crate::crypto_simd_new_types::*;

pub(crate) mod features {
    #[derive(Copy, Clone)]
    pub struct YesS3;
    #[derive(Copy, Clone)]
    pub struct NoS3;

    #[derive(Copy, Clone)]
    pub struct YesS4;
    #[derive(Copy, Clone)]
    pub struct NoS4;

    #[derive(Copy, Clone)]
    pub struct YesA1;
    #[derive(Copy, Clone)]
    pub struct NoA1;

    #[derive(Copy, Clone)]
    pub struct YesA2;
    #[derive(Copy, Clone)]
    pub struct NoA2;

    #[derive(Copy, Clone)]
    pub struct YesNI;
    #[derive(Copy, Clone)]
    pub struct NoNI;
}
pub(crate) use crate::features::*;

pub trait Machine: Sized + Copy {
    type u32x4: u32x4<Self>;
    type u64x2: u64x2<Self>;
    type u128x1: u128x1<Self>;

    type u32x4x2: u32x4x2<Self>;
    type u64x2x2: u64x2x2<Self>;
    type u64x4: u64x4<Self>;
    type u128x2: u128x2<Self>;

    type u32x4x4: u32x4x4<Self>;
    type u64x2x4: u64x2x4<Self>;
    type u128x4: u128x4<Self>;

    #[inline(always)]
    fn unpack<S, V: Store<S>>(self, s: S) -> V {
        unsafe { V::unpack(s) }
    }

    #[inline(always)]
    fn vec<V, A>(self, a: A) -> V
    where
        V: MultiLane<A>,
    {
        V::from_lanes(a)
    }

    #[inline(always)]
    fn read_le<V>(self, input: &[u8]) -> V
    where
        V: StoreBytes,
    {
        unsafe { V::unsafe_read_le(input) }
    }

    #[inline(always)]
    fn read_be<V>(self, input: &[u8]) -> V
    where
        V: StoreBytes,
    {
        unsafe { V::unsafe_read_be(input) }
    }

    unsafe fn instance() -> Self;
}

pub mod machine {
    pub mod x86 {
        use core::marker::PhantomData;
        use crate::crypto_simd_new::*;
        use crate::*;

        #[derive(Copy, Clone)]
        pub struct Machine86<S3, S4, NI>(PhantomData<(S3, S4, NI)>);
        impl<S3: Copy, S4: Copy, NI: Copy> Machine for Machine86<S3, S4, NI>
        where
            sse2::u128x1_sse2<S3, S4, NI>: Swap64,
            sse2::u64x2_sse2<S3, S4, NI>:
                BSwap + RotateEachWord32 + MultiLane<[u64; 2]> + Vec2<u64>,
            sse2::u32x4_sse2<S3, S4, NI>:
                BSwap + RotateEachWord32 + MultiLane<[u32; 4]> + Vec4<u32>,
            sse2::u64x4_sse2<S3, S4, NI>: BSwap + Words4,
            sse2::u128x1_sse2<S3, S4, NI>: BSwap,
            sse2::u128x2_sse2<S3, S4, NI>: Into<sse2::u64x2x2_sse2<S3, S4, NI>>,
            sse2::u128x2_sse2<S3, S4, NI>: Into<sse2::u64x4_sse2<S3, S4, NI>>,
            sse2::u128x2_sse2<S3, S4, NI>: Into<sse2::u32x4x2_sse2<S3, S4, NI>>,
            sse2::u128x4_sse2<S3, S4, NI>: Into<sse2::u64x2x4_sse2<S3, S4, NI>>,
            sse2::u128x4_sse2<S3, S4, NI>: Into<sse2::u32x4x4_sse2<S3, S4, NI>>,
        {
            type u32x4 = sse2::u32x4_sse2<S3, S4, NI>;
            type u64x2 = sse2::u64x2_sse2<S3, S4, NI>;
            type u128x1 = sse2::u128x1_sse2<S3, S4, NI>;

            type u32x4x2 = sse2::u32x4x2_sse2<S3, S4, NI>;
            type u64x2x2 = sse2::u64x2x2_sse2<S3, S4, NI>;
            type u64x4 = sse2::u64x4_sse2<S3, S4, NI>;
            type u128x2 = sse2::u128x2_sse2<S3, S4, NI>;

            type u32x4x4 = sse2::u32x4x4_sse2<S3, S4, NI>;
            type u64x2x4 = sse2::u64x2x4_sse2<S3, S4, NI>;
            type u128x4 = sse2::u128x4_sse2<S3, S4, NI>;

            #[inline(always)]
            unsafe fn instance() -> Self {
                Machine86(PhantomData)
            }
        }

        pub type SSE2 = Machine86<NoS3, NoS4, NoNI>;
        pub type SSSE3 = Machine86<YesS3, NoS4, NoNI>;
        pub type SSE41 = Machine86<YesS3, YesS4, NoNI>;
        /// AVX but not AVX2: only 128-bit integer operations, but use VEX versions of everything
        /// to avoid expensive SSE/VEX conflicts.
        pub type AVX = Machine86<YesS3, YesS4, NoNI>;
    }
}

/// Generic wrapper for unparameterized storage of any of the possible impls.
/// Converting into and out of this type should be essentially free, although it may be more
/// aligned than a particular impl requires.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub union vec128_storage {
    u32x4: [u32; 4],
    u64x2: [u64; 2],
    u128x1: [u128; 1],
    sse2: __m128i,
}
macro_rules! impl_into {
    ($storage:ident, $array:ty, $name:ident) => {
        impl Into<$array> for $storage {
            #[inline(always)]
            fn into(self) -> $array {
                unsafe { self.$name }
            }
        }
    };
}
impl_into!(vec128_storage, [u32; 4], u32x4);
impl_into!(vec128_storage, [u64; 2], u64x2);
impl_into!(vec128_storage, [u128; 1], u128x1);
impl Store<vec128_storage> for vec128_storage {
    #[inline(always)]
    unsafe fn unpack(p: vec128_storage) -> Self {
        p
    }
}
impl<'a> Into<&'a [u32; 4]> for &'a vec128_storage {
    fn into(self) -> &'a [u32; 4] {
        unsafe { &self.u32x4 }
    }
}
impl Into<vec128_storage> for [u32; 4] {
    fn into(self) -> vec128_storage {
        vec128_storage { u32x4: self }
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub union vec256_storage {
    u32x8: [u32; 8],
    u64x4: [u64; 4],
    u128x2: [u128; 2],
    sse2: [vec128_storage; 2],
    avx: __m256i,
}
impl_into!(vec256_storage, [u32; 8], u32x8);
impl_into!(vec256_storage, [u64; 4], u64x4);
impl_into!(vec256_storage, [u128; 2], u128x2);
impl Into<vec256_storage> for [u64; 4] {
    #[inline(always)]
    fn into(self) -> vec256_storage {
        vec256_storage { u64x4: self }
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub union vec512_storage {
    u32x16: [u32; 16],
    u64x8: [u64; 8],
    u128x4: [u128; 4],
    sse2: [vec128_storage; 4],
    avx: [vec256_storage; 2],
}
impl_into!(vec512_storage, [u32; 16], u32x16);
impl_into!(vec512_storage, [u64; 8], u64x8);
impl_into!(vec512_storage, [u128; 4], u128x4);

pub trait Store<S> {
    unsafe fn unpack(p: S) -> Self;
}

pub trait StoreBytes {
    unsafe fn unsafe_read_le(input: &[u8]) -> Self;
    unsafe fn unsafe_read_be(input: &[u8]) -> Self;
    fn write_le(self, out: &mut [u8]);
    fn write_be(self, out: &mut [u8]);
}

/// Generate the full set of optimized implementations to take advantage of the most important
/// hardware feature sets.
///
/// This dispatcher is suitable for maximizing throughput.
#[macro_export]
macro_rules! dispatch {
    ($mach:ident, $MTy:ident, { $([$pub:tt$(($krate:tt))*])* fn $name:ident($($arg:ident: $argty:ty),* $(,)*) -> $ret:ty $body:block }) => {
        #[inline(always)]
        $($pub$(($krate))*)* fn $name($($arg: $argty),*) -> $ret {
            #[inline(always)]
            fn fn_impl<$MTy: $crate::Machine>($mach: $MTy, $($arg: $argty),*) -> $ret $body
            type FnTy = unsafe fn($($arg: $argty),*) -> $ret;
            lazy_static! {
                static ref IMPL: FnTy = { dispatch_init() };
            }
            #[cold]
            fn dispatch_init() -> FnTy {
                use std::arch::x86_64::*;
                if is_x86_feature_detected!("avx") {
                    #[target_feature(enable = "avx")]
                    unsafe fn impl_avx($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::AVX::instance(), $($arg),*)
                    }
                    impl_avx
                } else if is_x86_feature_detected!("sse4.1") {
                    #[target_feature(enable = "sse4.1")]
                    unsafe fn impl_sse41($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::SSE41::instance(), $($arg),*)
                    }
                    impl_sse41
                } else if is_x86_feature_detected!("ssse3") {
                    #[target_feature(enable = "ssse3")]
                    unsafe fn impl_ssse3($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::SSSE3::instance(), $($arg),*)
                    }
                    impl_ssse3
                } else if is_x86_feature_detected!("sse2") {
                    #[target_feature(enable = "sse2")]
                    unsafe fn impl_sse2($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::SSE2::instance(), $($arg),*)
                    }
                    impl_sse2
                } else {
                    unimplemented!()
                }
            }
            unsafe { IMPL($($arg),*) }
        }
    };
    ($mach:ident, $MTy:ident, { $([$pub:tt $(($krate:tt))*])* fn $name:ident($($arg:ident: $argty:ty),* $(,)*) $body:block }) => {
        dispatch!($mach, $MTy, {
            $([$pub $(($krate))*])* fn $name($($arg: $argty),*) -> () $body
        });
    }
}

/// Generate only the basic implementations necessary to be able to operate efficiently on 128-bit
/// vectors on this platfrom. For x86-64, that would mean SSE2 and AVX.
///
/// This dispatcher is suitable for vector operations that do not benefit from advanced hardware
/// features (e.g. because they are done infrequently), so minimizing their contribution to code
/// size is more important.
#[macro_export]
macro_rules! dispatch_light128 {
    ($mach:ident, $MTy:ident, { $([$pub:tt$(($krate:tt))*])* fn $name:ident($($arg:ident: $argty:ty),* $(,)*) -> $ret:ty $body:block }) => {
        #[inline(always)]
        $([$pub $(($krate))*])* fn $name($($arg: $argty),*) -> $ret {
            #[inline(always)]
            fn fn_impl<$MTy: $crate::Machine>($mach: $MTy, $($arg: $argty),*) -> $ret $body
            type FnTy = unsafe fn($($arg: $argty),*) -> $ret;
            lazy_static! {
                static ref IMPL: FnTy = { dispatch_init() };
            }
            #[cold]
            fn dispatch_init() -> FnTy {
                use std::arch::x86_64::*;
                if is_x86_feature_detected!("avx") {
                    #[target_feature(enable = "avx")]
                    unsafe fn impl_avx($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::AVX::instance(), $($arg),*)
                    }
                    impl_avx
                } else if is_x86_feature_detected!("sse2") {
                    #[target_feature(enable = "sse2")]
                    unsafe fn impl_sse2($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::SSE2::instance(), $($arg),*)
                    }
                    impl_sse2
                } else {
                    unimplemented!()
                }
            }
            unsafe { IMPL($($arg),*) }
        }
    };
    ($mach:ident, $MTy:ident, { $([$pub:tt$(($krate:tt))*])* fn $name:ident($($arg:ident: $argty:ty),* $(,)*) $body:block }) => {
        dispatch_light128!($mach, $MTy, {
            $([$pub $(($krate))*])* fn $name($($arg: $argty),*) -> () $body
        });
    }
}

/// Generate only the basic implementations necessary to be able to operate efficiently on 256-bit
/// vectors on this platfrom. For x86-64, that would mean SSE2, AVX, and AVX2.
///
/// This dispatcher is suitable for vector operations that do not benefit from advanced hardware
/// features (e.g. because they are done infrequently), so minimizing their contribution to code
/// size is more important.
#[macro_export]
macro_rules! dispatch_light256 {
    ($mach:ident, $MTy:ident, { $([$pub:tt$(($krate:tt))*])* fn $name:ident($($arg:ident: $argty:ty),* $(,)*) -> $ret:ty $body:block }) => {
        #[inline(always)]
        $([$pub $(($krate))*])* fn $name($($arg: $argty),*) -> $ret {
            #[inline(always)]
            fn fn_impl<$MTy: $crate::Machine>($mach: $MTy, $($arg: $argty),*) -> $ret $body
            type FnTy = unsafe fn($($arg: $argty),*) -> $ret;
            lazy_static! {
                static ref IMPL: FnTy = { dispatch_init() };
            }
            #[cold]
            fn dispatch_init() -> FnTy {
                use std::arch::x86_64::*;
                if is_x86_feature_detected!("avx") {
                    #[target_feature(enable = "avx")]
                    unsafe fn impl_avx($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::AVX::instance(), $($arg),*)
                    }
                    impl_avx
                } else if is_x86_feature_detected!("sse2") {
                    #[target_feature(enable = "sse2")]
                    unsafe fn impl_sse2($($arg: $argty),*) -> $ret {
                        fn_impl($crate::machine::x86::SSE2::instance(), $($arg),*)
                    }
                    impl_sse2
                } else {
                    unimplemented!()
                }
            }
            unsafe { IMPL($($arg),*) }
        }
    };
    ($mach:ident, $MTy:ident, { $([$pub:tt$(($krate:tt))*])* fn $name:ident($($arg:ident: $argty:ty),* $(,)*) $body:block }) => {
        dispatch_light128!($mach, $MTy, {
            $([$pub $(($krate))*])* fn $name($($arg: $argty),*) -> () $body
        });
    }
}
