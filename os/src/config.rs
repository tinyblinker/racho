//! Constants used in rCore

/// 看不懂就多写注释整理思路
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
/// 当前设置的PAGE大小(典型值0x1000:4KB)
pub const PAGE_SIZE: usize = 0x1000;
/// 用于存放PAGE的比特位数目(在0x1000大小的page size 中,只有三个比特位用于存放PAGE的offset参数)
pub const PAGE_SIZE_BITS: usize = 0xc;
