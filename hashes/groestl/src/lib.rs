// copyright 2017 Kaz Wesley

//! Implemenation of the Groestl hash function optimized for x86-64 systems with AES extensions.
//! WARNING: CPU feature detection and portable fallback are left to user!
//!
//! Currently this is a FFI wrapper over the optimized reference implementation.

#![no_std]
#![feature(attr_literals)]

pub extern crate digest;
extern crate block_buffer;
extern crate byte_tools;

use block_buffer::BlockBuffer512;
use byte_tools::write_u64v_le;
pub use digest::Digest;
use digest::generic_array::GenericArray;
use digest::generic_array::typenum::{U32, U64};

const ROWS: usize = 8;
const COLS: usize = 8;
const SIZE: usize = ROWS * COLS;
const BITS: u64 = 256;

#[derive(Clone)]
#[repr(C, align(128))]
struct HashState {
    chaining: [u64; SIZE / 8],
    block_counter: u64,
}

impl core::fmt::Debug for HashState {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        f.debug_struct("HashState")
            .field("chaining", &"(array)")
            .field("block_counter", &self.block_counter)
            .finish()
    }
}

extern "C" {
    fn init(ctx: *mut [u64; SIZE / 8]);
    fn tf512(ctx: *mut [u64; SIZE / 8], block: *const [u8; SIZE]);
    fn of512(ctx: *mut [u64; SIZE / 8]);
}

impl Default for HashState {
    fn default() -> Self {
        let mut hasher = Self {
            chaining: [0u64; SIZE / 8],
            block_counter: 0,
        };
        hasher.chaining[COLS-1] = BITS.to_be();
        unsafe { init(&mut hasher.chaining) };
        hasher
    }
}

impl HashState {
    fn input_block(&mut self, block: &[u8; SIZE]) {
        self.block_counter += 1;
        unsafe { tf512(&mut self.chaining, block); }
    }

    fn finalize(mut self) -> [u64; SIZE / 8] {
        unsafe { of512(&mut self.chaining); }
        self.chaining
    }
}

#[derive(Clone, Default)]
pub struct Groestl256 {
    state: HashState,
    buffer: BlockBuffer512,
}

impl core::fmt::Debug for Groestl256 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        f.debug_struct("Groestl256")
            .field("state", &self.state)
            .field("buffer", &"(BlockBuffer512)")
            .finish()
    }
}

impl digest::BlockInput for Groestl256 {
    type BlockSize = U64;
}

impl digest::Input for Groestl256 {
    fn process(&mut self, data: &[u8]) {
        let state = &mut self.state;
        self.buffer.input(data, |b| state.input_block(b));
    }
}

impl digest::FixedOutput for Groestl256 {
    type OutputSize = U32;

    fn fixed_result(self) -> GenericArray<u8, U32> {
        let mut state = self.state;
        let mut buffer = self.buffer;
        let count = state.block_counter + 1 + (buffer.remaining() <= 8) as u64;
        buffer.len_padding(count.to_be(), |b| state.input_block(b));
        let result = state.finalize();
        let mut out = GenericArray::default();
        write_u64v_le(&mut out, &result[4..]);
        out
    }
}
