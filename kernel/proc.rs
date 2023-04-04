use crate::arch::trap::Trapframe;
use crate::arch::vm::{kernel_procmap, Pagetable};
use crate::elf;
use crate::kalloc::{zalloc, zallocpage};
use crate::sys;
use crate::vm::{perm, PageMap};

use alloc::boxed::Box;
use core::ptr::addr_of_mut;

pub enum ProcState {
    Runnable,
    Blocked,
    Exited,
}

pub struct ProcData {
    pid: u32,
    pub pt: Box<Pagetable>,
    nchild: usize,

    state: ProcState,
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
        let mut pt = match zalloc::<Pagetable>() {
            Err(_) => {
                return None;
            }
            Ok(pt) => pt,
        };

        // Map code by loading the elf data.
        let (entry, _) = elf::load64(&mut pt, bin)?;

        // Map kernel.
        kernel_procmap(&mut pt);

        // Allocate/map stack.
        let ustack = match zallocpage() {
            Err(_) => {
                return None;
            }
            Ok(ustack) => ustack,
        };
        pt.mappg(Self::STACK_VA, ustack, perm::URW)?;
        let mut trapframe = Trapframe::default();
        trapframe.regs.sp = Self::STACK_VA + sys::PAGESIZE - 16;
        trapframe.epc = entry as usize;

        let proc = unsafe {
            let mut data = match Box::<Proc>::try_new_uninit() {
                Err(_) => {
                    return None;
                }
                Ok(data) => data,
            };
            let proc = data.as_mut_ptr();
            addr_of_mut!((*proc).trapframe).write(trapframe);
            addr_of_mut!((*proc).data).write(ProcData {
                pid: 0,
                pt,
                nchild: 0,
                state: ProcState::Runnable,
            });
            data.assume_init()
        };

        Some(proc)
    }

    pub fn kstackp(p: *mut Self) -> *const u8 {
        unsafe { (*p).kstack.0.as_ptr() }
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        // TODO: drop all pages "owned" by the pagetable.
    }
}
