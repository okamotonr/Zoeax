#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{arch::naked_asm, cmp::min, panic::PanicInfo, ptr};
use core::arch::asm;

use kernel::common::align_up;
use kernel::handler::trap_entry;
use kernel::memory::{alloc_pages, init_memory, VirtAddr, PAGE_SIZE};
use kernel::println;
use kernel::process::{init_proc, yield_proc, Process, CPU_VAR, IDLE_PROC};
use kernel::riscv::{
    r_sie, r_sstatus, w_sie, w_sscratch, w_sstatus, w_stvec, wfi, SIE_SEIE, SIE_SSIE, SIE_STIE,
    SSTATUS_SIE, SSTATUS_SUM
};
use kernel::timer::set_timer;
use kernel::vm::{kernel_vm_init, PAGE_R, PAGE_U, PAGE_W, PAGE_X};

use common::elf::*;
extern "C" {
    static mut __bss: u8;
    static __bss_end: u8;
    static __stack_top: u8;
    pub static __kernel_base: u8;
}

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

#[no_mangle]
fn kernel_main(hartid: usize) {
    unsafe {
        let bss = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    };
    println!("booting kernel");

    w_stvec(trap_entry as usize);

    init_memory();
    unsafe { kernel_vm_init().unwrap() };

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
        let init_proc = Process::allocate((*elf_header).e_entry).unwrap();
        let pong = Process::allocate((*pong_elf).e_entry).unwrap();
        load_elf(init_proc, elf_header);
        load_elf(pong, pong_elf);

        println!("{:?}, {:?}, {:?}", init_proc.pid, init_proc.stack_bottom, init_proc.stack_top);
        println!("{:?}, {:?}, {:?}", pong.pid, pong.stack_bottom, pong.stack_top);
        println!("{:x}", *(init_proc.stack_bottom.addr as *const usize));
    }

    set_timer(100000);

    w_sie(r_sie() | SIE_SEIE | SIE_STIE | SIE_SSIE);

    idle()
}

#[no_mangle]
fn idle() -> ! {
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

fn load_elf(process: &mut Process, elf_header: *const Elf64Hdr) {
    unsafe {
        for idx in 0..(*elf_header).e_phnum {
            let p_header = (*elf_header)
                .get_pheader(elf_header.cast::<usize>(), idx)
                .unwrap();
            if !((*p_header).p_type == ProgramType::Load) {
                continue;
            }
            let flags = get_flags((*p_header).p_flags) | PAGE_U;
            // this is start address of mapping segment
            let p_vaddr = VirtAddr::new((*p_header).p_vaddr);
            let p_start_addr = elf_header.cast::<usize>().byte_add((*p_header).p_offset);
            // Sometime memsz > filesz, for example bss
            // so have to call copy with caring of this situation.
            let page_num = (align_up((*p_header).p_memsz, PAGE_SIZE)) / PAGE_SIZE;
            let mut file_sz_rem = (*p_header).p_filesz;

            for page_idx in 0..page_num {
                let page = alloc_pages(1).unwrap();
                if !(file_sz_rem == 0) {
                    let copy_src = p_start_addr.byte_add(PAGE_SIZE * page_idx);
                    let copy_dst = page.addr as *mut usize;
                    let copy_size = min(PAGE_SIZE, file_sz_rem);
                    file_sz_rem = file_sz_rem.saturating_sub(PAGE_SIZE);
                    ptr::copy(copy_src, copy_dst, copy_size);
                }
                process.map_page(p_vaddr.add(PAGE_SIZE * page_idx), page, flags);
            }
        }
    }
}

#[inline]
fn get_flags(flags: u32) -> usize {
    let ret = if ProgramFlags::is_executable(flags) {
        PAGE_X
    } else {
        0
    } | if ProgramFlags::is_writable(flags) {
        PAGE_W
    } else {
        0
    } | if ProgramFlags::is_readable(flags) {
        PAGE_R
    } else {
        0
    };
    ret
}
