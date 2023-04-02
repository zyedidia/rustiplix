use kernel::arch::monitor::init::{enter_kernel, init_kernel, init_monitor};
use kernel::cpu::cpu;

pub struct BootData {
    pub entry: *mut u8,
    pub data: &'static [u8],
}

#[repr(C)]
struct Payload {
    entry: u64,
    size: u32,
    cksum: u32,
    data: u8,
}

#[cfg(feature = "uart")]
fn unpack() -> BootData {
    crate::uart::recv()
}

#[cfg(feature = "payload")]
fn unpack() -> BootData {
    extern "C" {
        static payload: Payload;
    }

    unsafe {
        use core::slice;
        use kernel::crc::crc32;

        let entry = payload.entry as *mut u8;
        let length = payload.size as usize;
        let data = slice::from_raw_parts(&payload.data as *const u8, length);

        assert!(payload.cksum == crc32(data));

        BootData { entry, data }
    }
}

static mut BOOT: BootData = BootData {
    entry: core::ptr::null_mut(),
    data: &[],
};

#[no_mangle]
pub extern "C" fn kmain() {
    init_monitor();
    enter_kernel();

    let primary = cpu().primary;
    let coreid = cpu().coreid;

    init_kernel(primary);

    let boot = if primary {
        let boot = unpack();
        assert!(!boot.data.is_empty());

        for i in (0..boot.data.len()).rev() {
            unsafe {
                boot.entry.add(i).write_volatile(boot.data[i]);
            }
        }
        // Store the boot data into a global so that the secondary cores can read it when they boot
        // up.
        unsafe {
            BOOT = boot;
            &BOOT
        }
    } else {
        unsafe { &BOOT }
    };

    kernel::sync::fence::insn_fence();

    let entry = boot.entry as *const ();
    let func: extern "C" fn(coreid: usize) -> ! = unsafe { core::mem::transmute(entry) };
    func(coreid);
}
