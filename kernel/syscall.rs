use crate::proc::{Proc, ProcState};
use crate::schedule::{RUN_QUEUE, TICKS_QUEUE};
use core::slice;

mod num {
    pub const SYS_WRITE: usize = 0;
    pub const SYS_GETPID: usize = 1;
    pub const SYS_EXIT: usize = 2;
    pub const SYS_FORK: usize = 3;
    pub const SYS_SBRK: usize = 5;
    pub const SYS_USLEEP: usize = 6;
}

mod err {
    pub const BADF: isize = -9;
    pub const NOSYS: isize = -38;
    pub const FAULT: isize = -14;
    pub const NOMEM: isize = -12;
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
    }
    p.unblock();
}
