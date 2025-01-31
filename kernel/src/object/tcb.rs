use crate::capability::page_table::PageCap;
use crate::capability::{cnode::CNodeCap, page_table::PageTableCap};
use crate::common::{ErrKind, KernelResult};
use crate::kerr;
use crate::list::ListItem;
use crate::object::PageTable;
use crate::println;

use crate::scheduler::push;
use core::ops::{Index, IndexMut};
use core::ptr;

use super::cnode::CNodeEntry;
use super::page_table::Page;
use super::{CNode, KObject};
#[cfg(debug_assertions)]
static mut TCBIDX: usize = 0;

pub type ThreadControlBlock = ListItem<ThreadInfo>;

impl KObject for ListItem<ThreadInfo> {}

pub fn resume(thread: &mut ThreadControlBlock) {
    thread.resume();
    push(thread)
}

#[allow(dead_code)]
pub fn suspend(_thread: &mut ThreadControlBlock) {
    // TODO: Impl Double linked list
    // 1, check self status is Runnable.
    // 2, if true, then self.next.prev = self.prev and self.prev.next = self.next
    // (i.e take self out from runqueue)
    // then call self.suspend()
    todo!()
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
    pub root_cnode: Option<CNodeEntry<CNode>>,
    pub vspace: Option<CNodeEntry<PageTable>>,
    pub registers: Registers,
    pub ipc_buffer: Option<CNodeEntry<Page>>,
    #[cfg(debug_assertions)]
    pub tid: usize,
}

impl ThreadInfo {
    pub fn new() -> Self {
        let mut ret = Self::default();
        if cfg!(debug_assertions) {
            let tid = unsafe {
                TCBIDX += 1;
                TCBIDX
            };
            ret.tid = tid;
        }
        ret
    }
    pub fn resume(&mut self) {
        self.status = ThreadState::Runnable;
    }
    pub fn suspend(&mut self) {
        self.status = ThreadState::Blocked;
    }
    pub fn is_runnable(&self) -> bool {
        self.status == ThreadState::Runnable
    }

    pub fn set_ipc_msg(&mut self, ipc_buffer_ref: Option<&mut [u64; 512]>) {
        if let (Some(reciever_ref), Some(sender_ref)) = (self.ipc_buffer_ref(), ipc_buffer_ref) {
            unsafe { ptr::copy(sender_ref, reciever_ref, 512) }
        }
    }

    pub fn ipc_buffer_ref(&self) -> Option<&mut [u64; 512]> {
        self.ipc_buffer.as_ref().map(|page_cap_e| {
            let page_cap = page_cap_e.cap();
            let address: *mut [u64; 512] = page_cap.get_address().into();
            unsafe { &mut *{ address } }
        })
    }
    pub fn set_timeout(&mut self, time_out: usize) {
        self.time_slice = time_out
    }

    pub const fn idle_init() -> Self {
        Self {
            status: ThreadState::Idle,
            time_slice: 0,
            root_cnode: None,
            vspace: None,
            registers: Registers::null(),
            ipc_buffer: None,
            #[cfg(debug_assertions)]
            tid: 0,
        }
    }

    pub unsafe fn activate_vspace(&mut self) {
        if let Err(e) = self.activate_vspace_inner() {
            println!("{e:?}");
            PageTable::activate_kernel_table();
        }
    }

    unsafe fn activate_vspace_inner(&mut self) -> KernelResult<()> {
        let cap_entry = self
            .vspace
            .as_mut()
            .ok_or(kerr!(ErrKind::PageTableNotMappedYet))?;
        let pt_cap = cap_entry.cap_ref_mut();
        unsafe { pt_cap.activate() }
    }

    pub fn set_root_cspace(&mut self, cspace_cap: CNodeCap, parent: &mut CNodeEntry<CNode>) {
        // TODO: you should consider when already set.
        assert!(self.root_cnode.is_none(), "{:?}", self.root_cnode);
        let mut new_entry = CNodeEntry::new_with_rawcap(cspace_cap);
        new_entry.insert(parent.up_cast_ref_mut());
        self.root_cnode = Some(new_entry)
    }

    pub fn set_root_vspace(
        &mut self,
        vspace_cap: PageTableCap,
        parent: &mut CNodeEntry<PageTable>,
    ) {
        // TODO: you should consider when already set.
        assert!(self.vspace.is_none(), "{:?}", self.vspace);
        let mut new_entry = CNodeEntry::new_with_rawcap(vspace_cap);
        new_entry.insert(parent.up_cast_ref_mut());
        self.vspace = Some(new_entry)
    }

    pub fn set_ipc_buffer(&mut self, page_cap: PageCap, parent: &mut CNodeEntry<Page>) {
        // TODO: check right
        // TODO: you should consider when already set.
        assert!(self.ipc_buffer.is_none());
        let mut new_entry = CNodeEntry::new_with_rawcap(page_cap);
        new_entry.insert(parent.up_cast_ref_mut());
        self.ipc_buffer = Some(new_entry)
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
