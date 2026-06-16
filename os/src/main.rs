#![no_std]
#![no_main]

#[path = "boards/qemu.rs"]
mod boards;

//add my MODs
#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod logging;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

use core::arch::global_asm;
use log::*;
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

/// the rust entry-point of OS
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    info!("[kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main");
}
