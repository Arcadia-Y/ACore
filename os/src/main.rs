#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod lang_items;
mod config;
mod drivers;
mod io;
mod time;
mod mm;

extern crate alloc;

use core::arch::{global_asm, asm};
use mm::kernel_heap::init_kernel_heap;
use mm::frame_allocator::init_frame_allocator;
use drivers::uart::UART;
use riscv::register::*;
use time::init_timer;

global_asm!(include_str!("entry.asm"));

// initialize, from M-mode to S-mode
#[no_mangle]
pub unsafe fn rust_start() -> ! {
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    // disable pagetable
    satp::write(0);

    // enable interrupts
    asm!("csrw medeleg, {ones}", ones = in(reg) !0);
    asm!("csrw mideleg, {ones}", ones = in(reg) !0);
    sie::set_uext();
    sie::set_stimer();
    sie::set_usoft();

    // physical memory protection
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);

    init_timer();
    asm!("mret", options(noreturn));
}

#[no_mangle]
extern "C" fn rust_main() {
    clear_bss();
    UART.init();
    println!("UART initilized.");

    init_kernel_heap();
    println!("Kernel heap allocator initilized.");

    init_frame_allocator();
    println!("Frame allocator initilized.");

    println!("Hello, World!");
    panic!("test!");
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
