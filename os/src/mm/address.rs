// Implementation of physical and virtual address

use crate::config::*;

const PA_WIDTH: usize = 56;
const VA_WIDTH: usize = 39;
const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

// from/into usize for T (address and page number)
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PA_WIDTH) - 1))
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VA_WIDTH) - 1))
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH) - 1))
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH) - 1))
    }
}

impl Into<usize> for PhysAddr {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<usize> for VirtAddr {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<usize> for PhysPageNum {
    fn into(self) -> usize {
        self.0
    }
}

impl Into<usize> for VirtPageNum {
    fn into(self) -> usize {
        self.0
    }
}

// utilities
impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> PAGE_SIZE_BITS)
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_SIZE_BITS)
    }
}

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 >> PAGE_SIZE_BITS)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_SIZE_BITS)
    }
}

impl PhysPageNum {
    pub fn to_addr(&self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_SIZE_BITS)
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.to_addr();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
}
