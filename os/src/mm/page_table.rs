use alloc::vec:: Vec;
use alloc::vec;
use bitflags::*;
use spin::SpinLock;
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
    frames: SpinLock<Vec<FrameTracker>> 
}

impl PageTable {
    pub fn new() -> Self {
        let root_frame = frame_alloc();
        Self {
            root_ppn: root_frame.ppn,
            frames: SpinLock::new(vec![root_frame])
        }
    }

    pub fn map(&self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn);
        *pte = PageTableEntry::new(ppn, flags);
    }

    pub fn unmap(&self, vpn: VirtPageNum) {
        if let Some(pte) = self.find_pte(vpn) {
            *pte = PageTableEntry::empty();
        } else {
            panic!("The vpn:{} is not valid when unmapping", vpn.0);
        }
    }

    // find vpn's pte, create new pages if necessary
    fn create_pte(&self, vpn: VirtPageNum) -> &mut PageTableEntry {
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
                self.frames.lock().push(frame);
            }
            ppn = pte.ppn();
        }
        unreachable!();
    }

    // find vpn's pte, return none if it's invalid
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
    pub fn from_token(satp: usize) -> Self{
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: SpinLock::new(Vec::new())
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn)
            .map(|pte| {pte.clone()})
    }
}
