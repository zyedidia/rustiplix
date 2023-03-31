use kernel::cpu::cpu;
use kernel::println;

#[no_mangle]
pub extern "C" fn kmain() {
    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );
}
