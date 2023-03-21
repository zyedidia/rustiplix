use core::arch::asm;

#[inline]
pub fn vm_fence() {
    unsafe {
        asm!("sfence.vma");
    }
}
