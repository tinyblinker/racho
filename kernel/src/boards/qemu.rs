//! Constants used in rCore for qemu

// Board-level configuration for the QEMU virt machine
pub const CLOCK_FREQ: usize = 1250_0000;
pub const MEMORY_END: usize = 0x8800_0000;

// MMIO regions: (start_addr, size)
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC in virt-machine
];
