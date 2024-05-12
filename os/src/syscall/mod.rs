pub mod id;
mod fs;
mod proc;
mod rpc;
use id::*;
use fs::*;
use proc::*;
use rpc::*;

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
        _ => {
            panic!("Unsupported syscall id: {}", id);
        }
    }
}
