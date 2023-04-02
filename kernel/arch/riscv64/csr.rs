macro_rules! csr {
    ($name:ident) => {{
        #[cfg(target_arch = "riscv64")]
        {
            let value: usize;
            #[allow(unused_unsafe)]
            unsafe {
                use ::core::arch::asm;
                asm!(concat!("csrr ", "{}, ", stringify!($name)), out(reg) value);
            }
            value
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            panic!("This macro can only be used on RISC-V platforms");
        }
    }};

    ($name:ident = $val:expr) => {{
        #[cfg(target_arch = "riscv64")]
        {
            let value = $val;
            #[allow(unused_unsafe)]
            unsafe {
                use ::core::arch::asm;
                asm!(concat!("csrw ", stringify!($name), ", {}"), in(reg) value);
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            panic!("This macro can only be used on RISC-V platforms");
        }
    }};
}

pub enum Priv {
    U = 0b00,
    S = 0b01,
    M = 0b11,
}

pub mod sstatus {
    pub const SIE: usize = 1;
    pub const SPIE: usize = 5;
    pub const SPP: usize = 8;
    pub const SUM: usize = 18;
}

pub mod sie {
    pub const SEIE: usize = 9;
    pub const STIE: usize = 5;
    pub const SSIE: usize = 1;
}

pub mod mip {
    pub const STIP: usize = 5;
    pub const MTIP: usize = 7;
}

pub mod mie {
    pub const STIE: usize = 5;
    pub const MTIE: usize = 7;
}

pub mod cause {
    // interrupts
    // software timer interrupt
    pub const STI: usize = 0x8000000000000005;
    // machine timer interrupt
    pub const MTI: usize = 0x8000000000000007;

    // exceptions
    pub const ILLEGAL: usize = 2;
    pub const BREAKPOINT: usize = 3;
    pub const ECALL_U: usize = 8;
    pub const ECALL_S: usize = 9;
    pub const ECALL_M: usize = 11;
    pub const WPGFLT: usize = 15;
}
