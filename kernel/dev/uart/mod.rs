pub mod dwapb;
pub mod virt;

use core::fmt::{Error, Write};

pub trait Uart {
    fn init(&mut self, baud: u32);
    fn tx(&mut self, b: u8);
    fn tx_flush(&mut self);
    fn rx(&mut self) -> u8;
    fn rx_empty(&mut self) -> bool;
}

pub struct UartWrapper<T: Uart> {
    pub base: *mut T,
}

unsafe impl<T: Uart> Send for UartWrapper<T> {}

impl<T: Uart> UartWrapper<T> {
    pub const fn new(base: *mut T) -> Self {
        Self { base }
    }

    pub fn write_bytes(&mut self, b: &[u8]) {
        for c in b {
            self.tx(*c);
        }
    }
}

use core::ops::{Deref, DerefMut};

impl<T: Uart> Deref for UartWrapper<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.base }
    }
}

impl<T: Uart> DerefMut for UartWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.base }
    }
}

impl<T: Uart> Write for UartWrapper<T> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.tx(c);
        }
        Ok(())
    }
}
