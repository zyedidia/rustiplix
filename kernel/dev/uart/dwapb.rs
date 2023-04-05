use crate::bit::Bit;
use crate::dev::uart::Uart;

#[repr(C)]
pub struct DwApbUart {
    thr_rbr: u32,
    _pad: [u32; 4],
    lsr: u32,
}

mod lsr {
    pub const DATA_READY: usize = 0;
    pub const THR_EMPTY: usize = 5;
}

impl DwApbUart {
    fn thr_empty(&mut self) -> bool {
        let lsr = unsafe { (&mut self.lsr as *mut u32).read_volatile() };
        lsr.bit(lsr::THR_EMPTY)
    }
}

impl Uart for DwApbUart {
    fn init(&mut self, _baud: u32) {}

    fn tx(&mut self, b: u8) {
        // Wait for thr to be empty before writing the byte.
        while !self.thr_empty() {}
        unsafe {
            (&mut self.thr_rbr as *mut u32).write_volatile(b as u32);
        }
    }

    fn tx_flush(&mut self) {
        while !self.thr_empty() {}
    }

    fn rx(&mut self) -> u8 {
        // Wait until there is data available.
        while self.rx_empty() {}
        // Read the data from rbr (same offset as thr).
        let rbr = unsafe { (&mut self.thr_rbr as *mut u32).read_volatile() };
        rbr as u8
    }

    fn rx_empty(&mut self) -> bool {
        let lsr = unsafe { (&mut self.lsr as *mut u32).read_volatile() };
        !lsr.bit(lsr::DATA_READY)
    }
}
