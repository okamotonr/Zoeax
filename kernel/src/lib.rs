#![no_std]
#![feature(naked_functions)]
#![feature(ptr_as_uninit)]
#![allow(clippy::missing_safety_doc)]

mod address;
pub mod capability;
pub mod common;
pub mod handler;
pub mod init;
mod memlayout;
pub mod object;
pub mod riscv;
pub mod sbi;
pub mod scheduler;
pub mod syscall;
pub mod timer;
pub mod uart;
pub mod vm;
