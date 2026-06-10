#![no_std]
#![no_main]

//add my MODs
#[macro_use]
mod console;
pub mod batch;
mod lang_items;
mod logging;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
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
    unsafe extern "C" {
        safe fn stext(); // begin addr of text segment
        safe fn etext(); // end addr of text segment
        safe fn srodata(); // start addr of Read-Only data segment
        safe fn erodata(); // end addr of Read-Only data ssegment
        safe fn sdata(); // start addr of data segment
        safe fn edata(); // end addr of data segment
        safe fn sbss(); // start addr of BSS segment
        safe fn ebss(); // end addr of BSS segment
        safe fn boot_stack_lower_bound(); // stack lower bound
        safe fn boot_stack_top(); // stack top
    }
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as *const u8 as usize, etext as *const u8 as usize,
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as *const u8 as usize, erodata as *const u8 as usize,
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as *const u8 as usize, edata as *const u8 as usize,
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as *const u8 as usize, boot_stack_lower_bound as *const u8 as usize,
    );
    error!(
        "[kernel] .bss [{:#x}, {:#x})",
        sbss as *const u8 as usize, ebss as *const u8 as usize,
    );
    trap::init();
    batch::init();
    batch::run_next_app();
}
