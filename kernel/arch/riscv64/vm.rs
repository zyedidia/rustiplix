use crate::bit::Bit;
use crate::kalloc::{kfree, zalloc_raw};
use crate::sys;
use crate::vm::{ka2pa, pa2ka, perm};
use core::arch::asm;

// TODO: should this be an enum or just usize constants?
#[derive(Copy, Clone)]
pub enum PtLevel {
    Normal = 0,
    Mega = 1,
    Giga = 2,
}

const SV39: usize = 8;

#[derive(Copy, Clone)]
pub struct Pte {
    data: u64,
}

impl Pte {
    bitfield!(data: u64;
        valid, set_valid: 0, 0;
        read, set_read: 1, 1;
        write, set_write: 2, 2;
        exec, set_exec: 3, 3;
        user, set_user: 4, 4;
        _global, _set_global: 5, 5;
        accessed, set_accessed: 6, 6;
        dirty, set_dirty: 7, 7;
        cow, set_cow: 8, 8;
        _rsw, _set_rsw: 9, 9;
        ppn0, set_ppn0: 18, 10;
        ppn1, set_ppn1: 27, 19;
        ppn2, set_ppn2: 53, 28;
    );

    fn validate(&mut self) {
        self.set_valid(1);
        self.set_accessed(1);
        self.set_dirty(1);
    }

    pub fn pa(&self) -> usize {
        ((self.ppn0() << 12) | (self.ppn1() << 21) | (self.ppn2() << 30)) as usize
    }

    pub fn set_pa(&mut self, pa: usize) {
        self.data = self.data.set_bits(53, 10, pa.bits(55, 12) as u64);
    }

    // If this function is made public, it should also take an additional `level` parameter since
    // on other architectures determining if a PTE is a leaf requires knowing the level.
    fn leaf(&self) -> bool {
        self.data.bits(3, 1) != 0
    }

    pub fn set_perm(&mut self, perm: u8) {
        self.set_read((perm & perm::READ != 0) as u64);
        self.set_write((perm & perm::WRITE != 0) as u64);
        self.set_exec((perm & perm::EXEC != 0) as u64);
        self.set_user((perm & perm::USER != 0) as u64);
        self.set_cow((perm & perm::COW != 0) as u64);
    }

    pub fn perm(&self) -> u8 {
        let mut p: u8 = 0;
        if self.read() != 0 {
            p |= perm::READ;
        }
        if self.write() != 0 {
            p |= perm::WRITE;
        }
        if self.exec() != 0 {
            p |= perm::EXEC;
        }
        if self.user() != 0 {
            p |= perm::USER;
        }
        if self.cow() != 0 {
            p |= perm::COW;
        }
        p
    }
}

fn vpn(level: usize, va: usize) -> usize {
    (va >> (12 + 9 * level)) & usize::mask(9)
}

#[repr(align(4096))]
pub struct Pagetable {
    ptes: [Pte; 512],
}

impl Pagetable {
    pub const fn new() -> Self {
        Self {
            ptes: [Pte { data: 0 }; 512],
        }
    }

    pub fn walk<const ALLOC: bool>(
        &mut self,
        va: usize,
        endlevel: PtLevel,
    ) -> Option<(*mut Pte, PtLevel)> {
        let mut pt = self;
        for level in (0..=endlevel as usize).rev() {
            let pte = &mut pt.ptes[vpn(level, va)];
            if pte.leaf() {
                // TODO: level should be `level`, not PtLevel::Normal.
                return Some((pte, PtLevel::Normal));
            } else if pte.valid() != 0 {
                pt = unsafe { &mut *(pa2ka(pte.pa()) as *mut Pagetable) };
            } else {
                if !ALLOC {
                    return None;
                }
                // Allocate an internal pagetable. This pagetable is not owned by anything that
                // Rust knows about (it is owned by the hardware), so we have to manually allocate
                // (and free it) using raw pointers.
                let pt = match zalloc_raw::<Pagetable>() {
                    None => {
                        return None;
                    }
                    Some(pt) => pt,
                };
                pte.set_pa(ka2pa(pt.addr().get()));
                pte.set_valid(1);
            }
        }
        Some((&mut pt.ptes[vpn(endlevel as usize, va)], endlevel))
    }

    fn free(&mut self, level: usize) {
        // Iterate over internal pagetables and recursively call free and manually free the
        // un-owned pointers.
        for i in 0..self.ptes.len() {
            let pte = &mut self.ptes[i];
            if pte.valid() != 0 && pte.leaf() {
                pte.data = 0;
            } else if pte.valid() != 0 {
                let pt = unsafe { &mut *(pa2ka(pte.pa()) as *mut Pagetable) };
                pt.free(level - 1);

                unsafe {
                    kfree(pt);
                }

                pte.data = 0;
            }
        }
    }

    pub fn map(&mut self, va: usize, pa: usize, level: PtLevel, perm: u8) -> bool {
        let pte = match self.walk::<true>(va, level) {
            None => {
                return false;
            }
            Some((pte, _)) => unsafe { &mut *pte },
        };
        pte.set_pa(pa);
        pte.set_perm(perm);
        pte.validate();
        true
    }

    pub fn map_giga(&mut self, va: usize, pa: usize, perm: u8) {
        let vpn = vpn(PtLevel::Giga as usize, va);
        self.ptes[vpn].set_perm(perm);
        self.ptes[vpn].set_pa(pa);
        self.ptes[vpn].validate();
    }

    pub fn satp(&self) -> usize {
        let pn = &self.ptes[0] as *const _ as usize / sys::PAGESIZE;
        pn.set_bits(63, 60, SV39)
    }

    pub fn level2size(level: PtLevel) -> usize {
        match level {
            PtLevel::Normal => 4096,
            PtLevel::Mega => 1024 * 1024 * 2,
            PtLevel::Giga => 1024 * 1024 * 1024,
        }
    }
}

impl Drop for Pagetable {
    fn drop(&mut self) {
        self.free(PtLevel::Giga as usize);
    }
}

#[inline]
pub fn vm_fence() {
    unsafe {
        asm!("sfence.vma");
    }
}
