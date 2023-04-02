use kernel::arch::riscv64::fwi::wake_cores;
use kernel::cpu::cpu;
use kernel::kalloc::global::init_alloc;
use kernel::println;

use alloc::boxed::Box;

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
        unsafe { init_alloc(heap_start(), 4096 * 6) };
        wake_cores();
    }

    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );

    if !cpu().primary {
        return;
    }

    let x = Box::try_new([0u8; 4096]);
    if let Ok(y) = x {
        println!("{:p}", y);
    } else {
        println!("allocation failed");
    }
}
