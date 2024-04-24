use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::{loader::{get_app_data, get_num_app}, println, trap::context::TrapContext};
use manager::TaskManager;

use self::task::TaskStatus;
mod context;
mod task; 
mod manager;
mod switch;

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("Initializing TaskManager...");
        let num_app = get_num_app();
        let mut tasks = Vec::new();
        for i in 0..num_app {
            tasks.push(task::TaskControlBlock::new(i, get_app_data(i)));
        }
        println!("Initialized TaskManager.");
        TaskManager::new(num_app, tasks)
    };
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current(TaskStatus::Exit);
    TASK_MANAGER.run_next_task(TaskStatus::Exit);
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current(TaskStatus::Ready);
    TASK_MANAGER.run_next_task(TaskStatus::Ready);
}

pub fn current_user_satp() -> usize {
    TASK_MANAGER.get_current_satp()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}
