use crate::address::KernelVAddress;
use crate::capability::{Capability, CapabilityType};
use crate::common::KernelResult;
use crate::object::page_table::Page;
use crate::object::{resume, CNode, CNodeEntry, PageTable, ThreadControlBlock, ThreadInfo};
use core::mem;

use super::CapabilityData;


impl TCBCap {
    pub fn set_registers(&mut self, registers: &[(usize, usize)]) {
        let tcb = self.get_tcb();
        for (r_id, val) in registers {
            tcb.registers[*r_id] = *val
        }
    }

    pub fn get_tcb(&mut self) -> &mut ThreadControlBlock {
        let addr = KernelVAddress::from(self.get_address());
        let ptr = <KernelVAddress as Into<*mut <TCBCap as Capability>::KernelObject>>::into(addr);
        unsafe { ptr.as_mut().unwrap() }
    }

    pub fn make_runnable(&mut self) {
        let tcb = self.get_tcb();
        resume(tcb)
    }

    pub fn set_cspace(&mut self, src: &mut CNodeEntry<CNode>) -> KernelResult<()> {
        let cspace_src = src.cap();
        let cspace_new = cspace_src.derive(src.up_cast_ref())?;
        self.get_tcb().set_root_cspace(cspace_new, src);
        Ok(())
    }

    pub fn set_vspace(&mut self, src: &mut CNodeEntry<PageTable>) -> KernelResult<()> {
        let vspace = src.cap();
        let vspace_new = vspace.derive(src.up_cast_ref())?;
        self.get_tcb().set_root_vspace(vspace_new, src);
        Ok(())
    }
    pub fn set_ipc_buffer(&mut self, src: &mut CNodeEntry<Page>) -> KernelResult<()> {
        let page_cap = src.cap();
        let page_cap_new = page_cap.derive(src.up_cast_ref())?;
        self.get_tcb().set_ipc_buffer(page_cap_new, src);
        Ok(())
    }
}

impl Capability for TCBCap {
    const CAP_TYPE: CapabilityType = CapabilityType::Tcb;
    type KernelObject = ThreadControlBlock;

    fn init_object(&mut self) {
        let addr = KernelVAddress::from(self.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr);
        unsafe {
            *ptr = ThreadControlBlock::new(ThreadInfo::new());
        }
    }

    fn get_object_size(_user_size: usize) -> usize {
        mem::size_of::<Self::KernelObject>()
    }
}

pub type TCBCap = CapabilityData<ThreadControlBlock>;
