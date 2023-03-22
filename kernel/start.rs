use crate::cpu::init_cpu;

extern "C" {
    fn kmain();
}

#[no_mangle]
unsafe extern "C" fn start(coreid: usize, primary: bool) {
    if primary {
        init_bss();
    }
    init_cpu(coreid);
    kmain();
}

unsafe fn init_bss() {
    extern "C" {
        static mut _bss_start: u64;
        static mut _bss_end: u64;
    }

    let mut bss = &mut _bss_start as *mut u64;
    let bss_end = &mut _bss_end as *mut u64;

    while bss < bss_end {
        *bss = 0;
        bss = bss.add(1);
    }
}
