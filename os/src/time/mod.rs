use core::arch::global_asm;
use core::ptr::addr_of_mut;
use riscv::register::*;

const MTIME: usize = 0x0200bff8;
const MTIMECMP: usize = 0x02004000;
const TIME_INTERVAL : usize = 1000000;

global_asm!(include_str!("timer_trap.s"));

pub fn set_timer(time: usize) {
    unsafe {
        let timer = MTIMECMP as *mut usize;
        *timer = time;
    }
}

pub fn get_time() -> usize {
    unsafe {
        let time = MTIME as *const usize;
        time.read_volatile()
    }
}

#[link_section = ".bss.stack"]
#[no_mangle]
pub static mut TIMER_SCRATCH: [usize; 5] = [0; 5];

#[no_mangle]
pub unsafe fn init_timer() {
    set_timer(get_time() + TIME_INTERVAL);
    
    // TIMER_SCRATCH is stack base for M-mode when handling timer interrupt
    // TIMER_SCRATCH[3]: address of MTIMECMP
    // TIMER_SCRATCH[4]: TIME_INTERVAL
    TIMER_SCRATCH[3] = MTIMECMP;
    TIMER_SCRATCH[4] = TIME_INTERVAL;
    mscratch::write(addr_of_mut!(TIMER_SCRATCH) as usize);

    // set mtvec
    extern "C" {
        fn _timer_trap();
    }
    mtvec::write(_timer_trap as usize, mtvec::TrapMode::Direct);

    mstatus::set_mie();
    mie::set_mtimer();
}
