use crate::object::{ThreadInfo, ThreadControlBlock};
use crate::println;
use common::list::{LinkedList, ListItem};
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
#[derive(Debug)]
pub struct CpuVar {
    pub sscratch: usize,
    pub sptop: usize,
}

#[derive(Default)]
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

// TODO: remove this attribute
#[allow(static_mut_refs)]
pub unsafe fn schedule() {
    let next = if let Some(next) = SCHEDULER.sched() {
        println!("get next");
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

pub fn create_idle_thread() {
    unsafe {
        IDLE_THREAD.registers.sepc = idle as usize;
        IDLE_THREAD.registers.sstatus = SSTATUS_SPP | SSTATUS_SPIE;
        IDLE_THREAD.registers.sp = &raw const __stack_top as usize;
        CURRENT_PROC = &raw mut IDLE_THREAD;
    }
}

#[no_mangle]
fn idle() -> ! {
    println!("In the Idle");
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


