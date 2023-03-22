use crate::arch::riscv64::trap::irq;
use crate::cpu::cpu;

pub struct SifiveClint {}

impl SifiveClint {
    pub fn rd_mtime(&self) -> u64 {
        let base = self as *const _ as *const u8;
        unsafe {
            let mtime = base.add(0xbff8) as *const u64;
            mtime.read_volatile()
        }
    }

    pub fn wr_mtimecmp(&self, val: u64) {
        assert!(!irq::enabled());
        let base = self as *const _ as *const u8;
        unsafe {
            let mtimecmp = base.add(0x4000 + 8 * cpu().coreid) as *mut u64;
            mtimecmp.write_volatile(val);
        }
    }
}
