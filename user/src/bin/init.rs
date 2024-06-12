#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::syscall::*;
use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("shell");
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            }
            println!(
                "[init] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }
    0
}
