use core::cell::UnsafeCell;

pub struct PrimaryCell<T> {
    value: UnsafeCell<T>,
}

impl<T> PrimaryCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    // The mutable reference that is returned must be gone by the time the secondary cores are
    // booted.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut(&self) -> &mut T {
        // Writing to a PrimaryCell can only be done if the secondary cores haven't booted yet.
        use crate::cpu;
        assert!(!cpu::booted_all());
        &mut *self.value.get()
    }
}

unsafe impl<T> Sync for PrimaryCell<T> where T: Send {}

use core::ops::Deref;

impl<T> Deref for PrimaryCell<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // It is always safe to read from a PrimaryCell.
        unsafe { &*self.value.get() }
    }
}
