use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

use crate::mm::{
    address::{PhysPageNum, VirtPageNum},
    frame_allocator::{FrameTracker, frame_alloc},
};

// 用bitflags宏把u8封装成一个标志位的集合类型:支持一些集合运算
bitflags! {
    pub struct PTEFlags: u8{
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

// 接下来实现页表项PageTableEntry
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

// 为页表项添加一些methods
impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        Self { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

// page table structure
pub struct PageTable {
    // 保存它根节点的物理页号root_ppn(此作为区分不同物理页表的区分方式)
    root_ppn: PhysPageNum,
    // frames保存了所有节点(包括根节点),所在的物理页帧
    frames: Vec<FrameTracker>,
}
impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                // R=W=X=0 在 Sv39 中意味着:这不是最终映射,而是"下一张页表在哪里"
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            // SV39中每次查询PageTableEntry都用的是上一次查询(第一次用根)的地址作为基准
            ppn = pte.ppn();
        }
        result
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            // 映射'中途'出现非法PTE,直接跳过并返回None,操作失败
            if !pte.is_valid() {
                return None;
            }
            // SV39中每次查询PageTableEntry都用的是上一次查询(第一次用根)的地址作为基准
            ppn = pte.ppn();
        }
        result
    }
    // 在多级页表中插入和删除键值对
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(
            !pte.is_valid(),
            "vpn {:?} is mapped(valid) before mapping",
            vpn
        );
        // 创建新的PTE,完成一次map
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(
            pte.is_valid(),
            "vpn {:?} is unmapped(invalid) before unmapping",
            vpn
        );
        // 清空PTE,完成一次unmap
        *pte = PageTableEntry::empty();
    }
    // 手动查询页表
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    // 构造RISCV的satp寄存器值(选择SV39分页模式,设置root_ppn)
    // | MODE | ASID | PPN |
    // | 4bit | 16bit | 44bit |
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
