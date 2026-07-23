//! App management syscalls
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// get time in milliseconds
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_fork() -> isize {
    unimplemented!()
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    unimplemented!()
}

pub fn sys_exec(path: *const u8) -> isize {
    unimplemented!()
}

pub fn sys_getpid() -> isize {
    unimplemented!()
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    unimplemented!()
}
