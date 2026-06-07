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
pub mod trap;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

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
    loop {}
}

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
