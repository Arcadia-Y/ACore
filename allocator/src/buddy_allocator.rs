use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use super::linked_list::LinkedList;
use core::cmp::{max, min};
use spin::SpinLock;

const BUDDY_LEVEL_COUNT: usize = 32;

pub struct BuddyAllocator {
    pub inner: SpinLock<BuddyAllocatorInner>
}


impl BuddyAllocator {
    pub const fn empty(unit: usize) -> Self {
        Self {
            inner: SpinLock::new(BuddyAllocatorInner::empty(unit))
        }
    }
    pub unsafe fn add_space(&self, start: usize, end: usize) {
        self.inner.lock().add_space(start, end);
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.lock().dealloc(ptr, layout)
    }  
}

pub struct BuddyAllocatorInner {
    // free_blocks[i] is a linked list of free blocks of size 2^i bytes
    free_blocks: [LinkedList; BUDDY_LEVEL_COUNT],
    // minimum unit for allocation is 2^unit bytes
    unit: usize
}

impl BuddyAllocatorInner {
    pub const fn empty(unit: usize) -> Self {
        Self {
            free_blocks: [LinkedList::new(); BUDDY_LEVEL_COUNT],
            unit: if unit > 3 {
                unit
            } else {
                3
            }
        }
    }

    // caller should ensure that [start, end) is allocatable
    pub unsafe fn add_space(&mut self, mut start: usize, mut end: usize) {
        let unit_space = (1 << self.unit) as usize;
        start = (start + unit_space - 1) & !(unit_space - 1);
        end &= !(unit_space - 1);
        while start < end {
            let i = min((end - start).ilog2(), start.trailing_zeros()) as usize;
            self.free_blocks[i].push(start as *mut usize);
            start += 1 << i;
        }
    }

    pub unsafe fn new(start: usize, end: usize, unit: usize) -> Self {
        let mut allocator = Self::empty(unit);
        allocator.add_space(start, end);
        allocator
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let level = self.calc_level(&layout);
        for i in level..BUDDY_LEVEL_COUNT {
            if !self.free_blocks[i].empty() {
                self.split(i, level);
                return self.free_blocks[level].pop().unwrap() as *mut u8;
            }
        }
        panic!("[buddy_allocator] unable to allocate memory for {} bytes", layout.size());
    }

    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let level = self.calc_level(&layout);
        self.merge(ptr, level);
    }

    // split from free_blocks[start] to make space for free_blocks[end]
    fn split(&mut self, start: usize, end: usize) {
        for i in (end..start).rev() {
            let block = self.free_blocks[i+1].pop().unwrap() as usize;
            let buddy = block + (1 << i);
            self.free_blocks[i].push(buddy as *mut usize);
            self.free_blocks[i].push(block as *mut usize);
        }
    }

    // merge from free_blocks[start]
    fn merge(&mut self, ptr: *mut u8, start: usize) {
        let mut cur = ptr as usize;
        for i in start..BUDDY_LEVEL_COUNT {
            let buddy = cur  ^ (1 << i);
            let goal = self.free_blocks[i].iter()
                       .find(|it| it.get() as usize == buddy);
            if let Some(it) = goal {
                it.pop();
                cur = min(cur, buddy);
            } else {
                self.free_blocks[i].push(cur as *mut usize);
                break;
            }
        }
    }

    fn calc_level(&self, layout: &Layout) -> usize {
        max(layout.size().next_power_of_two().trailing_zeros() as usize, 
            max(self.unit, layout.align().trailing_zeros() as usize))
    }
}
