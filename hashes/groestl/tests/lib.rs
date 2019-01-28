#![no_std]
#[macro_use]
extern crate digest;
extern crate groestl_aesni;

use digest::dev::digest_test;

new_test!(
    groestl_224,
    "groestl224",
    groestl_aesni::Groestl224,
    digest_test
);

new_test!(
    groestl_256,
    "groestl256",
    groestl_aesni::Groestl256,
    digest_test
);

new_test!(
    groestl_384,
    "groestl384",
    groestl_aesni::Groestl384,
    digest_test
);

new_test!(
    groestl_512,
    "groestl512",
    groestl_aesni::Groestl512,
    digest_test
);
