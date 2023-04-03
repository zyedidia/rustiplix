use alloc::boxed::Box;
use core::ptr::addr_of_mut;
use kernel::arch::fwi::wake_cores;
use kernel::arch::timer;
use kernel::arch::trap::irq;
use kernel::cpu::cpu;
use kernel::kalloc::{init_alloc, kallocpage};
use kernel::println;

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

    let f = unsafe {
        let mut buf = Box::<Foo>::new_uninit();
        let f = buf.as_mut_ptr();
        addr_of_mut!((*f).i).write(10);
        buf.assume_init()
    };

    println!("{:p}", f);

    unsafe { irq::on() };

    timer::intr(timer::TIME_SLICE_US);
}
