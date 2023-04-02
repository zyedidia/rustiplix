use kernel::arch::riscv64::fwi::wake_cores;
use kernel::arch::riscv64::timer;
use kernel::arch::riscv64::trap::irq;
use kernel::cpu::cpu;
use kernel::kalloc::global::{init_alloc, kallocpage};
use kernel::println;

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

    irq::init();

    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );

    if !cpu().primary {
        return;
    }

    let x = kallocpage();
    if let Ok(y) = x {
        println!("allocated page: {:p}", y);
    } else {
        println!("allocation failed");
    }

    unsafe { irq::on() };

    timer::intr(timer::TIME_SLICE_US);
}
