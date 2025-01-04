use crate::capability::{RawCapability, CapabilityType, Capability};
use crate::object::{ThreadControlBlock, ThreadInfo};
use crate::vm::KernelVAddress;


pub struct TCBCap(RawCapability);

impl Capability for TCBCap {
    const CAP_TYPE: CapabilityType = CapabilityType::TCB;
    type KernelObject<'x> = ThreadControlBlock<'x>;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn init_object<'x>(&mut self) -> () {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject<'x>>>::into(addr); 
        unsafe {
            *ptr = ThreadControlBlock::new(ThreadInfo::default());
        }
    }
}
