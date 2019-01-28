#![no_std]
#![feature(test)]
#[macro_use]
extern crate digest;
extern crate skein_hash;

use digest::generic_array::typenum::U32;

bench!(skein_hash::Skein256<U32>);
