use crate::kalloc::pg::PageAlloc;
use crate::sync::spinlock::SpinLock;

pub const fn gb(n: u64) -> u64 {
    1024 * 1024 * 1024 * n
}

pub const fn mb(n: u64) -> u64 {
    1024 * 1024 * n
}

pub const HIGHMEM_BASE: usize = 0xffff_ffc0_0000_0000;
pub const PAGESIZE: usize = 4096;

pub static ALLOCATOR: SpinLock<PageAlloc> = SpinLock::new(PageAlloc::new_uninit());
