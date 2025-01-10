#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{arch::naked_asm, cmp::min, panic::PanicInfo, ptr};
use core::arch::asm;

use kernel::common::align_up;
use kernel::handler::{return_to_user, trap_entry};
use kernel::memory::{BumpAllocator, PhysAddr};
use kernel::println;
use kernel::scheduler::{CPU_VAR, IDLE_THREAD, SCHEDULER};
use kernel::riscv::{
    r_sie, r_sstatus, w_sie, w_sscratch, w_sstatus, w_stvec, wfi, SIE_SEIE, SIE_SSIE, SIE_STIE,
    SSTATUS_SIE, SSTATUS_SUM
};
use kernel::timer::set_timer;
use kernel::vm::{kernel_vm_init, PAGE_R, PAGE_U, PAGE_W, PAGE_X};
use kernel::init_proc::kernel_init;
use core::arch::global_asm;

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

static ALIGNED: &'static AlignedTo<u8, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!("../shell"),
};

static ALIGNED_PONG: &'static AlignedTo<u8, [u8]> = &AlignedTo {
    _align: [],
    bytes: *include_bytes!("../pong"),
};

#[no_mangle]
static SHELL: &'static [u8] = &ALIGNED.bytes;
#[no_mangle]
static PONG: &'static [u8] = &ALIGNED_PONG.bytes;

#[export_name = "_kernel_main"]
extern "C" fn kernel_main(hartid: usize, _dtb_addr: PhysAddr, free_ram_phys: usize, free_ram_end_phys: usize) -> ! {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    };
    println!("booting kernel");

    w_stvec(trap_entry as usize);
    let bump_allocator = unsafe {
        BumpAllocator::new(free_ram_phys, free_ram_end_phys)
    };
    unsafe { kernel_vm_init(free_ram_end_phys) };

    println!("cpu id is {}", hartid);
    w_sscratch(&raw const CPU_VAR as usize);

    w_sstatus(SSTATUS_SUM);

    let elf_header = (SHELL as *const [u8]).cast::<Elf64Hdr>();

    set_timer(100000);

    w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);
    kernel_init(bump_allocator, elf_header);
    return_to_user()

}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
