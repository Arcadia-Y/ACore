use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use crate::{ipc::rpc::RPC_BUFFER, loader::get_app_data_by_name};
use self::{context::TaskContext, processor::{current_task, schedule, take_current_task}, scheduler::SCHEDULER, task::{TaskControlBlock, TaskStatus}};
mod context;
pub mod task; 
mod scheduler;
mod switch;
mod id;
pub mod processor;

lazy_static!{
    pub static ref PROCESS_MANAGER: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("process_manager").unwrap(), scheduler::Priority::SERVICE
    ));
    pub static ref INIT: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("init").unwrap(), scheduler::Priority::USER
    ));
}

pub fn add_service() {
    add_task(PROCESS_MANAGER.clone());
}

pub fn add_init() {
    add_task(INIT.clone());
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    SCHEDULER.lock().add_task(task);
}

pub fn push_task(task: Arc<TaskControlBlock>) {
    SCHEDULER.lock().push_task(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    SCHEDULER.lock().fetch_task()
}

pub fn id2task(id: usize) -> Option<Arc<TaskControlBlock>> {
    SCHEDULER.lock().id2task(id)
}

pub fn recycle_id(id: usize) {
    SCHEDULER.lock().recycle_id(id);
}

#[allow(unused)]
// only for debug
pub fn show_task_frames() {
    SCHEDULER.lock().show_task_frames();
}

pub fn rpc_call(calleeid: usize, args: Vec<usize>) {
    let mut rpc = RPC_BUFFER.lock();
    // save current caller
    let caller = rpc.caller.take();
    let current = current_task().unwrap();
    // let caller be current task
    rpc.caller = Some(current);
    rpc.callee = id2task(calleeid);
    rpc.data = args;
    drop(rpc);
    block_current_and_run_next();
    // now back to current task
    let mut rpc = RPC_BUFFER.lock();
    rpc.caller = caller;
}

pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut inner = task.inner.lock();
    let task_cx_ptr = &mut inner.task_cx as *mut TaskContext;
    inner.task_status = TaskStatus::Ready;
    drop(inner);
    if RPC_BUFFER.lock().callee.is_none() {
        push_task(task);
    }
    schedule(task_cx_ptr);
}

pub fn block_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut inner = task.inner.lock();
    let task_cx_ptr = &mut inner.task_cx as *mut TaskContext;
    inner.task_status = TaskStatus::Block;
    drop(inner);
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut inner = task.inner.lock();
    inner.task_status = TaskStatus::Exit;
    drop(inner);
    drop(task);
    let mut _unused = TaskContext::new(0, 0);
    schedule(&mut _unused as *mut _);
}
