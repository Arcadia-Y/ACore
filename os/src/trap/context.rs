#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub trap_handler: usize,
    pub kernel_sp: usize,
}

impl TrapContext {
    pub fn new(
        user_sp: usize,
        sepc: usize,
        sstatus: usize,
        kernel_satp: usize,
        trap_handler: usize,
        kernel_sp: usize,
    ) -> Self {
        let mut saved_regs: [usize; 32] = [0; 32];
        saved_regs[2] = user_sp;
        Self {
            x: saved_regs,
            sstatus,
            sepc,
            kernel_satp,
            trap_handler,
            kernel_sp,
        }
    }
}
