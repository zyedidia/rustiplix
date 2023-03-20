use crate::uart::uart;

pub fn kmain(_coreid: u32) {
    unsafe {
        uart::tx('h' as u8);
        uart::tx('i' as u8);
        uart::tx('\n' as u8);
    }
}
