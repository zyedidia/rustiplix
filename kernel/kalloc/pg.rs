use crate::sys;

use crate::kalloc::global::Alloc;
use core::mem::size_of;

// This is a simple page allocator. It is horribly space-inefficient because it allocates twice as
// much memory as necessary.

pub struct PageAlloc {
    start: *mut u8,
    end: *mut u8,

    base: Header,
    freep: *mut Header,
}

unsafe impl Send for PageAlloc {}

#[repr(align(4096))]
struct Header {
    next: *mut Header,
    size: usize,
}

impl PageAlloc {
    pub const fn new_uninit() -> PageAlloc {
        PageAlloc {
            start: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
            base: Header {
                next: core::ptr::null_mut(),
                size: 0,
            },
            freep: core::ptr::null_mut(),
        }
    }

    fn sbrk(&mut self, incr: usize) -> *mut u8 {
        let start = unsafe { self.start.add(incr) };
        if start > self.end {
            return core::ptr::null_mut();
        }
        let p = self.start;
        self.start = start;
        p
    }

    const NALLOC: usize = 1;
    fn morecore(&mut self, nu: usize) -> *mut Header {
        let mut nu = nu;
        if nu < Self::NALLOC {
            nu = Self::NALLOC;
        }

        let up = self.sbrk(nu * size_of::<Header>()) as *mut Header;
        if up.is_null() {
            return core::ptr::null_mut();
        }

        unsafe {
            (*up).size = nu;
            self.dealloc(up.add(1) as *mut u8);
        }
        self.freep
    }
}

impl Alloc for PageAlloc {
    unsafe fn init(&mut self, start: *mut u8, size: usize) {
        assert!(size != 0 && size % sys::PAGESIZE == 0);
        assert!(!start.is_null() && start as usize % 16 == 0);
        self.start = start;
        self.end = unsafe { start.add(size) };
    }

    fn alloc(&mut self, size: usize) -> *mut u8 {
        assert!(!self.start.is_null());
        assert!(size > 0);

        let nunits = (size + size_of::<Header>() - 1) / size_of::<Header>() + 1;
        let mut prevp = self.freep;

        if prevp.is_null() {
            prevp = &mut self.base as *mut Header;
            self.freep = prevp;
            self.base.next = prevp;
            self.base.size = 0;
        }

        unsafe {
            let mut p = (*prevp).next;
            loop {
                if (*p).size >= nunits {
                    if (*p).size == nunits {
                        (*prevp).next = (*p).next;
                    } else {
                        (*p).size -= nunits;
                        p = p.add((*p).size);
                        (*p).size = nunits;
                    }
                    self.freep = prevp;
                    return p.add(1) as *mut u8;
                }

                if p == self.freep {
                    p = self.morecore(nunits);
                    if p.is_null() {
                        return core::ptr::null_mut();
                    }
                }

                prevp = p;
                p = (*p).next;
            }
        }
    }

    fn dealloc(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        assert!(!self.start.is_null());

        unsafe {
            let bp = (ptr as *mut Header).sub(1);
            let mut p = self.freep;
            while !(bp > p && bp < (*p).next) {
                if p >= (*p).next && (bp > p || bp < (*p).next) {
                    break;
                }
                p = (*p).next;
            }

            if bp.add((*bp).size) == (*p).next {
                (*bp).size += (*(*p).next).size;
                (*bp).next = (*(*p).next).next;
            } else {
                (*bp).next = (*p).next;
            }
            if p.add((*p).size) == bp {
                (*p).size += (*bp).size;
                (*p).next = (*bp).next;
            } else {
                (*p).next = bp;
            }
            self.freep = p;
        }
    }
}
