use crate::capability::{cnode::CNodeCap, page_table::PageTableCap};
use crate::capability::Capability;
use crate::common::KernelResult;
use crate::object::PageTable;
use crate::println;
use common::list::ListItem;

use crate::scheduler::SCHEDULER;
use core::ops::{Index, IndexMut};

use super::cnode::CNodeEntry;

pub type ThreadControlBlock = ListItem<ThreadInfo>;

// Because type alias cannot impl method
#[allow(static_mut_refs)]
pub fn resume(thread: &mut ThreadControlBlock) {
    thread.resume();
    unsafe { SCHEDULER.push(thread) }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub enum ThreadState {
    #[default]
    Inactive,
    Runnable,
    Blocked,
    Idle,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Registers {
    pub ra: usize,
    pub sp: usize,
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

    // End of general purpose registers
    pub scause: usize,
    pub sstatus: usize,
    pub sepc: usize,
}

impl Registers {
    pub const fn null() -> Self {
        Self {
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            scause: 0,
            sstatus: 0,
            sepc: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct ThreadInfo {
    pub status: ThreadState,
    pub time_slice: usize,
    pub root_cnode: CNodeEntry,
    pub vspace: CNodeEntry,
    pub registers: Registers,
    pub msg_buffer: usize,
}

impl ThreadInfo {
    pub fn set_msg(&mut self, _msg_buffer: usize) {}
    pub fn resume(&mut self) {
        self.status = ThreadState::Runnable;
    }
    pub fn suspend(&mut self) {
        todo!()
    }
    pub fn is_runnable(&self) -> bool {
        self.status == ThreadState::Runnable
    }
    pub fn set_timeout(&mut self, time_out: usize) {
        self.time_slice = time_out
    }

    pub const fn idle_init() -> Self {
        Self {
            status: ThreadState::Idle,
            time_slice: 0,
            root_cnode: CNodeEntry::null(),
            vspace: CNodeEntry::null(),
            registers: Registers::null(),
            msg_buffer: 0,
        }
    }

    pub unsafe fn activate_vspace(&self) {
        if let Err(e) = self.activate_vspace_inner() {
            println!("{e:?}");
            PageTable::activate_kernel_table();
        }
    }

    unsafe fn activate_vspace_inner(&self) -> KernelResult<()> {
        let mut pt_cap = PageTableCap::try_from_raw(self.vspace.cap())?;
        unsafe { pt_cap.activate() }
    }

    pub fn set_root_cspace(&mut self, cspace_cap: CNodeCap, parent: &mut CNodeEntry) {
        assert!(self.root_cnode.is_null());
        self.root_cnode.insert(parent, cspace_cap.get_raw_cap());
    }

    pub fn set_root_vspace(&mut self, vspace_cap: PageTableCap, parent: &mut CNodeEntry) {
        assert!(self.root_cnode.is_null());
        self.vspace.insert(parent, vspace_cap.get_raw_cap());
    }
}

// TODO: use enum instead of usize
impl Index<usize> for Registers {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            1 => &self.ra,
            2 => &self.sp,
            3 => &self.gp,
            4 => &self.tp,
            5 => &self.t0,
            6 => &self.t1,
            7 => &self.t2,
            8 => &self.s0,
            9 => &self.s1,
            10 => &self.a0,
            11 => &self.a1,
            12 => &self.a2,
            13 => &self.a3,
            14 => &self.a4,
            15 => &self.a5,
            16 => &self.a6,
            17 => &self.a7,
            18 => &self.s2,
            19 => &self.s3,
            20 => &self.s4,
            21 => &self.s5,
            22 => &self.s6,
            23 => &self.s7,
            24 => &self.s8,
            25 => &self.s9,
            26 => &self.s10,
            27 => &self.s11,
            28 => &self.t3,
            29 => &self.t4,
            30 => &self.t5,
            31 => &self.t6,

            // end of gp rs
            32 => &self.scause,
            33 => &self.sstatus,
            34 => &self.sepc,
            _ => panic!("Unknown Index"),
        }
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            1 => &mut self.ra,
            2 => &mut self.sp,
            3 => &mut self.gp,
            4 => &mut self.tp,
            5 => &mut self.t0,
            6 => &mut self.t1,
            7 => &mut self.t2,
            8 => &mut self.s0,
            9 => &mut self.s1,
            10 => &mut self.a0,
            11 => &mut self.a1,
            12 => &mut self.a2,
            13 => &mut self.a3,
            14 => &mut self.a4,
            15 => &mut self.a5,
            16 => &mut self.a6,
            17 => &mut self.a7,
            18 => &mut self.s2,
            19 => &mut self.s3,
            20 => &mut self.s4,
            21 => &mut self.s5,
            22 => &mut self.s6,
            23 => &mut self.s7,
            24 => &mut self.s8,
            25 => &mut self.s9,
            26 => &mut self.s10,
            27 => &mut self.s11,
            28 => &mut self.t3,
            29 => &mut self.t4,
            30 => &mut self.t5,
            31 => &mut self.t6,

            // end of gp rs
            32 => &mut self.scause,
            33 => &mut self.sstatus,
            34 => &mut self.sepc,
            _ => panic!("Unknown Index"),
        }
    }
}
