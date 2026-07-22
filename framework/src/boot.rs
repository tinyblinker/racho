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
    fn _num_app();
    fn __alltraps();
    fn __restore();
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

pub fn get_num_app() -> usize {
    unsafe { (_num_app as *const () as *const usize).read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    let num_app = get_num_app();
    let num_app_ptr = _num_app as *const () as *const usize;
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}

pub fn alltraps_addr() -> usize {
    __alltraps as *const () as usize
}

pub fn restore_addr() -> usize {
    __restore as *const () as usize
}

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    unsafe extern "C" {
        fn kernel_main() -> !;
    }
    unsafe {
        kernel_main();
    }
}
