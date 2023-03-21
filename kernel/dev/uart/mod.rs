pub mod dwapb;

use core::fmt::{Error, Write};
use core::marker::PhantomData;

pub trait Putc {
    fn putc(&mut self, c: u8);
}

pub struct Uart<T: Putc> {
    pub base: usize,

    pub x: PhantomData<T>,
}

impl<T: Putc> Uart<T> {
    pub const fn new(base: usize) -> Self {
        Uart {
            base,
            x: PhantomData{},
        }
    }

    fn device(&mut self) -> &mut T {
        unsafe { &mut *(self.base as *mut T) }
    }
}

impl <T: Putc> Putc for Uart<T> {
    fn putc(&mut self, c: u8) {
        let uart = self.device();
        uart.putc(c);
    }
}

impl<T: Putc> Write for Uart<T> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        let uart = self.device();
        for c in s.bytes() {
            uart.putc(c);
        }
        Ok(())
    }
}
