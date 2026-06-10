use super::sync::UPSafeCell;
use crate::sbi::shutdown;
use core::arch::asm;
use lazy_static::*;

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

/// suspend current task
pub fn mark_current_suspended() {}

/// run next task
pub fn run_next_task() {}

/// suspend current task and run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}
