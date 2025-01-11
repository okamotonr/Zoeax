#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{panic::PanicInfo, ptr};

use core::arch::global_asm;
use kernel::handler::return_to_user;
use kernel::init::init_kernel;
use kernel::println;

use common::elf::*;
extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __stack_top: u8;
}

global_asm!(include_str!("boot.S"));

#[repr(C)] // guarantee 'bytes' comes after '_align'
pub struct AlignedTo<Align, Bytes: ?Sized> {
    _align: [Align; 0],
    pub bytes: Bytes,
}

static ALIGNED: &AlignedTo<u8, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!("../shell"),
};

static ALIGNED_PONG: &AlignedTo<u8, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!("../pong"),
};

#[no_mangle]
static SHELL: &[u8] = &ALIGNED.bytes;
#[no_mangle]
static PONG: &[u8] = &ALIGNED_PONG.bytes;

#[export_name = "_kernel_main"]
extern "C" fn kernel_main(
    hartid: usize,
    _dtb_addr: usize,
    free_ram_phys: usize,
    free_ram_end_phys: usize,
) -> ! {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    };
    println!("cpu id is {}", hartid);
    let elf_header = (SHELL as *const [u8]).cast::<Elf64Hdr>();

    init_kernel(
        elf_header,
        free_ram_phys,
        free_ram_end_phys,
        &raw const __stack_top as usize,
    );
    println!("return to user");
    unsafe { return_to_user() }
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
