#![no_std]

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));
global_asm!(include_str!("trap/trap.S"));

mod boot;
mod memory;
mod sync;
mod task;
mod trap;

pub use boot::{
    Kernel, alltraps_addr, boot, clear_bss, ebss_addr, edata_addr, ekernel_addr, erodata_addr,
    etext_addr, get_app_data, get_num_app, restore_addr, sbss_addr, sbss_with_stack_addr,
    sdata_addr, skernel_addr, srodata_addr, stext_addr, strampoline_addr,
};
pub use memory::{
    KERNEL_HEAP_SIZE, fence_i, heap_region, init_heap_allocator, phys_to_byte_slice, phys_to_ref,
    phys_to_slice, sfence_vma, write_satp, zero_memory,
};
pub use sync::UPSafeCell;
pub use task::{TaskContext, context_switch};
pub use trap::{restore_to_user, set_sie_enable_stimer, set_stvec};
