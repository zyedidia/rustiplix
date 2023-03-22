#[derive(Copy, Clone)]
pub struct Cpu {
    pub coreid: u32,
}

static mut CPUS: [Cpu; 4] = [Cpu { coreid: 0 }; 4];

// Initializes the core-local CPU state for coreid (the current core).
pub unsafe fn init_cpu(coreid: u32) {
    use crate::arch::riscv64::cpu::wr_cpu;
    CPUS[coreid as usize].coreid = coreid;
    wr_cpu(&mut CPUS[coreid as usize]);
}

pub struct CpuGuard<'a> {
    cpu: &'a mut Cpu,
    was_en: bool,
}

use core::ops::Deref;

impl CpuGuard<'_> {
    pub fn new() -> Self {
        // TODO:
        // * get current irqs status
        // * disable irqs
        use crate::arch::riscv64::cpu::rd_cpu;
        Self {
            // Safety: we know interrupts are disabled now.
            cpu: unsafe { rd_cpu() },
            was_en: false,
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
            // TODO: re-enable irqs
        }
    }
}
