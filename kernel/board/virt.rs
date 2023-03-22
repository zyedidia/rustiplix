use crate::dev::irq::sfclint::SifiveClint;
use crate::dev::uart::virt::VirtUart;
use crate::dev::uart::Uart;

use crate::sync::spinlock::SpinLock;

// TODO: in the future we should use pa2ka to construct addresses so that these devices are usable
// from a high kernel mapping.
pub static UART: SpinLock<Uart<VirtUart>> = SpinLock::new(Uart::<VirtUart>::new(0x10000000));
pub static CLINT: &SifiveClint = unsafe { &*(0x200_0000 as *const SifiveClint) };
