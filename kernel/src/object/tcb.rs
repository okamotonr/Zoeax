use core::ptr::NonNull;

use common::list::ListItem;

use crate::memory::{VirtAddr, PAGE_SIZE};
use crate::vm::PageTable;

use super::cnode::CNodeEntry;

const STACK_SIZE: usize = PAGE_SIZE * 4;
pub type ThreadControlBlock<'a> = ListItem<'a, ThreadInfo>;

pub enum ThreadState {
    Inactive,
    Runnable,
    Blocked,
}


#[repr(C)]
#[derive(Debug)]
pub struct Registers {
    pub pc: usize,
    pub sstatus: usize,
    pub sp: usize,
    pub ra: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

pub struct ThreadInfo {
    pub status: ThreadState,
    pub time_slice: usize,
    pub page_table: PageTable,
    pub stack_top: VirtAddr,
    pub stack_bottom: VirtAddr,
    pub sp: VirtAddr,
    pub stack: [u8; STACK_SIZE],
    pub root_cnode: CNodeEntry,
    pub vspace: CNodeEntry,
    pub registers: Registers,
    pub msg_buffer: usize
}

impl ThreadInfo {
    pub fn set_msg(&mut self, msg_buffer: usize) {
    }
}

