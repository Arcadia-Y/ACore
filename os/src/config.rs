pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const MEMORY_END: usize = 0x88_000_000;

pub const UART_BASE: usize = 0x10000000;
pub const UART_SIZE: usize = 0x6;
pub const VIRT_TEST: usize = 0x100000;

pub const MTIME: usize = 0x0200bff8;
pub const MTIMECMP: usize = 0x02004000;
pub const TIME_INTERVAL : usize = 1000000;

pub const TRAMPOLINE_ADDR : usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT : usize = TRAMPOLINE_ADDR - PAGE_SIZE;
pub const USER_STACK_SIZE : usize = 4096 * 2;
pub const KERNEL_STACK_SIZE : usize = 4096 * 2;

pub fn kernel_stack_pos(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE_ADDR - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE); // guard page
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
