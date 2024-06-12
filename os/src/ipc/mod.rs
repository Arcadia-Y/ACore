use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::SpinLock;
use crate::task::task::TaskControlBlock;

pub struct RpcBuffer {
    pub caller: Option<Arc<TaskControlBlock>>,
    pub callee: Option<Arc<TaskControlBlock>>,
    pub sender: usize,
    pub data: Vec<usize>,  
}

impl RpcBuffer {
    pub fn new() -> Self {
        Self {
            caller: None,
            callee: None,
            sender: 0,
            data: Vec::new()
        }
    }
}

lazy_static!{
    pub static ref RPC_BUFFER: SpinLock<RpcBuffer> = SpinLock::new(RpcBuffer::new());
}
