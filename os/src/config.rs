//! Constants used in rCore

/// Maximum number of user applications
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
/// Current page size (typical value 0x1000: 4KB)
pub const PAGE_SIZE: usize = 0x1000;
/// Number of bits used for page offset (with 0x1000 page size,
/// only the lower 12 bits hold the offset)
pub const PAGE_SIZE_BITS: usize = 12;
/// Number of page table entries per page
pub const PTES_PER_PAGE: usize = 512;
/// Trampoline virtual address (top of the address space)
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::boards::MEMORY_END;
