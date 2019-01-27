// copyright 2017 Kaz Wesley

//! Implemenation of the Groestl hash function optimized for x86-64 systems with AES extensions.
//! WARNING: CPU feature detection and portable fallback are left to user!

#![no_std]

pub extern crate digest;

use block_buffer::byteorder::{BigEndian, ByteOrder, LE};
use block_buffer::generic_array::typenum::{U28, U32, U64};
use block_buffer::generic_array::GenericArray as BBGenericArray;
use block_buffer::BlockBuffer;
use core::fmt::{Debug, Formatter, Result};
use core::mem;
use digest::generic_array::GenericArray as DGenericArray;
pub use digest::Digest;

mod sse2;
use sse2::sse2::{init, of512, tf512};

const ROWS: usize = 8;
const COLS: usize = 8;
const SIZE: usize = ROWS * COLS;

#[derive(Clone)]
#[repr(C, align(16))]
struct HashState {
    chaining: [u64; SIZE / 8],
    block_counter: u64,
}

#[repr(C, align(16))]
struct Align16<T>(T);

impl HashState {
    fn new(bits: u32) -> Self {
        unsafe {
            let mut iv = Align16([0u64; SIZE / 8]);
            iv.0[COLS - 1] = u64::from(bits).to_be();
            Self {
                chaining: mem::transmute(init(mem::transmute(iv))),
                block_counter: 0,
            }
        }
    }

    fn input_block(&mut self, block: &BBGenericArray<u8, U64>) {
        self.block_counter += 1;
        unsafe {
            tf512(
                mem::transmute(&mut self.chaining),
                &*(block.as_ptr() as *const _),
            );
        }
    }

    fn finalize(mut self) -> [u64; SIZE / 8] {
        unsafe {
            of512(mem::transmute(&mut self.chaining));
        }
        self.chaining
    }
}

#[derive(Clone)]
pub struct Groestl256 {
    buffer: BlockBuffer<U64>,
    state: HashState,
}
impl Groestl256 {
    fn new_truncated(bits: u32) -> Self {
        Self {
            buffer: BlockBuffer::default(),
            state: HashState::new(bits),
        }
    }
    fn finalize(self) -> [u64; SIZE / 8] {
        let mut state = self.state;
        let mut buffer = self.buffer;
        let count = state.block_counter + 1 + (buffer.remaining() <= 8) as u64;
        buffer.len64_padding::<BigEndian, _>(count, |b| state.input_block(b));
        state.finalize()
    }
}
impl Default for Groestl256 {
    fn default() -> Self {
        Self::new_truncated(256)
    }
}
impl Debug for Groestl256 {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_str("<Groestl256>")
    }
}
impl digest::BlockInput for Groestl256 {
    type BlockSize = U64;
}
impl digest::Input for Groestl256 {
    fn input<T: AsRef<[u8]>>(&mut self, data: T) {
        let state = &mut self.state;
        self.buffer.input(data.as_ref(), |b| state.input_block(b));
    }
}
impl digest::FixedOutput for Groestl256 {
    type OutputSize = U32;
    fn fixed_result(self) -> DGenericArray<u8, U32> {
        let result = self.finalize();
        let mut out: DGenericArray<u8, U32> = DGenericArray::default();
        for (out, &input) in out.as_mut_slice().chunks_exact_mut(8).zip(&result[4..8]) {
            LE::write_u64(out, input);
        }
        out
    }
}
impl digest::Reset for Groestl256 {
    fn reset(&mut self) {
        *self = Groestl256::default();
    }
}

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
