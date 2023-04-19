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
    Blocked,
    Exited,
}

/// Process metadata.
pub struct ProcData {
    pub pid: u32,
    pub pt: Box<Pagetable>,
    pub nchild: usize,
    pub parent: *mut Proc,
    pub wq: Option<QueueType>,

    // Context for kernel context switches.
    pub context: Context,

    // Wait/run queue links. These are modified by scheduler::Queue.
    pub next: *mut Proc,
    pub prev: *mut Proc,

    pub state: ProcState,
}

/// Process struct, which contains the process metadata, trapframe information, and the kernel
/// stack.
#[repr(C)]
pub struct Proc {
    /// The trapframe stores the registers and context of the process when a trap occurs, and is
    /// used to restore this information when the process resumes after the trap.
    pub trapframe: Trapframe,
    pub data: ProcData,
    canary: u64,
    kstack: KStack,
}

#[repr(align(16))]
struct KStack([u8; 3008]);

impl Proc {
    /// Virtual address of the user stack.
    pub const STACK_VA: usize = 0x7fff0000;
    /// Size of a user stack.
    pub const STACK_SIZE: usize = sys::PAGESIZE;
    /// Maximum virtual address that a user process can access.
    pub const MAX_VA: usize = Self::STACK_VA + Self::STACK_SIZE;
    /// Stack canary.
    pub const CANARY: u64 = 0xfeedface_deadbeef;

    /// Constructs a new empty process. This process is given an empty pagetable with only the
    /// kernel mappings, and a valid kernel context that initializes the process to return into
    /// Proc::forkret.
    fn new_empty() -> Option<Box<Proc>> {
        let mut pt = match zalloc::<Pagetable>() {
            Err(_) => {
                return None;
            }
            Ok(pt) => pt,
        };
        kernel_procmap(&mut pt);

        // We have to use try_new_uninit to make sure this process is allocated directly into the
        // heap (otherwise might cause a stack overflow).
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

    /// Allocates a new process given a parent. The new process copies all user mappings from the
    /// parent into its pagetable, and copies over the parent's trapframe (registers and epc).
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

    /// Allocates a new process from an ELF binary. The bytes must be 64-bit aligned. The process
    /// pagetable is initialized from the ELF segments and is given a valid user stack and
    /// trapframe that returns into the ELF entrypoint.
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

    /// Returns a pointer to this process's kernel stack.
    pub unsafe fn kstackp(p: *mut Self) -> *const u8 {
        let len = (*p).kstack.0.len();
        (*p).kstack.0.as_ptr().add(len - 16)
    }

    /// Verifies that the kernel stack canary is valid.
    pub fn check_stack(&mut self) {
        assert!(self.canary == Self::CANARY);
    }

    pub fn watch_canary(&mut self) {
        use crate::arch::fwi;
        fwi::set_watchpoint(&self.canary as *const u64 as usize);
    }

    /// Yields this process and switches back to the current core's scheduler. Interrupts must be
    /// disabled to call this function.
    pub fn yield_(&mut self) {
        use crate::arch::trap::irq;
        use crate::schedule::{kswitch, CONTEXT};
        assert!(!irq::enabled());

        unsafe { kswitch(&mut self.data.context, &mut CONTEXT) }
        self.watch_canary();
    }

    /// Puts this process on the given wait queue.
    pub fn block(&mut self, queue: &mut Queue) {
        self.wait(queue, ProcState::Blocked);
    }

    /// Puts this process on the given exit queue.
    pub fn exit(&mut self, queue: &mut Queue) {
        self.wait(queue, ProcState::Exited);
    }

    // Assigns this process state and registers it on the given queue. It is up to the caller to
    // then call yield_ to actually stop the current process.
    fn wait(&mut self, queue: &mut Queue, state: ProcState) {
        self.data.state = state;
        self.data.wq = Some(queue.id);
        unsafe { queue.push_front_raw(self as *mut Proc) };
    }

    /// Marks this process as not on any wait queue.
    pub fn unblock(&mut self) {
        self.data.wq = None;
    }

    /// This is the entrypoint for newly created processes.
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
