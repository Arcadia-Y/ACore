use alloc::sync::Arc;
use riscv::register::sstatus;
use spin::SpinLock;

use crate::{mm::{address::{PhysPageNum, VirtAddr}, address_space::{AddrSpace, KERNEL_SPACE}}, trap::{context::TrapContext, trap_handler, trap_return}};
use super::{context::TaskContext, id::{alloc_task_id, IdTracker, KernelStack}, scheduler::Priority};
use crate::config::*;


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Block,
    Exit,
}

pub struct TaskControlBlock {
    pub taskid: IdTracker,
    pub kernel_stack: KernelStack,
    pub priority: Priority,
    pub inner: SpinLock<TaskControlBlockInner>,
}


pub struct TaskControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub user_space: AddrSpace,
    pub trap_cx_ppn: PhysPageNum,
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.inner.lock().trap_cx_ppn.get_mut()
    }
    pub fn get_user_satp(&self) -> usize {
        self.inner.lock().user_space.root_table.get_satp()
    }

    pub fn new(elf_data: &[u8], priority: Priority) -> Self {
        let (user_space, user_stack_top, entry_point) = AddrSpace::new_user(elf_data);
        let trap_cx_ppn = user_space.root_table
                         .translate_vpn(VirtAddr(TRAP_CONTEXT).floor())
                         .unwrap().ppn();
        let task_status = TaskStatus::Ready;
        // set up kernel stack
        let id_tracker = alloc_task_id();
        let kernel_stack = KernelStack::new(&id_tracker);
        let kernel_stack_top = kernel_stack.get_top();
        let inner = SpinLock::new(TaskControlBlockInner{
            task_status,
            task_cx: TaskContext::new(trap_return as usize, kernel_stack_top),
            user_space,
            trap_cx_ppn,
        });
        let control_block = Self{
            taskid: id_tracker,
            kernel_stack,
            priority,
            inner
        };
        let trap_cx = control_block.get_trap_cx();
        *trap_cx = TrapContext::new(
            user_stack_top,
            entry_point,
            sstatus::read().bits(),
            KERNEL_SPACE.lock().root_table.get_satp(),
            trap_handler as usize,
            kernel_stack_top,
        );
        control_block
    }   

    pub fn fork(self: &Arc<Self>) -> Arc<TaskControlBlock> {
        let parent_inner = self.inner.lock();
        // create child addrspace 
        let user_space = AddrSpace::from_existed_user(&parent_inner.user_space);
        let trap_cx_ppn = user_space.root_table
                         .translate_vpn(VirtAddr(TRAP_CONTEXT).floor())
                         .unwrap().ppn();
        // alloc taskid and kernel stack
        let id_tracker = alloc_task_id();
        let kernel_stack = KernelStack::new(&id_tracker);
        let kernel_stack_top = kernel_stack.get_top();
        let block = Arc::new(TaskControlBlock{
            taskid: id_tracker,
            kernel_stack,
            priority: self.priority,
            inner: SpinLock::new(TaskControlBlockInner{
                task_status: TaskStatus::Ready,
                task_cx: TaskContext::new(trap_return as usize, kernel_stack_top),
                user_space,
                trap_cx_ppn,
            })
        });
        let trap_cx = block.get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        let (user_space, user_sp, entry_point) = AddrSpace::new_user(elf_data);
        let trap_cx_ppn = user_space.root_table
                         .translate_vpn(VirtAddr(TRAP_CONTEXT).floor())
                         .unwrap().ppn();
        let mut inner = self.inner.lock();
        inner.user_space = user_space;
        inner.trap_cx_ppn = trap_cx_ppn;
        let trap_cx = inner.trap_cx_ppn.get_mut();
        *trap_cx = TrapContext::new(
            user_sp,
            entry_point,
            sstatus::read().bits(),
            KERNEL_SPACE.lock().root_table.get_satp(),
            trap_handler as usize,
            self.kernel_stack.get_top(),
        );
    }
}
