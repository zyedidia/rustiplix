use crate::arch::trap::Trapframe;
use crate::arch::vm::Pagetable;
use crate::sys;

use alloc::boxed::Box;

pub enum ProcState {
    Runnable,
    Blocked,
    Exited,
}

pub struct ProcData {
    pid: u32,
    pub pt: Box<Pagetable>,
    ustack: *mut [u8; Proc::STACK_SIZE],
    parent: *mut Proc,
    nchild: usize,

    state: ProcState,
    wq: *const (),
}

#[repr(C)]
pub struct Proc {
    pub trapframe: Trapframe,
    pub data: ProcData,
    kstack: KStack,
}

#[repr(align(16))]
struct KStack([u8; 3008]);

impl Proc {
    pub const STACK_VA: usize = 0x7fff0000;
    pub const STACK_SIZE: usize = sys::PAGESIZE;
    pub const MAX_VA: usize = Self::STACK_VA + Self::STACK_SIZE;

    pub fn new_boxed(bin: &[u8]) -> Option<Box<Proc>> {
        None
    }

    pub fn kstackp(p: *mut Self) -> *mut u8 {
        unsafe { &raw mut (*p).kstack as *mut u8 }
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        // TODO: drop all pages "owned" by the pagetable.
    }
}
