use core::fmt::{self, Debug, Formatter};

use crate::boards::MEMORY_END;
use crate::mm::address::{PhysAddr, PhysPageNum};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// manage a frame which has the same lifecycle as the tracker
/// 用RAII的思想,把PhysPageNum的生命周期绑定于FrameTracker上,方便管理
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}
impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}
// 当一个FrameTracker生命周期结束被编译器回收的时候
// 我们需要将他控制的物理页帧回收到FRAME_ALLOCATOR中
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn)
    }
}
// 为FrameTracker实现Debug trait,从能能格式化输出调试信息
impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN:{:#x}", self.ppn.0))
    }
}

// 我们申明一个FrameAllocator Trait来描述一个物理页帧管理器需要提供哪些功能
// 以物理页号为单位进行物理页帧的分配和回收
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

// 我们实现一种最简单的栈式物理页帧管理策略StackFrameAllocator
pub struct StackFrameAllocator {
    current: usize, // 空闲内存的起始物理页号
    end: usize,     // 空闲内存的结束物理页号
    recycled: Vec<usize>,
}

// 为StackFrameAllocator实现init()
impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

// 为StackFrameAllocator实现FrameAllocator的Trait
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        // 先检查回收的ppn中有没有能用的
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            // 回收的ppn中没有能用的,那就看一下维护的空闲ppn区间((self.current,self.end])
            // 能不能分一块出来
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.contains(&ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

// 创建StackFrameAllocator的全局实例FRAME_ALLOCATOR
type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

// 这里我们使用UPSafeCell<T>来包裹栈式物理页帧分配器,每次对该分配器进行操作之前,
// 我们都需要先通过,FRAME_ALLOCATOR.exclusive_access()拿到分配器的可变借用
pub fn init_frame_allocator() {
    unsafe extern "C" {
        safe fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as *const u8 as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

// TODO: (finished)2026.6.25参考ch4-管理SV39多级页表-分配/回收物理页帧的接口
// 明天从此处开始:验收标准->参考tutorials代码的frame_allocator_test函数

// 声明公开给其它内核模块调用的分配/回收物理页帧的接口
// alloc a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(FrameTracker::new)
}
/// deallocate a frame
fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
