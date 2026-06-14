mod context;
mod task;

use super::sync::UPSafeCell;
use crate::config::MAX_APP_NUM;
use crate::task::context::TaskContext;
use crate::task::task::TaskControlBlock;
use crate::task::task::TaskStatus;
use crate::{loader::get_num_app, sbi::shutdown};
use core::arch::asm;
use lazy_static::*;

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access;
    inner: UPSafeCell<TaskManagerInner>,
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock{
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];
        for (i,task) in tasks.iter_mut().enumerate(){
            task.task_cx = TaskContext::goto_restore();
            task.task_status = TaskStatus::Ready;
        }
        TaskManager{
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    current_task: 0,
                })
            }
        }
    };
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
