use crate::arch::riscv64::cpu::rd_cpu;

#[derive(Copy, Clone)]
pub struct Cpu {
    pub coreid: usize,
}

static mut CPUS: [Cpu; 4] = [Cpu { coreid: 0 }; 4];

// Initializes the core-local CPU state for coreid (the current core).
pub unsafe fn init_cpu(coreid: usize) {
    use crate::arch::riscv64::cpu::wr_cpu;
    CPUS[coreid].coreid = coreid;
    wr_cpu(&mut CPUS[coreid]);
}

// Get the core-local CPU struct without any guard. Requires that interrupts are disabled.
pub unsafe fn get_cpu() -> &'static Cpu {
    rd_cpu()
}

pub struct CpuGuard<'a> {
    cpu: &'a mut Cpu,
    was_en: bool,
}

use crate::arch::riscv64::trap::irq;
use core::ops::Deref;

impl CpuGuard<'_> {
    pub fn new() -> Self {
        // TODO:
        // * get current irqs status
        // * disable irqs
        let was_en = irq::enabled();
        unsafe {
            irq::off();
        }
        Self {
            // Safety: we know interrupts are disabled now.
            cpu: unsafe { rd_cpu() },
            was_en,
        }
    }
}

impl Default for CpuGuard<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CpuGuard<'_> {
    type Target = Cpu;
    fn deref(&self) -> &Cpu {
        self.cpu
    }
}

impl Drop for CpuGuard<'_> {
    fn drop(&mut self) {
        if self.was_en {
            unsafe { irq::on() }
        }
    }
}
