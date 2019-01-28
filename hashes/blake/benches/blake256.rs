#![no_std]
#![feature(test)]
#[macro_use]
extern crate digest;
extern crate blake_hash;

bench!(blake_hash::Blake256);
