use kernel::arch::riscv64::monitor::init::{enter_smode, init};
use kernel::cpu::CpuGuard;
use kernel::primary::PrimaryCell;
use kernel::println;

static Y: PrimaryCell<i32> = PrimaryCell::new(42);

#[no_mangle]
pub extern "C" fn kmain() {
    init();
    enter_smode();

    unsafe {
        let y = Y.get_mut();
        *y = 12;
    }
    println!("{}", *Y);

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
