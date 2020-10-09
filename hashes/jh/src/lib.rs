// copyright 2017 Kaz Wesley

//! Portable JH with optimizations for x86-64

#![cfg_attr(not(feature = "std"), no_std)]

pub extern crate digest;
#[macro_use]
extern crate hex_literal;
#[macro_use]
pub extern crate simd;

pub mod compressor;
mod consts;

pub use digest::Digest;

use crate::compressor::Compressor;
use block_buffer::generic_array::GenericArray as BBGenericArray;
use block_buffer::BlockBuffer;
use core::fmt::{Debug, Formatter, Result};
use digest::generic_array::typenum::{Unsigned, U28, U32, U48, U64};
use digest::generic_array::GenericArray as DGenericArray;

macro_rules! define_hasher {
    ($name:ident, $init:path, $OutputBytes:ident) => {
        #[derive(Clone)]
        pub struct $name {
            state: Compressor,
            buffer: BlockBuffer<U64>,
            datalen: usize,
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter) -> Result {
                f.debug_struct("Jh")
                    .field("state", &"(state)")
                    .field("buffer", &"(BlockBuffer<U64>)")
                    .field("datalen", &self.datalen)
                    .finish()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    state: Compressor::new($init),
                    buffer: BlockBuffer::default(),
                    datalen: 0,
                }
            }
        }

        impl digest::BlockInput for $name {
            type BlockSize = U64;
        }

        impl digest::Update for $name {
            fn update(&mut self, data: impl AsRef<[u8]>) {
                let data = data.as_ref();
                self.datalen += data.len();
                let state = &mut self.state;
                self.buffer.input_block(data, |b| state.input(b))
            }
        }

        impl digest::FixedOutputDirty for $name {
            type OutputSize = $OutputBytes;

            fn finalize_into_dirty(&mut self, out: &mut DGenericArray<u8, Self::OutputSize>) {
                let state = &mut self.state;
                let buffer = &mut self.buffer;
                let len = self.datalen as u64 * 8;
                if buffer.position() == 0 {
                    buffer.len64_padding_be(len, |b| state.input(b));
                } else {
                    use block_buffer::block_padding::Iso7816;
                    state.input(buffer.pad_with::<Iso7816>().unwrap());
                    let mut last = BBGenericArray::default();
                    last[56..].copy_from_slice(&len.to_be_bytes());
                    state.input(&last);
                }
                let finalized = self.state.finalize();
                out.copy_from_slice(&finalized[(128 - $OutputBytes::to_usize())..]);
            }
        }

        impl digest::Reset for $name {
            fn reset(&mut self) {
                *self = Self::default();
            }
        }
    };
}

define_hasher!(Jh224, consts::JH224_H0, U28);
define_hasher!(Jh256, consts::JH256_H0, U32);
define_hasher!(Jh384, consts::JH384_H0, U48);
define_hasher!(Jh512, consts::JH512_H0, U64);
