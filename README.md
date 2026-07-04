<p align="center">
  <a href="https://github.com/shyweeds/racho/stargazers">
    <img src="https://img.shields.io/github/stars/shyweeds/racho?style=for-the-badge&color=f5c842&labelColor=1e1e2e" alt="Stars">
  </a>
  <a href="https://github.com/shyweeds/racho/actions/workflows/CI.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/shyweeds/racho/CI.yml?style=for-the-badge&branch=main&color=89b4fa&labelColor=1e1e2e" alt="CI">
  </a>
  <img src="https://img.shields.io/badge/rustc-nightly-f38ba8?style=for-the-badge&logo=rust&labelColor=1e1e2e" alt="Rust">
  <img src="https://img.shields.io/badge/RISC--V--64-89dceb?style=for-the-badge&logo=riscv&labelColor=1e1e2e" alt="RISC-V 64">
  <img src="https://img.shields.io/badge/license-GPLv2-a6e3a1?style=for-the-badge&labelColor=1e1e2e" alt="License">
</p>

<h1 align="center">🌾 racho</h1>

<p align="center">
  <strong>A Rust kernel for RISC-V 64 — from bare-metal to SV39 paging, progressive refactoring toward a <a href="https://github.com/asterinas/asterinas">framekernel</a> architecture with an Alpine-like userland</strong>
</p>

<p align="center">
  <sub>Built along <a href="https://rcore-os.cn/rCore-Tutorial-Book-v3/">rCore Tutorial</a> (Ch.1–4) · ELF task loading · per-task page tables · trampoline satp switching · `KERNEL_SPACE` singleton</sub>
</p>

---

## 🎯 Design Philosophy & Architecture Goal

racho is evolving toward the **[framekernel](https://github.com/asterinas/asterinas)** architecture pioneered by [Asterinas](https://github.com/asterinas/asterinas) — a novel OS architecture that achieves monolithic-kernel performance while enforcing microkernel-like separation between safe and unsafe code:

```
┌──────────────────────────────────────┐
│          Safe Kernel (core)           │  100% safe Rust
│   syscall / fs / net / task / mm ... │
├──────────────────────────────────────┤
│       Framework (ostd / hal)          │  Minimal unsafe Rust
│   page table / trap / context switch  │  Small, auditable TCB
├──────────────────────────────────────┤
│           RustSBI (M-mode)            │
└──────────────────────────────────────┘
```

The current codebase follows the rCore monolithic structure. The medium-term refactoring goal is to extract a thin **unsafe framework layer** (akin to Asterinas's OSTD) that encapsulates all `unsafe` operations — page table manipulation, context switching, trap entry/exit, and hardware interaction — while the rest of the kernel is written entirely in safe Rust.

### Why Framekernel?

| Aspect | Traditional Monolithic | Framekernel |
|--------|----------------------|-------------|
| **Memory safety** | Unsafe Rust pervasive | Unsafe confined to thin framework |
| **TCB size** | Entire kernel | Framework layer only (~few KLOC) |
| **Performance** | Direct function calls | Direct function calls (not IPC) |
| **Auditability** | Hard to isolate | Framework is small & explicit |

---

## ✨ Features

- **Bare-metal kernel** — runs directly on QEMU `virt` (RISC-V 64, Supervisor mode), no host OS, no `std`
- **Batch processing** — ELF-based loading: `TaskControlBlock::new()` calls `MemorySet::from_elf()` → maps LOAD segments + user stack (guard page) + heap + TrapContext + kernel stack in `KERNEL_SPACE`, populates `TrapContext` with entry/sp/kernel_satp/kernel_sp/trap_handler; `TaskManager` uses `Vec<TaskControlBlock>` with `current_user_token()` and `current_trap_cx()`
- **Time-sharing scheduling** — round-robin scheduler with preemptive timer interrupts (~100 Hz)
- **Trap handling** — `__alltraps` saves context → `csrw satp` + `sfence.vma` → `jr trap_handler`; `trap_handler()` returns `!`, dispatches syscalls/page faults/timer, calls `trap_return()`; `trap_return()` sets `stvec` to trampoline, computes `__restore` VA, passes `a0=TrapContext` / `a1=user_satp`, `jr` to `__restore`; `__restore` switches back to user page table, restores regs, `sret`; `trap_from_kernel()` catches kernel-mode traps
- **Syscall interface** — `write`, `exit`, `yield`, `get_time`
- **Virtual memory** — SV39 paging with runtime activation: `KERNEL_SPACE` (`Arc<UPSafeCell<MemorySet>>`) global singleton; `mm::init()` enables paging via `satp::write()` + `sfence.vma`; `MemorySet::active()` writes satp token; `MemorySet::remap_test()` verifies .text not writable, .rodata not writable, .data not executable; `MemorySet::new_kernel()` maps all kernel sections/.data/.bss/physical memory/MMIO/trampoline; `MemorySet::from_elf()` parses ELF (`xmas-elf`) → maps `LOAD` segments + user stack (guard page) + heap + `TrapContext`; `PageTableEntry` with `readable()`/`writable()`/`executable()` query methods; `MapArea` with `MapType` (Identical/Framed) & `MapPermission` (R/W/X/U) + `copy_data()`; `PageTable` with 3-level walk, `map`/`unmap`/`translate`; `StackFrameAllocator` (recycled); `FrameTracker` (RAII); `VPNRange` iterator; `VirtPageNum.indexes()` 3-level VPN decomposition
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
| Memory   | `0x80000000` .. `0x88000000` | 128 MiB |
| Trampoline| `0xFFFFFFFFFFFFF000`| 4 KiB     |
| TrapCtx  | `0xFFFFFFFFFFFFE000`| 4 KiB     |
| MMIO     | `0x00100000`        | 8 KiB     |
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
│   │   ├── config.rs         # Constants + kernel_stack_position() (virtual addr placement)
│   │   ├── link_app.S        # Generated: embeds user app binaries into .data
│   │   ├── trap/             # trap_handler() → trap_return() → __restore trampoline flow
│   │   ├── task/             # Vec<TaskControlBlock> — per-task MemorySet + trap_cx_ppn + base_size + goto_trap_return
│   │   ├── syscall/          # Syscall dispatcher (mod.rs / fs.rs / process.rs)
│   │   ├── sync/             # UPSafeCell (uniprocessor-safe interior mutability)
│   │   ├── mm/               # Memory management (heap / frame_allocator / memory_set / page_table / address)
│   │   ├── loader.rs         # Loads apps (load_apps) & provides get_app_data() for ELF parsing
│   │   ├── timer.rs          # RISC-V timer (mtime), ~100 Hz tick
│   │   ├── logging.rs        # Color-coded kernel logger
│   │   ├── console.rs        # print!/println! via SBI console_putchar
│   │   ├── sbi.rs            # SBI ecall wrappers (console, timer, shutdown)
│   │   ├── boards/qemu.rs     # Board constants: CLOCK_FREQ, MEMORY_END (128 MiB), MMIO
│   │   └── lang_items.rs     # Panic handler
│   ├── linker-qemu.ld        # Linker script: .text.trampoline section (page-aligned), base 0x80200000
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

- ~~SV39 page table management~~ — done: `PageTable` with `map`/`unmap`/`translate`, `satp` token
- ~~Address space (`MemorySet`)~~ — done: `new_kernel()` + `from_elf()` + `active()` + `remap_test()` + `KERNEL_SPACE` + `translate()`
- ~~Trap-based page table switching~~ — done: `trap_handler() → trap_return() → __restore` trampoline flow, full satp switching
- ~~ELF-based task loading~~ — done: `TaskControlBlock::new()` uses `MemorySet::from_elf()`, per-task page tables, `goto_trap_return`, kernel stack in `KERNEL_SPACE`
- Wire `mm::init()` into `main.rs` — replace manual heap/frame init with unified boot, enable SV39 paging at startup
- Virtual file system (VFS) layer
- `fork` + `exec` process model
- Signal handling

### Long-term Vision

- **Framekernel refactoring** — extract unsafe framework layer (page tables, trap, context switch) from safe kernel core, following Asterinas's OSTD/kernel split
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
- **Chapter 4** — Address space & paging: Full integration achieved. `TaskControlBlock::new()` calls `MemorySet::from_elf()` → ELF parsing (`xmas-elf`) → maps LOAD segments + user stack (guard page) + heap + TrapContext + kernel stack in `KERNEL_SPACE` → populates `TrapContext` (entry/sp/kernel_satp/kernel_sp/trap_handler). `trap_handler()` (returns `!`) → `trap_return()` (sets `stvec` to trampoline, computes `__restore` VA, passes `a0=TrapContext`/`a1=user_satp`) → `__restore` (`csrw satp` + `sfence.vma` → restore → `sret`). `TaskManager` uses `Vec<TaskControlBlock>` with `current_user_token()`/`current_trap_cx()`. `TaskContext::goto_trap_return()` replaces `goto_restore()`. `kernel_stack_position()` computes virtual addresses from `TRAMPOLINE`. `trap_from_kernel()` catches kernel-mode traps. `mm::init()` orchestrates heap + frame allocator + paging activation via `KERNEL_SPACE.active()`

The **[framekernel](https://asterinas.github.io/book/kernel/the-framekernel-architecture.html)** architecture target is inspired by [Asterinas](https://github.com/asterinas/asterinas), a production-grade Rust OS kernel that confines unsafe code to a small, auditable framework (OSTD) while keeping the rest of the kernel in safe Rust.

---

## 📄 License

[GPLv2](LICENSE) © 2026 tinyblinker
