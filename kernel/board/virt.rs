use crate::dev::irq::sfclint::SifiveClint;
use crate::dev::uart::virt::VirtUart;
use crate::dev::uart::Uart;

use crate::sync::spinlock::SpinLock;

pub static UART: SpinLock<Uart<VirtUart>> = SpinLock::new(Uart::<VirtUart>::new(0x10000000));
pub static CLINT: &SifiveClint = unsafe { &*(0x200_0000 as *const SifiveClint) };
