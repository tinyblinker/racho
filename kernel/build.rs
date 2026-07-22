use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-arg=-T{}/../framework/linker-qemu.ld",
        manifest_dir
    );
    println!("cargo:rerun-if-changed=../user_lib/src/");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_dir = manifest_dir
        .parent()
        .unwrap()
        .join("target")
        .join("riscv64gc-unknown-none-elf")
        .join("release");

    let target_path = target_dir.to_string_lossy().into_owned();
    println!("cargo:rerun-if-changed={}", target_path);
}
