pub mod irq {
    use crate::arch::riscv64::csr::Sstatus;
    use crate::bit::Bit;

    pub fn init() {
        extern "C" {
            fn kernelvec();
        }

        csr!(stvec = kernelvec as usize);
    }

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

use crate::arch::riscv64::csr::cause;
use crate::cpu::cpu;

#[no_mangle]
pub extern "C" fn kerneltrap() {
    let sepc = csr!(sepc);
    let scause = csr!(scause);

    println!("[kernel trap] sepc: {:#x}, cause: {:#x}", sepc, scause,);

    if scause == cause::STI {
        use crate::arch::riscv64::timer;
        timer::intr(timer::TIME_SLICE_US);
    } else {
        panic!(
            "[unhandled kernel trap] core: {}, epc: {:#x}, cause: {:#x}, stval: {:#x}",
            cpu().coreid,
            sepc,
            scause,
            csr!(stval)
        );
    }
}
