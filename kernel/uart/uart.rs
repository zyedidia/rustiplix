pub unsafe fn tx(b: u8) {
    let thr = 0x10000000 as *mut u32;
    thr.write_volatile(b as u32);
}
