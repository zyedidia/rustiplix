use crate::arch::riscv64::cpu::rd_cpu;
use crate::board::virt;

// A global that becomes true when the secondary cores boot up.
static mut BOOTED_ALL: bool = false;

// Safe because this will only be able to write to BOOTED_ALL if the secondary cores have not yet
// booted.
pub fn set_booted_all() {
    unsafe {
        assert!(!BOOTED_ALL);
        BOOTED_ALL = true;
    }
}

// Safe because if BOOTED_ALL is false, there is no possible race (only one core), and if
// BOOTED_ALL is true, it is impossible for set_booted_all to write to the global.
pub fn booted_all() -> bool {
    unsafe { BOOTED_ALL }
}

#[derive(Copy, Clone)]
pub struct Cpu {
    pub coreid: usize,
    pub primary: bool,
}

static mut CPUS: [Cpu; virt::machine::NCORES] = [Cpu {
    coreid: 0,
    primary: false,
}; virt::machine::NCORES];

// Initializes the core-local CPU state for coreid (the current core).
pub unsafe fn init_cpu(coreid: usize, primary: bool) {
    use crate::arch::riscv64::cpu::wr_cpu;
    CPUS[coreid].coreid = coreid;
    CPUS[coreid].primary = primary;
    wr_cpu(&mut CPUS[coreid]);
}

// Get the core-local CPU struct without any guard. Requires that interrupts are disabled.
pub unsafe fn cpu_noguard<'a>() -> &'a Cpu {
    rd_cpu()
}

pub fn cpu<'a>() -> CpuGuard<'a> {
    CpuGuard::new()
}

pub struct CpuGuard<'a> {
    cpu: &'a mut Cpu,
    was_en: bool,
}

use crate::arch::riscv64::trap::irq;
use core::ops::Deref;

impl CpuGuard<'_> {
    pub fn new() -> Self {
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
