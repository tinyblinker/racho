use crate::boards::MEMORY_END;
use crate::mm::address::{PhysAddr, PhysPageNum};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::lazy_static;

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
        if ppn >= self.current || (self.recycled).iter().find(|&v| *v == ppn).is_some() {
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

// TODO: 2026.6.25参考ch4-管理SV39多级页表-分配/回收物理页帧的接口
// 明天从此处开始:验收标准->参考tutorials代码的frame_allocator_test函数

// 声明公开给其它内核模块调用的分配/回收物理页帧的接口
pub fn frame_alloc() -> Option<FrameTracker> {}
