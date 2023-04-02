use crate::dev::irq::sfclint::SifiveClint;
use crate::dev::uart::virt::VirtUart;
use crate::dev::uart::Uart;
use crate::vm::pa2ka;

use crate::sync::spinlock::SpinLock;

pub static UART: SpinLock<Uart<VirtUart>> =
    SpinLock::new(Uart::<VirtUart>::new(pa2ka(0x10000000) as *mut VirtUart));
pub static CLINT: &SifiveClint = unsafe { &*(pa2ka(0x200_0000) as *const SifiveClint) };

pub mod machine {
    use crate::sys;

    pub const NCORES: usize = 4;
    pub const MTIME_FREQ: u64 = 3_580_000 * 2;

    pub struct MemRange {
        pub start: usize,
        pub size: usize,
    }

    pub const MAIN_MEMORY: MemRange = MemRange {
        start: 0x8000_0000,
        size: sys::gb(2) as usize,
    };

    pub const MEM_RANGES: [MemRange; 2] = [
        MemRange {
            start: 0,
            size: sys::gb(2) as usize,
        },
        MAIN_MEMORY,
    ];
}
