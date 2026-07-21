unsafe extern "C" {
    pub fn skernel();
    pub fn stext();
    pub fn etext();
    pub fn srodata();
    pub fn erodata();
    pub fn sdata();
    pub fn edata();
    pub fn sbss();
    pub fn ebss();
    pub fn sbss_with_stack();
    pub fn ekernel();
    pub fn strampoline();
}

pub fn clear_bss() {
    for addr in sbss as usize..ebss as usize {
        unsafe {
            (addr as *mut u8).write_volatile(0);
        }
    }
}
