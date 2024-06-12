use core::arch::{asm, global_asm};

use riscv::register::{scause::{self, Exception, Interrupt, Trap}, stval, stvec, utvec::TrapMode};
use crate::{config::{TRAMPOLINE_ADDR, TRAP_CONTEXT}, ipc::rpc::RPC_BUFFER, syscall::syscall, task::{processor::{current_trap_cx, current_user_satp}, show_task_frames, suspend_current_and_run_next}, time::{get_mtime_cmp, get_time}};
pub mod context;

global_asm!(include_str!("trap.S"));

pub fn set_kernel_stvec() {
    unsafe { stvec::write(trap_from_kernel as usize, TrapMode::Direct) };
}

pub fn set_user_stvec() {
    unsafe { stvec::write(TRAMPOLINE_ADDR, TrapMode::Direct) };
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_stvec();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let cx = current_trap_cx();
            cx.sepc += 4;
            // syscall id in a0, args in a1-a4
            let result = syscall(cx.x[10], [cx.x[11], cx.x[12], cx.x[13], cx.x[14]]) as usize;
            let cx = current_trap_cx();
            cx.x[10] = result;
            if get_time() > get_mtime_cmp() {
                suspend_current_and_run_next();
            }
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            let cx = current_trap_cx();
            cx.sepc += 4;
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_stvec();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_satp();
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE_ADDR;
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    unsafe {
        asm!(
            "fence.i",
            "jr {va}",
            va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            panic!("user_env_call!");
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            panic!("timer!");
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
}
