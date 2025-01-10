use common::elf::Elf64Hdr;

mod root_server;
mod pm;
mod vm;

use pm::BumpAllocator;
use vm::kernel_vm_init;
use root_server::init_root_server;
use crate::scheduler::CPU_VAR;
use crate::riscv::{w_sscratch, w_stvec, w_sie, r_sie, SIE_SEIE, SIE_STIE, SIE_SSIE};
use crate::handler::trap_entry;
use crate::println;
use crate::timer::set_timer;

pub fn init_kernel(elf_header: *const Elf64Hdr, free_ram_phys: usize, free_ram_end_phys: usize, stack_top: usize) {
    println!("initialising kernel");
    w_stvec(trap_entry as usize);
    let bump_allocator = unsafe {
        BumpAllocator::new(free_ram_phys, free_ram_end_phys)
    };
    unsafe { kernel_vm_init(free_ram_end_phys) };
    w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);
    init_root_server(bump_allocator, elf_header);
    unsafe {
        CPU_VAR.sptop = stack_top;
    }
    w_sscratch(&raw const CPU_VAR as usize);
    set_timer(100000);
    println!("initialization finished");
}
