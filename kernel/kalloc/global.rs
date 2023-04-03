use crate::sync::spinlock::SpinLock;
use crate::sys;

use alloc::alloc::{AllocError, GlobalAlloc, Layout};
use alloc::boxed::Box;

use core::ptr::NonNull;

extern crate buddyalloc;

use buddyalloc::Heap;

struct LockedHeap<const N: usize> {
    heap: SpinLock<Heap<N>>,
}

unsafe impl<const N: usize> GlobalAlloc for LockedHeap<N> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.heap.lock();
        if let Ok(ptr) = heap.allocate(layout) {
            return ptr;
        }
        return core::ptr::null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut heap = self.heap.lock();
        heap.deallocate(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap<10> = LockedHeap {
    heap: SpinLock::new(unsafe { Heap::new_unchecked(core::ptr::null_mut(), 0) }),
};

pub unsafe fn init_alloc(start: *mut u8, size: usize) {
    assert!(!start.is_null());
    unsafe {
        *ALLOCATOR.heap.lock() = Heap::new(NonNull::new_unchecked(start), size).unwrap();
    }
}

pub fn kallocpage() -> Result<Box<[u8; sys::PAGESIZE]>, AllocError> {
    let page = Box::<[u8; sys::PAGESIZE]>::try_new_uninit()?;
    unsafe { Ok(page.assume_init()) }
}

pub fn zallocpage() -> Result<Box<[u8; sys::PAGESIZE]>, AllocError> {
    let page = Box::<[u8; sys::PAGESIZE]>::try_new_zeroed()?;
    unsafe { Ok(page.assume_init()) }
}

pub fn zalloc_raw<T>() -> Option<NonNull<T>> {
    let val = unsafe { ALLOCATOR.alloc(Layout::new::<T>()) as *mut T };
    if val == core::ptr::null_mut() {
        return None;
    }
    unsafe {
        core::intrinsics::write_bytes(val, 0, core::mem::size_of::<T>());
        Some(NonNull::new_unchecked(val))
    }
}

pub unsafe fn kfree<T>(ptr: *mut T) {
    ALLOCATOR.dealloc(ptr as *mut u8, Layout::new::<T>());
}

use core::mem::MaybeUninit;

pub fn kalloc<T>(init: fn(&mut MaybeUninit<T>)) -> Result<Box<T>, AllocError> {
    let mut val = Box::<T>::try_new_uninit()?;
    init(&mut val);
    unsafe { Ok(val.assume_init()) }
}
