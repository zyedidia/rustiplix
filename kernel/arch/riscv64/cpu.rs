use crate::cpu::Cpu;
use core::arch::asm;

/// Reads the CPU struct out of the thread-local pointer. Unsafe because interrupts must be
/// disabled.
pub unsafe fn rd_cpu() -> &'static mut Cpu {
    let val: usize;
    asm!("mv {}, tp", out(reg) val);
    &mut *(val as *mut Cpu)
}

/// Writes a CPU struct into the core-local thread-pointer. Unsafe because interrupts must be
/// disabled.
pub unsafe fn wr_cpu(cpu: &mut Cpu) {
    asm!("mv tp, {}", in(reg) cpu as *mut _ as usize);
}

/// Puts the processor into a low-power state while waiting for an interrupt.
pub fn wfi() {
    unsafe {
        asm!("wfi");
    }
}
