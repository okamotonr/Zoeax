use crate::capability::{RawCapability, CapabilityType, Capability};
use crate::object::ThreadControlBlock;
use crate::vm::KernelVAddress;


pub struct TCBCap(RawCapability);

impl Capability for TCBCap {
    const CAP_TYPE: CapabilityType = CapabilityType::TCB;
    type KernelObject<'x> = ThreadControlBlock<'static>;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

}
