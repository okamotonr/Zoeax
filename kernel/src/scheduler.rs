use crate::common::{Err, KernelResult};
use crate::object::{ThreadInfo, ThreadControlBlock};
use common::list::{LinkedList, ListItem};
use core::arch::naked_asm;
use core::ptr;
use crate::riscv::{r_sstatus, w_sstatus, wfi, SSTATUS_SIE, SSTATUS_SPIE, SSTATUS_SPP};

// TODO: use once_cell
pub static mut IDLE_THREAD: ThreadControlBlock = ThreadControlBlock::new(ThreadInfo::idle_init());

// TODO: use unsafe_cell
pub static mut CURRENT_PROC: *mut ThreadControlBlock = ptr::null_mut();
pub const TICK_HZ: usize = 1000;
pub const TASK_QUANTUM: usize = 20 * (TICK_HZ / 1000); // 20 ms;
pub static mut CPU_VAR: CpuVar = CpuVar {
    sscratch: 0,
    sptop: 0,
};

// TODO: use unsafe_cell
pub static mut SCHEDULER: Scheduler = Scheduler::new();

extern "C" {
    static __stack_top: u8;
}

#[repr(C)]
pub struct CpuVar {
    pub sscratch: usize,
    pub sptop: usize,
}

pub struct Scheduler {
    runqueue: LinkedList<ThreadInfo>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            runqueue: LinkedList::new(),
        }
    }

    pub fn push(&mut self, proc: &mut ThreadControlBlock) {
        self.runqueue.push(proc);
    }

    pub fn sched(&mut self) -> Option<&mut ListItem<ThreadInfo>> {
        self.runqueue.pop()
    }
}

pub unsafe fn schedule() {
    let next = if let Some(next) = SCHEDULER.sched() {
        next.set_timeout(TASK_QUANTUM);
        if (*CURRENT_PROC).is_runnable() {
            SCHEDULER.push(CURRENT_PROC.as_mut().unwrap());
        }
        next
    } else {
        if (*CURRENT_PROC).is_runnable() {
            return;
        }
        &raw mut IDLE_THREAD
    };
    // change page table
    (*next).activate_vspace();
    CURRENT_PROC = next;
}

fn switch_context(prev: &ThreadInfo, next: &ThreadInfo) {}

#[naked]
#[no_mangle]
pub extern "C" fn asm_switch_context(prev_sp: *mut usize, next_sp: *const usize) {
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

pub fn create_idle_thread() {
    unsafe {
        (*IDLE_THREAD).registers.nextpc = idle as usize;
        (*IDLE_THREAD).registers.sstatus = SSTATUS_SPP | SSTATUS_SPIE;
        (*IDLE_THREAD).registers.sp = __stack_top as usize;
    }
}

#[no_mangle]
fn idle() -> ! {
    loop {
        w_sstatus(r_sstatus() | SSTATUS_SIE);
        wfi();
    }
}

pub fn get_current_tcb<'a>() -> &'a ThreadControlBlock {
    unsafe {
        & *CURRENT_PROC
    }
}


