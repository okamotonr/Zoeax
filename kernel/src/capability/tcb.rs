use crate::address::KernelVAddress;
use crate::capability::{Capability, CapabilityType, RawCapability};
use crate::common::KernelResult;
use crate::object::{resume, CNodeEntry, ThreadControlBlock, ThreadInfo};
use core::mem;

use super::cnode::CNodeCap;
use super::page_table::{PageCap, PageTableCap};

#[derive(Debug)]
pub struct TCBCap(RawCapability);

impl TCBCap {
    pub fn set_registers(&mut self, registers: &[(usize, usize)]) {
        let tcb = self.get_tcb();
        for (r_id, val) in registers {
            tcb.registers[*r_id] = *val
        }
    }

    pub fn get_tcb(&mut self) -> &mut ThreadControlBlock {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr = <KernelVAddress as Into<*mut <TCBCap as Capability>::KernelObject>>::into(addr);
        unsafe { ptr.as_mut().unwrap() }
    }

    pub fn make_runnable(&mut self) {
        let tcb = self.get_tcb();
        resume(tcb)
    }

    pub fn make_suspend(&mut self) {
        let tcb = self.get_tcb();
        tcb.suspend()
    }

    pub fn set_cspace(&mut self, src: &mut CNodeEntry) -> KernelResult<()> {
        let cspace_src = CNodeCap::try_from_raw(src.cap())?;
        let cspace_new = cspace_src.derive(src)?;
        self.get_tcb().set_root_cspace(cspace_new, src);
        Ok(())
    }

    pub fn set_vspace(&mut self, src: &mut CNodeEntry) -> KernelResult<()> {
        let vspace = PageTableCap::try_from_raw(src.cap())?;
        let vspace_new = vspace.derive(src)?;
        self.get_tcb().set_root_vspace(vspace_new, src);
        Ok(())
    }
    pub fn set_ipc_buffer(&mut self, src: &mut CNodeEntry) -> KernelResult<()> {
        let page_cap = PageCap::try_from_raw(src.cap())?;
        let page_cap_new = page_cap.derive(src)?;
        self.get_tcb().set_ipc_buffer(page_cap_new, src);
        Ok(())
    }
}

impl Capability for TCBCap {
    const CAP_TYPE: CapabilityType = CapabilityType::TCB;
    type KernelObject = ThreadControlBlock;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn init_object(&mut self) {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = ThreadControlBlock::new(ThreadInfo::new());
        }
    }

    fn get_object_size(_user_size: usize) -> usize {
        mem::size_of::<Self::KernelObject>()
    }
}
