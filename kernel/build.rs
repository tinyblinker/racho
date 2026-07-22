use std::fs::{File, read_dir};
use std::io::{Result, Write};
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

    insert_app_data(&target_path).unwrap();
}

fn insert_app_data(target_path: &str) -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user_lib/src/bin")
        .unwrap()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
        .align 3
        .section .data
        .global _num_app
    _num_app:
        .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
            .section .data
            .global app_{0}_start
            .global app_{0}_end
        app_{0}_start:
            .incbin "{1}/{2}"
        app_{0}_end:"#,
            idx, target_path, app
        )?;
    }
    Ok(())
}
