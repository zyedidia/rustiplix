use crate::dev::uart::virt::VirtUart;
use crate::dev::uart::Uart;

use crate::sync::spinlock::SpinLock;

pub static UART: SpinLock<Uart<VirtUart>> = SpinLock::new(Uart::<VirtUart>::new(0x10000000));
