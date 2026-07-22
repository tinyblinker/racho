mod context;

pub use context::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("task/switch.S"));

unsafe extern "C" {
    unsafe fn __switch(current_taskcx_ptr: *mut (), next_taskcx_ptr: *const ());
}

// in the core of framework, we can never depend on structs in kernel, or it will be a mess
pub fn context_switch(current_taskcx_ptr: *mut TaskContext, next_taskcx_ptr: *const TaskContext) {
    unsafe {
        __switch(current_taskcx_ptr as *mut (), next_taskcx_ptr as *const ());
    }
}
