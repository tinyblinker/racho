//! Types related to task management

use super::context::TaskContext;
use crate::config::TRAP_CONTEXT;
use crate::config::kernel_stack_position;
use crate::mm::KERNEL_SPACE;
use crate::mm::MemorySet;
use crate::mm::PhysPageNum;
use crate::mm::VirtAddr;

/// task contrl block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    // 地址的应用空间
    pub memory_set: MemorySet,
    // trap context实际在的physpagenum
    pub trap_cx_ppn: PhysPageNum,
    // 统计了应用数据的大小
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            permission,
        );
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
