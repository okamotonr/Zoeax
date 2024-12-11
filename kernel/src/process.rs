use common::syscall::Message;

use crate::{
    common::{Err, KernelResult},
    memory::{PhysAddr, VirtAddr, PAGE_SIZE},
    riscv::{w_sepc, SSTATUS_SPIE, w_sstatus, r_sstatus},
    vm::{allocate_page_table, map_page, PageTableAddress},
};
use core::{
    arch::{asm, naked_asm},
    ptr,
};


const STACK_SIZE: usize = PAGE_SIZE * 4;
const STACK_CANARY: usize = 0xdeadbee21;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Unused,
    Runnable,
    Sleeping,
    Wating
}

#[derive(Debug, Clone, Copy)]
pub struct Process {
    pub pid: usize,
    pub status: ProcessStatus,
    pub stack_top: VirtAddr,
    pub sp: VirtAddr,
    pub stack: [u8; STACK_SIZE],
    pub stack_bottom: VirtAddr,
    pub page_table: PageTableAddress,
    pub timeout_ms: usize,
    pub message: Option<Message>,
    pub waiter: *mut Process
}

#[no_mangle]
fn user_entry(ip: usize) -> ! {
    unsafe {
        w_sepc(ip);
        w_sstatus(r_sstatus() | SSTATUS_SPIE);
        asm!(
            "sret",
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
            stack: [0; STACK_SIZE],
            stack_bottom: VirtAddr::new(0),
            page_table: PageTableAddress::init(),
            timeout_ms: 0,
            message: None,
            waiter: ptr::null_mut()
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
            let sp_bottom = &self.stack[0] as *const u8;
            let sp = sp_bottom.add(STACK_SIZE) as *mut usize;
            let sp_top = VirtAddr::new(sp as usize);
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
            self.stack_bottom = VirtAddr::new(sp_bottom as usize);
            *(self.stack_bottom.addr as *mut usize) = STACK_CANARY;
        }

        let page_table = unsafe { allocate_page_table().unwrap() };

        self.page_table = page_table;
        Ok(())

    }

    pub fn set_message(&mut self, message: Message) -> KernelResult<()> {
        if self.is_unused() {
            Err(Err::ProcessNotFound)
        }
        else if self.message.is_some() {
            Err(Err::MessageBoxIsFull)
        } else {
            self.message = Some(message);
            if self.status == ProcessStatus::Wating {
                self.status = ProcessStatus::Runnable;
            }
            Ok(())
        }
    }

    pub fn waiting(&mut self) {
        self.status = ProcessStatus::Wating
    }

    pub fn resume(&mut self) {
        self.status = ProcessStatus::Runnable
    }

    pub fn sleep(&mut self) {
        self.status = ProcessStatus::Sleeping
    }

    pub fn is_waiting(&self) -> bool {
        self.status == ProcessStatus::Wating
    }

    pub fn is_unused(&self) -> bool {
        self.status == ProcessStatus::Unused
    }
    pub fn is_runnable(&self) -> bool {
        self.status == ProcessStatus::Runnable
    }

    pub fn is_sleeping(&self) -> bool {
        self.status == ProcessStatus::Sleeping
    }

    pub fn set_timeout(&mut self, timeout_ms: usize) {
        self.timeout_ms = timeout_ms
    }
}


pub fn check_canary() {
    let mut top: usize;
    unsafe {
        asm!(
            "csrrw tp, sscratch, tp",
            "ld {}, 8 * 1(tp)",
            "csrrw tp, sscratch, tp",
            out(reg) top
        );
        let bottom = (top - STACK_SIZE) as *const usize;
        assert!(*bottom == STACK_CANARY)
    }
}
