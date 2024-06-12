use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::SpinLock;

use crate::ipc::rpc::RPC_BUFFER;
use crate::println;

use super::task::TaskControlBlock;
#[derive(Clone, Copy)]
pub enum Priority {
    SERVICE,
    USER,
}

pub struct Scheduler {
    queue: [VecDeque<Arc<TaskControlBlock>>; 2],
    id2task: BTreeMap<usize, Arc<TaskControlBlock>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            queue: [VecDeque::new(), VecDeque::new()],
            id2task: BTreeMap::new(),
        }
    }

    pub fn add_task(&mut self, task: Arc<TaskControlBlock>) {
        self.queue[task.priority as usize].push_back(task.clone());
        self.id2task.insert(task.taskid.0, task);
    }

    pub fn push_task(&mut self, task: Arc<TaskControlBlock>) {
        self.queue[task.priority as usize].push_back(task);
    }

    pub fn fetch_task(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut rpc = RPC_BUFFER.lock();
        if let Some(callee) = rpc.callee.clone() {
            return Some(callee)
        }
        if let Some(caller) = rpc.caller.take() {
            return Some(caller)
        }
        for queue in self.queue.iter_mut() {
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }
        None
    }

    pub fn id2task(&self, id: usize) -> Option<Arc<TaskControlBlock>> {
        self.id2task.get(&id).cloned()
    }

    pub fn recycle_id(&mut self, id: usize) {
        self.id2task.remove(&id);
    }

    pub fn show_task_frames(&self) {
        for task in self.id2task.values() {
            println!("task {} frames:", task.taskid.0);
            task.inner.lock().user_space.root_table.show_frames();
        }
    }
}

lazy_static!{
    pub static ref SCHEDULER: SpinLock<Scheduler> = SpinLock::new(Scheduler::new());
}
