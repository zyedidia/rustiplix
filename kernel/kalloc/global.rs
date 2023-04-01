use crate::sync::spinlock::SpinLock;

use alloc::alloc::{GlobalAlloc, Layout};

pub trait Alloc {
    unsafe fn init(&mut self, start: *mut u8, size: usize);
    fn alloc(&mut self, size: usize) -> *mut u8;
    fn dealloc(&mut self, ptr: *mut u8);
}

pub struct KernelAlloc<T: Alloc> {
    internal: SpinLock<T>,
}

impl<T: Alloc> KernelAlloc<T> {
    pub const fn new(val: T) -> Self {
        Self {
            internal: SpinLock::new(val),
        }
    }
}

unsafe impl<T: Alloc> GlobalAlloc for KernelAlloc<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.internal.lock().alloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.internal.lock().dealloc(ptr);
    }
}

use crate::kalloc::pg::PageAlloc;

#[global_allocator]
static ALLOCATOR: KernelAlloc<PageAlloc> = KernelAlloc::new(PageAlloc::new_uninit());

pub unsafe fn init_alloc(start: *mut u8, size: usize) {
    unsafe { ALLOCATOR.internal.lock().init(start, size) };
}
