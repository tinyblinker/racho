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

struct RachoKernel;

impl framework::Kernel for RachoKernel {
    fn main() -> ! {
        logging::init();
        mm::init();
        trap::init();
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
        task::run_first_task();
        panic!("Unreachable in main!");
    }
}

framework::register_kernel!(RachoKernel);
