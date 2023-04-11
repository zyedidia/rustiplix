use crate::arch::regs::Context;
use crate::arch::trap::{usertrapret, Trapframe};
use crate::arch::vm::{kernel_procmap, Pagetable};
use crate::elf;
use crate::kalloc::{zalloc, zallocpage};
use crate::sys;
use crate::vm::{perm, PageMap};

use alloc::boxed::Box;
use core::ptr::{addr_of_mut, null_mut};

#[derive(PartialEq)]
pub enum ProcState {
    Runnable,
    Running,
    Blocked,
    Exited,
}

pub struct ProcData {
    pid: u32,
    pub pt: Box<Pagetable>,
    nchild: usize,
    parent: u32,

    pub context: Context,

    pub next: *mut Proc,
    pub prev: *mut Proc,

    pub state: ProcState,
}

#[repr(C)]
pub struct Proc {
    pub trapframe: Trapframe,
    pub data: ProcData,
    canary: u64,
    kstack: KStack,
}

#[repr(align(16))]
struct KStack([u8; 3008]);

impl Proc {
    pub const STACK_VA: usize = 0x7fff0000;
    pub const STACK_SIZE: usize = sys::PAGESIZE;
    pub const MAX_VA: usize = Self::STACK_VA + Self::STACK_SIZE;
    pub const CANARY: u64 = 0xfeedface_deadbeef;

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

        let mut proc = unsafe {
            let mut data = match Box::<Proc>::try_new_uninit() {
                Err(_) => {
                    return None;
                }
                Ok(data) => data,
            };
            let proc = data.as_mut_ptr();
            addr_of_mut!((*proc).trapframe).write(trapframe);
            addr_of_mut!((*proc).data).write(ProcData {
                pid: 1,
                pt,
                nchild: 0,
                state: ProcState::Runnable,
                parent: 0,
                next: null_mut(),
                prev: null_mut(),
                context: Context::new(
                    Self::kstackp(proc) as usize,
                    Self::forkret as *const () as usize,
                ),
            });
            addr_of_mut!((*proc).canary).write(Self::CANARY);
            data.assume_init()
        };

        proc.data.context.set_pt(&proc.data.pt);

        Some(proc)
    }

    pub fn kstackp(p: *mut Self) -> *const u8 {
        unsafe {
            let len = (*p).kstack.0.len();
            (*p).kstack.0.as_ptr().add(len - 16)
        }
    }

    pub fn check_stack(&mut self) {
        assert!(self.canary == Self::CANARY);
    }

    pub unsafe extern "C" fn forkret(proc: *mut Proc) {
        usertrapret(Box::<Proc>::from_raw(proc));
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        // TODO: drop all pages "owned" by the pagetable.
    }
}
