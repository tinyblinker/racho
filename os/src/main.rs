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

use crate::mm::{frame_allocator_test, init_frame_allocator, init_heap};
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

/// the rust entry-point of OS
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    // Referrence
    // clear_bss();
    // logging::init();
    // info!("[kernel] Hello, world!");
    //
    // // Initialize the kernel heap and run a heap memory test
    // init_heap();
    // heap_test();
    //
    // // Initialize the physical frame allocator
    // init_frame_allocator();
    // frame_allocator_test();
    //
    // // Run user applications
    // trap::init();
    // loader::load_apps();
    // trap::enable_timer_interrupt();
    // timer::set_next_trigger();
    // task::run_first_task();
    //
    // panic!("Unreachable in rust_main");
    clear_bss();
    logging::init();
    info!("[kernal] Hello, world!");
    mm::init();
    info!("[kernel] back to world!");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
    // NOTE: 7.14决定完全重构,刚检查完主函数,后续继续重构
}
