use riscv::{
    self,
    register::{sie, stvec, utvec::TrapMode},
};
pub fn set_stvec(addr: usize, mode: TrapMode) {
    unsafe {
        stvec::write(addr, mode);
    }
}
pub fn set_sie_enable_stimer() {
    unsafe {
        sie::set_stimer();
    }
}
