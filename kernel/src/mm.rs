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
pub use memory_set::{KERNEL_SPACE, MapPermission, MemorySet};
pub use page_table::translate_byte_buffer;

pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    let bss_range = framework::sbss_addr()..framework::ebss_addr();
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as *const () as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as *const _ as *const () as usize)));
    drop(v);
}

/// initiate the heap allocator, frame allocator and kernel space
pub fn init() {
    init_heap();
    heap_test();
    init_frame_allocator();
    frame_allocator_test();
    KERNEL_SPACE.exclusive_access().active();
}
