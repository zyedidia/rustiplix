use crate::sys;

pub mod perm {
    pub const READ: u8 = 1 << 0;
    pub const WRITE: u8 = 1 << 1;
    pub const EXEC: u8 = 1 << 2;
    pub const USER: u8 = 1 << 3;
    pub const COW: u8 = 1 << 4;
}

#[inline]
pub const fn iska(va: usize) -> bool {
    va >= sys::HIGHMEM_BASE
}

#[inline]
pub const fn ka2pa(ka: usize) -> usize {
    ka - sys::HIGHMEM_BASE
}

#[inline]
pub const fn pa2ka(pa: usize) -> usize {
    // In the future this function should be statically different at compile-time depending on
    // whether the monitor or kernel is being compiled.
    pa + sys::HIGHMEM_BASE
}

#[inline]
pub const fn pa2hka(pa: usize) -> usize {
    pa + sys::HIGHMEM_BASE
}
