use super::{RawCapability, CapabilityType, Capability};
use crate::object::Endpoint;
use crate::vm::KernelVAddress;

pub struct EndPointCap(RawCapability);

impl Capability for EndPointCap {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
    // TODO
    type KernelObject = Endpoint;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }

    fn init_object<'x>(&mut self) {
        let addr = KernelVAddress::from(self.0.get_address());
        let ptr = <KernelVAddress as Into<*mut Self::KernelObject>>::into(addr); 
        unsafe {
            *ptr = Self::KernelObject::new();
        }
    }
}

