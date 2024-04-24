use riscv::register::sstatus;

use crate::{mm::{address::{PhysPageNum, VirtAddr}, address_space::{AddrSpace, MapArea, MapType, KERNEL_SPACE}, page_table::PTEFlags}, trap::{context::TrapContext, trap_handler, trap_return}};
use super::context::TaskContext;
use crate::config::*;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Block,
    Exit,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub user_space: AddrSpace,
    pub trap_cx_ppn: PhysPageNum,
}

impl TaskControlBlock {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn new(app_id: usize, elf_data: &[u8]) -> Self {
        let (user_space, user_stack_top, entry_point) = AddrSpace::new_user(elf_data);
        let trap_cx_ppn = user_space.root_table
                         .translate_vpn(VirtAddr(TRAP_CONTEXT).floor())
                         .unwrap().ppn();
        let task_status = TaskStatus::Ready;
        // set up kernel stack
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_pos(app_id);
        let mut kernel_space = KERNEL_SPACE.lock();
        kernel_space.push(
            MapArea::new(VirtAddr(kernel_stack_bottom),
            VirtAddr(kernel_stack_top), MapType::Framed, 
            PTEFlags::R | PTEFlags::W), 
            None
        );
        let control_block = Self{
            task_status,
            task_cx: TaskContext::new(trap_return as usize, kernel_stack_top),
            user_space,
            trap_cx_ppn,
        };
        let trap_cx = control_block.get_trap_cx();
        *trap_cx = TrapContext::new(
            user_stack_top,
            entry_point,
            sstatus::read().bits(),
            kernel_space.root_table.get_satp(),
            trap_handler as usize,
            kernel_stack_top,
        );
        control_block
    }   
}
