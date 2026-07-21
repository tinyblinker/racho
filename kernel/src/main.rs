#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;

mod boards;
mod config;
mod lang_items;
mod loader;
mod logging;
mod mm;
mod sbi;
mod sync;
mod timer;

pub mod syscall;
pub mod task;
pub mod trap;

use core::arch::global_asm;
use framework::clear_bss;
use log::*;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

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
    info!("[kernel test02] heap init test passed!");
}

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    // mm::init();
    // trap::init();
    // trap::enable_timer_interrupt();
    // timer::set_next_trigger();
    // task::run_first_task();
    panic!("Unreachable in rust_main!");
}
