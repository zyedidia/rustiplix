use kernel::arch::timer;
use kernel::arch::trap::irq;
use kernel::arch::trap::usertrapret;
use kernel::cpu::cpu;
use kernel::kalloc::init_alloc;
use kernel::println;
use kernel::proc::Proc;

use alloc::boxed::Box;

struct Foo {
    i: i64,
    data: [u64; 4096],
}

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
        // use kernel::arch::fwi::wake_cores;
        // wake_cores();
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

    let hello = include_bytes!("../user/hello/hello.elf");

    let proc = Proc::new_boxed(hello).unwrap();

    unsafe { irq::on() };

    timer::intr(timer::TIME_SLICE_US);

    unsafe { usertrapret(Box::<Proc>::into_raw(proc)) };
}
