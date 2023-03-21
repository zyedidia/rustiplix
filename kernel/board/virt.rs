use crate::dev::uart::dwapb::DwApb;
use crate::dev::uart::Uart;

use crate::sync::spinlock::SpinLock;

pub static UART: SpinLock<Uart<DwApb>> = SpinLock::new(Uart::<DwApb>::new(0x10000000));
