#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use core::str::from_utf8_unchecked;

use user_lib::{console::getchar, syscall::{exec, fork}, waitpid};

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

const BUF_SIZE: usize = 1024;
static mut BUF: [u8; BUF_SIZE] = [0u8; BUF_SIZE];

#[no_mangle]
fn main() -> i32 {
    let mut cursor: usize = 0;
    print!("root# ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                print!("\n");
                if cursor != 0 {
                    let pid = fork();
                    if pid == 0 {
                        unsafe {
                            let name = from_utf8_unchecked(&BUF[0..cursor]);
                            if exec(name) == -1 {
                                println!("[shell] Error during execution!");
                                return -1;
                            }
                        }
                    } else {
                        let mut exit_code = 0;
                        let exit_pid = waitpid(pid, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("[shell] Process {} exited with exitcode {}", pid, exit_code);
                    }
                    cursor = 0;
                }
                print!("root# ");
            }
            BS | DL => {
                if cursor != 0 {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    cursor -= 1;
                }
            } 
            _ => {
                if cursor >= BUF_SIZE {
                    print!("\n");
                    println!("[shell] Warning: exceed shell buffer limit, buffer abandoned!");
                    cursor = 0;
                    print!("root# ");
                } else {
                    print!("{}", c as char);
                    unsafe {
                        BUF[cursor] = c;
                    }
                    cursor += 1;
                }
            }
        }
    }
}
