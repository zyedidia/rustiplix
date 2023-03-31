use kernel::cpu::cpu;
use kernel::println;
use kernel::sys::ALLOCATOR;
use kernel::vm;

#[no_mangle]
pub extern "C" fn kmain() {
    println!(
        "core: {}, entered kmain at: {:?}",
        cpu().coreid,
        &kmain as *const _
    );

    let mut a = ALLOCATOR.lock();
    a.init(vm::pa2ka(0x9000_0000) as *mut u8, 4096 * 16);
    let x = a.alloc(4096);
    println!("allocated: {:p}", x);
    a.dealloc(x);
    println!("allocated: {:p}", a.alloc(4096));
    println!("allocated: {:p}", a.alloc(4096));
    println!("allocated: {:p}", a.alloc(4096));
    println!("allocated: {:p}", a.alloc(4096));
    println!("allocated: {:p}", a.alloc(4096));
}
