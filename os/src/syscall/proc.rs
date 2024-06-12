use alloc::{string::String, vec};
use crate::{ipc::RPC_BUFFER, loader::get_app_data_by_name, mm::page_table::{get_user_byte_buffer, translate_refmut}, println, task::{add_task, exit_current_and_run_next, processor::{current_task, current_user_satp, take_current_task}, recycle_id, rpc_call, show_task_frames, suspend_current_and_run_next}};
use super::id::*;

const PROCESS_MANAGER_ID: usize = 1;

pub fn sys_exit(exit_code: i32) -> ! {
    let task = current_task().unwrap();
    let id = task.taskid.0;
    drop(task);
    rpc_call(PROCESS_MANAGER_ID, vec![SYSCALL_EXIT, id, exit_code as usize]);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().taskid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let current_id = current_task.taskid.0;
    let new_id = new_task.taskid.0;
    rpc_call(PROCESS_MANAGER_ID, vec![SYSCALL_FORK, current_id, new_id]);
    let trap_cx = new_task.get_trap_cx();
    trap_cx.x[10] = 0;
    add_task(new_task);
    new_id as isize
} 

pub fn sys_exec(path: *const u8, len: usize) -> isize {
    let satp = current_user_satp();
    let name_vec = get_user_byte_buffer(satp, path, len);
    let name = String::from_utf8(name_vec).unwrap();
    if name == "process_manager" {
        return -1;
    }
    if let Some(data) = get_app_data_by_name(name.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    let id = task.taskid.0;
    rpc_call(PROCESS_MANAGER_ID, vec![SYSCALL_WAITPID, id, pid as usize]);
    let rpc = RPC_BUFFER.lock();
    let ret = rpc.data[0] as isize;
    if ret > 0 {
        let satp = current_user_satp();
        let exit_code = translate_refmut(satp, exit_code_ptr);
        *exit_code = rpc.data[1] as i32;
        recycle_id(ret as usize);
    }
    ret
}
