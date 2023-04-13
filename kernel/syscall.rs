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

pub fn syscall(p: &mut Proc, sysno: usize) -> isize {
    let ret;
    match sysno {
        num::SYS_GETPID => {
            ret = sys_getpid(p) as isize;
        }
        num::SYS_WRITE => {
            ret = sys_write(
                p,
                p.trapframe.regs.arg0() as i32,
                p.trapframe.regs.arg1(),
                p.trapframe.regs.arg2(),
            );
        }
        num::SYS_SBRK => {
            ret = sys_sbrk(p);
        }
        num::SYS_EXIT => {
            sys_exit(p);
        }
        num::SYS_FORK => {
            ret = sys_fork(p);
        }
        num::SYS_USLEEP => {
            sys_usleep(p, p.trapframe.regs.arg0() as u64);
            ret = 0;
        }
        num::SYS_WAIT => {
            ret = sys_wait(p);
        }
        _ => {
            println!("unknown syscall {}", sysno);
            return err::NOSYS;
        }
    }
    ret
}

fn sys_getpid(p: &mut Proc) -> u32 {
    p.data.pid
}

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
    return pid as isize;
}

fn sys_sbrk(_p: &mut Proc) -> isize {
    -1
}

fn sys_exit(p: &mut Proc) -> ! {
    println!("{}: exited", p.data.pid);
    p.data.state = ProcState::Exited;

    // TODO: reparent all children

    if !p.data.parent.is_null() {
        unsafe {
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

fn sys_usleep(p: &mut Proc, us: u64) {
    use crate::timer;
    let start = timer::time();

    loop {
        if timer::us_since(start) >= us {
            break;
        }
        unsafe { p.block(&mut TICKS_QUEUE) };
        p.yield_();
    }
    p.unblock();
}

fn sys_wait(p: &mut Proc) -> isize {
    if p.data.nchild == 0 {
        return err::CHILD;
    }

    loop {
        {
            let mut exited = EXIT_QUEUE.lock();
            for zombie in QueueIter::new(&mut exited) {
                unsafe {
                    if (*zombie).data.parent == p as *mut Proc {
                        let pid = (*zombie).data.pid;
                        exited.remove(zombie);
                        core::ptr::drop_in_place(zombie);
                        p.data.nchild -= 1;
                        return pid as isize;
                    }
                }
            }
        }
        p.block(&mut WAIT_QUEUE.lock());
        p.yield_();
    }
}
