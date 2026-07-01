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
    /// 创建内核地址空间的全局实例
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = Arc::new(unsafe{
        UPSafeCell::new(MemorySet::new_kernel())
    });
}

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
            // 往后取最多一页数据,不满一页就直接取len就行
            let src = &data[start..len.min(start + PAGE_SIZE)];
            // 在vpn上取一块和src大小相同的dst数据
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            // 如果start大于数据总大小,就直接跳出循环,拷贝完成
            if start >= len {
                break;
            }
            // 向后步进一页
            current_vpn.step();
        }
    }
}

/// memory set structure, controls virtual-memory space
/// 地址空间是一段有关联但不一定连续的逻辑段,
/// 这种关联一般是指这些逻辑段组成的虚拟地址空间和
/// 一个运行的程序绑定
/// NOTE: 一个运行的应用程序对代码和数据的直接访问限制
/// 在它关联的虚拟地址空间之内,这个地址空间就叫应用程序的地址空间
pub struct MemorySet {
    page_table: PageTable, // 该地址空间的多级页表
    areas: Vec<MapArea>,
}

// TODO: (done)6.28日一直在实现这个结构体,(ch4-内核与应用的地址空间)后面继续完善
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
    /// 映射页表,构造地址空间
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// 把trampoline这段特殊代码映射进当前地址空间的页表里,但他不属于
    /// 普通的memory areas管理体系
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(trampoline as *const () as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    // NOTE: 6.29日完成了new_kernel()方法的实现
    // 生成kernel的地址空间
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
    // 根据一份ELF可执行文件,给用户程序创建一套完整的地址空间布局
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
                // elf.input其实就是输出elf的原始字节
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
        // 构造用户栈地址空间
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
    // 激活地址空间管理
    pub fn active(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            // 因为内核页表建立后执行恒等映射,所以在开启页表后仍然能
            // 正确访问内核地址获取指令
            asm!("sfence.vma");
        }
    }
    // 调用mm::init()后我们使能了内核动态内存分配,物理页帧管理
    // ,还启用了分页模式进入了内核地址空间,之后通过mm::remap_test
    // 来检查内核地址空间的多级页表是否被正确设置
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
