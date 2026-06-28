use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use bitflags::bitflags;

use crate::{
    config::PAGE_SIZE,
    mm::{
        address::{PhysPageNum, StepByOne, VPNRange, VirtAddr, VirtPageNum},
        frame_allocator::{FrameTracker, frame_alloc},
        page_table::{self, PageTable},
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

// TODO: 6.28日一直在实现这个结构体,(ch4-内核与应用的地址空间)后面继续完善
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
}
