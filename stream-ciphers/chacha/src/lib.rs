// copyright 2019 Kaz Wesley

//! Pure Rust ChaCha with SIMD optimizations.
//!
//! Usage:
//! ```
//! extern crate c2_chacha;
//!
//! use c2_chacha::stream_cipher::{NewStreamCipher, SyncStreamCipher, SyncStreamCipherSeek};
//! use c2_chacha::{ChaCha20, ChaCha12};
//!
//! let key = b"very secret key-the most secret.";
//! let iv = b"my nonce";
//! let plaintext = b"The quick brown fox jumps over the lazy dog.";
//!
//! let mut buffer = plaintext.to_vec();
//! // create cipher instance
//! let mut cipher = ChaCha20::new_var(key, iv).unwrap();
//! // apply keystream (encrypt)
//! cipher.apply_keystream(&mut buffer);
//! // and decrypt it back
//! cipher.seek(0);
//! cipher.apply_keystream(&mut buffer);
//! // stream ciphers can be used with streaming messages
//! let mut cipher = ChaCha12::new_var(key, iv).unwrap();
//! for chunk in buffer.chunks_mut(3) {
//!     cipher.apply_keystream(chunk);
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate byteorder;
extern crate crypto_simd;
#[cfg(test)]
#[macro_use]
extern crate hex_literal;
#[cfg(feature = "std")]
#[macro_use]
extern crate lazy_static;
pub extern crate stream_cipher;

#[cfg(feature = "packed_simd")]
extern crate packed_simd_crate;
#[cfg(not(any(feature = "simd", feature = "packed_simd")))]
extern crate ppv_null;
#[cfg(all(feature = "simd", not(feature = "packed_simd")))]
extern crate simd;
#[cfg(feature = "packed_simd")]
use crypto_simd::packed_simd::u32x4x4;
#[cfg(feature = "packed_simd")]
use packed_simd_crate::u32x4;
#[cfg(not(any(feature = "simd", feature = "packed_simd")))]
use ppv_null::{u32x4, u32x4x4};
#[cfg(all(feature = "simd", not(feature = "packed_simd")))]
use simd::Machine;

use byteorder::{ByteOrder, LE};
use core::{cmp, u32, u64};
use stream_cipher::generic_array::typenum::{Unsigned, U10, U12, U24, U32, U4, U6, U8};
use stream_cipher::generic_array::{ArrayLength, GenericArray};
use stream_cipher::{LoopError, NewStreamCipher, SyncStreamCipher, SyncStreamCipherSeek};

use simd::{
    machine, vec128_storage, ArithOps, BitOps32, LaneWords4, MultiLane, Store, StoreBytes, Vec2,
    Vec4,
};

const BLOCK: usize = 64;
const BLOCK64: u64 = BLOCK as u64;
const LOG2_BUFBLOCKS: u64 = 2;
const BUFBLOCKS: u64 = 1 << LOG2_BUFBLOCKS;
const BUFSZ64: u64 = BLOCK64 * BUFBLOCKS;
const BUFSZ: usize = BUFSZ64 as usize;

const BIG_LEN: u64 = 0;
const SMALL_LEN: u64 = 1 << 32;

#[derive(Clone)]
pub struct State<V> {
    pub a: V,
    pub b: V,
    pub c: V,
    pub d: V,
}
#[inline(always)]
pub fn round<V: ArithOps + BitOps32>(mut x: State<V>) -> State<V> {
    x.a += x.b;
    x.d ^= x.a;
    x.d = x.d.rotate_each_word_right16();
    x.c += x.d;
    x.b ^= x.c;
    x.b = x.b.rotate_each_word_right20();
    x.a += x.b;
    x.d ^= x.a;
    x.d = x.d.rotate_each_word_right24();
    x.c += x.d;
    x.b ^= x.c;
    x.b = x.b.rotate_each_word_right25();
    x
}
#[inline(always)]
pub fn diagonalize<V: LaneWords4>(mut x: State<V>) -> State<V> {
    x.b = x.b.shuffle_lane_words3012();
    x.c = x.c.shuffle_lane_words2301();
    x.d = x.d.shuffle_lane_words1230();
    x
}
#[inline(always)]
pub fn undiagonalize<V: LaneWords4>(mut x: State<V>) -> State<V> {
    x.b = x.b.shuffle_lane_words1230();
    x.c = x.c.shuffle_lane_words2301();
    x.d = x.d.shuffle_lane_words3012();
    x
}

macro_rules! impl_dispatch {
        ($fn:ident, $fn_impl:ident, $width:expr, $ret:ty) => {
    /// Fill a new buffer from the state, autoincrementing internal block count. Caller must count
    /// blocks to ensure this doesn't wrap a 32/64 bit counter, as appropriate.
    #[cfg(not(all(
        feature = "std",
        target_arch = "x86_64",
        any(feature = "simd", feature = "packed_simd")
    )))]
    fn $fn(&mut self, drounds: u32, words: &mut [u8; $width]) -> $ret {
        self.$fn_impl(drounds, words)
    }

    /// Fill a new buffer from the state, autoincrementing internal block count. Caller must count
    /// blocks to ensure this doesn't wrap a 32/64 bit counter, as appropriate.
    #[cfg(all(
        feature = "std",
        target_arch = "x86_64",
        any(feature = "simd", feature = "packed_simd")
    ))]
    fn $fn(&mut self, drounds: u32, words: &mut [u8; $width]) -> $ret {
        type Refill = unsafe fn(state: &mut ChaCha, drounds: u32, words: &mut [u8; $width]) -> $ret;
        lazy_static! {
            static ref IMPL: Refill = { dispatch_init() };
        }
        fn dispatch_init() -> Refill {
            use simd::machine::x86::*;
            /*if is_x86_feature_detected!("avx2") {
                // wide issue
                #[target_feature(enable = "avx2")]
                unsafe fn refill_avx2(
                    state: &mut ChaCha,
                    drounds: u32,
                    words: &mut [u8; $width],
                ) {
                    ChaCha::$fn_impl(state, AVX2, drounds, words);
                }
                refill_avx2
            } else */if is_x86_feature_detected!("sse4.1") {
                #[target_feature(enable = "sse4.1")]
                unsafe fn refill_sse4(
                    state: &mut ChaCha,
                    drounds: u32,
                    words: &mut [u8; $width],
                ) -> $ret {
                    ChaCha::$fn_impl(state, SSE41, drounds, words)
                }
                refill_sse4
            } else if is_x86_feature_detected!("ssse3") {
                // faster rotates
                #[target_feature(enable = "ssse3")]
                unsafe fn refill_ssse3(
                    state: &mut ChaCha,
                    drounds: u32,
                    words: &mut [u8; $width],
                ) -> $ret {
                    ChaCha::$fn_impl(state, SSSE3, drounds, words)
                }
                refill_ssse3
            } else {
                // fallback
                unsafe fn refill_fallback(
                    state: &mut ChaCha,
                    drounds: u32,
                    words: &mut [u8; $width],
                ) -> $ret {
                    ChaCha::$fn_impl(state, SSE2, drounds, words)
                }
                refill_fallback
            }
        }
        unsafe { IMPL(self, drounds, words) }
    }
}}

impl ChaCha {
    /// Set 32-bit block count, affecting next refill.
    #[inline(always)]
    fn seek32(&mut self, blockct: u32) {
        let m = machine::x86::SSE2;
        let d: <machine::x86::SSE2 as Machine>::u32x4 = m.unpack(self.d);
        self.d = d.insert(blockct, 0).pack();
    }

    /// Set 64-bit block count, affecting next refill.
    #[inline(always)]
    fn seek64(&mut self, blockct: u64) {
        let m = machine::x86::SSE2;
        // x86 is little-endian
        let d: <machine::x86::SSE2 as Machine>::u64x2 = m.unpack(self.d);
        self.d = d.insert(blockct, 0).pack();
    }

    #[inline(always)]
    fn refill_wide_impl<M: Machine>(&mut self, m: M, drounds: u32, out: &mut [u8; BUFSZ]) {
        let k = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        // TODO: endian
        let inc = m.vec([1, 0]);
        let d0: M::u64x2 = m.unpack(self.d);
        let d1 = d0 + inc;
        let d2 = d1 + inc;
        let d3 = d2 + inc;
        let b = m.unpack(self.b);
        let c = m.unpack(self.c);
        let mut x = State {
            a: M::u32x4x4::from_lanes([k, k, k, k]),
            b: M::u32x4x4::from_lanes([b, b, b, b]),
            c: M::u32x4x4::from_lanes([c, c, c, c]),
            d: m.unpack(M::u64x2x4::from_lanes([d0, d1, d2, d3]).pack()),
        };
        for _ in 0..drounds {
            x = round(x);
            x = undiagonalize(round(diagonalize(x)));
        }
        let inc = m.vec([1, 0]);
        let d0: M::u64x2 = m.unpack(self.d);
        let d1 = d0 + inc;
        let d2 = d1 + inc;
        let d3 = d2 + inc;
        let d4 = d3 + inc;
        let d1: M::u32x4 = m.unpack(d1.pack());
        let d2: M::u32x4 = m.unpack(d2.pack());
        let d3: M::u32x4 = m.unpack(d3.pack());
        let (a, b, c, d) = (
            x.a.to_lanes(),
            x.b.to_lanes(),
            x.c.to_lanes(),
            x.d.to_lanes(),
        );
        let sb = m.unpack(self.b);
        let sc = m.unpack(self.c);
        let sd = [m.unpack(self.d), d1, d2, d3];
        self.d = d4.pack();
        let mut words = out.chunks_exact_mut(16);
        for ((((&a, &b), &c), &d), &sd) in a.iter().zip(&b).zip(&c).zip(&d).zip(&sd) {
            (a + k).write_le(words.next().unwrap());
            (b + sb).write_le(words.next().unwrap());
            (c + sc).write_le(words.next().unwrap());
            (d + sd).write_le(words.next().unwrap());
        }
    }

    /// Single-block, rounds-only; shared by try_apply_keystream for tails shorter than BUFSZ
    /// and XChaCha's setup step.
    #[inline(always)]
    fn refill_narrow_rounds_impl<M: Machine>(
        &mut self,
        m: M,
        drounds: u32,
        _: &mut [u8; 0],
    ) -> State<vec128_storage> {
        let k: M::u32x4 = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        let mut x = State {
            a: k,
            b: m.unpack(self.b),
            c: m.unpack(self.c),
            d: m.unpack(self.d),
        };
        for _ in 0..drounds {
            x = round(x);
            x = undiagonalize(round(diagonalize(x)));
        }
        State {
            a: x.a.pack(),
            b: x.b.pack(),
            c: x.c.pack(),
            d: x.d.pack(),
        }
    }

    /// Produce output from the current state.
    #[inline(always)]
    fn output_narrow<M: Machine>(&mut self, m: M, x: State<M::u32x4>, out: &mut [u8; BLOCK]) {
        let k = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        (x.a + k).write_le(&mut out[0..16]);
        (x.b + m.unpack(self.b)).write_le(&mut out[16..32]);
        (x.c + m.unpack(self.c)).write_le(&mut out[32..48]);
        (x.d + m.unpack(self.d)).write_le(&mut out[48..64]);
    }

    /// Add one to the block counter (no overflow check).
    #[inline(always)]
    fn inc_block_ct(&mut self) {
        let m = machine::x86::SSE2;
        let d0: <machine::x86::SSE2 as Machine>::u64x2 = m.unpack(self.d);
        self.d = (d0 + m.vec([1, 0])).pack();
    }

    impl_dispatch!(refill_wide, refill_wide_impl, BUFSZ, ());
    impl_dispatch!(
        refill_narrow_rounds,
        refill_narrow_rounds_impl,
        0,
        State<vec128_storage>
    );

    /// Refill the buffer from a single-block round, updating the block count.
    #[inline(always)]
    fn refill_narrow(&mut self, drounds: u32, out: &mut [u8; BLOCK]) {
        let x = self.refill_narrow_rounds(drounds, &mut []);
        let m = machine::x86::SSE2;
        let x = State {
            a: m.unpack(x.a),
            b: m.unpack(x.b),
            c: m.unpack(x.c),
            d: m.unpack(x.d),
        };
        self.output_narrow(m, x, out);
        self.inc_block_ct();
    }
}

mod chacha_any {
    use super::*;
    #[derive(Clone)]
    pub struct ChaCha {
        pub b: vec128_storage,
        pub c: vec128_storage,
        pub d: vec128_storage,
    }

    #[derive(Clone)]
    pub struct Buffer {
        pub state: ChaCha,
        pub out: [u8; BLOCK],
        pub have: i8,
        pub len: u64,
        pub fresh: bool,
    }

    #[derive(Default)]
    pub struct X;
    #[derive(Default)]
    pub struct O;
    #[derive(Clone)]
    pub struct ChaChaAny<NonceSize, Rounds, IsX> {
        pub state: Buffer,
        pub _nonce_size: NonceSize,
        pub _rounds: Rounds,
        pub _is_x: IsX,
    }
}
use self::chacha_any::*;

trait AsBool {
    const BOOL: bool;
}
struct WideEnabled;
impl AsBool for WideEnabled {
    const BOOL: bool = true;
}
#[cfg(test)]
struct WideDisabled;
#[cfg(test)]
impl AsBool for WideDisabled {
    const BOOL: bool = false;
}

impl Buffer {
    fn try_apply_keystream<EnableWide: AsBool>(
        &mut self,
        mut data: &mut [u8],
        drounds: u32,
    ) -> Result<(), LoopError> {
        // Lazy fill: after a seek() we may be partway into a block we don't have yet.
        // We can do this before the overflow check because this is not an effect of the current
        // operation.
        if self.have < 0 {
            self.state.refill_narrow(drounds, &mut self.out);
            self.have += BLOCK as i8;
            // checked in seek()
            self.len -= 1;
        }
        let mut have = self.have as usize;
        let have_ready = cmp::min(have, data.len());
        // Check if the requested position would wrap the block counter. Use self.fresh as an extra
        // bit to distinguish the initial state from the valid state with no blocks left.
        let datalen = (data.len() - have_ready) as u64;
        let blocks_needed = datalen / BLOCK64 + u64::from(datalen % BLOCK64 != 0);
        let (l, o) = self.len.overflowing_sub(blocks_needed);
        if o && !self.fresh {
            return Err(LoopError);
        }
        self.len = l;
        self.fresh &= blocks_needed == 0;
        // If we have data in the buffer, use that first.
        let (d0, d1) = data.split_at_mut(have_ready);
        for (data_b, key_b) in d0.iter_mut().zip(&self.out[(BLOCK - have)..]) {
            *data_b ^= *key_b;
        }
        data = d1;
        have -= have_ready;
        // Process wide chunks.
        if EnableWide::BOOL {
            let (d0, d1) = data.split_at_mut(data.len() & !(BUFSZ - 1));
            for dd in d0.chunks_exact_mut(BUFSZ) {
                let mut buf = [0; BUFSZ];
                self.state.refill_wide(drounds, &mut buf);
                for (data_b, key_b) in dd.iter_mut().zip(buf.iter()) {
                    *data_b ^= *key_b;
                }
            }
            data = d1;
        }
        // Handle the tail a block at a time so we'll have storage for any leftovers.
        for dd in data.chunks_mut(BLOCK) {
            self.state.refill_narrow(drounds, &mut self.out);
            for (data_b, key_b) in dd.iter_mut().zip(self.out.iter()) {
                *data_b ^= *key_b;
            }
            have = BLOCK - dd.len();
        }
        self.have = have as i8;
        Ok(())
    }
}

#[cfg(test)]
impl<NonceSize, Rounds: Unsigned, IsX> ChaChaAny<NonceSize, Rounds, IsX> {
    pub fn try_apply_keystream_narrow(&mut self, data: &mut [u8]) -> Result<(), LoopError> {
        self.state
            .try_apply_keystream::<WideDisabled>(data, Rounds::U32)
    }
}

impl<NonceSize, Rounds> NewStreamCipher for ChaChaAny<NonceSize, Rounds, O>
where
    NonceSize: Unsigned + ArrayLength<u8> + Default,
    Rounds: Default,
{
    type KeySize = U32;
    type NonceSize = NonceSize;
    #[inline]
    fn new(
        key: &GenericArray<u8, Self::KeySize>,
        nonce: &GenericArray<u8, Self::NonceSize>,
    ) -> Self {
        let m = machine::x86::SSE2;
        let ctr_nonce = m.vec([
            0,
            if NonceSize::U32 == 12 {
                LE::read_u32(&nonce[0..4])
            } else {
                0
            },
            LE::read_u32(&nonce[NonceSize::USIZE - 8..NonceSize::USIZE - 4]),
            LE::read_u32(&nonce[NonceSize::USIZE - 4..NonceSize::USIZE]),
        ]);
        let key0 = m.vec([
            LE::read_u32(&key[0..4]),
            LE::read_u32(&key[4..8]),
            LE::read_u32(&key[8..12]),
            LE::read_u32(&key[12..16]),
        ]);
        let key1 = m.vec([
            LE::read_u32(&key[16..20]),
            LE::read_u32(&key[20..24]),
            LE::read_u32(&key[24..28]),
            LE::read_u32(&key[28..32]),
        ]);
        let state = ChaCha {
            b: key0,
            c: key1,
            d: ctr_nonce,
        };
        ChaChaAny {
            state: Buffer {
                state,
                out: [0; BLOCK],
                have: 0,
                len: if NonceSize::U32 == 12 {
                    SMALL_LEN
                } else {
                    BIG_LEN
                },
                fresh: NonceSize::U32 != 12,
            },
            _nonce_size: Default::default(),
            _rounds: Default::default(),
            _is_x: Default::default(),
        }
    }
}

impl<Rounds: Unsigned + Default> NewStreamCipher for ChaChaAny<U24, Rounds, X> {
    type KeySize = U32;
    type NonceSize = U24;
    fn new(
        key: &GenericArray<u8, Self::KeySize>,
        nonce: &GenericArray<u8, Self::NonceSize>,
    ) -> Self {
        let m = machine::x86::SSE2;
        let key0 = m.vec([
            LE::read_u32(&key[0..4]),
            LE::read_u32(&key[4..8]),
            LE::read_u32(&key[8..12]),
            LE::read_u32(&key[12..16]),
        ]);
        let key1 = m.vec([
            LE::read_u32(&key[16..20]),
            LE::read_u32(&key[20..24]),
            LE::read_u32(&key[24..28]),
            LE::read_u32(&key[28..32]),
        ]);
        let nonce0 = m.vec([
            LE::read_u32(&nonce[0..4]),
            LE::read_u32(&nonce[4..8]),
            LE::read_u32(&nonce[8..12]),
            LE::read_u32(&nonce[12..16]),
        ]);
        let mut state = ChaCha {
            b: key0,
            c: key1,
            d: nonce0,
        };
        let x = state.refill_narrow_rounds(Rounds::U32, &mut [0; 0]);
        let ctr_nonce1 = m.vec([
            0,
            0,
            LE::read_u32(&nonce[16..20]),
            LE::read_u32(&nonce[20..24]),
        ]);
        state.b = x.a;
        state.c = x.d;
        state.d = ctr_nonce1;
        ChaChaAny {
            state: Buffer {
                state,
                out: [0; BLOCK],
                have: 0,
                len: BIG_LEN,
                fresh: true,
            },
            _nonce_size: Default::default(),
            _rounds: Default::default(),
            _is_x: Default::default(),
        }
    }
}

impl<NonceSize: Unsigned, Rounds, IsX> SyncStreamCipherSeek for ChaChaAny<NonceSize, Rounds, IsX> {
    #[inline]
    fn current_pos(&self) -> u64 {
        unimplemented!()
        /*
        if NonceSize::U32 != 12 {
            ((u64::from(self.state.state.d.extract(0))
                | (u64::from(self.state.state.d.extract(1)) << 32))) * BLOCK64
        } else {
            u64::from(self.state.state.d.extract(0)) * BLOCK64
        }
        */
    }
    #[inline]
    fn seek(&mut self, ct: u64) {
        let blockct = ct / BLOCK64;
        if NonceSize::U32 != 12 {
            self.state.len = BIG_LEN.wrapping_sub(blockct);
            self.state.state.seek64(blockct);
            self.state.fresh = blockct == 0;
        } else {
            assert!(blockct < SMALL_LEN || (blockct == SMALL_LEN && ct % BLOCK64 == 0));
            self.state.len = SMALL_LEN - blockct;
            self.state.state.seek32(blockct as u32);
        }
        self.state.have = -((ct % BLOCK64) as i8);
    }
}

impl<NonceSize, Rounds: Unsigned, IsX> SyncStreamCipher for ChaChaAny<NonceSize, Rounds, IsX> {
    #[inline]
    fn try_apply_keystream(&mut self, data: &mut [u8]) -> Result<(), LoopError> {
        self.state
            .try_apply_keystream::<WideEnabled>(data, Rounds::U32)
    }
}

/// IETF RFC 7539 ChaCha. Unsuitable for messages longer than 256 GiB.
pub type Ietf = ChaChaAny<U12, U10, O>;
/// ChaCha20, as used in several standards; from Bernstein's original publication.
pub type ChaCha20 = ChaChaAny<U8, U10, O>;
/// Similar to ChaCha20, but with fewer rounds for higher performance.
pub type ChaCha12 = ChaChaAny<U8, U6, O>;
/// Similar to ChaCha20, but with fewer rounds for higher performance.
pub type ChaCha8 = ChaChaAny<U8, U4, O>;
/// Constructed analogously to XSalsa20; mixes during initialization to support both a long nonce
/// and a full-length (64-bit) block counter.
pub type XChaCha20 = ChaChaAny<U24, U10, X>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chacha20_case_1() {
        let key = hex!("fa44478c59ca70538e3549096ce8b523232c50d9e8e8d10c203ef6c8d07098a5");
        let nonce = hex!("8d3a0d6d7827c007");
        let expected = hex!("
                1546a547ff77c5c964e44fd039e913c6395c8f19d43efaa880750f6687b4e6e2d8f42f63546da2d133b5aa2f1ef3f218b6c72943089e4012
                210c2cbed0e8e93498a6825fc8ff7a504f26db33b6cbe36299436244c9b2eff88302c55933911b7d5dea75f2b6d4761ba44bb6f814c9879d
                2ba2ac8b178fa1104a368694872339738ffb960e33db39efb8eaef885b910eea078e7a1feb3f8185dafd1455b704d76da3a0ce4760741841
                217bba1e4ece760eaf68617133431feb806c061173af6b8b2a23be90c5d145cc258e3c119aab2800f0c7bc1959dae75481712cab731b7dfd
                783fa3a228f9968aaea68f36a92f43c9b523337a55b97bcaf5f5774447bf41e8");
        let mut state = ChaCha20::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let offset = 0x3fffffff70u64;
        assert!((offset >> 38) != ((offset + 240) >> 38)); // This will overflow the small word of the counter
        state.seek(offset);
        let mut result = [0; 256];
        state.apply_keystream(&mut result);
        assert_eq!(&expected[..], &result[..]);
    }

    #[test]
    fn chacha12_case_1() {
        let key = hex!("27fc120b013b829f1faeefd1ab417e8662f43e0d73f98de866e346353180fdb7");
        let nonce = hex!("db4b4a41d8df18aa");
        let expected = hex!("
                5f3c8c190a78ab7fe808cae9cbcb0a9837c893492d963a1c2eda6c1558b02c83fc02a44cbbb7e6204d51d1c2430e9c0b58f2937bf593840c
                850bda9051a1f051ddf09d2a03ebf09f01bdba9da0b6da791b2e645641047d11ebf85087d4de5c015fddd044");
        let mut state = ChaCha12::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let mut result = [0u8; 100];
        state.apply_keystream(&mut result);
        assert_eq!(&expected[..], &result[..]);
    }

    #[test]
    fn chacha8_case_1() {
        let key = hex!("641aeaeb08036b617a42cf14e8c5d2d115f8d7cb6ea5e28b9bfaf83e038426a7");
        let nonce = hex!("a14a1168271d459b");
        let mut state = ChaCha8::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let expected = hex!(
        "1721c044a8a6453522dddb3143d0be3512633ca3c79bf8ccc3594cb2c2f310f7bd544f55ce0db38123412d6c45207d5cf9af0c6c680cce1f
        7e43388d1b0346b7133c59fd6af4a5a568aa334ccdc38af5ace201df84d0a3ca225494ca6209345fcf30132e");
        let mut result = [0u8; 100];
        state.apply_keystream(&mut result);
        assert_eq!(&expected[..], &result[..]);
    }

    #[test]
    fn test_ietf() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let nonce = hex!("000000090000004a00000000");
        let expected = hex!(
            "
            10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4e
            d2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e"
        );
        let mut state = Ietf::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let mut result = [0; 64];
        state.seek(64);
        state.apply_keystream(&mut result);
        assert_eq!(&expected[..], &result[..]);
    }

    #[test]
    fn rfc_7539_case_1() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let nonce = hex!("000000090000004a00000000");
        let mut state = Ietf::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let mut result = [0; 128];
        state.apply_keystream(&mut result);
        let expected = hex!(
            "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4e
            d2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e"
        );
        assert_eq!(&expected[..], &result[64..]);
    }

    #[test]
    fn rfc_7539_case_2() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let nonce = hex!("000000000000004a00000000");
        let mut state = Ietf::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let mut buf = [0u8; 178];
        buf[64..].copy_from_slice(plaintext);
        state.apply_keystream(&mut buf);
        let expected = hex!("
            6e2e359a2568f98041ba0728dd0d6981e97e7aec1d4360c20a27afccfd9fae0bf91b65c5524733ab8f593dabcd62b3571639d624e65152ab
            8f530c359f0861d807ca0dbf500d6a6156a38e088a22b65e52bc514d16ccf806818ce91ab77937365af90bbf74a35be6b40b8eedf2785e42
            874d");
        assert_eq!(&expected[..], &buf[64..]);
    }

    #[test]
    fn rfc_7539_case_2_chunked() {
        let key = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let nonce = hex!("000000000000004a00000000");
        let mut state = Ietf::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let mut buf = [0u8; 178];
        buf[64..].copy_from_slice(plaintext);
        state.apply_keystream(&mut buf[..40]);
        state.apply_keystream(&mut buf[40..78]);
        state.apply_keystream(&mut buf[78..79]);
        state.apply_keystream(&mut buf[79..128]);
        state.apply_keystream(&mut buf[128..]);
        let expected = hex!("
            6e2e359a2568f98041ba0728dd0d6981e97e7aec1d4360c20a27afccfd9fae0bf91b65c5524733ab8f593dabcd62b3571639d624e65152ab
            8f530c359f0861d807ca0dbf500d6a6156a38e088a22b65e52bc514d16ccf806818ce91ab77937365af90bbf74a35be6b40b8eedf2785e42
            874d");
        assert_eq!(&expected[..], &buf[64..]);
    }

    #[test]
    fn xchacha20_case_1() {
        let key = hex!("82f411a074f656c66e7dbddb0a2c1b22760b9b2105f4ffdbb1d4b1e824e21def");
        let nonce = hex!("3b07ca6e729eb44a510b7a1be51847838a804f8b106b38bd");
        let mut state = XChaCha20::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );
        let mut xs = [0u8; 100];
        state.apply_keystream(&mut xs);
        let expected = hex!("
            201863970b8e081f4122addfdf32f6c03e48d9bc4e34a59654f49248b9be59d3eaa106ac3376e7e7d9d1251f2cbf61ef27000f3d19afb76b
            9c247151e7bc26467583f520518eccd2055ccd6cc8a195953d82a10c2065916778db35da2be44415d2f5efb0");
        assert_eq!(&expected[..], &xs[..]);
    }

    #[test]
    fn seek_off_end() {
        let mut st = Ietf::new(
            GenericArray::from_slice(&[0xff; 32]),
            GenericArray::from_slice(&[0; 12]),
        );
        st.seek(0x40_0000_0000);

        assert!(st.try_apply_keystream(&mut [0u8; 1]).is_err());
    }

    #[test]
    fn read_last_bytes() {
        let mut st = Ietf::new(
            GenericArray::from_slice(&[0xff; 32]),
            GenericArray::from_slice(&[0; 12]),
        );

        st.seek(0x40_0000_0000 - 10);
        st.apply_keystream(&mut [0u8; 10]);
        assert!(st.try_apply_keystream(&mut [0u8; 1]).is_err());

        st.seek(0x40_0000_0000 - 10);
        assert!(st.try_apply_keystream(&mut [0u8; 11]).is_err());
    }

    #[test]
    fn seek_consistency() {
        let mut st = Ietf::new(
            GenericArray::from_slice(&[50; 32]),
            GenericArray::from_slice(&[44; 12]),
        );

        let mut continuous = [0u8; 1000];
        st.apply_keystream(&mut continuous);

        let mut chunks = [0u8; 1000];

        st.seek(128);
        st.apply_keystream(&mut chunks[128..300]);

        st.seek(0);
        st.apply_keystream(&mut chunks[0..10]);

        st.seek(300);
        st.apply_keystream(&mut chunks[300..533]);

        st.seek(533);
        st.apply_keystream(&mut chunks[533..]);

        st.seek(10);
        st.apply_keystream(&mut chunks[10..128]);

        assert_eq!(&continuous[..], &chunks[..]);
    }

    #[test]
    fn wide_matches_narrow() {
        let key = hex!("fa44478c59ca70538e3549096ce8b523232c50d9e8e8d10c203ef6c8d07098a5");
        let nonce = hex!("8d3a0d6d7827c007");
        let mut buf = [0; 2048];
        let mut state = ChaCha20::new(
            GenericArray::from_slice(&key),
            GenericArray::from_slice(&nonce),
        );

        let lens = [
            2048, 2047, 1537, 1536, 1535, 1025, 1024, 1023, 768, 513, 512, 511, 200, 100, 50,
        ];

        for &len in &lens {
            let buf = &mut buf[0..len];

            // encrypt with hybrid wide/narrow
            state.seek(0);
            state.apply_keystream(buf);
            state.seek(0);
            // decrypt with narrow only
            state.try_apply_keystream_narrow(buf).unwrap();
            for &byte in buf.iter() {
                assert_eq!(byte, 0);
            }

            // encrypt with hybrid wide/narrow
            let offset = 0x3fffffff70u64;
            state.seek(offset);
            state.apply_keystream(buf);
            // decrypt with narrow only
            state.seek(offset);
            state.try_apply_keystream_narrow(buf).unwrap();
            for &byte in buf.iter() {
                assert_eq!(byte, 0);
            }
        }
    }
}
