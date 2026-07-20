#!/bin/bash
echo "'<C-a> x' to exit"
cd kernel
./rust_objcopy.sh 
cd ..
qemu-system-riscv64 \
 -machine virt \
 -nographic \
 -bios ./bootloader/rustsbi-qemu.bin \
 -device loader,file=./kernel/target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000 \
 -s -S

