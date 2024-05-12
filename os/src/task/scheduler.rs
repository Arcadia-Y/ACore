use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::SpinLock;

use crate::ipc::rpc::RPC_BUFFER;

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
        self.queue[task.priority as usize].push_back(task.clone());
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
}

lazy_static!{
    pub static ref SCHEDULER: SpinLock<Scheduler> = SpinLock::new(Scheduler::new());
}
