pub mod id;
mod fs;
mod proc;
use id::*;
use fs::*;
use proc::*;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        _ => {
            panic!("Unsupported syscall id: {}", id);
        }
    }
}
