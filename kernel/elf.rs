use crate::arch::vm::Pagetable;
use crate::kalloc::kallocpage;
use crate::sys;
use crate::vm::{perm, PageMap};

use core::intrinsics::write_bytes;

const MAGIC: u32 = 0x464C457F; // "\x7ELF" in little endian
const PROG_LOAD: u32 = 1;

#[repr(C)]
struct FileHeader64 {
    magic: u32,
    width: u8,
    _elf: [u8; 11],
    type_: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
struct ProgHeader64 {
    type_: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

// Returns the entrypoint and the breakpoint.
pub fn load64(pt: &mut Pagetable, elfdat: &[u8]) -> Option<(u64, u64)> {
    // The elf data must be properly aligned.
    assert!(&elfdat[0] as *const _ as usize % 8 == 0);
    let elf: &FileHeader64 = unsafe { &*(elfdat.as_ptr() as *const FileHeader64) };

    if elf.magic != MAGIC {
        return None;
    }

    let mut brk = 0;

    for i in 0..elf.phnum {
        let off = elf.phoff as usize + i as usize * core::mem::size_of::<ProgHeader64>();
        let ph: &ProgHeader64 = unsafe { &*(elfdat.as_ptr().add(off) as *const ProgHeader64) };
        if ph.type_ != PROG_LOAD || ph.memsz == 0 {
            continue;
        }

        if ph.memsz < ph.filesz {
            return None;
        }
        if ph.vaddr + ph.memsz < ph.vaddr {
            return None;
        }

        let mut pad = ph.vaddr % sys::PAGESIZE as u64;
        let va_start = ph.vaddr - pad;
        let sz = max(ph.memsz + pad, sys::PAGESIZE as u64);
        for va in (va_start..(va_start + sz)).step_by(sys::PAGESIZE) {
            let mut mem = match kallocpage() {
                Err(_) => {
                    return None;
                }
                Ok(mem) => mem,
            };
            unsafe { write_bytes(mem.as_mut_ptr(), 0, pad as usize) };
            // TODO: mem += pad?
            let mut written = pad as usize;
            pad = 0;
            let soff = va - ph.vaddr;
            if ph.filesz > soff {
                // Haven't yet reached ph.filesz, so there is more data from the ELF file to copy
                // in.
                let n = min(sys::PAGESIZE - written, ph.filesz as usize - off);
                let data_start = ph.offset as usize + soff as usize;
                let data_end = data_start + n;
                mem[written..written + n].copy_from_slice(&elfdat[data_start..data_end]);
                written += n;
            }

            // Set the rest of the data to 0.
            unsafe { write_bytes(mem.as_mut_ptr().add(written), 0, sys::PAGESIZE - written) };
            // Map the page with full permissions.
            pt.mappg(va as usize, mem, perm::URWX)?;

            // TODO: sync_idmem(mem, written)
        }

        brk = max(ph.vaddr + ph.memsz, brk);
    }

    Some((elf.entry, brk))
}

fn min(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

fn max(a: u64, b: u64) -> u64 {
    if a > b {
        a
    } else {
        b
    }
}
