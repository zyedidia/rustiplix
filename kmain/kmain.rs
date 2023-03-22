use kernel::println;

#[no_mangle]
pub extern "C" fn kmain() {
    use kernel::arch::riscv64::monitor::init::enter_smode;
    use kernel::cpu::cpu;
    enter_smode();

    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );
}
