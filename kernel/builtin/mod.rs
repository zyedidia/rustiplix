pub mod panic;

#[inline(never)]
#[no_mangle]
pub extern "C" fn mark() {}
