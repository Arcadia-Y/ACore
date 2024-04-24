# handle timer interrupts in M-mode and delegate it to S-mode
    .section .text.trap
    .globl _timer_trap
    .align 2
_timer_trap:
    csrrw sp, mscratch, sp
    sd t0, 0(sp)
    sd t1, 1*8(sp)
    sd t2, 2*8(sp)

    # set mtimecmp
    ld t0, 3*8(sp) # mtimecmp
    ld t1, 4*8(sp) # time interval
    ld t2, 0(t0)
    add t2, t2, t1
    sd t2, 0(t0)

    # setup sip
    li t0, 32
    csrw sip, t0

    ld t0, 0(sp)
    ld t1, 1*8(sp)
    ld t2, 2*8(sp)
    csrrw sp, mscratch, sp 
    mret
