use kernel::println;

#[no_mangle]
pub extern "C" fn kmain() {
    use kernel::arch::riscv64::monitor::init::enter_smode;
    use kernel::cpu::CpuGuard;
    enter_smode();

    println!(
        "core: {}, entered kmain at: {:?}",
        CpuGuard::new().coreid,
        &kmain as *const _
    );
}
