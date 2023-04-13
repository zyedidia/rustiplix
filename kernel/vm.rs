use crate::sys;

pub mod perm {
    pub const READ: u8 = 1 << 0;
    pub const WRITE: u8 = 1 << 1;
    pub const EXEC: u8 = 1 << 2;
    pub const USER: u8 = 1 << 3;
    pub const COW: u8 = 1 << 4;
    pub const RWX: u8 = READ | WRITE | EXEC;
    pub const URWX: u8 = USER | RWX;
    pub const URW: u8 = USER | READ | WRITE;
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

use crate::arch::vm::{Pagetable, PtLevel, Pte};
use crate::proc::Proc;
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

pub struct VaMapping<'a> {
    pte: &'a mut Pte,
    va: usize,
}

impl VaMapping<'_> {
    pub fn va(&self) -> usize {
        self.va
    }

    pub fn pa(&self) -> usize {
        self.pte.pa()
    }

    pub fn perm(&self) -> u8 {
        self.pte.perm()
    }

    pub fn pte(&mut self) -> &mut Pte {
        self.pte
    }

    pub fn pg(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(pa2ka(self.pte.pa()) as *const u8, sys::PAGESIZE) }
    }

    pub fn pg_raw(&mut self) -> *mut u8 {
        pa2ka(self.pte.pa()) as *mut u8
    }
}

pub struct PtIter<'a> {
    idx: usize,
    va: usize,
    pte: *mut Pte,
    pt: &'a mut Pagetable,
}

impl PtIter<'_> {
    pub fn new<'a>(pt: &'a mut Pagetable) -> PtIter<'a> {
        PtIter::<'a> {
            idx: 0,
            va: 0,
            pte: core::ptr::null_mut(),
            pt,
        }
    }

    fn advance(&mut self) -> bool {
        if self.va >= Proc::MAX_VA {
            return false;
        }

        if let Some((pte, lvl)) = self.pt.walk::<false>(self.va, PtLevel::Normal) {
            if lvl != PtLevel::Normal || !pte.is_valid() {
                self.pte = core::ptr::null_mut();
            } else {
                self.pte = pte as *mut Pte;
            }
            self.va += lvl.size();
        } else {
            self.pte = core::ptr::null_mut();
            self.va += PtLevel::Normal.size();
        }

        true
    }
}

impl<'a> Iterator for PtIter<'a> {
    type Item = VaMapping<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut va = self.va;
        if !self.advance() {
            return None;
        }
        while self.pte == core::ptr::null_mut() {
            va = self.va;
            if !self.advance() {
                return None;
            }
        }
        unsafe {
            Some(VaMapping {
                pte: &mut *self.pte,
                va,
            })
        }
    }
}
