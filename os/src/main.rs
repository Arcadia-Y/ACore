#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod drivers;
mod io;

use core::arch::global_asm;
use crate::drivers::uart::UART;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    UART.init();
    println!("Hello, World!");
    loop {}
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    for a in sbss as usize..ebss as usize {
        unsafe { (a as *mut u8).write_volatile(0) }
    }
}
