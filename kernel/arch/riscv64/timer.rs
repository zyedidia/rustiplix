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
