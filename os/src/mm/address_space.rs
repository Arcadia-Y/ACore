use core::arch::asm;

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use lazy_static::*;
use spin::SpinLock;
use crate::{config::*, println};

use super::{address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}, frame_allocator::{frame_alloc, FrameTracker}, page_table::{PTEFlags, PageTable}, range::Range};

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
    fn _trampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: SpinLock<AddrSpace> = SpinLock::new(AddrSpace::new_kernel());
}

pub fn set_up_page_table() {
    // note that KERNEL_SPACE has been initialized due to lazy_static
    let table = &KERNEL_SPACE.lock().root_table;
    let satp = table.get_satp();
    println!("Ready to write satp.");
    unsafe {
        riscv::register::satp::write(satp);
        asm!("sfence.vma");
    }
}

pub struct AddrSpace {
    pub root_table: PageTable,
    areas: Vec<MapArea>,
}

impl AddrSpace {
    pub fn new_empty() -> Self {
        Self {
            root_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    fn map_trampoline(&mut self) {
        self.root_table.map(
            VirtAddr(TRAMPOLINE_ADDR).floor(), 
            PhysAddr(_trampoline as usize).floor(), 
            PTEFlags::R | PTEFlags::X
        );
    }

    pub fn push(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.root_table);
        if let Some(data) = data {
            area.copy_from_bytes(data);
        }
        self.areas.push(area);
    }

    pub fn new_kernel() -> Self {
        let mut ret = Self::new_empty();
        ret.map_trampoline();

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
        // map VIRT_TEST
        ret.push( 
            MapArea::new(
                VIRT_TEST.into(), 
                (VIRT_TEST + 1).into(), 
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

    pub fn new_user(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut user_space = Self::new_empty();
        user_space.map_trampoline();
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
                let mut perm = PTEFlags::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    perm |= PTEFlags::R;
                }
                if ph_flags.is_write() {
                    perm |= PTEFlags::W;
                }
                if ph_flags.is_execute() {
                    perm |= PTEFlags::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, perm);
                max_end_vpn = map_area.range.end;
                user_space.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack
        let max_end_va: VirtAddr = max_end_vpn.to_addr();
        let mut user_stack_bottom: usize = max_end_va.into();
        user_stack_bottom += PAGE_SIZE; // guard page
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        user_space.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                PTEFlags::R | PTEFlags::W | PTEFlags::U,
            ),
            None,
        );
        // map TrapContext
        user_space.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE_ADDR.into(),
                MapType::Framed,
                PTEFlags::R | PTEFlags::W,
            ),
            None,
        );
        (
            user_space,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
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
            MapType::Identical => PhysPageNum(vpn.0),
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
                let ppn: PhysPageNum = PhysPageNum(vpn.0);
                let dst = &mut ppn.get_bytes_array()[..src.len()];
                dst.copy_from_slice(src);
                head += PAGE_SIZE;
                if head >= len {
                    break;
                }
            }
        } else {
            for frame in self.frame_map.values() {
                let src = &data[head..len.min(head + PAGE_SIZE)];
                let dst = &mut frame.ppn.get_bytes_array()[..src.len()];
                dst.copy_from_slice(src);
                head += PAGE_SIZE;
                if head >= len {
                    break;
                }
            }
        }
    }

}
