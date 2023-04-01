use kernel::arch::riscv64::fwi::wake_cores;
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
        unsafe { a.init(heap_start(), 4096 * 16) };
        wake_cores();
    }

    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );

    // use alloc::boxed::Box;
    // let x = Box::new(1);
    // println!("{}", x);
}
