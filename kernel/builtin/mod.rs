pub mod panic;

use core::arch::asm;

#[no_mangle]
pub extern "C" fn mark() {
    unsafe {
        asm!("nop");
    }
}
