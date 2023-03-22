use crate::arch::riscv64::csr::Priv;
use crate::bit::Bit;

pub fn init() {
    csr!(mcounteren = 0b111);
}

extern "C" {
    fn _enter_smode();
}

pub fn enter_smode() {
    // Write S-mode to mstatus.MPP.
    csr!(mstatus = csr!(mstatus).set_bits(12, 11, Priv::S as usize));
    // Disable paging.
    csr!(satp = 0);
    // Delegate certain interrupts and exceptions to S-mode.
    csr!(medeleg = 0x00f0b501);
    csr!(mideleg = 0x00001666);

    // Configure the PMP to allow all accesses for S-mode. Uses a TOR region to allow R/W/X
    // starting at 0x0 and ending at 0xffff_ffff_ffff.
    csr!(pmpcfg0 = 0b0001111);
    csr!(pmpaddr0 = 0xffff_ffff_ffffu64);

    // Call asm function that performs actual transition.
    unsafe {
        _enter_smode();
    }
}
