#!/usr/bin/env bash
BIN="$1"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

SYSROOT=$(rustc --print sysroot)
HOST=$(rustc -vV | sed -n 's/^host: //p')
LLVM_OBJCOPY="$SYSROOT/lib/rustlib/$HOST/bin/llvm-objcopy"
$LLVM_OBJCOPY --strip-all "$BIN" -O binary "$BIN.bin"

if [ "${RACHO_GDB:-0}" = "1" ]; then
    echo "'<C-a> x' to exit"
    exec qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios "$PROJECT_DIR/bootloader/rustsbi-qemu.bin" \
        -device loader,file="$BIN.bin",addr=0x80200000 \
        -s -S
else
    exec qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios "$PROJECT_DIR/bootloader/rustsbi-qemu.bin" \
        -device loader,file="$BIN.bin",addr=0x80200000
fi
