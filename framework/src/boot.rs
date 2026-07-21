unsafe extern "C" {
    fn skernel();
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn sbss_with_stack();
    fn ekernel();
    fn strampoline();
}

pub fn clear_bss() {
    for addr in sbss_addr()..ebss_addr() {
        unsafe {
            (addr as *mut u8).write_volatile(0);
        }
    }
}

macro_rules! define_addr_getter {
    ($name:ident, $sym:ident) => {
        pub fn $name() -> usize {
            $sym as *const () as usize
        }
    };
}

define_addr_getter!(skernel_addr, skernel);
define_addr_getter!(stext_addr, stext);
define_addr_getter!(etext_addr, etext);
define_addr_getter!(srodata_addr, srodata);
define_addr_getter!(erodata_addr, erodata);
define_addr_getter!(sdata_addr, sdata);
define_addr_getter!(edata_addr, edata);
define_addr_getter!(sbss_addr, sbss);
define_addr_getter!(ebss_addr, ebss);
define_addr_getter!(sbss_with_stack_addr, sbss_with_stack);
define_addr_getter!(ekernel_addr, ekernel);
define_addr_getter!(strampoline_addr, strampoline);
