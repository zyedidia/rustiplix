use core::arch::asm;

pub fn insn_fence() {
    unsafe { asm!("fence.i") };
}
