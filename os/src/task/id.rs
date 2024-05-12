use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::SpinLock;
use crate::{config::*, mm::{address::VirtAddr, address_space::{MapArea, MapType, KERNEL_SPACE}, page_table::PTEFlags}};

pub struct TaskidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

pub struct IdTracker(pub usize);

impl TaskidAllocator {
    pub fn new() -> Self {
        Self {
            current: 1,
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> IdTracker {
        if let Some(id) = self.recycled.pop() {
            IdTracker(id)
        } else {
            let id = self.current;
            self.current += 1;
            IdTracker(id)
        }
    }

    pub fn dealloc(&mut self, id: usize) {
        self.recycled.push(id);
    }
}

lazy_static! {
    pub static ref TASK_ID_ALLOCATOR: SpinLock<TaskidAllocator> = SpinLock::new(TaskidAllocator::new());
}

pub fn alloc_task_id() -> IdTracker {
    TASK_ID_ALLOCATOR.lock().alloc()
}

impl Drop for IdTracker {
    fn drop(&mut self) {
        TASK_ID_ALLOCATOR.lock().dealloc(self.0);
    }
}

pub fn kernel_stack_pos(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE_ADDR - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE); // guard page
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

pub struct KernelStack {
    taskid: usize
}

impl KernelStack {
    pub fn new(id_tracker: &IdTracker) -> Self {
        let id = id_tracker.0;
        let (bottom, top) = kernel_stack_pos(id);
         KERNEL_SPACE.lock().push(
            MapArea::new(
                VirtAddr(bottom),
                VirtAddr(top),
                MapType::Framed,
                PTEFlags::R | PTEFlags::W,
            ),
            None
         );
        KernelStack {
            taskid: id_tracker.0
        }
    }

    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_pos(self.taskid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (bottom, _) = kernel_stack_pos(self.taskid);
        let bottom_vpn = VirtAddr(bottom).floor();
        KERNEL_SPACE.lock().remove_area(bottom_vpn);
    }
}
