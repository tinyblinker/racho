use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

use crate::mm::{
    address::{PhysPageNum, VirtPageNum},
    frame_allocator::{FrameTracker, frame_alloc},
};

// Use the `bitflags` macro to wrap a u8 into a flag set type,
// supporting set operations like union and intersection
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

// Page Table Entry (PTE) implementation
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

// Methods for PageTableEntry
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
    // Stores the physical page number of the root node (uniquely identifies
    // each physical page table)
    root_ppn: PhysPageNum,
    // All physical frames held by page table nodes (including the root)
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
                // R=W=X=0 in Sv39 means: this is not a leaf PTE;
                // it points to the next-level page table
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            // In SV39, each page table walk uses the PPN from the previous
            // level (starting with the root) as the base for the next lookup
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
            // An invalid PTE encountered mid-walk means the mapping does
            // not exist; skip it and return None, the operation fails
            if !pte.is_valid() {
                return None;
            }
            // In SV39, each page table walk uses the PPN from the previous
            // level (starting with the root) as the base for the next lookup
            ppn = pte.ppn();
        }
        result
    }
    // Insert and remove key-value pairs in the multi-level page table
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(
            !pte.is_valid(),
            "vpn {:?} is mapped(valid) before mapping",
            vpn
        );
        // Create a new PTE, completing the map operation
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
        // Clear the PTE, completing the unmap operation
        *pte = PageTableEntry::empty();
    }
    // Manually walk the page table to translate a virtual page
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    // Construct the RISC-V satp register value:
    // SV39 paging mode (8), ASID=0, and the root PPN
    // | MODE | ASID | PPN |
    // | 4bit | 16bit | 44bit |
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
