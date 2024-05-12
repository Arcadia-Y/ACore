use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::SpinLock;
use crate::trap::context::TrapContext;

use super::{context::TaskContext, fetch_task, switch::__switch, task::{TaskControlBlock, TaskStatus}};


pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    // helper control flow for switching tasks
    idle_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_cx: TaskContext::new(0, 0),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_cx as *mut _
    }
    fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|x| Arc::clone(x))
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
}

lazy_static! {
    pub static ref PROCESSOR: SpinLock<Processor> = SpinLock::new(Processor::new());
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.lock();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            let mut inner = task.inner.lock();
            let next_task_cx_ptr = &inner.task_cx as *const TaskContext;
            inner.task_status = TaskStatus::Running;
            drop(inner);
            processor.current = Some(task);
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().current()
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().take_current()
}

pub fn current_user_satp() -> usize {
    let task = current_task().unwrap();
    task.get_user_satp()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .get_trap_cx()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.lock();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

