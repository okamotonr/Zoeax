#![no_std]
#![feature(naked_functions)]
#![feature(ptr_as_uninit)]
#![feature(min_specialization)]
#![allow(clippy::missing_safety_doc)]

mod address;
mod capability;
pub mod common;
pub mod handler;
pub mod init;
mod ipc_args;
mod memlayout;
mod object;
mod riscv;
mod sbi;
mod scheduler;
mod syscall;
mod timer;
pub mod uart;
