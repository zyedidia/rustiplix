#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Regs {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

impl Regs {
    pub fn arg0(&self) -> usize {
        self.a0
    }
    pub fn arg1(&self) -> usize {
        self.a1
    }
    pub fn arg2(&self) -> usize {
        self.a2
    }
    pub fn set_ret(&mut self, val: usize) {
        self.a0 = val;
    }
}

use core::arch::asm;

pub fn rd_tp() -> u64 {
    let value: u64;
    unsafe {
        asm!("mv {}, tp", out(reg) value);
    }
    value
}

pub fn rd_gp() -> u64 {
    let value: u64;
    unsafe {
        asm!("mv {}, gp", out(reg) value);
    }
    value
}

use super::trap::Trapframe;
use core::ptr::null_mut;

#[repr(C)]
pub struct Context {
    pub ra: usize,
    pub sp: usize,

    // callee-saved
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    pub satp: usize,
    pub proc: *mut Trapframe,
}

use super::vm::Pagetable;

impl Context {
    pub const fn zero() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            satp: 0,
            proc: null_mut(),
        }
    }

    pub fn set_pt(&mut self, pt: &Pagetable) {
        self.satp = pt.satp();
    }

    pub fn new(sp: usize, ra: usize) -> Self {
        Self {
            ra,
            sp,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            satp: 0,
            proc: null_mut(),
        }
    }
}
