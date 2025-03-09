extern crate blake_hash;
#[macro_use]
extern crate digest;

use digest::dev::{fixed_test, fixed_reset_test};

new_test!(blake224, "blake224", blake_hash::Blake224, fixed_test);
new_test!(blake256, "blake256", blake_hash::Blake256, fixed_test);
//new_test!(blake384, "blake384", blake_hash::Blake384, fixed_test);
//new_test!(blake512, "blake512", blake_hash::Blake512, fixed_test);

new_test!(blake224_reset, "blake224", blake_hash::Blake224, fixed_reset_test);
new_test!(blake256_reset, "blake256", blake_hash::Blake256, fixed_reset_test);
//new_test!(blake384_reset, "blake384", blake_hash::Blake384, fixed_reset_test);
//new_test!(blake512_reset, "blake512", blake_hash::Blake512, fixed_reset_test);
