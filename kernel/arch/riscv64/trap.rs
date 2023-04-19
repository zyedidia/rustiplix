pub mod irq {
    use crate::arch::riscv64::csr::sstatus;
    use crate::bit::Bit;

    pub fn init() {
        csr!(stvec = super::kernelvec);
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
use crate::trap;
use alloc::boxed::Box;

#[no_mangle]
/// When a trap occurs in kernel mode, execution switches to here.
pub extern "C" fn kerneltrap() {
    let sepc = csr!(sepc);
    let scause = csr!(scause);

    // println!("[kernel trap] sepc: {:#x}, cause: {:#x}", sepc, scause,);

    if scause == cause::STI {
        // Timer interrupt.
        trap::irq_handler_kern(trap::Irq::Timer);
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

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Trapframe {
    ktp: u64,
    ksp: u64,
    kgp: u64,
    pub epc: usize,
    pub regs: Regs,
}

use crate::proc::Proc;
use crate::syscall::syscall;

extern "C" {
    // Assembly routine for returning to user-mode.
    fn userret(p: *mut Trapframe) -> !;
    // Assembly entrypoint for user traps.
    fn uservec();
    // Assembly entrypoint for kernel traps.
    fn kernelvec();
}

#[no_mangle]
/// Called when a user process experiences a trap.

// Clippy wants us to mark this function as unsafe because it dereferences a raw pointer, but it
// isn't ever called by Rust code (only directly by assembly), so it would unnecessarily sacrifice
// some safety checking to mark the whole function as unsafe.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn usertrap(p: *mut Proc) {
    // Install the kernel trap handler.
    csr!(stvec = kernelvec);

    let mut p = unsafe { Box::<Proc>::from_raw(p) };
    p.watch_canary();

    // println!(
    //     "[user trap] epc: {:#x}, cause: {:#x}",
    //     csr!(sepc),
    //     csr!(scause)
    // );

    let cause = csr!(scause);

    match cause {
        cause::ECALL_U => {
            // System call.
            let sysno = p.trapframe.regs.a7;
            p.trapframe.epc = csr!(sepc) + 4;
            p.trapframe.regs.a0 = syscall(&mut p, sysno) as usize;
        }
        cause::STI => {
            // Timer interrupt.
            trap::irq_handler_user(&mut p, trap::Irq::Timer);
        }
        _ => {
            panic!(
                "[unhandled] usertrap: core: {}: cause: {:#x}, epc: {:#x}, tval: {:#x}",
                cpu().coreid,
                cause,
                csr!(sepc),
                csr!(stval)
            );
        }
    }

    unsafe { usertrapret(Box::<Proc>::into_raw(p)) };
}

use super::csr::sstatus;
use super::vm::vm_fence;
use crate::bit::Bit;

/// Return to user-mode and execute process 'p'.
pub unsafe fn usertrapret(p: *mut Proc) -> ! {
    // Disable interrupts to set up user-mode.
    irq::off();

    // Reset trap handler.
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
    // Set exception return address.
    csr!(sepc = (*p).trapframe.epc);
    // Switch pagetables.
    csr!(satp = (*p).data.pt.satp());
    vm_fence();

    userret(&raw mut (*p).trapframe);
}
