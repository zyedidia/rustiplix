use crate::dev::irq::sfclint::SifiveClint;
use crate::dev::uart::dwapb::DwApbUart;
use crate::dev::uart::Uart;
use crate::vm::pa2ka;

use crate::sync::spinlock::SpinLock;

use crate::sys;

pub static UART: SpinLock<Uart<DwApbUart>> =
    SpinLock::new(Uart::<DwApbUart>::new(pa2ka(0x10000000) as *mut DwApbUart));
pub static CLINT: &SifiveClint = unsafe { &*(pa2ka(0x200_0000) as *const SifiveClint) };

pub mod machine {
    pub const CPU_FREQ: u64 = 1_250_000_000;
    pub const NCORES: usize = 5;
    pub const MTIME_FREQ: u64 = 4_000_000;

    pub struct MemRange {
        pub start: usize,
        pub size: usize,
    }

    pub const MAIN_MEMORY: MemRange = MemRange {
        start: 0x4000_0000,
        size: sys::gb(3) as usize,
    };

    pub const MEM_RANGES: [MemRange; 2] = [
        MemRange {
            start: 0,
            size: sys::gb(1) as usize,
        },
        MAIN_MEMORY,
    ];
}
