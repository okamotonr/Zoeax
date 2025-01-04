#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{arch::naked_asm, cmp::min, panic::PanicInfo, ptr};
use core::arch::asm;

use kernel::common::align_up;
use kernel::handler::trap_entry;
use kernel::memory::{alloc_pages, init_memory, VirtAddr, PAGE_SIZE};
use kernel::process::Process;
use kernel::println;
use kernel::scheduler::{yield_proc, allocate_proc, CPU_VAR, init_proc, IDLE_PROC, SCHEDULER};
use kernel::riscv::{
    r_sie, r_sstatus, w_sie, w_sscratch, w_sstatus, w_stvec, wfi, SIE_SEIE, SIE_SSIE, SIE_STIE,
    SSTATUS_SIE, SSTATUS_SUM
};
use kernel::timer::set_timer;
use kernel::vm::{alloc_vm, kernel_vm_init, PAGE_R, PAGE_U, PAGE_W, PAGE_X};
use kernel::init_proc::load_elf;
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
extern "C" fn kernel_main(hartid: usize, _dtb_addr: usize, free_ram_phys: usize, free_ram_end_phys: usize) {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    };
    println!("booting kernel");

    w_stvec(trap_entry as usize);

    init_memory(free_ram_phys, free_ram_end_phys);
    unsafe { kernel_vm_init(free_ram_phys, free_ram_end_phys).unwrap() };

    println!("cpu id is {}", hartid);
    let cpu_var = &raw const CPU_VAR;
    w_sscratch(cpu_var as usize);

    // unsafe {
    //     virtio::init();
    //     println!("init virtio");
    //
    //     let mut buf: [u8; virtio::SECTOR_SIZE as usize] = [0; virtio::SECTOR_SIZE as usize];
    //     virtio::read_write_disk(&mut buf as *mut [u8] as *mut u8, 0, false).unwrap();
    //     let text = buf.iter().take_while(|c| **c != 0);
    //     for c in text {
    //         print!("{}", *c as char);
    //     }
    //     println!();
    //
    //     let buf = b"hello from kernel!!!\n";
    //     virtio::read_write_disk(buf as *const [u8] as *mut u8, 0, true).unwrap();
    // }


    init_proc();
    w_sstatus(SSTATUS_SUM);

    let elf_header = (SHELL as *const [u8]).cast::<Elf64Hdr>();
    let pong_elf = (PONG as *const [u8]).cast::<Elf64Hdr>();

    unsafe {
        let init_proc = allocate_proc((*elf_header).e_entry).unwrap();
        let pong = allocate_proc((*pong_elf).e_entry).unwrap();
        load_elf(init_proc, elf_header);
        load_elf(pong, pong_elf);


        println!("{:?}, {:?}, {:?}", init_proc.pid, init_proc.stack_bottom, init_proc.stack_top);
        println!("{:?}, {:?}, {:?}", pong.pid, pong.stack_bottom, pong.stack_top);
        println!("{:x}", *(init_proc.stack_bottom.addr as *const usize));
        SCHEDULER.push(init_proc);
        SCHEDULER.push(pong);
    }

    set_timer(100000);

    w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

    idle()
}

#[no_mangle]
fn idle() -> ! {
    println!("enter idle");
    unsafe {
        // initialize
        let sp = IDLE_PROC.sp.addr;
        asm!("mv sp, {sp}", sp = in(reg) sp);
    }
    loop {
        unsafe { yield_proc() }
        w_sstatus(r_sstatus() | SSTATUS_SIE);
        wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
