use kernel::arch::fwi::wake_cores;
use kernel::arch::timer;
use kernel::arch::trap::irq;
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
        unsafe { init_alloc(heap_start(), 4096 * 4096) };
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

    let x = kallocpage().unwrap();

    use alloc::boxed::Box;
    let a = Box::new(1);
    let b = Box::new(2);
    let c = Box::new(3);
    println!("{:p} {:p} {:p} {:p}", a, b, c, x);

    unsafe { irq::on() };

    timer::intr(timer::TIME_SLICE_US);
}
