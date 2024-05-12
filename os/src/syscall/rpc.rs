use core::cmp::min;

use alloc::vec::Vec;

use crate::{ipc::rpc::RPC_BUFFER, mm::page_table::{copy_bytes_to_user, get_user_byte_buffer}, task::{block_current_and_run_next, processor::current_user_satp}};

// receive len * usize at ptr
pub fn sys_recv(ptr: usize, len: usize) -> isize {
    let mut rpc = RPC_BUFFER.lock();
    rpc.callee = None;
    drop(rpc);
    block_current_and_run_next();
    let data = &RPC_BUFFER.lock().data;
    copy_bytes_to_user(current_user_satp(), data.as_ptr() as *const u8, ptr, min(data.len(), len)*8);
    0
}

// send len * usize at send and receive len * usize at recv
pub fn sys_sendrecv(send: usize, send_len: usize, recv: usize, recv_len: usize) -> isize {
    let tosend = get_user_byte_buffer(current_user_satp(), send as *const u8, send_len * 8);
    let (ptr, len, capa) = tosend.into_raw_parts();
    let mut rpc = RPC_BUFFER.lock();
    rpc.callee = None;
    unsafe {
        rpc.data = Vec::from_raw_parts(ptr as *mut usize, len*8, capa*8);
    }
    drop(rpc);
    block_current_and_run_next();
    let data = &RPC_BUFFER.lock().data;
    copy_bytes_to_user(current_user_satp(), data.as_ptr() as *const u8, recv, min(data.len(), recv_len)*8);
    0
}
