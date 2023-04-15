use crate::proc::{Proc, ProcState};
use crate::schedule::{QueueIter, QueueType, EXIT_QUEUE, RUN_QUEUE, TICKS_QUEUE, WAIT_QUEUE};
use core::slice;

mod num {
    pub const SYS_WRITE: usize = 0;
    pub const SYS_GETPID: usize = 1;
    pub const SYS_EXIT: usize = 2;
    pub const SYS_FORK: usize = 3;
    pub const SYS_WAIT: usize = 4;
    pub const SYS_SBRK: usize = 5;
    pub const SYS_USLEEP: usize = 6;
}

mod err {
    pub const BADF: isize = -9;
    pub const NOSYS: isize = -38;
    pub const FAULT: isize = -14;
    pub const NOMEM: isize = -12;
    pub const CHILD: isize = -10;
}

/// System call handler.
pub fn syscall(p: &mut Proc, sysno: usize) -> isize {
    match sysno {
        num::SYS_GETPID => sys_getpid(p) as isize,
        num::SYS_WRITE => sys_write(
            p,
            p.trapframe.regs.arg0() as i32,
            p.trapframe.regs.arg1(),
            p.trapframe.regs.arg2(),
        ),
        num::SYS_SBRK => sys_sbrk(p),
        num::SYS_EXIT => sys_exit(p),
        num::SYS_FORK => sys_fork(p),
        num::SYS_USLEEP => {
            sys_usleep(p, p.trapframe.regs.arg0() as u64);
            0
        }
        num::SYS_WAIT => sys_wait(p),
        _ => {
            println!("unknown syscall {}", sysno);
            err::NOSYS
        }
    }
}

/// Returns the process's PID.
fn sys_getpid(p: &mut Proc) -> u32 {
    p.data.pid
}

/// Write to 'sz' bytes at 'addr' into 'fd' on behalf of the process. Returns the number of bytes
/// written, or an error.
fn sys_write(_p: &mut Proc, fd: i32, addr: usize, sz: usize) -> isize {
    if sz == 0 {
        return 0;
    }

    // Validate buffer.
    let overflow = addr.wrapping_add(sz);
    if overflow < addr || addr >= Proc::MAX_VA {
        return err::FAULT;
    }

    // TODO: make sure all pages are user accessible

    // TODO: we only support stdout for now.
    if fd != 1 {
        return err::BADF;
    }

    let buf = unsafe { slice::from_raw_parts(addr as *const u8, sz) };
    {
        let mut uart = crate::board::UART.lock();
        uart.write_bytes(buf);
    }

    sz as isize
}

/// Create a new child process that is a clone of the current process. Returns:
/// * 0 to the child.
/// * Child's PID to the parent, or an error if the child could not be created.
fn sys_fork(p: &mut Proc) -> isize {
    let mut child = match Proc::new_from_parent(p) {
        None => {
            return err::NOMEM;
        }
        Some(p) => p,
    };
    child.trapframe.regs.set_ret(0);
    p.data.nchild += 1;

    let pid = child.data.pid;
    RUN_QUEUE.lock().push_front(child);
    pid as isize
}

fn sys_sbrk(_p: &mut Proc) -> isize {
    -1
}

/// Exit the current process and switches back to the scheduler.
fn sys_exit(p: &mut Proc) -> ! {
    println!("{}: exited", p.data.pid);
    p.data.state = ProcState::Exited;

    // TODO: reparent all children

    if !p.data.parent.is_null() {
        unsafe {
            // Wake up the parent if it is waiting for the child to exit.
            if (*p.data.parent).data.state == ProcState::Blocked
                && (*p.data.parent).data.wq == Some(QueueType::Wait)
            {
                WAIT_QUEUE.lock().wake(p.data.parent);
            }
        }
    }

    p.exit(&mut EXIT_QUEUE.lock());
    p.yield_();
    panic!("exited process resumed");
}

/// Wait for 'us' microseconds.
fn sys_usleep(p: &mut Proc, us: u64) {
    use crate::timer;
    let start = timer::time();

    loop {
        if timer::us_since(start) >= us {
            break;
        }
        // Enter the ticks wait queue that will be woken up every timer interrupt.
        unsafe { p.block(&mut TICKS_QUEUE) };
        p.yield_();
        // A timer interrupt has occurred and we are now runnable. Recheck the condition, and jump
        // back on the wait queue if there is still more time to wait.
    }
}

/// Wait for a child to exit. Returns the PID of the exited child.
fn sys_wait(p: &mut Proc) -> isize {
    if p.data.nchild == 0 {
        // No children.
        return err::CHILD;
    }

    loop {
        {
            // Look through all processes that have exited and are waiting for a parent to wait for
            // them (zombies).
            let mut exited = EXIT_QUEUE.lock();
            for zombie in QueueIter::new(&exited) {
                unsafe {
                    // This zombie has 'p' as a parent.
                    if (*zombie).data.parent == p as *mut Proc {
                        // Read the child's PID, and then remove it from the exit queue and free
                        // it.
                        let pid = (*zombie).data.pid;
                        exited.remove(zombie);
                        core::ptr::drop_in_place(zombie);
                        p.data.nchild -= 1;
                        return pid as isize;
                    }
                }
            }
        }
        // Push onto the wait queue and yield. We will be woken up when one of our children exits.
        p.block(&mut WAIT_QUEUE.lock());
        p.yield_();
    }
}
