.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sscratch -> user_stack, sp -> TrapContext
    
    # store registers except x0/sp/tp
    sd x1, 1*8(sp)
    sd x3, 1*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # store sstatus, sepc
    csrr t0, sstatus
    sd t0, 32*8(sp)
    csrr t0, sepc
    sd t0, 33*8(sp)

    # store origianl sp
    csrr t0, sscratch
    sd t0, 2*8(sp)

    # load kernel_satp, trap_handler, kernel_sp
    ld t0, 34*8(sp)
    ld t1, 35*8(sp)
    ld sp, 36*8(sp)

    # switch to kernel space
    sfence.vma
    csrw satp, t0
    sfence.vma

    # jump to trap_handler
    jr t1

# a0: TrapContext in user space, a1: user space satp
__restore:
    # switch to user space
    sfence.vma
    csrw satp, a1
    sfence.vma

    # set sp, sscratch to TrapContext
    csrw sscratch, a0
    mv sp, a0

    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # restore registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    # back to user stack
    ld sp, 2*8(sp)
    sret
