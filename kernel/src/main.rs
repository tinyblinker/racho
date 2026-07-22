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
mod timer;
mod trap;

pub mod syscall;
pub mod task;

use core::arch::global_asm;

global_asm!(include_str!("link_app.S"));

#[allow(unused)]
fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    let bss_range = framework::sbss_addr()..framework::ebss_addr();
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
}

#[unsafe(export_name = "kernel_main")]
pub fn main() -> ! {
    logging::init();
    mm::init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in main!");
}
