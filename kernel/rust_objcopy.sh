#!/bin/bash
SYSROOT=$(rustc --print sysroot)
HOST=$(rustc -vV | sed -n 's/^host: //p')
LLVM_OBJCOPY="$SYSROOT/lib/rustlib/$HOST/bin/llvm-objcopy"
$LLVM_OBJCOPY --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
