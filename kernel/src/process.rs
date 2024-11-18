use crate::memory::{PhysAddr, VirtAddr};
use crate::{
    memory::{__free_ram_end, alloc_pages, PAGE_SIZE},
    page::{map_page, PAGE_R, PAGE_U, PAGE_V, PAGE_W, PAGE_X, SATP_SV48},
    write_csr,
};
use core::{
    any::type_name,
    arch::{asm, naked_asm},
    mem, ptr,
};

extern "C" {
    static __kernel_base: u8;
}

const PROCS_MAX: usize = 8;
static mut PROCS: [Process; PROCS_MAX] = [Process::init(); PROCS_MAX];

pub static mut CURRENT_PROC: *mut Process = ptr::null_mut();
pub static mut IDLE_PROC: *mut Process = ptr::null_mut();
const USER_BASE: usize = 0x1000000;
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

#[naked]
extern "C" fn user_entry() {
    unsafe {
        naked_asm!(
            "la a0, {sepc}",
            "csrw sepc, a0",
            "la a0, {sstatus}",
            "csrw sstatus, a0",
            "sret",
            sepc = const USER_BASE,
            sstatus = const SSTATUS_SPIE,
        );
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
    pub fn create(image: *const usize, image_size: usize) {
        let (i, proc) = unsafe {
            PROCS
                .iter_mut()
                .enumerate()
                .find(|(_, &mut x)| x.is_usable())
                .expect("no free process slot")
        };

        proc.pid = i + 1;
        proc.status = ProcessStatus::Runnable;

        unsafe {
            let sp = (&mut proc.stack as *mut [u8] as *mut u8).add(mem::size_of_val(&proc.stack))
                as *mut usize;
            let stack = ptr::addr_of_mut!((*proc).stack) as *mut u8;
            let _sp = stack.add(proc.stack.len());
            *sp.sub(1) = 1; // s11
            *sp.sub(2) = 2; // s10
            *sp.sub(3) = 3; // s9
            *sp.sub(4) = 4; // s8
            *sp.sub(5) = 5; // s7
            *sp.sub(6) = 6; // s6
            *sp.sub(7) = 7; // s5
            *sp.sub(8) = 8; // s4
            *sp.sub(9) = 9; // s3
            *sp.sub(10) = 10; // s2
            *sp.sub(11) = 11; // s1
            *sp.sub(12) = 12; // s0
            *sp.sub(13) = user_entry as usize; // ra
            proc.sp = VirtAddr::new(sp.sub(13) as usize);
        }

        let page_table = alloc_pages(1);
        let mut paddr = PhysAddr::new(ptr::addr_of!(__kernel_base) as *const u8 as usize);
        while paddr < PhysAddr::new(ptr::addr_of!(__free_ram_end) as *const u8 as usize) {
            unsafe { map_page(page_table, paddr.into(), paddr, PAGE_R | PAGE_W | PAGE_X) };
            paddr += PhysAddr::new(PAGE_SIZE);
        }
        let mut off = VirtAddr::new(0);
        let pimage = image;
            while off < image_size.into() {
            let page = alloc_pages(1);
            unsafe {
                ptr::copy(pimage.add(off.into()), page.addr as *mut usize , PAGE_SIZE as usize);
            }
            map_page(
                page_table,
                off + USER_BASE.into(),
                page,
                PAGE_U | PAGE_R | PAGE_W | PAGE_X,
            );
            off += PAGE_SIZE.into();
        }
        proc.page_table = page_table
    }

    fn is_usable(&self) -> bool {
        self.status == ProcessStatus::Unused
    }
    fn is_runnable(&self) -> bool {
        self.status == ProcessStatus::Runnable
    }
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
