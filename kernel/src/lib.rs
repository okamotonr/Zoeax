#![no_std]
#![feature(naked_functions)]
#![feature(ptr_as_uninit)]
#![feature(min_specialization)]
#![allow(clippy::missing_safety_doc)]

mod address;
mod capability;
pub mod common;
pub mod elf;
mod handler;
mod init;
mod ipc_args;
pub mod list;
mod memlayout;
mod object;
mod riscv;
mod sbi;
mod scheduler;
mod syscall;
mod timer;
pub mod uart;

pub use capability::CapabilityType;
pub use common::{ErrKind, KernelError, KernelResult};
pub use handler::return_to_user;
pub use init::init_kernel;
pub use syscall::InvLabel;
pub use syscall::SysCallNo;
pub use object::Registers;
