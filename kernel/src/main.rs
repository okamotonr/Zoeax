#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{
    arch::naked_asm,
    panic::PanicInfo,
    ptr,
};

use mios::{handler::trap_entry, memory::PhysAddr};
use mios::memory;
use mios::process::{yield_proc, Process, init_proc};
use mios::riscv::{w_stvec, wfi};
use mios::println;

extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __stack_top: u8;
    pub static __kernel_base: u8;
}

#[no_mangle]
static SHELL: &'static [u8] = include_bytes!("shell");

#[no_mangle]
fn kernel_main() {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    };

    w_stvec(trap_entry as usize);

    memory::init_memory();
    println!("{:?}, {:?}, {:?}, {:?}", SHELL[0], SHELL[1], SHELL[2], SHELL[3]);
    println!("{:p}", &SHELL);
    println!("{:p}", SHELL);
    let start = ptr::addr_of!(SHELL);
    unsafe {
        println!("{:?}", (*start)[0]);
        println!("{:?}", *(start as *const u8));
    }

    let paddr0 = memory::alloc_pages(2);
    let paddr1 = memory::alloc_pages(1);
    println!("alloc_pages test: paddr0={:?}", paddr0);
    println!("alloc_pages test: paddr1={:?}", paddr1);

    init_proc();
    println!("Here");
    Process::create(SHELL);
    println!("Come");
    unsafe {
        yield_proc();
    }
    loop {
        wfi();
    }
}

#[link_section = ".text.boot"]
#[naked]
#[no_mangle]
extern "C" fn boot() {
    unsafe {
        naked_asm!(
            "la sp, {stack_top}",
            "j kernel_main",
            stack_top = sym  __stack_top,
            // options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

