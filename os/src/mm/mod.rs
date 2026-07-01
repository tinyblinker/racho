//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

use crate::mm::memory_set::KERNEL_SPACE;

mod address;
pub mod frame_allocator;
pub mod heap_allocator;
mod memory_set;
mod page_table;

/// initiate the heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    // NOTE: 内核页表是如何建立映射的(TOPIC)
    // 开启SV39分页模式
    KERNEL_SPACE.exclusive_access().active();
}
