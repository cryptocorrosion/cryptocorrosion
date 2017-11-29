// copyright 2017 Kaz Wesley

//! Implemenation of the Groestl hash function optimized for x86-64 systems with AES extensions.
//! WARNING: CPU feature detection and portable fallback are left to user!
//!
//! Currently this is a FFI wrapper over the optimized reference implementation.

#![no_std]
#![feature(repr_align, attr_literals)]

pub extern crate digest;

pub use digest::Digest;
use digest::generic_array::GenericArray;
use digest::generic_array::typenum::{U32, U64};

#[allow(dead_code)]
#[repr(C)]
enum HashReturn {
    Success = 0,
    Fail = 1,
}

const ROWS: usize = 8;
const COLS: usize = 8;
const SIZE: usize = ROWS * COLS;

#[derive(Clone)]
#[repr(C, align(128))]
struct HashState {
    chaining: [u64; SIZE / 8],
    buffer: [u8; SIZE],
    block_counter: u64,
    buf_ptr: usize,
    bits_in_last_byte: usize,
}

impl core::fmt::Debug for HashState {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        f.debug_struct("HashState")
            .field("chaining", &"(array)")
            .field("buffer", &"(array)")
            .field("block_counter", &self.block_counter)
            .field("buf_ptr", &self.buf_ptr)
            .field("bits_in_last_byte", &self.bits_in_last_byte)
            .finish()
    }
}

extern "C" {
    fn groestl_init(ctx: *mut HashState);
    fn groestl_update(ctx: *mut HashState, input: *const u8, databitlen: usize) -> HashReturn;
    fn groestl_final(ctx: *mut HashState, output: *mut u8) -> HashReturn;
}

#[derive(Clone, Debug)]
pub struct Groestl256 {
    state: HashState,
}

impl Default for Groestl256 {
    fn default() -> Self {
        let mut hasher = Groestl256 {
            state: HashState {
                chaining: [0u64; SIZE / 8],
                buffer: [0u8; SIZE],
                block_counter: 0,
                buf_ptr: 0,
                bits_in_last_byte: 0,
            },
        };
        unsafe { groestl_init(&mut hasher.state) };
        hasher
    }
}

impl digest::BlockInput for Groestl256 {
    type BlockSize = U64;
}

impl digest::Input for Groestl256 {
    fn process(&mut self, data: &[u8]) {
        match unsafe { groestl_update(&mut self.state, data.as_ptr(), data.len() * 8) } {
            HashReturn::Success => (),
            _ => unreachable!(),
        }
    }
}

impl digest::FixedOutput for Groestl256 {
    type OutputSize = U32;

    fn fixed_result(mut self) -> GenericArray<u8, U32> {
        let mut out = GenericArray::default();
        unsafe { groestl_final(&mut self.state, out.as_mut_ptr()) };
        out
    }
}
