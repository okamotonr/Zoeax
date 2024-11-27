#![no_std]
#![feature(naked_functions)]

pub mod sbi;
pub mod uart;
pub mod common;
pub mod handler;
pub mod riscv;
pub mod memory;
pub mod process;
pub mod page;
pub mod virtio;

#[macro_export]
macro_rules! write_csr {
    ($csr:expr, $value:expr) => {
        unsafe {
            use core::arch::asm;
            asm!(concat!("csrw ", $csr, ", {r}"), r = in(reg) $value);
        }
    };
}

