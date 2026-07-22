use buddy_system_allocator::LockedHeap;
use core::arch::asm;

pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn heap_region() -> &'static mut [u8] {
    unsafe { &mut *core::ptr::addr_of_mut!(HEAP_SPACE) }
}

pub fn init_heap_allocator(allocator: &LockedHeap) {
    let heap = heap_region();
    unsafe {
        allocator
            .lock()
            .init(heap.as_mut_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

pub fn phys_to_byte_slice(phys_addr: usize, len: usize) -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(phys_addr as *mut u8, len) }
}

pub fn phys_to_slice<T>(phys_addr: usize, count: usize) -> &'static mut [T] {
    unsafe { core::slice::from_raw_parts_mut(phys_addr as *mut T, count) }
}

pub fn phys_to_ref<T>(phys_addr: usize) -> &'static mut T {
    unsafe { (phys_addr as *mut T).as_mut().unwrap() }
}

pub fn write_satp(val: usize) {
    unsafe {
        asm!("csrw satp, {}", in(reg) val);
    }
}

pub fn sfence_vma() {
    unsafe {
        asm!("sfence.vma");
    }
}

pub fn zero_memory(phys_addr: usize, len: usize) {
    for addr in phys_addr..phys_addr + len {
        unsafe {
            (addr as *mut u8).write_volatile(0);
        }
    }
}

pub fn fence_i() {
    unsafe {
        asm!("fence.i");
    }
}
