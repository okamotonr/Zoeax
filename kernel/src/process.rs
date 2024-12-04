use crate::{
    common::{Err, KernelResult},
    memory::{PhysAddr, VirtAddr, PAGE_SIZE},
    println,
    riscv::{w_sepc, SSTATUS_SPIE},
    vm::{allocate_page_table, map_page, PageTableAddress, SATP_SV48},
};
use core::{
    arch::{asm, naked_asm},
    mem, ptr,
};

extern "C" {
    static __kernel_base: u8;
}

const PROCS_MAX: usize = 8;
static mut PROCS: [Process; PROCS_MAX] = [Process::new(); PROCS_MAX];

pub const TICK_HZ: usize = 1000;
pub const TASK_QUANTUM: usize = (20 * (TICK_HZ / 1000)); /* 20ミリ秒 */
pub static mut CPU_VAR: CpuVar = CpuVar {sscratch: 0, sptop: 0};
pub static mut CURRENT_PROC: *mut Process = ptr::null_mut();
pub static mut IDLE_PROC: Process = Process::new();


#[repr(C)]
pub struct CpuVar {
    pub sscratch: usize,
    pub sptop: usize,
}


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
    pub stack_top: VirtAddr,
    sp: VirtAddr,
    pub stack: [u8; 8192],
    page_table: PageTableAddress,
    timeout_ms: usize,
}

#[no_mangle]
fn user_entry(ip: usize) -> ! {
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
            ".balign 8",
            "ld a0, 0 * 8(sp)", // ip
            "j user_entry"
        )
    }
}

impl Process {
    pub const fn new() -> Self {
        Self {
            pid: 0,
            status: ProcessStatus::Unused,
            stack_top: VirtAddr::new(0),
            sp: VirtAddr::new(0),
            stack: [0; 8192],
            page_table: PageTableAddress::init(),
            timeout_ms: 0
        }
    }

    pub fn map_page(&mut self, v_addr: VirtAddr, p_addr: PhysAddr, flags: usize) {
        unsafe {
            map_page(self.page_table, v_addr, p_addr, flags).unwrap();
        }
    }

    pub fn init(&mut self, ip: usize, pid: usize) -> KernelResult<()> {
        self.pid = pid;
        self.status = ProcessStatus::Runnable;

        // kernel stack
        unsafe {
            let sp = (&mut self.stack as *mut [u8] as *mut u8).add(mem::size_of_val(&self.stack))
                as *mut usize;
            let sp_top = VirtAddr::new(sp as usize);
            println!("stack pointer {:?}", sp);
            println!("address is {:p}", &self.stack[0]);
            println!("address is {:p}", &self.stack[8191]);
            let stack = ptr::addr_of_mut!((*self).stack) as *mut u8;
            let _sp = stack.add(self.stack.len());
            println!("{}", self.stack.len());
            println!("stack pointer {:?}", _sp);
            println!("stack pointer {:?}", _sp.sub(1));
            *sp.sub(1) = ip;
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
            self.sp = VirtAddr::new(sp.sub(14) as usize);
            self.stack_top = sp_top;
        }

        let page_table = unsafe { allocate_page_table().unwrap() };

        self.page_table = page_table;
        Ok(())

    }
    pub fn allocate(ip: usize) -> KernelResult<&'static mut Self> {
        let (i, proc) = unsafe {
            PROCS
                .iter_mut()
                .enumerate()
                .find(|(_, &mut x)| x.is_usable())
                .ok_or(Err::TooManyTasks)?
        };

        proc.init(ip, i + 1)?;
        Ok(proc)
    }


    fn is_usable(&self) -> bool {
        self.status == ProcessStatus::Unused
    }
    fn is_runnable(&self) -> bool {
        self.status == ProcessStatus::Runnable
    }
}

pub unsafe fn yield_proc() {
    let mut next = &raw mut IDLE_PROC;
    let current_pid = (*CURRENT_PROC).pid;
    for i in 0..PROCS_MAX {
        let proc = &mut PROCS[current_pid.wrapping_add(i) % PROCS_MAX] as *mut Process;
        if (*proc).is_runnable() && (*proc).pid > 0 {
            next = proc;
            break;
        }
    }

    if (*next).pid != 0 {
        (*next).timeout_ms = TASK_QUANTUM;
    }

    if (*next).pid == current_pid {
        return;
    }

    let prev = CURRENT_PROC;
    CURRENT_PROC = next;
    unsafe {
        CPU_VAR.sptop = (*next).stack_top.addr;
        asm!(
            "sfence.vma",
            "csrw satp, {satp}",
            "sfence.vma",
            satp = in(reg) (((*next).page_table.get_address() / PAGE_SIZE) | SATP_SV48)
        );
    }

    switch_context(&mut ((*prev).sp.addr), &(*next).sp.addr)
}

pub fn init_proc() {
    unsafe {
        IDLE_PROC.init(0, 0).unwrap();
        CURRENT_PROC = &raw mut IDLE_PROC;
    };
}

pub fn sleep(ms_time: usize) {
    if ms_time == 0 {
        return
    }
    println!("timeout is {}", ms_time);

    unsafe {
        (*CURRENT_PROC).timeout_ms = ms_time;
        (*CURRENT_PROC).status = ProcessStatus::Waiting;
    }
    unsafe {
        yield_proc();
    }
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

pub fn count_down(tick: usize) {
    unsafe {
        for proc in PROCS.iter_mut() {
            (*proc).timeout_ms = (*proc).timeout_ms.saturating_sub(tick);
            if (*proc).timeout_ms == 0 && (*proc).status == ProcessStatus::Waiting {
                (*proc).status = ProcessStatus::Runnable;
            }
        }
    }

    unsafe {
        if (*CURRENT_PROC).timeout_ms == 0 {
            yield_proc();
        }
    }
}

