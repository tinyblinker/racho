//! Trap handling functionality
//!
//! For rCore, we have a single trap entry point, namely `__alltraps`. At
//! initialization in [`init()`], we set the `stvec` CSR to point to it.
//!
//! All traps go through `__alltraps`, which is defined in `trap.S`. The
//! assembly language code does just enough work restore the kernel space
//! context, ensuring that Rust code safely runs, and transfers control to
//! [`trap_handler()`].
//!
//! It then calls different functionality based on what exactly the exception
//! was. For example, timer interrupts trigger task preemption, and syscalls go
//! to [`syscall()`].
mod context;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::{syscall::syscall, timer::set_next_trigger};
pub use context::TrapContext;
use core::arch::{asm, global_asm};
use log::info;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval,
    stvec::{self, TrapMode},
};

global_asm!(include_str!("trap.S"));

/// set kernel trap entry
pub fn init() {
    set_kernel_trap_entry();
}

#[unsafe(no_mangle)]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// TODO: Chapter 9: I/O device
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

fn set_kernel_trap_entry() {
    info!("[kernel test04] set kernel trap entry ...");
    unsafe {
        stvec::write(trap_from_kernel as *const () as usize, TrapMode::Direct);
    }
    info!("[kernel test04] set kernel trap entry ok!");
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    info!("[kernel test05] use sie to enable stimer interrupt ...!");
    unsafe {
        sie::set_stimer(); // can be triggered in the trap_handler
    }
    info!("[kernel test05] use sie to enable stimer interrupt ok!");
}

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() -> ! {
    info!("[kernel notice] trap_handler(): Haha, Userspace fall into Trap!");
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            info!("[kernel notice] trap_handler(): Haha, give user syscall()!");
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it!",
                stval, cx.sepc
            );
            info!(
                "[kernel notice] trap_handler(): Haha, user task pagefault, kernel skip it and go next!"
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            info!(
                "[kernel notice] trap_handler(): Haha, user have IllegalInstruction, kernel skip it and go next!"
            );
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            info!(
                "[kernel notice] trap_handler(): Haha, timer interrupt comes, reset the timer and switch to the next task!"
            );
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            info!(
                "[kernel notice] trap_handler(): Haha, unknown case of trap, i have no idea with what happened!"
            );
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

fn set_user_trap_entry() {
    info!("[kernel test10] (stvec) setting user trap entry ...!");
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
    info!("[kernel test10] (stvec) setting user trap entry ok!");
}

#[unsafe(no_mangle)]
/// set the new addr of __restore asm function in TRAMPOINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    info!(
        "[kernel test09] in '__switch(task a,task b)' call 'ret' or trap_handler() call trap_return(), let PC=ra(trap_return())!"
    );
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    unsafe extern "C" {
        unsafe fn __alltraps();
        unsafe fn __restore();
    }
    let restore_va =
        __restore as *const () as usize - __alltraps as *const () as usize + TRAMPOLINE;
    info!(
        "[kernel notice] jr {restore_va}, goto '__restore' and 'sret' to userspace, until user fall into the trap->goto trap_handler addr stored in 'stvec' !"
    );
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",      // jump to new addr of __restore asm function
            // NOTE:
            // "Put the Rust variable restore_va into a register, and bind that register to the {restore_va} placeholder in the assembly string."
            restore_va = in(reg) restore_va, // compiler paras!! not asm!!
             // a0 = virt addr of Trap Context
            in("a0") trap_cx_ptr, // compiler paras!! not asm!!
            // a1 = userspace's satp setup value
            in("a1") user_satp, // compiler paras!! not asm!!
            // do not return
            options(noreturn)
        );
    }
}
