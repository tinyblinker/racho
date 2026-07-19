> **注意 / NOTE**
>
> **这个项目处于十分早期的开发阶段，几乎什么都还没做，各方面都很不成熟。README.md 也是用 AI 写的，看起来很唬人但实际功能远未达到。请勿过分严肃对待这个早期项目。待稳定出 release 后我会自行修改这行提示。**
>
> *This project is at an extremely early stage of development — almost nothing has been implemented yet, and everything is immature. The README.md was also written by AI; it looks impressive but actual functionality is far from complete. Please do not take this early, rough project too seriously. I will update this notice myself once a stable release is ready.*

# racho

A Rust kernel for RISC-V 64, built along the [rCore Tutorial](https://rcore-os.cn/rCore-Tutorial-Book-v3/) (Ch.1–4). Currently implements batch/time-sharing task scheduling with SV39 paging.

## Core Short-term Goal

**逐步将当前的测试代码重构为类 Framework Kernel 架构** — 从 rCore 的 monolithic 风格出发，逐步提取一个薄而明确的 unsafe framework 层（页表操作、trap、上下文切换），上层内核逻辑全部用 safe Rust 编写，最终对齐 [Asterinas](https://github.com/asterinas/asterinas) Framekernel 的设计理念。

## Build & Run

```bash
# 进入开发环境 (NixOS)
nix develop
# 或手动安装: rust nightly + riscv64gc target + qemu-system-riscv64

cd os && make run
```

## Project Structure

```
racho/
├── os/                    # 内核 crate
│   ├── src/
│   │   ├── main.rs        # 内核入口
│   │   ├── trap/          # trap 处理 (中断/异常/syscall)
│   │   ├── task/          # 任务管理 (TCB / 调度器 / __switch)
│   │   ├── syscall/       # 系统调用 (write/exit/yield/get_time)
│   │   ├── mm/            # 内存管理 (SV39 页表 / 帧分配器 / 堆)
│   │   ├── sync/          # UPSafeCell
│   │   └── boards/        # 板级配置 (CLOCK_FREQ, MMIO)
│   ├── build.rs           # 生成 link_app.S，嵌入用户程序
│   └── linker-qemu.ld
├── user/                  # 用户态 crate
│   └── src/bin/           # 用户应用 (power_3/5/7, sleep)
├── bootloader/            # RustSBI 二进制
└── flake.nix              # Nix 开发环境
```

## Roadmap

- [ ] **重构为 Framekernel 架构** — 提取 unsafe framework 层，上层 safe Rust
- [ ] 完善单元测试覆盖
- [ ] 文件系统支持
- [ ] `fork` + `exec` 进程模型

## Acknowledgements

本项目基于清华大学 OS 团队 [rCore Tutorial v3](https://rcore-os.cn/rCore-Tutorial-Book-v3/)。Framekernel 架构目标受 [Asterinas](https://github.com/asterinas/asterinas) 启发。

## License

[GPLv2](LICENSE) © 2026 tinyblinker
