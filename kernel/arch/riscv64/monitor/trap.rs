use crate::arch::riscv64::csr::{cause, mie, mip};
use crate::arch::riscv64::fwi::Func;
use crate::arch::riscv64::regs::Regs;
use crate::bit::Bit;

#[no_mangle]
pub extern "C" fn monitortrap(regs: &mut Regs) {
    let mcause = csr!(mcause);

    match mcause {
        cause::ECALL_S => {
            fwi_handler(regs);
            csr!(mepc = csr!(mepc) + 4);
        }
        cause::MTI => {
            csr!(mie = csr!(mie).set_bit(mie::MTIE, false));
            csr!(mip = csr!(mip).set_bit(mip::STIP, true));
        }
        _ => {
            panic!(
                "[unhandled] monitortrap: core: {}, cause: {:#x}, epc: {:#x}, mtval: {:#x}",
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
        x if x == Func::WakeCores as u64 => {
            wake_cores();
        }
        x if x == Func::SetTimer as u64 => {
            set_timer(regs.a0);
        }
        _ => {
            // error
            regs.a0 = u64::MAX;
            return;
        }
    }
    regs.a0 = 0;
}

fn wake_cores() {
    extern "C" {
        static mut wakeup: i32;
    }
    unsafe {
        wakeup = 1;
    }
}

fn set_timer(stime_value: u64) {
    use crate::board::virt::CLINT;

    CLINT.wr_mtimecmp(stime_value);
    csr!(mip = csr!(mip).set_bit(mip::STIP, false));
    csr!(mie = csr!(mie).set_bit(mie::MTIE, true));
}
