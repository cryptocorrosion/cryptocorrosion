extern crate cc;

fn main() {
    cc::Build::new()
        .opt_level(0)
        .file("src/hash.c")
        .compile("groestl_impl");
}
