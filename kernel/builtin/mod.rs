pub mod panic;

use core::hint::black_box;

#[inline(never)]
#[no_mangle]
pub extern "C" fn mark() {
    black_box(());
}
