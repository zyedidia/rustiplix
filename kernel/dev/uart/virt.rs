use crate::dev::uart::Uart;

#[repr(C)]
pub struct VirtUart {
    thr: u32,
}

impl VirtUart {}

impl Uart for VirtUart {
    fn init(&mut self, _baud: u32) {}

    fn tx(&mut self, b: u8) {
        unsafe {
            (&mut self.thr as *mut u32).write_volatile(b as u32);
        }
    }

    fn tx_flush(&mut self) {}

    fn rx(&mut self) -> u8 {
        0
    }

    fn rx_empty(&mut self) -> bool {
        true
    }
}
