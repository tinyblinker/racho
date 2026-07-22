use core::arch::asm;
use riscv::{
    self,
    register::{sie, stvec, utvec::TrapMode},
};

pub fn set_stvec(addr: usize, mode: TrapMode) {
    unsafe {
        stvec::write(addr, mode);
    }
}
pub fn set_sie_enable_stimer() {
    unsafe {
        sie::set_stimer();
    }
}

pub fn restore_to_user(trap_cx_ptr: usize, user_satp: usize, trampoline_base: usize) -> ! {
    let alltraps = crate::boot::alltraps_addr();
    let restore = crate::boot::restore_addr();
    let restore_va = restore - alltraps + trampoline_base;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}
