use alloc::vec::Vec;
use spin::SpinLock;
use crate::{config::TIME_INTERVAL, drivers::shutdown, println, time::{get_time, set_timer}, trap::{context::TrapContext, trap_return}};

use super::{context::TaskContext, task::{TaskControlBlock, TaskStatus}};

pub struct TaskManager {
    app_num: usize,
    inner: SpinLock<TaskManagerInner>,
}

pub struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current: usize,
}

extern "C" {
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}   

impl TaskManager {
    pub fn new(app_num: usize, tasks: Vec<TaskControlBlock>) -> Self {
        Self {
            app_num,
            inner: SpinLock::new(TaskManagerInner {
                tasks,
                current: 0,
            }),
        }
    }

    pub fn run_first_task(&self) -> ! {
        let mut inner = self.inner.lock();
        let first_task = &mut inner.tasks[0];
        first_task.task_status = TaskStatus::Running;
        let mut _empty= TaskContext::new(0, 0);
        let first_task_cx = &first_task.task_cx as *const TaskContext;
        unsafe {
            let ra = (*first_task_cx).ra;
            assert_eq!(trap_return as usize, ra);
        }
        drop(inner);
        println!("begin to run first task.");
        unsafe {
            __switch(&mut _empty as *mut _, first_task_cx);
        };
        unreachable!();
    }

    pub fn find_next_task(&self, current_state: TaskStatus) -> Option<usize> {
        let inner = self.inner.lock();
        let current = inner.current;
        let mut next = (current + 1) % self.app_num;
        while next != current {
            if inner.tasks[next].task_status == TaskStatus::Ready {
                return Some(next);
            }
            next = (next + 1) % self.app_num;
        }
        if current_state == TaskStatus::Ready {
            Some(current)
        } else {
            None
        }
    }

    pub fn mark_current(&self, status: TaskStatus) {
        let mut inner = self.inner.lock();
        let current = inner.current;
        inner.tasks[current].task_status = status;
    }

    pub fn run_next_task(&self, current_state: TaskStatus) {
        if let Some(next ) = self.find_next_task(current_state) {
            let mut inner = self.inner.lock();
            inner.tasks[next].task_status = TaskStatus::Running;
            let current = inner.current;
            inner.current = next;
            let current_cx = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_cx = & inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            set_timer(get_time() + TIME_INTERVAL);
            unsafe {
                __switch(current_cx, next_cx);
            }
        } else {
            println!("finish all tasks!");
            shutdown();
        }
    }

    pub fn get_current_satp(&self) -> usize {
        let inner = self.inner.lock();
        inner.tasks[inner.current].user_space.root_table.get_satp()
    }

    pub fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.lock();
        inner.tasks[inner.current].get_trap_cx()
    }

}
