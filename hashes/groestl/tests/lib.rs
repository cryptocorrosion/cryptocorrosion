#![no_std]
#[macro_use]
extern crate digest;
extern crate groestl_aesni;

use digest::dev::digest_test;

new_test!(
    groestl_256,
    "groestl256",
    groestl_aesni::Groestl256,
    digest_test
);
