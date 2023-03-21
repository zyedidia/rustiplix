use crate::dev::uart::Putc;

#[repr(C)]
pub struct VirtUart {
    thr: u32,
}

impl VirtUart {
    pub fn init(&mut self) {}

    pub fn tx(&mut self, b: u8) {
        unsafe {
            (&mut self.thr as *mut u32).write_volatile(b as u32);
        }
    }

    pub fn tx_flush(&mut self) {}

    pub fn rx(&mut self) -> u8 {
        0
    }

    pub fn rx_empty(&mut self) -> bool {
        true
    }
}

impl Putc for VirtUart {
    fn putc(&mut self, c: u8) {
        self.tx(c);
    }
}
