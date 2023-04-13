use crate::arch::timer;
use crate::proc::Proc;
use crate::schedule::TICKS_QUEUE;

#[derive(PartialEq, Copy, Clone)]
pub enum Irq {
    Timer,
}

pub fn irq_handler_kern(irq: Irq) {
    match irq {
        Irq::Timer => {
            unsafe { TICKS_QUEUE.wake_all() };
            timer::intr(timer::TIME_SLICE_US);
        }
    }
}

pub fn irq_handler_user(p: &mut Proc, irq: Irq) {
    irq_handler_kern(irq);

    if irq == Irq::Timer {
        p.yield_();
    }
}
