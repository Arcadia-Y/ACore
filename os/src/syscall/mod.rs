pub mod id;
mod fs;
mod proc;
mod ipc;
use id::*;
use fs::*;
use proc::*;
use ipc::*;
use crate::{config::CLOCK_FREQ, time::get_time};

pub fn syscall(id: usize, args: [usize; 4]) -> isize {
    match id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_RECV => sys_recv(args[0], args[1]),
        SYSCALL_SENDRECV => sys_sendrecv(args[0], args[1], args[2], args[3]),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_GETTIME => (get_time() / (CLOCK_FREQ / 1000)) as isize,
        _ => {
            panic!("Unsupported syscall id: {}", id);
        }
    }
}
