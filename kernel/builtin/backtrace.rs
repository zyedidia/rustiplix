use core::arch::asm;

const MAX_BACKTRACE_ADRESSES: usize = 10;

fn is_valid(ptr: usize) -> bool {
    crate::vm::iska(ptr)
}

/// Get an array of backtrace addresses.
///
/// This needs `force-frame-pointers` enabled.
pub fn backtrace() -> [Option<usize>; MAX_BACKTRACE_ADRESSES] {
    let fp = unsafe {
        let mut _tmp: u64;
        asm!("mv {0}, x8", out(reg) _tmp);
        _tmp
    };

    backtrace_internal(fp, 0)
}

pub fn backtrace_internal(fp: u64, suppress: i32) -> [Option<usize>; MAX_BACKTRACE_ADRESSES] {
    let mut result = [None; 10];
    let mut index = 0;

    let mut fp = fp;
    let mut suppress = suppress;
    let mut old_address = 0;
    loop {
        unsafe {
            if !is_valid(fp as usize) {
                break;
            }
            let address = (fp as *const u64).offset(-1).read_volatile(); // RA/PC
            fp = (fp as *const u64).offset(-2).read_volatile(); // next FP

            if old_address == address {
                break;
            }

            old_address = address;

            if address == 0 {
                break;
            }

            if suppress == 0 {
                result[index] = Some(address as usize);
                index += 1;

                if index >= MAX_BACKTRACE_ADRESSES {
                    break;
                }
            } else {
                suppress -= 1;
            }
        }
    }

    result
}
