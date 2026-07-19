> **NOTE**
>
> *This project is at an extremely early stage of development — almost nothing has been implemented yet, and everything is immature. The README.md was also written by AI; it looks impressive but actual functionality is far from complete. Please do not take this early, rough project too seriously. I will update this notice myself once a stable release is ready.*

# racho

A Rust kernel for RISC-V 64, built along the [rCore Tutorial](https://rcore-os.cn/rCore-Tutorial-Book-v3/) (Ch.1–4). Currently implements batch/time-sharing task scheduling with SV39 paging.

## Goal

Refactor the current test-oriented codebase toward a **Framekernel architecture** — extract a thin, well-defined unsafe framework layer (page tables, trap handling, context switching) from the monolithic rCore style, while the upper-layer kernel logic is written entirely in safe Rust, aligning with the design philosophy of [Asterinas](https://github.com/asterinas/asterinas).

## Build & Run

```bash
nix develop                     # or manually: rust nightly + riscv64gc target + qemu-system-riscv64
cd os && make run
```

## Project Structure

```
racho/
├── os/                    # Kernel crate
│   ├── src/
│   │   ├── main.rs        # Kernel entry
│   │   ├── trap/          # Trap handling (interrupt/exception/syscall)
│   │   ├── task/          # Task management (TCB / scheduler / __switch)
│   │   ├── syscall/       # Syscalls (write/exit/yield/get_time)
│   │   ├── mm/            # Memory management (SV39 paging / frame allocator / heap)
│   │   ├── sync/          # UPSafeCell
│   │   └── boards/        # Board config (CLOCK_FREQ, MMIO)
│   ├── build.rs           # Generates link_app.S, embeds user apps
│   └── linker-qemu.ld
├── user/                  # Userspace crate
│   └── src/bin/           # User apps (power_3/5/7, sleep)
├── bootloader/            # RustSBI binary
└── flake.nix              # Nix dev environment
```

## Roadmap

- [ ] **Refactor to Framekernel architecture** — extract unsafe framework layer, upper-layer safe Rust
- [ ] **Port musl libc** — bring up a minimal C runtime for userland
- [ ] **Boot BusyBox** — support statically-linked BusyBox with musl libc
- [ ] Filesystem support
- [ ] `fork` + `exec` process model

## Acknowledgements

Built upon the [rCore Tutorial v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/) by the THU OS team. Framekernel architecture target inspired by [Asterinas](https://github.com/asterinas/asterinas).

## License

[GPLv2](LICENSE) © 2026 tinyblinker
