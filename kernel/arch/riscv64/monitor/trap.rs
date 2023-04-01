use crate::arch::riscv64::csr::cause;
use crate::arch::riscv64::fwi::func;
use crate::arch::riscv64::regs::Regs;

#[no_mangle]
pub extern "C" fn monitortrap(regs: &mut Regs) {
    let mcause = csr!(mcause);

    match mcause {
        cause::ECALL_S => {
            fwi_handler(regs);
            csr!(mepc = csr!(mepc) + 4);
        }
        _ => {
            panic!(
                "[unhandled] monitortrap: core: {}, cause: {}, epc: {:#x}, mtval: {:#x}",
                csr!(mhartid),
                mcause,
                csr!(mepc),
                csr!(mtval)
            );
        }
    }
}

fn fwi_handler(regs: &mut Regs) {
    match regs.a7 {
        func::WAKE_CORES => {
            wake_cores();
        }
        _ => {
            regs.a0 = u64::MAX;
        }
    }
}

fn wake_cores() {
    extern "C" {
        static mut wakeup: i32;
    }
    unsafe {
        wakeup = 1;
    }
}
