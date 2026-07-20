fn main() {
    println!("cargo:rustc-link-arg=-Tuser_lib/src/linker.ld");
}
