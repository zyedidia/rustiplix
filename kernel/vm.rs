use crate::sys;

pub mod perm {
    pub const READ: u8 = 1 << 0;
    pub const WRITE: u8 = 1 << 1;
    pub const EXEC: u8 = 1 << 2;
    pub const USER: u8 = 1 << 3;
    pub const COW: u8 = 1 << 4;
    pub const RWX: u8 = READ | WRITE | EXEC;
    pub const URWX: u8 = USER | RWX;
}

#[inline]
pub const fn iska(va: usize) -> bool {
    va >= sys::HIGHMEM_BASE
}

#[inline]
pub const fn hka2pa(ka: usize) -> usize {
    ka - sys::HIGHMEM_BASE
}

#[inline]
#[cfg(feature = "kernel")]
pub const fn ka2pa(ka: usize) -> usize {
    ka - sys::HIGHMEM_BASE
}

#[inline]
#[cfg(feature = "monitor")]
pub const fn ka2pa(ka: usize) -> usize {
    ka
}

#[inline]
#[cfg(feature = "kernel")]
pub const fn pa2ka(pa: usize) -> usize {
    pa + sys::HIGHMEM_BASE
}

#[inline]
#[cfg(feature = "monitor")]
pub const fn pa2ka(pa: usize) -> usize {
    pa
}

#[inline]
pub const fn pa2hka(pa: usize) -> usize {
    pa + sys::HIGHMEM_BASE
}

use crate::arch::vm::{Pagetable, PtLevel};
use alloc::boxed::Box;
use core::ptr::drop_in_place;

pub trait PageMap {
    #[must_use]
    // Maps the given page at 'va' with permissions 'perm'. The pagetable takes ownership of the
    // page (the page will be freed when the pagetable is freed).
    fn mappg(&mut self, va: usize, pg: Box<[u8; sys::PAGESIZE]>, perm: u8) -> Option<()>;
}

impl PageMap for Pagetable {
    fn mappg(&mut self, va: usize, pg: Box<[u8; sys::PAGESIZE]>, perm: u8) -> Option<()> {
        let raw: *mut [u8; sys::PAGESIZE] = Box::into_raw(pg);
        if !self.map(va, ka2pa(raw as usize), PtLevel::Normal, perm) {
            // Free the box since we now own it and mapping failed.
            unsafe { drop_in_place(raw) };
            return None;
        }
        Some(())
    }
}
