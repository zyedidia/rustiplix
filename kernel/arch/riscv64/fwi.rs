use core::arch::asm;

pub mod func {
    pub const WAKE_CORES: u64 = 0;
}

pub fn wake_cores() {
    unsafe {
        asm!("ecall", in("a7") func::WAKE_CORES);
    }
}
