use kernel::dev::uart::dwapb::DwApb;

pub fn kmain(_coreid: u32) {
    let uart = unsafe { &mut *(0x10000000 as *mut DwApb) };

    uart.tx('h' as u8);
    uart.tx('e' as u8);
    uart.tx('y' as u8);
    uart.tx('\n' as u8);
}
