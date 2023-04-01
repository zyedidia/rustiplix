#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

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
pub mod dev;
pub mod kalloc;
pub mod primary;
pub mod start;
pub mod sync;
pub mod sys;
pub mod timer;
pub mod vm;
