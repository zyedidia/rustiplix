#[macro_export]
macro_rules! csr {
    ($name:ident) => {{
        #[cfg(target_arch = "riscv64")]
        {
            let value: usize;
            unsafe {
                llvm_asm!("csrr $0, $1" : "=r"(value) : "i"($name));
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
            unsafe {
                llvm_asm!("csrw $0, $1" :: "i"($name), "r"($val));
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            panic!("This macro can only be used on RISC-V platforms");
        }
    }};
}
