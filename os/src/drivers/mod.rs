use core::arch::asm;
use crate::config::VIRT_TEST;

pub mod uart;

const EXIT_SUCCESS: u32 = 0x5555;

pub fn shutdown() {
    unsafe {
        asm!(
          "sw {0}, 0({1})",
          in(reg) EXIT_SUCCESS,
          in(reg) VIRT_TEST,
        );
    }
    panic!("Fail to shutdown.")
}
