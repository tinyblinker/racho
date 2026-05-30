#!/bin/bash
riscv64-elf-gdb \
 -ex 'file target/riscv64gc-unknown-none-elf/release/racho' \
 -ex 'set arch riscv:rv64' \
 -ex 'target remote localhost:1234'
