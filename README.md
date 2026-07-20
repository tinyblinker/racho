# racho

A toy RISC-V 64 kernel written in Rust, following the [rCore Tutorial v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/) (Chapters 1–4). Implements batch and time-sharing task scheduling with SV39 paging. Runs on QEMU `virt` with RustSBI.

## Build & Run

Depends on Rust nightly with the `riscv64gc-unknown-none-elf` target, and `qemu-system-riscv64`.

```bash
nix develop        # or install dependencies manually
make run           # build user apps + kernel, launch QEMU
make debug         # launch with GDB stub
make clean
```

## Structure

```
racho/
├── kernel/              # Kernel crate
│   └── src/
│       ├── trap/        # Interrupt, exception, and syscall handling
│       ├── task/        # TCB, round-robin scheduler, context switch
│       ├── syscall/     # write, exit, yield, get_time
│       ├── mm/          # SV39 page tables, frame allocator, heap
│       ├── sync/        # UPSafeCell
│       └── boards/      # Board-specific config
├── user_lib/            # Userspace library + apps (power_3/5/7, sleep)
├── bootloader/          # RustSBI binary
├── tools/               # QEMU/GDB launch scripts
├── Makefile
├── flake.nix
└── Cargo.toml
```

## License

[GPLv2](LICENSE) © 2026 tinyblinker
