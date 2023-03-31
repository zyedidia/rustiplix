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

pub enum Sstatus {
    Sie = 1,
    Spie = 5,
    Spp = 8,
    Sum = 18,
}

pub enum Sie {
    Seie = 9,
    Stie = 5,
    Ssie = 1,
}
