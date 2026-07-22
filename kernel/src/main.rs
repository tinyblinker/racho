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
use framework::clear_bss;
use log::*;

global_asm!(include_str!("entry.asm"));
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

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    // TODO: 2026.7.22:README待加入:放代码到framekernel的原则:只有不得不unsafe{}的最小代码块放入,
    // 而在kernel中能用safe rust解决的绝对不放入framekernel的framework中去
    clear_bss();
    logging::init();
    mm::init();
    // NOTE: finished removed (only necessary unsafe in framework) all the unsafe in the "mm::init"
    // TODO: 7.22 remove necessary(minimal) unsafe to framework in trap::init();
    trap::init(); // NOTE: ToSafeRust: ok!
    trap::enable_timer_interrupt(); // NOTE: ToSafeRust: ok!
    timer::set_next_trigger(); // NOTE: ToSafeRust: ok!
    task::run_first_task(); // TODO: remove unsafe{} in run_first_task()
    panic!("Unreachable in rust_main!");
}
