use super::address::PhysPageNum;
use lazy_static::*;
use super::stack_frame_allocator::StackFrameAllocator;
use crate::config::MEMORY_END;
use super::address::PhysAddr;

pub trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&self) -> PhysPageNum;
    fn dealloc(&self, ppn: PhysPageNum);
}

// bind PhysPageNum with FrameTracker for RAII
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // clean the page
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: StackFrameAllocator = StackFrameAllocator::new();
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
        .init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}

pub fn frame_alloc() -> FrameTracker {
    FrameTracker::new(FRAME_ALLOCATOR.alloc())
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.dealloc(ppn);
}
