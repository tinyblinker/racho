use core::fmt::{self, Debug, Formatter};

use crate::boards::MEMORY_END;
use crate::mm::address::{PhysAddr, PhysPageNum};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::info;

/// manage a frame which has the same lifecycle as the tracker
/// Uses RAII: the lifecycle of a PhysPageNum is bound to a FrameTracker
/// for convenient management
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}
impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}
// When a FrameTracker's lifetime ends and it is dropped by the compiler,
// the physical frame it controls is recycled back into FRAME_ALLOCATOR
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn)
    }
}
// Implement the Debug trait for FrameTracker for formatted debug output
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN:{:#x}", self.ppn.0))
    }
}

// Define the FrameAllocator trait describing the interface for
// physical frame allocation and deallocation (in units of PhysPageNum)
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

// Implement a simple stack-based physical frame allocation strategy
pub struct StackFrameAllocator {
    current: usize, // Starting physical page number of the free region
    end: usize,     // Ending physical page number of the free region
    recycled: Vec<usize>,
}

// Initialize StackFrameAllocator
impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

// Implement the FrameAllocator trait for StackFrameAllocator
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        // Check if there is a recycled PPN available
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            // No recycled PPNs available; try to allocate from the free region
            // (self.current, self.end]
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.contains(&ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

// Create the global instance of StackFrameAllocator
type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

// We wrap the stack-based frame allocator in a UPSafeCell<T>. Before any
// operation on the allocator, we must call FRAME_ALLOCATOR.exclusive_access()
// to obtain a mutable reference.
pub fn init_frame_allocator() {
    unsafe extern "C" {
        safe fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as *const u8 as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

// TODO: (finished) 2026.6.25 — Refer to ch4: managing SV39 multi-level page tables —
// physical frame allocation and deallocation interface
// Starting point for tomorrow: acceptance criteria -> refer to the
// frame_allocator_test function in the tutorials code

// Public interface for physical frame allocation/deallocation exposed to
// other kernel modules
// alloc a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}
/// deallocate a frame
fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    info!("[kernel test03] frame allocator init test ...");
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    info!("[kernel test03] frame allocator init test ok!");
}
