use crate::arch::riscv64::csr::{Priv, Sie, Sstatus};
use crate::arch::riscv64::vm::Pagetable;
use crate::bit::Bit;
use crate::board::virt::machine;
use crate::primary::PrimaryCell;
use crate::sys;

pub fn init_monitor() {
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

static PAGETABLE: PrimaryCell<Pagetable> = PrimaryCell::new(Pagetable::new());

pub fn init_kernel(primary: bool) {
    use crate::arch::riscv64::vm::vm_fence;
    use crate::vm::{pa2ka, perm};

    if primary {
        // If primary core, create mappings for the initial pagetable.
        let map_giga = |pa: usize| unsafe {
            let pt = PAGETABLE.get_mut();
            pt.map_giga(pa, pa, perm::READ | perm::WRITE | perm::EXEC);
            pt.map_giga(pa2ka(pa), pa, perm::READ | perm::WRITE | perm::EXEC);
        };

        for mem in machine::MEM_RANGES {
            for pa in (mem.start..mem.start + mem.size).step_by(sys::gb(1) as usize) {
                map_giga(pa);
            }
        }
    }

    // Enable virtual memory with an identity-mapped pagetable.
    csr!(satp = PAGETABLE.satp());
    vm_fence();

    // Prepare to enable interrupts (will only be enabled when sstatus is written as well).
    csr!(sie = (1 << Sie::Stie as usize) | (1 << Sie::Ssie as usize));

    // Enable SUM bit so supervisor can access user-mode pages.
    csr!(sstatus = csr!(sstatus) | (1 << Sstatus::Sum as usize))
}
