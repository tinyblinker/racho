//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysPageNum, VirtAddr};
pub use frame_allocator::{frame_allocator_test, init_frame_allocator};
pub use heap_allocator::init_heap;
pub use memory_set::{KERNEL_SPACE, MemorySet};

/// initiate the heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    // NOTE: How the kernel page table is set up (TOPIC)
    // Activate SV39 paging mode
    KERNEL_SPACE.exclusive_access().active();
}
