pub mod alloc;
pub mod mem;
pub mod panic;

use core::arch::asm;

#[no_mangle]
pub extern "C" fn mark() {
    unsafe {
        asm!("nop");
    }
}
