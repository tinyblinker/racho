<p align="center">
  <img src="https://img.shields.io/github/stars/shyweeds/racho?style=social" alt="Stars">
  <a href="https://github.com/shyweeds/racho/actions/workflows/CI.yml"><img src="https://github.com/shyweeds/racho/actions/workflows/CI.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/rustc-nightly-orange.svg" alt="Rustc">
  <img src="https://img.shields.io/badge/arch-riscv64-blue.svg" alt="RISC-V 64">
  <img src="https://img.shields.io/badge/license-GPLv2-blue.svg" alt="License">
</p>

<h1 align="center">🌾 racho</h1>

<p align="center">
  <strong>A toy operating system kernel written in Rust for RISC-V 64 — from bare-metal boot to time-sharing multitasking, aiming to run BusyBox</strong>
</p>

<p align="center">
  <em>Built along the <a href="https://rcore-os.cn/rCore-Tutorial-Book-v3/">rCore Tutorial</a> (Ch.1–3). Kernel boots on QEMU virt; all subsystems (trap, task, syscall) are implemented — currently being wired up in <code>main.rs</code>.</em>
</p>

---

## ✨ Features

- **Bare-metal kernel** — runs directly on QEMU `virt` (RISC-V 64, Supervisor mode), no host OS, no `std`
- **Batch processing** — loads and executes multiple user-space applications sequentially
- **Time-sharing scheduling** — round-robin scheduler with preemptive timer interrupts (~100 Hz)
- **Trap handling** — full trap frame save/restore (32 GPRs + `sstatus` + `sepc`), dispatches interrupts, exceptions, and syscalls
- **Syscall interface** — `write`, `exit`, `yield`, `get_time`
- **User library** — small `user_lib` crate for writing user-space apps with `println!`, ecall wrappers, and a linker script
- **GDB debugging** — scripts for connecting `riscv64-elf-gdb` to QEMU
- **CI pipeline** — GitHub Actions builds and runs the kernel in QEMU on every push

---

## 🧱 Architecture

```
┌──────────────────────────────────────────────┐
│                  User Space                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ power_3  │  │ power_5  │  │  sleep   │   │
│  │ (app 0)  │  │ (app 1)  │  │ (app 3)  │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │  ecall      │  ecall      │  ecall    │
├───────┼─────────────┼─────────────┼───────────┤
│       ▼             ▼             ▼           │
│  ┌──────────────────────────────────────┐    │
│  │         Trap Handler                 │    │
│  │  (trap.S / trap_handler)             │    │
│  └──────────────┬───────────────────────┘    │
│                 │                             │
│  ┌──────────────▼───────────────────────┐    │
│  │         Syscall Dispatcher           │    │
│  │  write / exit / yield / get_time     │    │
│  └──────────────┬───────────────────────┘    │
│                 │                             │
│  ┌──────────────▼───────────────────────┐    │
│  │      Task Manager (Round-Robin)      │    │
│  │         __switch (switch.S)          │    │
│  └──────────────────────────────────────┘    │
│               Kernel Space                    │
├──────────────────────────────────────────────┤
│               RustSBI (M-mode)                │
└──────────────────────────────────────────────┘
│            QEMU virt (RISC-V 64)            │
└──────────────────────────────────────────────┘
```

**Memory Layout**

| Region   | Address             | Size      |
|----------|---------------------|-----------|
| Kernel   | `0x80200000`        | —         |
| App 0    | `0x80400000`        | 128 KiB   |
| App 1    | `0x80420000`        | 128 KiB   |
| ...      | ...                 | ...       |
| App 15   | `0x80400000 + 15*0x20000` | 128 KiB |
| Max apps | 16                  |           |
| Kernel stack | 8 KiB per app   |           |
| User stack   | 8 KiB per app   |           |

---

## 📁 Project Structure

```
racho/
├── bootloader/               # Prebuilt RustSBI binary
├── os/                       # Kernel crate
│   ├── src/
│   │   ├── main.rs           # Entry point: rust_main()
│   │   ├── entry.asm         # ASM entry: _start
│   │   ├── trap/             # Trap handling (mod.rs / context.rs / trap.S)
│   │   ├── task/             # Task manager & context switch (task.rs / switch.S)
│   │   ├── syscall/          # Syscall dispatcher (mod.rs / fs.rs / process.rs)
│   │   ├── sync/             # UPSafeCell (uniprocessor-safe interior mutability)
│   │   ├── loader.rs         # Loads app binaries into memory
│   │   ├── timer.rs          # RISC-V timer (mtime/mtimecmp)
│   │   ├── logging.rs        # Color-coded logger
│   │   ├── sbi.rs            # SBI ecall wrappers
│   │   └── boards/qemu.rs    # Board-specific constants
│   ├── linker-qemu.ld        # Linker script
│   └── build.rs              # Embeds user app binaries via link_app.S
├── user/                     # User-space crate
│   ├── src/
│   │   ├── lib.rs            # User library (_start, syscalls)
│   │   └── bin/              # Test applications
│   │       ├── 00power_3.rs  # 3^200000 mod M (CPU-bound)
│   │       ├── 01power_5.rs  # 5^140000 mod M
│   │       ├── 02power_7.rs  # 7^160000 mod M
│   │       └── 03sleep.rs    # Busy-wait 3s with yield
│   └── build.py              # Builds each app at incrementing base addresses
├── .github/workflows/CI.yml  # CI: builds & runs in QEMU
├── run_tcp_off.sh            # Run QEMU (no GDB)
├── run_tcp_on.sh             # Run QEMU (with GDB stub)
├── tcp_gdb_on.sh             # Connect GDB to QEMU
└── Makefile                  # Top-level build & run
```

---

## 🚀 Getting Started

### Prerequisites

- **Rust** nightly toolchain (see [`rust-toolchain.toml`](rust-toolchain.toml))
- **QEMU** with RISC-V 64 support (`qemu-system-riscv64`)
- **GDB** for RISC-V (`riscv64-elf-gdb`) — optional, for debugging

Install Rust with the required components:

```bash
rustup toolchain install nightly
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
rustup component add rust-src llvm-tools-preview
cargo install cargo-binutils
```

Install QEMU (Ubuntu/Debian):

```bash
sudo apt install qemu-system-riscv64
```

### Build

```bash
cd os && make build
```

This compiles the 4 user-space apps, embeds them into the kernel via `link_app.S`, builds the kernel ELF, and strips it to `os/target/riscv64gc-unknown-none-elf/release/os.bin`.

### Run

```bash
cd os && make run    # builds + runs in one command
# or:
make run             # top-level alias (requires pre-built os.bin)
```

Current output (trap/task init still commented out in `main.rs:77–81`):

```
[ INFO] [kernel] Hello, world!
heap_test passed!
Panicked at src/main.rs:82 Unreachable in rust_main
```

> Once lines 77–81 are uncommented, the kernel loads 4 user apps and runs the full time-sharing schedule:

<details>
<summary>Full multi-app output (after wiring up)</summary>

```
[ INFO] [kernel] Hello, world!
heap_test passed!
[ INFO] num_app = 4
power_3 [10000/200000]
power_3 [20000/200000]
...
3^200000 = 590847095(MOD 998244353)
Test power_3 OK!
power_5 [10000/140000]
...
Test sleep OK!
[ INFO] All applications completed!
```

</details>

### Debug with GDB

```bash
# Terminal 1: start QEMU with GDB stub
cd os && make build && ./rust_objcopy.sh
cd .. && ./run_tcp_on.sh

# Terminal 2: connect GDB
riscv64-elf-gdb \
  -ex 'file os/target/riscv64gc-unknown-none-elf/release/os' \
  -ex 'set arch riscv:rv64' \
  -ex 'target remote localhost:1234'
```

---

## 🔧 Syscall API

| ID  | Name       | Signature                        | Description              |
|-----|------------|----------------------------------|--------------------------|
| 64  | `write`    | `(fd: usize, buf: *const u8, len: usize) -> isize` | Write to stdout (fd=1) |
| 93  | `exit`     | `(code: i32) -> !`               | Terminate current task   |
| 124 | `yield`    | `() -> isize`                    | Voluntarily yield CPU    |
| 169 | `get_time` | `() -> isize`                    | Get uptime in ms         |

User-space apps call these via the `ecall` instruction (wrappers in [`user/src/syscall.rs`](user/src/syscall.rs)).

---

## 🗺 Roadmap

### Short-term Goal

- 🎯 **Run BusyBox** — extend the kernel with a file system, richer syscall support (`fork`, `exec`, `mmap`, etc.), and a proper process model to boot [BusyBox](https://busybox.net/) on racho.

### Long-term Vision

- Virtual memory / page table support
- Multi-core support (SMP)
- Networking stack
- POSIX compatibility layer

---

## 📚 Acknowledgements

This project follows the excellent **[rCore Tutorial Book v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/)** by the THU OS team. Chapters covered:

- **Chapter 1** — Bare-metal Rust: remove `std`, ASM entry, `println!` via SBI
- **Chapter 2** — Batch OS: trap handling, privilege levels, first syscalls, batch execution of multiple apps
- **Chapter 3** — Time-sharing OS: timer interrupts, task switching, round-robin scheduling, preemptive multitasking

---

## 📄 License

[GPLv2](LICENSE) © 2026 TinyBlink
