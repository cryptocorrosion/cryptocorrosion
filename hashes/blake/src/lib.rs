// copyright 2017 Kaz Wesley

//! Classic Blake in a Rustic setting

#![no_std]

extern crate block_buffer;
extern crate crypto_simd;
pub extern crate digest;
#[cfg(feature = "packed_simd")]
extern crate packed_simd_crate;
#[cfg(not(any(feature = "simd", feature = "packed_simd")))]
extern crate ppv_null;
#[cfg(all(feature = "simd", not(feature = "packed_simd")))]
extern crate simd;

mod consts;

use block_buffer::byteorder::{ByteOrder, BE};
use block_buffer::BlockBuffer;
use core::mem;
use crypto_simd::*;
use digest::generic_array::typenum::{PartialDiv, Unsigned, U2};
use digest::generic_array::GenericArray;
pub use digest::Digest;
#[cfg(feature = "packed_simd")]
use packed_simd_crate::{u32x4, u64x4};
#[cfg(not(any(feature = "simd", feature = "packed_simd")))]
use ppv_null::{u32x4, u64x4};
#[cfg(all(feature = "simd", not(feature = "packed_simd")))]
use simd::{u32x4, u64x4};

macro_rules! define_compressor {
    ($compressor:ident, $word:ident, $Bufsz:ty, $deserializer:path, $serializer:path, $uval:expr,
     $rounds:expr, $shift0:expr, $shift1:expr, $shift2: expr, $shift3: expr, $X4:ident) => {
        #[derive(Debug, Clone, Copy)]
        struct $compressor {
            h: [$word; 8],
        }

        impl $compressor {
            fn put_block(&mut self, block: &GenericArray<u8, $Bufsz>, t: ($word, $word)) {
                const U: [$word; 16] = $uval;

                #[inline(always)]
                fn round((mut a, mut b, mut c, mut d): ($X4, $X4, $X4, $X4), m0: $X4, m1: $X4) -> ($X4, $X4, $X4, $X4)
                {
                    a += m0;
                    a += b;
                    d ^= a;
                    d = d.splat_rotate_right($shift0);
                    c += d;
                    b ^= c;
                    b = b.splat_rotate_right($shift1);
                    a += m1;
                    a += b;
                    d ^= a;
                    d = d.splat_rotate_right($shift2);
                    c += d;
                    b ^= c;
                    b = b.splat_rotate_right($shift3);
                    (a, b, c, d)
                }

                #[inline(always)]
                fn diagonalize((a, b, c, d): ($X4, $X4, $X4, $X4)) -> ($X4, $X4, $X4, $X4) {
                    (a, b.rotate_words_right(3), c.rotate_words_right(2), d.rotate_words_right(1))
                }

                #[inline(always)]
                fn undiagonalize((a, b, c, d): ($X4, $X4, $X4, $X4)) -> ($X4, $X4, $X4, $X4) {
                    (a, b.rotate_words_right(1), c.rotate_words_right(2), d.rotate_words_right(3))
                }

                let mut m = [0; 16];
                for (mx, b) in m
                    .iter_mut()
                    .zip(block.chunks_exact(mem::size_of::<$word>()))
                {
                    *mx = $deserializer(b);
                }

                // TODO: align self.h
                let mut xs = ($X4::from_slice_unaligned(&self.h[0..4]), $X4::from_slice_unaligned(&self.h[4..8]), $X4::from_slice_unaligned(&U[0..4]), $X4::from_slice_unaligned(&U[4..8]));
                xs.3 ^= $X4::new(t.0, t.0, t.1, t.1);
                for sigma in &SIGMA[..$rounds] {
                    macro_rules! m0 { ($e:expr) => (m[sigma[$e] as usize] ^ U[sigma[$e + 1] as usize]) }
                    macro_rules! m1 { ($e:expr) => (m[sigma[$e + 1] as usize] ^ U[sigma[$e] as usize]) }
                    // column step
                    let m0 = $X4::new(m0!(0), m0!(2), m0!(4), m0!(6));
                    let m1 = $X4::new(m1!(0), m1!(2), m1!(4), m1!(6));
                    xs = round(xs, m0, m1);
                    // diagonal step
                    let m0 = $X4::new(m0!(8), m0!(10), m0!(12), m0!(14));
                    let m1 = $X4::new(m1!(8), m1!(10), m1!(12), m1!(14));
                    xs = undiagonalize(round(diagonalize(xs), m0, m1));
                }
                xs.0 ^= xs.2 ^ $X4::from_slice_unaligned(&self.h[0..4]);
                xs.1 ^= xs.3 ^ $X4::from_slice_unaligned(&self.h[4..8]);
                xs.0.write_to_slice_unaligned(&mut self.h[0..4]);
                xs.1.write_to_slice_unaligned(&mut self.h[4..8]);
            }

            fn finalize(self) -> GenericArray<u8, <$Bufsz as PartialDiv<U2>>::Output> {
                let mut out = GenericArray::default();
                for (h, out) in self
                    .h
                    .iter()
                    .zip(out.chunks_exact_mut(mem::size_of::<$word>()))
                {
                    $serializer(out, *h)
                }
                out
            }
        }
    };
}

macro_rules! define_hasher {
    ($name:ident, $word:ident, $buf:expr, $Bufsz:ty, $bits:expr, $Bytes:ident,
     $serializer:path, $compressor:ident, $iv:expr) => {
        #[derive(Clone)]
        pub struct $name {
            compressor: $compressor,
            buffer: BlockBuffer<$Bufsz>,
            t: ($word, $word),
        }

        impl $name {
            fn increase_count(t: &mut ($word, $word), count: $word) {
                let (new_t0, carry) = t.0.overflowing_add(count * 8);
                t.0 = new_t0;
                if carry {
                    t.1 += 1;
                }
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
                f.debug_struct("Blake")
                    .field("compressor", &self.compressor)
                    .field("buffer.position()", &self.buffer.position())
                    .finish()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    compressor: $compressor { h: $iv },
                    buffer: BlockBuffer::default(),
                    t: (0, 0),
                }
            }
        }

        impl digest::BlockInput for $name {
            type BlockSize = $Bytes;
        }

        impl digest::Input for $name {
            fn input<T: AsRef<[u8]>>(&mut self, data: T) {
                let compressor = &mut self.compressor;
                let t = &mut self.t;
                self.buffer.input(data.as_ref(), |block| {
                    Self::increase_count(t, (mem::size_of::<$word>() * 16) as $word);
                    compressor.put_block(block, *t);
                });
            }
        }

        impl digest::FixedOutput for $name {
            type OutputSize = $Bytes;

            fn fixed_result(self) -> GenericArray<u8, $Bytes> {
                let mut compressor = self.compressor;
                let mut buffer = self.buffer;
                let mut t = self.t;

                Self::increase_count(&mut t, buffer.position() as $word);

                let mut msglen = [0u8; $buf / 8];
                $serializer(&mut msglen[..$buf / 16], t.1);
                $serializer(&mut msglen[$buf / 16..], t.0);

                let footerlen = 1 + 2 * mem::size_of::<$word>();

                // low bit indicates full-length variant
                let isfull = ($bits == 8 * mem::size_of::<[$word; 8]>()) as u8;
                // high bit indicates fit with no padding
                let exactfit = if buffer.position() + footerlen != $buf {
                    0x00
                } else {
                    0x80
                };
                let magic = isfull | exactfit;

                // if header won't fit in last data block, pad to the end and start a new one
                let extra_block = buffer.position() + footerlen > $buf;
                if extra_block {
                    let pad = $buf - buffer.position();
                    buffer.input(&PADDING[..pad], |block| compressor.put_block(block, t));
                    debug_assert_eq!(buffer.position(), 0);
                }

                // pad last block up to footer start point
                if buffer.position() == 0 {
                    // don't xor t when the block is only padding
                    t = (0, 0);
                }
                // skip begin-padding byte if continuing padding
                let x = extra_block as usize;
                let (start, end) = (x, x + ($buf - footerlen - buffer.position()));
                buffer.input(&PADDING[start..end], |_| unreachable!());
                buffer.input(&[magic], |_| unreachable!());
                buffer.input(&msglen, |block| compressor.put_block(block, t));
                debug_assert_eq!(buffer.position(), 0);

                GenericArray::clone_from_slice(&compressor.finalize()[..$Bytes::to_usize()])
            }
        }

        impl digest::Reset for $name {
            fn reset(&mut self) {
                *self = Self::default()
            }
        }
    };
}

use consts::{
    BLAKE224_IV, BLAKE256_IV, BLAKE256_U, BLAKE384_IV, BLAKE512_IV, BLAKE512_U, PADDING, SIGMA,
};
use digest::generic_array::typenum::{U128, U28, U32, U48, U64};

#[rustfmt::skip]
define_compressor!(Compressor256, u32, U64, BE::read_u32, BE::write_u32, BLAKE256_U, 14, 16, 12, 8, 7, u32x4);

#[rustfmt::skip]
define_hasher!(Blake224, u32, 64, U64, 224, U28, BE::write_u32, Compressor256, BLAKE224_IV);

#[rustfmt::skip]
define_hasher!(Blake256, u32, 64, U64, 256, U32, BE::write_u32, Compressor256, BLAKE256_IV);

#[rustfmt::skip]
define_compressor!(Compressor512, u64, U128, BE::read_u64, BE::write_u64, BLAKE512_U, 16, 32, 25, 16, 11, u64x4);

#[rustfmt::skip]
define_hasher!(Blake384, u64, 128, U128, 384, U48, BE::write_u64, Compressor512, BLAKE384_IV);

#[rustfmt::skip]
define_hasher!(Blake512, u64, 128, U128, 512, U64, BE::write_u64, Compressor512, BLAKE512_IV);
