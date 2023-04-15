/// Terminology in this crate:
/// * hka: high canonical address, above 0xffff_ffc0_0000_0000.
/// * ka: kernel address, either hka in kernel-mode, or pa in monitor-mode.
/// * pa: physical address.
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
/// Returns true if 'va' is a high kernel address.
pub const fn ishka(va: usize) -> bool {
    va >= sys::HIGHMEM_BASE
}

#[inline]
/// Converts a high kernel address to a physical address.
pub const fn hka2pa(ka: usize) -> usize {
    ka - sys::HIGHMEM_BASE
}

#[inline]
#[cfg(feature = "kernel")]
/// Converts a kernel address to a physical address.
pub const fn ka2pa(ka: usize) -> usize {
    ka - sys::HIGHMEM_BASE
}

#[inline]
#[cfg(feature = "monitor")]
/// Converts a kernel address to a physical address.
pub const fn ka2pa(ka: usize) -> usize {
    ka
}

#[inline]
#[cfg(feature = "kernel")]
/// Converts a physical address to a kernel address.
pub const fn pa2ka(pa: usize) -> usize {
    pa + sys::HIGHMEM_BASE
}

#[inline]
#[cfg(feature = "monitor")]
/// Converts a physical address to a kernel address.
pub const fn pa2ka(pa: usize) -> usize {
    pa
}

#[inline]
/// Converts a physical address to a high kernel address.
pub const fn pa2hka(pa: usize) -> usize {
    pa + sys::HIGHMEM_BASE
}

use crate::arch::vm::{Pagetable, PtLevel, Pte};
use crate::proc::Proc;
use alloc::boxed::Box;
use core::ptr::drop_in_place;

pub trait PageMap {
    #[must_use]
    /// Maps the given page at 'va' with permissions 'perm'. The pagetable takes ownership of the
    /// page (the page will be freed when the pagetable is freed).
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

/// Information about a virtual address mapping, including the virtual address and a reference to
/// the PTE that controls the mapping.
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

    /// Returns a slice to the page that is mapped by this entry.
    pub fn pg(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(pa2ka(self.pte.pa()) as *const u8, sys::PAGESIZE) }
    }

    /// Returns a raw pointer to the page that is mapped by this entry.
    pub fn pg_raw(&mut self) -> *mut u8 {
        pa2ka(self.pte.pa()) as *mut u8
    }
}

/// Pagetable iterator.
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
        // This isn't a very efficient implementation of this iterator. It just increases 'va' by
        // the pagesize, and does a pagetable lookup. If a large page is found, it increases 'va'
        // by the size, and otherwise increases 'va' by the page size. Returns true if there are
        // still more PTEs. After execution, 'self.pte' will hold the PTE.
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
        // Advance while the current PTE is null (invalid or a large page).
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
