use crate::bit::BitIndex;
use crate::dev::uart::Putc;

#[repr(C)]
pub struct DwApbUart {
    thr_rbr: u32,
    _pad: [u32; 4],
    lsr: u32,
}

struct Lsr {}
impl Lsr {
    const DATA_READY: usize = 0;
    const THR_EMPTY: usize = 5;
}

impl DwApbUart {
    pub fn init(&mut self) {}

    pub fn tx(&mut self, b: u8) {
        // Wait for thr to be empty before writing the byte.
        while !self.thr_empty() {}
        unsafe {
            (&mut self.thr_rbr as *mut u32).write_volatile(b as u32);
        }
    }

    pub fn tx_flush(&mut self) {
        while !self.thr_empty() {}
    }

    pub fn rx(&mut self) -> u8 {
        // Wait until there is data available.
        while self.rx_empty() {}
        // Read the data from rbr (same offset as thr).
        let rbr = unsafe { (&mut self.thr_rbr as *mut u32).read_volatile() };
        rbr as u8
    }

    fn thr_empty(&mut self) -> bool {
        let lsr = unsafe { (&mut self.lsr as *mut u32).read_volatile() };
        lsr.bit(Lsr::THR_EMPTY)
    }

    pub fn rx_empty(&mut self) -> bool {
        let lsr = unsafe { (&mut self.lsr as *mut u32).read_volatile() };
        lsr.bit(Lsr::DATA_READY)
    }
}

impl Putc for DwApbUart {
    fn putc(&mut self, c: u8) {
        self.tx(c);
    }
}
