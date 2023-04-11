use crate::arch::regs::Context;
use crate::arch::trap::irq;
use crate::proc::{Proc, ProcState};
use crate::sync::spinlock::SpinLock;

use alloc::boxed::Box;
use core::ptr::null_mut;

pub static RUN_QUEUE: SpinLock<Queue> = SpinLock::new(Queue::new());
pub static mut CONTEXT: Context = Context::zero();

pub struct Queue {
    front: *mut Proc,
    back: *mut Proc,
}

unsafe impl Send for Queue {}

impl Queue {
    pub const fn new() -> Self {
        Self {
            front: null_mut(),
            back: null_mut(),
        }
    }

    pub fn push_front(&mut self, n: Box<Proc>) {
        unsafe {
            let n = Box::<Proc>::into_raw(n);
            (*n).data.next = self.front;
            (*n).data.prev = null_mut();
            if self.front != null_mut() {
                (*self.front).data.prev = n;
            } else {
                self.back = n;
            }
            self.front = n;
        }
    }
    pub unsafe fn remove(&mut self, n: *mut Proc) {
        if (*n).data.next != null_mut() {
            (*(*n).data.next).data.prev = (*n).data.prev;
        } else {
            self.back = (*n).data.prev;
        }
        if (*n).data.prev != null_mut() {
            (*(*n).data.prev).data.next = (*n).data.next;
        } else {
            self.front = (*n).data.next;
        }
    }
    pub fn pop_back(&mut self) -> Option<Box<Proc>> {
        let b = self.back;
        if b == null_mut() {
            return None;
        }
        unsafe {
            self.remove(b);
            Some(Box::<Proc>::from_raw(b))
        }
    }
}

fn runnable_proc() -> Box<Proc> {
    loop {
        match RUN_QUEUE.lock().pop_back() {
            None => {
                // no runnable procs -- wait until something happens
                crate::arch::cpu::wfi();
            }
            Some(proc) => {
                return proc;
            }
        }
    }
}

extern "C" {
    fn kswitch_proc(proc: *mut (), oldp: &mut Context, newp: &mut Context) -> *mut ();
    pub fn kswitch(oldp: &mut Context, newp: &mut Context);
}

pub fn scheduler() -> ! {
    loop {
        let mut p = runnable_proc();

        unsafe {
            irq::off();
            // TODO: should have one context per core
            let rp = Box::<Proc>::into_raw(p);
            p = Box::<Proc>::from_raw(kswitch_proc(
                rp as *mut (),
                &mut CONTEXT,
                &mut (*rp).data.context,
            ) as *mut Proc);
        }

        if p.data.state == ProcState::Runnable {
            RUN_QUEUE.lock().push_front(p);
        }
    }
}
