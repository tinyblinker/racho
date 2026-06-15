//! Rust wrapper around `__switch`
//!
//! Switching to a different task's context actually happens here!

use super::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    /// Switching to the context of `next_task_cx_ptr`,
    /// saving the current context in `current_task_cx_ptr`.
    pub unsafe fn __switch(
        current_task_cx_ptr: *mut TaskContext,
        next_task_cx_ptr: *const TaskContext,
    );
}
