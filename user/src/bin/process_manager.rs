#![no_std]
#![no_main]

use allocator::buddy_allocator::BuddyAllocator;
extern crate user_lib;

use user_lib::syscall::*;

const HEAP_SIZE: usize = 0x80_000;
const HEAP_UNIT: usize = 6;

#[link_section = ".data.heap"]
static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static mut HEAP_ALLOCATOR: BuddyAllocator = BuddyAllocator::empty(HEAP_UNIT);

#[no_mangle]
fn main() -> i32 {
    unsafe {
        let heap_begin = HEAP_SPACE.as_ptr() as usize;
        HEAP_ALLOCATOR
            .inner.lock()
            .add_space(heap_begin, heap_begin + HEAP_SIZE);
    }
    let mut process_manager = ProcessManager::new(2);
    let mut buffer = [0usize; 3];
    recv(buffer.as_mut_ptr(), 3);
    loop {
        match buffer[0] {
            SYSCALL_FORK => {
                process_manager.fork(buffer[1], buffer[2]);
                recv(buffer.as_mut_ptr(), 3);
            },
            SYSCALL_EXIT => {
                process_manager.exit(buffer[1], buffer[2] as i32);
                recv(buffer.as_mut_ptr(), 3);
            },
            SYSCALL_WAITPID => {
                let (pid, exitcode) = process_manager.waitpid(buffer[1], buffer[2] as isize);
                buffer[0] = pid as usize;
                buffer[1] = exitcode as usize;
                sendrecv(buffer.as_ptr(), 2, buffer.as_mut_ptr(), 3);
            },
            _ => {recv(buffer.as_mut_ptr(), 3);},
        }
    }
}

extern crate alloc;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use spin::SpinLock;
#[derive(Copy, Clone, PartialEq)]
pub enum ProcessStatus {
    Live,
    Exit,
}

struct ProcessBlock {
    pub pid: usize,
    pub parent: usize,
    pub children: Vec<Arc<SpinLock<ProcessBlock>>>,
    pub status: ProcessStatus,
    pub exit_code: i32,
}

pub struct ProcessManager {
    initid: usize,
    initproc: Arc<SpinLock<ProcessBlock>>,
    id2proc: BTreeMap<usize, Arc<SpinLock<ProcessBlock>>>,
}

impl ProcessBlock {
    pub fn new(pid: usize, parent: usize) -> Self {
        Self{
            pid,
            parent,
            children: Vec::new(),
            status: ProcessStatus::Live,
            exit_code: 0,
        }
    }
    pub fn add_child(&mut self, child: Arc<SpinLock<ProcessBlock>>) {
        self.children.push(child);
    }
}

impl ProcessManager {
    pub fn new(initid: usize) -> Self {
        let mut id2proc: BTreeMap<usize, Arc<SpinLock<ProcessBlock>>> = BTreeMap::new();
        let initproc = Arc::new(SpinLock::new(ProcessBlock::new(initid, 0)));
        id2proc.insert(initid, initproc.clone());
        Self {
            initid: 0,
            initproc,
            id2proc
        }
    }

    pub fn fork(&mut self, parentid: usize, childid: usize) {
        let parent = self.id2proc.get(&parentid).unwrap();
        let child = Arc::new(SpinLock::new(ProcessBlock::new(childid, parentid)));
        parent.lock().add_child(child.clone());
        self.id2proc.insert(childid, child);
        
    }

    pub fn exit(&mut self, pid: usize, exit_code: i32) {
        let proc = self.id2proc.get(&pid).unwrap();
        let mut inner = proc.lock();
        inner.status = ProcessStatus::Exit;
        inner.exit_code = exit_code;
        let mut initproc = self.initproc.lock();
        for c in inner.children.iter() {
            c.lock().parent = self.initid;
            initproc.add_child(c.clone());
        }
        inner.children.clear();
    }

    // isize is pid of child, = -1 if pid doesn't exist, = -2 if not exit
    // i32 = exitcode
    pub fn waitpid(&mut self, parentid: usize, pid: isize) -> (isize, i32) {
        let mut parent = self.id2proc.get(&parentid).unwrap().lock();
        if parent.children
            .iter()
            .find(|c| {pid == -1 || pid as usize == c.lock().pid})
            .is_none() {
                return (-1, 0)
        }
        let pair = parent.children
            .iter()
            .enumerate()
            .find(|(_, c)|{
                c.lock().status == ProcessStatus::Exit && (pid == -1 || pid as usize == c.lock().pid)
            });
        if let Some((idx, _)) = pair {
            let child = parent.children.remove(idx);
            let inner = child.lock();
            let cpid = inner.pid;
            let exit_code = inner.exit_code;
            drop(parent);
            self.id2proc.remove(&cpid);
            (cpid as isize, exit_code)
        } else {
            (-2, 0)
        }
    }

}

