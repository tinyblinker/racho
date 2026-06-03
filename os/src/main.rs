#![no_std]
#![no_main]

//add my MODs
#[macro_use]
mod console;
pub mod batch;
mod lang_items;
mod sbi;
mod sync;
pub mod trap;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    panic!("Shutdown machine!");
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
