pub mod irq {
    use crate::arch::riscv64::csr::sstatus;
    use crate::bit::Bit;

    pub fn init() {
        extern "C" {
            fn kernelvec();
        }

        csr!(stvec = kernelvec);
    }

    pub unsafe fn on() {
        csr!(sstatus = csr!(sstatus).set_bit(sstatus::SIE, true));
    }

    pub unsafe fn off() {
        csr!(sstatus = csr!(sstatus).set_bit(sstatus::SIE, false));
    }

    pub fn enabled() -> bool {
        csr!(sstatus).bit(sstatus::SIE)
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

use super::regs::{rd_gp, rd_tp, Regs};

#[repr(C)]
pub struct Trapframe {
    ktp: u64,
    ksp: u64,
    kgp: u64,
    epc: usize,
    regs: Regs,
}

use crate::proc::Proc;

extern "C" {
    fn userret(p: &mut Trapframe) -> !;
    fn uservec();
}

#[no_mangle]
pub extern "C" fn usertrap(p: *mut Proc) {
    println!(
        "[user trap] epc: {:#x}, cause: {:#x}",
        csr!(sepc),
        csr!(scause)
    );

    unsafe { usertrapret(p) };
}

use super::csr::sstatus;
use super::vm::vm_fence;
use crate::bit::Bit;

unsafe fn usertrapret(p: *mut Proc) -> ! {
    irq::off();

    csr!(stvec = uservec);

    // Set up trapframe.
    (*p).trapframe.ktp = rd_tp();
    (*p).trapframe.ksp = Proc::kstackp(p) as u64;
    (*p).trapframe.kgp = rd_gp();
    csr!(sscratch = p);

    csr!(
        sstatus = csr!(sstatus)
            .set_bit(sstatus::SPP, false) // force return to user mode
            .set_bit(sstatus::SPIE, true) // enable interrupts in user mode
    );
    csr!(sepc = (*p).trapframe.epc);
    csr!(satp = (*p).data.pt.satp());
    vm_fence();

    userret(&mut (*p).trapframe);
}
