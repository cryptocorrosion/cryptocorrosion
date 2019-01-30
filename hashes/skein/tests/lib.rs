#![no_std]
extern crate skein_hash;
#[macro_use]
extern crate digest;

use digest::dev::digest_test;
use digest::generic_array::typenum::{U32, U64};

new_test!(
    skein256_32,
    "skein256_32",
    skein_hash::Skein256<U32>,
    digest_test
);
new_test!(
    skein512_32,
    "skein512_32",
    skein_hash::Skein512<U32>,
    digest_test
);
new_test!(
    skein1024_32,
    "skein1024_32",
    skein_hash::Skein1024<U32>,
    digest_test
);
new_test!(
    skein256_64,
    "skein256_64",
    skein_hash::Skein256<U64>,
    digest_test
);
new_test!(
    skein512_64,
    "skein512_64",
    skein_hash::Skein512<U64>,
    digest_test
);
new_test!(
    skein1024_64,
    "skein1024_64",
    skein_hash::Skein1024<U64>,
    digest_test
);
