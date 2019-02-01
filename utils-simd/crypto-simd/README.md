# Crypto-oriented SIMD abstractions

Design:
    - interface as close to RFC2366/packed\_simd as practical
    - pluggable backends

Backends supported:
    - ppv\_null: Emulated SIMD. Safe, portable.
    - ppv\_lite86: x86 implementation using coresimd intrinsics, stable and
      fast to compile.
    - packed\_simd: Support for future compatibility--can probably replace
      ppv\_lite when it's eventually stable; in the meantime, offers an
      unstable SIMD backend for platforms ppv\_lite doesn't support.

## Experimental status

Initially I'm adding functionality as needed for my crypto implementations, so
there will be random gaps in the interface. Eventually I will round out the
feature set and define the available functionality in traits, to ensure that
the backends support the same functionality.
