mod context;
mod switch;
mod task;

use super::sync::UPSafeCell;
use crate::info;
use crate::loader::get_app_data;
use crate::task::context::TaskContext;
use crate::task::switch::__switch;
use crate::task::task::TaskControlBlock;
use crate::task::task::TaskStatus;
use crate::trap::TrapContext;
use crate::{loader::get_num_app, sbi::shutdown};
use alloc::vec::Vec;
use lazy_static::*;

pub struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
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
        info!("[kernel test07] init TASK_MANAGER ...!");
        let num_app = get_num_app();
        println!("num_app = {}",num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app{
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        info!("[kernel test07] init TASK_MANAGER ok!");
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    current_task: 0,
                })
            },
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

        let mut fake_task_context = TaskContext::zero_init();
        // NOTE: let the "first task" be the "next task" being switch to

        info!("[kernel test08] use 'switch.S' to load the first task's context ...!");
        unsafe {
            // NOTE:
            // __switch(context_being_stored, context_being_restored);
            // store "current context"(group of registers) to context_being_stored
            // restore context_being_restored to "current context"(group of registers)
            __switch(&mut fake_task_context as *mut TaskContext, next_task_cx_ptr);
            // paraments will be put in the a0,a1 for expression in "__switch()" to use
        }

        panic!("unreachable in run_first_task")
    }

    /// Change the status of current
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Ready`
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications complecated
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped mannually
            info!(
                "[kernel notice] run_next_task(): next task exist!Use __switch(task a, task b) to switch task!"
            );
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            info!("[kernel notice] next task not exists! All applications completed! shutdown!");
            shutdown(false);
        }
    }
    /// get the current `Running` task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// run next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task and run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task, then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// get the current `Running` task's token
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// get current task's TrapContext
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}
