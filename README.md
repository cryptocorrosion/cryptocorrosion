# Cryptocorrosion

Cryptographic algorithms in pure Rust.

The main interface to these crates is the RustCrypto traits.

All crates are no-std compatible.

Minimum Rust version: 1.31.

## Supported algorithms

### Cryptographic hashes

| Algo   | Crate name    |
| ------ | ------------- |
| Blake  | blake-hash    |
| Grøstl | groestl-aesni |
| JH     | jh-x86\_64    |
| Skein  | skein-hash    |

### Block ciphers

| Algo       | Crate name       |
| ---------- | ---------------- |
| Threefish  | threefish-cipher |

### Stream ciphers

## SIMD

Many of the crates in this project include optimized SIMD implementations,
enabled by default on x86-64 by the "simd" feature. The fastest implementation
available for your hardware will be automatically selected at runtime, except
in no-std builds.

For other hardware platforms, e.g. ARM: an alternative, portable SIMD backend
based on the packed\_simd crate is available for recent nightly Rust; you can
enable it as "packed\_simd". 

If you'd prefer to minimize usage of `unsafe` code: disable the "simd" feature
to switch to a generic implementation.

| feature        | crate        | no `unsafe`        | rust version   | build time? | performance   |
| -------------- | ------------ | ------------------ | -------------- | ----------- | ------------- |
| simd (default) | ppv\_lite86  | :x:                | 1.27           | fast        | fastest (x86) |
| (no simd)      | ppv\_null    | :heavy_check_mark: |                | fast        | slow          |
| packed\_simd   | packed\_simd |                    | recent nightly | slow        | fast          |

## License

All crates licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
