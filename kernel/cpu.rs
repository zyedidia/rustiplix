use crate::arch::cpu::rd_cpu;
use crate::board;

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
    pub stack: usize,
}

static mut CPUS: [Cpu; board::machine::NCORES] = [Cpu {
    coreid: 0,
    primary: false,
    stack: 0,
}; board::machine::NCORES];

// Initializes the core-local CPU state for coreid (the current core).
pub unsafe fn init_cpu(coreid: usize, primary: bool) {
    extern "C" {
        static _stack_start: u8;
    }

    use crate::arch::cpu::wr_cpu;
    CPUS[coreid].coreid = coreid;
    CPUS[coreid].primary = primary;
    CPUS[coreid].stack = (&_stack_start as *const _ as usize) + (coreid + 1) * 4096;
    wr_cpu(&mut CPUS[coreid]);
}

// Get the core-local CPU struct without any guard. Requires that interrupts are disabled.
pub unsafe fn cpu_noguard() -> &'static mut Cpu {
    rd_cpu()
}

pub fn cpu() -> CpuGuard {
    CpuGuard::new()
}

pub struct CpuGuard {
    cpu: &'static mut Cpu,
    was_en: bool,
}

use crate::arch::trap::irq;
use core::ops::Deref;

impl CpuGuard {
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

impl Default for CpuGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CpuGuard {
    type Target = Cpu;
    fn deref(&self) -> &Cpu {
        self.cpu
    }
}

impl Drop for CpuGuard {
    fn drop(&mut self) {
        if self.was_en {
            unsafe { irq::on() }
        }
    }
}
