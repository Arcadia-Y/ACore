use allocator::buddy_allocator::BuddyAllocator;
use core::alloc::GlobalAlloc;

const KERNEL_HEAP_SIZE: usize = 0x800_000;
const KERNEL_HEAP_UNIT: usize = 8;

#[link_section = ".data.heap"]
static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static mut KERNEL_HEAP_ALLOCATOR: BuddyAllocator = BuddyAllocator::empty(KERNEL_HEAP_UNIT);

pub fn init_kernel_heap() {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        let end = start + KERNEL_HEAP_SIZE;
        KERNEL_HEAP_ALLOCATOR.add_space(start, end);
    }
}
