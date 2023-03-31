use crate::dev::irq::sfclint::SifiveClint;
use crate::dev::uart::virt::VirtUart;
use crate::dev::uart::Uart;

use crate::sync::spinlock::SpinLock;

// TODO: in the future we should use pa2ka to construct addresses so that these devices are usable
// from a high kernel mapping.
pub static UART: SpinLock<Uart<VirtUart>> = SpinLock::new(Uart::<VirtUart>::new(0x10000000));
pub static CLINT: &SifiveClint = unsafe { &*(0x200_0000 as *const SifiveClint) };

pub mod machine {
    use crate::sys;

    pub const NCORES: usize = 4;
    pub const MTIME_FREQ: u64 = 3_580_000 * 2;

    pub struct MemRange {
        pub start: usize,
        pub size: usize,
    }

    const MAIN_MEMORY: MemRange = MemRange {
        start: 0x8000_0000,
        size: sys::gb(2) as usize,
    };

    const MEM_RANGES: [MemRange; 2] = [
        MemRange {
            start: 0,
            size: sys::gb(2) as usize,
        },
        MAIN_MEMORY,
    ];
}
