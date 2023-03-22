use crate::cpu::Cpu;
use core::arch::asm;

pub fn rd_cpu() -> &'static Cpu {
    let val: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) val);
        &*(val as *const Cpu)
    }
}

/// # Safety
///
/// Writes a CPU struct into the core-local thread-pointer.
pub unsafe fn wr_cpu(cpu: &Cpu) {
    asm!("mv tp, {}", in(reg) cpu as *const _ as usize);
}
