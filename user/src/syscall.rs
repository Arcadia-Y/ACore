pub const SYSCALL_WRITE: usize = 1;
pub const SYSCALL_EXIT: usize = 2;
pub const SYSCALL_YIELD: usize = 3;
pub const SYSCALL_RECV: usize = 4;
pub const SYSCALL_SENDRECV: usize = 5;
pub const SYSCALL_FORK: usize = 6;
pub const SYSCALL_EXEC: usize = 7;
pub const SYSCALL_WAITPID: usize = 8;
pub const SYSCALL_READ: usize = 9;
pub const SYSCALL_GETPID: usize = 10;
pub const SYSCALL_GETTIME: usize = 11;

use core::arch::asm;

fn syscall(id: usize, args: [usize; 4]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") id => ret,
            in("x11") args[0],
            in("x12") args[1],
            in("x13") args[2],
            in("x14") args[3],
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(), 0])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0, 0])
}

pub fn recv(ptr: *mut usize, len: usize) -> isize {
    syscall(SYSCALL_RECV, [ptr as usize, len, 0, 0])
}

pub fn sendrecv(send: *const usize, send_len: usize, recv: *mut usize, recv_len: usize) -> isize {
    syscall(SYSCALL_SENDRECV, [send as usize, send_len, recv as usize, recv_len])
}

pub fn fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0, 0])
}

pub fn exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, path.len(), 0, 0])
}

pub fn sys_waitpid(pid: isize,  exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_ptr() as usize, buffer.len(), 0])
}

pub fn getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0, 0])
}

pub fn get_time() -> isize {
    syscall(SYSCALL_GETTIME, [0, 0, 0, 0])
}
