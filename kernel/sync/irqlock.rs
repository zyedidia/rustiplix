use crate::cpu::Cpu;

// An IrqLock disables interrupts during a critical region allowing access to a value of type T in
// the region.
pub struct IrqLock<T> {
    was_en: bool
}

impl<T> IrqLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            was_en: false,
        }
    }

    pub fn lock(&self) -> Guard<&Cpu> {
        // TODO: disable irqs
        Guard {
            was_en: false,
            value: cpu(),
        }
    }
}

use core::ops::{Deref, DerefMut};

pub struct Guard<'a, T> {
    lock: &'a IrqLock<T>,
    value: &'a Cpu,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // Safety: the existence of this Guard guarantees we own the lock.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: the existence of this Guard guaratees we own the lock.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        // TODO: enable irqs if they were enabled
    }
}
