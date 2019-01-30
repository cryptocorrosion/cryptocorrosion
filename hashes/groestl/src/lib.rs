// copyright 2017 Kaz Wesley

//! Implemenation of the Groestl hash function optimized for x86-64 systems.
//! Makes use of sse2, ssse3, and aes extensions as available.

#![cfg_attr(not(feature = "std"), no_std)]

pub extern crate digest;
#[cfg(feature = "std")]
#[macro_use]
extern crate lazy_static;

use block_buffer::byteorder::{BigEndian, ByteOrder, LE};
use block_buffer::generic_array::typenum::{
    PartialDiv, Unsigned, U1024, U16, U28, U48, U512, U64, U8,
};
use block_buffer::generic_array::GenericArray as BBGenericArray;
use block_buffer::BlockBuffer;
use core::fmt::{Debug, Formatter, Result};
use core::mem;
use digest::generic_array::GenericArray as DGenericArray;
pub use digest::Digest;

mod sse2;
use sse2::{init1024, init512, of1024, of512, tf1024, tf512};

#[repr(C, align(16))]
struct Align16<T>(T);

type Block512 = [u64; 512 / 64];
#[repr(C, align(16))]
#[derive(Clone)]
struct Compressor512 {
    cv: [u64; 512 / 64],
}
impl Compressor512 {
    fn new(block: Block512) -> Self {
        let cv = unsafe { mem::transmute(init512(mem::transmute(block))) };
        Compressor512 { cv }
    }
    fn input(&mut self, data: &Block512) {
        unsafe {
            tf512(mem::transmute(&mut self.cv), &*(data.as_ptr() as *const _));
        }
    }
    fn finalize(mut self) -> Block512 {
        unsafe {
            of512(mem::transmute(&mut self.cv));
        }
        self.cv
    }
}

type Block1024 = [u64; 1024 / 64];
#[repr(C, align(16))]
#[derive(Clone)]
struct Compressor1024 {
    cv: [u64; 1024 / 64],
}
impl Compressor1024 {
    fn new(block: Block1024) -> Self {
        let cv = unsafe { mem::transmute(init1024(mem::transmute(block))) };
        Compressor1024 { cv }
    }
    fn input(&mut self, data: &Block1024) {
        unsafe {
            tf1024(mem::transmute(&mut self.cv), &*(data.as_ptr() as *const _));
        }
    }
    fn finalize(mut self) -> Block1024 {
        unsafe {
            of1024(mem::transmute(&mut self.cv));
        }
        self.cv
    }
}

macro_rules! impl_digest {
    ($groestl:ident, $compressor:ident, $bits:ident) => {
        #[derive(Clone)]
        pub struct $groestl {
            buffer: BlockBuffer<<$bits as PartialDiv<U8>>::Output>,
            block_counter: u64,
            compressor: $compressor,
        }
        impl $groestl {
            fn new_truncated(bits: u32) -> Self {
                let mut iv = Align16([0u64; $bits::USIZE / 64]);
                iv.0[iv.0.len() - 1] = u64::from(bits).to_be();
                let compressor = $compressor::new(iv.0);
                Self {
                    buffer: BlockBuffer::default(),
                    compressor: compressor,
                    block_counter: 0,
                }
            }
            fn finalize(self) -> [u64; $bits::USIZE / 64] {
                let mut buffer = self.buffer;
                let mut compressor = self.compressor;
                let count = self.block_counter + 1 + (buffer.remaining() <= 8) as u64;
                buffer.len64_padding::<BigEndian, _>(count, |b| {
                    compressor.input(unsafe { mem::transmute(b) })
                });
                compressor.finalize()
            }
        }
        impl Default for $groestl {
            fn default() -> Self {
                Self::new_truncated($bits::U32 / 2)
            }
        }
        impl Debug for $groestl {
            fn fmt(&self, f: &mut Formatter) -> Result {
                f.write_str("<$groestl>")
            }
        }
        impl digest::BlockInput for $groestl {
            type BlockSize = <$bits as PartialDiv<U8>>::Output;
        }
        impl digest::Input for $groestl {
            fn input<T: AsRef<[u8]>>(&mut self, data: T) {
                let block_counter = &mut self.block_counter;
                let compressor = &mut self.compressor;
                self.buffer.input(data.as_ref(), |b| {
                    *block_counter += 1;
                    compressor.input(unsafe { mem::transmute(b) })
                });
            }
        }
        impl digest::FixedOutput for $groestl {
            type OutputSize = <$bits as PartialDiv<U16>>::Output;
            fn fixed_result(self) -> DGenericArray<u8, Self::OutputSize> {
                let result = self.finalize();
                let mut out: DGenericArray<u8, Self::OutputSize> = DGenericArray::default();
                for (out, &input) in out.chunks_exact_mut(8).zip(&result[$bits::USIZE / 128..]) {
                    LE::write_u64(out, input);
                }
                out
            }
        }
        impl digest::Reset for $groestl {
            fn reset(&mut self) {
                *self = $groestl::default();
            }
        }
    };
}

impl_digest!(Groestl256, Compressor512, U512);
impl_digest!(Groestl512, Compressor1024, U1024);

#[derive(Clone, Debug)]
pub struct Groestl224(Groestl256);
impl Default for Groestl224 {
    fn default() -> Self {
        Groestl224(Groestl256::new_truncated(224))
    }
}
impl digest::BlockInput for Groestl224 {
    type BlockSize = U64;
}
impl digest::Input for Groestl224 {
    fn input<T: AsRef<[u8]>>(&mut self, data: T) {
        digest::Input::input(&mut self.0, data.as_ref());
    }
}
impl digest::FixedOutput for Groestl224 {
    type OutputSize = U28;
    fn fixed_result(self) -> DGenericArray<u8, U28> {
        let result = self.0.finalize();
        let mut out: DGenericArray<u8, U28> = DGenericArray::default();
        LE::write_u32(&mut out[..4], (result[4] >> 32) as u32);
        for (out, &input) in out[4..].chunks_exact_mut(8).zip(&result[5..8]) {
            LE::write_u64(out, input);
        }
        out
    }
}
impl digest::Reset for Groestl224 {
    fn reset(&mut self) {
        self.0 = Groestl256::new_truncated(224);
    }
}

#[derive(Clone, Debug)]
pub struct Groestl384(Groestl512);
impl Default for Groestl384 {
    fn default() -> Self {
        Groestl384(Groestl512::new_truncated(384))
    }
}
impl digest::BlockInput for Groestl384 {
    type BlockSize = <Groestl512 as digest::BlockInput>::BlockSize;
}
impl digest::Input for Groestl384 {
    fn input<T: AsRef<[u8]>>(&mut self, data: T) {
        digest::Input::input(&mut self.0, data.as_ref());
    }
}
impl digest::FixedOutput for Groestl384 {
    type OutputSize = U48;
    fn fixed_result(self) -> DGenericArray<u8, U48> {
        let result = self.0.finalize();
        let mut out: DGenericArray<u8, U48> = DGenericArray::default();
        for (out, &input) in out.chunks_exact_mut(8).zip(&result[10..]) {
            LE::write_u64(out, input);
        }
        out
    }
}
impl digest::Reset for Groestl384 {
    fn reset(&mut self) {
        *self = Groestl384::default();
    }
}
