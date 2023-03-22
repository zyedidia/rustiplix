use kernel::arch::riscv64::monitor::init::{enter_smode, init};
use kernel::cpu::CpuGuard;
use kernel::println;

#[no_mangle]
pub extern "C" fn kmain() {
    init();
    enter_smode();

    println!(
        "core: {}, entered kmain at: {:?}",
        CpuGuard::new().coreid,
        &kmain as *const _
    );

    println!("waiting...");
    use kernel::timer;
    timer::delay_cycles(1000000000);
    println!("done");
}
