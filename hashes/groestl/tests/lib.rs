use digest::Digest;

fn do_test(path: &str) {
    let input = std::fs::read(format!("tests/data/{}.input.bin", path)).unwrap();
    let output = std::fs::read(format!("tests/data/{}.output.bin", path)).unwrap();
    let hash = groestl_aesni::Groestl256::digest(&input);
    assert_eq!(&hash[..], &output[..]);
}

#[test]
fn groestl_256_0() {
    for path in &[
        "groestl_256/test32_0",
        "groestl_256/test32_17",
        "groestl_256/test32_32",
        "groestl_256/test32_64",
        "groestl_256/test32_123",
        "groestl_256/test32_131",
    ] {
        do_test(path);
    }
}
