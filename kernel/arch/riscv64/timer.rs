use crate::arch::riscv64::fwi;
use crate::board::virt;

pub fn cycles() -> u64 {
    csr!(cycle) as u64
}

pub fn freq() -> u64 {
    virt::machine::MTIME_FREQ
}

pub fn time() -> u64 {
    virt::CLINT.rd_mtime()
}

pub const TIME_SLICE_US: u64 = 100_000;

pub fn intr(us: u64) {
    let next = time() + freq() / 1_000_000 * us;
    fwi::set_timer(next);
}
