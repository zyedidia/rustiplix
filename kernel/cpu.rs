#[derive(Copy, Clone)]
pub struct Cpu {
    pub coreid: u32,
}

static mut CPUS: [Cpu; 4] = [Cpu { coreid: 0 }; 4];

pub fn cpu() -> &'static Cpu {
    use crate::arch::riscv64::cpu::rd_cpu;
    rd_cpu()
}

/// # Safety
///
/// Initializes the core-local CPU struct. This function must be called before `cpu` is used.
pub unsafe fn init_cpu(coreid: u32) {
    use crate::arch::riscv64::cpu::wr_cpu;

    CPUS[coreid as usize].coreid = coreid;

    wr_cpu(&CPUS[coreid as usize]);
}
