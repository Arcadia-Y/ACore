use core::sync::atomic::{AtomicU8, Ordering};
use volatile::Volatile;
use lazy_static::lazy_static;

macro_rules! wait_till {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop();
        }
    };
}

const UART_BASE: usize = 0x10000000;
lazy_static! {
    pub static ref UART: Uart = unsafe { Uart::new(UART_BASE) };
}

// some UART control register bits
const THR_EMPTY: u8 = 1 << 5;
const RBR_READY: u8 = 1;
const DLAB_ENABLE: u8 = 1 << 7;
const LCR_EIGHT_BITS: u8 = 3;
const FCR_FIFO_ENABLE: u8 = 1;
const FCR_FIFO_CLEAR: u8 = 3 << 1;
const IER_RX_ENABLE: u8 = 1;
const IER_TX_ENABLE: u8 = 1 << 1;

// packed ReadPort for uart
pub struct ReadPort {
    rbr: AtomicU8,
    ier: AtomicU8,
    iir: AtomicU8,
    lcr: AtomicU8,
    mcr: AtomicU8,
    lsr: AtomicU8,
    msr: AtomicU8,
    scr: AtomicU8
}

// packed WritePort for uart
pub struct WritePort {
    thr: AtomicU8,
    ier: Volatile<u8>,
    fcr: Volatile<u8>,
    lcr: Volatile<u8>,
    mcr: Volatile<u8>,
    lsr: AtomicU8,
    not_used: AtomicU8,
    scr: AtomicU8
}

pub struct Uart {
    base: usize
}

impl Uart {
    pub unsafe fn new(base: usize) -> Self {
        Uart{base}
    }

    fn read_port(&self) -> &'static mut ReadPort {
        unsafe { &mut *(self.base as *mut ReadPort) }
    }

    fn write_port(&self) -> &'static mut WritePort {
        unsafe { &mut *(self.base as *mut WritePort) }
    }

    // put a character to the output buffer.
    // spins waiting until the output register to be empty
    pub fn putc(&self, c: u8) {
        let write_port = self.write_port();
        let lsr = &write_port.lsr;
        let thr = &write_port.thr;
        wait_till!(lsr.load(Ordering::Acquire) & THR_EMPTY != 0);
        thr.store(c, Ordering::Release);
    }

    // read a input character.
    // return none if no input is available.
    pub fn getc(&self) -> Option<u8> {
        let read_port = self.read_port();
        let lsr = &read_port.lsr;
        let rbr = &read_port.rbr;
        if lsr.load(Ordering::Acquire) & RBR_READY != 0 {
            Some(rbr.load(Ordering::Acquire))
        } else {
            None
        }
    }

    pub fn init(&self) {
        let write_port = self.write_port();

        // disable interrupts
        write_port.ier.write(0x00);

        // enable DLAB
        write_port.lcr.write(DLAB_ENABLE);

        // set LSB for baud rate of 38.4K
        write_port.thr.store(0x03, Ordering::Relaxed);

        // set MSB for baud rate of 38.4K
        write_port.ier.write(0x00);

        // disable DLAB 
        // and set word length to 8 bits, no parity
        write_port.lcr.write(LCR_EIGHT_BITS);

        // reset and enable FIFO
        write_port.fcr.write(FCR_FIFO_ENABLE | FCR_FIFO_CLEAR);

        // enable transmit and receive interrupts
        write_port.ier.write(IER_TX_ENABLE | IER_RX_ENABLE);
    }
}
