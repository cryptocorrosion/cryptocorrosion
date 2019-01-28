#![no_std]
#![feature(test)]
#[macro_use]
extern crate digest;
extern crate groestl_aesni;

bench!(groestl_aesni::Groestl512);
