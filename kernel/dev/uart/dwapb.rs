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

pub unsafe fn tx(b: u8) {
    let thr = 0x10000000 as *mut u32;
    thr.write_volatile(b as u32);
}
