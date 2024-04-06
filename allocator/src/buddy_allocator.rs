use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use super::linked_list::LinkedList;
use core::cmp::{max, min};

const BUDDY_LEVEL_COUNT: usize = 32;

pub struct BuddyAllocator {
    // free_blocks[i] is a linked list of free blocks of size 2^i bytes
    free_blocks: [LinkedList; BUDDY_LEVEL_COUNT],
    // minimum unit for allocation is 2^unit bytes
    unit: usize
}

impl GlobalAlloc for BuddyAllocator {
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let level = self.calc_level(&layout);
        for i in level..BUDDY_LEVEL_COUNT {
            if !self.free_blocks[i].empty() {
                self.split(i, level);
                return self.free_blocks[level].pop().unwrap() as mut* u8;
            }
        }
        panic!("[buddy_allocator] unable to allocate memory for {} bytes", layout.size());
    }

    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let level = self.calc_level(&layout);
        self.merge(ptr, level);
    }
}

impl BuddyAllocator {
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
        let unit_space = (1 << unit) as usize;
        start = (start + unit_space - 1) & !(unit_space - 1);
        end &= !(unit_space - 1);
        while start < end {
            let i = (end - start).trailing_zeros() as usize;
            self.free_blocks[i].push(start as *mut usize);
            start += 1 << i;
        }
    }

    pub unsafe fn new(start: usize, end: usize, unit: usize) -> Self {
        let mut allocator = Self::empty(unit);
        allocator.add_space(start, end);
        allocator
    }

    // split from free_blocks[start] to make space for free_blocks[end]
    fn split(&self, start: usize, end: usize) {
        for i in end..start.rev() {
            let block = self.free_blocks[i+1].pop().unwrap() as usize;
            let buddy = block + (1 << i);
            unsafe {
                self.free_blocks[i].push(buddy as *mut usize);
                self.free_blocks[i].push(block as *mut usize);
            }
        }
    }

    // merge from free_blocks[start]
    fn merge(&self, ptr: *mut u8, start: usize) {
        let mut cur = ptr;
        for i in start..BUDDY_LEVEL_COUNT {
            let buddy = cur ^ (1 << i);
            let goal = self.free_blocks[i].iter()
                       .find(|it| it.get() as usize == buddy);
            if let Some(it) = goal {
                it.pop();
                curr = min(curr, buddy);
            } else {
                unsafe { self.free_blocks[i].push(cur as *mut usize); }
                break;
            }
        }
    }

    fn calc_level(&self, layout: &Layout) -> usize {
        max(layout.size().next_power_of_two().trailing_zeros(), 
            max(self.unit, layout.align().trailing_zeros())) as usize
    }
}
