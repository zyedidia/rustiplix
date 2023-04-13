use crate::arch::riscv64::fwi;
use crate::board;

pub fn cycles() -> u64 {
    csr!(cycle) as u64
}

pub fn freq() -> u64 {
    board::machine::MTIME_FREQ
}

pub fn time() -> u64 {
    board::CLINT.rd_mtime()
}

pub const TIME_SLICE_US: u64 = 10_000;

pub fn intr(us: u64) {
    let next = time() + freq() / 1_000_000 * us;
    fwi::set_timer(next);
}
