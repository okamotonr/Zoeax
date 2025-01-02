use super::{RawCapability, CapabilityType, Capability};
use crate::object::Endpoint;
use crate::vm::KernelVAddress;

pub struct EndPointCap(RawCapability);

impl Capability for EndPointCap {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
    // TODO
    type KernelObject<'x> = Endpoint<'x>;
    fn new(raw_cap: RawCapability) -> Self {
        Self(raw_cap)
    }
    fn get_raw_cap(&self) -> RawCapability {
        self.0
    }
}

