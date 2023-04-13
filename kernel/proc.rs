use crate::arch::regs::Context;
use crate::arch::trap::{usertrapret, Trapframe};
use crate::arch::vm::{kernel_procmap, Pagetable};
use crate::elf;
use crate::kalloc::{kallocpage, kfree, zalloc, zallocpage};
use crate::schedule::{Queue, QueueType};
use crate::sys;
use crate::vm::{perm, PageMap, PtIter};

use alloc::boxed::Box;
use core::ptr::{addr_of_mut, null_mut};

use core::sync::atomic::{AtomicU32, Ordering};

static NEXTPID: AtomicU32 = AtomicU32::new(1);

#[derive(PartialEq)]
pub enum ProcState {
    Runnable,
    Running,
    Blocked,
    Exited,
}

pub struct ProcData {
    pub pid: u32,
    pub pt: Box<Pagetable>,
    pub nchild: usize,
    pub parent: *mut Proc,
    pub wq: Option<QueueType>,

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

    fn new_empty() -> Option<Box<Proc>> {
        let mut pt = match zalloc::<Pagetable>() {
            Err(_) => {
                return None;
            }
            Ok(pt) => pt,
        };
        kernel_procmap(&mut pt);

        let mut proc = unsafe {
            let mut data = match Box::<Proc>::try_new_uninit() {
                Err(_) => {
                    return None;
                }
                Ok(data) => data,
            };
            let proc = data.as_mut_ptr();
            addr_of_mut!((*proc).data).write(ProcData {
                pid: NEXTPID.fetch_add(1, Ordering::Relaxed),
                pt,
                nchild: 0,
                state: ProcState::Runnable,
                parent: null_mut(),
                next: null_mut(),
                prev: null_mut(),
                wq: None,
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

    pub fn new_from_parent(parent: &mut Proc) -> Option<Box<Proc>> {
        let mut p = Self::new_empty()?;

        for map in PtIter::new(&mut parent.data.pt) {
            let mut pg = match kallocpage() {
                Err(_) => {
                    return None;
                }
                Ok(pg) => pg,
            };
            pg.copy_from_slice(map.pg());
            p.data.pt.mappg(map.va(), pg, map.perm())?;
        }

        p.data.parent = parent as *mut Proc;
        p.trapframe = parent.trapframe;

        Some(p)
    }

    pub fn new_from_elf(bin: &[u8]) -> Option<Box<Proc>> {
        let mut p = Self::new_empty()?;

        // Map code by loading the elf data.
        let (entry, _) = elf::load64(&mut p.data.pt, bin)?;

        // Allocate/map stack.
        let ustack = match zallocpage() {
            Err(_) => {
                return None;
            }
            Ok(ustack) => ustack,
        };
        p.data.pt.mappg(Self::STACK_VA, ustack, perm::URW)?;
        p.trapframe = Trapframe::default();
        p.trapframe.regs.sp = Self::STACK_VA + sys::PAGESIZE - 16;
        p.trapframe.epc = entry as usize;

        Some(p)
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

    pub fn yield_(&mut self) {
        use crate::arch::trap::irq;
        use crate::schedule::{kswitch, CONTEXT};
        assert!(!irq::enabled());

        unsafe { kswitch(&mut self.data.context, &mut CONTEXT) }
    }

    pub fn block(&mut self, queue: &mut Queue) {
        self.wait(queue, ProcState::Blocked);
    }

    pub fn exit(&mut self, queue: &mut Queue) {
        self.wait(queue, ProcState::Exited);
    }

    fn wait(&mut self, queue: &mut Queue, state: ProcState) {
        self.data.state = state;
        self.data.wq = Some(queue.id);
        unsafe { queue.push_front_raw(self as *mut Proc) };
        self.yield_();
    }

    pub fn unblock(&mut self) {
        self.data.wq = None;
    }

    pub unsafe extern "C" fn forkret(proc: *mut Proc) {
        usertrapret(proc);
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        println!("{}: dropped", self.data.pid);
        for mut map in PtIter::new(&mut self.data.pt) {
            unsafe { kfree(map.pg_raw()) };
        }
    }
}
