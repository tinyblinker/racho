use riscv::register::sstatus::Sstatus;

#[repr(C)]
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: Sstatus,
    /// CSR spec
    pub spec: usize,
}
