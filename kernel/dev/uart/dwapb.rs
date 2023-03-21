use crate::dev::uart::Putc;

#[repr(C)]
pub struct DwApb {
    thr: u32,
}

impl DwApb {
    pub fn init(&mut self) {}

    pub fn tx(&mut self, b: u8) {
        unsafe {
            (&mut self.thr as *mut u32).write_volatile(b as u32);
        }
    }
}

impl Putc for DwApb {
    fn putc(&mut self, c: u8) {
        self.tx(c);
    }
}
