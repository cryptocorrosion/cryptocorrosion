#![no_std]
extern crate groestl_aesni;
#[macro_use]
extern crate digest;

use digest::dev::{main_test, Test};

#[test]
fn groestl_256_0() {
    let tests = new_tests!("groestl_256/test32_0");
    main_test::<groestl_aesni::Groestl256>(&tests);
}

#[test]
fn groestl_256_17() {
    let tests = new_tests!("groestl_256/test32_17");
    main_test::<groestl_aesni::Groestl256>(&tests);
}

#[test]
fn groestl_256_32() {
    let tests = new_tests!("groestl_256/test32_32");
    main_test::<groestl_aesni::Groestl256>(&tests);
}

#[test]
fn groestl_256_64() {
    let tests = new_tests!("groestl_256/test32_64");
    main_test::<groestl_aesni::Groestl256>(&tests);
}

#[test]
fn groestl_256_123() {
    let tests = new_tests!("groestl_256/test32_123");
    main_test::<groestl_aesni::Groestl256>(&tests);
}

#[test]
fn groestl_256_131() {
    let tests = new_tests!("groestl_256/test32_131");
    main_test::<groestl_aesni::Groestl256>(&tests);
}
