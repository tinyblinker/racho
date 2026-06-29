//! Constants used in rCore for qemu

// 针对qemu虚拟机的"板级"设置
pub const CLOCK_FREQ: usize = 1250_0000;
pub const MEMORY_END: usize = 0x8800_0000;

// 设置MMIO (start_addr, offset)
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC in virt-machine
];
