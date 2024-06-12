use super::address::PhysPageNum;
use alloc::vec::Vec;
use spin::SpinLock;
use super::frame_allocator::FrameAllocator;

pub struct StackFrameAllocator {
    inner: SpinLock<StackFrameAllocatorInner>
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            inner: SpinLock::new(StackFrameAllocatorInner::new())
        }
    }

    fn alloc(&self) -> PhysPageNum {
        let mut inner = self.inner.lock();
        if let Some(ppn) = inner.recycled.pop() {
            ppn.into()
        } else {
            if inner.current < inner.end {
                let ppn = inner.current;
                inner.current += 1;
                ppn.into()
            } else {
                panic!("[frame allocator] out of memory!")
            }
        }
    }

    fn dealloc(&self, ppn: PhysPageNum) {
        let mut inner = self.inner.lock();
        // validity check
        if ppn.0 >= inner.current || inner.recycled.iter().any(|&v| v == ppn.0) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn.0);
        }
        inner.recycled.push(ppn.0);
    }
}

impl StackFrameAllocator {
    pub fn init(&self, start: PhysPageNum, end: PhysPageNum) {
        self.inner.lock().init(start, end);
    }
}
pub struct StackFrameAllocatorInner {
    current: usize,
    end: usize,
    recycled: Vec<usize>
}

impl StackFrameAllocatorInner {
    pub fn new() -> Self {
        StackFrameAllocatorInner {
            current: 0,
            end: 0,
            recycled: Vec::new()
        }
    }

    pub fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current = start.0;
        self.end = end.0;
    }
}
