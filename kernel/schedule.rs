use crate::arch::regs::Context;
use crate::arch::trap::irq;
use crate::proc::{Proc, ProcState};
use crate::sync::spinlock::SpinLock;

use alloc::boxed::Box;
use core::ptr::null_mut;

pub static RUN_QUEUE: SpinLock<Queue> = SpinLock::new(Queue::new(QueueType::Run));
pub static EXIT_QUEUE: SpinLock<Queue> = SpinLock::new(Queue::new(QueueType::Exit));
pub static WAIT_QUEUE: SpinLock<Queue> = SpinLock::new(Queue::new(QueueType::Wait));
pub static mut TICKS_QUEUE: Queue = Queue::new(QueueType::Ticks);
pub static mut CONTEXT: Context = Context::zero();

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum QueueType {
    Run,
    Exit,
    Wait,
    Ticks,
}

pub struct QueueIter {
    cur: *mut Proc,
}

impl QueueIter {
    pub fn new(q: &Queue) -> QueueIter {
        Self { cur: q.front }
    }
}

impl Iterator for QueueIter {
    type Item = *mut Proc;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == null_mut() {
            return None;
        }
        let ret = self.cur;
        unsafe { self.cur = (*self.cur).data.next };
        Some(ret)
    }
}

pub struct Queue {
    front: *mut Proc,
    back: *mut Proc,
    size: usize,
    pub id: QueueType,
}

unsafe impl Send for Queue {}

impl Queue {
    pub const fn new(id: QueueType) -> Self {
        Self {
            front: null_mut(),
            back: null_mut(),
            size: 0,
            id,
        }
    }

    pub fn push_front(&mut self, n: Box<Proc>) {
        unsafe { self.push_front_raw(Box::<Proc>::into_raw(n)) };
    }

    pub unsafe fn push_front_raw(&mut self, n: *mut Proc) {
        (*n).data.next = self.front;
        (*n).data.prev = null_mut();
        if self.front != null_mut() {
            (*self.front).data.prev = n;
        } else {
            self.back = n;
        }
        self.front = n;
        self.size += 1;
    }

    pub unsafe fn remove(&mut self, n: *mut Proc) {
        assert!(self.size > 0);
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
        self.size -= 1;
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

    pub fn wake_all(&mut self) {
        while self.front != null_mut() {
            unsafe {
                // Removes front from the queue.
                self.wake(self.front);
            }
        }
    }

    pub unsafe fn wake(&mut self, p: *mut Proc) {
        assert!((*p).data.state == ProcState::Blocked);
        self.remove(p);
        (*p).data.state = ProcState::Runnable;
        (*p).data.wq = None;
        RUN_QUEUE.lock().push_front_raw(p);
    }
}

fn runnable_proc() -> Box<Proc> {
    unsafe { irq::on() };
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
        let mut p = Box::<Proc>::into_raw(runnable_proc());

        unsafe {
            irq::off();
            // TODO: should have one context per core
            p = kswitch_proc(p as *mut (), &mut CONTEXT, &mut (*p).data.context) as *mut Proc;

            if (*p).data.state == ProcState::Runnable {
                RUN_QUEUE.lock().push_front(Box::<Proc>::from_raw(p));
            } else if (*p).data.wq.is_none() {
                // Not runnable and not on any queue means we can free this process.
                core::ptr::drop_in_place(p);
            }
        }
    }
}
