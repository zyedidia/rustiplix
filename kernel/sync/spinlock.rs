use core::cell::UnsafeCell;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Release};

use crate::arch::trap::irq;

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        let en = irq::enabled();
        unsafe { irq::off() };
        while self.locked.swap(true, Acquire) {}
        Guard {
            irqen: en,
            lock: self,
        }
    }
}

use core::ops::{Deref, DerefMut};

pub struct Guard<'a, T> {
    irqen: bool,
    lock: &'a SpinLock<T>,
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
        // Safety: the existence of this Guard guarantees we own the lock.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
        if self.irqen {
            unsafe { irq::on() };
        }
    }
}
