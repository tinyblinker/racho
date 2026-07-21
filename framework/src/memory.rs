use core::arch::asm;

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
