#!/bin/sh

if [ -n "$FAILFAST" ]; then set -e; fi

# no SIMD yet:
# - hashes/threefish
# - block-ciphers/skein

# not ported to crypto-simd API yet:
# - hashes/groestl

echo GENERIC
pushd stream-ciphers/chacha; cargo test --features no_simd; popd

#echo BACKEND packed_simd
#cd hashes/blake; cargo test --no-default-features --features packed_simd,std; cd ../..
#cd hashes/jh; cargo test -p jh-x86_64 --no-default-features --features packed_simd,std; cd ../..
#cd stream-ciphers/chacha; cargo test --no-default-features --features packed_simd,std; cd ../..
