use core::panic::PanicInfo;

use crate::builtin::backtrace::backtrace;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    println!("backtrace:");
    let unwind = backtrace();
    for addr in unwind {
        if let Some(val) = addr {
            println!("  {:#x}", val);
        } else {
            break;
        }
    }

    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}
