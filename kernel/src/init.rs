use shared::elf::Elf64Hdr;

mod pm;
mod root_server;
mod vm;

use crate::handler::trap_entry;
use crate::println;
use crate::riscv::{r_sie, w_sie, w_sscratch, w_stvec, SIE_SEIE, SIE_SSIE, SIE_STIE};
use crate::scheduler::CPU_VAR;
use crate::timer::{set_timer, MTIME_PER_1MS};
use pm::BumpAllocator;
use root_server::init_root_server;
use vm::kernel_vm_init;

pub fn init_kernel(elf_header: *const Elf64Hdr, free_ram_phys: usize, free_ram_end_phys: usize) {
    println!("initialising kernel");
    w_stvec(trap_entry as usize);
    let bump_allocator = unsafe { BumpAllocator::new(free_ram_phys, free_ram_end_phys) };
    unsafe { kernel_vm_init(free_ram_end_phys) };
    w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);
    init_root_server(bump_allocator, elf_header);
    w_sscratch(&raw const CPU_VAR as usize);
    set_timer(MTIME_PER_1MS);
    println!("initialization finished");
}
