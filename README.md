<p align="center">
  <img src="https://img.shields.io/github/stars/shyweeds/racho?style=social" alt="Stars">
  <a href="https://github.com/shyweeds/racho/actions/workflows/CI.yml"><img src="https://github.com/shyweeds/racho/actions/workflows/CI.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/rustc-nightly-orange.svg" alt="Rustc">
  <img src="https://img.shields.io/badge/arch-riscv64-blue.svg" alt="RISC-V 64">
  <img src="https://img.shields.io/badge/license-GPLv2-blue.svg" alt="License">
</p>

<h1 align="center">🌾 racho</h1>

<p align="center">
  <strong>A toy operating system kernel written in Rust for RISC-V 64 — from bare-metal boot to SV39 paging, aiming for an Alpine-like userland with musl + BusyBox</strong>
</p>

<p align="center">
  <em>Built along the <a href="https://rcore-os.cn/rCore-Tutorial-Book-v3/">rCore Tutorial</a> (Ch.1–4). Boots on QEMU virt with full multi-app time-sharing: trap, task, syscall, and frame allocator are all wired up.</em>
</p>

---

## ✨ Features

- **Bare-metal kernel** — runs directly on QEMU `virt` (RISC-V 64, Supervisor mode), no host OS, no `std`
- **Batch processing** — loads and executes multiple user-space applications sequentially
- **Time-sharing scheduling** — round-robin scheduler with preemptive timer interrupts (~100 Hz)
- **Trap handling** — full trap frame save/restore (32 GPRs + `sstatus` + `sepc`), dispatches interrupts, exceptions, and syscalls
- **Syscall interface** — `write`, `exit`, `yield`, `get_time`
- **Virtual memory** — SV39 paging primitives: `PageTableEntry` with `PTEFlags` (V/R/W/X/U/G/A/D), `StackFrameAllocator` with recycled frame reuse, `FrameTracker` (RAII auto-dealloc), and address type conversions (`VirtAddr`/`PhysAddr`/`PhysPageNum`/`VirtPageNum` with page alignment helpers)
- **User library** — `user_lib` crate for writing user-space apps with `println!`, ecall wrappers, and a linker script
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
| Memory   | `0x80200000` .. `0x80800000` | 8 MiB |
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
├── bootloader/               # Prebuilt RustSBI binary (rustsbi-qemu.bin)
├── os/                       # Kernel crate
│   ├── src/
│   │   ├── main.rs           # Entry point: rust_main()
│   │   ├── entry.asm         # ASM entry: _start (sets up boot stack)
│   │   ├── config.rs         # Constants: MAX_APP_NUM, stack/heap sizes
│   │   ├── link_app.S        # Generated: embeds user app binaries into .data
│   │   ├── trap/             # Trap handler (mod.rs / context.rs / trap.S)
│   │   ├── task/             # Task manager & context switch (task.rs / switch.S)
│   │   ├── syscall/          # Syscall dispatcher (mod.rs / fs.rs / process.rs)
│   │   ├── sync/             # UPSafeCell (uniprocessor-safe interior mutability)
│   │   ├── mm/               # Memory management (heap_allocator / frame_allocator / page_table / address)
│   │   ├── loader.rs         # Loads app binaries from link_app.S into memory
│   │   ├── timer.rs          # RISC-V timer (mtime), ~100 Hz tick
│   │   ├── logging.rs        # Color-coded kernel logger
│   │   ├── console.rs        # print!/println! via SBI console_putchar
│   │   ├── sbi.rs            # SBI ecall wrappers (console, timer, shutdown)
│   │   ├── boards/qemu.rs     # Board constants: CLOCK_FREQ, MEMORY_END
│   │   └── lang_items.rs     # Panic handler
│   ├── linker-qemu.ld        # Kernel linker script (base 0x80200000)
│   ├── build.rs              # Generates link_app.S from user app binaries
│   ├── rust_objcopy.sh       # Strips kernel ELF → raw binary
│   ├── rust-analyzer.toml
│   └── Makefile
├── user/                     # User-space crate (user_lib)
│   ├── src/
│   │   ├── lib.rs            # User library: _start entry, syscall wrappers
│   │   ├── syscall.rs        # ecall wrappers (write, exit, yield, get_time)
│   │   ├── console.rs        # print!/println! via write syscall
│   │   ├── lang_items.rs     # Panic handler (infinite loop)
│   │   ├── linker.ld         # App linker script (base 0x80400000, patched per app)
│   │   └── bin/              # User applications
│   │       ├── 00power_3.rs  # 3^200000 mod 998244353 (CPU-bound)
│   │       ├── 01power_5.rs  # 5^140000 mod 998244353
│   │       ├── 02power_7.rs  # 7^160000 mod 998244353
│   │       └── 03sleep.rs    # Busy-wait 3s with yield (cooperative multitasking)
│   ├── build.py              # Builds each app at incrementing base addresses
│   ├── rust-analyzer.toml
│   └── Makefile
├── rust-toolchain.toml       # Nightly Rust + RISC-V target
├── run_tcp_off.sh            # Run kernel in QEMU
├── run_tcp_on.sh             # Run QEMU with GDB stub (-s -S)
├── tcp_gdb_on.sh             # Connect GDB to QEMU
├── .github/workflows/CI.yml  # GitHub Actions: builds & runs in QEMU
└── Makefile                  # Top-level build & run aliases
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

Compiles the 4 user-space apps, embeds them into the kernel via `link_app.S`, builds the kernel ELF, and strips it to `os/target/riscv64gc-unknown-none-elf/release/os.bin`.

### Run

```bash
cd os && make run    # builds + runs in one command
```

Expected output:

```
[ INFO] [kernel] Hello, world!
heap_test passed!
frame_allocator_test passed!
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

### Debug with GDB

```bash
# Build first, then start QEMU with GDB stub
cd os && make build
# Terminal 1:
./run_tcp_on.sh

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

racho's userland design follows the **[Alpine Linux](https://alpinelinux.org/)** philosophy — lightweight, secure, and simple:

| Layer        | Component                                          | Status |
|-------------|----------------------------------------------------|--------|
| C library   | [musl libc](https://musl.libc.org/)               | 🔲      |
| Core utils  | [BusyBox](https://busybox.net/)                    | 🔲      |
| Init system | [OpenRC](https://github.com/OpenRC/openrc) *(TBD)* | 🔲      |

### Short-term Goal

- 🎯 **Boot BusyBox on racho** — implement file system support, expand syscalls (`fork`, `exec`, `mmap`, `brk`, etc.), and build a minimal process model sufficient to run a statically-linked BusyBox with musl libc.

### Medium-term Milestones

- SV39 page table management — map kernel/user address spaces, set up `satp`
- Virtual file system (VFS) layer
- `fork` + `exec` process model
- ELF loader for user-space executables
- Signal handling

### Long-term Vision

- Multi-core support (SMP)
- TCP/IP networking stack
- Port OpenRC as init system
- POSIX compatibility layer

---

## 📚 Acknowledgements

This project follows the excellent **[rCore Tutorial Book v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/)** by the THU OS team. Chapters covered:

- **Chapter 1** — Bare-metal Rust: remove `std`, ASM entry, `println!` via SBI
- **Chapter 2** — Batch OS: trap handling, privilege levels, first syscalls, batch execution of multiple apps
- **Chapter 3** — Time-sharing OS: timer interrupts, task switching, round-robin scheduling, preemptive multitasking
- **Chapter 4** — Address space & paging: SV39 `VirtAddr`/`PhysAddr`/`PhysPageNum`/`VirtPageNum` types, `StackFrameAllocator` (with recycled frame reuse + `frame_allocator_test`), `FrameTracker` (RAII auto-dealloc), and `PageTableEntry` with full `PTEFlags` (V/R/W/X/U/G/A/D)

---

## 📄 License

[GPLv2](LICENSE) © 2026 shyweeds
