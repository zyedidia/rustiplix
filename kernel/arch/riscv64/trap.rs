pub mod irq {
    use crate::arch::riscv64::csr::Sstatus;
    use crate::bit::Bit;

    pub unsafe fn on() {
        csr!(sstatus = csr!(sstatus).set_bit(Sstatus::Sie as usize, true));
    }

    pub unsafe fn off() {
        csr!(sstatus = csr!(sstatus).set_bit(Sstatus::Sie as usize, false));
    }

    pub fn enabled() -> bool {
        csr!(sstatus).bit(Sstatus::Sie as usize)
    }
}
