#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{panic::PanicInfo, ptr};

use core::arch::global_asm;
use kernel::elf::*;
use kernel::init_kernel;
use kernel::println;
use kernel::return_to_user;

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
    bytes: *include_bytes!("../rootserver"),
};

#[no_mangle]
static ROOTSERVER: &[u8] = &ALIGNED.bytes;

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
    let elf_header = (ROOTSERVER as *const [u8]).cast::<Elf64Hdr>();

    init_kernel(elf_header, free_ram_phys, free_ram_end_phys);
    println!("return to user");
    unsafe { return_to_user() }
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
