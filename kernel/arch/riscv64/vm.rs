use crate::bit::Bit;
use crate::kalloc::{kfree, zalloc_raw};
use crate::sys;
use crate::vm::{ka2pa, pa2hka, pa2ka, perm};
use core::arch::asm;

#[derive(Copy, Clone, PartialEq)]
pub enum PtLevel {
    Normal = 0,
    Mega = 1,
    Giga = 2,
}

impl PtLevel {
    pub fn size(self) -> usize {
        match self {
            Self::Normal => 4096,
            Self::Mega => 1024 * 1024 * 2,
            Self::Giga => 1024 * 1024 * 1024,
        }
    }

    pub fn next(self) -> PtLevel {
        match self {
            Self::Normal => Self::Normal,
            Self::Mega => Self::Normal,
            Self::Giga => Self::Mega,
        }
    }
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

    pub fn is_valid(&self) -> bool {
        self.valid() != 0
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
#[repr(C)]
pub struct Pagetable {
    pub ptes: [Pte; 512],
}

use crate::kalloc::Zero;

// Mark Pagetable as valid if initialized to all zeroes.
impl Zero for Pagetable {}

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
    ) -> Option<(&mut Pte, PtLevel)> {
        let mut pt = self;
        let mut level = PtLevel::Giga;
        while level != endlevel {
            let pte = &mut pt.ptes[vpn(level as usize, va)];
            if pte.leaf() {
                return Some((pte, level));
            } else if pte.valid() != 0 {
                pt = unsafe { &mut *(pa2ka(pte.pa()) as *mut Pagetable) };
            } else {
                if !ALLOC {
                    return None;
                }
                // Allocate an internal pagetable. This internal pagetable is owned by the current
                // pagetable via the physical address that gets stored in the PTE. Since this
                // ownership information is not available to Rust, we have to manage it manually
                // with raw pointers.
                let lower = match zalloc_raw::<Pagetable>() {
                    None => {
                        return None;
                    }
                    Some(pt) => pt,
                };
                pte.set_pa(ka2pa(lower.as_ptr() as usize));
                pte.set_valid(1);
                unsafe { pt = &mut *lower.as_ptr() };
            }
            level = level.next();
        }
        Some((&mut pt.ptes[vpn(endlevel as usize, va)], endlevel))
    }

    fn free(&mut self, level: usize) {
        // Iterate over internal pagetables and recursively free the raw pointers.
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

    #[must_use]
    pub fn map(&mut self, va: usize, pa: usize, level: PtLevel, perm: u8) -> bool {
        assert!(perm != 0);
        let pte = match self.walk::<true>(va, level) {
            None => {
                return false;
            }
            Some((pte, _)) => pte,
        };
        pte.set_pa(pa);
        pte.set_perm(perm);
        pte.validate();
        true
    }

    pub fn map_giga(&mut self, va: usize, pa: usize, perm: u8) {
        assert!(perm != 0);
        let vpn = vpn(PtLevel::Giga as usize, va);
        self.ptes[vpn].set_perm(perm);
        self.ptes[vpn].set_pa(pa);
        self.ptes[vpn].validate();
    }

    pub fn satp(&self) -> usize {
        let pn = ka2pa(&self.ptes[0] as *const _ as usize) / sys::PAGESIZE;
        pn.set_bits(63, 60, SV39)
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

use crate::board::machine;

pub fn kernel_procmap(pt: &mut Pagetable) {
    for mem in machine::MEM_RANGES {
        for pa in (mem.start..mem.start + mem.size).step_by(sys::gb(1) as usize) {
            pt.map_giga(pa2hka(pa), pa, perm::RWX);
        }
    }
}
