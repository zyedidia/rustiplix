use crate::sync::spinlock::SpinLock;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct KernelAlloc<T> {
    internal: SpinLock<T>,
}

impl<T> KernelAlloc<T> {
    pub const fn new(val: T) -> Self {
        Self {
            internal: SpinLock::new(val),
        }
    }
}

unsafe impl<T> GlobalAlloc for KernelAlloc<T> {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("todo");
    }
}

use crate::kalloc::pg::PageAlloc;

#[global_allocator]
static ALLOCATOR: KernelAlloc<PageAlloc> = KernelAlloc::new(PageAlloc::new_uninit());
