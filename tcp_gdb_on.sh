#!/bin/bash
gdb \
 -ex 'file kernel/target/riscv64gc-unknown-none-elf/release/kernel' \
 -ex 'set arch riscv:rv64' \
 -ex 'target remote localhost:1234'
