use crate::address::KernelVAddress;
use crate::capability::page_table::PageCap;
use crate::capability::{cnode::CNodeCap, page_table::PageTableCap};
use crate::common::{ErrKind, IPCBuffer, KernelResult};
use crate::kerr;
use crate::list::ListItem;
use crate::object::PageTable;
use crate::println;

use crate::scheduler::push;
use core::ops::{Index, IndexMut};
use core::ptr;

use super::cnode::CNodeEntry;
use super::page_table::Page;
use super::{CNode, CSlot, KObject};
#[cfg(debug_assertions)]
static mut TCBIDX: usize = 0;

pub type ThreadControlBlock = ListItem<ThreadInfo>;

impl KObject for ListItem<ThreadInfo> {}

#[derive(Debug, Clone, Copy)]
pub enum Register {
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,

    SCause,
    SStatus,
    SEpc,
}

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
#[derive(Debug, Default, Clone, Copy)]
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
    pub root_cnode: CSlot<CNode>,
    pub vspace: CSlot<PageTable>,
    pub registers: Registers,
    pub ipc_buffer: CSlot<Page>,
    pub badge: usize,
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

    pub fn set_ipc_msg(&mut self, ipc_buffer_ref: Option<&mut IPCBuffer>) {
        if let (Some(reciever_ref), Some(sender_ref)) = (self.ipc_buffer_ref(), ipc_buffer_ref) {
            unsafe { ptr::copy(sender_ref, reciever_ref, 1) }
        }
    }

    pub fn ipc_buffer_ref(&self) -> Option<&mut IPCBuffer> {
        self.ipc_buffer.as_ref().map(|page_cap_e| {
            let page_cap = page_cap_e.cap();
            let address: *mut IPCBuffer = KernelVAddress::from(page_cap.get_address()).into();
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
            badge: 0,
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

impl Index<Register> for Registers {
    type Output = usize;

    fn index(&self, reg: Register) -> &Self::Output {
        match reg {
            Register::Ra => &self.ra,
            Register::Sp => &self.sp,
            Register::Gp => &self.gp,
            Register::Tp => &self.tp,
            Register::T0 => &self.t0,
            Register::T1 => &self.t1,
            Register::T2 => &self.t2,
            Register::T3 => &self.t3,
            Register::T4 => &self.t4,
            Register::T5 => &self.t5,
            Register::T6 => &self.t6,
            Register::A0 => &self.a0,
            Register::A1 => &self.a1,
            Register::A2 => &self.a2,
            Register::A3 => &self.a3,
            Register::A4 => &self.a4,
            Register::A5 => &self.a5,
            Register::A6 => &self.a6,
            Register::A7 => &self.a7,
            Register::S0 => &self.s0,
            Register::S1 => &self.s1,
            Register::S2 => &self.s2,
            Register::S3 => &self.s3,
            Register::S4 => &self.s4,
            Register::S5 => &self.s5,
            Register::S6 => &self.s6,
            Register::S7 => &self.s7,
            Register::S8 => &self.s8,
            Register::S9 => &self.s9,
            Register::S10 => &self.s10,
            Register::S11 => &self.s11,
            Register::SCause => &self.scause,
            Register::SStatus => &self.sstatus,
            Register::SEpc => &self.sepc,
        }
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, reg: Register) -> &mut Self::Output {
        match reg {
            Register::Ra => &mut self.ra,
            Register::Sp => &mut self.sp,
            Register::Gp => &mut self.gp,
            Register::Tp => &mut self.tp,
            Register::T0 => &mut self.t0,
            Register::T1 => &mut self.t1,
            Register::T2 => &mut self.t2,
            Register::T3 => &mut self.t3,
            Register::T4 => &mut self.t4,
            Register::T5 => &mut self.t5,
            Register::T6 => &mut self.t6,
            Register::A0 => &mut self.a0,
            Register::A1 => &mut self.a1,
            Register::A2 => &mut self.a2,
            Register::A3 => &mut self.a3,
            Register::A4 => &mut self.a4,
            Register::A5 => &mut self.a5,
            Register::A6 => &mut self.a6,
            Register::A7 => &mut self.a7,
            Register::S0 => &mut self.s0,
            Register::S1 => &mut self.s1,
            Register::S2 => &mut self.s2,
            Register::S3 => &mut self.s3,
            Register::S4 => &mut self.s4,
            Register::S5 => &mut self.s5,
            Register::S6 => &mut self.s6,
            Register::S7 => &mut self.s7,
            Register::S8 => &mut self.s8,
            Register::S9 => &mut self.s9,
            Register::S10 => &mut self.s10,
            Register::S11 => &mut self.s11,
            Register::SCause => &mut self.scause,
            Register::SStatus => &mut self.sstatus,
            Register::SEpc => &mut self.sepc,
        }
    }
}
