use kernel::cpu::cpu;
use kernel::println;
use kernel::sys::ALLOCATOR;

fn heap_start() -> *mut u8 {
    unsafe {
        extern "C" {
            static mut _heap_start: u8;
        }
        &mut _heap_start as *mut u8
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    if cpu().primary {
        let mut a = ALLOCATOR.lock();
        a.init(heap_start(), 4096 * 16);
    }

    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );

    unsafe {
        use core::arch::asm;
        asm!("ecall");
    }
}
