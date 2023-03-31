use crate::arch::riscv64::csr::cause;
use crate::arch::riscv64::fwi::func;
use crate::arch::riscv64::regs::Regs;

#[no_mangle]
pub extern "C" fn monitortrap(regs: &Regs) {
    let mcause = csr!(mcause);

    match mcause {
        cause::ECALL_S => {
            // TODO: handle fwi functions in a separate handler
            match regs.a7 {
                func::WAKE_CORES => {
                    extern "C" {
                        static mut wakeup: i32;
                    }
                    unsafe {
                        wakeup = 1;
                    }
                }
                _ => {
                    panic!("invalid ecall");
                }
            }

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
