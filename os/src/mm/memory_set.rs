use alloc::vec::Vec;
use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use bitflags::bitflags;
use core::arch::asm;
use lazy_static::lazy_static;
use riscv::register::satp;

unsafe extern "C" {
    safe fn stext();
    safe fn etext();
    safe fn srodata();
    safe fn erodata();
    safe fn sdata();
    safe fn edata();
    safe fn sbss_with_stack();
    safe fn ebss();
    safe fn ekernel();
    safe fn trampoline();
}

lazy_static! {
    /// Create the global kernel address space instance
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = Arc::new(unsafe{
        UPSafeCell::new(MemorySet::new_kernel())
    });
}

use crate::mm::page_table::PageTableEntry;
use crate::sync::UPSafeCell;
use crate::{
    boards::{MEMORY_END, MMIO},
    config::{PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE},
    mm::{
        address::{PhysAddr, PhysPageNum, StepByOne, VPNRange, VirtAddr, VirtPageNum},
        frame_allocator::{FrameTracker, frame_alloc},
        page_table::{self, PTEFlags, PageTable},
    },
};

#[derive(Copy, Clone, PartialEq, Debug)]
/// map_type for memory_set: identical or framed
pub enum MapType {
    Identical,
    Framed,
}
bitflags! {
/// map permission corresponding to that in PTE: `R W X U`
    pub struct MapPermission : u8 {
        const R = 1<<1;
        const W = 1<<2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}
impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            // Take at most one page of data at a time
            let src = &data[start..len.min(start + PAGE_SIZE)];
            // Get a mutable slice of same size as src from the destination VPN
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            // If start exceeds total data length, break out — copy complete
            if start >= len {
                break;
            }
            // Advance to the next page
            current_vpn.step();
        }
    }
}

/// memory set structure, controls virtual-memory space
///
/// An address space consists of logical segments that are related but not
/// necessarily contiguous. This relationship typically means that the virtual
/// address space formed by these segments is bound to a running program.
///
/// NOTE: A running application's direct access to code and data is confined
/// to its associated virtual address space — this is the application's address space.
pub struct MemorySet {
    page_table: PageTable, // The multi-level page table for this address space
    areas: Vec<MapArea>,
}

// TODO: (done) Spent 6.28 implementing this struct; continue from ch4-kernel
// and application address spaces.
impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// Assume that no conflicts
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }
    /// Map pages and construct the address space
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// Map the trampoline code into the page table of the current address space.
    /// The trampoline is not part of the normal memory areas management.
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(trampoline as *const () as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    // NOTE: new_kernel() implementation completed on 6.29
    // Build the kernel's address space
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(
            ".text [{:#x}, {:#x})",
            stext as *const () as usize, etext as *const () as usize
        );
        println!(
            ".rodata [{:#x}, {:#x})",
            srodata as *const () as usize, erodata as *const () as usize
        );
        println!(
            ".data [{:#x}, {:#x})",
            sdata as *const () as usize, edata as *const () as usize
        );
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as *const () as usize, ebss as *const () as usize
        );
        println!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as *const () as usize).into(),
                (etext as *const () as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as *const () as usize).into(),
                (erodata as *const () as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as *const () as usize).into(),
                (edata as *const () as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as *const () as usize).into(),
                (ebss as *const () as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as *const () as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping memory-mapped registers");
        for pairs in MMIO {
            memory_set.push(
                MapArea::new(
                    pairs.0.into(),
                    (pairs.0 + pairs.1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        memory_set
    }
    // TODO: 6.30 add nothing
    // Including sections in elf and trampoline and TrapContext and user stack,
    // also returns user_sp and entry point.
    // pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize);
    // Build a complete address space layout for a user program from
    // an ELF executable
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map program headers of elf, eith U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                // elf.input contains the raw ELF bytes
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // add guard page
        user_stack_bottom += PAGE_SIZE;
        // Set up the user stack address space
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // used in sbrk
        memory_set.push(
            MapArea::new(
                user_stack_top.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        // map TrapContext
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
    // Activate the address space management
    pub fn active(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            // Since the kernel page table uses identity mapping, after paging
            // is enabled we can still correctly access kernel addresses to
            // fetch instructions
            asm!("sfence.vma");
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    // After calling mm::init(), we have enabled kernel dynamic memory allocation,
    // physical frame management, and switched to paging mode to enter the kernel
    // address space. Then mm::remap_test verifies that the kernel's multi-level
    // page tables are set up correctly.
    #[allow(unused)]
    pub fn remap_test() {
        let mut kernel_space = KERNEL_SPACE.exclusive_access();
        let mid_text: VirtAddr =
            ((stext as *const () as usize + etext as *const () as usize) / 2).into();
        let mid_rodata: VirtAddr =
            ((srodata as *const () as usize + erodata as *const () as usize) / 2).into();
        let mid_data: VirtAddr =
            ((sdata as *const () as usize + edata as *const () as usize) / 2).into();
        assert!(
            !kernel_space
                .page_table
                .translate(mid_text.floor())
                .unwrap()
                .writable(),
        );
        assert!(
            !kernel_space
                .page_table
                .translate(mid_rodata.floor())
                .unwrap()
                .writable(),
        );
        assert!(
            !kernel_space
                .page_table
                .translate(mid_data.floor())
                .unwrap()
                .executable(),
        )
    }
}
