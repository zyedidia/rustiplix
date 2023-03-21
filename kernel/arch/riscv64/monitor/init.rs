use crate::arch::riscv64::csr::Priv;
use crate::bit::Bit;

pub fn enter_smode() {
    csr!(mstatus = csr!(mstatus).set_bits(12, 11, Priv::S as usize));
}
