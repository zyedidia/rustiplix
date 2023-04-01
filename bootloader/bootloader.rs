use kernel::arch::riscv64::monitor::init::{enter_smode, init_kernel, init_monitor};
use kernel::cpu::cpu;

use core::slice;

struct BootData {
    entry: *mut u8,
    data: &'static [u8],
}

#[repr(C)]
struct Payload {
    entry: u64,
    size: u32,
    cksum: u32,
    data: u8,
}

fn unpack() -> BootData {
    extern "C" {
        static payload: Payload;
    }

    unsafe {
        let entry = payload.entry as *mut u8;
        let length = payload.size as usize;

        BootData {
            entry: entry,
            data: slice::from_raw_parts(&payload.data as *const u8, length),
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    init_monitor();
    enter_smode();

    let primary = cpu().primary;
    let coreid = cpu().coreid;

    init_kernel(primary);

    let boot = unpack();
    assert!(boot.data.len() > 0);

    for i in (0..boot.data.len()).rev() {
        unsafe {
            boot.entry.add(i).write_volatile(boot.data[i]);
        }
    }

    let entry = boot.entry as *const ();
    let func: extern "C" fn(coreid: usize) -> ! = unsafe { core::mem::transmute(entry) };
    func(coreid);
}