#![no_std]
extern crate groestl_aesni;
#[macro_use]
extern crate digest;

use digest::dev::{main_test, Test};

#[test]
fn groestl_256() {
    let tests = new_tests!("groestl_256/test32_0", "groestl_256/test32_17",
                          "groestl_256/test32_64", "groestl_256/test32_123");
    main_test::<groestl_aesni::Groestl256>(&tests);
}
