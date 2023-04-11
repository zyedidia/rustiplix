use crate::proc::Proc;
use core::slice;

mod num {
    pub const SYS_WRITE: usize = 0;
    pub const SYS_GETPID: usize = 1;
    pub const SYS_SBRK: usize = 5;
}

mod err {
    pub const BADF: isize = -9;
    pub const NO_SYS: isize = -38;
    pub const FAULT: isize = -14;
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
        _ => {
            println!("unknown syscall {}", sysno);
            return err::NO_SYS;
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

fn sys_sbrk(_p: &mut Proc) -> isize {
    -1
}
