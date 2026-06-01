#![no_std]
#![feature(linkage)]

use core::panicking::panic;
use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize{sys_write(fd, buf)}
pub fn exit(exit_code: i32)->isize{sys_exit(exit_code)}

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[unsafe(no_mangle)]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main -> i32{
    panic!("Cannot find main");
}
