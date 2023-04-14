pub mod panic;

#[inline(never)]
#[no_mangle]
pub extern "C" fn mark() {
    unsafe { core::arch::asm!("nop") };
}
