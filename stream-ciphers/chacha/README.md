# The ChaCha family of stream ciphers

## Features

- pure Rust implementation
- supports the RustCrypto API
- builds on stable Rust (using ppv-lite86 for SIMD)
- portable (using packed\_simd for SIMD on non-x86 platforms)
- fast: within 15% of throughput of a hand-optimized ASM SIMD implementation
  (floodberry/chacha-opt) on my machine (a Xeon X5650, using ppv-lite86)
- no-std compatible (std required only for runtime algorithm selection)

## Supported Variants

ChaCha20: used in chacha20-poly1305 in TLS, OpenSSH; arc4random in the BSDs,
Linux /dev/urandom since 4.8.

Ietf: IETF RFC 7539. Longer nonce, short block counter.

XChaCha20: constructed analogously to XSalsa20; a mixing step during
initialization allows using a long nonce and along with a full-sized block
counter.

ChaCha12, ChaCha8: faster; lower security margin of safety.

## Configuration

The default features are recommended unless you have special requirements;
ppv-lite86 runs significantly faster than packed\_simd on my machine that has
SSE4.1 but not AVX2, compiles much faster, and builds on stable. To take
advantage of AVX2 or AVX512, use packed\_simd. NEON, WASM, and other platforms
should work through packed\_simd, but are unsupported as I haven't set up
testing yet (if you're interested, file a bug!). Non-SIMD builds should be
possible on any arch Rust can be compiled for.
