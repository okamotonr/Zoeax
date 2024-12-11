use crate::common::{Err, KernelResult};
use crate::memory::PAGE_SIZE;
use crate::process::Process;
use crate::vm::SATP_SV48;
use common::list::{LinkedList, ListItem};
use core::arch::asm;
use core::arch::naked_asm;
use core::ptr;

type Proc<'a> = ListItem<'a, Process>;

const PROCS_MAX: usize = 8;
static mut PROCS: [Proc; PROCS_MAX] = [
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
    ListItem::new(Process::new()),
];

pub static mut IDLE_PROC: Proc = ListItem::new(Process::new());
pub static mut CURRENT_PROC: *mut Proc = ptr::null_mut();
pub const TICK_HZ: usize = 1000;
pub const TASK_QUANTUM: usize = 20 * (TICK_HZ / 1000); // 20 ms;
pub static mut CPU_VAR: CpuVar = CpuVar {
    sscratch: 0,
    sptop: 0,
};
pub static mut SCHEDULER: Scheduler<'static> = Scheduler::new();

pub fn allocate_proc(ip: usize) -> KernelResult<&'static mut ListItem<'static, Process>> {
    let (i, proc) = unsafe {
        PROCS
            .iter_mut()
            .enumerate()
            .find(|(_, ref x)| x.is_unused())
            .ok_or(Err::TooManyTasks)?
    };

    proc.init(ip, i + 1)?;
    Ok(proc)
}

#[repr(C)]
pub struct CpuVar {
    pub sscratch: usize,
    pub sptop: usize,
}

pub struct Scheduler<'a> {
    runqueue: LinkedList<'a, Process>,
}

impl<'a> Scheduler<'a> {
    pub const fn new() -> Self {
        Self {
            runqueue: LinkedList::new(),
        }
    }

    pub fn push(&mut self, proc: &'a mut ListItem<'a, Process>) {
        self.runqueue.push(proc);
    }

    pub fn sched(&mut self) -> Option<&mut ListItem<'a, Process>> {
        self.runqueue.pop()
    }
}

pub unsafe fn yield_proc() {
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
        &raw mut IDLE_PROC
    };

    let prev = CURRENT_PROC;
    switch_context(prev, next);
}

unsafe fn switch_context(prev: *mut Proc, next: *const Proc) {
    CURRENT_PROC = next as *mut Proc;
    CPU_VAR.sptop = (*next).stack_top.addr;
    asm!(
        "sfence.vma x0, x0",
        "csrw satp, {satp}",
        "sfence.vma x0, x0",
        satp = in(reg) (((*next).page_table.get_address() / PAGE_SIZE) | SATP_SV48)
    );
    asm_switch_context(&mut ((*prev).sp.addr), &(*next).sp.addr)
}

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

pub fn sleep(ms_time: usize) {
    if ms_time == 0 {
        return;
    }

    unsafe {
        (*CURRENT_PROC).set_timeout(ms_time);
        (*CURRENT_PROC).sleep();
    }
    unsafe {
        yield_proc();
    }
}

pub fn find_proc_by_id(pid: usize) -> Option<&'static mut Proc<'static>> {
    unsafe {
        for proc in PROCS.iter_mut() {
            if proc.pid == pid {
                return Some(proc);
            }
        }
    }
    None
}

pub fn count_down(tick: usize) {
    unsafe {
        for proc in PROCS.iter_mut() {
            (*proc).timeout_ms = (*proc).timeout_ms.saturating_sub(tick);
            if (*proc).timeout_ms == 0 && (*proc).is_sleeping() {
                (*proc).resume();
                SCHEDULER.push(proc);
            }
        }
    }

    unsafe {
        if (*CURRENT_PROC).timeout_ms == 0 {
            yield_proc();
        }
    }
}

pub fn init_proc() {
    unsafe {
        IDLE_PROC.init(0, 0).unwrap();
        IDLE_PROC.waiting();
        CURRENT_PROC = &raw mut IDLE_PROC;
    };
}

