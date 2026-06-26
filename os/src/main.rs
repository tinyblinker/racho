#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[path = "boards/qemu.rs"]
mod boards;

//add my MODs
extern crate bitflags;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod logging;
mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;
extern crate alloc;

use core::arch::global_asm;
use log::*;

use crate::mm::{
    frame_allocator::{frame_allocator_test, init_frame_allocator},
    heap_allocator::init_heap,
};
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    for item in sbss as unsafe extern "C" fn() as usize..ebss as unsafe extern "C" fn() as usize {
        unsafe {
            (item as *mut u8).write_volatile(0);
        }
    }
}

#[allow(unused)]
fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    let bss_range = sbss as *const () as usize..ebss as *const () as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as *const () as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as *const _ as *const () as usize)));
    drop(v);
    println!("heap_test passed!");
}

// TODO: ch4-管理多级页表-多级页表管理(从这里开始)(昨天的核心完成点是跑通frame_allocator_test())
// 页表基本数据结构与访问接口
// 我们知道，SV39 多级页表是以节点为单位进行管理的。每个节点恰好存储在一个物理页帧中，它的位置可以用一个物理页号来表示。

/// the rust entry-point of OS
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    info!("[kernel] Hello, world!");

    // 初始化堆并测试堆内存可行性
    init_heap();
    heap_test();

    //初始化内存分配器
    init_frame_allocator();
    frame_allocator_test();

    // 运行应用程序
    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();

    panic!("Unreachable in rust_main");
}
