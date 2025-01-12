use crate::capability::{Capability, CapabilityType, RawCapability};
use crate::object::{resume, ThreadControlBlock, ThreadInfo};
use crate::address::KernelVAddress;

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
            *ptr = ThreadControlBlock::new(ThreadInfo::default());
        }
    }
}
