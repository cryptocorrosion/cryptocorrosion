// copyright 2019 Kaz Wesley

//! Pure Rust ChaCha with SIMD optimizations.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate byteorder;
extern crate crypto_simd;
#[cfg(test)]
#[macro_use]
extern crate hex_literal;
extern crate stream_cipher;

#[cfg(feature = "packed_simd")]
extern crate packed_simd_crate;
#[cfg(not(any(feature = "simd", feature = "packed_simd")))]
extern crate ppv_null;
#[cfg(all(feature = "simd", not(feature = "packed_simd")))]
extern crate simd;
#[cfg(feature = "packed_simd")]
use crypto_simd::packed_simd::u32x4x2;
#[cfg(feature = "packed_simd")]
use packed_simd_crate::u32x4;
#[cfg(not(any(feature = "simd", feature = "packed_simd")))]
use ppv_null::{u32x4, u32x4x2};
#[cfg(all(feature = "simd", not(feature = "packed_simd")))]
use simd::{u32x4, u32x4x2};

use byteorder::{ByteOrder, LE};
use core::{cmp, u32, u64};
use crypto_simd::*;
use stream_cipher::generic_array::typenum::{Unsigned, U10, U12, U24, U32, U4, U6, U8};
use stream_cipher::generic_array::{ArrayLength, GenericArray};
use stream_cipher::{LoopError, NewStreamCipher, SyncStreamCipher, SyncStreamCipherSeek};

#[derive(Clone, Copy)]
union WordBytes {
    words: [u32; 32],
    bytes: [u8; 128],
}

#[derive(Clone)]
struct X4 {
    a: u32x4x2,
    b: u32x4x2,
    c: u32x4x2,
    d: u32x4x2,
}

#[inline(always)]
fn round(mut x: X4) -> X4 {
    x.a += x.b;
    x.d ^= x.a;
    x.d = x.d.splat_rotate_right(16);
    x.c += x.d;
    x.b ^= x.c;
    x.b = x.b.splat_rotate_right(20);
    x.a += x.b;
    x.d ^= x.a;
    x.d = x.d.splat_rotate_right(24);
    x.c += x.d;
    x.b ^= x.c;
    x.b = x.b.splat_rotate_right(25);
    x
}

#[inline(always)]
fn diagonalize(mut x: X4) -> X4 {
    x.b = x.b.rotate_words_right(3);
    x.c = x.c.rotate_words_right(2);
    x.d = x.d.rotate_words_right(1);
    x
}

#[inline(always)]
fn undiagonalize(mut x: X4) -> X4 {
    x.b = x.b.rotate_words_right(1);
    x.c = x.c.rotate_words_right(2);
    x.d = x.d.rotate_words_right(3);
    x
}

#[derive(Clone)]
struct ChaCha {
    b: u32x4,
    c: u32x4,
    d: u32x4,
}

const BLOCK: u64 = 64;
const BUFBLOCKS: u64 = 2;
const BUFSZ64: u64 = BLOCK * BUFBLOCKS;
const BUFSZ8: u8 = BUFSZ64 as u8;
const BUFSZ: usize = BUFSZ64 as usize;

impl ChaCha {
    // Set 32-bit block count, affecting next refill.
    #[inline(always)]
    fn seek32(&mut self, blockct: u32) {
        self.d = self.d.replace(0, blockct)
    }

    // Set 64-bit block count, affecting next refill.
    #[inline(always)]
    fn seek64(&mut self, blockct: u64) {
        self.d = self
            .d
            .replace(0, blockct as u32)
            .replace(1, (blockct >> 32) as u32);
    }

    // Fill a new buffer from the state, autoincrementing internal block count. Caller must count
    // blocks to ensure this doesn't wrap a 32/64 bit counter, as appropriate.
    #[inline(always)]
    fn refill(&mut self, drounds: u32, words: &mut [u32; 32]) {
        let k = u32x4::new(0x61707865, 0x3320646e, 0x79622d32, 0x6b206574);
        // can ignore high word: value to increment is initially even
        let d1 = self.d + u32x4::new(1, 0, 0, 0);
        let mut x = X4 {
            a: u32x4x2::from_halves(k, k),
            b: u32x4x2::from_halves(self.b, self.b),
            c: u32x4x2::from_halves(self.c, self.c),
            d: u32x4x2::from_halves(self.d, d1),
        };
        for _ in 0..drounds {
            x = round(x);
            x = undiagonalize(round(diagonalize(x)));
        }
        let (a, b, c, d) = (
            x.a.into_halves(),
            x.b.into_halves(),
            x.c.into_halves(),
            x.d.into_halves(),
        );
        (a.0 + k).write_to_slice_unaligned(&mut words[0..4]);
        (b.0 + self.b).write_to_slice_unaligned(&mut words[4..8]);
        (c.0 + self.c).write_to_slice_unaligned(&mut words[8..12]);
        (d.0 + self.d).write_to_slice_unaligned(&mut words[12..16]);
        // can ignore high word
        let d1 = self.d + u32x4::new(1, 0, 0, 0);
        (a.1 + k).write_to_slice_unaligned(&mut words[16..20]);
        (b.1 + self.b).write_to_slice_unaligned(&mut words[20..24]);
        (c.1 + self.c).write_to_slice_unaligned(&mut words[24..28]);
        (d.1 + d1).write_to_slice_unaligned(&mut words[28..32]);
        for w in words {
            *w = w.to_le();
        }
        let blockct = (u64::from(d1.extract(0)) | (u64::from(d1.extract(1)) << 32)) + 1;
        self.d = d1
            .replace(0, blockct as u32)
            .replace(1, (blockct >> 32) as u32);
    }
}

#[derive(Clone)]
struct Buffer {
    state: ChaCha,
    out: WordBytes,
    pos: u8,
    len: u64,
}

impl Buffer {
    fn try_apply_keystream(&mut self, data: &mut [u8], drounds: u32) -> Result<(), LoopError> {
        let mut pos = usize::from(self.pos);
        let (d0, d1) = data.split_at_mut(cmp::min(BUFSZ - (pos % BUFSZ), data.len()));
        let d1 = d1.chunks_mut(BUFSZ);
        // check for overflow
        let (l, o) = self
            .len
            .overflowing_sub(d1.len() as u64 + (pos / BUFSZ) as u64);
        if o {
            return Err(LoopError);
        }
        self.len = l;
        // lazy fill for startup / seek / emptied buffer
        if pos >= BUFSZ {
            self.state.refill(drounds, unsafe { &mut self.out.words });
            pos -= BUFSZ;
        }
        // consume remaining keybytes
        for (data_b, key_b) in d0.iter_mut().zip(unsafe { &self.out.bytes[pos..] }) {
            *data_b ^= *key_b;
        }
        pos += d0.len();
        // use new blocks of keybytes for remaining data
        for dd in d1 {
            self.state.refill(drounds, unsafe { &mut self.out.words });
            for (data_b, key_b) in dd.iter_mut().zip(unsafe { self.out.bytes.iter() }) {
                *data_b ^= *key_b;
            }
            pos = dd.len();
        }
        self.pos = pos as u8;
        Ok(())
    }
}

#[derive(Default)]
pub struct X;
#[derive(Default)]
pub struct O;

#[derive(Clone)]
pub struct ChaChaAny<NonceSize, Rounds, IsX> {
    state: Buffer,
    _nonce_size: NonceSize,
    _rounds: Rounds,
    _is_x: IsX,
}

const BIG_LEN: u64 = 1 << 63;
const SMALL_LEN: u64 = 1 << 31;

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
        let ctr_nonce = u32x4::new(
            0,
            if NonceSize::U32 == 12 {
                LE::read_u32(&nonce[0..4])
            } else {
                0
            },
            LE::read_u32(&nonce[NonceSize::USIZE - 8..NonceSize::USIZE - 4]),
            LE::read_u32(&nonce[NonceSize::USIZE - 4..NonceSize::USIZE]),
        );
        let key0 = u32x4::new(
            LE::read_u32(&key[0..4]),
            LE::read_u32(&key[4..8]),
            LE::read_u32(&key[8..12]),
            LE::read_u32(&key[12..16]),
        );
        let key1 = u32x4::new(
            LE::read_u32(&key[16..20]),
            LE::read_u32(&key[20..24]),
            LE::read_u32(&key[24..28]),
            LE::read_u32(&key[28..32]),
        );
        let state = ChaCha {
            b: key0,
            c: key1,
            d: ctr_nonce,
        };
        ChaChaAny {
            state: Buffer {
                state,
                out: WordBytes { words: [0; 32] },
                pos: BUFSZ8,
                len: if NonceSize::U32 == 12 {
                    BIG_LEN
                } else {
                    SMALL_LEN
                },
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
        let k = u32x4::new(0x61707865, 0x3320646e, 0x79622d32, 0x6b206574);
        let key0 = u32x4::new(
            LE::read_u32(&key[0..4]),
            LE::read_u32(&key[4..8]),
            LE::read_u32(&key[8..12]),
            LE::read_u32(&key[12..16]),
        );
        let key1 = u32x4::new(
            LE::read_u32(&key[16..20]),
            LE::read_u32(&key[20..24]),
            LE::read_u32(&key[24..28]),
            LE::read_u32(&key[28..32]),
        );
        let nonce0 = u32x4::new(
            LE::read_u32(&nonce[0..4]),
            LE::read_u32(&nonce[4..8]),
            LE::read_u32(&nonce[8..12]),
            LE::read_u32(&nonce[12..16]),
        );
        let ctr_nonce1 = u32x4::new(
            0,
            0,
            LE::read_u32(&nonce[16..20]),
            LE::read_u32(&nonce[20..24]),
        );
        let mut x = X4 {
            a: u32x4x2::from_half(k),
            b: u32x4x2::from_half(key0),
            c: u32x4x2::from_half(key1),
            d: u32x4x2::from_half(nonce0),
        };
        for _ in 0..Rounds::U32 {
            x = round(x);
            x = undiagonalize(round(diagonalize(x)));
        }
        let state = ChaCha {
            b: x.a.into_halves().0,
            c: x.d.into_halves().0,
            d: ctr_nonce1,
        };
        ChaChaAny {
            state: Buffer {
                state,
                out: WordBytes { words: [0; 32] },
                pos: BUFSZ8,
                len: BIG_LEN,
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
        u64::from(self.state.state.d.extract(0))
    }
    #[inline]
    fn seek(&mut self, ct: u64) {
        let blockct = ct / BUFSZ64;
        if NonceSize::U32 != 12 {
            self.state.state.seek64(blockct << 1);
            self.state.pos = BUFSZ8 + (ct % BUFSZ64) as u8;
            self.state.len = BIG_LEN - blockct;
        } else {
            let blockct32 = blockct as u32;
            assert_eq!(blockct, u64::from(blockct32));
            self.state.state.seek32(blockct32 << 1);
            self.state.len = SMALL_LEN - blockct;
        }
        self.state.pos = BUFSZ8 + (ct % BUFSZ64) as u8;
    }
}

impl<NonceSize, Rounds: Unsigned, IsX> SyncStreamCipher for ChaChaAny<NonceSize, Rounds, IsX> {
    #[inline]
    fn try_apply_keystream(&mut self, data: &mut [u8]) -> Result<(), LoopError> {
        self.state.try_apply_keystream(data, Rounds::U32)
    }
}

pub type Ietf = ChaChaAny<U12, U10, O>;
pub type ChaCha20 = ChaChaAny<U8, U10, O>;
pub type ChaCha12 = ChaChaAny<U8, U6, O>;
pub type ChaCha8 = ChaChaAny<U8, U4, O>;
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
}
