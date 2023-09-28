# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2023-09-21

### Added
- Introduced `zeroize_support` feature for securely zeroing out sensitive data in memory.
  - Implemented `Zeroize` trait for relevant types in the library to enable secure zeroing when the feature is enabled.
  - This feature can be enabled via `ppv_lite86 = { version = "0.3", features = ["zeroize_support"] }` in your `Cargo.toml`.

## [0.2.16]
### Added
- add [u64; 4] conversion for generic vec256, to support BLAKE on non-x86.
- impl `From` (rather than just `Into`) for conversions between `*_storage` types and arrays.
