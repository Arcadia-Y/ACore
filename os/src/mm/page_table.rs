use alloc::vec:: Vec;
use alloc::vec;
use bitflags::*;
use core::cmp::min;
use crate::{config::PAGE_SIZE, print};

use super::{address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum}, frame_allocator::{frame_alloc, FrameTracker}};

bitflags! {
    pub struct PTEFlags: u8 {
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
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
}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker> 
}

impl PageTable {
    pub fn new() -> Self {
        let root_frame = frame_alloc();
        Self {
            root_ppn: root_frame.ppn,
            frames: vec![root_frame]
        }
    }

    pub fn  get_satp(&self) -> usize {
        self.root_ppn.0 | (8usize << 60) // set MODE = 8 for SV39
    }

    // NOTE that map() ensures that the mapped pte is valid
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&self, vpn: VirtPageNum) {
        if let Some(pte) = self.find_pte(vpn) {
            *pte = PageTableEntry::empty();
        } else {
            panic!("[page table] The vpn:{} is not valid when unmapping", vpn.0);
        }
    }

    // find vpn's pte, create new pages if necessary
    fn create_pte(&mut self, vpn: VirtPageNum) -> &mut PageTableEntry {
        let index = vpn.get_index();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[index[i]];
            if i == 2 {
                return pte;
            }
            if !pte.is_valid() {
                let frame = frame_alloc();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        unreachable!();
    }

    // find vpn's pte, return none if its page table is invalid
    // however even if !pte.is_valid() we still return Some(pte)
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let index = vpn.get_index();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[index[i]];
            if i == 2 {
                return Some(pte);
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        unreachable!();
    }
}

// used for kernel to access user's page
impl PageTable {
    pub fn from_satp(satp: usize) -> Self{
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new()
        }
    }

    pub fn translate_vpn(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    }

    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        if let Some(pte) = self.translate_vpn(va.floor()) {
            if !pte.is_valid() {
                return None;
            }
            let ppn = pte.ppn();
            let offset = va.offset();
            Some(PhysAddr(ppn.0 * PAGE_SIZE + offset))
        } else {
            None
        }
    }
}

pub fn get_user_byte_buffer(satp: usize, ptr: *const u8, len: usize) -> Vec<u8> {
    let mut res = Vec::new();
    let mut start = ptr as usize;
    let end = start + len;
    let page_table = PageTable::from_satp(satp);
    while start < end {
        let strat_va = VirtAddr(start);
        let vpn = strat_va.floor();
        let ppn = page_table.translate_vpn(vpn).unwrap().ppn();
        let offset = strat_va.offset();
        let amount = min(end - start, PAGE_SIZE - offset);
        let bytes = ppn.get_bytes_array();
        for byte in bytes[offset..offset + amount].iter() {
            res.push(*byte);
        }
        start += amount;
    }
    res
}

pub fn copy_bytes_to_user(satp: usize, src: *const u8, dst: usize, len: usize) {
    let mut start = dst;
    let end = start + len;
    let page_table = PageTable::from_satp(satp);
    let mut src_ptr = src;
    while start < end {
        let strat_va = VirtAddr(start);
        let vpn = strat_va.floor();
        let ppn = page_table.translate_vpn(vpn).unwrap().ppn();
        let offset = strat_va.offset();
        let amount = min(end - start, PAGE_SIZE - offset);
        let bytes = ppn.get_bytes_array();
        for i in 0..amount {
            bytes[offset + i] = unsafe { *src_ptr.add(i) };
        }
        start += amount;
        src_ptr = unsafe { src_ptr.add(amount) };
    }
}

pub fn translate_refmut<T>(satp: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_satp(satp);
    let va = ptr as usize;
    page_table
        .translate_va(VirtAddr(va))
        .unwrap()
        .get_mut()
}

impl PageTable {
    #[allow(unused)]
    // only for debug
    pub fn show_frames(&self) {
        for frame in self.frames.iter() {
            print!("{} ", frame.ppn.0);
        }
        print!("\n");
    }
}
