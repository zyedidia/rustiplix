use core::arch::asm;

pub enum Func {
    WakeCores,
    SetTimer,
}

pub fn wake_cores() {
    unsafe { asm!("ecall", in("a7") Func::WakeCores as u64) };
}

pub fn set_timer(val: u64) {
    unsafe { asm!("ecall", in("a7") Func::SetTimer as u64, in("a0") val) };
}
