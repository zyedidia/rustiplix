use crate::sync::spinlock::SpinLock;
use crate::sys;
use core::cell::Cell;

struct Run {
    freelist: Cell<&Run>,
}

static KMEM: SpinLock<Cell<Run>>;

pub unsafe fn kinit() {

}

pub unsafe fn kfree(pa: *mut u8) {

}

pub unsafe fn kallocpage() -> *mut [u8; sys::PAGESIZE] {

}
