#!/bin/bash
cd os
./rust_objcopy.sh 
cd ..
qemu-system-riscv64 \
 -machine virt \
 -nographic \
 -bios ./bootloader/rustsbi-qemu.bin \
 -device loader,file=./os/target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
