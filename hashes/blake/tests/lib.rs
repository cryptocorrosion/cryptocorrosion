extern crate blake_hash;
#[macro_use]
extern crate digest;

use digest::dev::digest_test;

new_test!(blake224, "blake224", blake_hash::Blake224, digest_test);
new_test!(blake256, "blake256", blake_hash::Blake256, digest_test);
new_test!(blake384, "blake384", blake_hash::Blake384, digest_test);
new_test!(blake512, "blake512", blake_hash::Blake512, digest_test);
