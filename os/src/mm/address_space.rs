use core::arch::asm;

use alloc::{collections::BTreeMap, vec::Vec};
use lazy_static::*;
use crate::config::*;

use super::{address::{PhysPageNum, VirtAddr, VirtPageNum}, frame_allocator::{frame_alloc, FrameTracker}, page_table::{PTEFlags, PageTable}, range::Range};

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
}

lazy_static! {
    pub static ref KERNEL_SPACE: AddrSpace = AddrSpace::new_kernel();
}

pub fn set_up_page_table() {
    // note that KERNEL_SPACE has been initialized due to lazy_static
    let satp = KERNEL_SPACE.get_satp();
    riscv::register::satp::write(satp);
    unsafe {
        asm!("sfence.vma");
    }
}

pub struct AddrSpace {
    root_table: PageTable,
    areas: Vec<MapArea>,
}

impl AddrSpace {
    pub fn new_empty() -> Self {
        Self {
            root_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn push(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.root_table);
        if let Some(data) = data {
            area.copy_from_bytes(data);
        }
        self.areas.push(area);
    }

    pub fn get_satp(&self) -> usize {
        self.root_table.get_satp()
    }

    pub fn new_kernel() -> Self {
        // TODO: add trampoline
        let mut ret = Self::new_empty();

        // map .text
        ret.push( 
            MapArea::new(
                (stext as usize).into(), 
                (etext as usize).into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::X),
            None
        );
        // map .rodata
        ret.push( 
            MapArea::new(
                (srodata as usize).into(), 
                (erodata as usize).into(), 
                MapType::Identical,
                PTEFlags::R),
            None
        );
        // map .data
        ret.push( 
            MapArea::new(
                (sdata as usize).into(), 
                (edata as usize).into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::W),
            None
        );
        // map .bss
        ret.push( 
            MapArea::new(
                (sbss_with_stack as usize).into(), 
                (ebss as usize).into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::W),
            None
        );
        // map physical memory
        ret.push( 
            MapArea::new(
                (ekernel as usize).into(), 
                MEMORY_END.into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::W),
            None
        );
        // map UART port
        ret.push( 
            MapArea::new(
                UART_BASE.into(), 
                (UART_BASE + UART_SIZE).into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::W),
            None
        );
        // map timer port
        ret.push( 
            MapArea::new(
                MTIME.into(), 
                (MTIME+1).into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::W),
            None
        );
        ret.push( 
            MapArea::new(
                MTIMECMP.into(), 
                (MTIMECMP+1).into(), 
                MapType::Identical,
                PTEFlags::R | PTEFlags::W),
            None
        );

        ret
    }
}

pub struct MapArea {
    range: Range<VirtPageNum>,
    frame_map: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    perm: PTEFlags,
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for MapArea: identical or framed
pub enum MapType {
    Identical,
    Framed,
}

impl MapArea {
    pub fn new(start: VirtAddr, end: VirtAddr, map_type: MapType, perm: PTEFlags) -> Self {
        let l = start.floor();
        let r = end.ceil();
        Self {
            range: Range::new(l, r),
            frame_map: BTreeMap::new(),
            map_type,
            perm
        }
    }

    pub fn map_one(&mut self, table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = match self.map_type {
            MapType::Identical => vpn.0.into(),
            MapType::Framed => {
                let frame = frame_alloc();
                let res = frame.ppn;
                self.frame_map.insert(vpn, frame);
                res
            }
        };
        table.map(vpn, ppn, self.perm);
    }

    pub fn unmap_one(&mut self, table: &mut PageTable, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            self.frame_map.remove(&vpn);
        }
        table.unmap(vpn);
    }

    pub fn map(&mut self, table: &mut PageTable) {
        for vpn in self.range.iter() {
            self.map_one(table, vpn);
        }
    }

    pub fn unmap(&mut self, table: &mut PageTable) {
        for vpn in self.range.iter() {
            self.unmap_one(table, vpn);
        }
    }

    pub fn copy_from_bytes(&mut self, data: &[u8]) {
        let mut head = 0;
        let len = data.len();
        if self.map_type == MapType::Identical {
            for vpn in self.range.iter() {
                let src = &data[head..len.min(head + PAGE_SIZE)];
                let ppn: PhysPageNum = vpn.0.into();
                let dst = ppn.get_bytes_array();
                dst.copy_from_slice(src);
                head += PAGE_SIZE;
                if head >= len {
                    break;
                }
            }
        } else {
            for frame in self.frame_map.values() {
                let src = &data[head..len.min(head + PAGE_SIZE)];
                let dst = frame.ppn.get_bytes_array();
                dst.copy_from_slice(src);
                head += PAGE_SIZE;
                if head >= len {
                    break;
                }
            }
        }
    }

}
