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
/// Scheduler context (registers and pagetable).
pub static mut CONTEXT: Context = Context::zero();

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum QueueType {
    Run,   // runnable
    Exit,  // exited
    Wait,  // waiting for child
    Ticks, // waiting for next timer interrupt
}

/// Iterator for all processes in a queue.
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
        if self.cur.is_null() {
            return None;
        }
        let ret = self.cur;
        // Move to the next element in the queue.
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

    /// Pushes a process onto the queue.
    pub fn push_front(&mut self, n: Box<Proc>) {
        unsafe { self.push_front_raw(Box::<Proc>::into_raw(n)) };
    }

    /// Pushes a process onto the queue from a raw pointer.
    pub unsafe fn push_front_raw(&mut self, n: *mut Proc) {
        (*n).data.next = self.front;
        (*n).data.prev = null_mut();
        if !self.front.is_null() {
            (*self.front).data.prev = n;
        } else {
            self.back = n;
        }
        self.front = n;
        self.size += 1;
    }

    /// Removes a process from the queue.
    pub unsafe fn remove(&mut self, n: *mut Proc) {
        assert!(self.size > 0);
        if !(*n).data.next.is_null() {
            (*(*n).data.next).data.prev = (*n).data.prev;
        } else {
            self.back = (*n).data.prev;
        }
        if !(*n).data.prev.is_null() {
            (*(*n).data.prev).data.next = (*n).data.next;
        } else {
            self.front = (*n).data.next;
        }
        self.size -= 1;
    }

    /// Removes the last element from the queue (or None if the queue is empty). Ownership is moved
    /// from the queue to the caller.
    pub fn pop_back(&mut self) -> Option<Box<Proc>> {
        let b = self.back;
        if b.is_null() {
            return None;
        }
        unsafe {
            self.remove(b);
            Some(Box::<Proc>::from_raw(b))
        }
    }

    /// Wakes up all processes on the queue by removing them, marking them as runnable, and pushing
    /// them onto the RUN_QUEUE.
    pub fn wake_all(&mut self) {
        while !self.front.is_null() {
            unsafe {
                // Removes front from the queue.
                self.wake(self.front);
            }
        }
    }

    /// Wakes an individiaul process. The process must be blocked.
    pub unsafe fn wake(&mut self, p: *mut Proc) {
        assert!((*p).data.state == ProcState::Blocked);
        self.remove(p);
        (*p).data.state = ProcState::Runnable;
        (*p).data.wq = None;
        RUN_QUEUE.lock().push_front_raw(p);
    }
}

/// Returns the next process available to run, or blocks waiting for a process to become available.
fn runnable_proc() -> Box<Proc> {
    // Enable interrupts to avoid deadlock if there are no runnable processes.
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

/// The scheduler selects the next process to run and switches the execution context. When the
/// process is done executing, it switches back to the scheduler, which then chooses the next
/// process to run.
pub fn scheduler() -> ! {
    loop {
        // Get the next runnable process.
        let mut p = Box::<Proc>::into_raw(runnable_proc());

        unsafe {
            irq::off();
            // TODO: should have one context per core
            p = kswitch_proc(p as *mut (), &mut CONTEXT, &mut (*p).data.context) as *mut Proc;

            if (*p).data.state == ProcState::Runnable {
                // Put the process back on the run queue.
                RUN_QUEUE.lock().push_front(Box::<Proc>::from_raw(p));
            } else if (*p).data.wq.is_none() {
                // Not runnable and not on any queue means we can free this process.
                core::ptr::drop_in_place(p);
            }
        }
    }
}
