#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(new_uninit)]
#![feature(raw_ref_op)]

#[cfg(all(feature = "kernel", feature = "monitor"))]
compile_error!("feature \"kernel\" and feature \"monitor\" cannot be enabled at the same time");

#[cfg(not(any(feature = "kernel", feature = "monitor")))]
compile_error!("one of feature \"kernel\" or feature \"monitor\" must be enabled");

pub extern crate alloc;

#[macro_use]
pub mod fmt;
#[macro_use]
pub mod bitfield;

pub mod arch;
pub mod bit;
pub mod board;
pub mod builtin;
pub mod cpu;
pub mod crc;
pub mod dev;
pub mod elf;
pub mod kalloc;
pub mod primary;
pub mod proc;
pub mod schedule;
pub mod start;
pub mod sync;
pub mod sys;
pub mod timer;
pub mod vm;
