# Cryptocorrosion

Cryptographic algorithms in pure Rust.

The main interface to these crates is the RustCrypto traits.

All crates are no-std compatible.

Minimum Rust version:
- algorithm crates (with RustCrypto API): 1.41.0
- support crates: 1.32.0

[![Build Status](https://travis-ci.org/cryptocorrosion/cryptocorrosion.svg?branch=master)](https://travis-ci.org/cryptocorrosion/cryptocorrosion)

## Supported algorithms

### Cryptographic hashes

| Algo   | Crate name    | SIMD               |
| ------ | ------------- | ------------------ |
| Blake  | blake-hash    | [1]                |
| Grøstl | groestl-aesni | :heavy_check_mark: |
| JH     | jh-x86\_64    | :heavy_check_mark: |
| Skein  | skein-hash    | :x:                |

[1] SIMD is available for builds with target-cpu/target-feature configured, but
runtime CPU detection is not yet supported.

### Block ciphers

| Algo       | Crate name       | SIMD               |
| ---------- | ---------------- | ------------------ |
| Threefish  | threefish-cipher | :x:                |

### Stream ciphers

| Algo       | Crate name | SIMD               |
| ---------- | ---------- | ------------------ |
| ChaCha     | c2-chacha  | :heavy_check_mark: |

## License

All crates licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
