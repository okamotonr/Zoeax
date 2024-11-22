use crate::common::align_up;
use crate::memory::{PhysAddr, VirtAddr};
use crate::{
    memory::{__free_ram_end, alloc_pages, PAGE_SIZE},
    page::{map_page, PAGE_R, PAGE_U, PAGE_V, PAGE_W, PAGE_X, SATP_SV48},
    write_csr,
    riscv::w_sepc,
    println,
};
use core::cmp::min;
use core::{
    any::type_name,
    arch::{asm, naked_asm},
    mem, ptr,
};

use common::elf::*;

extern "C" {
    static __kernel_base: u8;
}

const PROCS_MAX: usize = 8;
static mut PROCS: [Process; PROCS_MAX] = [Process::init(); PROCS_MAX];

pub static mut CURRENT_PROC: *mut Process = ptr::null_mut();
pub static mut IDLE_PROC: *mut Process = ptr::null_mut();
const SSTATUS_SPIE: u64 = 1 << 5;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Unused,
    Runnable,
    Waiting,
}

#[derive(Debug, Clone, Copy)]
pub struct Process {
    pub pid: usize,
    pub status: ProcessStatus,
    sp: VirtAddr,
    pub stack: [u8; 8192],
    page_table: PhysAddr,
}

#[no_mangle]
fn user_entry(ip: usize) -> ! {
    // a0 is ip,
    unsafe {
        w_sepc(ip);
        asm!(
            "la a0, {sstatus}",
            "csrw sstatus, a0",
            "sret",
            sstatus = const SSTATUS_SPIE,
            options(noreturn)
        );
    }
}

#[no_mangle]
#[naked]
extern "C" fn user_entry_trampoline() {
    unsafe {
      naked_asm!(
      "ld a0, 0 * 8(sp)", // ip
      "j user_entry"
      )
    }
}

impl Process {
    pub const fn init() -> Self {
        Self {
            pid: 0,
            status: ProcessStatus::Unused,
            sp: VirtAddr::new(0),
            stack: [0; 8192],
            page_table: PhysAddr::new(0),
        }
    }
    pub fn create(elf_image: *const [u8]) {
        let (i, proc) = unsafe {
            PROCS
                .iter_mut()
                .enumerate()
                .find(|(_, &mut x)| x.is_usable())
                .expect("no free process slot")
        };

        proc.pid = i + 1;
        proc.status = ProcessStatus::Runnable;

        let elf_header = elf_image as *const Elf64Hdr;
        println!("{:?}", elf_header);

        // kernel stack
        unsafe {
            let sp = (&mut proc.stack as *mut [u8] as *mut u8).add(mem::size_of_val(&proc.stack))
                as *mut usize;
            let stack = ptr::addr_of_mut!((*proc).stack) as *mut u8;
            let _sp = stack.add(proc.stack.len());
            *sp.sub(1) = (*elf_header).e_entry; // s0
            *sp.sub(2) = 0; // s11
            *sp.sub(3) = 0; // s10
            *sp.sub(4) = 0; // s9
            *sp.sub(5) = 0; // s8
            *sp.sub(6) = 0; // s7
            *sp.sub(7) = 0; // s6
            *sp.sub(8) = 0; // s5
            *sp.sub(9) = 0; // s4
            *sp.sub(10) = 0; // s3
            *sp.sub(11) = 0; // s2
            *sp.sub(12) = 0; // s1
            *sp.sub(13) = 0; // s0
            *sp.sub(14) = user_entry_trampoline as usize; // ra
            proc.sp = VirtAddr::new(sp.sub(14) as usize);
        }

        // kernel memory
        let page_table = alloc_pages(1);
        let mut paddr = PhysAddr::new(ptr::addr_of!(__kernel_base) as *const u8 as usize);
        while paddr < PhysAddr::new(ptr::addr_of!(__free_ram_end) as *const u8 as usize) {
            unsafe { map_page(page_table, paddr.into(), paddr, PAGE_R | PAGE_W | PAGE_X) };
            paddr += PhysAddr::new(PAGE_SIZE);
        }


        load_elf(page_table, elf_header);
        
        proc.page_table = page_table
    }

    fn is_usable(&self) -> bool {
        self.status == ProcessStatus::Unused
    }
    fn is_runnable(&self) -> bool {
        self.status == ProcessStatus::Runnable
    }
}

fn load_elf(page_table: PhysAddr, elf_header: *const Elf64Hdr) {
    let mut idx = 0;

    unsafe {
        println!("elf addr is {:?}", elf_header);
        println!("elf ident addr is {:?}", ptr::addr_of!((*elf_header).e_ident));
        println!("elf ident head is {:?}", *(ptr::addr_of!((*elf_header).e_ident) as *const u8));
        println!("elf magic is {:?}", (*elf_header).e_ident);
        println!("elf header, {:?}, {:0x}, {:?}", (*elf_header).e_phnum,
        (*elf_header).e_entry, (*elf_header).e_phoff
            );
        while idx < (*elf_header).e_phnum {
            println!("before Get");
            let p_header = (*elf_header).get_pheader(elf_header.cast::<usize>(), idx).unwrap(); 
            if !((*p_header).p_type == ProgramType::Load) {
                idx += 1;
                continue
            }
            println!("after Get, {:?}", p_header);
            let flags = get_flags((*p_header).p_flags) | PAGE_U;
            // this is start address of mapping segment
            let p_vaddr = VirtAddr::new((*p_header).p_vaddr);
            let p_start_addr = elf_header.cast::<usize>().add((*p_header).p_offset);
            // Sometime memsz > filesz, for example bss
            // so have to call copy with caring of this situation.
            let page_num = (align_up((*p_header).p_memsz, PAGE_SIZE)) / PAGE_SIZE;
            let mut file_sz_rem = (*p_header).p_filesz;

            println!("Before map, {:?}, {:?}, {:?}, {:?}, {:?}", file_sz_rem, p_start_addr, page_num, page_num, p_vaddr);
            for page_idx in 0..page_num {
                let page = alloc_pages(1);
                println!("Before copy");
                let copy_src = p_start_addr.add(PAGE_SIZE * page_idx);
                let copy_dst = page.addr as *mut usize;
                let copy_size = min(PAGE_SIZE, file_sz_rem);
                println!("Copy args {:?}, {:?}, {:?}", copy_src, copy_dst, copy_size);
                ptr::copy(copy_src, copy_dst, copy_size);
                println!("After copy");
                map_page(
                    page_table,
                    p_vaddr.add(PAGE_SIZE * page_idx),
                    page,
                    flags
                );
                file_sz_rem = file_sz_rem.wrapping_sub(PAGE_SIZE);
            }
            println!("After map, {:?}", idx);
            idx += 1;
        }
    }
}

#[inline]
fn get_flags(flags: u32) -> usize {
    let ret = if ProgramFlags::is_executable(flags) { PAGE_X } else {0} | if ProgramFlags::is_writable(flags) { PAGE_W } else {0} | if ProgramFlags::is_readable(flags) {PAGE_R} else {0};
    ret
}

pub unsafe fn yield_proc() {
    let mut next = IDLE_PROC;
    let current_pid = (*CURRENT_PROC).pid;
    for i in 0..PROCS_MAX {
        let proc = &mut PROCS[current_pid.wrapping_add(i) % PROCS_MAX] as *mut Process;
        if (*proc).is_runnable() && (*proc).pid > 0 {
            next = proc;
            break;
        }
    }

    if (*next).pid == current_pid {
        return;
    }

    let prev = CURRENT_PROC;
    CURRENT_PROC = next;
    unsafe {
        asm!(
            "sfence.vma",
            "csrw satp, {satp}",
            "sfence.vma",
            satp = in(reg) (((*next).page_table.addr / PAGE_SIZE) | SATP_SV48)
        );
        write_csr!(
            "sscratch",
            (&mut (*next).stack as *mut [u8] as *mut u8)
                .offset(mem::size_of_val(&(*next).stack) as isize) as *mut u64
        );
    }

    switch_context(&mut ((*prev).sp.addr), &(*next).sp.addr)
}

pub fn init_proc() {
    let proc = unsafe { &mut PROCS[0] };
    proc.status = ProcessStatus::Runnable;
    proc.pid = 0;
    unsafe {
        let stack = ptr::addr_of_mut!(proc.stack) as *mut usize;
        let sp = stack.add(proc.stack.len());
        *sp.offset(-1) = 0; // s11
        *sp.offset(-2) = 0; // s10
        *sp.offset(-3) = 0; // s9
        *sp.offset(-4) = 0; // s8
        *sp.offset(-5) = 0; // s7
        *sp.offset(-6) = 0; // s6
        *sp.offset(-7) = 0; // s5
        *sp.offset(-8) = 0; // s4
        *sp.offset(-9) = 0; // s3
        *sp.offset(-10) = 0; // s2
        *sp.offset(-11) = 0; // s1
        *sp.offset(-12) = 0; // s0
        *sp.offset(-13) = 0; // ra
        proc.sp = VirtAddr::new(sp.offset(-13) as usize);
    };

    let page_table = alloc_pages(1);
    let mut paddr = PhysAddr::new(ptr::addr_of!(__kernel_base) as *const u8 as usize);
    while paddr < PhysAddr::new(ptr::addr_of!(__free_ram_end) as *const u8 as usize) {
        unsafe {
            map_page(page_table, paddr.into(), paddr, PAGE_R | PAGE_W | PAGE_X);
        }
        paddr += PhysAddr::new(PAGE_SIZE);
    }
    proc.page_table = page_table;

    unsafe {
        CURRENT_PROC = proc;
        IDLE_PROC = proc;
    };
}

#[naked]
#[no_mangle]
pub extern "C" fn switch_context(prev_sp: *mut usize, next_sp: *const usize) {
    unsafe {
        naked_asm!(
            "addi sp, sp, -13 * 8",
            "sd ra, 0 * 8(sp)",
            "sd s0, 1 * 8(sp)",
            "sd s1, 2 * 8(sp)",
            "sd s2, 3 * 8(sp)",
            "sd s3, 4 * 8(sp)",
            "sd s4, 5 * 8(sp)",
            "sd s5, 6 * 8(sp)",
            "sd s6, 7 * 8(sp)",
            "sd s7, 8 * 8(sp)",
            "sd s8, 9 * 8(sp)",
            "sd s9, 10 * 8(sp)",
            "sd s10, 11 * 8(sp)",
            "sd s11, 12 * 8(sp)",
            "sd sp, (a0)",
            "ld sp, (a1)",
            "ld ra, 0 * 8(sp)",
            "ld s0, 1 * 8(sp)",
            "ld s1, 2 * 8(sp)",
            "ld s2, 3 * 8(sp)",
            "ld s3, 4 * 8(sp)",
            "ld s4, 5 * 8(sp)",
            "ld s5, 6 * 8(sp)",
            "ld s6, 7 * 8(sp)",
            "ld s7, 8 * 8(sp)",
            "ld s8, 9 * 8(sp)",
            "ld s9, 10 * 8(sp)",
            "ld s10, 11 * 8(sp)",
            "ld s11, 12 * 8(sp)",
            "addi sp, sp, 13 * 8",
            "ret"
        )
    }
}
