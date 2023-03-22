use crate::board::virt;

pub fn cycles() -> u64 {
    csr!(cycle) as u64
}

// pub fn freq() -> u64 {
//
// }

pub fn time() -> u64 {
    virt::CLINT.rd_mtime()
}
