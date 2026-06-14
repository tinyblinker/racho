mod context;
mod task;

use super::sync::UPSafeCell;
use crate::config::MAX_APP_NUM;
use crate::loader::init_app_cx;
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
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
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

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);

        let mut _unused = TaskContext::zero_init();

        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }

        panic!("unreachable in run_first_task")
    }

    /// Change the status of current
    fn mark_current_suspended(&self)
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
